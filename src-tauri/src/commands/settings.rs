use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::PacketProcessingState;
use crate::network::processing::packet_processing::start_packet_processing;
use crate::network::processing::packet_receiving::receive_packets;
use crate::network::types::probability::Probability;
use crate::settings::bandwidth::BandwidthOptions;
use crate::settings::delay::DelayOptions;
use crate::settings::drop::DropOptions;
use crate::settings::duplicate::DuplicateOptions;
use crate::settings::packet_manipulation::PacketManipulationSettings;
use crate::settings::reorder::ReorderOptions;
use crate::settings::tamper::TamperOptions;
use crate::settings::throttle::ThrottleOptions;

/// Information about a network condition simulation module
///
/// Contains the configuration, state, and parameters for a specific
/// network condition simulation module (e.g., lag, drop, throttle).
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Internal identifier for the module
    pub name: String,
    /// User-friendly display name
    pub display_name: String,
    /// Whether the module is enabled
    pub enabled: bool,
    /// Module configuration settings
    pub config: ModuleConfig,
    /// Additional module-specific parameters
    pub params: Option<ModuleParams>,
}

/// Configuration for a network condition simulation module
///
/// Contains settings that control how a module behaves,
/// including which directions to affect and the probability of action.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Whether to apply to inbound traffic
    pub inbound: bool,
    /// Whether to apply to outbound traffic
    pub outbound: bool,
    /// Probability of applying the effect (0.0-100.0%)
    pub chance: f64,
    /// Whether the module is enabled
    pub enabled: bool,
    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Optional throttle time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle_ms: Option<u64>,
    /// Optional bandwidth limit in KB/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_kbps: Option<u64>,
    /// Optional count parameter (e.g., for duplication)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

/// Additional parameters for a network condition simulation module
///
/// Contains module-specific parameters that don't fit into the standard config.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleParams {
    /// Optional delay time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lag_time: Option<u64>,
}

/// Status information about the packet processing engine
///
/// Contains the current state of the packet processing engine,
/// including whether it's running, statistics, and module configurations.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStatus {
    /// Whether packet processing is currently running
    pub running: bool,
    /// Current processing statistics, if available
    pub statistics: Option<String>,
    /// Configuration of all available modules
    pub modules: Vec<ModuleInfo>,
}

/// Starts packet processing with the given settings and filter
///
/// Creates and launches the packet receiving and processing threads
/// that will intercept and modify network packets according to the settings.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
/// * `settings` - The packet manipulation settings to apply
/// * `filter` - Optional WinDivert filter expression to select packets
///
/// # Returns
///
/// * `Ok(())` - If processing was started successfully
/// * `Err(String)` - If there was an error starting processing
#[tauri::command]
pub async fn start_processing(
    state: State<'_, PacketProcessingState>,
    settings: PacketManipulationSettings,
    filter: Option<String>,
) -> Result<(), String> {
    let running = state.running.load(Ordering::SeqCst);
    if running {
        return Err("Packet processing already running".to_string());
    }

    // Update settings and filter
    *state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))? = settings;
    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = filter;

    // Create channel for packet communication
    let (packet_sender, packet_receiver) = mpsc::channel();

    // Start packet receiving thread
    let running_recv = state.running.clone();
    let settings_recv = state.settings.clone();
    let filter_recv = state.filter.clone();

    thread::spawn(move || {
        if let Err(e) = receive_packets(packet_sender, running_recv, settings_recv, filter_recv) {
            error!("Packet receiving error: {}", e);
        }
    });

    // Start packet processing thread
    let running_proc = state.running.clone();
    let settings_proc = state.settings.clone();
    let statistics = state.statistics.clone();

    thread::spawn(move || {
        if let Err(e) =
            start_packet_processing(settings_proc, packet_receiver, running_proc, statistics)
        {
            error!("Packet processing error: {}", e);
        }
    });

    state.running.store(true, Ordering::SeqCst);
    info!("Started packet processing");
    Ok(())
}

