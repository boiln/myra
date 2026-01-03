use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::burst_stats::BurstStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::burst::BurstOptions;
use log::debug;
use rand::{rng, Rng};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Unit struct for the Burst packet module.
///
/// This module implements a "lag switch" by buffering packets for a
/// specified duration and then releasing them all at once, creating
/// a teleport/burst effect in games.
#[derive(Debug, Default)]
pub struct BurstModule;

/// State maintained by the burst module between processing calls.
#[derive(Debug)]
pub struct BurstState {
    /// Queue of buffered packets with their capture time
    pub buffer: VecDeque<(PacketData<'static>, Instant)>,
    /// When the current burst cycle started
    pub cycle_start: Option<Instant>,
    /// When the last keepalive packet was sent
    pub last_keepalive: Option<Instant>,
}

impl Default for BurstState {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            cycle_start: None,
            last_keepalive: None,
        }
    }
}

impl PacketModule for BurstModule {
    type Options = BurstOptions;
    type State = BurstState;

    fn name(&self) -> &'static str {
        "burst"
    }

    fn display_name(&self) -> &'static str {
        "Packet Burst"
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
        let buffer: &mut VecDeque<(PacketData<'a>, Instant)> =
            unsafe { std::mem::transmute(&mut state.buffer) };

        burst_packets(
            packets,
            buffer,
            &mut state.cycle_start,
            &mut state.last_keepalive,
            Duration::from_millis(options.buffer_ms),
            Duration::from_millis(options.keepalive_ms),
            options.probability,
            options.inbound,
            options.outbound,
            options.reverse,
            &mut stats.burst_stats,
        );
        Ok(())
    }
}

/// Implements packet bursting by buffering packets then releasing all at once.
///
/// # How it works
///
/// **Timed mode (buffer_ms > 0):**
/// 1. Buffer packets for the specified duration
/// 2. Release ALL packets at once when timer expires
/// 3. Start new cycle
///
/// **Manual mode (buffer_ms = 0):**
/// 1. Buffer all packets indefinitely
/// 2. Release happens when module is disabled (call flush_buffer)
///
/// **Keepalive (keepalive_ms > 0):**
/// Lets one packet through every keepalive_ms to prevent disconnection
///
/// **Direction filtering:**
/// - `apply_inbound`: Buffer inbound (download) packets
/// - `apply_outbound`: Buffer outbound (upload) packets
/// - Packets not matching direction settings pass through unmodified
///
/// This creates the "teleport" effect - your actions are recorded locally,
/// then all sent at once when the buffer releases.
pub fn burst_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<(PacketData<'a>, Instant)>,
    cycle_start: &mut Option<Instant>,
    last_keepalive: &mut Option<Instant>,
    buffer_duration: Duration,
    keepalive_duration: Duration,
    probability: Probability,
    apply_inbound: bool,
    apply_outbound: bool,
    reverse: bool,
    stats: &mut BurstStats,
) {
    let now = Instant::now();
    let mut rng = rng();

    // Initialize cycle if not started
    if cycle_start.is_none() {
        *cycle_start = Some(now);
    }

    // Check if we need to send a keepalive (let one packet through)
    let send_keepalive = keepalive_duration.as_millis() > 0 && match last_keepalive {
        None => {
            *last_keepalive = Some(now);
            false
        }
        Some(last) if now.duration_since(*last) >= keepalive_duration => {
            *last_keepalive = Some(now);
            true
        }
        Some(_) => false,
    };

    // If keepalive is due, find a packet that matches direction and preserve it
    let keepalive_packet = match send_keepalive && !packets.is_empty() {
        false => None,
        true => packets.iter().position(|p| {
            (p.is_outbound && apply_outbound) || (!p.is_outbound && apply_inbound)
        }).map(|idx| packets.remove(idx)),
    };

    // Buffer packets based on probability AND direction
    let mut i = 0;
    while i < packets.len() {
        let packet = &packets[i];
        
        // Check if this packet's direction should be buffered
        let should_buffer_direction = 
            (packet.is_outbound && apply_outbound) || 
            (!packet.is_outbound && apply_inbound);
        
        if !should_buffer_direction {
            // Direction doesn't match - let packet through
            i += 1;
            continue;
        }
        
        if rng.random::<f64>() >= probability.value() {
            // Probability says don't buffer
            i += 1;
            continue;
        }
        
        let packet = packets.remove(i);
        let static_packet: PacketData<'static> = unsafe { std::mem::transmute(packet) };
        buffer.push_back((static_packet, now));
        stats.record_buffer(1);
    }

    // Restore keepalive packet at the front to be sent
    if let Some(first_packet) = keepalive_packet {
        packets.insert(0, first_packet);
    }

    // THEN: Check if it's time to release (only in timed mode)
    // buffer_duration of 0 means "manual mode" - hold until toggled off
    if buffer_duration.as_millis() > 0 {
        let cycle_started = cycle_start.unwrap();
        let elapsed = now.duration_since(cycle_started);

        if elapsed >= buffer_duration && !buffer.is_empty() {
            let released_count = buffer.len();
            
            debug!(
                "BURST: Releasing {} packets after {}ms buffer (reverse={})",
                released_count,
                elapsed.as_millis(),
                reverse
            );
            
            // Collect packets to release
            let mut released_packets: Vec<_> = buffer.drain(..).map(|(p, _)| p).collect();
            
            // In reverse mode, reverse the order of released packets
            if reverse {
                released_packets.reverse();
                debug!("BURST: Reversed {} packets for rewind effect", released_packets.len());
            }
            
            packets.extend(released_packets);

            stats.record_release(released_count);
            *cycle_start = Some(now);
        }
    }

    stats.set_buffered_count(buffer.len());
}

