use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::throttle_stats::ThrottleStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::throttle::ThrottleOptions;
use log::{debug, info};
use rand::Rng;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Unit struct for the Throttle packet module.
///
/// This module implements throttling:
/// - Buffers packets during a timeframe
/// - When timeframe ends OR buffer is full, either releases or drops all packets
/// - Supports direction filtering (inbound/outbound)
#[derive(Debug, Default)]
pub struct ThrottleModule;

/// State maintained by the throttle module between processing calls.
#[derive(Debug)]
pub struct ThrottleState {
    /// Queue of buffered packets
    pub buffer: VecDeque<PacketData<'static>>,
    /// When the current throttle cycle started (None = not throttling)
    pub cycle_start: Option<Instant>,
    /// When the last flush occurred (for cooldown period)
    pub last_flush: Option<Instant>,
    /// When we last let a packet through as keepalive (during throttle)
    pub last_leak: Option<Instant>,
}

impl Default for ThrottleState {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            cycle_start: None,
            last_flush: None,
            last_leak: None,
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
    ) -> Result<()> {
        let mut stats = ctx.write_stats(self.name())?;

        let buffer: &mut VecDeque<PacketData<'a>> =
            unsafe { std::mem::transmute(&mut state.buffer) };

        throttle_packets(
            packets,
            buffer,
            &mut state.cycle_start,
            &mut state.last_flush,
            &mut state.last_leak,
            options.probability,
            Duration::from_millis(options.throttle_ms),
            options.drop,
            options.max_buffer,
            options.inbound,
            options.outbound,
            options.freeze_mode,
            &mut stats.throttle_stats,
        );
        Ok(())
    }
}

