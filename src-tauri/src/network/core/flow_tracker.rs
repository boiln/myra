//! Flow tracking for process-based filtering.
//!
//! Uses WinDivert's Flow layer to track network connections by process ID,
//! enabling reliable process-based packet filtering.

use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use windivert::layer::FlowLayer;
use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;

/// Tracked flow information
#[derive(Debug, Clone)]
pub struct FlowInfo {
    pub local_addr: IpAddr,
    pub remote_addr: IpAddr,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: u8,
}

/// Tracks active flows for a specific process
#[derive(Debug, Default)]
pub struct ProcessFlows {
    pub flows: Vec<FlowInfo>,
}

/// Flow tracker that monitors connections for target processes
pub struct FlowTracker {
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    flows: Arc<RwLock<HashMap<u32, ProcessFlows>>>,
    target_pid: Arc<RwLock<Option<u32>>>,
}

impl FlowTracker {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            flows: Arc::new(RwLock::new(HashMap::new())),
            target_pid: Arc::new(RwLock::new(None)),
        }
    }

    /// Start tracking flows for a specific process
    pub fn start(&mut self, pid: u32) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            self.stop();
        }

        *self.target_pid.write().map_err(|e| e.to_string())? = Some(pid);
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let flows = Arc::clone(&self.flows);
        let target_pid = Arc::clone(&self.target_pid);

        let handle = thread::spawn(move || {
            run_flow_tracker(running, flows, target_pid);
        });

        self.thread_handle = Some(handle);
        info!("Started flow tracker for PID {}", pid);
        Ok(())
    }

    /// Stop tracking
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        if let Ok(mut flows) = self.flows.write() {
            flows.clear();
        }
        if let Ok(mut pid) = self.target_pid.write() {
            *pid = None;
        }

        info!("Stopped flow tracker");
    }

    /// Get current flows for the target process
    pub fn get_flows(&self) -> Vec<FlowInfo> {
        let target = match self.target_pid.read() {
            Ok(guard) => *guard,
            Err(_) => return Vec::new(),
        };

        let Some(pid) = target else {
            return Vec::new();
        };

        let flows = match self.flows.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        flows.get(&pid).map(|p| p.flows.clone()).unwrap_or_default()
    }

    /// Build a WinDivert filter string for the tracked flows
    pub fn build_filter(&self) -> Option<String> {
        let flows = self.get_flows();

        if flows.is_empty() {
            return None;
        }

        let mut conditions: Vec<String> = Vec::new();

        for flow in &flows {
            let remote_ip = flow.remote_addr;
            let local_port = flow.local_port;
            let remote_port = flow.remote_port;

            // Match by remote IP and ports
            conditions.push(format!(
                "(ip.DstAddr == {} and localPort == {} and remotePort == {})",
                remote_ip, local_port, remote_port
            ));
        }

        // Also add standalone remote IPs for broader matching
        let unique_ips: Vec<IpAddr> = flows.iter().map(|f| f.remote_addr).collect();
        for ip in unique_ips.iter().take(10) {
            // Limit to prevent filter explosion
            if !ip.is_loopback() {
                conditions.push(format!("ip.DstAddr == {}", ip));
            }
        }

        conditions.dedup();

        if conditions.is_empty() {
            return None;
        }

        Some(format!("outbound and ({})", conditions.join(" or ")))
    }

    /// Check if tracker is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for FlowTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FlowTracker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_flow_tracker(
    running: Arc<AtomicBool>,
    flows: Arc<RwLock<HashMap<u32, ProcessFlows>>>,
    target_pid: Arc<RwLock<Option<u32>>>,
) {
    // Open flow layer handle - filter for all flows, we'll check PID ourselves
    let flow_handle = match WinDivert::<FlowLayer>::flow("true", 0, WinDivertFlags::new()) {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to open flow layer handle: {}", e);
            return;
        }
    };

    info!("Flow tracker started");

    while running.load(Ordering::SeqCst) {
        let packet = match flow_handle.recv(None) {
            Ok(p) => p,
            Err(e) => {
                if running.load(Ordering::SeqCst) {
                    warn!("Flow recv error: {}", e);
                }
                continue;
            }
        };

        let addr = packet.address;
        let pid = addr.process_id();

        // Check if this is our target process
        let target = match target_pid.read() {
            Ok(guard) => *guard,
            Err(_) => continue,
        };

        let Some(target_pid_value) = target else {
            continue;
        };

        if pid != target_pid_value {
            continue;
        }

        let flow_info = FlowInfo {
            local_addr: addr.local_address(),
            remote_addr: addr.remote_address(),
            local_port: addr.local_port(),
            remote_port: addr.remote_port(),
            protocol: addr.protocol(),
        };

        // Skip loopback
        if flow_info.remote_addr.is_loopback() {
            continue;
        }

        let event = addr.event();
        let mut flows_guard = match flows.write() {
            Ok(g) => g,
            Err(_) => continue,
        };

        let process_flows = flows_guard.entry(pid).or_default();

        match event {
            windivert::prelude::WinDivertEvent::FlowStablished => {
                debug!(
                    "Flow established: PID {} -> {}:{} (proto: {})",
                    pid, flow_info.remote_addr, flow_info.remote_port, flow_info.protocol
                );
                process_flows.flows.push(flow_info);
            }
            windivert::prelude::WinDivertEvent::FlowDeleted => {
                debug!(
                    "Flow deleted: PID {} -> {}:{}",
                    pid, flow_info.remote_addr, flow_info.remote_port
                );
                process_flows.flows.retain(|f| {
                    f.remote_addr != flow_info.remote_addr
                        || f.remote_port != flow_info.remote_port
                        || f.local_port != flow_info.local_port
                });
            }
            _ => {}
        }
    }

    debug!("Flow tracker thread exiting");
}
