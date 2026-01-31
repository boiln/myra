use crate::error::{MyraError, Result};
use crate::network::core::PacketData;
use crate::network::modules::registry::process_all_modules;
use crate::network::modules::stats::PacketProcessingStatistics;
use crate::network::processing::module_state::ModuleProcessingState;
use crate::settings::Settings;
use crate::utils::log_statistics;
use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use windivert::layer::NetworkLayer;
use windivert::{CloseAction, WinDivert};
use windivert_sys::WinDivertFlags;

/// Swap source and destination IP addresses in a packet (MGO2 bypass technique).
/// This is called when `WinDivertSend` fails - swapping IPs and retrying can bypass
/// certain game anti-lag detection mechanisms.
fn swap_ip_addresses(packet_data: &mut PacketData) -> bool {
    let data = packet_data.packet.data.to_mut();
    
    if data.len() < 20 {
        return false;
    }
    
    let version = (data[0] >> 4) & 0x0F;
    
    match version {
        4 => {
            if data.len() < 20 {
                return false;
            }
            
            for i in 0..4 {
                data.swap(12 + i, 16 + i);
            }
            
            data[10] = 0;
            data[11] = 0;
            
            true
        }
        6 => {
            if data.len() < 40 {
                return false;
            }
            
            for i in 0..16 {
                data.swap(8 + i, 24 + i);
            }
            
            true
        }
        _ => false,
    }
}

/// If the initial send fails, swap source/destination IPs and retry.
fn send_with_bypass(
    wd: &WinDivert<NetworkLayer>,
    packet_data: &mut PacketData,
    enable_bypass: bool,
) -> std::result::Result<(), windivert::error::WinDivertError> {
    match wd.send(&packet_data.packet) {
        Ok(_bytes_sent) => Ok(()),
        Err(e) => {
            if !enable_bypass {
                return Err(e);
            }
            
            warn!("Send failed, attempting IP swap bypass: {}", e);
            
            if !swap_ip_addresses(packet_data) {
                return Err(e);
            }
            
            let current = packet_data.packet.address.outbound();
            packet_data.packet.address.set_outbound(!current);
            
            match wd.send(&packet_data.packet) {
                Ok(_bytes_sent) => {
                    debug!("IP swap bypass successful");
                    Ok(())
                }
                Err(e2) => {
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
/// This function creates a `WinDivert` handle configured for sending packets only,
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
/// Result indicating success or a `MyraError` if something fails
pub fn start_packet_processing(
    settings: Arc<Mutex<Settings>>,
    packet_receiver: Receiver<PacketData>,
    running: Arc<AtomicBool>,
    statistics: Arc<RwLock<PacketProcessingStatistics>>,
) -> Result<()> {
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

    let mut state = ModuleProcessingState::new();

    info!("Starting packet interception.");

    fn sleep_precise(duration: Duration) {
        if duration.is_zero() {
            return;
        }

        if duration < Duration::from_millis(2) {
            let target = Instant::now() + duration;
            while Instant::now() < target {
                std::hint::spin_loop();
            }
            return;
        }

        let to_sleep = duration.checked_sub(Duration::from_millis(1)).unwrap();
        std::thread::sleep(to_sleep);
        let target = Instant::now() + Duration::from_millis(1);
        while Instant::now() < target {
            std::hint::spin_loop();
        }
    }

    let mut enable_bypass = false;
    
    const CYCLE_TIME_MS: u64 = 40;
    
    while running.load(Ordering::SeqCst) {
        let cycle_start = Instant::now();
        let mut packets = Vec::new();

        while let Ok(packet_data) = packet_receiver.try_recv() {
            packets.push(packet_data);
            received_packet_count += 1;
        }

        match settings.lock() {
            Ok(settings) => {
                state.burst_release_delay_us = settings.burst_release_delay_us;
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

        let pacing_needed = packets.len() > 20;
        let release_delay = state.burst_release_delay_us;
        if pacing_needed {
            info!("BURST REPLAY: Sending {} packets with {}us delay each", packets.len(), release_delay);
        }
        for mut packet_data in packets {
            if let Err(e) = send_with_bypass(&wd, &mut packet_data, enable_bypass) {
                error!("Failed to send packet: {e}");
                continue;
            }

            sent_packet_count += 1;
            
            if pacing_needed && release_delay > 0 {
                sleep_precise(Duration::from_micros(release_delay));
            }
        }

        if last_log_time.elapsed() >= log_interval {
            log_statistics(received_packet_count, sent_packet_count);
            received_packet_count = 0;
            sent_packet_count = 0;
            last_log_time = Instant::now();
        }
        
        let elapsed = cycle_start.elapsed();
        if elapsed < Duration::from_millis(CYCLE_TIME_MS) {
            std::thread::sleep(Duration::from_millis(CYCLE_TIME_MS).checked_sub(elapsed).unwrap());
        }
    }

    if !state.burst.buffer.is_empty() {
        while let Some((mut packet, _)) = state.burst.buffer.pop_front() {
            if let Err(e) = send_with_bypass(&wd, &mut packet, enable_bypass) {
                error!("Failed to send buffered packet on shutdown: {e}");
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    debug!("Closing packet processing WinDivert handle");

    let close_result = wd.close(CloseAction::Nothing);
    if let Err(e) = &close_result {
        error!("Failed to close WinDivert handle: {}", e);
    }

    if close_result.is_ok() {
        debug!("Successfully closed packet processing WinDivert handle");
    }

    match WinDivert::<NetworkLayer>::network(
        "false",
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
/// Delegates to the module registry which handles all modules in order:
/// drop → lag → throttle → reorder → corruption → duplicate → bandwidth → burst
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
pub fn process_packets(
    settings: &Settings,
    packets: &mut Vec<PacketData<'_>>,
    state: &mut ModuleProcessingState,
    statistics: &Arc<RwLock<PacketProcessingStatistics>>,
) -> Result<()> {
    if !packets.is_empty() {
        debug!(
            "Processing {} packets through module registry",
            packets.len()
        );
    }

    process_all_modules(settings, packets, state, statistics)
}
