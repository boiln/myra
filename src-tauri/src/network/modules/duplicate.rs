use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::duplicate_stats::DuplicateStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::duplicate::DuplicateOptions;
use rand::Rng;
use std::vec::Vec;

/// Unit struct for the Duplicate packet module.
///
/// This module simulates packet duplication by creating copies of
/// packets based on a configured probability and count.
#[derive(Debug, Default)]
pub struct DuplicateModule;

impl PacketModule for DuplicateModule {
    type Options = DuplicateOptions;
    type State = ();

    fn name(&self) -> &'static str {
        "duplicate"
    }

    fn display_name(&self) -> &'static str {
        "Packet Duplicate"
    }

    fn get_duration_ms(&self, options: &Self::Options) -> u64 {
        options.duration_ms
    }

    fn should_skip(&self, options: &Self::Options) -> bool {
        options.count == 0 || options.probability.value() <= 0.0
    }

    fn process<'a>(
        &self,
        packets: &mut Vec<PacketData<'a>>,
        options: &Self::Options,
        _state: &mut Self::State,
        ctx: &mut ModuleContext,
    ) -> Result<()> {
        let mut stats = ctx.write_stats(self.name())?;

        duplicate_packets(
            packets,
            options.count,
            options.probability,
            &mut stats.duplicate_stats,
        );
        Ok(())
    }
}

/// Duplicates packets according to a probability
///
/// Creates copies of packets based on given probability and duplication count.
/// Updates statistics for each packet processed.
///
/// # Arguments
///
/// * `packets` - Vector of packets to process
/// * `count` - Number of duplicates to create for each selected packet
/// * `probability` - Probability of duplicating a packet
/// * `stats` - Statistics tracker to update
pub fn duplicate_packets(
    packets: &mut Vec<PacketData>,
    count: usize,
    probability: Probability,
    stats: &mut DuplicateStats,
) {
    let mut rng = rand::rng();
    let mut duplicate_packets = Vec::with_capacity(packets.len() * count);

    for packet_data in packets.iter() {
        if rng.random::<f64>() >= probability.value() {
            stats.record(1);
            continue;
        }

        for _ in 1..=count {
            duplicate_packets.push(PacketData::from(packet_data.packet.clone()));
        }

        stats.record(1 + count);
    }

    packets.extend(duplicate_packets);
}

#[cfg(test)]
mod tests {
    use crate::network::core::packet_data::PacketData;
    use crate::network::modules::duplicate::duplicate_packets;
    use crate::network::modules::stats::duplicate_stats::DuplicateStats;
    use crate::network::types::probability::Probability;
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    #[test]
    fn test_packet_duplication() {
        unsafe {
            let original_packets = vec![PacketData::from(WinDivertPacket::<NetworkLayer>::new(
                vec![1, 2, 3],
            ))];
            let original_len = original_packets.len();
            let mut packets = original_packets.clone();
            let mut stats = DuplicateStats::new(0.05);

            duplicate_packets(&mut packets, 3, Probability::new(1.0).unwrap(), &mut stats);

            // Ensure three times as many packets
            assert_eq!(packets.len(), original_len * 4);

            // Ensure data consistency
            for chunk in packets.chunks(original_len) {
                for packet_data in chunk.iter() {
                    assert_eq!(packet_data.packet.data[..], [1, 2, 3]);
                }
            }
        }
    }
}
