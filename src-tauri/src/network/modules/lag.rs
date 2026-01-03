use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::lag_stats::LagStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::lag::LagOptions;
use rand::{rng, Rng};
use std::collections::VecDeque;
use std::time::Duration;

/// Unit struct for the Lag packet module.
///
/// This module simulates network latency by holding packets for a
/// specified duration before releasing them.
/// With default probability of 100%, all traffic is lagged by the configured time.
#[derive(Debug, Default)]
pub struct LagModule;

/// State maintained by the lag module between processing calls.
pub type LagState = VecDeque<PacketData<'static>>;

impl PacketModule for LagModule {
    type Options = LagOptions;
    type State = LagState;

    fn name(&self) -> &'static str {
        "lag"
    }

    fn display_name(&self) -> &'static str {
        "Lag"
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

        lag_packets(
            packets,
            storage,
            Duration::from_millis(options.delay_ms),
            options.probability,
            options.inbound,
            options.outbound,
            &mut stats.lag_stats,
        );
        Ok(())
    }
}

/// Simulates network lag by holding packets for a specified duration.
///
/// This function holds incoming packets in a buffer and only releases them
/// after the specified lag time has elapsed.
/// With probability set to 1.0 (100%, the default), all traffic is lagged.
///
/// # How it works
///
/// 1. Incoming packets are moved to the lag storage queue based on probability
/// 2. On each processing cycle, packets that have been in the queue for at least
///    the lag duration are moved back to the outgoing packets vector
/// 3. Statistics are updated with the number of packets still being lagged
///
/// # Arguments
///
/// * `packets` - Mutable vector of packets that will be processed
/// * `storage` - Persistent queue for storing lagged packets
/// * `lag` - The duration to lag each packet
/// * `probability` - Probability of lagging each packet (default 1.0 = 100%)
/// * `stats` - Statistics tracker that will be updated with lag information
///
/// # Example
///
/// ```
/// let mut packets = vec![packet1, packet2];
/// let mut storage = VecDeque::new();
/// let lag = Duration::from_millis(100);
/// let probability = Probability::new(1.0).unwrap(); // 100% - all packets lagged
/// let mut stats = LagStats::new();
///
/// lag_packets(&mut packets, &mut storage, lag, probability, &mut stats);
/// ```
pub fn lag_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    storage: &mut VecDeque<PacketData<'a>>,
    lag: Duration,
    probability: Probability,
    apply_inbound: bool,
    apply_outbound: bool,
    stats: &mut LagStats,
) {
    let mut rng = rng();
    let mut passthrough_packets = Vec::new();
    let prob_value = probability.value();

    // Move packets to the lag buffer based on probability and direction
    // With default probability of 1.0, ALL matching packets are lagged
    for packet in packets.drain(..) {
        // Check if this packet's direction should be affected
        let matches_direction = (packet.is_outbound && apply_outbound)
            || (!packet.is_outbound && apply_inbound);

        if !matches_direction {
            // Direction doesn't match - let packet pass through
            passthrough_packets.push(packet);
            continue;
        }

        if rng.random::<f64>() >= prob_value {
            passthrough_packets.push(packet);
            continue;
        }
        storage.push_back(packet);
    }

    // Collect packets that have been lagged long enough
    // Check packets from the front (oldest first) and release those that have waited long enough
    while let Some(packet_data) = storage.front() {
        if packet_data.arrival_time.elapsed() < lag {
            // Since packets are ordered by arrival time, if this one isn't ready,
            // none of the following ones will be either
            break;
        }
        
        let Some(packet) = storage.pop_front() else { break };
        passthrough_packets.push(packet);
    }

    // Put all packets (passthrough + released) back into the output
    packets.extend(passthrough_packets);
    stats.lagged_package_count(storage.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::modules::stats::lag_stats::LagStats;
    use std::time::{Duration, Instant};
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    #[test]
    fn test_lag_packets_immediate_release_after_lag() {
        unsafe {
            // Create test packet with an arrival time in the past
            let mut old_packet =
                PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]));

            // Manually set arrival time to be in the past by enough to bypass lag
            let now = Instant::now();
            let past = now - Duration::from_millis(200);
            std::ptr::write(&mut old_packet.arrival_time as *mut Instant, past);

            let mut packets = vec![old_packet];
            let mut storage = VecDeque::new();
            let mut stats = LagStats::new();

            // Lag of 100ms with 100% probability (should be immediately released because arrival was 200ms ago)
            lag_packets(
                &mut packets,
                &mut storage,
                Duration::from_millis(100),
                Probability::new(1.0).unwrap(),
                true,  // apply_inbound
                true,  // apply_outbound
                &mut stats,
            );

            // Packet should have passed through immediately (it was already lagged 200ms)
            assert_eq!(packets.len(), 1);
            assert_eq!(storage.len(), 0);
            assert_eq!(stats.current_lagged(), 0);
        }
    }

    #[test]
    fn test_lag_packets_held_until_lag_elapsed() {
        unsafe {
            // Create a new packet (will have recent arrival time)
            let packet = PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]));

            let mut packets = vec![packet];
            let mut storage = VecDeque::new();
            let mut stats = LagStats::new();

            // Apply a long lag with 100% probability (ensuring the packet will be held)
            lag_packets(
                &mut packets,
                &mut storage,
                Duration::from_millis(1000),
                Probability::new(1.0).unwrap(),
                true,  // apply_inbound
                true,  // apply_outbound
                &mut stats,
            );

            // ALL packets should be held in storage with 100% probability
            assert_eq!(packets.len(), 0);
            assert_eq!(storage.len(), 1);
            assert_eq!(stats.current_lagged(), 1);
        }
    }

    #[test]
    fn test_all_packets_lagged_with_100_percent() {
        unsafe {
            // Create multiple packets
            let packet1 = PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]));
            let packet2 = PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![4, 5, 6]));
            let packet3 = PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![7, 8, 9]));

            let mut packets = vec![packet1, packet2, packet3];
            let mut storage = VecDeque::new();
            let mut stats = LagStats::new();

            // Apply lag with 100% probability - ALL packets should be lagged
            lag_packets(
                &mut packets,
                &mut storage,
                Duration::from_millis(1000),
                Probability::new(1.0).unwrap(),
                true,  // apply_inbound
                true,  // apply_outbound
                &mut stats,
            );

            // ALL packets should be in storage, none passed through
            assert_eq!(packets.len(), 0);
            assert_eq!(storage.len(), 3);
            assert_eq!(stats.current_lagged(), 3);
        }
    }
}
