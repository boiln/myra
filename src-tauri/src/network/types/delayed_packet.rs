use crate::network::core::PacketData;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

/// A packet with a delay time for network condition simulation
///
/// Represents a packet to be delayed for a specified amount of time.
/// Used in reordering functionality to delay packet delivery.
/// Implements `Ord` and `PartialOrd` to enable use in a priority queue.
#[derive(Debug)]
pub struct DelayedPacket<'a> {
    /// The packet data to be delivered
    pub packet: PacketData<'a>,
    /// Timestamp representing when this packet should be delivered
    pub delay_until: Instant,
}

impl PartialEq for DelayedPacket<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.delay_until == other.delay_until
    }
}

impl Eq for DelayedPacket<'_> {}

impl PartialOrd for DelayedPacket<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DelayedPacket<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Note: We flip the ordering here to turn BinaryHeap into a min-heap based on delay_until
        other.delay_until.cmp(&self.delay_until)
    }
}

impl<'a> DelayedPacket<'a> {
    /// Creates a new delayed packet
    ///
    /// # Arguments
    ///
    /// * `packet` - The packet to be delayed
    /// * `delay` - How long to delay the packet
    ///
    /// # Returns
    ///
    /// A new `DelayedPacket` with delivery time set to now + delay
    pub fn new(packet: PacketData<'a>, delay: Duration) -> Self {
        Self {
            packet,
            delay_until: Instant::now() + delay,
        }
    }
}
