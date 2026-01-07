//! Stop processing command.
//!
//! Handles the shutdown of the packet processing engine.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use log::info;
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::network::core::{flush_wfp_cache, restore_timer_resolution};
use crate::settings::Settings;

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

    // Temporarily disable modules to trigger proper buffer flushes (like burst)
    // This ensures packets are released through the normal path before shutdown
    // We preserve the original settings and restore them after flushing
    let original_settings: Settings;
    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings mutex: {}", e))?;
        original_settings = settings.clone();
        
        // Reset to default (all modules disabled) for flushing
        *settings = Settings::default();
    }
    
    // Give time for the processing loop to flush buffers
    thread::sleep(Duration::from_millis(300));
    
    // Restore the original settings (so frontend sees them correctly)
    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings mutex: {}", e))?;
        *settings = original_settings;
    }

    // Preserve the current filter across stop/start cycles.
    // We no longer clear the filter here so the frontend retains
    // the user's selection or loaded config filter when restarting.

    thread::sleep(Duration::from_millis(100));

    state.running.store(false, Ordering::SeqCst);

    thread::sleep(Duration::from_millis(500));

    flush_wfp_cache();

    // Restore timer resolution
    restore_timer_resolution();

    info!("Stopped packet processing and cleaned up resources");
    Ok(())
}
