/// Statistics for tracking packet lag behavior.
///
/// This struct maintains statistics about packets currently being lagged
/// in the simulation.
#[derive(Debug)]
pub struct LagStats {
    /// Number of packets currently being lagged
    lagged_package_count: usize,

    /// Maximum number of packets that have been lagged simultaneously
    max_lagged: usize,

    /// Total number of packets that have been processed by the lag module
    total_processed: usize,
}

impl Default for LagStats {
    /// Creates a new LagStats instance with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl LagStats {
    /// Creates a new LagStats instance with zeroed counters.
    ///
    /// # Returns
    ///
    /// A new LagStats instance initialized with zero counts.
    ///
    /// # Example
    ///
    /// ```
    /// let stats = LagStats::new();
    /// ```
    pub fn new() -> Self {
        LagStats {
            lagged_package_count: 0,
            max_lagged: 0,
            total_processed: 0,
        }
    }

    /// Updates the count of currently lagged packets.
    ///
    /// # Arguments
    ///
    /// * `value` - The current number of packets being lagged
    ///
    /// # Example
    ///
    /// ```
    /// let mut stats = LagStats::new();
    /// stats.lagged_package_count(5);
    /// assert_eq!(stats.current_lagged(), 5);
    /// ```
    pub fn lagged_package_count(&mut self, value: usize) {
        self.lagged_package_count = value;

        // Update maximum count if current count is higher
        if value > self.max_lagged {
            self.max_lagged = value;
        }

        // Each call to this method represents a processing cycle
        self.total_processed += 1;
    }

    /// Returns the current number of packets being lagged.
    ///
    /// # Returns
    ///
    /// The count of packets currently in the lag queue.
    pub fn current_lagged(&self) -> usize {
        self.lagged_package_count
    }

    /// Returns the maximum number of packets that have been lagged simultaneously.
    ///
    /// # Returns
    ///
    /// The highest count of packets that have been in the lag queue at once.
    pub fn max_lagged(&self) -> usize {
        self.max_lagged
    }

    /// Returns the total number of processing cycles.
    ///
    /// # Returns
    ///
    /// The count of times the lag module has processed packets.
    pub fn total_processed(&self) -> usize {
        self.total_processed
    }

    /// Resets all statistics to zero.
    pub fn reset(&mut self) {
        self.lagged_package_count = 0;
        self.max_lagged = 0;
        self.total_processed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lag_stats() {
        let stats = LagStats::new();
        assert_eq!(stats.current_lagged(), 0);
        assert_eq!(stats.max_lagged(), 0);
        assert_eq!(stats.total_processed(), 0);
    }

    #[test]
    fn test_update_lag_stats() {
        let mut stats = LagStats::new();

        // First update
        stats.lagged_package_count(3);
        assert_eq!(stats.current_lagged(), 3);
        assert_eq!(stats.max_lagged(), 3);
        assert_eq!(stats.total_processed(), 1);

        // Second update (higher count)
        stats.lagged_package_count(5);
        assert_eq!(stats.current_lagged(), 5);
        assert_eq!(stats.max_lagged(), 5);
        assert_eq!(stats.total_processed(), 2);

        // Third update (lower count)
        stats.lagged_package_count(2);
        assert_eq!(stats.current_lagged(), 2);
        assert_eq!(stats.max_lagged(), 5); // Max should remain 5
        assert_eq!(stats.total_processed(), 3);
    }

    #[test]
    fn test_reset() {
        let mut stats = LagStats::new();

        // Add some data
        stats.lagged_package_count(5);

        // Reset
        stats.reset();

        // Verify all counters are zeroed
        assert_eq!(stats.current_lagged(), 0);
        assert_eq!(stats.max_lagged(), 0);
        assert_eq!(stats.total_processed(), 0);
    }
}
