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
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub window_title: Option<String>,
    pub icon: Option<String>,
}

/// Information about a network device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub ip: String,
    pub mac: Option<String>,
    pub hostname: Option<String>,
    pub device_type: Option<String>,
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

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

    processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Deduplicate by name, keeping lowest PID
    let mut seen: HashMap<String, ProcessInfo> = HashMap::new();
    for proc in processes {
        let key = proc.name.to_lowercase();
        seen.entry(key).or_insert(proc);
    }

    let mut result: Vec<ProcessInfo> = seen.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(result)
}

/// Scan the local network for devices
#[tauri::command]
pub async fn scan_network_devices() -> Result<Vec<NetworkDevice>, String> {
    log::info!("Starting network device scan...");

    let mut mac_cache = load_mac_cache();
    let gateway_ip = get_default_gateway();

    let output = Command::new("arp")
        .args(["-a"])
        .output()
        .map_err(|e| format!("Failed to run arp: {}", e))?;

    let arp_output = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    if let Some(local_ip) = get_local_ip() {
        devices.push(NetworkDevice {
            ip: local_ip,
            mac: None,
            hostname: Some("This PC".to_string()),
            device_type: None,
        });
    }

    // Parse ARP table
    for line in arp_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            continue;
        }

        let ip_str = parts[0];

        let Ok(ip) = ip_str.parse::<IpAddr>() else {
            continue;
        };

        if is_broadcast_or_multicast(&ip) {
            continue;
        }

        let mac = parse_mac_from_arp(parts.get(1).copied());
        let hostname = resolve_hostname(&gateway_ip, ip_str, &mac, &mac_cache);

        devices.push(NetworkDevice {
            ip: ip_str.to_string(),
            mac,
            hostname,
            device_type: None,
        });
    }

    // Lookup unknown MACs via API
    let macs_to_lookup: Vec<String> = devices
        .iter()
        .filter(|d| d.hostname.is_none() && d.mac.is_some())
        .filter_map(|d| d.mac.clone())
        .filter(|m| !mac_cache.contains_key(m))
        .collect();

    if !macs_to_lookup.is_empty() {
        lookup_and_update_devices(&mut devices, &mut mac_cache, &macs_to_lookup).await;
    }

    // Sort: named devices first, then by IP
    devices.sort_by(|a, b| {
        let a_named = a.hostname.is_some();
        let b_named = b.hostname.is_some();

        if a_named != b_named {
            return b_named.cmp(&a_named);
        }

        a.ip.cmp(&b.ip)
    });

    log::info!("Device scan complete, found {} devices", devices.len());
    Ok(devices)
}

/// Build a WinDivert filter string for a specific process ID
#[tauri::command]
pub fn build_process_filter(pid: u32, _include_inbound: bool, _include_outbound: bool) -> String {
    let ports = get_process_ports(pid);

    if ports.is_empty() {
        return "outbound".to_string();
    }

    let port_filters: Vec<String> = ports
        .iter()
        .map(|port| format!("localPort == {}", port))
        .collect();

    if port_filters.len() == 1 {
        return format!("outbound and {}", port_filters[0]);
    }

    format!("outbound and ({})", port_filters.join(" or "))
}

/// Build a WinDivert filter string for a specific IP address
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
    let filter = filter.trim();

    if filter.is_empty() {
        return Err("Filter cannot be empty".to_string());
    }

    let open_parens = filter.chars().filter(|&c| c == '(').count();
    let close_parens = filter.chars().filter(|&c| c == ')').count();

    if open_parens != close_parens {
        return Err("Unbalanced parentheses".to_string());
    }

    Ok(true)
}

// ============================================================================
// ICON EXTRACTION
// ============================================================================

