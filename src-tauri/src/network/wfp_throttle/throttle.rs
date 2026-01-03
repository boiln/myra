//! Precise bandwidth throttle using WinDivert with high-resolution timing
//! 
//! This creates NetLimiter-like behavior by:
//! 1. Capturing packets based on direction (excluding system-critical traffic)
//! 2. Releasing them at a controlled rate (bytes per second)
//! 3. Letting TCP control packets through immediately to maintain connection

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use log::{info, warn};
use thiserror::Error;
use windivert::prelude::*;
use windivert::packet::WinDivertPacket;
use windivert::CloseAction;

// Minimum packet size to be throttled - smaller packets pass through
// TCP ACK with no payload: 20 (IP) + 20-40 (TCP) = 40-60 bytes
// Set threshold to 52 to let pure ACKs through but throttle data packets
const MIN_PAYLOAD_THRESHOLD: usize = 52;

// Tauri devtools port to exclude
const TAURI_PORT: u16 = 1420;

#[derive(Error, Debug)]
pub enum WfpError {
    #[error("Failed to open WinDivert: {0}")]
    OpenFailed(String),
    
    #[error("Failed to start throttle thread: {0}")]
    ThreadFailed(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}

/// Shared buffer between receiver and sender threads
struct SharedBuffer {
    packets: VecDeque<WinDivertPacket<'static, NetworkLayer>>,
    total_bytes: usize,
}

impl SharedBuffer {
    fn new() -> Self {
        Self {
            packets: VecDeque::new(),
            total_bytes: 0,
        }
    }
}

/// High-precision bandwidth throttle
pub struct WfpThrottle {
    running: Arc<AtomicBool>,
    receiver_handle: Option<JoinHandle<()>>,
    sender_handle: Option<JoinHandle<()>>,
    wd_handle: Arc<Mutex<Option<WinDivert<NetworkLayer>>>>,
    limit_kbps: f64,
}

impl WfpThrottle {
    /// Create and start a new bandwidth throttle
    /// 
    /// # Arguments
    /// * `limit_kbps` - Bandwidth limit in KB/s (e.g., 0.5 = 0.5 KB/s, 10.0 = 10 KB/s)
    /// * `_process_name` - Process filter (currently not used, filters all IP traffic)
    /// * `inbound` - Throttle inbound traffic
    /// * `outbound` - Throttle outbound traffic
    pub fn new(
        limit_kbps: f64, 
        _process_name: &str,
        inbound: bool,
        outbound: bool,
    ) -> Result<Self, WfpError> {
        if limit_kbps <= 0.0 {
            return Err(WfpError::InvalidParam("limit_kbps must be > 0".into()));
        }
        if !inbound && !outbound {
            return Err(WfpError::InvalidParam("must throttle inbound or outbound".into()));
        }
        
        let running = Arc::new(AtomicBool::new(true));
        let buffer = Arc::new(Mutex::new(SharedBuffer::new()));
        
        info!("WFP Throttle: Starting {} KB/s throttle (in={}, out={})", 
              limit_kbps, inbound, outbound);
        
        // Build filter for TCP + UDP traffic, exclude Tauri port
        // Must be explicit about direction for BOTH tcp and udp parts
        // to avoid capturing wrong direction traffic
        let filter = if inbound && outbound {
            // Both directions
            format!(
                "(tcp and tcp.DstPort != {} and tcp.SrcPort != {}) or udp",
                TAURI_PORT, TAURI_PORT
            )
        } else if inbound {
            // Inbound only - explicit direction for both protocols
            format!(
                "(inbound and tcp and tcp.DstPort != {} and tcp.SrcPort != {}) or (inbound and udp)",
                TAURI_PORT, TAURI_PORT
            )
        } else {
            // Outbound only - explicit direction for both protocols
            format!(
                "(outbound and tcp and tcp.DstPort != {} and tcp.SrcPort != {}) or (outbound and udp)",
                TAURI_PORT, TAURI_PORT
            )
        };
        
        // Open WinDivert handle - use priority -1000 (higher priority than main processor at 0)
        // This ensures we intercept packets BEFORE the main processor
        let wd = WinDivert::network(&filter, -1000, WinDivertFlags::new())
            .map_err(|e| WfpError::OpenFailed(e.to_string()))?;
        
        info!("WFP Throttle: WinDivert opened with filter: {}", filter);
        
        let wd = Arc::new(Mutex::new(Some(wd)));
        
        // Spawn receiver thread
        let running_rx = running.clone();
        let buffer_rx = buffer.clone();
        let wd_rx = wd.clone();
        
        let receiver_handle = thread::Builder::new()
            .name("wfp-throttle-rx".into())
            .spawn(move || {
                run_receiver(wd_rx, buffer_rx, running_rx);
            })
            .map_err(|e| WfpError::ThreadFailed(e.to_string()))?;
        
        // Spawn sender thread
        let running_tx = running.clone();
        let buffer_tx = buffer.clone();
        let wd_tx = wd.clone();
        
        let sender_handle = thread::Builder::new()
            .name("wfp-throttle-tx".into())
            .spawn(move || {
                run_sender(wd_tx, buffer_tx, running_tx, limit_kbps);
            })
            .map_err(|e| WfpError::ThreadFailed(e.to_string()))?;
        
        Ok(Self {
            running,
            receiver_handle: Some(receiver_handle),
            sender_handle: Some(sender_handle),
            wd_handle: wd,
            limit_kbps,
        })
    }
    
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    pub fn limit_kbps(&self) -> f64 {
        self.limit_kbps
    }
    
