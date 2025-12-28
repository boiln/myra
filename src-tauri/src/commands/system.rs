//! System commands for process and network device discovery.

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

#[tauri::command]
pub async fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
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

    let mut seen: HashMap<String, ProcessInfo> = HashMap::new();
    for proc in processes {
        let key = proc.name.to_lowercase();
        seen.entry(key).or_insert(proc);
    }

    let mut result: Vec<ProcessInfo> = seen.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(result)
}

#[tauri::command]
pub async fn scan_network_devices() -> Result<Vec<NetworkDevice>, String> {
    log::info!("Starting network device scan ..");

    let mut mac_cache = load_mac_cache();
    let mut hostname_cache = load_hostname_cache();
    let gateway_ip = get_default_gateway();

    if let Some(local_ip) = get_local_ip() {
        ping_sweep_subnet(&local_ip);
    }

    let output = Command::new("arp")
        .args(["-a"])
        .output()
        .map_err(|e| format!("Failed to run arp: {}", e))?;

    let arp_output = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    // Add this PC
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

        let hostname = match gateway_ip.as_ref() {
            Some(gw) if gw == ip_str => Some("Router / Gateway".to_string()),
            _ => None,
        };

        devices.push(NetworkDevice {
            ip: ip_str.to_string(),
            mac,
            hostname,
            device_type: None,
        });
    }

    // Apply cached hostnames
    let hostname_cache_len_before = hostname_cache.len();

    for device in devices.iter_mut() {
        if device.hostname.is_some() {
            continue;
        }

        if let Some(cached_name) = hostname_cache.get(&device.ip) {
            device.hostname = Some(cached_name.clone());
        }
    }

    // Apply cached MAC vendor names
    for device in devices.iter_mut() {
        if device.hostname.is_some() {
            continue;
        }

        let Some(mac) = &device.mac else {
            continue;
        };

        if let Some(vendor) = mac_cache.get(mac) {
            device.hostname = Some(vendor.clone());
        }
    }

    // Run parallel discovery
    let ips_needing_resolution: Vec<String> = devices
        .iter()
        .filter(|d| d.hostname.is_none())
        .map(|d| d.ip.clone())
        .collect();

    if !ips_needing_resolution.is_empty() {
        log::info!(
            "Starting parallel discovery (mDNS + SSDP + NetBIOS) for {} devices ..",
            ips_needing_resolution.len()
        );

        let ips_for_mdns = ips_needing_resolution.clone();
        let ips_for_ssdp = ips_needing_resolution.clone();
        let ips_for_netbios = ips_needing_resolution.clone();

        let mdns_handle = std::thread::spawn(move || discover_mdns_names(&ips_for_mdns));
        let ssdp_handle = std::thread::spawn(move || discover_ssdp_names(&ips_for_ssdp));
        let netbios_handle = std::thread::spawn(move || discover_netbios_names(&ips_for_netbios));

        let mdns_results = mdns_handle.join().unwrap_or_default();
        let ssdp_results = ssdp_handle.join().unwrap_or_default();
        let netbios_results = netbios_handle.join().unwrap_or_default();

        log::info!(
            "Parallel discovery complete: mDNS={}, SSDP={}, NetBIOS={}",
            mdns_results.len(),
            ssdp_results.len(),
            netbios_results.len()
        );

        // Apply results (mDNS > SSDP > NetBIOS priority)
        for device in devices.iter_mut() {
            if device.hostname.is_some() {
                continue;
            }

            let name = mdns_results
                .get(&device.ip)
                .or_else(|| ssdp_results.get(&device.ip))
                .or_else(|| netbios_results.get(&device.ip));

            let Some(name) = name else {
                continue;
            };

            device.hostname = Some(name.clone());
            hostname_cache.insert(device.ip.clone(), name.clone());
        }
    }

    if hostname_cache.len() > hostname_cache_len_before {
        save_hostname_cache(&hostname_cache);
    }

    // Fallback: MAC vendor lookup
    let macs_to_lookup: Vec<String> = devices
        .iter()
        .filter(|d| d.hostname.is_none() && d.mac.is_some())
        .filter_map(|d| d.mac.clone())
        .filter(|m| !mac_cache.contains_key(m))
        .collect();

    if !macs_to_lookup.is_empty() {
        lookup_and_update_devices(&mut devices, &mut mac_cache, &macs_to_lookup).await;
    }

    // Sort: named first, then by IP numerically
    devices.sort_by(|a, b| {
        let a_named = a.hostname.is_some();
        let b_named = b.hostname.is_some();

        if a_named != b_named {
            return b_named.cmp(&a_named);
        }

        let a_octets: Vec<u8> = a.ip.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_octets: Vec<u8> = b.ip.split('.').filter_map(|s| s.parse().ok()).collect();
        a_octets.cmp(&b_octets)
    });

    log::info!("Device scan complete, found {} devices", devices.len());
    Ok(devices)
}

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
        let mut large_icon: HICON = null_mut();
        let count = ExtractIconExW(wide_path.as_ptr(), 0, &mut large_icon, null_mut(), 1);

        if count == 0 || large_icon.is_null() {
            return None;
        }

        let mut icon_info: ICONINFO = std::mem::zeroed();

        if GetIconInfo(large_icon, &mut icon_info) == 0 {
            DestroyIcon(large_icon);
            return None;
        }

        let hdc = CreateCompatibleDC(null_mut());

        if hdc.is_null() {
            cleanup_icon_resources(&icon_info, large_icon);
            return None;
        }

        let width = 32i32;
        let height = 32i32;

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

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
// DISCOVERY FUNCTIONS
// ============================================================================

