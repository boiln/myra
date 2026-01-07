//! Tauri command handlers.
//!
//! This module contains all Tauri commands exposed to the frontend,
//! organized into submodules by functionality.

pub mod config;
pub mod start;
pub mod state;
pub mod status;
pub mod stop;
pub mod system;
pub mod tc_bandwidth;
pub mod filter_history;
pub mod types;
pub mod update;

pub use state::PacketProcessingState;
pub use tc_bandwidth::TcLimiterState;

pub use start::{__cmd__start_processing, start_processing};
pub use status::{
    __cmd__get_filter, __cmd__get_settings, __cmd__get_status, __cmd__update_filter, get_filter,
    get_settings, get_status, update_filter,
};
pub use filter_history::{
    __cmd__get_filter_history, __cmd__clear_filter_history, get_filter_history, clear_filter_history,
};
pub use stop::{__cmd__stop_processing, stop_processing};
pub use system::{
    __cmd__build_device_filter, __cmd__build_process_filter, __cmd__get_flow_filter,
    __cmd__is_flow_tracking, __cmd__list_processes, __cmd__scan_network_devices,
    __cmd__start_flow_tracking, __cmd__stop_flow_tracking, __cmd__validate_filter,
    build_device_filter, build_process_filter, get_flow_filter, is_flow_tracking, list_processes,
    scan_network_devices, start_flow_tracking, stop_flow_tracking, validate_filter,
};
pub use tc_bandwidth::{
    __cmd__start_tc_bandwidth, __cmd__stop_tc_bandwidth, __cmd__get_tc_bandwidth_status,
    start_tc_bandwidth, stop_tc_bandwidth, get_tc_bandwidth_status,
};
pub use update::{__cmd__update_settings, update_settings};

use tauri::App;

pub fn register_commands(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    state::register_state(app)
}
