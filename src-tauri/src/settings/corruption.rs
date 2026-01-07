use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct CorruptionOptions {
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

    /// Probability of corruptioning packets, ranging from 0.0 to 1.0
    #[arg(long = "corruption-probability", id = "corruption-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Amount of corruptioning that should be applied, ranging from 0.0 to 1.0
    #[arg(long = "corruption-amount", default_value_t = Probability::new(0.1).unwrap(), id = "corruption-amount")]
    #[serde(default)]
    pub amount: Probability,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "corruption-duration", id = "corruption-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,

    /// Whether corruptioned packets should have their checksums recalculated to mask the corruptioning and avoid the packets getting automatically dropped
    #[arg(
        long = "corruption-recalculate-checksums",
        id = "corruption-recalculate-checksums"
    )]
    #[serde(default)]
    pub recalculate_checksums: Option<bool>,
}

impl Default for CorruptionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            probability: Probability::default(),
            amount: Probability::new(0.1).unwrap(),
            duration_ms: 0,
            recalculate_checksums: Some(true),
        }
    }
}
