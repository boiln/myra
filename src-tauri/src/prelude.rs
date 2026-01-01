//! Prelude module for convenient imports.
//!
//! This module re-exports commonly used types and traits from the crate,
//! allowing users to import everything they need with a single use statement:
//!
//! ```rust
//! use myra::prelude::*;
//! ```

// Error handling
pub use crate::error::{MyraError, Result};

// Network core
pub use crate::network::core::{flush_wfp_cache, HandleConfig, HandleManager, PacketData};

// Module traits
pub use crate::network::modules::traits::{ModuleContext, ModuleOptions, PacketModule};

// Statistics
pub use crate::network::modules::stats::PacketProcessingStatistics;

// Probability type
pub use crate::network::types::probability::Probability;

// Settings
pub use crate::settings::{Settings, SettingsBuilder};

// Individual module options (for advanced usage)
pub use crate::settings::{
    bandwidth::BandwidthOptions, delay::DelayOptions, drop::DropOptions,
    duplicate::DuplicateOptions, reorder::ReorderOptions, tamper::TamperOptions,
    throttle::ThrottleOptions,
};
