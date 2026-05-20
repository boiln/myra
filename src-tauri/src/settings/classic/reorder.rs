//! Classic Reorder module settings.
//!
//! Swaps adjacent packets to create out-of-order delivery.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {

    true

}

fn default_chance() -> f64 {

    10.0

}

fn default_max_hold_cycles() -> u32 {

    10

}

/// Classic Reorder module options.
///
/// Swaps adjacent packets to create out-of-order delivery.
/// Can hold a single packet for up to N cycles waiting for more packets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicReorderOptions {

    /// Whether this module is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound traffic
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound traffic
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Chance to swap packets (0-100%)
    #[serde(default = "default_chance")]
    pub chance: f64,

    /// How many cycles to hold a lone packet before releasing
    #[serde(default = "default_max_hold_cycles")]
    pub max_hold_cycles: u32,

}

impl Default for ClassicReorderOptions {
    fn default() -> Self {

        Self {

            enabled: false,
            inbound: true,
            outbound: true,
            chance: 10.0,
            max_hold_cycles: 10,

        }

    }
}
