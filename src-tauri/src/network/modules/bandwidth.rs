use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::stats::bandwidth_stats::BandwidthStats;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::settings::bandwidth::BandwidthOptions;
use std::collections::VecDeque;
use std::time::Instant;

/// Maximum size of the packet buffer in bytes (100 MB)
/// Increased from 10MB to prevent packet drops during heavy throttling
/// When this limit is exceeded, oldest packets will be dropped from the buffer
const MAX_BUFFER_SIZE: usize = 100 * 1024 * 1024; // 100 MB in bytes

/// Unit struct for the Bandwidth packet module.
///
/// This module simulates bandwidth limitations using a token bucket
/// algorithm to control the rate at which packets are released.
#[derive(Debug, Default)]
pub struct BandwidthModule;

/// A packet with its scheduled release time for pacing
#[derive(Debug)]
struct ScheduledPacket<'a> {
    packet: PacketData<'a>,
    release_time: Instant,
}

/// State maintained by the bandwidth module between processing calls.
#[derive(Debug)]
pub struct BandwidthState {
    pub buffer: VecDeque<PacketData<'static>>,
    pub total_buffer_size: usize,
    pub last_send_time: Instant,
    /// For packet pacing mode: tracks when the next packet can be released
    pub next_release_time: Instant,
}

impl Default for BandwidthState {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            total_buffer_size: 0,
            last_send_time: Instant::now(),
            next_release_time: Instant::now(),
        }
    }
}

impl PacketModule for BandwidthModule {
    type Options = BandwidthOptions;
    type State = BandwidthState;

    fn name(&self) -> &'static str {
        "bandwidth"
    }

    fn display_name(&self) -> &'static str {
        "Bandwidth Limit"
    }

    fn get_duration_ms(&self, options: &Self::Options) -> u64 {
        options.duration_ms
    }

    fn should_skip(&self, options: &Self::Options) -> bool {
        // Skip if limit is 0 OR if using WFP mode (external throttle handles it)
        options.limit == 0 || options.use_wfp
    }

    fn process<'a>(
        &self,
        packets: &mut Vec<PacketData<'a>>,
        options: &Self::Options,
        state: &mut Self::State,
        ctx: &mut ModuleContext,
    ) -> Result<()> {
        let mut stats = ctx.write_stats(self.name())?;

        // Safety: We need to transmute lifetimes here because the buffer persists
        // across processing calls.
        let buffer: &mut VecDeque<PacketData<'a>> =
            unsafe { std::mem::transmute(&mut state.buffer) };

        bandwidth_limiter(
            packets,
            buffer,
            &mut state.total_buffer_size,
            &mut state.next_release_time,
            options.limit,
            options.inbound,
            options.outbound,
            options.passthrough_threshold,
            &mut stats.bandwidth_stats,
        );
        Ok(())
    }
}

/// Limits network bandwidth by controlling the rate at which packets are released
///
/// This function implements a token bucket algorithm to limit bandwidth. It buffers incoming
/// packets and releases them at a rate determined by the specified bandwidth limit.
///
/// # Arguments
///
/// * `packets` - Mutable vector that initially contains incoming packets and will contain outgoing packets after the function runs
/// * `buffer` - Queue used to store packets that exceed the current bandwidth allowance
/// * `total_buffer_size` - Running total of the buffer size in bytes
/// * `last_send_time` - The time when packets were last sent, used to calculate allowable bytes
/// * `bandwidth_limit_kbps` - The maximum bandwidth allowed in kilobits per second
/// * `apply_inbound` - Whether to apply bandwidth limiting to inbound (download) traffic
/// * `apply_outbound` - Whether to apply bandwidth limiting to outbound (upload) traffic
/// * `stats` - Statistics tracker for bandwidth usage
///
/// # Example
///
/// ```
/// let mut packets = vec![packet1, packet2];
/// let mut buffer = VecDeque::new();
/// let mut total_buffer_size = 0;
/// let mut last_send_time = Instant::now();
/// let bandwidth_limit_kbps = 1000; // 1 Mbps
/// let mut stats = BandwidthStats::new(0.5);
///
/// bandwidth_limiter(
///     &mut packets,
///     &mut buffer,
///     &mut total_buffer_size,
///     &mut last_send_time,
///     bandwidth_limit_kbps,
///     true,  // apply to inbound
///     true,  // apply to outbound
///     &mut stats,
/// );
/// ```
pub fn bandwidth_limiter<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<PacketData<'a>>,
    total_buffer_size: &mut usize,
    last_send_time: &mut Instant,
    bandwidth_limit_kbps: usize,
    apply_inbound: bool,
    apply_outbound: bool,
    passthrough_threshold: usize,
    stats: &mut BandwidthStats,
) {
    bandwidth_limiter_paced(
        packets,
        buffer,
        total_buffer_size,
        last_send_time,
        bandwidth_limit_kbps,
        apply_inbound,
        apply_outbound,
        passthrough_threshold,
        stats,
    )
}

