use crate::network::core::packet_data::PacketData;
use crate::network::types::delayed_packet::DelayedPacket;
use std::collections::{BinaryHeap, VecDeque};
use std::time::Instant;

/// Maintains state for the packet processing operations
#[derive(Debug)]
pub struct PacketProcessingState<'a> {
    pub delay_storage: VecDeque<PacketData<'a>>,
    pub reorder_storage: BinaryHeap<DelayedPacket<'a>>,
    pub bandwidth_limit_storage: VecDeque<PacketData<'a>>,
    pub bandwidth_storage_total_size: usize,
    pub throttle_storage: VecDeque<PacketData<'a>>,
    pub throttled_start_time: Instant,
    pub last_sent_package_time: Instant,

    /// Time when each module's effect was started
    pub effect_start_times: ModuleEffectStartTimes,
}

/// Tracks when each module's effect was started
#[derive(Debug)]
pub struct ModuleEffectStartTimes {
    /// Time when drop effect was started
    pub drop_start: Instant,

    /// Time when delay effect was started  
    pub delay_start: Instant,

    /// Time when throttle effect was started
    pub throttle_start: Instant,

    /// Time when duplicate effect was started
    pub duplicate_start: Instant,

    /// Time when tamper effect was started
    pub tamper_start: Instant,

    /// Time when reorder effect was started
    pub reorder_start: Instant,

    /// Time when bandwidth effect was started
    pub bandwidth_start: Instant,
}

impl Default for ModuleEffectStartTimes {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            drop_start: now,
            delay_start: now,
            throttle_start: now,
            duplicate_start: now,
            tamper_start: now,
            reorder_start: now,
            bandwidth_start: now,
        }
    }
}

impl<'a> PacketProcessingState<'a> {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            delay_storage: VecDeque::new(),
            reorder_storage: BinaryHeap::new(),
            bandwidth_limit_storage: VecDeque::new(),
            bandwidth_storage_total_size: 0,
            throttle_storage: VecDeque::new(),
            throttled_start_time: now,
            last_sent_package_time: now,
            effect_start_times: ModuleEffectStartTimes::default(),
        }
    }
}
