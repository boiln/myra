//! Classic Latency module settings.
//!
//! Holds packets for a fixed duration before releasing them.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_delay() -> u64 {
    100
}

fn default_chance() -> f64 {
    100.0
}

/// Classic Latency module options.
///
/// Unlike Standard lag which applies per-packet delay with probability,
/// Classic latency buffers ALL matching packets and releases them after
/// the delay expires (with optional probability for which packets to affect).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicLatencyOptions {

    /// Whether this module is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound traffic
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound traffic
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Chance to affect each packet (0-100%)
    #[serde(default = "default_chance")]
    pub chance: f64,

    /// Fixed delay in milliseconds (0-15000)
    #[serde(default = "default_delay")]
    pub delay_ms: u64,

}

impl Default for ClassicLatencyOptions {

    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100.0,
            delay_ms: 100,
        }
    }

}
