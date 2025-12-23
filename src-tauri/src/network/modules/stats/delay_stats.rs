/// Statistics for tracking packet delay behavior.
///
/// This struct maintains statistics about packets currently being delayed
/// in the simulation.
#[derive(Debug)]
pub struct DelayStats {
    /// Number of packets currently being delayed
    delayed_package_count: usize,
    
    /// Maximum number of packets that have been delayed simultaneously
    max_delayed: usize,
    
    /// Total number of packets that have been processed by the delay module
    total_processed: usize,
}

impl Default for DelayStats {
    /// Creates a new DelayStats instance with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl DelayStats {
    /// Creates a new DelayStats instance with zeroed counters.
    ///
    /// # Returns
    ///
    /// A new DelayStats instance initialized with zero counts.
    ///
    /// # Example
    ///
    /// ```
    /// let stats = DelayStats::new();
    /// ```
    pub fn new() -> Self {
        DelayStats {
            delayed_package_count: 0,
            max_delayed: 0,
            total_processed: 0,
        }
    }

    /// Updates the count of currently delayed packets.
    ///
    /// # Arguments
    ///
    /// * `value` - The current number of packets being delayed
    ///
    /// # Example
    ///
    /// ```
    /// let mut stats = DelayStats::new();
    /// stats.delayed_package_count(5);
    /// assert_eq!(stats.current_delayed(), 5);
    /// ```
    pub fn delayed_package_count(&mut self, value: usize) {
        self.delayed_package_count = value;
        
        // Update maximum count if current count is higher
        if value > self.max_delayed {
            self.max_delayed = value;
        }
        
        // Each call to this method represents a processing cycle
        self.total_processed += 1;
    }
    
    /// Returns the current number of packets being delayed.
    ///
    /// # Returns
    ///
    /// The count of packets currently in the delay queue.
    pub fn current_delayed(&self) -> usize {
        self.delayed_package_count
    }
    
    /// Returns the maximum number of packets that have been delayed simultaneously.
    ///
    /// # Returns
    ///
    /// The highest count of packets that have been in the delay queue at once.
    pub fn max_delayed(&self) -> usize {
        self.max_delayed
    }
    
    /// Returns the total number of processing cycles.
    ///
    /// # Returns
    ///
    /// The count of times the delay module has processed packets.
    pub fn total_processed(&self) -> usize {
        self.total_processed
    }
    
    /// Resets all statistics to zero.
    pub fn reset(&mut self) {
        self.delayed_package_count = 0;
        self.max_delayed = 0;
        self.total_processed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_delay_stats() {
        let stats = DelayStats::new();
        assert_eq!(stats.current_delayed(), 0);
        assert_eq!(stats.max_delayed(), 0);
        assert_eq!(stats.total_processed(), 0);
    }
    
    #[test]
    fn test_update_delay_stats() {
        let mut stats = DelayStats::new();
        
        // First update
        stats.delayed_package_count(3);
        assert_eq!(stats.current_delayed(), 3);
        assert_eq!(stats.max_delayed(), 3);
        assert_eq!(stats.total_processed(), 1);
        
        // Second update (higher count)
        stats.delayed_package_count(5);
        assert_eq!(stats.current_delayed(), 5);
        assert_eq!(stats.max_delayed(), 5);
        assert_eq!(stats.total_processed(), 2);
        
        // Third update (lower count)
        stats.delayed_package_count(2);
        assert_eq!(stats.current_delayed(), 2);
        assert_eq!(stats.max_delayed(), 5); // Max should remain 5
        assert_eq!(stats.total_processed(), 3);
    }
    
    #[test]
    fn test_reset() {
        let mut stats = DelayStats::new();
        
        // Add some data
        stats.delayed_package_count(5);
        
        // Reset
        stats.reset();
        
        // Verify all counters are zeroed
        assert_eq!(stats.current_delayed(), 0);
        assert_eq!(stats.max_delayed(), 0);
        assert_eq!(stats.total_processed(), 0);
    }
}