/// Packet-pacing bandwidth limiter (like NetLimiter)
/// Releases packets one at a time at smooth intervals based on rate limit
/// This keeps the connection alive and provides continuous data flow
fn bandwidth_limiter_paced<'a>(
    packets: &mut Vec<PacketData<'a>>,
    buffer: &mut VecDeque<PacketData<'a>>,
    total_buffer_size: &mut usize,
    next_release_time: &mut Instant,
    bandwidth_limit_kbps: usize,
    apply_inbound: bool,
    apply_outbound: bool,
    passthrough_threshold: usize,
    stats: &mut BandwidthStats,
) {
    // Separate packets by direction and size
    // Small packets (ACKs, keepalives) pass through to keep connection alive
    let mut passthrough = Vec::new();
    let mut to_buffer = Vec::new();
    
    for packet in packets.drain(..) {
        let packet_size = packet.packet.data.len();
        let matches_direction = (packet.is_outbound && apply_outbound) 
            || (!packet.is_outbound && apply_inbound);
        
        // Small packets always pass through (keepalives, ACKs)
        // This keeps the connection alive like NetLimiter does
        let is_small = passthrough_threshold > 0 && packet_size <= passthrough_threshold;
        
        if matches_direction && !is_small {
            to_buffer.push(packet);
        } else {
            // Packets not matching direction OR small keepalive packets pass through
            passthrough.push(packet);
        }
    }
    
    stats.storage_packet_count += to_buffer.len();

    add_packets_to_buffer(buffer, &mut to_buffer, total_buffer_size);
    maintain_buffer_size(buffer, total_buffer_size, stats);

    let now = Instant::now();
    let mut to_send = Vec::new();
    let mut bytes_sent = 0;
    
    // Packet pacing: release packets when their "transmission time" has passed
    // At 1 KB/s, a 500 byte packet takes 500ms to "transmit"
    // This mimics NetLimiter's smooth rate limiting
    
    // Only release if we've reached the next release time
    if now >= *next_release_time {
        if let Some(packet) = remove_packet_from_buffer(buffer, total_buffer_size, stats) {
            let packet_size = packet.packet.data.len();
            bytes_sent = packet_size;
            
            // Calculate how long this packet "takes" to transmit at our rate
            // At 1 KB/s (1024 bytes/sec), 500 bytes takes 500/1024 = 0.488 seconds
            let bytes_per_sec = (bandwidth_limit_kbps as f64) * 1024.0;
            let transmission_time_secs = if bytes_per_sec > 0.0 {
                packet_size as f64 / bytes_per_sec
            } else {
                1.0 // Default to 1 second if rate is 0
            };
            
            // Schedule next release after this packet's "transmission time"
            let transmission_duration = std::time::Duration::from_secs_f64(transmission_time_secs);
            *next_release_time = now + transmission_duration;
            
            to_send.push(packet);
        }
    }

    // Add passthrough packets first, then rate-limited packets
    packets.extend(passthrough);
    packets.extend(to_send);

    if bytes_sent > 0 {
        stats.record(bytes_sent);
    }
}

/// Adds a single packet to the buffer and updates the total buffer size
///
/// # Arguments
///
/// * `buffer` - The packet buffer
/// * `packet` - The packet to add
/// * `total_size` - Running total of the buffer size in bytes
fn add_packet_to_buffer<'a>(
    buffer: &mut VecDeque<PacketData<'a>>,
    packet: PacketData<'a>,
    total_size: &mut usize,
) {
    *total_size += packet.packet.data.len();
    buffer.push_back(packet);
}

