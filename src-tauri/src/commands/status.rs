//! Status and query commands.
//!
//! Handles retrieving the current state of the packet processing engine,
//! including running status, statistics, settings, and filter.

use std::sync::atomic::Ordering;

use log::debug;
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::commands::types::{ModuleConfig, ModuleInfo, ModuleParams, ProcessingStatus};
use crate::settings::manipulation::PacketManipulationSettings;
use crate::settings::Settings;

/// Gets the current status of the processing engine.
///
/// Returns the current state of the packet processing engine, including
/// whether it's running, statistics, and configurations of all modules.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
///
/// # Returns
///
/// * `ProcessingStatus` - The current processing status
#[tauri::command]
pub async fn get_status(
    state: State<'_, PacketProcessingState>,
) -> Result<ProcessingStatus, String> {
    let running = state.running.load(Ordering::SeqCst);
    let statistics = if !running {
        None
    } else {
        let stats = state.statistics.read().map_err(|e| e.to_string())?;
        Some(format!("{:?}", stats))
    };
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let modules = build_module_info_list(&settings);

    Ok(ProcessingStatus {
        running,
        statistics,
        modules,
    })
}

/// Gets the current packet manipulation settings.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
///
/// # Returns
///
/// * `Ok(PacketManipulationSettings)` - The current settings
/// * `Err(String)` - If there was an error retrieving settings
#[tauri::command]
pub async fn get_settings(
    state: State<'_, PacketProcessingState>,
) -> Result<PacketManipulationSettings, String> {
    Ok(state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))?
        .clone())
}

/// Gets the current WinDivert filter expression.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
///
/// # Returns
///
/// * `Ok(Option<String>)` - The current filter expression
/// * `Err(String)` - If there was an error retrieving the filter
#[tauri::command]
pub async fn get_filter(state: State<'_, PacketProcessingState>) -> Result<Option<String>, String> {
    Ok(state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))?
        .clone())
}

/// Updates the WinDivert filter expression.
///
/// Changes which packets are captured for manipulation.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
/// * `filter` - The new filter expression
///
/// # Returns
///
/// * `Ok(())` - If filter was updated successfully
/// * `Err(String)` - If there was an error updating the filter
#[tauri::command]
pub async fn update_filter(
    state: State<'_, PacketProcessingState>,
    filter: Option<String>,
) -> Result<(), String> {
    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = filter;
    debug!("Updated packet filter");
    Ok(())
}

