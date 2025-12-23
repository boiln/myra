use crate::settings::bandwidth::BandwidthOptions;
use crate::settings::delay::DelayOptions;
use crate::settings::drop::DropOptions;
use crate::settings::duplicate::DuplicateOptions;
use crate::settings::reorder::ReorderOptions;
use crate::settings::tamper::TamperOptions;
use crate::settings::throttle::ThrottleOptions;
use serde::{Deserialize, Serialize, Serializer};
use std::io::Write;
use std::path::Path;
use std::{fs, io};

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

impl PacketManipulationSettings {
    /// Loads configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// * `io::Result<Self>` - The loaded configuration or an IO error
    #[allow(dead_code)]
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config =
            toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    /// Saves current configuration to a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the configuration will be saved
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` - Success or an IO error
    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Creates a default configuration file with all fields set to default values
    /// but commented out for user guidance.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the template configuration will be saved
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` - Success or an IO error
    #[allow(dead_code)]
    pub fn create_default_config_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let default_config = Self::default();

        // serialize the default configuration to TOML
        let serialized = toml::to_string_pretty(&default_config)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let commented_out = serialized
            .lines()
            .map(|line| {
                if line.trim().is_empty() || line.starts_with('[') {
                    line.to_string()
                } else {
                    format!("# {}", line)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let mut file = fs::File::create(path)?;
        file.write_all(commented_out.as_bytes())?;
        Ok(())
    }
}
