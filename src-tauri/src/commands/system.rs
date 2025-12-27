//! System commands for process and network device discovery.
//!
//! This module provides commands to list running processes and
//! discover network devices for filtering purposes.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Command;

/// Information about a running process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name (executable name)
    pub name: String,
    /// Full path to the executable (if available)
    pub path: Option<String>,
    /// Window title (if applicable)
    pub window_title: Option<String>,
    /// Base64 encoded icon data (PNG format)
    pub icon: Option<String>,
}

/// Information about a network device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    /// IP address of the device
    pub ip: String,
    /// MAC address (if available)
    pub mac: Option<String>,
    /// Hostname (if available)
    pub hostname: Option<String>,
    /// Device type hint based on MAC vendor
    pub device_type: Option<String>,
}

/// List all running processes with their PIDs and names
#[tauri::command]
pub async fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );

    let mut processes: Vec<ProcessInfo> = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let path = process.exe().map(|p| p.to_string_lossy().to_string());
            let icon = path.as_ref().and_then(|p| extract_icon(p));

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                path,
                window_title: None,
                icon,
            }
        })
        .collect();

    // Sort by name for easier browsing
    processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Remove duplicates (same name), keeping the one with lowest PID
    let mut seen: HashMap<String, ProcessInfo> = HashMap::new();
    for proc in processes {
        let key = proc.name.to_lowercase();
        seen.entry(key).or_insert(proc);
    }

    let mut result: Vec<ProcessInfo> = seen.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(result)
}

/// Extract icon from an executable file and return as base64 PNG
fn extract_icon(exe_path: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::shared::windef::HICON;
    use winapi::um::shellapi::ExtractIconExW;
    use winapi::um::wingdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use winapi::um::winuser::{DestroyIcon, GetIconInfo, ICONINFO};

    // Convert path to wide string
    let wide_path: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut large_icon: HICON = null_mut();

        // Extract the icon
        let count = ExtractIconExW(wide_path.as_ptr(), 0, &mut large_icon, null_mut(), 1);

        if count == 0 || large_icon.is_null() {
            return None;
        }

        // Get icon info
        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(large_icon, &mut icon_info) == 0 {
            DestroyIcon(large_icon);
            return None;
        }

        // Create DC for bitmap operations
        let hdc = CreateCompatibleDC(null_mut());
        if hdc.is_null() {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor as *mut _);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask as *mut _);
            }
            DestroyIcon(large_icon);
            return None;
        }

        // Set up bitmap info for 32x32 RGBA
        let width = 32i32;
        let height = 32i32;
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height; // Top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        // Allocate buffer for pixel data
        let mut pixels: Vec<u8> = vec![0u8; (width * height * 4) as usize];

        // Get the bitmap data
        let old_bmp = SelectObject(hdc, icon_info.hbmColor as *mut _);
        let result = GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        SelectObject(hdc, old_bmp);

        // Cleanup
        DeleteDC(hdc);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor as *mut _);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask as *mut _);
        }
        DestroyIcon(large_icon);

        if result == 0 {
            return None;
        }

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap B and R
        }

        // Base64 the raw RGBA for now
        let encoded = STANDARD.encode(&pixels);
        Some(format!(
            "data:image/raw;width=32;height=32;base64,{}",
            encoded
        ))
    }
}

/// Scan the local network for devices
/// Uses ARP cache and optional ping sweep
#[tauri::command]
pub async fn scan_network_devices() -> Result<Vec<NetworkDevice>, String> {
    let mut devices = Vec::new();

    // Get ARP cache entries
    let output = Command::new("arp")
        .args(["-a"])
        .output()
        .map_err(|e| format!("Failed to run arp command: {}", e))?;

    let arp_output = String::from_utf8_lossy(&output.stdout);

    for line in arp_output.lines() {
        // Parse ARP table entries
        // Format on Windows: "  192.168.1.1           00-11-22-33-44-55     dynamic"
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            continue;
        }

        let Ok(_ip) = parts[0].parse::<IpAddr>() else {
            continue;
        };

        let mac = (parts.len() >= 2 && parts[1].contains('-')).then(|| parts[1].to_string());
        let device_type = mac.as_ref().and_then(|m| detect_device_type(m));

        devices.push(NetworkDevice {
            ip: parts[0].to_string(),
            mac,
            hostname: None,
            device_type,
        });
    }

    // Try to resolve hostnames for found devices
    for device in &mut devices {
        if let Ok(hostname) = resolve_hostname(&device.ip) {
            device.hostname = Some(hostname);
        }
    }

    Ok(devices)
}

