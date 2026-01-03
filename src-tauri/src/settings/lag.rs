use crate::network::types::probability::Probability;
use clap::Parser;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_probability_100() -> Probability {
    Probability::new(1.0).unwrap()
}

/// Options for the Lag module.
/// 
/// This module lags packets (matching direction criteria) by a fixed time,
/// creating a true network latency effect. By default, probability is 100%
/// so all matching traffic is lagged.
#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct LagOptions {
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

    /// Lag time in milliseconds to introduce for each packet
    #[arg(long = "lag-ms", id = "lag-ms", default_value_t = 0)]
    #[serde(default)]
    pub lag_ms: u64,

    /// Probability of lagging packets, ranging from 0.0 to 1.0 (default 1.0 = 100%)
    #[arg(long = "lag-probability", id = "lag-probability", default_value_t = default_probability_100())]
    #[serde(default = "default_probability_100")]
    pub probability: Probability,

    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "lag-duration", id = "lag-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,
}

impl Default for LagOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            lag_ms: 0,
            probability: default_probability_100(),
            duration_ms: 0,
        }
    }
}