/// Moves all packets from the input vector to the buffer
///
/// This function consumes the packets from the input vector by popping them one by one
/// and adding them to the buffer. The input vector will be empty after this operation.
///
/// # Arguments
///
/// * `buffer` - The packet buffer
/// * `packets` - Vector of packets to add to the buffer
/// * `total_size` - Running total of the buffer size in bytes
fn add_packets_to_buffer<'a>(
    buffer: &mut VecDeque<PacketData<'a>>,
    packets: &mut Vec<PacketData<'a>>,
    total_size: &mut usize,
) {
    while let Some(packet) = packets.pop() {
        add_packet_to_buffer(buffer, packet, total_size);
    }
}

/// Removes a packet from the front of the buffer and updates the total buffer size
///
/// # Arguments
///
/// * `buffer` - The packet buffer
/// * `total_size` - Running total of the buffer size in bytes
/// * `stats` - Statistics tracker to update
///
/// # Returns
///
/// * `Option<PacketData<'a>>` - The removed packet, or None if the buffer is empty
fn remove_packet_from_buffer<'a>(
    buffer: &mut VecDeque<PacketData<'a>>,
    total_size: &mut usize,
    stats: &mut BandwidthStats,
) -> Option<PacketData<'a>> {
    let packet = buffer.pop_front()?;

    *total_size -= packet.packet.data.len();
    stats.storage_packet_count = stats.storage_packet_count.saturating_sub(1);

    Some(packet)
}

