use crate::network::modules::stats::util::ewma::Ewma;

/// Statistics tracker for packet duplication
///
/// Tracks incoming and outgoing packet counts along with
/// exponentially weighted moving average of duplication rates.
#[derive(Debug)]
pub struct DuplicateStats {
    pub(crate) incoming_packet_count: usize,
    pub(crate) outgoing_packet_count: usize,
    ewma: Ewma,
}

impl DuplicateStats {
    /// Creates a new DuplicateStats with specified alpha for EWMA calculation
    ///
    /// # Arguments
    ///
    /// * `alpha` - Smoothing factor for exponentially weighted moving average
    pub fn new(alpha: f64) -> Self {
        Self {
            incoming_packet_count: 0,
            outgoing_packet_count: 0,
            ewma: Ewma::new(alpha),
        }
    }

    /// Records a packet duplication event
    ///
    /// # Arguments
    ///
    /// * `outgoing_count` - Number of packets sent out for this one incoming packet
    pub fn record(&mut self, outgoing_count: usize) {
        self.incoming_packet_count += 1;
        self.outgoing_packet_count += outgoing_count;

        let current_duplication_multiplier = outgoing_count as f64;
        self.ewma.update(current_duplication_multiplier);
    }

    /// Returns the total duplication multiplier (ratio of outgoing to incoming packets)
    ///
    /// Returns 1.0 if no packets have been processed.
    #[allow(dead_code)]
    pub fn total_duplication_multiplier(&self) -> f64 {
        if self.incoming_packet_count == 0 {
            1.0
        } else {
            self.outgoing_packet_count as f64 / self.incoming_packet_count as f64
        }
    }

    /// Returns the recent duplication multiplier based on the EWMA
    ///
    /// Returns 1.0 if no data available.
    #[allow(dead_code)]
    pub fn recent_duplication_multiplier(&self) -> f64 {
        self.ewma.get().unwrap_or(1.0)
    }
}
