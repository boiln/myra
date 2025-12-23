/// Statistics for network throttling operations
///
/// This struct tracks statistics related to network throttling, including:
/// - Whether throttling is currently active
/// - The total number of packets dropped due to throttling
#[derive(Debug)]
pub struct ThrottleStats {
    /// Flag indicating whether throttling is currently active
    pub(crate) is_throttling: bool,
    
    /// Total number of packets dropped due to throttling
    pub(crate) dropped_count: usize,
}

impl Default for ThrottleStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ThrottleStats {
    /// Creates a new ThrottleStats instance with default values
    ///
    /// # Returns
    ///
    /// A new ThrottleStats instance with throttling disabled and zero dropped packets
    ///
    /// # Example
    ///
    /// ```
    /// let stats = ThrottleStats::new();
    /// ```
    pub fn new() -> Self {
        ThrottleStats {
            is_throttling: false,
            dropped_count: 0,
        }
    }
    
    /// Returns whether throttling is currently active
    ///
    /// # Returns
    ///
    /// `true` if throttling is currently active, `false` otherwise
    pub fn is_throttling(&self) -> bool {
        self.is_throttling
    }
    
    /// Returns the total number of packets dropped due to throttling
    ///
    /// # Returns
    ///
    /// The total number of packets that have been dropped
    pub fn dropped_count(&self) -> usize {
        self.dropped_count
    }
    
    /// Resets all statistics to their default values
    ///
    /// This resets the throttling status to inactive and the dropped count to zero
    pub fn reset(&mut self) {
        self.is_throttling = false;
        self.dropped_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new() {
        let stats = ThrottleStats::new();
        assert!(!stats.is_throttling(), "New stats should not be throttling");
        assert_eq!(stats.dropped_count(), 0, "New stats should have 0 dropped packets");
    }
    
    #[test]
    fn test_reset() {
        let mut stats = ThrottleStats {
            is_throttling: true,
            dropped_count: 10,
        };
        
        stats.reset();
        
        assert!(!stats.is_throttling(), "Stats should not be throttling after reset");
        assert_eq!(stats.dropped_count(), 0, "Stats should have 0 dropped packets after reset");
    }
}
