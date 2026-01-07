use std::ops::Sub;
use std::time::{Duration, Instant};

/// Statistics for network packet corruptioning operations
///
/// This struct tracks information about corruptioned packets, including:
/// - The payload data of the most recently corruptioned packet
/// - Which bytes in the payload were modified
/// - Whether checksums are still valid after corruptioning
///
/// It also includes logic to control how frequently statistics are updated
/// to avoid excessive resource usage when monitoring high-traffic networks.
#[derive(Debug)]
pub struct CorruptionStats {
    /// Raw payload data from the most recently corruptioned packet
    pub(crate) data: Vec<u8>,

    /// Boolean flags indicating which bytes in the data were corruptioned with (true = corruptioned)
    pub(crate) corruption_flags: Vec<bool>,

    /// Indicates whether packet checksums are still valid after corruptioning
    pub(crate) checksum_valid: bool,

    /// When statistics were last updated
    pub last_update: Instant,

    /// How often statistics should be updated
    pub update_interval: Duration,
}

impl CorruptionStats {
    /// Creates a new `CorruptionStats` instance with the specified refresh interval
    ///
    /// # Arguments
    ///
    /// * `refresh_interval` - How frequently the statistics should be updated
    ///
    /// # Returns
    ///
    /// A new `CorruptionStats` instance
    ///
    /// # Example
    ///
    /// ```
    /// let stats = CorruptionStats::new(Duration::from_millis(100));
    /// ```
    pub fn new(refresh_interval: Duration) -> Self {
        Self {
            data: vec![],
            corruption_flags: vec![],
            checksum_valid: true,
            last_update: Instant::now().sub(refresh_interval),
            update_interval: refresh_interval,
        }
    }

    /// Determines if it's time to update the statistics
    ///
    /// This method helps control the frequency of statistics updates
    /// to avoid excessive processing on high-traffic networks.
    ///
    /// # Returns
    ///
    /// `true` if enough time has passed since the last update, `false` otherwise
    pub fn should_update(&mut self) -> bool {
        self.last_update.elapsed() >= self.update_interval
    }

    /// Records that statistics have been updated
    ///
    /// Call this method after updating the statistics to reset the update timer.
    pub fn updated(&mut self) {
        self.last_update = Instant::now();
    }

    /// Returns the raw payload data from the most recently corruptioned packet
    ///
    /// # Returns
    ///
    /// A slice of the payload data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the corruption flags indicating which bytes were modified
    ///
    /// # Returns
    ///
    /// A slice of boolean flags where `true` indicates the byte was corruptioned with
    pub fn corruption_flags(&self) -> &[bool] {
        &self.corruption_flags
    }

    /// Returns whether packet checksums are still valid after corruptioning
    ///
    /// # Returns
    ///
    /// `true` if the checksums are valid, `false` otherwise
    pub fn checksum_valid(&self) -> bool {
        self.checksum_valid
    }

    /// Resets all statistics
    ///
    /// Clears the data and corruption flags and resets the checksum status.
    pub fn reset(&mut self) {
        self.data.clear();
        self.corruption_flags.clear();
        self.checksum_valid = true;
        self.last_update = Instant::now();
    }

    /// Returns the number of corruptioned bytes in the most recent packet
    ///
    /// # Returns
    ///
    /// The count of bytes that were corruptioned with
    pub fn corruptioned_byte_count(&self) -> usize {
        self.corruption_flags
            .iter()
            .filter(|&&corruptioned| corruptioned)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let stats = CorruptionStats::new(Duration::from_millis(100));
        assert!(stats.data.is_empty());
        assert!(stats.corruption_flags.is_empty());
        assert!(stats.checksum_valid);
    }

    #[test]
    fn test_should_update() {
        // Create with a refresh interval that's already elapsed
        let mut stats = CorruptionStats::new(Duration::from_millis(0));
        assert!(stats.should_update());

        // Update and check again immediately
        stats.updated();
        stats.update_interval = Duration::from_secs(1);
        assert!(!stats.should_update());
    }

    #[test]
    fn test_corruptioned_byte_count() {
        let mut stats = CorruptionStats::new(Duration::from_millis(100));
        stats.corruption_flags = vec![true, false, true, false, true];
        assert_eq!(stats.corruptioned_byte_count(), 3);

        stats.corruption_flags = vec![false, false, false];
        assert_eq!(stats.corruptioned_byte_count(), 0);
    }
}
