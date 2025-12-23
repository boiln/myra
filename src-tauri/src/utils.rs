use log::info;
use std::fmt;

/// Logs packet statistics including received count, sent count, and drop percentage.
/// 
/// This function calculates and logs metrics about packet processing, including
/// how many packets were received, sent, and the percentage that were dropped.
/// 
/// # Arguments
/// 
/// * `received` - Number of packets received
/// * `sent` - Number of packets sent
/// 
/// # Example
/// 
/// ```
/// use myra::utils::log_statistics;
/// 
/// // Log that we received 100 packets and sent 95
/// log_statistics(100, 95);
/// ```
pub fn log_statistics(received: usize, sent: usize) {
    // calculate number of dropped packets safely
    let dropped = received.saturating_sub(sent);
    
    // calculate drop percentage, avoiding division by zero
    let dropped_percentage = if received > 0 {
        (dropped as f64 / received as f64) * 100.0
    } else {
        0.0
    };
    
    info!(
        "Received Packets: {}, Sent Packets: {}, Skipped Packets: {} - {:.2}%",
        received, sent, dropped, dropped_percentage
    );
}

/// Formats a byte count into a human-readable string with appropriate units.
/// 
/// # Arguments
/// 
/// * `bytes` - The number of bytes to format
/// 
/// # Returns
/// 
/// A formatted string with units (B, KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// A generic result type for functions that can return various errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the application.
#[derive(Debug)]
pub enum Error {
    /// I/O errors from the standard library
    Io(std::io::Error),
    /// WinDivert-specific errors
    WinDivert(String),
    /// Configuration errors
    Config(String),
    /// Other general errors
    Other(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::WinDivert(msg) => write!(f, "WinDivert error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

/// Checks if a module effect is still active based on its duration and start time
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
/// * `bool` - True if the effect is still active, false otherwise
pub fn is_effect_active(duration_ms: u64, start_time: std::time::Instant) -> bool {
    if duration_ms == 0 {
        // Duration of 0 means infinite duration
        return true;
    }
    
    // Check if the elapsed time is less than the configured duration
    let elapsed = start_time.elapsed().as_millis() as u64;
    elapsed < duration_ms
}
