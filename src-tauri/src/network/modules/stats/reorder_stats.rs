use crate::network::modules::stats::util::ewma::Ewma;

/// Statistics tracker for packet reordering
///
/// Tracks total packets, reordered packets, and delayed packets along with
/// exponentially weighted moving average of reordering rates.
#[derive(Debug)]
pub struct ReorderStats {
    pub(crate) total_packets: usize,
    pub(crate) reordered_packets: usize,
    pub(crate) delayed_packets: usize,
    ewma: Ewma,
}

impl ReorderStats {
    /// Creates a new ReorderStats with specified alpha for EWMA calculation
    ///
    /// # Arguments
    ///
    /// * `alpha` - Smoothing factor for exponentially weighted moving average
    pub fn new(alpha: f64) -> Self {
        Self {
            total_packets: 0,
            reordered_packets: 0,
            delayed_packets: 0,
            ewma: Ewma::new(alpha),
        }
    }

    /// Records a packet reordering event
    ///
    /// # Arguments
    ///
    /// * `reordered` - Whether the packet was reordered
    pub fn record(&mut self, reordered: bool) {
        self.total_packets += 1;
        if reordered {
            self.reordered_packets += 1;
        }

        let current_reorder_rate = if reordered { 1.0 } else { 0.0 };
        self.ewma.update(current_reorder_rate);
    }

    /// Returns the total reorder rate (ratio of reordered to total packets)
    ///
    /// Returns 0.0 if no packets have been processed.
    #[allow(dead_code)]
    pub fn total_reorder_rate(&self) -> f64 {
        if self.total_packets == 0 {
            0.0
        } else {
            self.reordered_packets as f64 / self.total_packets as f64
        }
    }

    /// Returns the recent reorder rate based on the EWMA
    ///
    /// Returns 0.0 if no data available.
    #[allow(dead_code)]
    pub fn recent_reorder_rate(&self) -> f64 {
        self.ewma.get().unwrap_or(0.0)
    }
}
