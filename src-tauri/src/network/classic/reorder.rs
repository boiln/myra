//! Classic Reorder module processor.
//!
//! Swaps adjacent packets to create out-of-order delivery.
use crate::network::classic::state::ClassicReorderState;
use crate::network::core::PacketData;
use crate::settings::classic::ClassicReorderOptions;
use rand::Rng;

/// Process packets through the Classic Reorder module.
pub fn process_reorder<'a>(
    packets: &mut Vec<PacketData<'a>>,
    options: &ClassicReorderOptions,
    state: &mut ClassicReorderState,
) {

    let mut rng = rand::rng();
    let chance = options.chance / 100.0;

    // SAFETY: Storage outlives processing calls
    let held_packet: &mut Option<PacketData<'a>> =
        unsafe { std::mem::transmute(&mut state.held_packet) };

    // If we're holding a packet, check if we should release it
    if held_packet.is_some() {
        state.hold_cycles += 1;

        // Release if we have new packets OR exceeded hold limit
        let has_new_packets = !packets.is_empty();
        if has_new_packets || state.hold_cycles >= options.max_hold_cycles {
            // Insert held packet at the front (it was originally first)
            if let Some(packet) = held_packet.take() {
                packets.insert(0, packet);
            }
            state.hold_cycles = 0;
        }
        return;
    }

    // Filter packets by direction
    let matching_indices: Vec<usize> = packets
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            (p.is_outbound && options.outbound) || (!p.is_outbound && options.inbound)
        })
        .map(|(i, _)| i)
        .collect();

    if matching_indices.is_empty() {
        return;
    }

    // If only one matching packet, consider holding it
    if matching_indices.len() == 1 {
        if rng.random::<f64>() < chance {
            let idx = matching_indices[0];
            *held_packet = Some(packets.remove(idx));
            state.hold_cycles = 0;
        }
        return;
    }

    // Multiple matching packets - swap adjacent pairs
    if rng.random::<f64>() >= chance {
        return; // Chance failed, don't reorder
    }

    // Swap adjacent matching packets
    let mut i = 0;
    while i + 1 < matching_indices.len() {
        if rng.random::<f64>() < chance {
            let idx1 = matching_indices[i];
            let idx2 = matching_indices[i + 1];
            packets.swap(idx1, idx2);
        }
        i += 1;
    }

}
