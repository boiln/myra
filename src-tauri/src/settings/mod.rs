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
//!     .delay(100)
//!     .build();
//! ```

pub mod bandwidth;
pub mod builder;
pub mod delay;
pub mod drop;
pub mod duplicate;
pub mod manipulation;
pub mod reorder;
pub mod tamper;
pub mod throttle;

// Re-export commonly used types
pub use builder::SettingsBuilder;
pub use manipulation::Settings;