/// Stops packet processing
///
/// Signals the packet processing and receiving threads to shut down
/// and waits a short time for them to clean up resources.
///
/// # Arguments
///
/// * `state` - The application state containing shared resources
///
/// # Returns
///
/// * `Ok(())` - If processing was stopped successfully
/// * `Err(String)` - If there was an error stopping processing
#[tauri::command]
pub async fn stop_processing(state: State<'_, PacketProcessingState>) -> Result<(), String> {
    if !state.running.load(Ordering::SeqCst) {
        return Err("Packet processing not running".to_string());
    }

    // First reset filter to ensure no new packets are captured
    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = None;

    // Give a small time for filter change to propagate
    thread::sleep(Duration::from_millis(100));

    // Signal threads to stop
    state.running.store(false, Ordering::SeqCst);

    // Give threads time to properly close their WinDivert handles and flush caches
    thread::sleep(Duration::from_millis(500));

    // Try to force a final WFP cache flush using WinDivert directly
    use windivert::layer::NetworkLayer;
    use windivert::prelude::WinDivertFlags;
    use windivert::CloseAction;
    use windivert::WinDivert;

    // Multiple attempts to flush cache with different priorities
    for priority in [0, 1000, -1000] {
        if let Ok(mut handle) = WinDivert::<NetworkLayer>::network(
            "false", // Filter that matches nothing
            priority,
            WinDivertFlags::new(),
        ) {
            let _ = handle.close(CloseAction::Nothing);
            debug!("Successfully flushed WFP cache with priority {}", priority);
        }
    }

    info!("Stopped packet processing and cleaned up resources");
    Ok(())
}

/// Gets the current status of the processing engine
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
    let mut modules = Vec::new();

    // Drop module
    if let Some(drop) = &settings.drop {
        modules.push(ModuleInfo {
            name: "drop".to_string(),
            display_name: "Drop".to_string(),
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

    // Freeze module (was Delay)
    if let Some(delay) = &settings.delay {
        modules.push(ModuleInfo {
            name: "delay".to_string(),
            display_name: "Freeze".to_string(),
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

    Ok(ProcessingStatus {
        running,
        statistics,
        modules,
    })
}

/// Updates the packet manipulation settings
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
    // Create new packet manipulation settings
    let mut settings = PacketManipulationSettings::default();

    // Process each module and update the corresponding settings
    for module in modules {
        match module.name.as_str() {
            "drop" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid drop probability: {}", e))?;

                settings.drop = Some(DropOptions {
                    probability,
                    duration_ms: module.config.duration_ms.unwrap_or(0),
                });
            }
            "delay" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid delay probability: {}", e))?;

                // For Freeze module: duration_ms is the freeze time (how long to hold packets)
                // The effect duration is always infinite (0)
                let freeze_time = module.config.duration_ms.unwrap_or(1000);
                let freeze_time = if freeze_time == 0 { 1000 } else { freeze_time };

                settings.delay = Some(DelayOptions {
                    delay_ms: freeze_time,
                    probability,
                    duration_ms: 0, // Always infinite
                });
            }
            "throttle" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid throttle probability: {}", e))?;

                settings.throttle = Some(ThrottleOptions {
                    probability,
                    throttle_ms: module.config.throttle_ms.unwrap_or(30),
                    duration_ms: module.config.duration_ms.unwrap_or(0),
                    drop: false, // Default to false, can be updated if needed
                });
            }
            "duplicate" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid duplicate probability: {}", e))?;

                settings.duplicate = Some(DuplicateOptions {
                    probability,
                    count: module.config.count.unwrap_or(1),
                    duration_ms: module.config.duration_ms.unwrap_or(0),
                });
            }
            "bandwidth" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid bandwidth probability: {}", e))?;

                let limit = module.config.limit_kbps.unwrap_or(0) as usize;

                settings.bandwidth = Some(BandwidthOptions {
                    limit,
                    probability,
                    duration_ms: module.config.duration_ms.unwrap_or(0),
                });
            }
            "tamper" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid tamper probability: {}", e))?;

                // Default amount to 10% if not specified
                let amount = Probability::new(0.1).unwrap_or_else(|_| Probability::default());

                settings.tamper = Some(TamperOptions {
                    probability,
                    amount,
                    duration_ms: module.config.duration_ms.unwrap_or(0),
                    recalculate_checksums: Some(true),
                });
            }
            "reorder" => {
                let probability = Probability::new(module.config.chance / 100.0)
                    .map_err(|e| format!("Invalid reorder probability: {}", e))?;

                settings.reorder = Some(ReorderOptions {
                    probability,
                    max_delay: module.config.throttle_ms.unwrap_or(100),
                    duration_ms: module.config.duration_ms.unwrap_or(0),
                });
            }
            _ => {
                return Err(format!("Unknown module: {}", module.name));
            }
        }
    }

    // Update the settings in the application state
    let mut state_settings = state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))?;
    *state_settings = settings;

    info!("Settings updated successfully");
    Ok(())
}

/// Gets the current packet manipulation settings
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

/// Updates the WinDivert filter expression
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

/// Gets the current WinDivert filter expression
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
