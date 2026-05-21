use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
pub struct DropOptions {
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

    /// Probability of dropping packets, ranging from 0.0 to 1.0
    #[arg(long = "drop-probability", id = "drop-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "drop-duration", id = "drop-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,
}