fn discover_mdns_names(ips_to_resolve: &[String]) -> HashMap<String, String> {
    use mdns_sd::{ServiceDaemon, ServiceEvent};
    use std::collections::HashSet;
    use std::time::Duration;

    let ips_set: HashSet<String> = ips_to_resolve.iter().cloned().collect();

    if ips_set.is_empty() {
        return HashMap::new();
    }

    log::info!("mDNS: Starting discovery for {} devices ..", ips_set.len());

    let mdns = match ServiceDaemon::new() {
        Ok(daemon) => daemon,
        Err(e) => {
            log::warn!("mDNS: Failed to create daemon: {}", e);
            return HashMap::new();
        }
    };

    let service_types = [
        "_services._dns-sd._udp.local.",
        "_workstation._tcp.local.",
        "_device-info._tcp.local.",
        "_http._tcp.local.",
        "_https._tcp.local.",
        "_googlecast._tcp.local.",
        "_airplay._tcp.local.",
        "_raop._tcp.local.",
        "_spotify-connect._tcp.local.",
        "_homekit._tcp.local.",
        "_hap._tcp.local.",
        "_companion-link._tcp.local.",
        "_sleep-proxy._udp.local.",
        "_smb._tcp.local.",
        "_afpovertcp._tcp.local.",
        "_printer._tcp.local.",
        "_ipp._tcp.local.",
        "_pdl-datastream._tcp.local.",
        "_scanner._tcp.local.",
        "_daap._tcp.local.",
        "_dacp._tcp.local.",
        "_touch-able._tcp.local.",
        "_appletv-v2._tcp.local.",
        "_mediaremotetv._tcp.local.",
        "_nvstream._tcp.local.",
        "_amzn-wplay._tcp.local.",
        "_amzn-alexa._tcp.local.",
        "_sonos._tcp.local.",
        "_soundtouch._tcp.local.",
        "_ewelink._tcp.local.",
        "_hue._tcp.local.",
        "_miio._tcp.local.",
        "_matter._tcp.local.",
        "_matter._udp.local.",
    ];

    let mut discovered: HashMap<String, String> = HashMap::new();

    let mut receivers = Vec::new();
    for service_type in &service_types {
        if let Ok(receiver) = mdns.browse(service_type) {
            receivers.push(receiver);
        }
    }

    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        for receiver in &receivers {
            while let Ok(event) = receiver.try_recv() {
                let ServiceEvent::ServiceResolved(info) = event else {
                    continue;
                };

                for addr in info.get_addresses() {
                    let ip_str = addr.to_string();

                    if !ips_set.contains(&ip_str) || discovered.contains_key(&ip_str) {
                        continue;
                    }

                    let name = info
                        .get_fullname()
                        .split('.')
                        .next()
                        .unwrap_or(info.get_fullname())
                        .replace('_', " ")
                        .trim()
                        .to_string();

                    if name.is_empty() || name.len() <= 1 {
                        continue;
                    }

                    log::info!("mDNS: {} -> {}", ip_str, name);
                    discovered.insert(ip_str, name);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    for service_type in &service_types {
        let _ = mdns.stop_browse(service_type);
    }

    log::info!("mDNS: Resolved {} device names", discovered.len());

    let _ = mdns.shutdown();
    discovered
}

fn discover_ssdp_names(ips_to_resolve: &[String]) -> HashMap<String, String> {
    use std::collections::HashSet;
    use std::net::{SocketAddr, UdpSocket};
    use std::time::Duration;

    let ips_set: HashSet<String> = ips_to_resolve.iter().cloned().collect();

    if ips_set.is_empty() {
        return HashMap::new();
    }

    log::info!("SSDP: Starting discovery for {} devices ..", ips_set.len());

    let ssdp_request = b"M-SEARCH * HTTP/1.1\r\n\
        HOST: 239.255.255.250:1900\r\n\
        MAN: \"ssdp:discover\"\r\n\
        MX: 3\r\n\
        ST: ssdp:all\r\n\
        \r\n";

    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        log::warn!("SSDP: Failed to bind UDP socket");
        return HashMap::new();
    };

    let _ = socket.set_read_timeout(Some(Duration::from_millis(100)));
    let _ = socket.set_broadcast(true);

    let ssdp_addr: SocketAddr = "239.255.255.250:1900".parse().unwrap();
    for _ in 0..3 {
        let _ = socket.send_to(ssdp_request, ssdp_addr);
        std::thread::sleep(Duration::from_millis(100));
    }

    let mut discovered: HashMap<String, String> = HashMap::new();
    let mut locations: HashMap<String, String> = HashMap::new();

    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    let mut buf = [0u8; 4096];

    while start.elapsed() < timeout {
        let Ok((len, addr)) = socket.recv_from(&mut buf) else {
            continue;
        };

        let ip = addr.ip().to_string();

        if !ips_set.contains(&ip) || discovered.contains_key(&ip) {
            continue;
        }

        let response = String::from_utf8_lossy(&buf[..len]);

        let mut server_name: Option<String> = None;
        let mut usn_name: Option<String> = None;
        let mut location_url: Option<String> = None;

        for line in response.lines() {
            let line_lower = line.to_lowercase();

            if line_lower.starts_with("location:") {
                if let Some(url) = line.split_once(':').map(|(_, v)| v.trim()) {
                    if url.contains(&format!("{}:", ip)) || url.contains(&format!("{}/", ip)) {
                        location_url = Some(url.to_string());
                    }
                }
            }

            if line_lower.starts_with("server:") {
                if let Some(server) = line.split_once(':').map(|(_, v)| v.trim()) {
                    server_name = extract_ssdp_server_name(server);
                }
            }

            if line_lower.starts_with("usn:") {
                if let Some(usn) = line.split_once(':').map(|(_, v)| v.trim()) {
                    usn_name = extract_ssdp_usn_name(usn);
                }
            }
        }

        let device_name = usn_name.or(server_name);

        if let Some(name) = device_name {
            log::info!("SSDP: {} -> {}", ip, name);
            discovered.insert(ip, name);
            continue;
        }

        if let Some(url) = location_url {
            locations.insert(ip, url);
        }
    }

    for (ip, url) in &locations {
        if discovered.contains_key(ip) {
            continue;
        }

        let Some(name) = fetch_upnp_friendly_name(url) else {
            continue;
        };

        log::info!("SSDP XML: {} -> {}", ip, name);
        discovered.insert(ip.clone(), name);
    }

    log::info!("SSDP: Resolved {} device names", discovered.len());
    discovered
}

fn discover_netbios_names(ips_to_resolve: &[String]) -> HashMap<String, String> {
    use std::collections::HashSet;
    use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
    use std::time::Duration;

    let ips_set: HashSet<String> = ips_to_resolve.iter().cloned().collect();

    if ips_set.is_empty() {
        return HashMap::new();
    }

    log::info!(
        "NetBIOS: Starting discovery for {} devices ..",
        ips_set.len()
    );

    let netbios_query: [u8; 49] = [
        0x00, 0x01, 0x00, 0x10, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x43, 0x4b,
        0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x00,
        0x00, 0x21, 0x00, 0x01,
    ];

    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        log::warn!("NetBIOS: Failed to bind UDP socket");
        return HashMap::new();
    };

    let _ = socket.set_read_timeout(Some(Duration::from_millis(50)));

    for ip in ips_to_resolve {
        let Ok(addr) = ip.parse::<Ipv4Addr>() else {
            continue;
        };

        let target = SocketAddr::new(addr.into(), 137);
        let _ = socket.send_to(&netbios_query, target);
    }

    let mut discovered: HashMap<String, String> = HashMap::new();
    let timeout = Duration::from_secs(3);
    let start = std::time::Instant::now();
    let mut buf = [0u8; 1024];

    while start.elapsed() < timeout {
        let Ok((len, addr)) = socket.recv_from(&mut buf) else {
            continue;
        };

        let ip = addr.ip().to_string();

        if !ips_set.contains(&ip) || discovered.contains_key(&ip) {
            continue;
        }

        let Some(name) = parse_netbios_response(&buf[..len]) else {
            continue;
        };

        log::info!("NetBIOS: {} -> {}", ip, name);
        discovered.insert(ip, name);
    }

    log::info!("NetBIOS: Resolved {} device names", discovered.len());
    discovered
}

// ============================================================================
// PROTOCOL PARSERS
// ============================================================================

fn parse_netbios_response(data: &[u8]) -> Option<String> {
    if data.len() < 57 {
        return None;
    }

    let mut pos = 12;

    while pos < data.len() && data[pos] != 0x00 {
        pos += 1;
    }

    pos += 1;
    pos += 4;
    pos += 12;

    if pos >= data.len() {
        return None;
    }

    let num_names = data[pos] as usize;
    pos += 1;

    for _ in 0..num_names.min(10) {
        if pos + 18 > data.len() {
            break;
        }

        let name_bytes = &data[pos..pos + 15];
        let suffix = data[pos + 15];

        if suffix != 0x00 && suffix != 0x20 {
            pos += 18;
            continue;
        }

        let name = String::from_utf8_lossy(name_bytes)
            .trim()
            .trim_end_matches(char::from(0))
            .to_string();

        if name.is_empty() || name.len() <= 1 {
            pos += 18;
            continue;
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            pos += 18;
            continue;
        }

        return Some(name);
    }

    None
}

fn extract_ssdp_server_name(server: &str) -> Option<String> {
    let s = server.to_lowercase();

    if s.contains("directv") {
        return Some("DIRECTV".to_string());
    }

    if s.contains("jetheadinc") {
        return Some("Cable Box".to_string());
    }

    if s.contains("roku") {
        return Some("Roku".to_string());
    }

    if s.contains("xbox") {
        return Some("Xbox".to_string());
    }

    if s.contains("playstation") || s.contains("ps4") || s.contains("ps5") {
        return Some("PlayStation".to_string());
    }

    if s.contains("nintendo") {
        return Some("Nintendo Switch".to_string());
    }

    if s.contains("samsung") {
        return Some("Samsung TV".to_string());
    }

    if s.contains("lg") && (s.contains("tv") || s.contains("webos")) {
        return Some("LG TV".to_string());
    }

    if s.contains("ht-a") || s.contains("ht-s") || s.contains("ht-x") {
        return Some("Sony Soundbar".to_string());
    }

    if s.contains("sony") && s.contains("bravia") {
        return Some("Sony TV".to_string());
    }

    if s.contains("plex") {
        return Some("Plex Server".to_string());
    }

    if s.contains("synology") {
        return Some("Synology NAS".to_string());
    }

    if s.contains("qnap") {
        return Some("QNAP NAS".to_string());
    }

    None
}

fn extract_ssdp_usn_name(usn: &str) -> Option<String> {
    let u = usn.to_lowercase();

    if u.contains("directv") {
        return Some("DIRECTV".to_string());
    }

    if u.contains("mediarenderer") && u.contains("46_34_a7") {
        return Some("Xfinity Cable Box".to_string());
    }

    if u.contains("manageabledevice") && u.contains("46_34_a7") {
        return Some("Xfinity Cable Box".to_string());
    }

    None
}

fn fetch_upnp_friendly_name(url: &str) -> Option<String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let url = url.trim_start_matches("http://");
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));

    let stream =
        TcpStream::connect_timeout(&host_port.parse().ok()?, Duration::from_millis(500)).ok()?;

    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_millis(500)))
        .ok()?;

    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host_port
    );

    let mut stream = stream;
    stream.write_all(request.as_bytes()).ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;

    let start = response.find("<friendlyName>")?;
    let start = start + "<friendlyName>".len();
    let end = response[start..].find("</friendlyName>")?;

    let name = response[start..start + end].trim();
    let name = name.trim_start_matches("[TV] ");

    if name.is_empty() || name.len() <= 1 {
        return None;
    }

    Some(name.to_string())
}

