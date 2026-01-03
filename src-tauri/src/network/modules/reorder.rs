use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::reorder_stats::ReorderStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::delayed_packet::DelayedPacket;
use crate::network::types::probability::Probability;
use crate::settings::reorder::ReorderOptions;
use log::{debug, error, warn};
use rand::{rng, Rng};
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

/// Unit struct for the Reorder packet module.
///
/// This module simulates packet reordering by delaying packets
/// by random amounts, causing them to arrive out of order.
#[derive(Debug, Default)]
pub struct ReorderModule;

/// State maintained by the reorder module between processing calls.
pub type ReorderState = BinaryHeap<DelayedPacket<'static>>;

impl PacketModule for ReorderModule {
    type Options = ReorderOptions;
    type State = ReorderState;

    fn name(&self) -> &'static str {
        "reorder"
    }

    fn display_name(&self) -> &'static str {
        "Packet Reorder"
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
        // across processing calls.
        let storage: &mut BinaryHeap<DelayedPacket<'a>> = unsafe { std::mem::transmute(state) };

        reorder_packets(
            packets,
            storage,
            options.probability,
            Duration::from_millis(options.max_delay),
            options.inbound,
            options.outbound,
            &mut stats.reorder_stats,
        );
        Ok(())
    }
}

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
/// * `apply_inbound` - Whether to affect inbound packets
/// * `apply_outbound` - Whether to affect outbound packets
/// * `stats` - Statistics tracker to update
pub fn reorder_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    storage: &mut BinaryHeap<DelayedPacket<'a>>,
    reorder_probability: Probability,
    max_delay: Duration,
    apply_inbound: bool,
    apply_outbound: bool,
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
        // Check if this packet's direction should be affected
        let matches_direction = (packet.is_outbound && apply_outbound)
            || (!packet.is_outbound && apply_inbound);

        if !matches_direction {
            // Direction doesn't match - let packet pass through
            skipped_packets.push(packet);
            continue;
        }

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
    let mut released_packets = Vec::new();

    while let Some(delayed_packet) = storage.peek() {
        if delayed_packet.delay_until > now {
            break;
        }

        let Some(delayed_packet) = storage.pop() else {
            error!("Expected a delayed packet, but none was found in storage.");
            break;
        };
        
        released_packets.push(delayed_packet.packet);
        released_count += 1;
    }

    packets.extend(released_packets);

    if released_count > 0 {
        debug!(
            "Reorder: released {} packets, {} still in storage",
            released_count,
            storage.len()
        );
    }
}
