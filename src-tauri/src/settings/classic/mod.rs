//! Classic mode settings for timer-based network manipulation.
//!
//! Classic mode provides deterministic, timer-based manipulation as opposed
//! to Standard mode's probabilistic per-packet approach.
pub mod latency;
pub mod drop;
pub mod throttle;
pub mod reorder;
pub mod tamper;
pub mod bandwidth;

pub use latency::ClassicLatencyOptions;
pub use drop::ClassicDropOptions;
pub use throttle::ClassicThrottleOptions;
pub use reorder::ClassicReorderOptions;
pub use tamper::ClassicTamperOptions;
pub use bandwidth::ClassicBandwidthOptions;

use serde::{Deserialize, Serialize};

/// All Classic mode settings combined.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassicSettings {
    /// Latency module - holds packets for fixed duration
    #[serde(default)]
    pub latency: Option<ClassicLatencyOptions>,

    /// Drop module - probabilistic packet dropping
    #[serde(default)]
    pub drop: Option<ClassicDropOptions>,

    /// Throttle module - buffer then release/drop
    #[serde(default)]
    pub throttle: Option<ClassicThrottleOptions>,

    /// Reorder module - swap adjacent packets
    #[serde(default)]
    pub reorder: Option<ClassicReorderOptions>,

    /// Tamper module - corrupt packet data
    #[serde(default)]
    pub tamper: Option<ClassicTamperOptions>,

    /// Bandwidth module - rate limiting
    #[serde(default)]
    pub bandwidth: Option<ClassicBandwidthOptions>,
}

impl ClassicSettings {
    /// Returns true if any classic module is enabled.
    pub fn has_any_enabled(&self) -> bool {

        self.latency.as_ref().is_some_and(|o| o.enabled)
            || self.drop.as_ref().is_some_and(|o| o.enabled)
            || self.throttle.as_ref().is_some_and(|o| o.enabled)
            || self.reorder.as_ref().is_some_and(|o| o.enabled)
            || self.tamper.as_ref().is_some_and(|o| o.enabled)
            || self.bandwidth.as_ref().is_some_and(|o| o.enabled)

    }
}
