use crate::settings::bandwidth::BandwidthOptions;
use crate::settings::delay::DelayOptions;
use crate::settings::drop::DropOptions;
use crate::settings::duplicate::DuplicateOptions;
use crate::settings::reorder::ReorderOptions;
use crate::settings::tamper::TamperOptions;
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
pub struct PacketManipulationSettings {
    /// Controls random packet dropping
    #[serde(serialize_with = "serialize_option")]
    pub drop: Option<DropOptions>,

    /// Controls packet delay simulation
    #[serde(default, serialize_with = "serialize_option")]
    pub delay: Option<DelayOptions>,

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
}


