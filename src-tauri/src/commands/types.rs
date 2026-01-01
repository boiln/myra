//! Type definitions for command responses.
//!
//! This module contains the data structures used for communication
//! between the Tauri frontend and backend.

use serde::{Deserialize, Serialize};

/// Information about a network condition simulation module.
///
/// Contains the configuration, state, and parameters for a specific
/// network condition simulation module (e.g., lag, drop, throttle).
#[derive(Debug, Serialize, Deserialize, Clone)]
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

/// Configuration for a network condition simulation module.
///
/// Contains settings that control how a module behaves,
/// including which directions to affect and the probability of action.
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    /// Optional buffer time in milliseconds (for burst)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_ms: Option<u64>,
    /// Optional keepalive interval in milliseconds (for burst)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepalive_ms: Option<u64>,
    /// Optional release delay in microseconds (for burst replay speed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_delay_us: Option<u64>,
}

/// Additional parameters for a network condition simulation module.
///
/// Contains module-specific parameters that don't fit into the standard config.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleParams {
    /// Optional delay time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lag_time: Option<u64>,
}

/// Status information about the packet processing engine.
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
