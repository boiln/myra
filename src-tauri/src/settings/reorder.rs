use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct ReorderOptions {
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

    /// Probability of reordering packets, ranging from 0.0 to 1.0
    #[arg(long = "reorder-probability", id = "reorder-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,
    /// Maximum random delay in milliseconds to apply when reordering packets
    #[arg(
        long = "reorder-max-delay",
        id = "reorder-max-delay",
        default_value_t = 100
    )]
    #[serde(default)]
    pub max_delay: u64,
    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(
        long = "reorder-duration",
        id = "reorder-duration",
        default_value_t = 0
    )]
    #[serde(default)]
    pub duration_ms: u64,
}

impl Default for ReorderOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            probability: Probability::default(),
            max_delay: 100,
            duration_ms: 0,
        }
    }
}
