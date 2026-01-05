//! # Myra - A network condition simulation tool
//!
//! Myra is a tool for simulating poor network conditions by manipulating
//! network packets using WinDivert on Windows systems.
//!
//! ## Features
//!
//! * Packet dropping - Randomly drop packets to simulate packet loss
//! * Packet lag - Add latency to packets
//! * Network throttling - Slow down connections temporarily
//! * Packet reordering - Change the order packets arrive in
//! * Packet tampering - Corrupt packet data
//! * Packet duplication - Send multiple copies of packets
//! * Bandwidth limiting - Restrict connection speeds
//!
//! ## Architecture
//!
//! Myra is built using a Tauri 2.0 app structure with:
//! * Rust backend: Handles packet interception and manipulation
//! * TypeScript frontend: Provides a user-friendly interface
//!
//! The core functionality relies on WinDivert to intercept and inject
//! network packets on Windows systems.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use myra::prelude::*;
//!
//! // Build settings using the fluent builder API
//! let settings = SettingsBuilder::new()
//!     .drop(25.0)     // 25% packet drop rate
//!     .lag(100)     // 100ms lag
//!     .with_lag_chance(50.0)  // 50% chance of lag
//!     .build();
//! ```

pub mod commands;
pub mod error;
pub mod network;
pub mod prelude;
pub mod settings;
pub mod utils;

pub use error::{MyraError, Result};
