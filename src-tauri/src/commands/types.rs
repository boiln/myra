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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
    /// Optional drop flag (for throttle - drop buffered packets instead of releasing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drop: Option<bool>,
    /// Optional max buffer size (for throttle)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer: Option<usize>,
    /// MGO2 bypass mode - swap IPs on failed sends
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lag_bypass: Option<bool>,
    /// Freeze mode - disable cooldown for death loop effect (may DC faster)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeze_mode: Option<bool>,
    /// Passthrough threshold - packets smaller than this size pass through (for bandwidth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passthrough_threshold: Option<usize>,
    /// Use WFP (`WinDivert`) token bucket algorithm for precise rate limiting (for bandwidth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_wfp: Option<bool>,
    /// Reverse mode - release packets in reverse order (for reorder/burst)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingStatisticsDto {
    // Burst stats
    pub burst_buffered: usize,
    pub burst_released: usize,
    pub burst_buffered_count: usize,
    // Throttle stats
    pub throttle_buffered_count: usize,
    pub throttle_dropped_count: usize,
    pub throttle_is_throttling: bool,
    // Lag stats
    pub lag_current_lagged: usize,
    // Reorder stats (optional, useful to know queued delayed packets)
    pub reorder_delayed_packets: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStatus {
    /// Whether packet processing is currently running
    pub running: bool,
    /// Current processing statistics, if available
    pub statistics: Option<ProcessingStatisticsDto>,
    /// Configuration of all available modules
    pub modules: Vec<ModuleInfo>,
}
