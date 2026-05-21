use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Default, Clone)]
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

    /// Keepalive interval in milliseconds - lets one packet through periodically
    /// to prevent disconnection. 0 = disabled (buffer everything)
    #[arg(long = "burst-keepalive-ms", id = "burst-keepalive-ms", default_value_t = 0)]
    #[serde(default)]
    pub keepalive_ms: u64,

    /// Delay between packets when releasing in microseconds.
    /// Controls replay speed - higher = slower/longer replay.
    /// Default 500us (0.5ms). Set to 0 for instant release.
    #[arg(long = "burst-release-delay-us", id = "burst-release-delay-us", default_value_t = 500)]
    #[serde(default = "default_release_delay")]
    pub release_delay_us: u64,

    /// Reverse mode - release packets in reverse order when buffer releases
    /// Creates a "rewind" effect where actions appear backwards
    #[arg(long = "burst-reverse", id = "burst-reverse", default_value_t = false)]
    #[serde(default)]
    pub reverse: bool,
}

fn default_release_delay() -> u64 {
    500
}
