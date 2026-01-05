use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

fn default_max_buffer() -> usize {
    2000
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct ThrottleOptions {
    /// Whether this module is enabled
    #[arg(skip)]
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound (download) traffic
    #[arg(skip)]
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound (upload) traffic
    #[arg(skip)]
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Probability of triggering a throttle event, ranging from 0.0 to 1.0
    #[arg(long = "throttle-probability", id = "throttle-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Timeframe in milliseconds - how long to buffer packets before releasing/dropping
    /// This is the "lag window" duration
    #[arg(long = "throttle-ms", default_value_t = 300, id = "throttle-ms")]
    #[serde(default = "default_throttle_ms")]
    pub throttle_ms: u64,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(
        long = "throttle-duration",
        id = "throttle-duration",
        default_value_t = 0
    )]
    #[serde(default)]
    pub duration_ms: u64,

    /// If true, DROP all buffered packets when timeframe ends
    /// If false, RELEASE all buffered packets when timeframe ends
    #[arg(long = "throttle-drop", default_value_t = false, id = "throttle-drop")]
    #[serde(default)]
    pub drop: bool,

    /// Maximum number of packets to buffer (default 2000)
    /// When buffer is full, triggers immediate release/drop
    #[arg(long = "throttle-max-buffer", default_value_t = 2000, id = "throttle-max-buffer")]
    #[serde(default = "default_max_buffer")]
    pub max_buffer: usize,

    /// Freeze mode - disables cooldown gap between throttle cycles
    /// When true: Continuous buffering for freeze effect (may disconnect faster)
    /// When false: Normal mode with cooldown between cycles (more stable)
    #[arg(long = "throttle-freeze-mode", default_value_t = false, id = "throttle-freeze-mode")]
    #[serde(default)]
    pub freeze_mode: bool,
}

fn default_throttle_ms() -> u64 {
    300
}

impl Default for ThrottleOptions {
    fn default() -> Self {
        ThrottleOptions {
            enabled: false,
            inbound: true,
            outbound: true,
            probability: Probability::default(),
            throttle_ms: 300,
            duration_ms: 0,
            drop: false,
            max_buffer: 2000,
            freeze_mode: false,
        }
    }
}
