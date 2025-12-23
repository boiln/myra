use clap::Parser;
use serde::{Deserialize, Serialize};
use crate::network::types::probability::Probability;

#[derive(Parser, Debug, Serialize, Deserialize, Default, Clone)]
pub struct BandwidthOptions {
    /// Maximum bandwidth limit in KB/s
    #[arg(long = "bandwidth-limit", id = "bandwidth-limit", default_value_t = 0)]
    #[serde(default)]
    pub limit: usize,
    
    /// Probability of applying bandwidth limitation, ranging from 0.0 to 1.0
    #[arg(long = "bandwidth-probability", id = "bandwidth-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,
    
    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "bandwidth-duration", id = "bandwidth-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,
}
