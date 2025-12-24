//! Stop processing command.
//!
//! Handles the shutdown of the packet processing engine.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use log::{debug, info};
use tauri::State;
use windivert::layer::NetworkLayer;
use windivert::prelude::WinDivertFlags;
use windivert::{CloseAction, WinDivert};

use crate::commands::state::PacketProcessingState;

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

/// Flushes the Windows Filtering Platform (WFP) cache.
///
/// Attempts to clear any cached state in the WFP by opening and closing
/// WinDivert handles with different priorities.
fn flush_wfp_cache() {
    for priority in [0, 1000, -1000] {
        if let Ok(mut handle) = WinDivert::<NetworkLayer>::network(
            "false", // Filter that matches nothing
            priority,
            WinDivertFlags::new(),
        ) {
            let _ = handle.close(CloseAction::Nothing);
            debug!("Successfully flushed WFP cache with priority {}", priority);
        }
    }
}