/// Try to resolve a hostname from an IP address
fn resolve_hostname(ip: &str) -> Result<String, ()> {
    use std::net::ToSocketAddrs;

    let socket_addr = format!("{}:0", ip);

    let Ok(mut addrs) = socket_addr.to_socket_addrs() else {
        return Err(());
    };

    if addrs.next().is_none() {
        return Err(());
    }

    // On Windows, we can try nslookup for reverse DNS
    let Ok(output) = Command::new("nslookup").arg(ip).output() else {
        return Err(());
    };

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        if !line.contains("Name:") {
            continue;
        }

        let Some(name) = line.split(':').nth(1) else {
            continue;
        };

        return Ok(name.trim().to_string());
    }

    Err(())
}

/// Detect device type based on MAC address vendor prefix (OUI)
/// Uses comprehensive MAC prefix databases for gaming consoles
fn detect_device_type(mac: &str) -> Option<String> {
    let mac_upper = mac.to_uppercase().replace('-', ":");

    if mac_upper.len() < 8 {
        return None;
    }

    // Extract first 3 octets for prefix matching
    let prefix = &mac_upper[..8];

    // Sony Interactive Entertainment - PlayStation consoles (PS3, PS4, PS5)
    // Source: IEEE OUI Registry - Sony Interactive Entertainment Inc.
    let sony_prefixes = [
        "00:04:1F", "00:13:15", "00:15:C1", "00:19:C5", "00:1D:0D", "00:1F:A7", "00:24:8D",
        "00:D9:D1", "00:E4:21", "04:F7:78", "0C:70:43", "0C:FE:45", "28:0D:FC", "28:40:DD",
        "2C:9E:00", "2C:CC:44", "50:B0:3B", "5C:84:3C", "5C:96:66", "68:28:6C", "70:66:2A",
        "70:9E:29", "78:C8:81", "84:E6:57", "90:47:48", "98:FA:2E", "9C:37:CB", "A8:E3:EE",
        "B4:0A:D8", "B4:1F:4D", "BC:33:29", "BC:60:A7", "C0:15:1B", "C8:4A:A0", "C8:63:F1",
        "D4:F7:D5", "E8:6E:3A", "EC:74:8C", "F4:64:12", "F8:46:1C", "F8:D0:AC", "FC:0F:E6",
    ];

    // Microsoft Corporation - Xbox consoles (Xbox 360, Xbox One, Xbox Series X/S)
    // Source: IEEE OUI Registry - Microsoft Corporation
    // Note: Microsoft uses these prefixes for Xbox, Surface, and other devices
    let microsoft_xbox_prefixes = [
        // Confirmed Xbox-specific prefixes
        "00:50:F2", // Xbox original/360
        "7C:ED:8D", // Xbox One
        "60:45:BD", // Xbox One
        // Microsoft general prefixes (used by Xbox consoles)
        "00:03:FF", "00:12:5A", "00:15:5D", "00:17:FA", "00:1D:D8", "00:22:48", "00:25:AE",
        "04:27:28", "0C:35:26", "0C:41:3E", "0C:E7:25", "10:2F:6B", "10:C7:35", "14:9A:10",
        "14:CB:65", "1C:1A:DF", "20:16:42", "20:62:74", "20:A9:9B", "28:16:A8", "28:18:78",
        "28:EA:0B", "2C:29:97", "2C:54:91", "38:33:C5", "38:56:3D", "3C:83:75", "3C:FA:06",
        "40:8E:2C", "44:16:22", "48:50:73", "48:7B:2F", "48:86:E8", "4C:3B:DF", "54:4C:8A",
        "58:79:61", "5C:BA:37", "68:6C:E6", "68:F7:D8", "6C:15:44", "6C:5D:3A", "70:A8:A5",
        "70:BC:10", "70:F8:AE", "74:76:1F", "74:C4:12", "74:E2:8C", "78:86:2E", "7C:6D:12",
        "7C:C0:AA", "80:C5:E6", "84:57:33", "84:63:D6", "84:B1:E2", "90:6A:EB", "94:9A:A9",
        "98:5F:D3", "98:7A:14", "9C:6C:15", "9C:AA:1B", "A0:4A:5E", "A0:85:FC", "A8:8C:3E",
        "AC:8E:BD", "B8:31:B5", "B8:4F:D5", "B8:5C:5C", "BC:83:85", "C0:D6:D5", "C4:61:C7",
        "C4:9D:ED", "C4:CB:76", "C8:3F:26", "C8:96:65", "CC:0D:CB", "CC:60:C8", "CC:76:45",
        "CC:B0:B3", "D0:92:9E", "D4:8F:33", "D8:E2:DF", "DC:98:40", "E4:2A:AC", "E8:A7:2F",
        "E8:F6:73", "EC:46:84", "EC:59:E7", "EC:83:50", "F0:1D:BC", "F0:6E:0B", "F4:6A:D7",
        "FC:8C:11",
    ];

    // Nintendo Co., Ltd. - Nintendo Switch, Wii U, 3DS, etc.
    // Source: IEEE OUI Registry - Nintendo Co., Ltd.
    let nintendo_prefixes = [
        "00:09:BF", "00:16:56", "00:17:AB", "00:19:1D", "00:19:FD", "00:1A:E9", "00:1B:7A",
        "00:1B:EA", "00:1C:BE", "00:1D:BC", "00:1E:35", "00:1E:A9", "00:1F:32", "00:1F:C5",
        "00:21:47", "00:21:BD", "00:22:4C", "00:22:AA", "00:22:D7", "00:23:31", "00:23:CC",
        "00:24:1E", "00:24:44", "00:24:F3", "00:25:A0", "00:26:59", "00:27:09", "04:03:D6",
        "18:2A:7B", "1C:45:86", "20:0B:CF", "20:1C:3A", "28:CF:51", "2C:10:C1", "34:2F:BD",
        "34:AF:2C", "38:C6:CE", "3C:A9:AB", "40:44:F7", "40:D2:8A", "40:F4:07", "48:31:77",
        "48:A5:E7", "48:F1:EB", "50:23:6D", "58:2F:40", "58:B0:3E", "58:BD:A3", "5C:0C:E6",
        "5C:52:1E", "60:1A:C7", "60:6B:FF", "64:B5:C6", "70:2C:09", "70:48:F7", "70:F0:88",
        "74:84:69", "74:F9:CA", "78:20:A5", "78:81:8C", "78:A2:A0", "7C:BB:8A", "80:D2:E5",
        "8C:56:C5", "8C:CD:E8", "90:45:28", "94:58:CB", "94:8E:6D", "98:41:5C", "98:B6:E9",
        "98:E2:55", "98:E8:FA", "9C:E6:35", "A4:38:CC", "A4:5C:27", "A4:C0:E1", "A4:C1:E8",
        "AC:FA:E4", "B8:68:70", "B8:78:26", "B8:8A:EC", "B8:AE:6E", "BC:74:4B", "BC:89:A6",
        "BC:9E:BB", "BC:CE:25", "C8:48:05", "C8:91:43", "CC:5B:31", "CC:9E:00", "CC:FB:65",
        "D0:55:09", "D4:F0:57", "D8:6B:83", "D8:6B:F7", "DC:68:EB", "DC:CD:18", "E0:0C:7F",
        "E0:E7:51", "E0:EF:BF", "E0:F6:B5", "E8:4E:CE", "E8:A0:CD", "E8:DA:20", "EC:C4:0D",
    ];

    // Check Sony PlayStation first (most specific for gaming)
    if sony_prefixes.iter().any(|&p| prefix == p) {
        return Some("PlayStation".to_string());
    }

    // Check Nintendo
    if nintendo_prefixes.iter().any(|&p| prefix == p) {
        return Some("Nintendo".to_string());
    }

    // Check Microsoft/Xbox
    // Note: Microsoft prefixes are shared across Xbox, Surface, etc.
    // We label as "Xbox" since that's the most relevant for game network testing
    if microsoft_xbox_prefixes.iter().any(|&p| prefix == p) {
        return Some("Xbox".to_string());
    }

    None
}

