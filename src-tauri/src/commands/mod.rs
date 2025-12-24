//! Tauri command handlers.
//!
//! This module contains all Tauri commands exposed to the frontend,
//! organized into submodules by functionality.

pub mod config;
pub mod start;
pub mod state;
pub mod status;
pub mod stop;
pub mod types;
pub mod update;

// Re-export state for convenient access
pub use state::PacketProcessingState;

// Re-export all command functions for use in main.rs
pub use start::start_processing;
pub use status::{get_filter, get_settings, get_status, update_filter};
pub use stop::stop_processing;
pub use update::update_settings;

use tauri::App;

/// Registers the packet processing state with the Tauri application.
pub fn register_commands(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    state::register_state(app)
}
