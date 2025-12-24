//! Packet receiving module.
//!
//! This module handles receiving network packets using WinDivert
//! and forwarding them to the processing thread.

use crate::network::core::{
    construct_filter_with_exclusions, flush_wfp_cache, HandleConfig, HandleManager, PacketData,
};
use crate::settings::Settings;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use windivert::error::WinDivertError;

/// Receives network packets using WinDivert.
///
/// This function runs in a separate thread and continuously receives packets
/// from the network. It sends these packets to the main processing thread
/// via a channel.
///
/// # Arguments
///
/// * `packet_sender` - Channel to send received packets to the processor
/// * `running` - Atomic flag to control thread execution
/// * `_settings` - Shared packet manipulation settings (reserved for future use)
/// * `filter` - Shared filter string to determine which packets to capture
///
/// # Returns
///
/// * `Ok(())` - If thread completes cleanly
/// * `Err(WinDivertError)` - If there's an error with WinDivert operations
pub fn receive_packets(
    packet_sender: mpsc::Sender<PacketData<'_>>,
    running: Arc<AtomicBool>,
    _settings: Arc<Mutex<Settings>>,
    filter: Arc<Mutex<Option<String>>>,
) -> Result<(), WinDivertError> {
    let mut buffer = vec![0u8; 1500]; // Standard MTU size
    let mut last_filter: Option<String> = None;
    let mut handle_manager = HandleManager::new();
    let mut logged_missing_handle = false;

    while running.load(Ordering::SeqCst) {
        // Check for filter updates
        let current_filter = match filter.lock() {
            Ok(filter_guard) => construct_filter_with_exclusions(&filter_guard),
            Err(e) => {
                error!("Failed to lock filter for reading: {}", e);
                continue;
            }
        };

        // If filter changed, update WinDivert handle
        if current_filter != last_filter {
            info!(
                "Filter changed to: {}",
                current_filter.as_deref().unwrap_or("none")
            );

            match &current_filter {
                Some(filter_str) => {
                    let config = HandleConfig::with_filter(filter_str)
                        .recv_only(true)
                        .exclude_tauri_port(false); // Already excluded by construct_filter_with_exclusions

                    if let Err(e) = handle_manager.open(config) {
                        error!("Failed to open WinDivert handle: {}", e);
                    }
                }
                None => {
                    if let Err(e) = handle_manager.close() {
                        error!("Failed to close WinDivert handle: {}", e);
                    }
                }
            }

            last_filter = current_filter;
        }

        // Process packets if WinDivert handle exists
        if let Some(wd_handle) = handle_manager.handle() {
            logged_missing_handle = false;
            match wd_handle.recv(Some(&mut buffer)) {
                Ok(packet) => {
                    let packet_data = PacketData::from(packet.into_owned());
                    if packet_sender.send(packet_data).is_err() {
                        if should_shutdown(&running) {
                            break;
                        }
                        error!("Failed to send packet data to main thread");
                    }
                }
                Err(e) => {
                    error!("Failed to receive packet: {}", e);
                    if should_shutdown(&running) {
                        break;
                    }
                }
            }
        } else if !logged_missing_handle {
            debug!("WinDivert handle is not initialized. Waiting for filter.");
            logged_missing_handle = true;
        }
    }

    // HandleManager's Drop impl will clean up, but we explicitly close for logging
    if handle_manager.is_active() {
        debug!("Closing packet receiving WinDivert handle on shutdown");
        if let Err(e) = handle_manager.close() {
            error!("Failed to close WinDivert handle on shutdown: {}", e);
        }
    }

    flush_wfp_cache();
    debug!("Shutting down packet receiving thread");
    Ok(())
}

/// Checks if the thread should shut down.
fn should_shutdown(running: &Arc<AtomicBool>) -> bool {
    if !running.load(Ordering::SeqCst) {
        debug!("Packet receiving thread exiting due to shutdown signal.");
        return true;
    }
    false
}