/// Network throttle implementation.
///
/// # How it works
///
/// 1. When a packet matching direction arrives, start buffering
/// 2. Buffer packets for the timeframe duration OR until max_buffer reached
/// 3. When timeframe ends or buffer full:
///    - If drop=true: DROP all buffered packets
///    - If drop=false: RELEASE all buffered packets at once
/// 4. Repeat based on probability
///
/// This creates a "stutter" effect - packets are held then burst released.
///
/// # Arguments
///
/// * `packets` - Packets to process
/// * `buffer` - Storage for buffered packets
/// * `cycle_start` - When current throttle cycle started
/// * `last_flush` - When the last flush occurred (for cooldown)
/// * `probability` - Chance of starting a new throttle cycle
/// * `timeframe` - How long to buffer packets
/// * `drop` - If true, drop buffered packets; if false, release them
/// * `max_buffer` - Maximum packets to buffer before forcing release/drop
/// * `apply_inbound` - Apply to inbound (download) packets
/// * `apply_outbound` - Apply to outbound (upload) packets
/// * `freeze_mode` - If true, disable cooldown for continuous buffering (freeze effect)
/// * `last_leak` - When we last let a packet through as keepalive
/// * `stats` - Statistics tracker
pub fn throttle_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<PacketData<'a>>,
    cycle_start: &mut Option<Instant>,
    last_flush: &mut Option<Instant>,
    last_leak: &mut Option<Instant>,
    probability: Probability,
    timeframe: Duration,
    drop: bool,
    max_buffer: usize,
    apply_inbound: bool,
    apply_outbound: bool,
    freeze_mode: bool,
    stats: &mut ThrottleStats,
) {
    let now = Instant::now();
    let cooldown = Duration::from_millis(40);
    let _ = last_leak; // Reserved for future use
    
    // Cooldown period after flush
    // In freeze_mode: No cooldown - continuous buffering creates freeze effect
    // In normal mode: 40ms cooldown - allows packets to flow, prevents disconnects
    let in_cooldown = !freeze_mode && last_flush
        .map(|flush_time| now.duration_since(flush_time) < cooldown)
        .unwrap_or(false);

    // Check if we need to release/drop buffered packets
    let should_flush = match cycle_start {
        Some(start) => {
            let elapsed = now.duration_since(*start);
            // Flush if timeframe elapsed OR buffer full
            elapsed >= timeframe || buffer.len() >= max_buffer
        }
        None => false,
    };

    // Track if we just flushed to prevent immediate re-buffering
    let mut just_flushed = false;
    
    if should_flush {
        let count = buffer.len();
        if drop {
            // Drop Throttled mode - discard all buffered packets
            info!("THROTTLE: Dropping {} buffered packets", count);
            buffer.clear();
            stats.dropped_count += count;
        } else {
            // Release mode - send all buffered packets at once
            info!("THROTTLE: Releasing {} buffered packets (throttle was {}ms)", 
                  count, timeframe.as_millis());
            while let Some(packet) = buffer.pop_front() {
                packets.push(packet);
            }
        }
        *cycle_start = None;
        *last_flush = Some(now); // Start cooldown
        stats.is_throttling = false;
        just_flushed = true; // Mark that we just flushed
    }

    // If not currently throttling and not in cooldown (or just flushed), check if we should start
    // In freeze_mode: can start immediately after flush (just_flushed doesn't block)
    // In normal mode: just_flushed prevents immediate restart, cooldown prevents on next call
    let can_start_new_cycle = if freeze_mode {
        cycle_start.is_none()
    } else {
        cycle_start.is_none() && !in_cooldown && !just_flushed
    };
    
    if can_start_new_cycle && rand::rng().random_bool(probability.value()) {
        *cycle_start = Some(now);
        info!("THROTTLE: Starting new {}ms throttle cycle", timeframe.as_millis());
    }
    
    if !can_start_new_cycle && in_cooldown {
        // During cooldown, packets pass through freely
        // This is logged only occasionally to avoid spam
        if stats.buffered_count > 0 {
            info!("THROTTLE: In cooldown, {} packets passing through", packets.len());
            stats.buffered_count = 0;
        }
    }

    // If throttling, buffer matching packets
    // But let very small packets through (ACKs, keepalives) to maintain connection
    const KEEPALIVE_THRESHOLD: usize = 80; // Packets <= this size pass through (ACKs, keepalives)
    
    if cycle_start.is_some() {
        let mut buffered_this_cycle = 0;
        let mut passthrough_count = 0;
        let mut i = 0;
        while i < packets.len() {
            let packet = &packets[i];

            // Check direction
            let should_buffer = (packet.is_outbound && apply_outbound)
                || (!packet.is_outbound && apply_inbound);

            if !should_buffer {
                i += 1;
                continue;
            }

            // Let very small packets through to keep connection alive (ACKs, TCP keepalives)
            // This mimics how NetLimiter keeps TCP happy at socket layer
            let packet_size = packet.packet.data.len();
            if packet_size <= KEEPALIVE_THRESHOLD {
                // Small packet - let it through as keepalive
                passthrough_count += 1;
                i += 1;
                continue;
            }

            // Check buffer limit
            if buffer.len() >= max_buffer {
                // Buffer full - stop buffering, will flush next cycle
                break;
            }

            let packet = packets.remove(i);
            let static_packet: PacketData<'static> = unsafe { std::mem::transmute(packet) };
            buffer.push_back(static_packet);
            buffered_this_cycle += 1;
        }

        if buffered_this_cycle > 0 || passthrough_count > 0 {
            debug!("THROTTLE: Buffered {} packets, {} small packets passed through (total buffered: {})", 
                  buffered_this_cycle, passthrough_count, buffer.len());
        }

        stats.is_throttling = true;
        stats.buffered_count = buffer.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    #[test]
    fn test_throttle_buffering() {
        unsafe {
            let mut packets = vec![
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]), true),
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![4, 5, 6]), true),
                PacketData::new(WinDivertPacket::<NetworkLayer>::new(vec![7, 8, 9]), true),
            ];
            let mut buffer = VecDeque::new();
            let mut cycle_start = Some(Instant::now());
            let mut last_flush = None;
            let mut last_leak = None;
            let mut stats = ThrottleStats::new();

            throttle_packets(
                &mut packets,
                &mut buffer,
                &mut cycle_start,
                &mut last_flush,
                &mut last_leak,
                Probability::new(1.0).unwrap(),
                Duration::from_secs(10), // Long timeframe
                false,
                2000,
                true,
                true,
                false, // freeze_mode
                &mut stats,
            );

            // All packets should be buffered
            assert_eq!(packets.len(), 0);
            assert_eq!(buffer.len(), 3);
            assert!(stats.is_throttling);
        }
    }

    #[test]
    fn test_throttle_release_on_timeframe_end() {
        unsafe {
            let mut packets = Vec::new();
            let mut buffer = VecDeque::new();
            buffer.push_back(PacketData::new(
                WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]),
                true,
            ));
            buffer.push_back(PacketData::new(
                WinDivertPacket::<NetworkLayer>::new(vec![4, 5, 6]),
                true,
            ));

            // Set cycle start in the past so timeframe has elapsed
            let mut cycle_start = Some(Instant::now() - Duration::from_secs(10));
            let mut last_flush = None;
            let mut last_leak = None;
            let mut stats = ThrottleStats::new();

            throttle_packets(
                &mut packets,
                &mut buffer,
                &mut cycle_start,
                &mut last_flush,
                &mut last_leak,
                Probability::new(0.0).unwrap(), // Don't start new cycle
                Duration::from_millis(100),     // Short timeframe (already elapsed)
                false,                          // Release mode
                2000,
                true,
                true,
                false, // freeze_mode
                &mut stats,
            );

            // Buffered packets should be released
            assert_eq!(packets.len(), 2);
            assert_eq!(buffer.len(), 0);
            // last_flush should be set after release
            assert!(last_flush.is_some());
        }
    }

    #[test]
    fn test_throttle_drop_mode() {
        unsafe {
            let mut packets = Vec::new();
            let mut buffer = VecDeque::new();
            buffer.push_back(PacketData::new(
                WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3]),
                true,
            ));

            let mut cycle_start = Some(Instant::now() - Duration::from_secs(10));
            let mut last_flush = None;
            let mut last_leak = None;
            let mut stats = ThrottleStats::new();

            throttle_packets(
                &mut packets,
                &mut buffer,
                &mut cycle_start,
                &mut last_flush,
                &mut last_leak,
                Probability::new(0.0).unwrap(),
                Duration::from_millis(100),
                true, // Drop mode
                2000,
                true,
                true,
                false, // freeze_mode
                &mut stats,
            );

            // Packets should be dropped, not released
            assert_eq!(packets.len(), 0);
            assert_eq!(buffer.len(), 0);
            assert_eq!(stats.dropped_count, 1);
        }
    }
}
