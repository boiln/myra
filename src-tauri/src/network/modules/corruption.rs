use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::corruption_stats::CorruptionStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::network::types::probability::Probability;
use crate::settings::corruption::CorruptionOptions;
use log::error;
use rand::{rng, Rng};
use std::collections::HashSet;
use windivert_sys::ChecksumFlags;

/// Unit struct for the Corruption packet module.
///
/// This module simulates packet corruption by randomly modifying
/// packet payload data.
#[derive(Debug, Default)]
pub struct CorruptionModule;

impl PacketModule for CorruptionModule {
    type Options = CorruptionOptions;
    type State = ();

    fn name(&self) -> &'static str {
        "corruption"
    }

    fn display_name(&self) -> &'static str {
        "Packet Corruption"
    }

    fn get_duration_ms(&self, options: &Self::Options) -> u64 {
        options.duration_ms
    }

    fn process(
        &self,
        packets: &mut Vec<PacketData<'_>>,
        options: &Self::Options,
        _state: &mut Self::State,
        ctx: &mut ModuleContext,
    ) -> Result<()> {
        let mut stats = ctx.write_stats(self.name())?;

        corruption_packets(
            packets,
            options.probability,
            options.amount,
            options.recalculate_checksums.unwrap_or(true),
            options.inbound,
            options.outbound,
            &mut stats.corruption_stats,
        );
        Ok(())
    }
}

/// Randomly corruptions with packet data based on specified probabilities
///
/// This function selectively modifies packet payload data to simulate corrupted network traffic.
/// It applies various corruptioning techniques (bit manipulation, bit flipping, value adjustment) to
/// the packet payloads based on the provided probabilities.
///
/// # Arguments
///
/// * `packets` - Slice of packet data to potentially corruption with
/// * `corruption_probability` - Probability of corruptioning with each packet
/// * `corruption_amount` - Proportion of bytes to corruption with in each selected packet
/// * `recalculate_checksums` - Whether to recalculate packet checksums after corruptioning
/// * `stats` - Statistics collector for corruptioning operations
///
/// # Example
///
/// ```
/// let mut packets = vec![packet1, packet2];
/// let corruption_probability = Probability::new(0.5).unwrap(); // 50% chance to corruption with a packet
/// let corruption_amount = Probability::new(0.1).unwrap(); // Modify approximately 10% of selected packets' bytes
/// let recalculate_checksums = true;
/// let mut stats = CorruptionStats::new(Duration::from_millis(100));
///
/// corruption_packets(
///     &mut packets,
///     corruption_probability,
///     corruption_amount,
///     recalculate_checksums,
///     &mut stats,
/// );
/// ```
pub fn corruption_packets(
    packets: &mut [PacketData],
    corruption_probability: Probability,
    corruption_amount: Probability,
    recalculate_checksums: bool,
    apply_inbound: bool,
    apply_outbound: bool,
    stats: &mut CorruptionStats,
) {
    let should_update_stats = stats.should_update();
    let mut rng = rng();

    for packet_data in packets.iter_mut() {
        // Check if this packet's direction should be affected
        let matches_direction = (packet_data.is_outbound && apply_outbound)
            || (!packet_data.is_outbound && apply_inbound);

        if !matches_direction {
            // Direction doesn't match - skip this packet
            continue;
        }

        let should_skip = rng.random::<f64>() >= corruption_probability.value();

        if should_skip && !should_update_stats {
            continue;
        }

        let data = packet_data.packet.data.to_mut();

        let (ip_header_len, protocol) = match get_ip_version(data) {
            Some((4, data)) => parse_ipv4_header(data),
            Some((6, data)) => parse_ipv6_header(data),
            _ => {
                error!("Unsupported IP version");
                continue;
            }
        };

        let total_header_len = match protocol {
            17 => parse_udp_header(data, ip_header_len),
            6 => parse_tcp_header(data, ip_header_len),
            _ => ip_header_len,
        };

        let payload_offset = total_header_len;
        let payload_length = data.len() - payload_offset;

        if should_skip {
            if !should_update_stats {
                continue;
            }

            stats.data = data[payload_offset..].to_owned();
            stats.corruption_flags = vec![false; stats.data.len()];
            stats.checksum_valid = true;
            stats.updated();
            continue;
        }

        if payload_length > 0 {
            let bytes_to_corruption = (payload_length as f64 * corruption_amount.value()) as usize;
            let corruptioned_indices = apply_corruptioning(&mut data[payload_offset..], bytes_to_corruption);

            if should_update_stats {
                let corruptioned_flags = calculate_corruptioned_flags(data.len(), &corruptioned_indices);
                stats.corruption_flags = corruptioned_flags;
                stats.data = data[payload_offset..].to_owned();
                stats.updated();
            }
        }

        if recalculate_checksums {
            if let Err(e) = packet_data
                .packet
                .recalculate_checksums(ChecksumFlags::new())
            {
                error!("Error recalculating checksums: {}", e);
            }
        }

        if !should_update_stats {
            continue;
        }

        stats.checksum_valid = packet_data.packet.address.ip_checksum()
            && packet_data.packet.address.tcp_checksum()
            && packet_data.packet.address.udp_checksum();
        stats.updated();
    }
}

