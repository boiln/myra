//! # Myra - A network condition simulation tool
//!
//! Myra is a tool for simulating poor network conditions by manipulating
//! network packets using `WinDivert` on Windows systems.
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
//! The core functionality relies on `WinDivert` to intercept and inject
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

// Enforce stricter lints for code quality
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,        // Many serde defaults can't be const
    clippy::cast_precision_loss,         // Acceptable for stats calculations
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,              // Safe in context (e.g., known positive values)
    clippy::unused_async,                // Tauri requires async for commands
    clippy::too_many_lines,              // Some functions are necessarily long
    clippy::too_many_arguments,          // Module processing requires many params
    clippy::uninlined_format_args,       // Style preference
    clippy::needless_pass_by_value,      // Arc clones are intentional for thread safety
    clippy::return_self_not_must_use,    // Builder pattern methods don't need this
    clippy::similar_names,               // Variable naming is intentional
    clippy::significant_drop_tightening, // Lock scopes are intentional
    clippy::items_after_statements,      // Allow const/type definitions after code
    clippy::ptr_arg,                     // Vec param is intentional for ownership
    clippy::unnecessary_debug_formatting, // Debug formatting is intentional in logs
    clippy::assigning_clones,            // Micro-optimization not critical
    clippy::or_fun_call,                 // Closure overhead is negligible
    clippy::match_same_arms,             // Explicit fallback patterns are clearer
    clippy::if_not_else                  // Style preference
)]

/// Commands exposed to the Tauri frontend.
pub mod commands;
/// Centralized error handling.
pub mod error;
/// Network packet manipulation functionality.
pub mod network;
/// Prelude for convenient imports.
pub mod prelude;
/// Configuration settings for packet manipulation.
pub mod settings;
/// Shared utility functions.
pub mod utils;

pub use error::{MyraError, Result};
