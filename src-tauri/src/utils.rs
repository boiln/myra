//! Utility functions for packet processing.
//!
//! This module contains shared utility functions used throughout the application.

use log::info;

/// Logs packet statistics including received count, sent count, and drop percentage.
///
/// This function calculates and logs metrics about packet processing, including
/// how many packets were received, sent, and the percentage that were dropped.
///
/// # Arguments
///
/// * `received` - Number of packets received
/// * `sent` - Number of packets sent
pub fn log_statistics(received: usize, sent: usize) {
    let dropped = received.saturating_sub(sent);
    let dropped_percentage = if received == 0 {
        0.0
    } else {
        (dropped as f64 / received as f64) * 100.0
    };

    info!(
        "Received Packets: {}, Sent Packets: {}, Skipped Packets: {} - {:.2}%",
        received, sent, dropped, dropped_percentage
    );
}

/// Checks if a module effect is still active based on its duration and start time.
///
/// This function determines whether a module effect should still be active
/// based on its configuration duration and when it was started.
///
/// # Arguments
///
/// * `duration_ms` - Duration of the effect in milliseconds (0 = infinite)
/// * `start_time` - When the effect was started
///
/// # Returns
///
/// `true` if the effect is still active, `false` otherwise
pub fn is_effect_active(duration_ms: u64, start_time: std::time::Instant) -> bool {
    if duration_ms == 0 {
        return true;
    }

    let elapsed = start_time.elapsed().as_millis() as u64;
    elapsed < duration_ms
}
