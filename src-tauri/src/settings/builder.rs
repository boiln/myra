//! Packet manipulation settings.
//!
//! This module provides a fluent builder API for constructing
//! `Settings` in a type-safe and ergonomic way.
//!
//! # Example
//!
//! ```rust
//! use crate::settings::builder::SettingsBuilder;
//!
//! let settings = SettingsBuilder::new()
//!     .drop(50.0)  // 50% drop rate
//!     .lag(100)  // 100ms lag
//!     .with_lag_chance(75.0)  // 75% chance
//!     .throttle(30)  // 30ms throttle
//!     .build();
//! ```

use crate::network::types::probability::Probability;
use crate::settings::bandwidth::BandwidthOptions;
use crate::settings::lag::LagOptions;
use crate::settings::drop::DropOptions;
use crate::settings::duplicate::DuplicateOptions;
use crate::settings::manipulation::Settings;
use crate::settings::reorder::ReorderOptions;
use crate::settings::tamper::TamperOptions;
use crate::settings::throttle::ThrottleOptions;

/// Builder for constructing `Settings`.
///
/// Provides a fluent API for configuring network condition simulations.
#[derive(Debug, Default)]
pub struct SettingsBuilder {
    settings: Settings,
}

impl SettingsBuilder {
    /// Creates a new builder with default (empty) settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables packet dropping with the given probability percentage (0-100).
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn drop(mut self, chance: f64) -> Self {
        let probability = Probability::new(chance / 100.0).unwrap_or_default();
        self.settings.drop = Some(DropOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            probability,
            duration_ms: 0,
        });
        self
    }

    /// Sets the duration for the drop effect.
    ///
    /// # Arguments
    ///
    /// * `duration_ms` - Duration in milliseconds (0 = infinite)
    pub fn with_drop_duration(mut self, duration_ms: u64) -> Self {
        if let Some(ref mut drop) = self.settings.drop {
            drop.duration_ms = duration_ms;
        }
        self
    }

    /// Enables packet lag with the given lag time.
    ///
    /// # Arguments
    ///
    /// * `delay_ms` - Lag in milliseconds
    pub fn lag(mut self, delay_ms: u64) -> Self {
        self.settings.lag = Some(LagOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            delay_ms,
            probability: Probability::new(1.0).unwrap_or_default(),
            duration_ms: 0,
        });
        self
    }

    /// Sets the probability for the lag effect.
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn with_lag_chance(mut self, chance: f64) -> Self {
        if let Some(ref mut lag) = self.settings.lag {
            lag.probability = Probability::new(chance / 100.0).unwrap_or_default();
        }
        self
    }

    /// Sets the duration for the lag effect.
    ///
    /// # Arguments
    ///
    /// * `duration_ms` - Duration in milliseconds (0 = infinite)
    pub fn with_lag_duration(mut self, duration_ms: u64) -> Self {
        if let Some(ref mut lag) = self.settings.lag {
            lag.duration_ms = duration_ms;
        }
        self
    }

    /// Enables network throttling with the given throttle time.
    ///
    /// # Arguments
    ///
    /// * `throttle_ms` - Throttle time in milliseconds
    pub fn throttle(mut self, throttle_ms: u64) -> Self {
        self.settings.throttle = Some(ThrottleOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            probability: Probability::new(1.0).unwrap_or_default(),
            throttle_ms,
            duration_ms: 0,
            drop: false,
            max_buffer: 2000,
            freeze_mode: false,
        });
        self
    }

    /// Sets the probability for the throttle effect.
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn with_throttle_chance(mut self, chance: f64) -> Self {
        if let Some(ref mut throttle) = self.settings.throttle {
            throttle.probability = Probability::new(chance / 100.0).unwrap_or_default();
        }
        self
    }

    /// Sets whether throttling should drop packets instead of delaying.
    ///
    /// # Arguments
    ///
    /// * `drop` - Whether to drop packets
    pub fn with_throttle_drop(mut self, drop: bool) -> Self {
        if let Some(ref mut throttle) = self.settings.throttle {
            throttle.drop = drop;
        }
        self
    }

    /// Enables packet reordering with the given max delay.
    ///
    /// # Arguments
    ///
    /// * `max_delay_ms` - Maximum reorder delay in milliseconds
    pub fn reorder(mut self, max_delay_ms: u64) -> Self {
        self.settings.reorder = Some(ReorderOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            probability: Probability::new(1.0).unwrap_or_default(),
            max_delay: max_delay_ms,
            duration_ms: 0,
            reverse: false,
        });
        self
    }

    /// Sets the probability for the reorder effect.
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn with_reorder_chance(mut self, chance: f64) -> Self {
        if let Some(ref mut reorder) = self.settings.reorder {
            reorder.probability = Probability::new(chance / 100.0).unwrap_or_default();
        }
        self
    }

    /// Enables packet tampering/corruption.
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn tamper(mut self, chance: f64) -> Self {
        self.settings.tamper = Some(TamperOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            probability: Probability::new(chance / 100.0).unwrap_or_default(),
            amount: Probability::new(0.5).unwrap_or_default(),
            duration_ms: 0,
            recalculate_checksums: Some(true),
        });
        self
    }

    /// Sets the corruption amount for tampering.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of corruption as percentage (0.0 to 100.0)
    pub fn with_tamper_amount(mut self, amount: f64) -> Self {
        if let Some(ref mut tamper) = self.settings.tamper {
            tamper.amount = Probability::new(amount / 100.0).unwrap_or_default();
        }
        self
    }

    /// Sets whether to recalculate checksums after tampering.
    ///
    /// # Arguments
    ///
    /// * `recalculate` - Whether to recalculate checksums
    pub fn with_tamper_checksums(mut self, recalculate: bool) -> Self {
        if let Some(ref mut tamper) = self.settings.tamper {
            tamper.recalculate_checksums = Some(recalculate);
        }
        self
    }

    /// Enables packet duplication.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of duplicates to create
    pub fn duplicate(mut self, count: usize) -> Self {
        self.settings.duplicate = Some(DuplicateOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            probability: Probability::new(1.0).unwrap_or_default(),
            count,
            duration_ms: 0,
        });
        self
    }

    /// Sets the probability for the duplicate effect.
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn with_duplicate_chance(mut self, chance: f64) -> Self {
        if let Some(ref mut duplicate) = self.settings.duplicate {
            duplicate.probability = Probability::new(chance / 100.0).unwrap_or_default();
        }
        self
    }

    /// Enables bandwidth limitation.
    ///
    /// # Arguments
    ///
    /// * `limit_kbps` - Bandwidth limit in KB/s
    pub fn bandwidth(mut self, limit_kbps: usize) -> Self {
        self.settings.bandwidth = Some(BandwidthOptions {
            enabled: true,
            inbound: true,
            outbound: true,
            limit: limit_kbps,
            probability: Probability::new(1.0).unwrap_or_default(),
            duration_ms: 0,
            passthrough_threshold: 200,
            use_wfp: false,
        });
        self
    }

    /// Sets the probability for the bandwidth effect.
    ///
    /// # Arguments
    ///
    /// * `chance` - Probability as percentage (0.0 to 100.0)
    pub fn with_bandwidth_chance(mut self, chance: f64) -> Self {
        if let Some(ref mut bandwidth) = self.settings.bandwidth {
            bandwidth.probability = Probability::new(chance / 100.0).unwrap_or_default();
        }
        self
    }

    /// Clears all settings, resetting to default.
    pub fn clear(mut self) -> Self {
        self.settings = Settings::default();
        self
    }

    /// Builds and returns the configured `Settings`.
    pub fn build(self) -> Settings {
        self.settings
    }
}

