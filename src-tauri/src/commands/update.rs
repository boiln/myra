//! Settings update commands.
//!
//! Handles updating packet manipulation settings based on module configurations.

use log::info;
use tauri::State;

use crate::commands::state::PacketProcessingState;
use crate::commands::types::ModuleInfo;
use crate::network::types::probability::Probability;
use crate::settings::bandwidth::BandwidthOptions;
use crate::settings::burst::BurstOptions;
use crate::settings::delay::DelayOptions;
use crate::settings::drop::DropOptions;
use crate::settings::duplicate::DuplicateOptions;
use crate::settings::reorder::ReorderOptions;
use crate::settings::tamper::TamperOptions;
use crate::settings::throttle::ThrottleOptions;
use crate::settings::Settings;

/// Updates the packet manipulation settings.
///
/// Updates the settings used for packet manipulation, such as drop rate,
/// latency, throttling, and bandwidth limitations.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
/// * `modules` - List of module configurations to update
///
/// # Returns
///
/// * `Ok(())` - If settings were successfully updated
/// * `Err(String)` - If there was an error updating settings
#[tauri::command]
pub async fn update_settings(
    state: State<'_, PacketProcessingState>,
    modules: Vec<ModuleInfo>,
) -> Result<(), String> {
    let settings = build_settings_from_modules(modules)?;

    let mut state_settings = state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))?;

    *state_settings = settings;

    info!("Settings updated successfully");

    Ok(())
}

/// Builds Settings from a list of ModuleInfo.
fn build_settings_from_modules(modules: Vec<ModuleInfo>) -> Result<Settings, String> {
    let mut settings = Settings::default();

    for module in &modules {
        match module.name.as_str() {
            "drop" => {
                // Always save settings, enabled flag tracks if module is active
                settings.drop = Some(build_drop_options(module)?);
            }
            "delay" => {
                settings.delay = Some(build_delay_options(module)?);
            }
            "throttle" => {
                settings.throttle = Some(build_throttle_options(module)?);
            }
            "duplicate" => {
                settings.duplicate = Some(build_duplicate_options(module)?);
            }
            "bandwidth" => {
                settings.bandwidth = Some(build_bandwidth_options(module)?);
            }
            "tamper" => {
                settings.tamper = Some(build_tamper_options(module)?);
            }
            "reorder" => {
                settings.reorder = Some(build_reorder_options(module)?);
            }
            "burst" => {
                // Always capture release_delay_us at top level so it persists when burst is disabled
                settings.burst_release_delay_us = module.config.release_delay_us.unwrap_or(500);
                // Capture lag_bypass setting if present
                if let Some(lag_bypass) = module.config.lag_bypass {
                    settings.lag_bypass = lag_bypass;
                }
                settings.burst = Some(build_burst_options(module)?);
            }
            _ => {
                return Err(format!("Unknown module: {}", module.name));
            }
        }
    }

    Ok(settings)
}

fn build_drop_options(module: &ModuleInfo) -> Result<DropOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid drop probability: {}", e))?;

    Ok(DropOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        probability,
        duration_ms: module.config.duration_ms.unwrap_or(0),
    })
}

fn build_delay_options(module: &ModuleInfo) -> Result<DelayOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid delay probability: {}", e))?;

    let delay_time = module.config.duration_ms.unwrap_or(1000);
    let delay_time = if delay_time == 0 { 1000 } else { delay_time };

    Ok(DelayOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        delay_ms: delay_time,
        probability,
        duration_ms: 0,
    })
}

fn build_throttle_options(module: &ModuleInfo) -> Result<ThrottleOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid throttle probability: {}", e))?;

    Ok(ThrottleOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        probability,
        throttle_ms: module.config.throttle_ms.unwrap_or(300),
        duration_ms: module.config.duration_ms.unwrap_or(0),
        drop: module.config.drop.unwrap_or(false),
        max_buffer: module.config.max_buffer.unwrap_or(2000),
        freeze_mode: module.config.freeze_mode.unwrap_or(false),
    })
}

fn build_duplicate_options(module: &ModuleInfo) -> Result<DuplicateOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid duplicate probability: {}", e))?;

    Ok(DuplicateOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        probability,
        count: module.config.count.unwrap_or(1),
        duration_ms: module.config.duration_ms.unwrap_or(0),
    })
}

fn build_bandwidth_options(module: &ModuleInfo) -> Result<BandwidthOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid bandwidth probability: {}", e))?;

    let limit = module.config.limit_kbps.unwrap_or(0) as usize;

    Ok(BandwidthOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        limit,
        probability,
        duration_ms: module.config.duration_ms.unwrap_or(0),
        passthrough_threshold: module.config.passthrough_threshold.unwrap_or(200),
        use_wfp: module.config.use_wfp.unwrap_or(false),
    })
}

fn build_tamper_options(module: &ModuleInfo) -> Result<TamperOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid tamper probability: {}", e))?;

    let amount = Probability::new(0.5).unwrap_or_default();

    Ok(TamperOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        probability,
        amount,
        duration_ms: module.config.duration_ms.unwrap_or(0),
        recalculate_checksums: Some(true),
    })
}

fn build_reorder_options(module: &ModuleInfo) -> Result<ReorderOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid reorder probability: {}", e))?;

    Ok(ReorderOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        probability,
        max_delay: module.config.throttle_ms.unwrap_or(100),
        duration_ms: module.config.duration_ms.unwrap_or(0),
    })
}

fn build_burst_options(module: &ModuleInfo) -> Result<BurstOptions, String> {
    let probability = Probability::new(module.config.chance / 100.0)
        .map_err(|e| format!("Invalid burst probability: {}", e))?;

    Ok(BurstOptions {
        enabled: module.enabled,
        inbound: module.config.inbound,
        outbound: module.config.outbound,
        probability,
        buffer_ms: module.config.buffer_ms.unwrap_or(0),
        duration_ms: module.config.duration_ms.unwrap_or(0),
        keepalive_ms: module.config.keepalive_ms.unwrap_or(0),
        release_delay_us: module.config.release_delay_us.unwrap_or(500),
    })
}
