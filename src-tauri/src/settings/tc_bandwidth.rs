//! Settings for Traffic Control (NetLimiter-style) bandwidth limiting
//!
//! This provides true OS-level bandwidth limiting that operates at the
//! socket layer, like NetLimiter does.

use serde::{Deserialize, Serialize};

/// Direction for TC bandwidth limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TcDirection {
    /// Limit inbound (download) traffic only
    Inbound,
    /// Limit outbound (upload) traffic only
    Outbound,
    /// Limit both directions
    #[default]
    Both,
}

/// Settings for Traffic Control bandwidth limiting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TcBandwidthOptions {
    /// Whether TC bandwidth limiting is enabled
    #[serde(default)]
    pub enabled: bool,
    
    /// Bandwidth limit in KB/s (like NetLimiter)
    /// Default: 1 KB/s
    #[serde(default = "default_limit")]
    pub limit_kbps: u32,
    
    /// Direction to limit
    #[serde(default)]
    pub direction: TcDirection,
}

fn default_limit() -> u32 {
    1 // 1 KB/s default, like NetLimiter
}

impl TcBandwidthOptions {
    pub fn new(limit_kbps: u32, direction: TcDirection) -> Self {
        Self {
            enabled: true,
            limit_kbps,
            direction,
        }
    }
}


