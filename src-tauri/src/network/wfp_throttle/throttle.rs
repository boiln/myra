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
    packets: VecDeque<(WinDivertPacket<'static, NetworkLayer>, Instant)>, // (packet, queued_time)
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
        
        let filter = match (inbound, outbound) {
            (true, true) => format!(
                "(tcp and tcp.DstPort != {} and tcp.SrcPort != {}) or udp",
                TAURI_PORT, TAURI_PORT
            ),
            (true, false) => format!(
                "(inbound and tcp and tcp.DstPort != {} and tcp.SrcPort != {}) or (inbound and udp)",
                TAURI_PORT, TAURI_PORT
            ),
            (false, _) => format!(
                "(outbound and tcp and tcp.DstPort != {} and tcp.SrcPort != {}) or (outbound and udp)",
                TAURI_PORT, TAURI_PORT
            ),
        };
        
        let wd = WinDivert::network(&filter, -1000, WinDivertFlags::new())
            .map_err(|e| WfpError::OpenFailed(e.to_string()))?;
        
        info!("WFP Throttle: WinDivert opened with filter: {}", filter);
        
        let wd = Arc::new(Mutex::new(Some(wd)));
        
        let running_rx = running.clone();
        let buffer_rx = buffer.clone();
        let wd_rx = wd.clone();
        
        let receiver_handle = thread::Builder::new()
            .name("wfp-throttle-rx".into())
            .spawn(move || {
                run_receiver(wd_rx, buffer_rx, running_rx);
            })
            .map_err(|e| WfpError::ThreadFailed(e.to_string()))?;
        
        let running_tx = running.clone();
        let buffer_tx = buffer;
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
        
        if let Some(handle) = self.sender_handle.take() {
            let _ = handle.join();
        }
        
        if let Ok(mut guard) = self.wd_handle.lock() {
            if let Some(mut wd) = guard.take() {
                info!("WFP Throttle: Closing WinDivert handle");
                let _ = wd.close(CloseAction::Nothing);
            }
        }
        
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
    
    let mut recv_buffer = vec![0u8; 65535];
    let mut packet_count: u64 = 0;
    let mut buffered_count: u64 = 0;
    let mut passthrough_count: u64 = 0;
    let mut last_log = Instant::now();
    
    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        
        let recv_result = {
            let Ok(guard) = wd.lock() else { break };
            match guard.as_ref() {
                Some(handle) => handle.recv(Some(&mut recv_buffer)),
                None => break,
            }
        };
        
        match recv_result {
            Ok(packet) => {
                packet_count += 1;
                let packet_size = packet.data.len();
                
                if packet_size <= MIN_PAYLOAD_THRESHOLD {
                    passthrough_count += 1;
                    if let Ok(guard) = wd.lock() {
                        if let Some(handle) = guard.as_ref() {
                            let _ = handle.send(&packet);
                        }
                    }
                    continue;
                }
                
                buffered_count += 1;
                let owned_packet = packet.into_owned();
                if let Ok(mut buf) = buffer.lock() {
                    buf.total_bytes += packet_size;
                    buf.packets.push_back((owned_packet, Instant::now()));
                }
                
                if last_log.elapsed() > Duration::from_secs(5) {
                    info!("WFP Throttle RX: {} total, {} buffered, {} passthrough", 
                          packet_count, buffered_count, passthrough_count);
                    last_log = Instant::now();
                }
            }
            Err(_) => {
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
    
    unsafe {
        windows::Win32::Media::timeBeginPeriod(1);
    }
    
    let bytes_per_ms = limit_kbps * 1024.0 / 1000.0;
    
    let burst_size = limit_kbps * 1024.0 * 8.0;
    let max_bucket = limit_kbps * 1024.0 * 4.0;
    
    let max_packet_age = Duration::from_secs(12);
    
    let mut bytes_credit: f64 = burst_size;
    let mut last_time = Instant::now();
    
    while running.load(Ordering::SeqCst) {
        {
            let Ok(guard) = wd.lock() else { break };
            if guard.is_none() {
                break;
            }
        }
        
        let now = Instant::now();
        let elapsed_ms = now.duration_since(last_time).as_secs_f64() * 1000.0;
        bytes_credit += bytes_per_ms * elapsed_ms;
        last_time = now;
        
        if bytes_credit > max_bucket {
            bytes_credit = max_bucket;
        }
        
        let mut released = false;
        loop {
            let packet_to_send = {
                let Ok(mut buf) = buffer.lock() else { break };
                
                let Some((packet, queued_time)) = buf.packets.front() else {
                    break;
                };
                
                let size = packet.data.len() as f64;
                let packet_age = queued_time.elapsed();
                
                let force_release = packet_age >= max_packet_age;
                
                if !force_release && bytes_credit < size {
                    break;
                }
                
                if !force_release {
                    bytes_credit -= size;
                }
                
                buf.total_bytes -= packet.data.len();
                buf.packets.pop_front()
            };
            
            let Some((packet, _)) = packet_to_send else {
                break;
            };
            
            if let Ok(guard) = wd.lock() {
                if let Some(handle) = guard.as_ref() {
                    let _ = handle.send(&packet);
                }
            }
            released = true;
        }
        
        let sleep_duration = if released { Duration::from_micros(100) } else { Duration::from_millis(1) };
        thread::sleep(sleep_duration);
    }
    
    let Ok(mut buf) = buffer.lock() else {
        warn!("WFP Throttle: Could not lock buffer for flush!");
        unsafe {
            windows::Win32::Media::timeEndPeriod(1);
        }
        info!("WFP Throttle: Sender thread exiting");
        return;
    };

    let remaining = buf.packets.len();
    if remaining > 0 {
        info!("WFP Throttle: FLUSHING {} buffered packets immediately", remaining);
        let mut sent = 0;
        let mut failed = 0;
        if let Ok(guard) = wd.lock() {
            let Some(handle) = guard.as_ref() else {
                warn!("WFP Throttle: Handle already closed, {} packets LOST!", remaining);
                buf.total_bytes = 0;
                unsafe {
                    windows::Win32::Media::timeEndPeriod(1);
                }
                info!("WFP Throttle: Sender thread exiting");
                return;
            };
            while let Some((packet, _)) = buf.packets.pop_front() {
                match handle.send(&packet) {
                    Ok(_) => sent += 1,
                    Err(_) => failed += 1,
                }
            }
        }
        info!("WFP Throttle: Flushed {} packets (sent={}, failed={})", remaining, sent, failed);
        buf.total_bytes = 0;
    }
    
    unsafe {
        windows::Win32::Media::timeEndPeriod(1);
    }
    
    info!("WFP Throttle: Sender thread exiting");
}

unsafe impl Send for WfpThrottle {}
unsafe impl Sync for WfpThrottle {}
