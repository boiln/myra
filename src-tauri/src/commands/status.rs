//! Status and query commands.
//!
//! Handles retrieving the current state of the packet processing engine,
//! including running status, statistics, settings, and filter.

use std::sync::atomic::Ordering;

use log::debug;
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::commands::types::{ModuleConfig, ModuleInfo, ModuleParams, ProcessingStatus};
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

    let statistics = if running {
        let stats = state.statistics.read().map_err(|e| e.to_string())?;
        Some(format!("{:?}", stats))
    } else {
        None
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
fn build_module_info_list(settings: &Settings) -> Vec<ModuleInfo> {
    let mut modules = Vec::new();

    // Drop module (Freeze)
    if let Some(drop) = &settings.drop {
        modules.push(ModuleInfo {
            name: "drop".to_string(),
            display_name: "Freeze".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: drop.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(drop.duration_ms),
                throttle_ms: None,
                limit_kbps: None,
                count: None,
            },
            params: None,
        });
    }

    // Delay module
    if let Some(delay) = &settings.delay {
        modules.push(ModuleInfo {
            name: "delay".to_string(),
            display_name: "Delay".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: delay.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(delay.duration_ms),
                throttle_ms: Some(delay.delay_ms),
                limit_kbps: None,
                count: None,
            },
            params: Some(ModuleParams {
                lag_time: Some(delay.delay_ms),
            }),
        });
    }

    // Throttle module
    if let Some(throttle) = &settings.throttle {
        modules.push(ModuleInfo {
            name: "throttle".to_string(),
            display_name: "Throttle".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: throttle.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(throttle.duration_ms),
                throttle_ms: Some(throttle.throttle_ms),
                limit_kbps: None,
                count: None,
            },
            params: None,
        });
    }

    // Duplicate module
    if let Some(duplicate) = &settings.duplicate {
        modules.push(ModuleInfo {
            name: "duplicate".to_string(),
            display_name: "Duplicate".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: duplicate.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(duplicate.duration_ms),
                throttle_ms: None,
                limit_kbps: None,
                count: Some(duplicate.count),
            },
            params: None,
        });
    }

    // Bandwidth module
    if let Some(bandwidth) = &settings.bandwidth {
        let limit_kbps = if bandwidth.limit > 0 {
            Some(bandwidth.limit as u64)
        } else {
            None
        };

        modules.push(ModuleInfo {
            name: "bandwidth".to_string(),
            display_name: "Bandwidth".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: bandwidth.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(bandwidth.duration_ms),
                throttle_ms: None,
                limit_kbps,
                count: None,
            },
            params: None,
        });
    }

    // Tamper module
    if let Some(tamper) = &settings.tamper {
        modules.push(ModuleInfo {
            name: "tamper".to_string(),
            display_name: "Tamper".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: tamper.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(tamper.duration_ms),
                throttle_ms: None,
                limit_kbps: None,
                count: None,
            },
            params: None,
        });
    }

    // Reorder module
    if let Some(reorder) = &settings.reorder {
        modules.push(ModuleInfo {
            name: "reorder".to_string(),
            display_name: "Reorder".to_string(),
            enabled: true,
            config: ModuleConfig {
                inbound: true,
                outbound: true,
                chance: reorder.probability.value() * 100.0,
                enabled: true,
                duration_ms: Some(reorder.duration_ms),
                throttle_ms: Some(reorder.max_delay),
                limit_kbps: None,
                count: None,
            },
            params: None,
        });
    }

    modules
}
