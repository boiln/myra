use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use tauri::Manager;

use crate::network::modules::stats::PacketProcessingStatistics;
use crate::settings::packet_manipulation::PacketManipulationSettings;

// Global state for the packet processing system
pub struct PacketProcessingState {
    pub running: Arc<AtomicBool>,
    pub settings: Arc<Mutex<PacketManipulationSettings>>,
    pub statistics: Arc<RwLock<PacketProcessingStatistics>>,
    pub filter: Arc<Mutex<Option<String>>>,
}

impl Default for PacketProcessingState {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(Mutex::new(PacketManipulationSettings::default())),
            statistics: Arc::new(RwLock::new(PacketProcessingStatistics::default())),
            filter: Arc::new(Mutex::new(None)),
        }
    }
}

pub mod settings;
pub mod config;

use tauri::App;

pub fn register_commands(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the packet processing state
    app.manage(PacketProcessingState::default());
    
    // The commands are registered in main.rs through invoke_handler
    Ok(())
}