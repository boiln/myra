use crate::settings::bandwidth::BandwidthOptions;
use crate::settings::burst::BurstOptions;
use crate::settings::lag::LagOptions;
use crate::settings::drop::DropOptions;
use crate::settings::duplicate::DuplicateOptions;
use crate::settings::reorder::ReorderOptions;
use crate::settings::tamper::TamperOptions;
use crate::settings::tc_bandwidth::TcBandwidthOptions;
use crate::settings::throttle::ThrottleOptions;
use serde::{Deserialize, Serialize, Serializer};

/// Custom serializer for Option<T> values in configuration.
///
/// This function allows for consistent serialization of Option values
/// across the application.
pub fn serialize_option<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    match value {
        Some(v) => v.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// Represents all network packet manipulation settings.
///
/// This struct contains all the different types of network condition simulations
/// that can be applied to packets, each as an optional setting.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    /// Controls random packet dropping
    #[serde(serialize_with = "serialize_option")]
    pub drop: Option<DropOptions>,

    /// Controls packet lag simulation
    #[serde(default, serialize_with = "serialize_option")]
    pub lag: Option<LagOptions>,

    /// Controls network throttling
    #[serde(serialize_with = "serialize_option")]
    pub throttle: Option<ThrottleOptions>,

    /// Controls packet reordering
    #[serde(serialize_with = "serialize_option")]
    pub reorder: Option<ReorderOptions>,

    /// Controls packet corruption/tampering
    #[serde(serialize_with = "serialize_option")]
    pub tamper: Option<TamperOptions>,

    /// Controls packet duplication
    #[serde(serialize_with = "serialize_option")]
    pub duplicate: Option<DuplicateOptions>,

    /// Controls bandwidth limitations
    #[serde(serialize_with = "serialize_option")]
    pub bandwidth: Option<BandwidthOptions>,

    /// Controls packet bursting (lag switch)
    #[serde(serialize_with = "serialize_option")]
    pub burst: Option<BurstOptions>,

    /// Enable MGO2/lag bypass mode - when send fails, swap IPs and retry
    /// This technique can bypass certain game anti-lag detection
    #[serde(default)]
    pub lag_bypass: bool,

    /// Traffic Control bandwidth limiting (NetLimiter-style)
    /// Works at OS socket layer for true rate limiting
    #[serde(default, serialize_with = "serialize_option")]
    pub tc_bandwidth: Option<TcBandwidthOptions>,
}

/// Type alias for backward compatibility.
pub type PacketManipulationSettings = Settings;

// Implement ModuleOptions trait for all option types
use crate::network::modules::traits::ModuleOptions;

impl ModuleOptions for DropOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for LagOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for ThrottleOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for ReorderOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for TamperOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for DuplicateOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for BandwidthOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ModuleOptions for BurstOptions {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}


