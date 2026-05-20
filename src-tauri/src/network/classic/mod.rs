//! Classic mode packet manipulation modules.
//!
//! These modules implement timer-based, deterministic network manipulation
//! as opposed to Standard mode's probabilistic per-packet approach.
pub mod latency;
pub mod drop;
pub mod throttle;
pub mod reorder;
pub mod tamper;
pub mod bandwidth;
pub mod processor;
pub mod state;

pub use processor::process_classic_packets;
pub use state::ClassicProcessingState;
