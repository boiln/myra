//! Stop processing command.
//!
//! Handles the shutdown of the packet processing engine.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use log::info;
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::network::core::flush_wfp_cache;

/// Stops packet processing.
///
/// Signals the packet processing and receiving threads to shut down
/// and waits a short time for them to clean up resources.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
///
/// # Returns
///
/// * `Ok(())` - If processing was stopped successfully
/// * `Err(String)` - If there was an error stopping processing
#[tauri::command]
pub async fn stop_processing(state: State<'_, PacketProcessingState>) -> Result<(), String> {
    if !state.running.load(Ordering::SeqCst) {
        return Err("Packet processing not running".to_string());
    }

    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = None;

    thread::sleep(Duration::from_millis(100));

    state.running.store(false, Ordering::SeqCst);

    thread::sleep(Duration::from_millis(500));

    flush_wfp_cache();

    info!("Stopped packet processing and cleaned up resources");
    Ok(())
}