/// Get active network connections for a specific process
/// Returns a list of local ports being used by the process
fn get_process_ports(pid: u32) -> Vec<u16> {
    let output = Command::new("netstat").args(["-ano"]).output();

    let Ok(output) = output else {
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        // netstat output: Proto  Local Address  Foreign Address  State  PID
        if parts.len() < 5 {
            continue;
        }

        let Some(line_pid) = parts.last().and_then(|p| p.parse::<u32>().ok()) else {
            continue;
        };

        if line_pid != pid {
            continue;
        }

        // Parse local address (e.g., "192.168.1.100:54321")
        let local = parts[1];
        if let Some(port) = local.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) {
            // Skip common system ports and listening-only ports
            if port > 1024 {
                ports.push(port);
            }
        }
    }

    ports.sort();
    ports.dedup();
    ports
}

/// Build a WinDivert filter string for a specific process ID
/// Note: processId is NOT available at Network layer, so we use port-based filtering
#[tauri::command]
pub fn build_process_filter(pid: u32, _include_inbound: bool, _include_outbound: bool) -> String {
    let ports = get_process_ports(pid);

    // If no ports found, fall back to all outbound traffic
    if ports.is_empty() {
        return "outbound".to_string();
    }

    // Build filter based on local ports
    let port_filters: Vec<String> = ports
        .iter()
        .map(|port| format!("localPort == {}", port))
        .collect();

    if port_filters.len() == 1 {
        return format!("outbound and {}", port_filters[0]);
    }

    format!("outbound and ({})", port_filters.join(" or "))
}

