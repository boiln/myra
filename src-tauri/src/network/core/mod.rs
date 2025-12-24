//! Core network functionality.
//!
//! This module contains the core components for packet interception
//! and manipulation, including handle management and packet data structures.

pub mod handle;
pub mod packet;

// Re-export commonly used types
pub use handle::{construct_filter_with_exclusions, flush_wfp_cache, HandleConfig, HandleManager};
pub use packet::PacketData;