    pub fn stop(&mut self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }
        
        info!("WFP Throttle: Stopping...");
        
        // FIRST: Close the WinDivert handle to stop capturing packets
        // This will cause recv() to fail and threads to exit
        if let Ok(mut guard) = self.wd_handle.lock() {
            if let Some(mut wd) = guard.take() {
                info!("WFP Throttle: Closing WinDivert handle");
                let _ = wd.close(CloseAction::Nothing);
            }
        }
        
        // Wait for sender thread (should exit quickly now)
        if let Some(handle) = self.sender_handle.take() {
            let _ = handle.join();
        }
        
        // Wait for receiver thread (should exit now that handle is closed)
        if let Some(handle) = self.receiver_handle.take() {
            let _ = handle.join();
        }
        
        info!("WFP Throttle: Stopped");
    }
}

impl Drop for WfpThrottle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Receiver thread: captures packets and buffers them (or passes through small ones)
fn run_receiver(
    wd: Arc<Mutex<Option<WinDivert<NetworkLayer>>>>,
    buffer: Arc<Mutex<SharedBuffer>>,
    running: Arc<AtomicBool>,
) {
    info!("WFP Throttle: Receiver thread started");
    
    // Pre-allocate receive buffer
    let mut recv_buffer = vec![0u8; 65535];
    let mut packet_count: u64 = 0;
    let mut buffered_count: u64 = 0;
    let mut passthrough_count: u64 = 0;
    let mut last_log = Instant::now();
    
    loop {
        // Check if we should stop
        if !running.load(Ordering::SeqCst) {
            break;
        }
        
        // Get handle, recv, then release lock before processing
        let recv_result = {
            let guard = match wd.lock() {
                Ok(g) => g,
                Err(_) => break,
            };
            match guard.as_ref() {
                Some(handle) => handle.recv(Some(&mut recv_buffer)),
                None => break, // Handle was closed
            }
            // Lock released here
        };
        
        match recv_result {
            Ok(packet) => {
                packet_count += 1;
                let packet_size = packet.data.len();
                
                // Small packets (TCP ACKs, SYNs, keepalives, control packets) pass through
                // This is critical to maintain connection - these are protocol overhead
                // not actual data being transferred. NetLimiter does the same.
                if packet_size <= MIN_PAYLOAD_THRESHOLD {
                    passthrough_count += 1;
                    if let Ok(guard) = wd.lock() {
                        if let Some(handle) = guard.as_ref() {
                            let _ = handle.send(&packet);
                        }
                    }
                    continue;
                }
                
                // Buffer larger packets for rate-limited release
                buffered_count += 1;
                let owned_packet = packet.into_owned();
                if let Ok(mut buf) = buffer.lock() {
                    buf.total_bytes += packet_size;
                    buf.packets.push_back(owned_packet);
                }
                
                // Log stats every 5 seconds
                if last_log.elapsed() > Duration::from_secs(5) {
                    info!("WFP Throttle RX: {} total, {} buffered, {} passthrough", 
                          packet_count, buffered_count, passthrough_count);
                    last_log = Instant::now();
                }
            }
            Err(_) => {
                // Handle closed or error - exit
                break;
            }
        }
    }
    
    info!("WFP Throttle: Receiver exiting. Total: {} packets ({} buffered, {} passthrough)", 
          packet_count, buffered_count, passthrough_count);
}