/// Builds a list of ModuleInfo from the current settings.
/// Always returns all modules with their settings, using enabled field to track active state.
fn build_module_info_list(settings: &Settings) -> Vec<ModuleInfo> {
    use crate::settings::bandwidth::BandwidthOptions;
    use crate::settings::burst::BurstOptions;
    use crate::settings::delay::DelayOptions;
    use crate::settings::drop::DropOptions;
    use crate::settings::duplicate::DuplicateOptions;
    use crate::settings::reorder::ReorderOptions;
    use crate::settings::tamper::TamperOptions;
    use crate::settings::throttle::ThrottleOptions;

    let mut modules = Vec::new();

    // Delay module - always include
    let delay = settings.delay.as_ref().cloned().unwrap_or_else(|| DelayOptions {
        enabled: false,
        inbound: true,
        outbound: true,
        delay_ms: 1000,
        ..Default::default()
    });
    modules.push(ModuleInfo {
        name: "delay".to_string(),
        display_name: "Delay".to_string(),
        enabled: delay.enabled,
        config: ModuleConfig {
            inbound: delay.inbound,
            outbound: delay.outbound,
            chance: delay.probability.value() * 100.0,
            enabled: delay.enabled,
            duration_ms: Some(delay.delay_ms),
            throttle_ms: Some(delay.delay_ms),
            limit_kbps: None,
            count: None,
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: None,
            max_buffer: None,
            lag_bypass: None,
            freeze_mode: None,
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: Some(ModuleParams {
            lag_time: Some(delay.delay_ms),
        }),
    });

    // Drop module (Freeze) - always include
    let drop = settings.drop.as_ref().cloned().unwrap_or_default();
    modules.push(ModuleInfo {
        name: "drop".to_string(),
        display_name: "Drop".to_string(),
        enabled: drop.enabled,
        config: ModuleConfig {
            inbound: drop.inbound,
            outbound: drop.outbound,
            chance: drop.probability.value() * 100.0,
            enabled: drop.enabled,
            duration_ms: Some(drop.duration_ms),
            throttle_ms: None,
            limit_kbps: None,
            count: None,
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: None,
            max_buffer: None,
            lag_bypass: None,
            freeze_mode: None,
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: None,
    });

    // Throttle module - always include
    let throttle = settings.throttle.as_ref().cloned().unwrap_or_default();
    modules.push(ModuleInfo {
        name: "throttle".to_string(),
        display_name: "Throttle".to_string(),
        enabled: throttle.enabled,
        config: ModuleConfig {
            inbound: throttle.inbound,
            outbound: throttle.outbound,
            chance: throttle.probability.value() * 100.0,
            enabled: throttle.enabled,
            duration_ms: Some(throttle.duration_ms),
            throttle_ms: Some(throttle.throttle_ms),
            limit_kbps: None,
            count: None,
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: Some(throttle.drop),
            max_buffer: Some(throttle.max_buffer),
            lag_bypass: None,
            freeze_mode: Some(throttle.freeze_mode),
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: None,
    });

    // Duplicate module - always include
    let duplicate = settings.duplicate.as_ref().cloned().unwrap_or_default();
    modules.push(ModuleInfo {
        name: "duplicate".to_string(),
        display_name: "Duplicate".to_string(),
        enabled: duplicate.enabled,
        config: ModuleConfig {
            inbound: duplicate.inbound,
            outbound: duplicate.outbound,
            chance: duplicate.probability.value() * 100.0,
            enabled: duplicate.enabled,
            duration_ms: Some(duplicate.duration_ms),
            throttle_ms: None,
            limit_kbps: None,
            count: Some(duplicate.count),
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: None,
            max_buffer: None,
            lag_bypass: None,
            freeze_mode: None,
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: None,
    });

    // Bandwidth module - always include
    let bandwidth = settings.bandwidth.as_ref().cloned().unwrap_or_default();
    let limit_kbps = Some(if bandwidth.limit > 0 {
        bandwidth.limit as u64
    } else {
        50 // Default limit
    });
    modules.push(ModuleInfo {
        name: "bandwidth".to_string(),
        display_name: "Bandwidth".to_string(),
        enabled: bandwidth.enabled,
        config: ModuleConfig {
            inbound: bandwidth.inbound,
            outbound: bandwidth.outbound,
            chance: bandwidth.probability.value() * 100.0,
            enabled: bandwidth.enabled,
            duration_ms: Some(bandwidth.duration_ms),
            throttle_ms: None,
            limit_kbps,
            count: None,
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: None,
            max_buffer: None,
            lag_bypass: None,
            freeze_mode: None,
            passthrough_threshold: Some(bandwidth.passthrough_threshold),
            use_wfp: Some(bandwidth.use_wfp),
        },
        params: None,
    });

    // Tamper module - always include
    let tamper = settings.tamper.as_ref().cloned().unwrap_or_default();
    modules.push(ModuleInfo {
        name: "tamper".to_string(),
        display_name: "Tamper".to_string(),
        enabled: tamper.enabled,
        config: ModuleConfig {
            inbound: tamper.inbound,
            outbound: tamper.outbound,
            chance: tamper.probability.value() * 100.0,
            enabled: tamper.enabled,
            duration_ms: Some(tamper.duration_ms),
            throttle_ms: None,
            limit_kbps: None,
            count: None,
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: None,
            max_buffer: None,
            lag_bypass: None,
            freeze_mode: None,
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: None,
    });

    // Reorder module - always include
    let reorder = settings.reorder.as_ref().cloned().unwrap_or_default();
    modules.push(ModuleInfo {
        name: "reorder".to_string(),
        display_name: "Reorder".to_string(),
        enabled: reorder.enabled,
        config: ModuleConfig {
            inbound: reorder.inbound,
            outbound: reorder.outbound,
            chance: reorder.probability.value() * 100.0,
            enabled: reorder.enabled,
            duration_ms: Some(reorder.duration_ms),
            throttle_ms: Some(reorder.max_delay),
            limit_kbps: None,
            count: None,
            buffer_ms: None,
            keepalive_ms: None,
            release_delay_us: None,
            drop: None,
            max_buffer: None,
            lag_bypass: None,
            freeze_mode: None,
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: None,
    });

    // Burst module (lag switch) - always include
    let burst = settings.burst.as_ref().cloned().unwrap_or_default();
    modules.push(ModuleInfo {
        name: "burst".to_string(),
        display_name: "Burst".to_string(),
        enabled: burst.enabled,
        config: ModuleConfig {
            inbound: burst.inbound,
            outbound: burst.outbound,
            chance: burst.probability.value() * 100.0,
            enabled: burst.enabled,
            duration_ms: Some(burst.duration_ms),
            throttle_ms: None,
            limit_kbps: None,
            count: None,
            buffer_ms: Some(burst.buffer_ms),
            keepalive_ms: Some(burst.keepalive_ms),
            release_delay_us: Some(burst.release_delay_us),
            drop: None,
            max_buffer: None,
            lag_bypass: Some(settings.lag_bypass),
            freeze_mode: None,
            passthrough_threshold: None,
            use_wfp: None,
        },
        params: None,
    });

    modules
}