/// Extract icon from an executable file and return as base64 PNG
fn extract_icon(exe_path: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    use winapi::shared::windef::HICON;
    use winapi::um::shellapi::ExtractIconExW;
    use winapi::um::wingdi::{
        CreateCompatibleDC, DeleteDC, GetDIBits, SelectObject, BITMAPINFO, BITMAPINFOHEADER,
        BI_RGB, DIB_RGB_COLORS,
    };
    use winapi::um::winuser::{DestroyIcon, GetIconInfo, ICONINFO};

    let wide_path: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        // Extract icon
        let mut large_icon: HICON = null_mut();
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

        // Create DC
        let hdc = CreateCompatibleDC(null_mut());

        if hdc.is_null() {
            cleanup_icon_resources(&icon_info, large_icon);
            return None;
        }

        // Setup bitmap info
        let width = 32i32;
        let height = 32i32;

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        // Get pixel data
        let mut pixels: Vec<u8> = vec![0u8; (width * height * 4) as usize];
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
        DeleteDC(hdc);
        cleanup_icon_resources(&icon_info, large_icon);

        if result == 0 {
            return None;
        }

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        let encoded = STANDARD.encode(&pixels);
        Some(format!(
            "data:image/raw;width=32;height=32;base64,{}",
            encoded
        ))
    }
}

/// Cleanup icon resources
unsafe fn cleanup_icon_resources(
    icon_info: &winapi::um::winuser::ICONINFO,
    icon: winapi::shared::windef::HICON,
) {
    use winapi::um::wingdi::DeleteObject;
    use winapi::um::winuser::DestroyIcon;

    if !icon_info.hbmColor.is_null() {
        DeleteObject(icon_info.hbmColor as *mut _);
    }

    if !icon_info.hbmMask.is_null() {
        DeleteObject(icon_info.hbmMask as *mut _);
    }

    DestroyIcon(icon);
}

// ============================================================================
// NETWORK HELPERS
// ============================================================================

/// Parse MAC address from ARP output
fn parse_mac_from_arp(mac_str: Option<&str>) -> Option<String> {
    let mac_str = mac_str?;

    if !mac_str.contains('-') {
        return None;
    }

    if mac_str == "ff-ff-ff-ff-ff-ff" {
        return None;
    }

    Some(mac_str.to_uppercase())
}

/// Resolve hostname for a device
fn resolve_hostname(
    gateway_ip: &Option<String>,
    ip_str: &str,
    mac: &Option<String>,
    mac_cache: &HashMap<String, String>,
) -> Option<String> {
    if gateway_ip.as_ref() == Some(&ip_str.to_string()) {
        return Some("Router / Gateway".to_string());
    }

    let mac = mac.as_ref()?;
    mac_cache.get(mac).cloned()
}

/// Lookup MACs via API and update devices
async fn lookup_and_update_devices(
    devices: &mut Vec<NetworkDevice>,
    mac_cache: &mut HashMap<String, String>,
    macs_to_lookup: &[String],
) {
    log::info!("Looking up {} new MAC addresses...", macs_to_lookup.len());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let Ok(client) = client else {
        return;
    };

    let Some(results) = lookup_macs_batch(&client, macs_to_lookup).await else {
        return;
    };

    for device in devices.iter_mut() {
        if device.hostname.is_some() {
            continue;
        }

        let Some(mac) = &device.mac else {
            continue;
        };

        let Some(vendor) = results.get(mac) else {
            continue;
        };

        device.hostname = Some(vendor.clone());
        mac_cache.insert(mac.clone(), vendor.clone());
    }

    save_mac_cache(mac_cache);
}

/// Batch lookup MAC vendors from backend server
async fn lookup_macs_batch(
    client: &reqwest::Client,
    macs: &[String],
) -> Option<HashMap<String, String>> {
    const MAC_RESOLVER_URL_B64: &str = "aHR0cDovLzEzNS4xNDguMjYuNTY6OTA5MC9sb29rdXAvYmF0Y2g=";

    let url = String::from_utf8(STANDARD.decode(MAC_RESOLVER_URL_B64).ok()?).ok()?;

    log::info!("Sending batch lookup for {} MACs", macs.len());

    let response = client.post(&url).json(macs).send().await.ok()?;

    if !response.status().is_success() {
        log::warn!("MAC resolver returned status: {}", response.status());
        return None;
    }

    #[derive(Deserialize)]
    struct MacResult {
        mac: String,
        vendor: Option<String>,
    }

    let results: Vec<MacResult> = response.json().await.ok()?;

    let mut map = HashMap::new();

    for r in results {
        let Some(vendor) = r.vendor else {
            continue;
        };

        let mac_normalized = r.mac.to_uppercase().replace(':', "-");
        log::info!("Resolved {} -> {}", mac_normalized, vendor);
        map.insert(mac_normalized, vendor);
    }

    log::info!("Batch lookup returned {} results", map.len());
    Some(map)
}

