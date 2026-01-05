use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateOptions {
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

    /// Probability of duplicating packets, ranging from 0.0 to 1.0
    #[arg(long = "duplicate-probability", id = "duplicate-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Number of times to duplicate each packet
    #[arg(long = "duplicate-count", default_value_t = 1, id = "duplicate-count")]
    #[serde(default)]
    pub count: usize,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(
        long = "duplicate-duration",
        id = "duplicate-duration",
        default_value_t = 0
    )]
    #[serde(default)]
    pub duration_ms: u64,
}

impl Default for DuplicateOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            count: 1,
            probability: Probability::default(),
            duration_ms: 0,
        }
    }
}
