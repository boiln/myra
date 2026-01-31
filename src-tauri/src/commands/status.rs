//! Status and query commands.
//!
//! Handles retrieving the current state of the packet processing engine,
//! including running status, statistics, settings, and filter.

use std::sync::atomic::Ordering;

use log::debug;
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::commands::types::{ModuleConfig, ModuleInfo, ModuleParams, ProcessingStatus, ProcessingStatisticsDto};
use crate::commands::filter_history::add_to_history;
use crate::commands::system::validate_filter;
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
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let modules = build_module_info_list(&settings);
    
    let statistics = if running {
        let stats = state.statistics.read().map_err(|e| e.to_string())?;
        Some(ProcessingStatisticsDto {
            burst_buffered: stats.burst_stats.buffered,
            burst_released: stats.burst_stats.released,
            burst_buffered_count: stats.burst_stats.buffered_count,
            throttle_buffered_count: stats.throttle_stats.buffered_count(),
            throttle_dropped_count: stats.throttle_stats.dropped_count(),
            throttle_is_throttling: stats.throttle_stats.is_throttling(),
            lag_current_lagged: stats.lag_stats.current_lagged(),
            reorder_delayed_packets: stats.reorder_stats.delayed_packets,
        })
    } else {
        None
    };

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
/// * `Ok(Settings)` - The current settings
/// * `Err(String)` - If there was an error retrieving settings
#[tauri::command]
pub async fn get_settings(
    state: State<'_, PacketProcessingState>,
) -> Result<Settings, String> {
    Ok(state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))?
        .clone())
}

/// Gets the current `WinDivert` filter expression.
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

/// Updates the `WinDivert` filter expression.
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
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = filter.clone();
    if let Some(ref f) = filter {
        if validate_filter(f.clone()).unwrap_or(false) {
            let _ = add_to_history(f);
        }
    }
    debug!("Updated packet filter");
    Ok(())
}

/// Builds a list of `ModuleInfo` from the current settings.
/// Always returns all modules with their settings, using enabled field to track active state.
fn build_module_info_list(settings: &Settings) -> Vec<ModuleInfo> {
    use crate::settings::lag::LagOptions;

    let module = |name: &str, display_name: &str, enabled: bool, config: ModuleConfig| ModuleInfo {
        name: name.to_string(),
        display_name: display_name.to_string(),
        enabled,
        config,
        params: None,
    };

    let lag = settings.lag.clone().unwrap_or_else(|| LagOptions {
        enabled: false,
        inbound: true,
        outbound: true,
        delay_ms: 1000,
        ..Default::default()
    });
    let lag_info = ModuleInfo {
        params: Some(ModuleParams {
            lag_time: Some(lag.delay_ms),
        }),
        ..module(
            "lag",
            "Lag",
            lag.enabled,
            ModuleConfig {
                inbound: lag.inbound,
                outbound: lag.outbound,
                chance: lag.probability.value() * 100.0,
                enabled: lag.enabled,
                duration_ms: Some(lag.delay_ms),
                throttle_ms: Some(lag.delay_ms),
                ..Default::default()
            },
        )
    };

    let drop = settings.drop.clone().unwrap_or_default();
    let drop_info = module(
        "drop",
        "Drop",
        drop.enabled,
        ModuleConfig {
            inbound: drop.inbound,
            outbound: drop.outbound,
            chance: drop.probability.value() * 100.0,
            enabled: drop.enabled,
            duration_ms: Some(drop.duration_ms),
            ..Default::default()
        },
    );

    let throttle = settings.throttle.clone().unwrap_or_default();
    let throttle_info = module(
        "throttle",
        "Throttle",
        throttle.enabled,
        ModuleConfig {
            inbound: throttle.inbound,
            outbound: throttle.outbound,
            chance: throttle.probability.value() * 100.0,
            enabled: throttle.enabled,
            duration_ms: Some(throttle.duration_ms),
            throttle_ms: Some(throttle.throttle_ms),
            drop: Some(throttle.drop),
            max_buffer: Some(throttle.max_buffer),
            freeze_mode: Some(throttle.freeze_mode),
            ..Default::default()
        },
    );

    let duplicate = settings.duplicate.clone().unwrap_or_default();
    let duplicate_info = module(
        "duplicate",
        "Duplicate",
        duplicate.enabled,
        ModuleConfig {
            inbound: duplicate.inbound,
            outbound: duplicate.outbound,
            chance: duplicate.probability.value() * 100.0,
            enabled: duplicate.enabled,
            duration_ms: Some(duplicate.duration_ms),
            count: Some(duplicate.count),
            ..Default::default()
        },
    );

    let bandwidth = settings.bandwidth.clone().unwrap_or_default();
    let bandwidth_info = module(
        "bandwidth",
        "Bandwidth",
        bandwidth.enabled,
        ModuleConfig {
            inbound: bandwidth.inbound,
            outbound: bandwidth.outbound,
            chance: bandwidth.probability.value() * 100.0,
            enabled: bandwidth.enabled,
            duration_ms: Some(bandwidth.duration_ms),
            limit_kbps: Some(if bandwidth.limit == 0 { 50 } else { bandwidth.limit as u64 }),
            passthrough_threshold: Some(bandwidth.passthrough_threshold),
            use_wfp: Some(bandwidth.use_wfp),
            ..Default::default()
        },
    );

    let corruption = settings.corruption.clone().unwrap_or_default();
    let corruption_info = module(
        "corruption",
        "Corruption",
        corruption.enabled,
        ModuleConfig {
            inbound: corruption.inbound,
            outbound: corruption.outbound,
            chance: corruption.probability.value() * 100.0,
            enabled: corruption.enabled,
            duration_ms: Some(corruption.duration_ms),
            ..Default::default()
        },
    );

    let reorder = settings.reorder.clone().unwrap_or_default();
    let reorder_info = module(
        "reorder",
        "Reorder",
        reorder.enabled,
        ModuleConfig {
            inbound: reorder.inbound,
            outbound: reorder.outbound,
            chance: reorder.probability.value() * 100.0,
            enabled: reorder.enabled,
            duration_ms: Some(reorder.duration_ms),
            throttle_ms: Some(reorder.max_delay),
            ..Default::default()
        },
    );

    let burst = settings.burst.clone().unwrap_or_default();
    let burst_info = module(
        "burst",
        "Burst",
        burst.enabled,
        ModuleConfig {
            inbound: burst.inbound,
            outbound: burst.outbound,
            chance: burst.probability.value() * 100.0,
            enabled: burst.enabled,
            duration_ms: Some(burst.duration_ms),
            buffer_ms: Some(burst.buffer_ms),
            keepalive_ms: Some(burst.keepalive_ms),
            release_delay_us: Some(burst.release_delay_us),
            lag_bypass: Some(settings.lag_bypass),
            reverse: Some(burst.reverse),
            ..Default::default()
        },
    );

    vec![
        lag_info,
        drop_info,
        throttle_info,
        duplicate_info,
        bandwidth_info,
        corruption_info,
        reorder_info,
        burst_info,
    ]
}
