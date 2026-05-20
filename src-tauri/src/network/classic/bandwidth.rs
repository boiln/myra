//! Classic Bandwidth module processor.
//!
//! Rate-limits by bytes per second using token bucket algorithm.
use crate::network::classic::state::ClassicBandwidthState;
use crate::network::core::PacketData;
use crate::settings::classic::ClassicBandwidthOptions;
use std::time::Instant;

/// Process packets through the Classic Bandwidth module.
pub fn process_bandwidth<'a>(
    packets: &mut Vec<PacketData<'a>>,
    options: &ClassicBandwidthOptions,
    state: &mut ClassicBandwidthState,
) {

    let now = Instant::now();

    // Calculate byte budget based on elapsed time
    let elapsed_ms = state.last_tick.elapsed().as_millis() as f64;

    state.last_tick = now;

    // Convert: elapsed_ms * (limit_kbps KB/s) * (1024 bytes/KB) * (1s/1000ms)
    // = elapsed_ms * limit_kbps * 1.024
    let bytes_earned = elapsed_ms * options.limit_kbps * 1.024;

    state.byte_budget += bytes_earned;

    // Cap budget to prevent accumulation (max 1 second worth)
    let max_budget = options.limit_kbps * 1024.0;

    if state.byte_budget > max_budget {
        state.byte_budget = max_budget;
    }

    // SAFETY: Storage outlives processing calls
    let buffer: &mut std::collections::VecDeque<PacketData<'a>> =

        unsafe { std::mem::transmute(&mut state.buffer) };

    let mut output = Vec::new();
    let mut bytes_used: f64 = 0.0;

    // First, release buffered packets within budget
    while let Some(packet) = buffer.front() {
        let packet_len = packet.packet.data.len() as f64;

        if bytes_used + packet_len > state.byte_budget {
            break; // Over budget
        }

        let packet = buffer.pop_front().unwrap();

        bytes_used += packet_len;
        output.push(packet);
    }

    // Process new incoming packets
    for packet in packets.drain(..) {
        let matches_direction = (packet.is_outbound && options.outbound)

            || (!packet.is_outbound && options.inbound);

        if !matches_direction {
            output.push(packet);
            continue;
        }

        let packet_len = packet.packet.data.len() as f64;

        // Check if within budget
        if bytes_used + packet_len <= state.byte_budget {
            bytes_used += packet_len;
            output.push(packet);
        } else if buffer.len() < options.max_buffer {
            // Over budget but buffer has space
            buffer.push_back(packet);
        } else {
            // Buffer full - DROP packet
            log::debug!("Classic bandwidth: dropping packet (buffer full at {})", options.max_buffer);
        }
    }

    // Update remaining budget
    state.byte_budget -= bytes_used;

    if state.byte_budget < 0.0 {
        state.byte_budget = 0.0;
    }

    *packets = output;

}
