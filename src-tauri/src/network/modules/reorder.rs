use crate::network::core::packet_data::PacketData;
use crate::network::modules::stats::reorder_stats::ReorderStats;
use crate::network::types::delayed_packet::DelayedPacket;
use crate::network::types::probability::Probability;
use log::{debug, error, warn};
use rand::{rng, Rng};
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

/// Reorders packets based on specified probability and delay parameters
///
/// Selectively delays packets according to provided probability, creating
/// a packet reordering effect. Packets with a delay are stored in the
/// binary heap until their delay time has elapsed.
///
/// # Arguments
///
/// * `packets` - Packets to potentially reorder
/// * `storage` - Binary heap for delayed packet storage
/// * `reorder_probability` - Probability of delaying a packet
/// * `max_delay` - Maximum delay duration
/// * `stats` - Statistics tracker to update
pub fn reorder_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    storage: &mut BinaryHeap<DelayedPacket<'a>>,
    reorder_probability: Probability,
    max_delay: Duration,
    stats: &mut ReorderStats,
) {
    if max_delay.as_millis() == 0 {
        warn!("Max delay cannot be zero. Skipping packet reordering.");
        return;
    }

    debug!(
        "Reorder: processing {} packets, storage has {}, max_delay={}ms, prob={}",
        packets.len(),
        storage.len(),
        max_delay.as_millis(),
        reorder_probability.value()
    );

    let mut skipped_packets = Vec::new();
    let mut rng = rng();
    let mut delayed_count = 0;

    for packet in packets.drain(..) {
        if rng.random::<f64>() >= reorder_probability.value() {
            skipped_packets.push(packet);
            stats.record(false);
            continue;
        }

        let delay_max = max_delay.as_millis() as u64;
        let delay_millis = rng.random_range(0..delay_max);
        let delay = Duration::from_millis(delay_millis);
        let delayed_packet = DelayedPacket::new(packet, delay);

        storage.push(delayed_packet);
        stats.record(true);
        delayed_count += 1;
    }

    stats.delayed_packets = storage.len();

    if delayed_count > 0 {
        debug!(
            "Reorder: delayed {} packets, now {} in storage",
            delayed_count,
            storage.len()
        );
    }

    packets.append(&mut skipped_packets);

    let now = Instant::now();
    let mut released_count = 0;

    while let Some(delayed_packet) = storage.peek() {
        if delayed_packet.delay_until > now {
            break;
        }

        if let Some(delayed_packet) = storage.pop() {
            packets.push(delayed_packet.packet);
            released_count += 1;
            continue;
        }

        error!("Expected a delayed packet, but none was found in storage.");
        break;
    }

    if released_count > 0 {
        debug!(
            "Reorder: released {} packets, {} still in storage",
            released_count,
            storage.len()
        );
    }
}
