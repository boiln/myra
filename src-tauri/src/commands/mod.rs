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

// Re-export all command functions and their generated Tauri command handlers for use in main.rs
pub use start::{__cmd__start_processing, start_processing};
pub use status::{
    __cmd__get_filter, __cmd__get_settings, __cmd__get_status, __cmd__update_filter, get_filter,
    get_settings, get_status, update_filter,
};
pub use stop::{__cmd__stop_processing, stop_processing};
pub use update::{__cmd__update_settings, update_settings};

use tauri::App;

/// Registers the packet processing state with the Tauri application.
pub fn register_commands(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    state::register_state(app)
}
