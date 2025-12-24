//! Packet processing pipeline.
//!
//! This module handles the core packet interception and processing logic.

pub mod module_state;
pub mod processor;
pub mod receiver;

// Re-export main entry points
pub use processor::start_packet_processing;
pub use receiver::receive_packets;