// ============================================================================
// NETWORK HELPERS
// ============================================================================

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

fn ping_sweep_subnet(local_ip: &str) {
    use std::thread;

    let parts: Vec<&str> = local_ip.split('.').collect();

    if parts.len() != 4 {
        log::warn!("Invalid local IP format: {}", local_ip);
        return;
    }

    let subnet_prefix = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
    log::info!("Starting ping sweep on subnet {}.1-254 ..", subnet_prefix);

    let batch_size = 50;
    let mut handles = Vec::new();

    for batch_start in (1..=254).step_by(batch_size) {
        let batch_end = (batch_start + batch_size - 1).min(254);
        let prefix = subnet_prefix.clone();

        let handle = thread::spawn(move || {
            for i in batch_start..=batch_end {
                let ip = format!("{}.{}", prefix, i);
                let _ = Command::new("ping")
                    .args(["-n", "1", "-w", "50", &ip])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    thread::sleep(std::time::Duration::from_millis(500));
    log::info!("Ping sweep complete");
}

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

fn is_broadcast_or_multicast(ip: &IpAddr) -> bool {
    let IpAddr::V4(ipv4) = ip else {
        return false;
    };

    let octets = ipv4.octets();

    if octets[3] == 255 {
        return true;
    }

    if octets[0] >= 224 && octets[0] <= 239 {
        return true;
    }

    false
}

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
// MAC VENDOR LOOKUP
// ============================================================================

async fn lookup_and_update_devices(
    devices: &mut Vec<NetworkDevice>,
    mac_cache: &mut HashMap<String, String>,
    macs_to_lookup: &[String],
) {
    log::info!("Looking up {} new MAC addresses ..", macs_to_lookup.len());

    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    else {
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

// ============================================================================
// CACHE MANAGEMENT
// ============================================================================

fn get_mac_cache_path() -> std::path::PathBuf {
    let Ok(exe_path) = std::env::current_exe() else {
        return std::path::PathBuf::from("devices.json");
    };

    let Some(dir) = exe_path.parent() else {
        return std::path::PathBuf::from("devices.json");
    };

    dir.join("devices.json")
}

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

fn get_hostname_cache_path() -> std::path::PathBuf {
    let Ok(exe_path) = std::env::current_exe() else {
        return std::path::PathBuf::from("hostname_cache.json");
    };

    let Some(dir) = exe_path.parent() else {
        return std::path::PathBuf::from("hostname_cache.json");
    };

    dir.join("hostname_cache.json")
}

fn load_hostname_cache() -> HashMap<String, String> {
    let path = get_hostname_cache_path();

    let Ok(contents) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };

    let Ok(cache): Result<HashMap<String, String>, _> = serde_json::from_str(&contents) else {
        return HashMap::new();
    };

    log::info!(
        "Loaded hostname cache from {:?} ({} entries)",
        path,
        cache.len()
    );
    cache
}

fn save_hostname_cache(cache: &HashMap<String, String>) {
    let path = get_hostname_cache_path();

    let Ok(json) = serde_json::to_string_pretty(cache) else {
        return;
    };

    if let Err(e) = std::fs::write(&path, json) {
        log::warn!("Failed to save hostname cache: {}", e);
        return;
    }

    log::info!(
        "Saved hostname cache to {:?} ({} entries)",
        path,
        cache.len()
    );
}
