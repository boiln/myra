//! Settings module for network condition simulation parameters.
//!
//! This module contains configuration structures for all the
//! different types of network manipulations that can be applied.
//!
//! # Example
//!
//! ```rust
//! use myra::settings::builder::SettingsBuilder;
//!
//! let settings = SettingsBuilder::new()
//!     .drop(50.0)
//!     .lag(100)
//!     .build();
//! ```

pub mod bandwidth;
pub mod builder;
pub mod burst;
pub mod lag;
pub mod drop;
pub mod duplicate;
pub mod manipulation;
pub mod reorder;
pub mod tamper;
pub mod tc_bandwidth;
pub mod throttle;

// Re-export commonly used types
pub use builder::SettingsBuilder;
pub use manipulation::Settings;
pub use tc_bandwidth::{TcBandwidthOptions, TcDirection};

/// Helper function for serde default values - returns true.
/// Used across all settings modules for inbound/outbound defaults.
pub fn default_true() -> bool {
    true
}