/// Ensures the buffer doesn't exceed the maximum size by removing packets if necessary
///
/// # Arguments
///
/// * `buffer` - The packet buffer
/// * `total_size` - Running total of the buffer size in bytes
/// * `stats` - Statistics tracker to update
fn maintain_buffer_size(
    buffer: &mut VecDeque<PacketData<'_>>,
    total_size: &mut usize,
    stats: &mut BandwidthStats,
) {
    while *total_size > MAX_BUFFER_SIZE {
        if remove_packet_from_buffer(buffer, total_size, stats).is_none() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::core::packet_data::PacketData;
    use crate::network::modules::bandwidth::{
        add_packet_to_buffer, add_packets_to_buffer, bandwidth_limiter, remove_packet_from_buffer,
        MAX_BUFFER_SIZE,
    };
    use std::collections::VecDeque;
    use std::time::Duration;
    use windivert::layer::NetworkLayer;
    use windivert::packet::WinDivertPacket;

    /// Safely creates a dummy packet with a specified length.
    /// Assumes the vector created with the specified length is valid for packet creation.
    fn create_dummy_packet<'a>(length: usize) -> WinDivertPacket<'a, NetworkLayer> {
        let data = vec![1; length];
        unsafe { WinDivertPacket::<NetworkLayer>::new(data) }
    }

    #[test]
    fn test_basic_bandwidth_limiting() {
        let mut packets = vec![
            PacketData::from(create_dummy_packet(1000)),
            PacketData::from(create_dummy_packet(1000)),
        ];
        let mut buffer = VecDeque::new();
        let total_buffer_size: &mut usize = &mut 0usize;
        let mut last_send_time = Instant::now() - Duration::from_secs(1);
        let bandwidth_limit = 1; // 1 KB/s
        let mut stats = BandwidthStats::new(0.5);

        bandwidth_limiter(
            &mut packets,
            &mut buffer,
            total_buffer_size,
            &mut last_send_time,
            bandwidth_limit,
            true,  // apply inbound
            true,  // apply outbound
            0,     // no passthrough threshold
            &mut stats,
        );

        assert!(packets.len() <= 1);
    }

    #[test]
    fn test_exceeding_buffer_size() {
        let mut packets = Vec::new();
        let mut buffer = VecDeque::new();
        let mut total_buffer_size = 0;

        // Fill the buffer with packets to exceed the max total size
        while total_buffer_size < MAX_BUFFER_SIZE + 10_000 {
            let packet = PacketData::from(create_dummy_packet(1000));
            total_buffer_size += packet.packet.data.len();
            buffer.push_back(packet);
        }
        let mut last_send_time = Instant::now();
        let bandwidth_limit = 100; // High enough to not limit the test
        let mut stats = BandwidthStats::new(0.5);

        bandwidth_limiter(
            &mut packets,
            &mut buffer,
            &mut total_buffer_size,
            &mut last_send_time,
            bandwidth_limit,
            true,
            true,
            0,
            &mut stats,
        );

        let actual_total_size: usize = buffer.iter().map(|p| p.packet.data.len()).sum();
        assert!(actual_total_size <= MAX_BUFFER_SIZE);
    }

    #[test]
    fn test_no_bandwidth_limiting() {
        let mut packets = vec![
            PacketData::from(create_dummy_packet(1000)),
            PacketData::from(create_dummy_packet(1000)),
        ];
        let mut buffer = VecDeque::new();
        let mut total_buffer_size = 0;
        let mut last_send_time = Instant::now() - Duration::from_secs(1);
        let bandwidth_limit = 10_000; // 10 MB/s
        let mut stats = BandwidthStats::new(0.5);

        bandwidth_limiter(
            &mut packets,
            &mut buffer,
            &mut total_buffer_size,
            &mut last_send_time,
            bandwidth_limit,
            true,
            true,
            0,
            &mut stats,
        );

        assert_eq!(packets.len(), 2);
    }

    #[test]
    fn test_zero_bandwidth() {
        let mut packets = vec![
            PacketData::from(create_dummy_packet(1000)),
            PacketData::from(create_dummy_packet(1000)),
        ];
        let mut buffer = VecDeque::new();
        let mut total_buffer_size = 0;
        let mut last_send_time = Instant::now();
        let bandwidth_limit = 0; // 0 KB/s
        let mut stats = BandwidthStats::new(0.5);

        bandwidth_limiter(
            &mut packets,
            &mut buffer,
            &mut total_buffer_size,
            &mut last_send_time,
            bandwidth_limit,
            true,
            true,
            0,
            &mut stats,
        );

        assert!(packets.is_empty());
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_empty_packet_vector() {
        let mut packets = Vec::new();
        let mut buffer = VecDeque::new();
        let mut total_buffer_size = 0;
        let mut last_send_time = Instant::now();
        let bandwidth_limit = 10_000; // 10 MB/s
        let mut stats = BandwidthStats::new(0.5);

        bandwidth_limiter(
            &mut packets,
            &mut buffer,
            &mut total_buffer_size,
            &mut last_send_time,
            bandwidth_limit,
            true,
            true,
            0,
            &mut stats,
        );

        // Since the packets vector was empty, buffer should remain empty and nothing should be sent
        assert!(packets.is_empty());
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_add_packet_to_buffer() {
        let mut buffer = VecDeque::new();
        let mut total_size = 0;
        let packet = PacketData::from(create_dummy_packet(1000));

        add_packet_to_buffer(&mut buffer, packet.clone(), &mut total_size);

        assert_eq!(buffer.len(), 1);
        assert_eq!(total_size, 1000);
        assert_eq!(buffer.front().unwrap().packet.data.len(), 1000);
    }

    #[test]
    fn test_add_packets_to_buffer() {
        let mut buffer = VecDeque::new();
        let mut total_size = 0;
        let mut packets = vec![
            PacketData::from(create_dummy_packet(1000)),
            PacketData::from(create_dummy_packet(2000)),
        ];

        add_packets_to_buffer(&mut buffer, &mut packets, &mut total_size);

        assert_eq!(buffer.len(), 2);
        assert_eq!(total_size, 3000);
        assert_eq!(buffer.pop_front().unwrap().packet.data.len(), 2000);
        assert_eq!(buffer.pop_front().unwrap().packet.data.len(), 1000);
    }

    #[test]
    fn test_remove_packet_from_buffer() {
        let mut buffer = VecDeque::new();
        let mut total_size = 0;
        let packet = PacketData::from(create_dummy_packet(1000));
        add_packet_to_buffer(&mut buffer, packet.clone(), &mut total_size);
        let mut stats = BandwidthStats::new(0.5);

        let removed_packet = remove_packet_from_buffer(&mut buffer, &mut total_size, &mut stats);

        assert_eq!(removed_packet.unwrap().packet.data.len(), 1000);
        assert_eq!(buffer.len(), 0);
        assert_eq!(total_size, 0);
    }

    #[test]
    fn test_remove_packet_from_empty_buffer() {
        let mut buffer = VecDeque::new();
        let mut total_size = 0;
        let mut stats = BandwidthStats::new(0.5);

        let removed_packet = remove_packet_from_buffer(&mut buffer, &mut total_size, &mut stats);

        assert!(removed_packet.is_none());
        assert_eq!(buffer.len(), 0);
        assert_eq!(total_size, 0);
    }
}
