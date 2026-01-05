use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::drop_stats::DropStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::drop::DropOptions;
use rand::{rng, Rng};

/// Unit struct for the Drop packet module.
///
/// This module simulates packet loss by randomly dropping packets
/// based on a configured probability.
#[derive(Debug, Default)]
pub struct DropModule;

impl PacketModule for DropModule {
    type Options = DropOptions;
    type State = ();

    fn name(&self) -> &'static str {
        "drop"
    }

    fn display_name(&self) -> &'static str {
        "Packet Drop"
    }

    fn get_duration_ms(&self, options: &Self::Options) -> u64 {
        options.duration_ms
    }

    fn process(
        &self,
        packets: &mut Vec<PacketData<'_>>,
        options: &Self::Options,
        _state: &mut Self::State,
        ctx: &mut ModuleContext,
    ) -> Result<()> {
        let mut stats = ctx.write_stats(self.name())?;
        drop_packets(
            packets,
            options.probability,
            options.inbound,
            options.outbound,
            &mut stats.drop_stats,
        );
        Ok(())
    }
}

/// Simulates packet dropping based on a specified probability.
///
/// This function processes a vector of packets and removes some based on the
/// provided probability. It updates statistics about dropped packets.
///
/// # Arguments
///
/// * `packets` - Mutable vector of packets that will be filtered
/// * `drop_probability` - Probability (0.0-1.0) of dropping each packet
/// * `stats` - Statistics tracker that will be updated with drop information
///
/// # Example
///
/// ```
/// let mut packets = vec![packet1, packet2, packet3];
/// let probability = Probability::new(0.3).unwrap(); // 30% chance to drop
/// let mut stats = DropStats::new(0.1); // With EWMA alpha of 0.1
///
/// drop_packets(&mut packets, probability, &mut stats);
/// ```
pub fn drop_packets(
    packets: &mut Vec<PacketData<'_>>,
    drop_probability: Probability,
    apply_inbound: bool,
    apply_outbound: bool,
    stats: &mut DropStats,
) {
    let mut rng = rng();

    packets.retain(|packet| {
        // Check if this packet's direction should be affected
        let matches_direction = (packet.is_outbound && apply_outbound)
            || (!packet.is_outbound && apply_inbound);

        if !matches_direction {
            // Direction doesn't match - keep packet unchanged
            return true;
        }

        let drop = rng.random::<f64>() < drop_probability.value();

        if drop {
            stats.record(true);
            return false;
        }

        stats.record(false);
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    #[test]
    fn test_drop_all_packets() {
        unsafe {
            // Create a test packet
            let mut packets = vec![PacketData::from(WinDivertPacket::<NetworkLayer>::new(
                vec![1, 2, 3],
            ))];

            // Initialize drop statistics with EWMA alpha=0.3
            let mut drop_stats = DropStats::new(0.3);

            // Use 100% drop probability to ensure all packets are dropped
            drop_packets(
                &mut packets,
                Probability::new(1.0).unwrap(),
                true,  // apply_inbound
                true,  // apply_outbound
                &mut drop_stats,
            );

            // Verify that all packets were dropped
            assert!(packets.is_empty());
            assert_eq!(drop_stats.total_packets, 1);
            assert_eq!(drop_stats.total_dropped, 1);
            assert_eq!(drop_stats.total_drop_rate(), 1.0);
        }
    }

    #[test]
    fn test_drop_no_packets() {
        unsafe {
            // Create multiple test packets
            let mut packets = vec![
                PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3])),
                PacketData::from(WinDivertPacket::<NetworkLayer>::new(vec![4, 5, 6])),
            ];
            let initial_count = packets.len();

            // Initialize drop statistics
            let mut drop_stats = DropStats::new(0.3);

            // Use 0% drop probability to ensure no packets are dropped
            drop_packets(
                &mut packets,
                Probability::new(0.0).unwrap(),
                true,  // apply_inbound
                true,  // apply_outbound
                &mut drop_stats,
            );

            // Verify that no packets were dropped
            assert_eq!(packets.len(), initial_count);
            assert_eq!(drop_stats.total_packets, 2);
            assert_eq!(drop_stats.total_dropped, 0);
            assert_eq!(drop_stats.total_drop_rate(), 0.0);
        }
    }
}
