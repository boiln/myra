//! Core network functionality.
//!
//! This module contains the core components for packet interception
//! and manipulation, including handle management and packet data structures.

pub mod handle_manager;
pub mod packet_data;

// Re-export commonly used types
pub use handle_manager::{flush_wfp_cache, HandleConfig, HandleManager};
pub use packet_data::PacketData;
