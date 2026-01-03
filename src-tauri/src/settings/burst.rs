use crate::network::types::probability::Probability;
use clap::Parser;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_replay_speed() -> f64 {
    1.0
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct BurstOptions {
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

    /// Buffer time in milliseconds - how long to hold packets before releasing
    /// 0 = manual mode (hold until disabled)
    #[arg(long = "burst-buffer-ms", id = "burst-buffer-ms", default_value_t = 0)]
    #[serde(default)]
    pub buffer_ms: u64,

    /// Probability of buffering packets, ranging from 0.0 to 1.0
    #[arg(long = "burst-probability", id = "burst-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "burst-duration", id = "burst-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,

    /// Replay speed multiplier when releasing packets:
    /// - 1.0 = real-time (packets released at same rate they were captured)
    /// - 2.0 = 2x speed (faster replay, shorter teleport)
    /// - 0.5 = half speed (slower replay, longer rubber-band)
    /// - 0.0 = instant release (all packets at once)
    #[arg(long = "burst-replay-speed", id = "burst-replay-speed", default_value_t = 1.0)]
    #[serde(default = "default_replay_speed")]
    pub replay_speed: f64,

    /// If true, release packets in reverse order (LIFO instead of FIFO)
    /// Creates a "rewind" effect where recent actions play out first
    #[arg(long = "burst-reverse", id = "burst-reverse", default_value_t = false)]
    #[serde(default)]
    pub reverse_replay: bool,
}

impl Default for BurstOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            buffer_ms: 0,
            probability: Probability::default(),
            duration_ms: 0,
            replay_speed: 1.0,
            reverse_replay: false,
        }
    }
}
