//! Application state management for packet processing.
//!
//! This module contains the global state structures used throughout
//! the packet processing system.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use tauri::{App, Manager};

use crate::network::core::FlowTracker;
use crate::network::modules::stats::PacketProcessingStatistics;
use crate::settings::Settings;

/// Global state for the packet processing system.
///
/// This struct holds all shared state needed for packet interception
/// and manipulation, including settings, statistics, and control flags.
pub struct PacketProcessingState {
    /// Flag indicating whether packet processing is currently active
    pub running: Arc<AtomicBool>,
    /// Current packet manipulation settings
    pub settings: Arc<Mutex<Settings>>,
    /// Statistics collected during packet processing
    pub statistics: Arc<RwLock<PacketProcessingStatistics>>,
    /// Current `WinDivert` filter expression
    pub filter: Arc<Mutex<Option<String>>>,
    /// Flow tracker for process-based filtering
    pub flow_tracker: Arc<Mutex<FlowTracker>>,
}

impl Default for PacketProcessingState {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(Mutex::new(Settings::default())),
            statistics: Arc::new(RwLock::new(PacketProcessingStatistics::default())),
            filter: Arc::new(Mutex::new(None)),
            flow_tracker: Arc::new(Mutex::new(FlowTracker::new())),
        }
    }
}

impl PacketProcessingState {
    /// Creates a new `PacketProcessingState` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Registers the packet processing state with the Tauri application.
///
/// This function initializes and manages the global state that will be
/// accessible to all Tauri commands.
pub fn register_state(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    use crate::commands::tc_bandwidth::TcLimiterState;
    
    app.manage(PacketProcessingState::default());
    app.manage(TcLimiterState::default());
    Ok(())
}
