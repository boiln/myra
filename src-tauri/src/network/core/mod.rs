//! Core network functionality.
//!
//! This module contains the core components for packet interception
//! and manipulation, including handle management and packet data structures.

pub mod flow_tracker;
pub mod handle;
pub mod packet;

// Re-export commonly used types
pub use flow_tracker::FlowTracker;
pub use handle::{
    construct_filter_with_exclusions, flush_wfp_cache, restore_timer_resolution,
    set_high_precision_timer, HandleConfig, HandleManager,
};
pub use packet::PacketData;
