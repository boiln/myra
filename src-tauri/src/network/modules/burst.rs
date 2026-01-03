use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::burst_stats::BurstStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::burst::BurstOptions;
use log::{debug, info};
use rand::{rng, Rng};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Unit struct for the Burst packet module.
///
/// This module implements a "lag switch" by buffering packets for a
/// specified duration and then releasing them, creating a teleport/burst
/// effect in games. Supports variable replay speeds and reverse playback.
#[derive(Debug, Default)]
pub struct BurstModule;

/// State maintained by the burst module between processing calls.
#[derive(Debug)]
pub struct BurstState {
    /// Queue of buffered packets with their capture time
    pub buffer: VecDeque<(PacketData<'static>, Instant)>,
    /// When the current burst cycle started
    pub cycle_start: Option<Instant>,
    /// Accumulated time between packets for replay pacing
    pub replay_queue: VecDeque<(PacketData<'static>, Duration)>,
    /// When we last released a packet during replay
    pub last_release: Option<Instant>,
}

impl Default for BurstState {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            cycle_start: None,
            replay_queue: VecDeque::new(),
            last_release: None,
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
        let replay_queue: &mut VecDeque<(PacketData<'a>, Duration)> =
            unsafe { std::mem::transmute(&mut state.replay_queue) };

        burst_packets(
            packets,
            buffer,
            replay_queue,
            &mut state.cycle_start,
            &mut state.last_release,
            Duration::from_millis(options.buffer_ms),
            options.probability,
            options.replay_speed,
            options.reverse_replay,
            options.inbound,
            options.outbound,
            &mut stats.burst_stats,
        );
        Ok(())
    }
}

/// Implements packet bursting with variable replay speed and reverse mode.
///
/// # How it works
///
/// **Timed mode (buffer_ms > 0):**
/// 1. Buffer packets for the specified duration, recording inter-packet timing
/// 2. When timer expires, prepare replay queue
/// 3. Release packets according to replay_speed
///
/// **Manual mode (buffer_ms = 0):**
/// 1. Buffer all packets indefinitely
/// 2. Release happens when module is disabled (call flush_buffer)
///
/// **Replay Speed:**
/// - 1.0 = real-time (packets released at original timing)
/// - 2.0 = 2x speed (half the delay between packets)
/// - 0.5 = half speed (double the delay)
/// - 0.0 = instant (all at once)
///
/// **Reverse Replay:**
/// - If enabled, packets are released in LIFO order (last captured = first released)
/// - Creates a "rewind" effect
///
/// **Direction filtering:**
/// - `apply_inbound`: Buffer inbound (download) packets
/// - `apply_outbound`: Buffer outbound (upload) packets
#[allow(clippy::too_many_arguments)]
pub fn burst_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<(PacketData<'a>, Instant)>,
    replay_queue: &mut VecDeque<(PacketData<'a>, Duration)>,
    cycle_start: &mut Option<Instant>,
    last_release: &mut Option<Instant>,
    buffer_duration: Duration,
    probability: Probability,
    replay_speed: f64,
    reverse_replay: bool,
    apply_inbound: bool,
    apply_outbound: bool,
    stats: &mut BurstStats,
) {
    let now = Instant::now();
    let mut rng = rng();

    // First: Process any ongoing replay
    if !replay_queue.is_empty() {
        release_from_replay_queue(packets, replay_queue, last_release, replay_speed, stats);
    }

    // Initialize cycle if not started and not replaying
    if cycle_start.is_none() && replay_queue.is_empty() {
        *cycle_start = Some(now);
    }

    // Buffer packets based on probability AND direction
    let mut i = 0;
    while i < packets.len() {
        let packet = &packets[i];
        
        // Check if this packet's direction should be buffered
        let should_buffer_direction = 
            (packet.is_outbound && apply_outbound) || 
            (!packet.is_outbound && apply_inbound);
        
        if !should_buffer_direction {
            i += 1;
            continue;
        }
        
        if rng.random::<f64>() >= probability.value() {
            i += 1;
            continue;
        }
        
        let packet = packets.remove(i);
        let static_packet: PacketData<'static> = unsafe { std::mem::transmute(packet) };
        buffer.push_back((static_packet, now));
        stats.record_buffer(1);
    }

    // Check if it's time to release (only in timed mode)
    // buffer_duration of 0 means "manual mode" - hold until toggled off
    if buffer_duration.as_millis() > 0 && replay_queue.is_empty() {
        let Some(cycle_started) = *cycle_start else {
            return;
        };
        
        let elapsed = now.duration_since(cycle_started);
        if elapsed < buffer_duration || buffer.is_empty() {
            stats.set_buffered_count(buffer.len());
            return;
        }

        // Time to start replay - prepare the queue
        info!(
            "BURST: Starting replay of {} packets (speed={}, reverse={})",
            buffer.len(),
            replay_speed,
            reverse_replay
        );
        
        prepare_replay_queue(buffer, replay_queue, reverse_replay);
        *cycle_start = None;
        *last_release = Some(now);
        
        // Release first batch immediately
        release_from_replay_queue(packets, replay_queue, last_release, replay_speed, stats);
    }

    stats.set_buffered_count(buffer.len() + replay_queue.len());
}

/// Converts buffer to replay queue with inter-packet timing
fn prepare_replay_queue<'a>(
    buffer: &mut VecDeque<(PacketData<'a>, Instant)>,
    replay_queue: &mut VecDeque<(PacketData<'a>, Duration)>,
    reverse: bool,
) {
    if buffer.is_empty() {
        return;
    }

    // Calculate delays between consecutive packets
    let mut packets_with_delays: Vec<(PacketData<'a>, Duration)> = Vec::with_capacity(buffer.len());
    let mut prev_time: Option<Instant> = None;

    for (packet, capture_time) in buffer.drain(..) {
        let delay = match prev_time {
            Some(pt) => capture_time.saturating_duration_since(pt),
            None => Duration::ZERO,
        };
        prev_time = Some(capture_time);
        packets_with_delays.push((packet, delay));
    }

    // Reverse if requested
    if reverse {
        packets_with_delays.reverse();
    }

    replay_queue.extend(packets_with_delays);
}

/// Releases packets from replay queue according to timing and speed
fn release_from_replay_queue<'a>(
    packets: &mut Vec<PacketData<'a>>,
    replay_queue: &mut VecDeque<(PacketData<'a>, Duration)>,
    last_release: &mut Option<Instant>,
    replay_speed: f64,
    stats: &mut BurstStats,
) {
    let now = Instant::now();
    
    // Instant release mode
    if replay_speed <= 0.0 {
        let count = replay_queue.len();
        while let Some((packet, _)) = replay_queue.pop_front() {
            packets.push(packet);
        }
        stats.record_release(count);
        *last_release = Some(now);
        return;
    }

    // Paced release based on original timing
    loop {
        let Some((_, delay)) = replay_queue.front() else {
            break;
        };

        // Calculate scaled delay
        let scaled_delay = Duration::from_secs_f64(delay.as_secs_f64() / replay_speed);
        
        // Check if enough time has passed
        let time_since_last = match last_release {
            Some(lr) => now.saturating_duration_since(*lr),
            None => Duration::MAX, // First packet releases immediately
        };

        if time_since_last < scaled_delay {
            break;
        }

        // Release this packet
        let Some((packet, _)) = replay_queue.pop_front() else {
            break;
        };
        
        packets.push(packet);
        stats.record_release(1);
        *last_release = Some(now);
    }
}

/// Flushes all buffered packets - called when module is disabled
pub fn flush_buffer<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<(PacketData<'a>, Instant)>,
    replay_queue: &mut VecDeque<(PacketData<'a>, Duration)>,
    cycle_start: &mut Option<Instant>,
) {
    // Flush main buffer
    while let Some((packet, _)) = buffer.pop_front() {
        packets.push(packet);
    }
    
    // Flush replay queue
    while let Some((packet, _)) = replay_queue.pop_front() {
        packets.push(packet);
    }

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
            let mut packets = vec![
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]), true),
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![4, 5, 6]), true),
            ];
            let mut buffer = VecDeque::new();
            let mut replay_queue = VecDeque::new();
            let mut cycle_start = None;
            let mut last_release = None;
            let mut stats = BurstStats::new(0.05);

            burst_packets(
                &mut packets,
                &mut buffer,
                &mut replay_queue,
                &mut cycle_start,
                &mut last_release,
                Duration::from_millis(1000),
                Probability::new(1.0).unwrap(),
                1.0,   // replay_speed
                false, // reverse_replay
                true,  // apply_inbound
                true,  // apply_outbound
                &mut stats,
            );

            assert_eq!(packets.len(), 0);
            assert_eq!(buffer.len(), 2);
        }
    }
}
