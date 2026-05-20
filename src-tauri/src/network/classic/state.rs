//! Classic mode processing state.
use crate::network::core::PacketData;
use std::collections::VecDeque;
use std::time::Instant;

/// State for Classic Latency module.
#[derive(Debug, Default)]
pub struct ClassicLatencyState {

    /// Queue of packets being held (with their capture time)
    pub buffer: VecDeque<(PacketData<'static>, Instant)>,

}

/// State for Classic Throttle module.
#[derive(Debug, Default)]
pub struct ClassicThrottleState {

    /// Queue of buffered packets
    pub buffer: VecDeque<PacketData<'static>>,
    /// When the current throttle window started (None = not throttling)
    pub window_start: Option<Instant>,

}

/// State for Classic Reorder module.
#[derive(Debug, Default)]
pub struct ClassicReorderState {

    /// Single packet being held for reordering
    pub held_packet: Option<PacketData<'static>>,
    /// How many cycles we've been holding this packet
    pub hold_cycles: u32,

}

/// State for Classic Tamper module.
#[derive(Debug)]
pub struct ClassicTamperState {

    /// XOR pattern for tampering
    pub patterns: [u8; 8],
    /// Current index into the pattern
    pub pattern_index: usize,

}

impl Default for ClassicTamperState {

    fn default() -> Self {
        Self {
            // Use a rotating pattern similar to the original
            patterns: [0xA5, 0x5A, 0xF0, 0x0F, 0xCC, 0x33, 0xAA, 0x55],
            pattern_index: 0,
        }
    }

}

/// State for Classic Bandwidth module.
#[derive(Debug)]
pub struct ClassicBandwidthState {

    /// Queue of buffered packets (exceeding bandwidth budget)
    pub buffer: VecDeque<PacketData<'static>>,
    /// Last tick time for calculating byte budget
    pub last_tick: Instant,
    /// Accumulated byte budget available to release
    pub byte_budget: f64,

}

impl Default for ClassicBandwidthState {

    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            last_tick: Instant::now(),
            byte_budget: 0.0,
        }
    }

}

/// Combined state for all Classic mode modules.
#[derive(Debug, Default)]
pub struct ClassicProcessingState {

    pub latency: ClassicLatencyState,
    pub throttle: ClassicThrottleState,
    pub reorder: ClassicReorderState,
    pub tamper: ClassicTamperState,
    pub bandwidth: ClassicBandwidthState,

}

impl ClassicProcessingState {

    pub fn new() -> Self {
        Self::default()
    }

    /// Flush all buffered packets (for shutdown).
    pub fn flush_all_buffers(&mut self) -> Vec<PacketData<'static>> {

        let mut packets = Vec::new();

        // Flush latency buffer
        while let Some((packet, _)) = self.latency.buffer.pop_front() {
            packets.push(packet);
        }

        // Flush throttle buffer
        while let Some(packet) = self.throttle.buffer.pop_front() {
            packets.push(packet);
        }

        // Flush held reorder packet
        if let Some(packet) = self.reorder.held_packet.take() {
            packets.push(packet);
        }

        // Flush bandwidth buffer
        while let Some(packet) = self.bandwidth.buffer.pop_front() {
            packets.push(packet);
        }

        packets

    }

}
