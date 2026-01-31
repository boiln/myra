//! Start processing command.
//!
//! Handles the initialization and starting of the packet processing engine.

use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;

use log::{error, info};
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::network::core::set_high_precision_timer;
use crate::network::processing::{receive_packets, start_packet_processing};
use crate::settings::Settings;

/// Starts packet processing with the given settings and filter.
///
/// Creates and launches the packet receiving and processing threads
/// that will intercept and modify network packets according to the settings.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
/// * `settings` - The packet manipulation settings to apply
/// * `filter` - Optional `WinDivert` filter expression to select packets
///
/// # Returns
///
/// * `Ok(())` - If processing was started successfully
/// * `Err(String)` - If there was an error starting processing
#[tauri::command]
pub async fn start_processing(
    state: State<'_, PacketProcessingState>,
    settings: Settings,
    filter: Option<String>,
) -> Result<(), String> {
    let running = state.running.load(Ordering::SeqCst);

    if running {
        return Err("Packet processing already running".to_string());
    }

    *state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))? = settings;

    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = filter;

    let (packet_sender, packet_receiver) = mpsc::channel();

    state.running.store(true, Ordering::SeqCst);

    set_high_precision_timer();

    let running_recv = state.running.clone();
    let settings_recv = state.settings.clone();
    let filter_recv = state.filter.clone();

    thread::spawn(move || {
        if let Err(e) = receive_packets(packet_sender, running_recv, settings_recv, filter_recv) {
            error!("Packet receiving error: {}", e);
        }
    });

    let running_proc = state.running.clone();
    let settings_proc = state.settings.clone();
    let statistics = state.statistics.clone();

    thread::spawn(move || {
        if let Err(e) =
            start_packet_processing(settings_proc, packet_receiver, running_proc, statistics)
        {
            error!("Packet processing error: {}", e);
        }
    });

    info!("Started packet processing");

    Ok(())
}
