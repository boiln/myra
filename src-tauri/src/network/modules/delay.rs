use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::delay_stats::DelayStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::delay::DelayOptions;
use rand::{rng, Rng};
use std::collections::VecDeque;
use std::time::Duration;

/// Unit struct for the Delay packet module.
///
/// This module simulates network latency by holding packets for a
/// specified duration before releasing them.
#[derive(Debug, Default)]
pub struct DelayModule;

/// State maintained by the delay module between processing calls.
pub type DelayState = VecDeque<PacketData<'static>>;

impl PacketModule for DelayModule {
    type Options = DelayOptions;
    type State = DelayState;

    fn name(&self) -> &'static str {
        "delay"
    }

    fn display_name(&self) -> &'static str {
        "Packet Delay"
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
    ) -> Result<()> {
        let mut stats = ctx.write_stats(self.name())?;

        // Safety: We need to transmute lifetimes here because the storage persists
        // across processing calls. The packets are owned by the storage until released.
        let storage: &mut VecDeque<PacketData<'a>> = unsafe { std::mem::transmute(state) };

        delay_packets(
            packets,
            storage,
            Duration::from_millis(options.delay_ms),
            options.probability,
            &mut stats.delay_stats,
        );
        Ok(())
    }
}

/// Simulates network delay by holding packets for a specified duration.
///
/// This function processes packets and holds them in a storage queue until
/// they've been delayed for the specified duration. It updates statistics
/// about the delayed packets.
///
/// # How it works
///
/// 1. Incoming packets are moved to the delay storage queue based on probability
/// 2. Packets that have been in the storage queue for at least the delay duration
///    are moved back to the outgoing packets vector
/// 3. Statistics are updated with the number of packets still being delayed
///
/// # Arguments
///
/// * `packets` - Mutable vector of packets that will be processed
/// * `storage` - Persistent queue for storing delayed packets
/// * `delay` - The duration to delay each packet
/// * `probability` - Probability of delaying each packet
/// * `stats` - Statistics tracker that will be updated with delay information
///
/// # Example
///
/// ```
/// let mut packets = vec![packet1, packet2];
/// let mut storage = VecDeque::new();
/// let delay = Duration::from_millis(100);
/// let probability = Probability::new(0.5).unwrap(); // 50% chance
/// let mut stats = DelayStats::new();
///
/// delay_packets(&mut packets, &mut storage, delay, probability, &mut stats);
/// ```
pub fn delay_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    storage: &mut VecDeque<PacketData<'a>>,
    delay: Duration,
    probability: Probability,
    stats: &mut DelayStats,
) {
    let mut rng = rng();
    let mut packets_to_process = Vec::new();

    for packet in packets.drain(..) {
        if rng.random::<f64>() < probability.value() {
            storage.push_back(packet);
            continue;
        }

        packets_to_process.push(packet);
    }

    while let Some(packet_data) = storage.pop_front() {
        if packet_data.arrival_time.elapsed() < delay {
            storage.push_front(packet_data);
            break;
        }

        packets_to_process.push(packet_data);
    }

    packets.extend(packets_to_process);

    stats.delayed_package_count(storage.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::modules::stats::delay_stats::DelayStats;
    use std::time::{Duration, Instant};
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    #[test]
    fn test_delay_packets_immediate() {
        unsafe {
            // Create test packet with an arrival time in the past
            let mut old_packet =
                PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]));

            // Manually set arrival time to be in the past by enough to bypass delay
            let now = Instant::now();
            let past = now - Duration::from_millis(200);
            std::ptr::write(&mut old_packet.arrival_time as *mut Instant, past);

            let mut packets = vec![old_packet];
            let mut storage = VecDeque::new();
            let mut stats = DelayStats::new();

            // Delay of 100ms (should be immediately bypassed by our packet)
            delay_packets(
                &mut packets,
                &mut storage,
                Duration::from_millis(100),
                Probability::new(0.5).unwrap(),
                &mut stats,
            );

            // Packet should have passed through immediately
            assert_eq!(packets.len(), 1);
            assert_eq!(storage.len(), 0);
            assert_eq!(stats.current_delayed(), 0);
        }
    }

    #[test]
    fn test_delay_packets_held() {
        unsafe {
            // Create a new packet (will have recent arrival time)
            let packet = PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]));

            let mut packets = vec![packet];
            let mut storage = VecDeque::new();
            let mut stats = DelayStats::new();

            // Apply a long delay (ensuring the packet will be held)
            delay_packets(
                &mut packets,
                &mut storage,
                Duration::from_millis(1000),
                Probability::new(0.5).unwrap(),
                &mut stats,
            );

            // Packet should be held in storage
            assert_eq!(packets.len(), 0);
            assert_eq!(storage.len(), 1);
            assert_eq!(stats.current_delayed(), 1);
        }
    }
}
