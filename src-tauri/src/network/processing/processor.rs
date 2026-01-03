use crate::error::{MyraError, Result};
use crate::network::core::PacketData;
use crate::network::modules::burst::flush_buffer;
use crate::network::modules::stats::PacketProcessingStatistics;
use crate::network::modules::traits::ModuleContext;
use crate::network::modules::{
    BandwidthModule, BurstModule, LagModule, DropModule, DuplicateModule, PacketModule,
    ReorderModule, TamperModule, ThrottleModule,
};
use crate::network::processing::module_state::ModuleProcessingState;
use crate::settings::Settings;
use crate::utils::{is_effect_active, log_statistics};
use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use windivert::layer::NetworkLayer;
use windivert::{CloseAction, WinDivert};
use windivert_sys::WinDivertFlags;

/// Swap source and destination IP addresses in a packet (MGO2 bypass technique).
/// This is called when WinDivertSend fails - swapping IPs and retrying can bypass
/// certain game anti-lag detection mechanisms.
fn swap_ip_addresses(packet_data: &mut PacketData) -> bool {
    // Get mutable access to the packet data
    let data = packet_data.packet.data.to_mut();
    
    if data.len() < 20 {
        return false; // Too small for IPv4 header
    }
    
    let version = (data[0] >> 4) & 0x0F;
    
    match version {
        4 => {
            // IPv4: Source IP at offset 12-15, Dest IP at offset 16-19
            if data.len() < 20 {
                return false;
            }
            
            // Swap the 4-byte source and destination addresses
            for i in 0..4 {
                data.swap(12 + i, 16 + i);
            }
            
            // Zero out the checksum (bytes 10-11) so WinDivert recalculates it
            data[10] = 0;
            data[11] = 0;
            
            true
        }
        6 => {
            // IPv6: Source IP at offset 8-23, Dest IP at offset 24-39
            if data.len() < 40 {
                return false;
            }
            
            // Swap the 16-byte source and destination addresses
            for i in 0..16 {
                data.swap(8 + i, 24 + i);
            }
            
            true
        }
        _ => false,
    }
}

