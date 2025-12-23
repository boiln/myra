use crate::network::core::packet_data::PacketData;
use crate::settings::packet_manipulation::PacketManipulationSettings;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use windivert::error::WinDivertError;
use windivert::layer::NetworkLayer;
use windivert::{CloseAction, WinDivert};
use windivert_sys::WinDivertFlags;

/// Constructs a WinDivert filter that excludes Tauri app ports
///
/// Takes a user-provided filter and adds conditions to exclude traffic
/// on ports used by the Tauri app to prevent disrupting the app's functionality.
///
/// # Arguments
///
/// * `user_filter` - Optional user-provided filter string
///
/// # Returns
///
/// The complete filter string with Tauri port exclusions
fn construct_filter_with_exclusions(user_filter: &Option<String>) -> Option<String> {
    if user_filter.is_none() {
        return None;
    }

    const TAURI_PORT: u16 = 1420;

    let tauri_exclusion = format!(
        "tcp.DstPort != {0} or tcp.SrcPort != {0} or udp.DstPort != {0} or udp.SrcPort != {0}",
        TAURI_PORT
    );

    Some(match user_filter {
        Some(filter) if !filter.is_empty() => {
            format!("{} and ({})", filter, tauri_exclusion)
        }
        _ => tauri_exclusion,
    })
}

/// Receives network packets using WinDivert
///
/// This function runs in a separate thread and continuously receives packets
/// from the network. It sends these packets to the main processing thread
/// via a channel.
///
/// # Arguments
///
/// * `packet_sender` - Channel to send received packets to the processor
/// * `running` - Atomic flag to control thread execution
/// * `_settings` - Shared packet manipulation settings
/// * `filter` - Shared filter string to determine which packets to capture
///
/// # Returns
///
/// * `Ok(())` - If thread completes cleanly
/// * `Err(WinDivertError)` - If there's an error with WinDivert operations
pub fn receive_packets(
    packet_sender: mpsc::Sender<PacketData<'_>>,
    running: Arc<AtomicBool>,
    _settings: Arc<Mutex<PacketManipulationSettings>>,
    filter: Arc<Mutex<Option<String>>>,
) -> Result<(), WinDivertError> {
    // Pre-allocate a buffer for packet receiving
    let mut buffer = vec![0u8; 1500]; // Standard MTU size
    let mut last_filter = None;
    let mut wd: Option<WinDivert<NetworkLayer>> = None;
    let mut logged_missing_handle = false;

    // Main processing loop
    while running.load(Ordering::SeqCst) {
        // Check for filter updates
        let current_filter = match filter.lock() {
            Ok(filter_guard) => {
                // Add Tauri port exclusions to the filter
                construct_filter_with_exclusions(&filter_guard)
            }
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

            if let Some(ref filter_str) = current_filter {
                update_windivert_handle(&mut wd, filter_str)?;
            }

            if current_filter.is_none() {
                if let Some(ref mut handle) = wd {
                    debug!("Closing WinDivert handle due to empty filter");
                    if let Err(e) = handle.close(CloseAction::Nothing) {
                        error!("Failed to close WinDivert handle: {}", e);
                    }
                }
                wd = None;
            }

            last_filter = current_filter;
        }

        // Process packets if WinDivert handle exists
        if let Some(ref wd_handle) = wd {
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
        }

        if wd.is_none() && !logged_missing_handle {
            error!("WinDivert handle is not initialized. Skipping packet reception.");
            logged_missing_handle = true;
        }
    }

    // Clean up resources
    if let Some(mut handle) = wd {
        debug!("Closing packet receiving WinDivert handle on shutdown");

        let close_result = handle.close(CloseAction::Nothing);
        if let Err(e) = &close_result {
            error!("Failed to close WinDivert handle on shutdown: {}", e);
        }

        if close_result.is_ok() {
            debug!("Successfully closed packet receiving WinDivert handle");
        }

        // Then flush the WFP cache by opening and immediately closing a new handle
        match WinDivert::<NetworkLayer>::network(
            "false", // A filter that matches nothing
            0,
            WinDivertFlags::new(),
        ) {
            Ok(mut flush_handle) => {
                let _ = flush_handle.close(CloseAction::Nothing);
                debug!("Successfully flushed WFP cache");
            }
            Err(e) => {
                error!("Failed to flush WFP cache: {}", e);
            }
        }
    }

    debug!("Shutting down packet receiving thread");
    Ok(())
}

/// Updates the WinDivert handle with a new filter
///
/// Closes any existing handle and creates a new one with the specified filter.
///
/// # Arguments
///
/// * `wd` - Mutable reference to the current WinDivert handle option
/// * `filter` - The new filter string to apply
///
/// # Returns
///
/// * `Ok(())` - If handle was updated successfully
/// * `Err(WinDivertError)` - If there was an error creating the new handle
fn update_windivert_handle(
    wd: &mut Option<WinDivert<NetworkLayer>>,
    filter: &str,
) -> Result<(), WinDivertError> {
    // Close existing handle if it exists
    if let Some(ref mut wd_handle) = wd {
        debug!("Filter changed, closing existing WinDivert handle");

        if let Err(e) = wd_handle.close(CloseAction::Nothing) {
            error!("Failed to close existing WinDivert handle: {}", e);
        }
        *wd = None;
    }

    // Flush WFP cache before creating new handle
    match WinDivert::<NetworkLayer>::network(
        "false", // A filter that matches nothing
        0,
        WinDivertFlags::new(),
    ) {
        Ok(mut flush_handle) => {
            let _ = flush_handle.close(CloseAction::Nothing);
            debug!("Successfully flushed WFP cache before creating new handle");
        }
        Err(e) => {
            error!("Failed to flush WFP cache: {}", e);
            // Continue anyway as this is just precautionary
        }
    }

    // Open a new WinDivert handle with the actual filter
    info!("Creating new WinDivert handle with filter: {}", filter);

    match WinDivert::<NetworkLayer>::network(
        filter,
        1, // High priority
        WinDivertFlags::set_recv_only(WinDivertFlags::new()),
    ) {
        Ok(handle) => {
            debug!("WinDivert handle opened with filter: {}", filter);
            *wd = Some(handle);
            Ok(())
        }
        Err(e) => {
            error!("Failed to initialize WinDivert: {}", e);
            debug!("WinDivert error detailed: {:?}", e);
            // Try one final flush of WFP cache
            if let Ok(mut h) = WinDivert::<NetworkLayer>::network("false", 0, WinDivertFlags::new())
            {
                let _ = h.close(CloseAction::Nothing);
            }
            Err(e)
        }
    }
}

/// Checks if the thread should shut down
///
/// # Arguments
///
/// * `running` - The atomic flag that controls thread execution
///
/// # Returns
///
/// * `true` - If the thread should exit
/// * `false` - If the thread should continue
fn should_shutdown(running: &Arc<AtomicBool>) -> bool {
    if !running.load(Ordering::SeqCst) {
        debug!("Packet receiving thread exiting due to shutdown signal.");
        return true;
    }

    false
}
