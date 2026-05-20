//! Classic Throttle module settings.
//!
//! Buffers packets for a time window, then releases or drops them.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {

    true

}

fn default_chance() -> f64 {

    10.0

}

fn default_window_ms() -> u64 {

    30

}

fn default_max_buffer() -> usize {

    1000

}

/// Classic Throttle module options.
///
/// Buffers packets during a time window, then either releases them
/// all at once (burst) or drops them entirely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicThrottleOptions {

    /// Whether this module is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound traffic
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound traffic
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Chance to start a throttle window (0-100%)
    #[serde(default = "default_chance")]
    pub chance: f64,

    /// Time window for buffering in milliseconds (0-1000)
    #[serde(default = "default_window_ms")]
    pub window_ms: u64,

    /// If true, DROP buffered packets; if false, RELEASE as burst
    #[serde(default)]
    pub drop_on_release: bool,

    /// Maximum packets to buffer before forced flush
    #[serde(default = "default_max_buffer")]
    pub max_buffer: usize,

}

impl Default for ClassicThrottleOptions {
    fn default() -> Self {

        Self {

            enabled: false,
            inbound: true,
            outbound: true,
            chance: 10.0,
            window_ms: 30,
            drop_on_release: false,
            max_buffer: 1000,

        }

    }
}