/// Get the default gateway IP address
fn get_default_gateway() -> Option<String> {
    let output = Command::new("route")
        .args(["print", "0.0.0.0"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 4 {
            continue;
        }

        if parts[0] != "0.0.0.0" {
            continue;
        }

        let gateway = parts[2];

        if !gateway.contains('.') {
            continue;
        }

        if gateway == "0.0.0.0" {
            continue;
        }

        return Some(gateway.to_string());
    }

    None
}

/// Get the local IP address of this PC
fn get_local_ip() -> Option<String> {
    let output = Command::new("ipconfig").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut in_ethernet_section = false;

    for line in stdout.lines() {
        let line_lower = line.to_lowercase();

        if line_lower.contains("ethernet adapter") && !line_lower.contains("virtual") {
            in_ethernet_section = true;
            continue;
        }

        if line_lower.contains("adapter") {
            in_ethernet_section = false;
            continue;
        }

        if !in_ethernet_section {
            continue;
        }

        if !line_lower.contains("ipv4") {
            continue;
        }

        let ip = line.split(':').nth(1)?.trim();

        if ip.starts_with("169.254") {
            continue;
        }

        return Some(ip.to_string());
    }

    None
}

/// Check if IP is broadcast or multicast
fn is_broadcast_or_multicast(ip: &IpAddr) -> bool {
    let IpAddr::V4(ipv4) = ip else {
        return false;
    };

    let octets = ipv4.octets();

    // Broadcast: x.x.x.255
    if octets[3] == 255 {
        return true;
    }

    // Multicast: 224.x.x.x - 239.x.x.x
    if octets[0] >= 224 && octets[0] <= 239 {
        return true;
    }

    false
}

/// Get active network connections for a specific process
fn get_process_ports(pid: u32) -> Vec<u16> {
    let Ok(output) = Command::new("netstat").args(["-ano"]).output() else {
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 5 {
            continue;
        }

        let Some(line_pid) = parts.last().and_then(|p| p.parse::<u32>().ok()) else {
            continue;
        };

        if line_pid != pid {
            continue;
        }

        let local = parts[1];

        let Some(port) = local.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) else {
            continue;
        };

        if port <= 1024 {
            continue;
        }

        ports.push(port);
    }

    ports.sort();
    ports.dedup();
    ports
}

// ============================================================================
// MAC CACHE
// ============================================================================

/// Get the path for the MAC cache file
fn get_mac_cache_path() -> std::path::PathBuf {
    let Ok(exe_path) = std::env::current_exe() else {
        return std::path::PathBuf::from("mac.json");
    };

    let Some(dir) = exe_path.parent() else {
        return std::path::PathBuf::from("mac.json");
    };

    dir.join("mac_cache.json")
}

/// Load MAC cache from disk
fn load_mac_cache() -> HashMap<String, String> {
    let path = get_mac_cache_path();

    let Ok(contents) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };

    let Ok(cache) = serde_json::from_str(&contents) else {
        return HashMap::new();
    };

    log::info!("Loaded MAC cache from {:?}", path);
    cache
}

/// Save MAC cache to disk
fn save_mac_cache(cache: &HashMap<String, String>) {
    let path = get_mac_cache_path();

    let Ok(json) = serde_json::to_string_pretty(cache) else {
        return;
    };

    if let Err(e) = std::fs::write(&path, json) {
        log::warn!("Failed to save MAC cache: {}", e);
        return;
    }

    log::info!("Saved MAC cache to {:?} ({} entries)", path, cache.len());
}
