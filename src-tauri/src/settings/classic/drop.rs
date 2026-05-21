//! Classic Drop module settings.
//!
//! Probabilistically drops packets immediately.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_chance() -> f64 {
    10.0
}

/// Classic Drop module options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicDropOptions {
    /// Whether this module is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Whether to apply to inbound traffic
    #[serde(default = "default_true")]
    pub inbound: bool,

    /// Whether to apply to outbound traffic
    #[serde(default = "default_true")]
    pub outbound: bool,

    /// Chance to drop each packet (0-100%)
    #[serde(default = "default_chance")]
    pub chance: f64,
}

impl Default for ClassicDropOptions {
    fn default() -> Self {

        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 10.0,
        }

    }
}
