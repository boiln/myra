use crate::network::modules::stats::bandwidth_stats::BandwidthStats;
use crate::network::modules::stats::burst_stats::BurstStats;
use crate::network::modules::stats::lag_stats::LagStats;
use crate::network::modules::stats::drop_stats::DropStats;
use crate::network::modules::stats::duplicate_stats::DuplicateStats;
use crate::network::modules::stats::reorder_stats::ReorderStats;
use crate::network::modules::stats::tamper_stats::TamperStats;
use crate::network::modules::stats::throttle_stats::ThrottleStats;
use std::time::Duration;

pub mod bandwidth_stats;
pub mod burst_stats;
pub mod lag_stats;
pub mod drop_stats;
pub mod duplicate_stats;
pub mod reorder_stats;
pub mod tamper_stats;
pub mod throttle_stats;
pub mod util;

/// Statistics collection for all packet processing modules
///
/// Maintains counters and metrics for various network conditions being simulated,
/// such as packet drops, lag, reordering, and duplication.
#[derive(Debug)]
pub struct PacketProcessingStatistics {
    /// Statistics for packet dropping
    pub drop_stats: DropStats,
    /// Statistics for packet lag
    pub lag_stats: LagStats,
    /// Statistics for bandwidth throttling
    pub throttle_stats: ThrottleStats,
    /// Statistics for packet reordering
    pub reorder_stats: ReorderStats,
    /// Statistics for packet tampering
    pub tamper_stats: TamperStats,
    /// Statistics for packet duplication
    pub duplicate_stats: DuplicateStats,
    /// Statistics for bandwidth usage
    pub bandwidth_stats: BandwidthStats,
    /// Statistics for packet bursting
    pub burst_stats: BurstStats,
}

impl Default for PacketProcessingStatistics {
    fn default() -> Self {
        Self {
            drop_stats: DropStats::new(0.005),
            lag_stats: LagStats::new(),
            throttle_stats: ThrottleStats::new(),
            reorder_stats: ReorderStats::new(0.005),
            tamper_stats: TamperStats::new(Duration::from_millis(500)),
            duplicate_stats: DuplicateStats::new(0.005),
            bandwidth_stats: BandwidthStats::new(0.005),
            burst_stats: BurstStats::new(0.005),
        }
    }
}
