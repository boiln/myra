use clap::Parser;
use serde::{Deserialize, Serialize};
use crate::network::types::probability::Probability;

#[derive(Parser, Debug, Serialize, Deserialize, Default, Clone)]
pub struct DelayOptions {
    /// Delay in milliseconds to introduce for each packet
    #[arg(long = "delay-ms", id = "delay-ms", default_value_t = 0)]
    #[serde(default)]
    pub delay_ms: u64,
    
    /// Probability of delaying packets, ranging from 0.0 to 1.0
    #[arg(long = "delay-probability", id = "delay-probability", default_value_t = Probability::default())]
    #[serde(default)]
    pub probability: Probability,
    
    /// Duration for which the effect is applied in milliseconds (0 = infinite)
    #[arg(long = "delay-duration", id = "delay-duration", default_value_t = 0)]
    #[serde(default)]
    pub duration_ms: u64,
}
