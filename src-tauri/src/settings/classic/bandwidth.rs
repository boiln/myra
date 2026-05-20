//! Classic Bandwidth module settings.
//!
//! Rate-limits by bytes per second using a token bucket algorithm.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {

    true

}

fn default_chance() -> f64 {

    100.0

}

fn default_limit_kbps() -> f64 {

    115.0

}

fn default_max_buffer() -> usize {

    6000

}

/// Classic Bandwidth module options.
///
/// Rate-limits traffic by bytes per second.
/// Excess packets are buffered up to a limit, then dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicBandwidthOptions {

    /// Whether this module is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound traffic
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound traffic
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Chance (usually 100% for bandwidth limiting)
    #[serde(default = "default_chance")]
    pub chance: f64,

    /// Bandwidth limit in KB/s
    #[serde(default = "default_limit_kbps")]
    pub limit_kbps: f64,

    /// Maximum packets to buffer before dropping
    #[serde(default = "default_max_buffer")]
    pub max_buffer: usize,

}

impl Default for ClassicBandwidthOptions {
    fn default() -> Self {

        Self {

            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100.0,
            limit_kbps: 115.0,
            max_buffer: 6000,

        }

    }
}
