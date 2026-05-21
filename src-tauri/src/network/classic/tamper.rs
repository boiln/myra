//! Classic Tamper module processor.
//!
//! XORs packet payload data with a rotating pattern.
use crate::network::classic::state::ClassicTamperState;
use crate::network::core::PacketData;
use crate::settings::classic::ClassicTamperOptions;
use rand::Rng;

/// Process packets through the Classic Tamper module.
pub fn process_tamper<'a>(
    packets: &mut Vec<PacketData<'a>>,
    options: &ClassicTamperOptions,
    state: &mut ClassicTamperState,
) {

    let mut rng = rand::rng();
    let chance = options.chance / 100.0;

    for packet in packets.iter_mut() {
        let matches_direction =
            (packet.is_outbound && options.outbound) || (!packet.is_outbound && options.inbound);

        if !matches_direction {
            continue;
        }

        if rng.random::<f64>() >= chance {
            continue;
        }

        // Get mutable access to packet data
        let data = packet.packet.data.to_mut();

        // Find the payload start (skip IP + TCP/UDP headers)
        // Minimum IPv4 header is 20 bytes
        if data.len() < 20 {
            continue;
        }

        let ip_version = (data[0] >> 4) & 0x0F;
        let header_len = match ip_version {
            4 => {
                let ihl = (data[0] & 0x0F) as usize * 4;
                let protocol = data[9];

                let transport_header = match protocol {
                    6 => 20, // TCP minimum header
                    17 => 8, // UDP header
                    _ => 0,
                };

                ihl + transport_header
            }
            6 => {
                // IPv6 has 40 byte fixed header + extension headers (simplified)
                let next_header = data[6];
                let transport_header = match next_header {
                    6 => 20, // TCP
                    17 => 8, // UDP
                    _ => 0,
                };
                40 + transport_header
            }
            _ => continue,
        };

        if data.len() <= header_len {
            continue; // No payload to tamper
        }

        let payload_start = header_len;
        let payload_len = data.len() - payload_start;

        if payload_len == 0 {
            continue;
        }

        // Apply XOR tampering based on payload size
        if payload_len < 5 {
            // Small packet: XOR entire payload
            for i in 0..payload_len {
                let pattern_idx = (state.pattern_index + i) % 8;

                data[payload_start + i] ^= state.patterns[pattern_idx];
            }
            state.pattern_index = (state.pattern_index + payload_len) % 8;
        } else {
            // Larger packet: XOR middle ~25% section
            let start_offset = (payload_len / 2) + 1 - (payload_len / 8);
            let tamper_len = payload_len / 4;

            for i in 0..tamper_len {
                let data_idx = payload_start + start_offset + i;

                if data_idx < data.len() {
                    let pattern_idx = (state.pattern_index + i) % 8;

                    data[data_idx] ^= state.patterns[pattern_idx];
                }
            }
            state.pattern_index = (state.pattern_index + tamper_len) % 8;
        }

        // Recalculate checksums if enabled
        if options.recalc_checksum {
            recalculate_checksums(data);
        }
    }

}

/// Recalculate IP and transport layer checksums.
fn recalculate_checksums(data: &mut [u8]) {

    if data.len() < 20 {
        return;
    }

    let ip_version = (data[0] >> 4) & 0x0F;

    if ip_version == 4 {
        // Zero out IP checksum (bytes 10-11)
        data[10] = 0;
        data[11] = 0;

        // Calculate IP header checksum
        let ihl = (data[0] & 0x0F) as usize * 4;
        let checksum = calculate_checksum(&data[..ihl]);

        data[10] = (checksum >> 8) as u8;
        data[11] = (checksum & 0xFF) as u8;

        // Handle transport layer checksum
        let protocol = data[9];

        match protocol {
            6 => {
                // TCP
                if data.len() >= ihl + 20 {
                    // Zero TCP checksum (offset 16-17 from TCP header start)
                    data[ihl + 16] = 0;
                    data[ihl + 17] = 0;
                    // Note: Full TCP checksum requires pseudo-header, simplified here
                }
            }
            17 => {
                // UDP
                if data.len() >= ihl + 8 {
                    // Zero UDP checksum (offset 6-7 from UDP header start)
                    data[ihl + 6] = 0;
                    data[ihl + 7] = 0;
                    // UDP checksum is optional for IPv4
                }
            }
            _ => {}
        }
    }

}

/// Calculate internet checksum.
fn calculate_checksum(data: &[u8]) -> u16 {

    let mut sum: u32 = 0;
    let mut i = 0;

    while i < data.len() - 1 {
        sum += u32::from(data[i]) << 8 | u32::from(data[i + 1]);
        i += 2;
    }

    if i < data.len() {
        sum += u32::from(data[i]) << 8;
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16

}
