use crate::network::types::probability::Probability;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct ThrottleOptions {
    /// Probability of triggering a throttle event, ranging from 0.0 to 1.0
    #[arg(long = "throttle-probability", id = "throttle-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,

    /// Duration in milliseconds for each throttling period
    #[arg(
        long = "throttle-ms",
        default_value_t = 30,
        id = "throttle-ms"
    )]
    #[serde(default)]
    pub throttle_ms: u64,
    
    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "throttle-duration", id = "throttle-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,

    /// Indicates whether throttled packets should be dropped
    #[arg(long = "throttle-drop", default_value_t = false, id = "throttle-drop")]
    #[serde(default)]
    pub drop: bool,
}

impl Default for ThrottleOptions {
    fn default() -> Self {
        ThrottleOptions {
            probability: Probability::default(),
            throttle_ms: 30,
            duration_ms: 0,
            drop: false,
        }
    }
}
