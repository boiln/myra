use crate::network::core::packet_data::PacketData;
use crate::network::modules::stats::throttle_stats::ThrottleStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::throttle::ThrottleOptions;
use log::error;
use rand::Rng;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Unit struct for the Throttle packet module.
///
/// This module simulates network throttling by either dropping packets
/// or storing them temporarily during throttle periods.
#[derive(Debug, Default)]
pub struct ThrottleModule;

/// State maintained by the throttle module between processing calls.
pub struct ThrottleState {
    pub storage: VecDeque<PacketData<'static>>,
    pub throttled_start_time: Instant,
}

impl Default for ThrottleState {
    fn default() -> Self {
        Self {
            storage: VecDeque::new(),
            throttled_start_time: Instant::now(),
        }
    }
}

impl PacketModule for ThrottleModule {
    type Options = ThrottleOptions;
    type State = ThrottleState;

    fn name(&self) -> &'static str {
        "throttle"
    }

    fn display_name(&self) -> &'static str {
        "Network Throttle"
    }

    fn get_duration_ms(&self, options: &Self::Options) -> u64 {
        options.duration_ms
    }

    fn process<'a>(
        &self,
        packets: &mut Vec<PacketData<'a>>,
        options: &Self::Options,
        state: &mut Self::State,
        ctx: &mut ModuleContext,
    ) {
        let mut stats = ctx.statistics.write().unwrap_or_else(|e| {
            error!("Failed to acquire write lock for throttle statistics: {}", e);
            panic!("Failed to acquire statistics lock");
        });
        
        // Safety: We need to transmute lifetimes here because the storage persists
        // across processing calls.
        let storage: &mut VecDeque<PacketData<'a>> = unsafe {
            std::mem::transmute(&mut state.storage)
        };
        
        throttle_packages(
            packets,
            storage,
            &mut state.throttled_start_time,
            options.probability,
            Duration::from_millis(options.throttle_ms),
            options.drop,
            &mut stats.throttle_stats,
        );
    }
}

/// Throttles network packets by either dropping them or storing them temporarily
///
/// This function simulates network throttling by controlling the flow of packets.
/// When throttling is active (based on probability and duration):
/// - If drop mode is enabled, packets are discarded entirely
/// - If drop mode is disabled, packets are stored temporarily and released when throttling stops
///
/// # Arguments
///
/// * `packets` - Vector of packets to process; may be modified by this function
/// * `storage` - Queue for storing packets when throttling is active and drop mode is disabled
/// * `throttled_start_time` - Time when the current throttling period began
/// * `throttle_probability` - Probability of starting a new throttling period
/// * `throttle_duration` - Duration of each throttling period
/// * `drop` - If true, packets are dropped during throttling; if false, they are stored
/// * `stats` - Statistics collector for throttling operations
///
/// # Example
///
/// ```
/// let mut packets = vec![packet1, packet2];
/// let mut storage = VecDeque::new();
/// let mut throttled_start_time = Instant::now();
/// let throttle_probability = Probability::new(0.1).unwrap(); // 10% chance
/// let throttle_duration = Duration::from_millis(500);
/// let drop = false; // Store packets rather than dropping them
/// let mut stats = ThrottleStats::new();
///
/// throttle_packages(
///     &mut packets,
///     &mut storage,
///     &mut throttled_start_time,
///     throttle_probability,
///     throttle_duration,
///     drop,
///     &mut stats,
/// );
/// ```
pub fn throttle_packages<'a>(
    packets: &mut Vec<PacketData<'a>>,
    storage: &mut VecDeque<PacketData<'a>>,
    throttled_start_time: &mut Instant,
    throttle_probability: Probability,
    throttle_duration: Duration,
    drop: bool,
    stats: &mut ThrottleStats,
) {
    if is_throttled(throttle_duration, throttled_start_time) {
        if drop {
            stats.dropped_count += packets.len();
            packets.clear();
        }

        if !drop {
            storage.extend(packets.drain(..));
        }

        stats.is_throttling = true;
        return;
    }

    packets.extend(storage.drain(..));

    if rand::rng().random_bool(throttle_probability.value()) {
        *throttled_start_time = Instant::now();
    }

    stats.is_throttling = false;
}

/// Determines if throttling is currently active
///
/// # Arguments
///
/// * `throttle_duration` - Duration of each throttling period
/// * `throttled_start_time` - Time when the current throttling period began
///
/// # Returns
///
/// `true` if currently within a throttling period, `false` otherwise
fn is_throttled(throttle_duration: Duration, throttled_start_time: &mut Instant) -> bool {
    throttled_start_time.elapsed() <= throttle_duration
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Creates a simple test packet for testing
    fn create_test_packet<'a>(id: u8) -> PacketData<'a> {
        // This is a simplification - in real tests we'd create proper packet data
        unsafe {
            let data = vec![id; 10]; // Simple packet with 10 bytes all set to id
            PacketData::from(windivert::packet::WinDivertPacket::<
                windivert::layer::NetworkLayer,
            >::new(data))
        }
    }

    #[test]
    fn test_is_throttled() {
        // Test with a throttling period that has not yet elapsed
        let throttle_duration = Duration::from_secs(1);
        let mut start_time = Instant::now();
        assert!(is_throttled(throttle_duration, &mut start_time));

        // Test with a throttling period that has elapsed
        let throttle_duration = Duration::from_millis(1);
        let mut start_time = Instant::now() - Duration::from_secs(1);
        assert!(!is_throttled(throttle_duration, &mut start_time));
    }

    #[test]
    fn test_throttle_packages_drop_mode() {
        let mut packets = vec![create_test_packet(1), create_test_packet(2)];
        let mut storage = VecDeque::new();
        let mut throttled_start_time = Instant::now();
        let throttle_probability = Probability::new(1.0).unwrap(); // Always throttle
        let throttle_duration = Duration::from_secs(1); // Long enough to ensure throttling is active
        let drop = true; // Drop mode enabled
        let mut stats = ThrottleStats::new();

        throttle_packages(
            &mut packets,
            &mut storage,
            &mut throttled_start_time,
            throttle_probability,
            throttle_duration,
            drop,
            &mut stats,
        );

        assert!(packets.is_empty(), "Packets should be dropped in drop mode");
        assert!(
            storage.is_empty(),
            "Storage should remain empty in drop mode"
        );
        assert!(stats.is_throttling, "Throttling status should be true");
        assert_eq!(stats.dropped_count, 2, "Should record 2 dropped packets");
    }
}
