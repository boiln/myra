use std::time::Instant;
use windivert::layer::NetworkLayer;
use windivert::packet::WinDivertPacket;

/// Represents a network packet with metadata for processing.
///
/// This structure wraps a WinDivert packet and associates it with
/// timing information, which is crucial for implementing various
/// network condition simulations like delays and bandwidth limits.
#[derive(Debug, Clone)]
pub struct PacketData<'a> {
    /// The actual network packet from WinDivert
    pub packet: WinDivertPacket<'a, NetworkLayer>,
    
    /// Timestamp when the packet was captured
    pub arrival_time: Instant,
}

impl<'a> From<WinDivertPacket<'a, NetworkLayer>> for PacketData<'a> {
    /// Creates a PacketData instance from a WinDivertPacket, 
    /// automatically recording the current time as arrival time.
    fn from(packet: WinDivertPacket<'a, NetworkLayer>) -> Self {
        PacketData {
            packet,
            arrival_time: Instant::now(),
        }
    }
}

/// Methods for working with packet data
impl<'a> PacketData<'a> {
    /// Returns the size of the packet in bytes
    pub fn size(&self) -> usize {
        self.packet.data.len()
    }
    
    /// Returns the time elapsed since the packet was captured
    pub fn age(&self) -> std::time::Duration {
        self.arrival_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_data_creation() {
        unsafe {
            let dummy_packet = WinDivertPacket::<NetworkLayer>::new(vec![1, 2, 3, 4]);
            let packet_data = PacketData::from(dummy_packet);
            
            // Assert that the packet data is correctly assigned
            assert_eq!(packet_data.packet.data.len(), 4);
            assert_eq!(packet_data.packet.data[..], [1, 2, 3, 4]);

            // Check that size() returns the correct value
            assert_eq!(packet_data.size(), 4);

            // Verify that the arrival time is recent
            assert!(packet_data.age().as_secs() < 1);
        }
    }
}