/// Attempt to send a packet with the MGO2 bypass technique.
/// If the initial send fails, swap source/destination IPs and retry.
fn send_with_bypass(
    wd: &mut WinDivert<NetworkLayer>,
    packet_data: &mut PacketData,
    enable_bypass: bool,
) -> std::result::Result<(), windivert::error::WinDivertError> {
    // First attempt - normal send
    match wd.send(&packet_data.packet) {
        Ok(_bytes_sent) => Ok(()),
        Err(e) => {
            if !enable_bypass {
                return Err(e);
            }
            
            // Send failed - try the MGO2 bypass technique
            warn!("Send failed, attempting IP swap bypass: {}", e);
            
            // Swap source and destination IPs
            if !swap_ip_addresses(packet_data) {
                return Err(e);
            }
            
            // Mark as outbound (flip the direction bit)
            let current = packet_data.packet.address.outbound();
            packet_data.packet.address.set_outbound(!current);
            
            // Retry the send
            match wd.send(&packet_data.packet) {
                Ok(_bytes_sent) => {
                    debug!("IP swap bypass successful");
                    Ok(())
                }
                Err(e2) => {
                    // Swap back to restore original state
                    swap_ip_addresses(packet_data);
                    let current = packet_data.packet.address.outbound();
                    packet_data.packet.address.set_outbound(!current);
                    Err(e2)
                }
            }
        }
    }
}

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
/// Result indicating success or a MyraError if something fails
pub fn start_packet_processing(
    settings: Arc<Mutex<Settings>>,
    packet_receiver: Receiver<PacketData>,
    running: Arc<AtomicBool>,
    statistics: Arc<RwLock<PacketProcessingStatistics>>,
) -> Result<()> {
    // Initialize WinDivert for sending packets only
    let mut wd = WinDivert::<NetworkLayer>::network(
        "false",
        0,
        WinDivertFlags::set_send_only(WinDivertFlags::new()),
    )
    .map_err(|e| {
        error!("Failed to initialize WinDivert: {}", e);
        error!("WinDivert error detailed: {:?}", e);
        MyraError::WinDivert(e)
    })?;

    let log_interval = Duration::from_secs(2);
    let mut last_log_time = Instant::now();

    let mut received_packet_count = 0;
    let mut sent_packet_count = 0;

    // Initialize module processing state
    let mut state = ModuleProcessingState::new();

    info!("Starting packet interception.");

    // Track whether bypass is enabled
    let mut enable_bypass = false;
    
    // Use 40ms processing cycles for consistent packet timing
    const CYCLE_TIME_MS: u64 = 40;
    
    // Main processing loop
    while running.load(Ordering::SeqCst) {
        let cycle_start = Instant::now();
        let mut packets = Vec::new();

        // Collect all available packets from the channel
        while let Ok(packet_data) = packet_receiver.try_recv() {
            packets.push(packet_data);
            received_packet_count += 1;
        }

        // Apply packet manipulations according to current settings
        match settings.lock() {
            Ok(settings) => {
                enable_bypass = settings.lag_bypass;
                
                if let Err(e) = process_packets(&settings, &mut packets, &mut state, &statistics) {
                    error!("Error processing packets: {}", e);
                }
            }
            Err(e) => {
                error!(
                    "Failed to acquire lock on packet manipulation settings: {}",
                    e
                );
            }
        }

        // Send the processed packets
        for mut packet_data in packets {
            if let Err(e) = send_with_bypass(&mut wd, &mut packet_data, enable_bypass) {
                error!("Failed to send packet: {}", e);
                continue;
            }
            sent_packet_count += 1;
        }

        // Periodically log statistics
        if last_log_time.elapsed() >= log_interval {
            log_statistics(received_packet_count, sent_packet_count);
            received_packet_count = 0;
            sent_packet_count = 0;
            last_log_time = Instant::now();
        }
        
        // Maintain 40ms cycle time for consistent packet processing
        // This ensures packets are processed at a consistent rate
        let elapsed = cycle_start.elapsed();
        if elapsed < Duration::from_millis(CYCLE_TIME_MS) {
            std::thread::sleep(Duration::from_millis(CYCLE_TIME_MS) - elapsed);
        }
    }

    // FLUSH BURST BUFFER ON SHUTDOWN - release all buffered packets before closing
    if !state.burst.buffer.is_empty() {
        while let Some((mut packet, _)) = state.burst.buffer.pop_front() {
            match send_with_bypass(&mut wd, &mut packet, enable_bypass) {
                Err(e) => error!("Failed to send buffered packet on shutdown: {}", e),
                Ok(_) => sent_packet_count += 1,
            }
        }
        // Give packets time to actually transmit before closing handle
        std::thread::sleep(Duration::from_millis(250));
    }

    // Cleanup when shutting down
    debug!("Closing packet processing WinDivert handle");

    // First close the handle
    let close_result = wd.close(CloseAction::Nothing);
    if let Err(e) = &close_result {
        error!("Failed to close WinDivert handle: {}", e);
    }

    if close_result.is_ok() {
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
///
/// # Returns
///
/// `Ok(())` on success, or `MyraError` if any module fails to process.
pub fn process_packets<'a>(
    settings: &Settings,
    packets: &mut Vec<PacketData<'a>>,
    state: &mut ModuleProcessingState,
    statistics: &Arc<RwLock<PacketProcessingStatistics>>,
) -> Result<()> {
    let has_packets = !packets.is_empty();

    // Debug: log active modules
    if has_packets {
        debug!(
            "Processing {} packets. Active modules: drop={}, lag={}, throttle={}, reorder={}, tamper={}, duplicate={}, bandwidth={}, burst={}",
            packets.len(),
            settings.drop.as_ref().map(|d| d.enabled).unwrap_or(false),
            settings.lag.as_ref().map(|d| d.enabled).unwrap_or(false),
            settings.throttle.as_ref().map(|t| t.enabled).unwrap_or(false),
            settings.reorder.as_ref().map(|r| r.enabled).unwrap_or(false),
            settings.tamper.as_ref().map(|t| t.enabled).unwrap_or(false),
            settings.duplicate.as_ref().map(|d| d.enabled).unwrap_or(false),
            settings.bandwidth.as_ref().map(|b| b.enabled).unwrap_or(false),
            settings.burst.as_ref().map(|b| b.enabled).unwrap_or(false)
        );
    }

    // Process each module using the trait-based approach
    process_module(
        &DropModule,
        settings.drop.as_ref(),
        packets,
        &mut (),
        &mut state.effect_start_times.drop,
        statistics,
        has_packets,
    )?;

    process_module(
        &LagModule,
        settings.lag.as_ref(),
        packets,
        &mut state.lag,
        &mut state.effect_start_times.lag,
        statistics,
        has_packets,
    )?;

    process_module(
        &ThrottleModule,
        settings.throttle.as_ref(),
        packets,
        &mut state.throttle,
        &mut state.effect_start_times.throttle,
        statistics,
        has_packets,
    )?;

    process_module(
        &ReorderModule,
        settings.reorder.as_ref(),
        packets,
        &mut state.reorder,
        &mut state.effect_start_times.reorder,
        statistics,
        has_packets,
    )?;

    process_module(
        &TamperModule,
        settings.tamper.as_ref(),
        packets,
        &mut (),
        &mut state.effect_start_times.tamper,
        statistics,
        has_packets,
    )?;

    process_module(
        &DuplicateModule,
        settings.duplicate.as_ref(),
        packets,
        &mut (),
        &mut state.effect_start_times.duplicate,
        statistics,
        has_packets,
    )?;

    process_module(
        &BandwidthModule,
        settings.bandwidth.as_ref(),
        packets,
        &mut state.bandwidth,
        &mut state.effect_start_times.bandwidth,
        statistics,
        has_packets,
    )?;

    // Special handling for burst module - flush buffer when disabled
    let burst_enabled = settings.burst.as_ref().map(|b| b.enabled).unwrap_or(false);
    if state.burst_was_enabled && !burst_enabled {
        // Burst was just disabled - flush all buffered packets
        let buffer: &mut std::collections::VecDeque<(PacketData<'a>, Instant)> =
            unsafe { std::mem::transmute(&mut state.burst.buffer) };
        let replay_queue: &mut std::collections::VecDeque<(PacketData<'a>, std::time::Duration)> =
            unsafe { std::mem::transmute(&mut state.burst.replay_queue) };
        flush_buffer(packets, buffer, replay_queue, &mut state.burst.cycle_start);
    }
    state.burst_was_enabled = burst_enabled;

    process_module(
        &BurstModule,
        settings.burst.as_ref(),
        packets,
        &mut state.burst,
        &mut state.effect_start_times.burst,
        statistics,
        has_packets,
    )?;

    Ok(())
}

/// Generic function to process a single module.
///
/// Handles the common logic of checking if a module should run,
/// managing effect duration, and invoking the module's process function.
///
/// # Type Parameters
///
/// * `M` - The module type implementing `PacketModule`
///
/// # Arguments
///
/// * `module` - The module instance to use for processing
/// * `options` - Optional configuration for the module
/// * `packets` - Vector of packets to process
/// * `module_state` - Module-specific state
/// * `effect_start` - When the effect started (for duration tracking)
/// * `statistics` - Shared statistics
/// * `has_packets` - Whether there are packets to process
///
/// # Returns
///
/// `Ok(())` on success, or `MyraError` if processing fails.
fn process_module<'a, M>(
    module: &M,
    options: Option<&M::Options>,
    packets: &mut Vec<PacketData<'a>>,
    module_state: &mut M::State,
    effect_start: &mut Instant,
    statistics: &Arc<RwLock<PacketProcessingStatistics>>,
    has_packets: bool,
) -> Result<()>
where
    M: PacketModule,
{
    use crate::network::modules::traits::ModuleOptions;
    
    let Some(opts) = options else {
        return Ok(());
    };

    // Check if module is enabled
    if !opts.is_enabled() {
        return Ok(());
    }

    // Check if module should be skipped based on its options
    if module.should_skip(opts) {
        return Ok(());
    }

    let duration_ms = module.get_duration_ms(opts);
    let effect_active = is_effect_active(duration_ms, *effect_start);

    // If effect duration has expired, don't process
    // Effect will reset when filter is restarted
    if !effect_active {
        return Ok(());
    }

    // Create context and process
    let mut ctx = ModuleContext {
        statistics,
        has_packets,
        effect_start,
    };

    module.process(packets, opts, module_state, &mut ctx)
}