/// Sender thread: releases buffered packets at the controlled rate
fn run_sender(
    wd: Arc<Mutex<Option<WinDivert<NetworkLayer>>>>,
    buffer: Arc<Mutex<SharedBuffer>>,
    running: Arc<AtomicBool>,
    limit_kbps: f64,
) {
    info!("WFP Throttle: Sender thread started ({:.2} KB/s)", limit_kbps);
    
    // Set high timer resolution
    unsafe {
        windows::Win32::Media::timeBeginPeriod(1);
    }
    
    // Bytes per millisecond
    let bytes_per_ms = limit_kbps * 1024.0 / 1000.0;
    
    // PROPER TOKEN BUCKET ALGORITHM:
    // - Start with a burst bucket (allows initial normal traffic)
    // - Tokens replenish at bytes_per_ms rate
    // - Bucket has a max capacity (burst size)
    //
    // For NetLimiter-like behavior at 1 KB/s:
    // - Initial burst: 4 KB (4 seconds worth) - allows movement before throttle kicks in
    // - Max bucket: 2 KB (2 seconds worth) - allows recovery after idle
    //
    // The key insight: at 1 KB/s, a 1400-byte packet needs 1.4 seconds of accumulated tokens
    // So we need a larger bucket to avoid instant disconnects
    
    let burst_size = limit_kbps * 1024.0 * 4.0; // 4 seconds worth as initial burst
    let max_bucket = limit_kbps * 1024.0 * 2.0; // 2 seconds max capacity
    
    let mut bytes_credit: f64 = burst_size;
    let mut last_time = Instant::now();
    
    while running.load(Ordering::SeqCst) {
        // Check if handle is still valid
        {
            let guard = match wd.lock() {
                Ok(g) => g,
                Err(_) => break,
            };
            if guard.is_none() {
                break; // Handle was closed
            }
        }
        
        // Accumulate byte credit based on elapsed time
        let now = Instant::now();
        let elapsed_ms = now.duration_since(last_time).as_secs_f64() * 1000.0;
        bytes_credit += bytes_per_ms * elapsed_ms;
        last_time = now;
        
        // Cap credit to max bucket size - this determines burst recovery
        if bytes_credit > max_bucket {
            bytes_credit = max_bucket;
        }
        
        // Try to release packets
        let mut released = false;
        loop {
            let packet_to_send = {
                let mut buf = match buffer.lock() {
                    Ok(b) => b,
                    Err(_) => break,
                };
                
                if let Some(packet) = buf.packets.front() {
                    let size = packet.data.len() as f64;
                    if bytes_credit >= size {
                        bytes_credit -= size;
                        buf.total_bytes -= packet.data.len();
                        buf.packets.pop_front()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            
            match packet_to_send {
                Some(packet) => {
                    if let Ok(guard) = wd.lock() {
                        if let Some(handle) = guard.as_ref() {
                            let _ = handle.send(&packet);
                        }
                    }
                    released = true;
                }
                None => break,
            }
        }
        
        // Sleep a bit - shorter if we just released packets
        if released {
            thread::sleep(Duration::from_micros(100));
        } else {
            thread::sleep(Duration::from_millis(1));
        }
    }
    
    // Release remaining buffered packets before exiting
    if let Ok(mut buf) = buffer.lock() {
        let remaining = buf.packets.len();
        if remaining > 0 {
            info!("WFP Throttle: Releasing {} buffered packets", remaining);
            if let Ok(guard) = wd.lock() {
                if let Some(handle) = guard.as_ref() {
                    while let Some(packet) = buf.packets.pop_front() {
                        let _ = handle.send(&packet);
                    }
                }
            }
            buf.total_bytes = 0;
        }
    }
    
    unsafe {
        windows::Win32::Media::timeEndPeriod(1);
    }
    
    info!("WFP Throttle: Sender thread exiting");
}

unsafe impl Send for WfpThrottle {}
unsafe impl Sync for WfpThrottle {}
