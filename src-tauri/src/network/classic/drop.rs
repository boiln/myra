//! Classic Drop module processor.
//!
//! Probabilistically drops packets immediately.
use crate::network::core::PacketData;
use crate::settings::classic::ClassicDropOptions;
use rand::Rng;

/// Process packets through the Classic Drop module.
pub fn process_drop<'a>(
    packets: &mut Vec<PacketData<'a>>,
    options: &ClassicDropOptions,
) {

    let mut rng = rand::rng();
    let chance = options.chance / 100.0;

    packets.retain(|packet| {

        let matches_direction = (packet.is_outbound && options.outbound)
            || (!packet.is_outbound && options.inbound);

        if !matches_direction {
            return true; // Keep packet (wrong direction)
        }

        // Drop based on probability
        if rng.random::<f64>() < chance {
            return false; // Drop packet
        }

        true // Keep packet

    });

}
