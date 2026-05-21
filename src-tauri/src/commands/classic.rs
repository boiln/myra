//! Classic mode Tauri commands.
//!
//! Commands for controlling Classic mode packet processing.
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;

use log::{error, info};
use tauri::State;

use crate::commands::classic_state::ClassicProcessingState;
use crate::commands::state::PacketProcessingState;
use crate::network::classic::{process_classic_packets, ClassicProcessingState as ClassicModuleState};
use crate::network::core::set_high_precision_timer;
use crate::settings::classic::ClassicSettings;

/// Starts Classic mode packet processing with the given settings and filter.
#[tauri::command]
pub async fn start_classic_processing(
    state: State<'_, PacketProcessingState>,
    classic_state: State<'_, ClassicProcessingState>,
    settings: ClassicSettings,
    filter: Option<String>,
) -> Result<(), String> {

    // Check if standard mode is running
    if state.running.load(Ordering::SeqCst) {
        return Err("Standard mode processing is running. Stop it first.".to_string());
    }

    // Check if classic mode is already running
    if classic_state.running.load(Ordering::SeqCst) {
        return Err("Classic mode processing already running".to_string());
    }

    // Store settings
    *classic_state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock classic settings mutex: {}", e))? = settings;

    // Store filter in the standard state (shared)
    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = filter;

    let (packet_sender, packet_receiver) = mpsc::channel();

    // Set running flag
    classic_state.running.store(true, Ordering::SeqCst);

    // Set high-precision timer
    set_high_precision_timer();

    let running_recv = classic_state.running.clone();
    let settings_recv = state.settings.clone(); // Needed for receiver
    let filter_recv = state.filter.clone();

    // Spawn packet receiver thread (reuses standard receiver)
    thread::spawn(move || {

        if let Err(e) = crate::network::processing::receive_packets(
            packet_sender,
            running_recv,
            settings_recv,
            filter_recv
        ) {
            error!("Classic mode packet receiving error: {}", e);
        }

    });

    let running_proc = classic_state.running.clone();
    let classic_settings = classic_state.settings.clone();

    // Spawn Classic mode processor thread
    thread::spawn(move || {

        if let Err(e) = start_classic_packet_processing(
            classic_settings,
            packet_receiver,
            running_proc,
        ) {
            error!("Classic mode packet processing error: {}", e);
        }

    });

    info!("Started Classic mode packet processing");

    Ok(())

}

/// Stops Classic mode packet processing.
#[tauri::command]
pub async fn stop_classic_processing(
    classic_state: State<'_, ClassicProcessingState>,
) -> Result<(), String> {

    if !classic_state.running.load(Ordering::SeqCst) {
        return Err("Classic mode processing not running".to_string());
    }

    classic_state.running.store(false, Ordering::SeqCst);
    info!("Stopped Classic mode packet processing");

    Ok(())

}

/// Updates Classic mode settings while processing is running.
#[tauri::command]
pub async fn update_classic_settings(
    classic_state: State<'_, ClassicProcessingState>,
    settings: ClassicSettings,
) -> Result<(), String> {

    info!("Updating Classic settings: latency={:?}, drop={:?}, throttle={:?}",
        settings.latency.as_ref().map(|o| o.enabled),
        settings.drop.as_ref().map(|o| o.enabled),
        settings.throttle.as_ref().map(|o| o.enabled)
    );

    *classic_state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock classic settings mutex: {}", e))? = settings;

    Ok(())

}

/// Gets the current Classic mode status.
#[tauri::command]
pub async fn get_classic_status(
    classic_state: State<'_, ClassicProcessingState>,
) -> Result<ClassicStatusResponse, String> {

    let running = classic_state.running.load(Ordering::SeqCst);
    let settings = classic_state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock classic settings mutex: {}", e))?
        .clone();

    Ok(ClassicStatusResponse { running, settings })

}

#[derive(serde::Serialize)]
pub struct ClassicStatusResponse {
    pub running: bool,
    pub settings: ClassicSettings,
}

/// Classic mode packet processing loop.
fn start_classic_packet_processing(
    settings: std::sync::Arc<std::sync::Mutex<ClassicSettings>>,
    packet_receiver: std::sync::mpsc::Receiver<crate::network::core::PacketData>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> crate::error::Result<()> {

    use std::time::{Duration, Instant};
    use windivert::layer::NetworkLayer;
    use windivert::{CloseAction, WinDivert};
    use windivert_sys::WinDivertFlags;

    // Initialize WinDivert for sending packets only
    let mut wd = WinDivert::<NetworkLayer>::network(
        "false",
        0,
        WinDivertFlags::set_send_only(WinDivertFlags::new()),
    ).map_err(|e| {
        error!("Failed to initialize WinDivert for Classic mode: {}", e);
        crate::error::MyraError::WinDivert(e)
    })?;

    let mut state = ClassicModuleState::new();

    // Use 40ms processing cycles (same as original)
    const CYCLE_TIME_MS: u64 = 40;

    info!("Classic mode processing started");

    let mut packet_count: u64 = 0;
    let mut last_log_time = Instant::now();

    while running.load(Ordering::SeqCst) {
        let cycle_start = Instant::now();
        let mut packets = Vec::new();

        // Collect all available packets
        while let Ok(packet_data) = packet_receiver.try_recv() {
            packets.push(packet_data);
        }

        let packets_this_cycle = packets.len();

        packet_count += packets_this_cycle as u64;

        // Log every 2 seconds
        if last_log_time.elapsed() >= Duration::from_secs(2) {
            let settings_guard = settings.lock().ok();
            let any_enabled = settings_guard.as_ref().map(|s| s.has_any_enabled()).unwrap_or(false);

            info!("Classic: {} packets received, {} this cycle, any_module_enabled={}",
                packet_count, packets_this_cycle, any_enabled);
            last_log_time = Instant::now();
        }

        // Apply Classic mode processing
        match settings.lock() {
            Ok(settings) => {
                process_classic_packets(&mut packets, &settings, &mut state);
            }
            Err(e) => {
                error!("Failed to acquire lock on Classic settings: {}", e);
            }
        }

        // Send processed packets
        for packet_data in packets {
            if let Err(e) = wd.send(&packet_data.packet) {
                error!("Failed to send packet: {}", e);
            }
        }

        // Maintain 40ms cycle time
        let elapsed = cycle_start.elapsed();

        if elapsed < Duration::from_millis(CYCLE_TIME_MS) {
            std::thread::sleep(Duration::from_millis(CYCLE_TIME_MS) - elapsed);
        }
    }

    // Flush all buffers on shutdown
    let remaining = state.flush_all_buffers();

    for packet_data in remaining {
        if let Err(e) = wd.send(&packet_data.packet) {
            error!("Failed to send buffered packet on shutdown: {}", e);
        }
    }

    // Close handle
    let _ = wd.close(CloseAction::Nothing);

    info!("Classic mode processing stopped");

    Ok(())

}