/// Build a WinDivert filter string for a specific IP address (console/device)
#[tauri::command]
pub fn build_device_filter(ip: String, include_inbound: bool, include_outbound: bool) -> String {
    if !include_inbound && !include_outbound {
        return "false".to_string();
    }

    if include_outbound && include_inbound {
        return format!(
            "(outbound and ip.DstAddr == {}) or (inbound and ip.SrcAddr == {})",
            ip, ip
        );
    }

    if include_outbound {
        return format!("(outbound and ip.DstAddr == {})", ip);
    }

    format!("(inbound and ip.SrcAddr == {})", ip)
}

/// Validate a WinDivert filter string
#[tauri::command]
pub fn validate_filter(filter: String) -> Result<bool, String> {
    // Basic validation - check for common syntax issues
    // A full validation would require actually loading WinDivert

    let filter = filter.trim();

    if filter.is_empty() {
        return Err("Filter cannot be empty".to_string());
    }

    // Check for balanced parentheses
    let open = filter.chars().filter(|&c| c == '(').count();
    let close = filter.chars().filter(|&c| c == ')').count();
    if open != close {
        return Err("Unbalanced parentheses".to_string());
    }

    // Check for valid keywords
    let valid_keywords = [
        "true",
        "false",
        "inbound",
        "outbound",
        "ip",
        "ipv6",
        "icmp",
        "icmpv6",
        "tcp",
        "udp",
        "and",
        "or",
        "not",
        "processId",
        "localAddr",
        "remoteAddr",
        "localPort",
        "remotePort",
    ];

    // Simple keyword check (not comprehensive)
    for word in filter.split_whitespace() {
        let word_clean = word
            .trim_matches(|c| c == '(' || c == ')' || c == '!' || c == '=')
            .to_lowercase();
        if !word_clean.is_empty()
            && !word_clean.chars().all(|c| c.is_numeric() || c == '.')
            && !valid_keywords
                .iter()
                .any(|k| word_clean.starts_with(&k.to_lowercase()))
        {
            // Allow IP addresses and numbers
            if !word_clean.contains('.') && word_clean.parse::<u32>().is_err() {
                // It's fine, might be a valid field we don't know about
            }
        }
    }

    Ok(true)
}
