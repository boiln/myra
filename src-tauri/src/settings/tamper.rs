use crate::network::types::probability::Probability;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct TamperOptions {
    /// Probability of tampering packets, ranging from 0.0 to 1.0
    #[arg(long = "tamper-probability", id = "tamper-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Amount of tampering that should be applied, ranging from 0.0 to 1.0
    #[arg(long = "tamper-amount", default_value_t = Probability::new(0.1).unwrap(), id = "tamper-amount")]
    #[serde(default)]
    pub amount: Probability,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "tamper-duration", id = "tamper-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,

    /// Whether tampered packets should have their checksums recalculated to mask the tampering and avoid the packets getting automatically dropped
    #[arg(
        long = "tamper-recalculate-checksums",
        id = "tamper-recalculate-checksums"
    )]
    #[serde(default)]
    pub recalculate_checksums: Option<bool>,
}

impl Default for TamperOptions {
    fn default() -> Self {
        TamperOptions {
            probability: Probability::default(),
            amount: Probability::new(0.1).unwrap(),
            duration_ms: 0,
            recalculate_checksums: Some(true),
        }
    }
}
