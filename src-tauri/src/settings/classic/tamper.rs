//! Classic Tamper module settings.
//!
//! XORs packet payload data with a rotating pattern.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {

    true

}

fn default_chance() -> f64 {

    10.0

}

/// Classic Tamper module options.
///
/// Corrupts packet payload by XORing with a rotating pattern.
/// Small packets (<5 bytes) get entire payload tampered.
/// Larger packets get ~25% of middle section tampered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicTamperOptions {

    /// Whether this module is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound traffic
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound traffic
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Chance to tamper each packet (0-100%)
    #[serde(default = "default_chance")]
    pub chance: f64,

    /// Whether to recalculate checksums after tampering
    #[serde(default = "default_true")]
    pub recalc_checksum: bool,

}

impl Default for ClassicTamperOptions {
    fn default() -> Self {

        Self {

            enabled: false,
            inbound: true,
            outbound: true,
            chance: 10.0,
            recalc_checksum: true,

        }

    }
}
