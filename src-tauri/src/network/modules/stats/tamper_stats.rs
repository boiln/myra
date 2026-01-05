use std::ops::Sub;
use std::time::{Duration, Instant};

/// Statistics for network packet tampering operations
///
/// This struct tracks information about tampered packets, including:
/// - The payload data of the most recently tampered packet
/// - Which bytes in the payload were modified
/// - Whether checksums are still valid after tampering
///
/// It also includes logic to control how frequently statistics are updated
/// to avoid excessive resource usage when monitoring high-traffic networks.
#[derive(Debug)]
pub struct TamperStats {
    /// Raw payload data from the most recently tampered packet
    pub(crate) data: Vec<u8>,

    /// Boolean flags indicating which bytes in the data were tampered with (true = tampered)
    pub(crate) tamper_flags: Vec<bool>,

    /// Indicates whether packet checksums are still valid after tampering
    pub(crate) checksum_valid: bool,

    /// When statistics were last updated
    pub last_update: Instant,

    /// How often statistics should be updated
    pub update_interval: Duration,
}

impl TamperStats {
    /// Creates a new `TamperStats` instance with the specified refresh interval
    ///
    /// # Arguments
    ///
    /// * `refresh_interval` - How frequently the statistics should be updated
    ///
    /// # Returns
    ///
    /// A new `TamperStats` instance
    ///
    /// # Example
    ///
    /// ```
    /// let stats = TamperStats::new(Duration::from_millis(100));
    /// ```
    pub fn new(refresh_interval: Duration) -> Self {
        Self {
            data: vec![],
            tamper_flags: vec![],
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

    /// Returns the raw payload data from the most recently tampered packet
    ///
    /// # Returns
    ///
    /// A slice of the payload data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the tamper flags indicating which bytes were modified
    ///
    /// # Returns
    ///
    /// A slice of boolean flags where `true` indicates the byte was tampered with
    pub fn tamper_flags(&self) -> &[bool] {
        &self.tamper_flags
    }

    /// Returns whether packet checksums are still valid after tampering
    ///
    /// # Returns
    ///
    /// `true` if the checksums are valid, `false` otherwise
    pub fn checksum_valid(&self) -> bool {
        self.checksum_valid
    }

    /// Resets all statistics
    ///
    /// Clears the data and tamper flags and resets the checksum status.
    pub fn reset(&mut self) {
        self.data.clear();
        self.tamper_flags.clear();
        self.checksum_valid = true;
        self.last_update = Instant::now();
    }

    /// Returns the number of tampered bytes in the most recent packet
    ///
    /// # Returns
    ///
    /// The count of bytes that were tampered with
    pub fn tampered_byte_count(&self) -> usize {
        self.tamper_flags
            .iter()
            .filter(|&&tampered| tampered)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let stats = TamperStats::new(Duration::from_millis(100));
        assert!(stats.data.is_empty());
        assert!(stats.tamper_flags.is_empty());
        assert!(stats.checksum_valid);
    }

    #[test]
    fn test_should_update() {
        // Create with a refresh interval that's already elapsed
        let mut stats = TamperStats::new(Duration::from_millis(0));
        assert!(stats.should_update());

        // Update and check again immediately
        stats.updated();
        stats.update_interval = Duration::from_secs(1);
        assert!(!stats.should_update());
    }

    #[test]
    fn test_tampered_byte_count() {
        let mut stats = TamperStats::new(Duration::from_millis(100));
        stats.tamper_flags = vec![true, false, true, false, true];
        assert_eq!(stats.tampered_byte_count(), 3);

        stats.tamper_flags = vec![false, false, false];
        assert_eq!(stats.tampered_byte_count(), 0);
    }
}
