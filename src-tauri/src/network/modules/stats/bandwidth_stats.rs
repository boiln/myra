use crate::network::modules::stats::util::ewma::Ewma;
use std::time::{Duration, Instant};

/// Statistics for bandwidth limiting operations
///
/// This struct tracks statistics related to bandwidth throttling, including:
/// - The number of packets currently in storage
/// - The total number of bytes that have been sent
/// - Recent throughput calculated using an exponential weighted moving average (EWMA)
///
/// It provides methods to record traffic and retrieve statistics about recent throughput.
#[derive(Debug)]
pub struct BandwidthStats {
    /// Number of packets currently held in the bandwidth limiter's buffer
    pub(crate) storage_packet_count: usize,

    /// Total number of bytes sent since this stats tracker was created
    pub(crate) total_byte_count: usize,

    /// EWMA calculator for smoothing throughput measurements
    ewma: Ewma,

    /// Bytes sent since the last EWMA update
    recent_byte_sent: usize,

    /// Timer used to determine when to update the EWMA
    recent_timer: Instant,

    /// Interval at which to update the EWMA
    update_interval: Duration,
}

impl BandwidthStats {
    /// Creates a new BandwidthStats instance with the specified alpha value for the EWMA
    ///
    /// The alpha value controls how quickly the EWMA responds to changes in throughput.
    /// Higher values (closer to 1.0) give more weight to recent measurements.
    /// Lower values (closer to 0.0) provide more smoothing over time.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The alpha value for the EWMA calculation (between 0.0 and 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// let stats = BandwidthStats::new(0.5); // Equal weight to recent and historical data
    /// ```
    pub fn new(alpha: f64) -> Self {
        BandwidthStats {
            storage_packet_count: 0,
            total_byte_count: 0,
            ewma: Ewma::new(alpha),
            recent_byte_sent: 0,
            recent_timer: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }

    /// Records bytes sent and updates the throughput statistics
    ///
    /// This method:
    /// 1. Adds the bytes sent to the total count
    /// 2. Accumulates bytes for the current measurement period
    /// 3. Updates the EWMA if the update interval has elapsed
    ///
    /// # Arguments
    ///
    /// * `bytes_sent` - The number of bytes sent in this operation
    pub fn record(&mut self, bytes_sent: usize) {
        self.total_byte_count += bytes_sent;
        self.recent_byte_sent += bytes_sent;
        if self.recent_timer.elapsed() >= self.update_interval {
            self.ewma.update(
                (self.recent_byte_sent as f64 / 1024f64) / self.update_interval.as_secs_f64(),
            );
            self.recent_byte_sent = 0;
            self.recent_timer = Instant::now();
        }
    }

    /// Returns the total number of bytes sent
    ///
    /// # Returns
    ///
    /// The total number of bytes that have passed through the bandwidth limiter
    pub fn total_bytes(&self) -> usize {
        self.total_byte_count
    }

    /// Returns the number of packets currently held in the buffer
    ///
    /// # Returns
    ///
    /// The number of packets being held in the bandwidth limiter's buffer
    pub fn buffered_packets(&self) -> usize {
        self.storage_packet_count
    }

    /// Resets all statistics to zero
    ///
    /// This resets the packet count, byte count, and EWMA calculations.
    pub fn reset(&mut self) {
        self.storage_packet_count = 0;
        self.total_byte_count = 0;
        self.recent_byte_sent = 0;
        self.ewma.reset();
        self.recent_timer = Instant::now();
    }
}
