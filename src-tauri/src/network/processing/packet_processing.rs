use crate::network::core::packet_data::PacketData;
use crate::network::modules::bandwidth::bandwidth_limiter;
use crate::network::modules::delay::delay_packets;
use crate::network::modules::drop::drop_packets;
use crate::network::modules::duplicate::duplicate_packets;
use crate::network::modules::reorder::reorder_packets;
use crate::network::modules::stats::PacketProcessingStatistics;
use crate::network::modules::tamper::tamper_packets;
use crate::network::modules::throttle::throttle_packages;
use crate::network::processing::packet_processing_state::PacketProcessingState;
use crate::settings::packet_manipulation::PacketManipulationSettings;
use crate::utils::is_effect_active;
use crate::utils::log_statistics;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use windivert::error::WinDivertError;
use windivert::layer::NetworkLayer;
use windivert::{CloseAction, WinDivert};
use windivert_sys::WinDivertFlags;

/// Starts the packet processing loop that handles network packet manipulation.
///
/// This function creates a WinDivert handle configured for sending packets only,
/// then enters a processing loop where it:
/// 1. Receives packets from the provided channel
/// 2. Applies various packet manipulations based on settings
/// 3. Sends the processed packets back to the network
///
/// The function continues running until the `running` flag is set to false.
///
/// # Arguments
///
/// * `settings` - Shared settings that control packet manipulation behavior
/// * `packet_receiver` - Channel receiver for incoming packet data
/// * `running` - Atomic flag that controls when processing should stop
/// * `statistics` - Shared statistics tracking various packet manipulations
///
/// # Returns
///
/// Result indicating success or a WinDivertError if something fails
pub fn start_packet_processing(
    settings: Arc<Mutex<PacketManipulationSettings>>,
    packet_receiver: Receiver<PacketData>,
    running: Arc<AtomicBool>,
    statistics: Arc<RwLock<PacketProcessingStatistics>>,
) -> Result<(), WinDivertError> {
    // Initialize WinDivert for sending packets only
    let mut wd = WinDivert::<NetworkLayer>::network(
        "false",
        0,
        WinDivertFlags::set_send_only(WinDivertFlags::new()),
    )
    .map_err(|e| {
        error!("Failed to initialize WinDivert: {}", e);
        error!("WinDivert error detailed: {:?}", e);
        e
    })?;

    let log_interval = Duration::from_secs(2);
    let mut last_log_time = Instant::now();

    let mut received_packet_count = 0;
    let mut sent_packet_count = 0;

    // Initialize packet processing state
    let mut state = PacketProcessingState::new();

    info!("Starting packet interception.");

    // Main processing loop
    while running.load(Ordering::SeqCst) {
        let mut packets = Vec::new();

        // Collect all available packets from the channel
        while let Ok(packet_data) = packet_receiver.try_recv() {
            packets.push(packet_data);
            received_packet_count += 1;
        }

        // Apply packet manipulations according to current settings
        if let Ok(settings) = settings.lock() {
            process_packets(&settings, &mut packets, &mut state, &statistics);
        } else {
            // If we can't access settings, log the error but continue processing
            error!("Failed to acquire lock on packet manipulation settings");
        }

        // Send the processed packets
        for packet_data in &packets {
            if let Err(e) = wd.send(&packet_data.packet) {
                error!("Failed to send packet: {}", e);
                // Continue processing other packets
            } else {
                sent_packet_count += 1;
            }
        }

        // Periodically log statistics
        if last_log_time.elapsed() >= log_interval {
            log_statistics(received_packet_count, sent_packet_count);
            received_packet_count = 0;
            sent_packet_count = 0;
            last_log_time = Instant::now();
        }
    }

    // Cleanup when shutting down
    debug!("Closing packet processing WinDivert handle");

    // First close the handle
    if let Err(e) = wd.close(CloseAction::Nothing) {
        error!("Failed to close WinDivert handle: {}", e);
    } else {
        debug!("Successfully closed packet processing WinDivert handle");
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

    Ok(())
}

/// Processes packets according to the current manipulation settings.
///
/// This function applies various packet manipulations in sequence based on the
/// provided settings. Each manipulation is only applied if it's enabled in the settings.
///
/// The manipulations include:
/// - Packet dropping
/// - Packet delaying
/// - Network throttling
/// - Packet reordering
/// - Packet tampering (corruption)
/// - Packet duplication
/// - Bandwidth limiting
///
/// # Arguments
///
/// * `settings` - The current packet manipulation settings
/// * `packets` - Vector of packets to process
/// * `state` - Current state of the packet processor
/// * `statistics` - Statistics tracker to record manipulation metrics
pub fn process_packets<'a>(
    settings: &PacketManipulationSettings,
    packets: &mut Vec<PacketData<'a>>,
    state: &mut PacketProcessingState<'a>,
    statistics: &Arc<RwLock<PacketProcessingStatistics>>,
) {
    // Apply packet dropping if enabled
    if let Some(drop) = &settings.drop {
        if is_effect_active(drop.duration_ms, state.effect_start_times.drop_start) {
            drop_packets(
                packets,
                drop.probability,
                &mut statistics
                    .write()
                    .unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for drop statistics");
                        panic!("Failed to acquire statistics lock");
                    })
                    .drop_stats,
            );
        } else if !packets.is_empty() {
            // Reset the effect start time when we get new packets and effect is inactive
            state.effect_start_times.drop_start = Instant::now();
        }
    }

    // Apply packet delay if enabled
    if let Some(delay) = &settings.delay {
        debug!(
            "Delay module check: duration_ms={}, elapsed={:?}, active={}",
            delay.duration_ms,
            state.effect_start_times.delay_start.elapsed(),
            is_effect_active(delay.duration_ms, state.effect_start_times.delay_start)
        );

        if is_effect_active(delay.duration_ms, state.effect_start_times.delay_start) {
            delay_packets(
                packets,
                &mut state.delay_storage,
                Duration::from_millis(delay.delay_ms),
                delay.probability,
                &mut statistics
                    .write()
                    .unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for delay statistics");
                        panic!("Failed to acquire statistics lock");
                    })
                    .delay_stats,
            );
        } else if !packets.is_empty() {
            debug!("Delay module deactivated, resetting start time");
            // Reset the effect start time when we get new packets and effect is inactive
            state.effect_start_times.delay_start = Instant::now();
        }
    }

    // Apply throttling if enabled
    if let Some(throttle) = &settings.throttle {
        if is_effect_active(
            throttle.duration_ms,
            state.effect_start_times.throttle_start,
        ) {
            throttle_packages(
                packets,
                &mut state.throttle_storage,
                &mut state.throttled_start_time,
                throttle.probability,
                Duration::from_millis(throttle.throttle_ms),
                throttle.drop,
                &mut statistics
                    .write()
                    .unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for throttle statistics");
                        panic!("Failed to acquire statistics lock");
                    })
                    .throttle_stats,
            );
        } else if !packets.is_empty() {
            // Reset the effect start time when we get new packets and effect is inactive
            state.effect_start_times.throttle_start = Instant::now();
        }
    }

    // Apply packet reordering if enabled
    if let Some(reorder) = &settings.reorder {
        let effect_active =
            is_effect_active(reorder.duration_ms, state.effect_start_times.reorder_start);
        debug!(
            "Reorder check: duration_ms={}, effect_active={}, max_delay={}",
            reorder.duration_ms, effect_active, reorder.max_delay
        );
        if effect_active {
            reorder_packets(
                packets,
                &mut state.reorder_storage,
                reorder.probability,
                Duration::from_millis(reorder.max_delay),
                &mut statistics
                    .write()
                    .unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for reorder statistics");
                        panic!("Failed to acquire statistics lock");
                    })
                    .reorder_stats,
            );
        } else if !packets.is_empty() {
            // Reset the effect start time when we get new packets and effect is inactive
            debug!("Reorder effect inactive, resetting start time");
            state.effect_start_times.reorder_start = Instant::now();
        }
    }

    // Apply packet tampering if enabled
    if let Some(tamper) = &settings.tamper {
        if is_effect_active(tamper.duration_ms, state.effect_start_times.tamper_start) {
            tamper_packets(
                packets,
                tamper.probability,
                tamper.amount,
                tamper.recalculate_checksums.unwrap_or(true),
                &mut statistics
                    .write()
                    .unwrap_or_else(|_| {
                        error!("Failed to acquire write lock for tamper statistics");
                        panic!("Failed to acquire statistics lock");
                    })
                    .tamper_stats,
            );
        } else if !packets.is_empty() {
            // Reset the effect start time when we get new packets and effect is inactive
            state.effect_start_times.tamper_start = Instant::now();
        }
    }

    // Apply packet duplication if enabled
    if let Some(duplicate) = &settings.duplicate {
        if duplicate.count > 1 && duplicate.probability.value() > 0.0 {
            if is_effect_active(
                duplicate.duration_ms,
                state.effect_start_times.duplicate_start,
            ) {
                duplicate_packets(
                    packets,
                    duplicate.count,
                    duplicate.probability,
                    &mut statistics
                        .write()
                        .unwrap_or_else(|_| {
                            error!("Failed to acquire write lock for duplicate statistics");
                            panic!("Failed to acquire statistics lock");
                        })
                        .duplicate_stats,
                );
            } else if !packets.is_empty() {
                // Reset the effect start time when we get new packets and effect is inactive
                state.effect_start_times.duplicate_start = Instant::now();
            }
        }
    }

    // Apply bandwidth limiting if enabled
    if let Some(bandwidth) = &settings.bandwidth {
        if bandwidth.limit > 0 {
            if is_effect_active(
                bandwidth.duration_ms,
                state.effect_start_times.bandwidth_start,
            ) {
                bandwidth_limiter(
                    packets,
                    &mut state.bandwidth_limit_storage,
                    &mut state.bandwidth_storage_total_size,
                    &mut state.last_sent_package_time,
                    bandwidth.limit,
                    &mut statistics
                        .write()
                        .unwrap_or_else(|_| {
                            error!("Failed to acquire write lock for bandwidth statistics");
                            panic!("Failed to acquire statistics lock");
                        })
                        .bandwidth_stats,
                );
            } else if !packets.is_empty() {
                // Reset the effect start time when we get new packets and effect is inactive
                state.effect_start_times.bandwidth_start = Instant::now();
            }
        }
    }
}
