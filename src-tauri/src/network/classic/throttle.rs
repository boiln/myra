//! Classic Throttle module processor.
//!
//! Buffers packets for a time window, then releases or drops them.
use crate::network::classic::state::ClassicThrottleState;
use crate::network::core::PacketData;
use crate::settings::classic::ClassicThrottleOptions;
use rand::Rng;
use std::time::{Duration, Instant};

/// Process packets through the Classic Throttle module.
pub fn process_throttle<'a>(
    packets: &mut Vec<PacketData<'a>>,
    options: &ClassicThrottleOptions,
    state: &mut ClassicThrottleState,
) {

    let mut rng = rand::rng();
    let window_duration = Duration::from_millis(options.window_ms);
    let now = Instant::now();
    let chance = options.chance / 100.0;

    // SAFETY: Storage outlives processing calls
    let buffer: &mut std::collections::VecDeque<PacketData<'a>> =
        unsafe { std::mem::transmute(&mut state.buffer) };

    // Check if we should start a new throttle window
    if state.window_start.is_none() {
        // Only start if we have packets and chance succeeds
        let has_matching_packets = packets.iter().any(|p| {
            (p.is_outbound && options.outbound) || (!p.is_outbound && options.inbound)
        });

        if has_matching_packets && rng.random::<f64>() < chance {
            state.window_start = Some(now);
            log::debug!("Classic throttle: starting {}ms window", options.window_ms);
        }
    }

    // If not throttling, pass all packets through
    let Some(window_start) = state.window_start else {
        return;
    };

    let mut passthrough = Vec::new();

    // Buffer matching packets during throttle window
    for packet in packets.drain(..) {
        let matches_direction = (packet.is_outbound && options.outbound)
            || (!packet.is_outbound && options.inbound);

        if !matches_direction {
            passthrough.push(packet);
            continue;
        }

        // Buffer this packet
        buffer.push_back(packet);
    }

    // Check if throttle window expired OR buffer is full
    let window_expired = now.duration_since(window_start) >= window_duration;
    let buffer_full = buffer.len() >= options.max_buffer;

    if window_expired || buffer_full {
        if buffer_full {
            log::debug!("Classic throttle: buffer full ({}), flushing", buffer.len());
        } else {
            log::debug!("Classic throttle: window expired, flushing {} packets", buffer.len());
        }

        if options.drop_on_release {
            // DROP all buffered packets
            log::debug!("Classic throttle: dropping {} buffered packets", buffer.len());
            buffer.clear();
        } else {
            // RELEASE all buffered packets as a burst
            log::debug!("Classic throttle: releasing {} packets as burst", buffer.len());
            while let Some(packet) = buffer.pop_front() {
                passthrough.push(packet);
            }
        }

        // Reset throttle state
        state.window_start = None;
    }

    packets.extend(passthrough);
}
