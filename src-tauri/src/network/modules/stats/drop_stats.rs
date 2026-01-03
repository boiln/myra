use crate::network::modules::stats::util::ewma::Ewma;

/// Statistics for tracking packet dropping behavior.
///
/// This struct maintains both aggregate statistics (total counts)
/// and recent trends using an exponentially weighted moving average (EWMA).
///
/// # Fields
///
/// * `total_packets` - Total number of packets processed
/// * `total_dropped` - Total number of packets dropped
/// * `ewma` - Exponentially weighted moving average for smoothing recent drop rates
#[derive(Debug)]
pub struct DropStats {
    /// Total number of packets processed
    pub total_packets: usize,

    /// Total number of packets dropped
    pub total_dropped: usize,

    /// EWMA for recent drop rate calculations
    ewma: Ewma,
}

impl DropStats {
    /// Creates a new DropStats instance with the specified alpha parameter for EWMA.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The smoothing factor (0.0-1.0) for the EWMA calculation.
    ///   Higher values give more weight to recent observations.
    ///
    /// # Returns
    ///
    /// A new DropStats instance initialized with zeroed counters.
    ///
    /// # Example
    ///
    /// ```
    /// let stats = DropStats::new(0.3); // EWMA with alpha = 0.3
    /// ```
    pub fn new(alpha: f64) -> Self {
        Self {
            total_packets: 0,
            total_dropped: 0,
            ewma: Ewma::new(alpha),
        }
    }

    /// Records a packet processing result, updating all statistics.
    ///
    /// # Arguments
    ///
    /// * `dropped` - Whether the packet was dropped (`true`) or not (`false`)
    ///
    /// # Example
    ///
    /// ```
    /// let mut stats = DropStats::new(0.3);
    /// stats.record(true);  // Record a dropped packet
    /// stats.record(false); // Record a non-dropped packet
    /// ```
    pub fn record(&mut self, dropped: bool) {
        self.total_packets += 1;
        if dropped {
            self.total_dropped += 1;
        }

        // Update the EWMA with the new drop status (1.0 if dropped, 0.0 if not)
        let current_drop_rate = match dropped {
            true => 1.0,
            false => 0.0,
        };
        self.ewma.update(current_drop_rate);
    }

    /// Calculates the overall drop rate since tracking began.
    ///
    /// # Returns
    ///
    /// A value between 0.0 and 1.0 representing the fraction of packets
    /// that have been dropped. Returns 0.0 if no packets have been processed.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stats = DropStats::new(0.3);
    /// stats.record(true);  // Dropped
    /// stats.record(false); // Not dropped
    /// assert_eq!(stats.total_drop_rate(), 0.5); // 50% drop rate
    /// ```
    pub fn total_drop_rate(&self) -> f64 {
        if self.total_packets == 0 {
            return 0.0;
        }

        self.total_dropped as f64 / self.total_packets as f64
    }

    /// Gets the recent drop rate based on the EWMA.
    ///
    /// This provides a smoothed view of recent drop behavior,
    /// giving more weight to recent packet processing results.
    ///
    /// # Returns
    ///
    /// A value between 0.0 and 1.0 representing the recent
    /// drop rate. Returns 0.0 if no packets have been processed.
    pub fn recent_drop_rate(&self) -> f64 {
        self.ewma.get().unwrap_or(0.0)
    }

    /// Resets all statistics to zero.
    ///
    /// This clears both the total counters and resets the EWMA.
    pub fn reset(&mut self) {
        self.total_packets = 0;
        self.total_dropped = 0;
        // Reset the EWMA to its initial state
        self.ewma.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_drop_stats() {
        let stats = DropStats::new(0.5);
        assert_eq!(stats.total_packets, 0);
        assert_eq!(stats.total_dropped, 0);
        assert_eq!(stats.total_drop_rate(), 0.0);
        assert_eq!(stats.recent_drop_rate(), 0.0);
    }

    #[test]
    fn test_record_drops() {
        let mut stats = DropStats::new(0.5);

        // Record 1 drop, 2 non-drops
        stats.record(true);
        stats.record(false);
        stats.record(false);

        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.total_dropped, 1);
        assert_eq!(stats.total_drop_rate(), 1.0 / 3.0);
    }

    #[test]
    fn test_reset() {
        let mut stats = DropStats::new(0.5);

        // Record some data
        stats.record(true);
        stats.record(true);

        // Reset and verify counters are zeroed
        stats.reset();
        assert_eq!(stats.total_packets, 0);
        assert_eq!(stats.total_dropped, 0);
        assert_eq!(stats.total_drop_rate(), 0.0);
    }
}
