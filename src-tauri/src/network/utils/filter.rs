use regex::Regex;
use thiserror::Error;
use windivert::layer::NetworkLayer;
use windivert::prelude::WinDivertFlags;
use windivert::{CloseAction, WinDivert};

/// Errors that can occur during filter validation
///
/// These errors are returned when a filter string has syntax issues
/// or contains invalid values such as out-of-range port numbers.
#[derive(Debug, Error, Clone)]
pub enum FilterError {
    /// Error for invalid filter syntax
    #[error("Invalid filter syntax: {0}")]
    InvalidSyntax(String),

    /// Error for invalid port numbers in filters
    #[error("Invalid port number detected in filter: {0}")]
    InvalidPort(String),
}

/// Validates a WinDivert filter and adds documentation links on error
///
/// Checks if the filter syntax is valid and enhances error messages
/// with documentation links to help users correct their filters.
///
/// # Arguments
///
/// * `filter` - The filter string to validate
///
/// # Returns
///
/// * `Ok(String)` - The validated filter string
/// * `Err(FilterError)` - Error with detailed message including documentation link
#[allow(dead_code)]
pub fn validate_filter_with_docs(filter: &str) -> Result<String, FilterError> {
    match validate_filter(filter) {
        Err(FilterError::InvalidSyntax(msg)) => {
            let detailed_msg = format!(
                "{}\n\nFor more details about the filter syntax, see the filter language documentation: https://reqrypt.org/windivert-doc.html#filter_language",
                msg
            );
            Err(FilterError::InvalidSyntax(detailed_msg))
        }
        other => other,
    }
}

/// Validates a WinDivert filter string
///
/// Attempts to create a WinDivert handle with the given filter to check
/// if the syntax is valid. Also validates port numbers to ensure they're
/// within the valid range (0-65535).
///
/// # Arguments
///
/// * `filter` - The filter string to validate
///
/// # Returns
///
/// * `Ok(String)` - The validated filter string
/// * `Err(FilterError)` - Detailed error message if validation fails
#[allow(dead_code)]
pub fn validate_filter(filter: &str) -> Result<String, FilterError> {
    // Attempt to open a handle to validate the filter string syntax
    let mut win_divert =
        WinDivert::<NetworkLayer>::network(filter, 0, WinDivertFlags::new().set_sniff())
            .map_err(|e| FilterError::InvalidSyntax(e.to_string()))?;

    win_divert
        .close(CloseAction::Nothing)
        .map_err(|_| FilterError::InvalidSyntax("Failed to close handle.".into()))?;

    // Additional check: ensure any provided port numbers are valid
    let port_pattern = Regex::new(r"(tcp|udp)\.(SrcPort|DstPort)\s*==\s*(\d+)(?:$|\s)").unwrap();
    for cap in port_pattern.captures_iter(filter) {
        if let Some(port_str) = cap.get(3) {
            let port_str = port_str.as_str();
            port_str.parse::<u16>().map_err(|_| {
                FilterError::InvalidPort(format!(
                    "Port number {} is out of range (0-65535)",
                    port_str
                ))
            })?;
        }
    }

    Ok(filter.to_string())
}