/// Flushes all buffered packets - called when module is disabled
/// Returns the packets to be sent with pacing
/// 
/// # Arguments
/// * `packets` - Output vector to add flushed packets to (new packets from this cycle)
/// * `buffer` - The buffer containing packets to flush
/// * `cycle_start` - Cycle start time to reset
/// * `reverse` - If true, release packets in reverse order (rewind effect)
pub fn flush_buffer<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<(PacketData<'a>, Instant)>,
    cycle_start: &mut Option<Instant>,
    reverse: bool,
) {
    if buffer.is_empty() {
        debug!("BURST FLUSH: Buffer is empty, nothing to flush");
        return;
    }

    let buffer_count = buffer.len();
    let new_packet_count = packets.len();
    
    // Collect all packets from buffer (oldest first)
    let mut released_packets: Vec<_> = buffer.drain(..).map(|(p, _)| p).collect();
    
    // Apply reverse if requested (for rewind effect)
    if reverse {
        released_packets.reverse();
        debug!("BURST FLUSH: Reversed {} packets for rewind effect", buffer_count);
    }
    
    // IMPORTANT: Buffered packets must be sent FIRST, before any new packets from this cycle
    // This ensures proper replay order: old actions → new actions
    // We prepend by: taking new packets out, adding buffered, then adding new back
    let new_packets: Vec<_> = packets.drain(..).collect();
    packets.extend(released_packets);
    packets.extend(new_packets);
    
    debug!(
        "BURST FLUSH: Released {} buffered packets, {} new packets queued after (reverse={})",
        buffer_count,
        new_packet_count,
        reverse
    );

    *cycle_start = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    #[test]
    fn test_packet_buffering() {
        unsafe {
            // Create outbound packets for testing
            let mut packets = vec![
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]), true),
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![4, 5, 6]), true),
            ];
            let mut buffer = VecDeque::new();
            let mut cycle_start = None;
            let mut last_keepalive = None;
            let mut stats = BurstStats::new(0.05);

            // Buffer with 100% probability, both directions
            burst_packets(
                &mut packets,
                &mut buffer,
                &mut cycle_start,
                &mut last_keepalive,
                Duration::from_millis(1000),
                Duration::from_millis(0), // No keepalive
                Probability::new(1.0).unwrap(),
                true,  // apply_inbound
                true,  // apply_outbound
                false, // reverse
                &mut stats,
            );

            // All packets should be buffered
            assert_eq!(packets.len(), 0);
            assert_eq!(buffer.len(), 2);
        }
    }
}
