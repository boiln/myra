//! Classic Latency module processor.
//!
//! Holds packets for a fixed duration before releasing them.
use crate::network::classic::state::ClassicLatencyState;
use crate::network::core::PacketData;
use crate::settings::classic::ClassicLatencyOptions;
use rand::Rng;
use std::time::{Duration, Instant};

/// Maximum packets to buffer before emergency release.
const MAX_BUFFER: usize = 15000;

/// Process packets through the Classic Latency module.
pub fn process_latency<'a>(
    packets: &mut Vec<PacketData<'a>>,
    options: &ClassicLatencyOptions,
    state: &mut ClassicLatencyState,
) {

    let mut rng = rand::rng();
    let lag_duration = Duration::from_millis(options.delay_ms);
    let now = Instant::now();
    let chance = options.chance / 100.0;

    // SAFETY: Storage outlives processing calls
    let buffer: &mut std::collections::VecDeque<(PacketData<'a>, Instant)> =
        unsafe { std::mem::transmute(&mut state.buffer) };

    let mut passthrough = Vec::new();

    // Move matching packets to lag buffer
    for packet in packets.drain(..) {
        let matches_direction = (packet.is_outbound && options.outbound)
            || (!packet.is_outbound && options.inbound);

        if !matches_direction {
            passthrough.push(packet);
            continue;
        }

        // Apply probability
        if rng.random::<f64>() >= chance {
            passthrough.push(packet);
            continue;
        }

        // Buffer this packet with current timestamp
        buffer.push_back((packet, now));
    }

    // Release packets that have been lagged long enough
    while let Some((_packet, capture_time)) = buffer.front() {
        if now.duration_since(*capture_time) < lag_duration {
            break; // Not ready yet (packets are ordered by time)
        }

        let (packet, _) = buffer.pop_front().unwrap();

        passthrough.push(packet);
    }

    // Emergency flush if buffer exceeds limit
    if buffer.len() > MAX_BUFFER {
        log::warn!("Classic latency buffer overflow, emergency release");

        while let Some((packet, _)) = buffer.pop_front() {
            passthrough.push(packet);
        }
    }

    packets.extend(passthrough);

}
