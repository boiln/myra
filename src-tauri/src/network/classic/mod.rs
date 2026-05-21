//! Classic mode packet manipulation modules.
//!
//! These modules implement timer-based, deterministic network manipulation
//! as opposed to Standard mode's probabilistic per-packet approach.
pub mod bandwidth;
pub mod drop;
pub mod latency;
pub mod processor;
pub mod reorder;
pub mod state;
pub mod tamper;
pub mod throttle;

pub use processor::process_classic_packets;
pub use state::ClassicProcessingState;