/// Extension trait for `Settings`.
impl Settings {
    /// Creates a new builder from existing settings.
    pub fn builder() -> SettingsBuilder {
        SettingsBuilder::new()
    }

    /// Returns whether any modules are enabled.
    pub fn has_active_modules(&self) -> bool {
        self.drop.is_some()
            || self.lag.is_some()
            || self.throttle.is_some()
            || self.reorder.is_some()
            || self.tamper.is_some()
            || self.duplicate.is_some()
            || self.bandwidth.is_some()
    }

    /// Returns a list of enabled module names.
    pub fn active_module_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.drop.is_some() {
            names.push("drop");
        }
        if self.lag.is_some() {
            names.push("lag");
        }
        if self.throttle.is_some() {
            names.push("throttle");
        }
        if self.reorder.is_some() {
            names.push("reorder");
        }
        if self.tamper.is_some() {
            names.push("tamper");
        }
        if self.duplicate.is_some() {
            names.push("duplicate");
        }
        if self.bandwidth.is_some() {
            names.push("bandwidth");
        }
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let settings = SettingsBuilder::new().build();
        assert!(!settings.has_active_modules());
    }

    #[test]
    fn test_builder_drop() {
        let settings = SettingsBuilder::new().drop(50.0).build();
        assert!(settings.drop.is_some());
        assert!((settings.drop.unwrap().probability.value() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_builder_lag_with_chance() {
        let settings = SettingsBuilder::new()
            .lag(100)
            .with_lag_chance(75.0)
            .build();
        
        assert!(settings.lag.is_some());
        let lag = settings.lag.unwrap();
        assert_eq!(lag.delay_ms, 100);
        assert!((lag.probability.value() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_builder_multiple_modules() {
        let settings = SettingsBuilder::new()
            .drop(25.0)
            .lag(50)
            .throttle(30)
            .build();

        assert!(settings.drop.is_some());
        assert!(settings.lag.is_some());
        assert!(settings.throttle.is_some());
        assert_eq!(settings.active_module_names().len(), 3);
    }

    #[test]
    fn test_builder_clear() {
        let settings = SettingsBuilder::new()
            .drop(50.0)
            .lag(100)
            .clear()
            .build();

        assert!(!settings.has_active_modules());
    }

    #[test]
    fn test_active_module_names() {
        let settings = SettingsBuilder::new()
            .drop(10.0)
            .bandwidth(1000)
            .build();

        let names = settings.active_module_names();
        assert!(names.contains(&"drop"));
        assert!(names.contains(&"bandwidth"));
        assert!(!names.contains(&"lag"));
    }
}