/// Applies random corruptioning to a slice of data
///
/// This function implements the actual corruptioning logic, selecting random bytes
/// and applying different types of modifications.
///
/// # Arguments
///
/// * `data` - The data slice to be corruptioned with
/// * `bytes_to_corruption` - The number of bytes to corruption with
///
/// # Returns
///
/// A `HashSet` containing the indices of all modified bytes
fn apply_corruptioning(data: &mut [u8], bytes_to_corruption: usize) -> HashSet<usize> {
    let mut corruptioned_indices = HashSet::new();
    let mut corruptioned_count = 0;
    let data_len = data.len();
    let mut rng = rng();

    while corruptioned_count < bytes_to_corruption && corruptioned_count < data_len {
        let index = rng.random_range(0..data.len());
        if corruptioned_indices.insert(index) {
            corruptioned_count += 1;
            let corruption_type = rng.random_range(0..3);
            let modified_indices = match corruption_type {
                0 => bit_manipulation(data, index, rng.random_range(0..8), true),
                1 => bit_flipping(data, index, rng.random_range(0..8)),
                2 => value_adjustment(data, index, rng.random_range(-64..64)),
                _ => vec![],
            };
            corruptioned_indices.extend(modified_indices);
        }
    }

    corruptioned_indices
}

/// Creates a vector of boolean flags indicating which bytes were corruptioned with
///
/// # Arguments
///
/// * `data_len` - Total length of the data
/// * `corruptioned_indices` - Set of indices that were corruptioned with
///
/// # Returns
///
/// A vector of boolean flags where true indicates a corruptioned byte
fn calculate_corruptioned_flags(data_len: usize, corruptioned_indices: &HashSet<usize>) -> Vec<bool> {
    let mut corruptioned_flags = vec![false; data_len];
    for &index in corruptioned_indices {
        if index < data_len {
            corruptioned_flags[index] = true;
        }
    }
    corruptioned_flags
}

/// Extracts the IP version from a packet data slice
///
/// # Arguments
///
/// * `data` - Packet data slice
///
/// # Returns
///
/// Option containing a tuple of (IP version, data slice reference) if successful
fn get_ip_version(data: &[u8]) -> Option<(u8, &[u8])> {
    if data.is_empty() {
        return None;
    }
    let version = data[0] >> 4;
    Some((version, data))
}

/// Parses an IPv4 header to extract header length and protocol
///
/// # Arguments
///
/// * `data` - Packet data slice starting at the IPv4 header
///
/// # Returns
///
/// A tuple of (header length in bytes, protocol number)
fn parse_ipv4_header(data: &[u8]) -> (usize, u8) {
    let header_length = ((data[0] & 0x0F) * 4) as usize;
    let protocol = data[9];
    (header_length, protocol)
}

/// Parses an IPv6 header to extract header length and next header type
///
/// # Arguments
///
/// * `data` - Packet data slice starting at the IPv6 header
///
/// # Returns
///
/// A tuple of (header length in bytes, next header type)
fn parse_ipv6_header(data: &[u8]) -> (usize, u8) {
    let header_length = 40; // IPv6 header is always 40 bytes
    let next_header = data[6];
    (header_length, next_header)
}

/// Calculates the total header length for a UDP packet
///
/// # Arguments
///
/// * `_data` - Packet data slice (unused but kept for consistency)
/// * `ip_header_len` - Length of the IP header in bytes
///
/// # Returns
///
/// Total header length (IP header + UDP header) in bytes
fn parse_udp_header(_data: &[u8], ip_header_len: usize) -> usize {
    let udp_header_len = 8; // UDP header is always 8 bytes
    ip_header_len + udp_header_len
}

/// Calculates the total header length for a TCP packet
///
/// # Arguments
///
/// * `data` - Packet data slice
/// * `ip_header_len` - Length of the IP header in bytes
///
/// # Returns
///
/// Total header length (IP header + TCP header) in bytes
fn parse_tcp_header(data: &[u8], ip_header_len: usize) -> usize {
    let tcp_data_offset = (data[ip_header_len + 12] >> 4) * 4;
    ip_header_len + tcp_data_offset as usize
}

/// Manipulates a specific bit in a byte to a specified value
///
/// # Arguments
///
/// * `data` - Data slice to modify
/// * `byte_index` - Index of the byte to modify
/// * `bit_position` - Position of the bit to set/clear (0-7)
/// * `new_bit` - The new bit value (true = 1, false = 0)
///
/// # Returns
///
/// A vector containing the index of the modified byte, or empty if no modification occurred
fn bit_manipulation(
    data: &mut [u8],
    byte_index: usize,
    bit_position: usize,
    new_bit: bool,
) -> Vec<usize> {
    if byte_index >= data.len() || bit_position >= 8 {
        return vec![];
    }

    if new_bit {
        data[byte_index] |= 1 << bit_position; // Set the bit
    }

    if !new_bit {
        data[byte_index] &= !(1 << bit_position); // Clear the bit
    }

    vec![byte_index]
}

/// Flips a specific bit in a byte (0 becomes 1, 1 becomes 0)
///
/// # Arguments
///
/// * `data` - Data slice to modify
/// * `byte_index` - Index of the byte to modify
/// * `bit_position` - Position of the bit to flip (0-7)
///
/// # Returns
///
/// A vector containing the index of the modified byte, or empty if no modification occurred
fn bit_flipping(data: &mut [u8], byte_index: usize, bit_position: usize) -> Vec<usize> {
    if byte_index >= data.len() || bit_position >= 8 {
        return vec![];
    }

    data[byte_index] ^= 1 << bit_position;
    vec![byte_index]
}

/// Adjusts a byte value by adding a signed offset
///
/// # Arguments
///
/// * `data` - Data slice to modify
/// * `offset` - Index of the byte to modify
/// * `value` - Signed value to add to the byte
///
/// # Returns
///
/// A vector containing the index of the modified byte, or empty if no modification occurred
fn value_adjustment(data: &mut [u8], offset: usize, value: i8) -> Vec<usize> {
    if offset >= data.len() {
        return vec![];
    }

    let adjusted_value = data[offset].wrapping_add(value as u8);
    data[offset] = adjusted_value;
    vec![offset]
}
