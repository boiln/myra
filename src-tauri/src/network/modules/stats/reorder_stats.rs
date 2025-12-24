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
}
