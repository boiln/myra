use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Settings for network manipulation
///
/// Contains configurable parameters for various network condition simulations
/// like packet dropping, latency, bandwidth limitation, etc.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ManipulationSettings {
    /// Whether manipulation is currently active
    pub enabled: bool,
    /// Percentage of packets to drop (0.0-100.0)
    pub drop_rate: f32,
    /// Base latency to add to packets in milliseconds
    pub latency: u32,
    /// Random jitter to add to latency in milliseconds
    pub jitter: u32,
    /// Bandwidth limitation in KB/s (None = unlimited)
    pub bandwidth: Option<u32>,
    /// Percentage of packets to corrupt (0.0-100.0)
    pub corruption_rate: f32,
    /// Percentage of packets to reorder (0.0-100.0)
    pub reorder_rate: f32,
    /// Percentage of packets to duplicate (0.0-100.0)
    pub duplicate_rate: f32,
}

/// State for the network manipulation system
///
/// Contains the current settings and running state of the manipulation engine.
#[derive(Default)]
pub struct ManipulationState {
    /// Current manipulation settings
    pub settings: Arc<Mutex<ManipulationSettings>>,
    /// Whether manipulation is currently running
    pub running: Arc<Mutex<bool>>,
}

/// Starts the network manipulation with the provided settings
///
/// # Arguments
///
/// * `state` - Current application state
/// * `settings` - The settings to apply for manipulation
///
/// # Returns
///
/// * `Ok(())` - If manipulation started successfully
/// * `Err(String)` - If there was an error starting manipulation
#[tauri::command]
pub async fn start_manipulation(
    state: State<'_, ManipulationState>,
    settings: ManipulationSettings,
) -> Result<(), String> {
    let mut running = state.running.lock().unwrap();
    if *running {
        return Err("Manipulation already running".to_string());
    }
    
    *state.settings.lock().unwrap() = settings;
    *running = true;
    
    Ok(())
}

/// Stops the network manipulation
///
/// # Arguments
///
/// * `state` - Current application state
///
/// # Returns
///
/// * `Ok(())` - If manipulation stopped successfully
/// * `Err(String)` - If there was an error stopping manipulation
#[tauri::command]
pub async fn stop_manipulation(state: State<'_, ManipulationState>) -> Result<(), String> {
    let mut running = state.running.lock().unwrap();
    if !*running {
        return Err("Manipulation not running".to_string());
    }
    
    *running = false;
    Ok(())
}

/// Gets the current running status of the manipulation
///
/// # Arguments
///
/// * `state` - Current application state
///
/// # Returns
///
/// * `Ok(bool)` - The current running status
/// * `Err(String)` - If there was an error fetching the status
#[tauri::command]
pub async fn get_manipulation_status(state: State<'_, ManipulationState>) -> Result<bool, String> {
    Ok(*state.running.lock().unwrap())
}

/// Gets the current manipulation settings
///
/// # Arguments
///
/// * `state` - Current application state
///
/// # Returns
///
/// * `Ok(ManipulationSettings)` - The current settings
/// * `Err(String)` - If there was an error fetching the settings
#[tauri::command]
pub async fn get_manipulation_settings(state: State<'_, ManipulationState>) -> Result<ManipulationSettings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

/// Updates the manipulation settings without changing the running state
///
/// # Arguments
///
/// * `state` - Current application state
/// * `settings` - The new settings to apply
///
/// # Returns
///
/// * `Ok(())` - If settings were updated successfully
/// * `Err(String)` - If there was an error updating the settings
#[tauri::command]
pub async fn update_manipulation_settings(
    state: State<'_, ManipulationState>,
    settings: ManipulationSettings,
) -> Result<(), String> {
    *state.settings.lock().unwrap() = settings;
    Ok(())
} 