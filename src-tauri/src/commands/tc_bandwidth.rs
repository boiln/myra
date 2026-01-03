//! Commands for NetLimiter-style bandwidth limiting
//!
//! These commands control the bandwidth limiter that provides
//! true rate limiting like NetLimiter.
//! 
//! Uses WinDivert with precise timing for throttling.

use crate::network::wfp_throttle::WfpThrottle;
use log::{error, info};
use std::sync::Mutex;
use tauri::State;

/// Global state for the bandwidth limiter
pub struct TcLimiterState {
    pub throttle: Mutex<Option<WfpThrottle>>,
}

impl Default for TcLimiterState {
    fn default() -> Self {
        Self {
            throttle: Mutex::new(None),
        }
    }
}

/// Start the bandwidth limiter
/// 
/// This provides NetLimiter-style bandwidth limiting using WinDivert with precise timing.
/// Supports both inbound and outbound direction control.
#[tauri::command]
pub fn start_tc_bandwidth(
    state: State<'_, TcLimiterState>,
    limit_kbps: f64,
    direction: String,
) -> Result<String, String> {
    let mut limiter_guard = state.throttle.lock().map_err(|e| e.to_string())?;
    
    // Stop existing limiter if any
    if let Some(mut existing) = limiter_guard.take() {
        existing.stop();
    }
    
    // Parse direction
    let (inbound, outbound) = match direction.to_lowercase().as_str() {
        "inbound" | "download" | "in" => (true, false),
        "outbound" | "upload" | "out" => (false, true),
        "both" | "all" => (true, true),
        _ => (true, false), // Default to inbound for freeze effect
    };
    
    info!("Starting bandwidth limiter: {:.2} KB/s, direction: {} (in={}, out={})", 
          limit_kbps, direction, inbound, outbound);
    
    // Use empty process name to match all traffic
    // The WfpThrottle uses a simple "ip" filter
    match WfpThrottle::new(limit_kbps, "all", inbound, outbound) {
        Ok(throttle) => {
            *limiter_guard = Some(throttle);
            let dir_str = if inbound && outbound { "both" } 
                else if inbound { "inbound" } 
                else { "outbound" };
            Ok(format!("Bandwidth limiter started: {:.2} KB/s ({})", limit_kbps, dir_str))
        }
        Err(e) => {
            error!("Failed to start bandwidth limiter: {:?}", e);
            Err(format!("Failed to start: {}", e))
        }
    }
}

/// Stop the bandwidth limiter
#[tauri::command]
pub fn stop_tc_bandwidth(state: State<'_, TcLimiterState>) -> Result<String, String> {
    let mut limiter_guard = state.throttle.lock().map_err(|e| e.to_string())?;
    
    let Some(mut throttle) = limiter_guard.take() else {
        return Ok("Bandwidth limiter was not running".to_string());
    };
    
    throttle.stop();
    info!("Bandwidth limiter stopped");
    Ok("Bandwidth limiter stopped".to_string())
}

/// Get the current status of the bandwidth limiter
#[tauri::command]
pub fn get_tc_bandwidth_status(state: State<'_, TcLimiterState>) -> Result<TcBandwidthStatus, String> {
    let limiter_guard = state.throttle.lock().map_err(|e| e.to_string())?;
    
    let Some(ref throttle) = *limiter_guard else {
        return Ok(TcBandwidthStatus {
            active: false,
            limit_kbps: 0.0,
            direction: "none".to_string(),
        });
    };
    
    Ok(TcBandwidthStatus {
        active: throttle.is_running(),
        limit_kbps: throttle.limit_kbps(),
        direction: "active".to_string(),
    })
}

/// Status response for bandwidth limiter
#[derive(serde::Serialize)]
pub struct TcBandwidthStatus {
    pub active: bool,
    pub limit_kbps: f64,
    pub direction: String,
}
