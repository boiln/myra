use crate::network::modules::bandwidth::BandwidthState;
use crate::network::modules::burst::BurstState;
use crate::network::modules::delay::DelayState;
use crate::network::modules::reorder::ReorderState;
use crate::network::modules::throttle::ThrottleState;
use std::time::Instant;

/// Maintains state for the packet processing modules.
///
/// This struct holds all module-specific state that needs to persist
/// between processing iterations, such as queued packets and timing info.
#[derive(Debug)]
pub struct ModuleProcessingState {
    /// State for the delay module
    pub delay: DelayState,
    /// State for the reorder module
    pub reorder: ReorderState,
    /// State for the bandwidth module
    pub bandwidth: BandwidthState,
    /// State for the throttle module
    pub throttle: ThrottleState,
    /// State for the burst module
    pub burst: BurstState,
    /// Whether burst was enabled in the previous processing cycle
    pub burst_was_enabled: bool,
    /// Release delay for burst packets in microseconds
    pub burst_release_delay_us: u64,

    /// Time when each module's effect was started
    pub effect_start_times: ModuleEffectStartTimes,
}

/// Tracks when each module's effect was started.
///
/// Used to implement duration-based effects that automatically
/// disable after a certain time period.
#[derive(Debug)]
pub struct ModuleEffectStartTimes {
    /// Time when drop effect was started
    pub drop: Instant,
    /// Time when delay effect was started  
    pub delay: Instant,
    /// Time when throttle effect was started
    pub throttle: Instant,
    /// Time when duplicate effect was started
    pub duplicate: Instant,
    /// Time when tamper effect was started
    pub tamper: Instant,
    /// Time when reorder effect was started
    pub reorder: Instant,
    /// Time when bandwidth effect was started
    pub bandwidth: Instant,
    /// Time when burst effect was started
    pub burst: Instant,
}

impl Default for ModuleEffectStartTimes {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            drop: now,
            delay: now,
            throttle: now,
            duplicate: now,
            tamper: now,
            reorder: now,
            bandwidth: now,
            burst: now,
        }
    }
}

impl ModuleProcessingState {
    pub fn new() -> Self {
        Self {
            delay: DelayState::default(),
            reorder: ReorderState::default(),
            bandwidth: BandwidthState::default(),
            throttle: ThrottleState::default(),
            burst: BurstState::default(),
            burst_was_enabled: false,
            burst_release_delay_us: 500, // Default 0.5ms
            effect_start_times: ModuleEffectStartTimes::default(),
        }
    }
}

impl Default for ModuleProcessingState {
    fn default() -> Self {
        Self::new()
    }
}
