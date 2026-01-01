use crate::network::types::probability::Probability;
use clap::Parser;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Parser, Debug, Serialize, Deserialize, Default, Clone)]
pub struct BandwidthOptions {
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

    /// Maximum bandwidth limit in KB/s
    #[arg(long = "bandwidth-limit", id = "bandwidth-limit", default_value_t = 0)]
    #[serde(default)]
    pub limit: usize,

    /// Probability of applying bandwidth limitation, ranging from 0.0 to 1.0
    #[arg(long = "bandwidth-probability", id = "bandwidth-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(
        long = "bandwidth-duration",
        id = "bandwidth-duration",
        default_value_t = 0
    )]
    #[serde(default)]
    pub duration_ms: u64,
}
