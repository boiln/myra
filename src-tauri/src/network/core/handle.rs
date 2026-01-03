//! WinDivert handle management.
//!
//! This module provides centralized management of WinDivert handles,
//! including creation, configuration, and proper cleanup.

use log::{debug, error, info, warn};
use windivert::error::WinDivertError;
use windivert::layer::NetworkLayer;
use windivert::{CloseAction, WinDivert};
use windivert_sys::WinDivertFlags;

#[cfg(windows)]
extern "system" {
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
}

/// Timer resolution tracker for high-precision timing (like Clumsy 0.6)
static TIMER_RESOLUTION_SET: std::sync::atomic::AtomicBool = 
    std::sync::atomic::AtomicBool::new(false);

/// Set Windows timer resolution to 4ms for high-precision timing
/// This is what Clumsy 0.6 (Wan-Destroyer) does to bypass lag detection
#[cfg(windows)]
pub fn set_high_precision_timer() {
    use std::sync::atomic::Ordering;
    if !TIMER_RESOLUTION_SET.load(Ordering::SeqCst) {
        unsafe {
            let result = timeBeginPeriod(4);
            if result == 0 {
                info!("Set Windows timer resolution to 4ms for high-precision timing");
                TIMER_RESOLUTION_SET.store(true, Ordering::SeqCst);
            } else {
                warn!("Failed to set high-precision timer: {}", result);
            }
        }
    }
}

/// Restore Windows timer resolution
#[cfg(windows)]
pub fn restore_timer_resolution() {
    use std::sync::atomic::Ordering;
    if TIMER_RESOLUTION_SET.load(Ordering::SeqCst) {
        unsafe {
            timeEndPeriod(4);
            TIMER_RESOLUTION_SET.store(false, Ordering::SeqCst);
            info!("Restored Windows timer resolution");
        }
    }
}

#[cfg(not(windows))]
pub fn set_high_precision_timer() {}

#[cfg(not(windows))]
pub fn restore_timer_resolution() {}

/// Default priority for packet interception.
pub const DEFAULT_PRIORITY: i16 = 1;

/// Port used by Tauri for local communication.
const TAURI_PORT: u16 = 1420;

/// Configuration for creating a WinDivert handle.
#[derive(Debug, Clone)]
pub struct HandleConfig {
    /// Filter expression for packet matching
    pub filter: String,
    /// Priority for the handle (higher = earlier interception)
    pub priority: i16,
    /// Whether to only receive packets (not send)
    pub recv_only: bool,
    /// Whether to exclude Tauri port from capture
    pub exclude_tauri_port: bool,
}

impl Default for HandleConfig {
    fn default() -> Self {
        Self {
            filter: "true".to_string(),
            priority: DEFAULT_PRIORITY,
            recv_only: true,
            exclude_tauri_port: true,
        }
    }
}

impl HandleConfig {
    /// Creates a new HandleConfig with the given filter.
    pub fn with_filter(filter: impl Into<String>) -> Self {
        Self {
            filter: filter.into(),
            ..Default::default()
        }
    }

    /// Sets the priority for the handle.
    pub fn priority(mut self, priority: i16) -> Self {
        self.priority = priority;
        self
    }

    /// Sets whether the handle should be receive-only.
    pub fn recv_only(mut self, recv_only: bool) -> Self {
        self.recv_only = recv_only;
        self
    }

    /// Sets whether to exclude Tauri port from capture.
    pub fn exclude_tauri_port(mut self, exclude: bool) -> Self {
        self.exclude_tauri_port = exclude;
        self
    }

    /// Builds the final filter string with any exclusions applied.
    fn build_filter(&self) -> String {
        if !self.exclude_tauri_port {
            return self.filter.clone();
        }

        // Use localPort/remotePort which work for both TCP and UDP
        let exclusion = format!("localPort != {0} and remotePort != {0}", TAURI_PORT);

        if self.filter.is_empty() || self.filter == "true" {
            return exclusion;
        }

        format!("({}) and {}", self.filter, exclusion)
    }
}

/// Manages WinDivert handle lifecycle.
///
/// This struct provides a safe wrapper around WinDivert handles,
/// ensuring proper initialization and cleanup.
pub struct HandleManager {
    handle: Option<WinDivert<NetworkLayer>>,
    current_config: Option<HandleConfig>,
}

impl HandleManager {
    /// Creates a new HandleManager without an active handle.
    pub fn new() -> Self {
        Self {
            handle: None,
            current_config: None,
        }
    }

    /// Returns whether a handle is currently active.
    pub fn is_active(&self) -> bool {
        self.handle.is_some()
    }

    /// Returns the current configuration, if any.
    pub fn config(&self) -> Option<&HandleConfig> {
        self.current_config.as_ref()
    }

    /// Opens a new WinDivert handle with the given configuration.
    ///
    /// If a handle is already open, it will be closed first.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the new handle
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the handle was created successfully
    /// * `Err(WinDivertError)` - If handle creation failed
    pub fn open(&mut self, config: HandleConfig) -> Result<(), WinDivertError> {
        // Close existing handle if present
        if self.handle.is_some() {
            self.close()?;
        }

        // Flush WFP cache before creating new handle
        flush_wfp_cache();

        let filter = config.build_filter();
        info!("Opening WinDivert handle with filter: {}", filter);

        let flags = if config.recv_only {
            WinDivertFlags::set_recv_only(WinDivertFlags::new())
        } else {
            WinDivertFlags::new()
        };

        match WinDivert::<NetworkLayer>::network(&filter, config.priority, flags) {
            Ok(mut handle) => {
                debug!("WinDivert handle opened successfully");
                
                // Set higher queue params like Clumsy 0.6 (Wan-Destroyer)
                // Queue length = 2048 packets, Queue time = 1024ms
                use windivert_sys::WinDivertParam;
                if let Err(e) = handle.set_param(WinDivertParam::QueueLength, 2048) {
                    warn!("Failed to set WinDivert queue length: {}", e);
                } else {
                    info!("Set WinDivert queue length to 2048 packets");
                }
                if let Err(e) = handle.set_param(WinDivertParam::QueueTime, 1024) {
                    warn!("Failed to set WinDivert queue time: {}", e);
                } else {
                    info!("Set WinDivert queue time to 1024ms");
                }
                
                self.handle = Some(handle);
                self.current_config = Some(config);
                Ok(())
            }
            Err(e) => {
                error!("Failed to open WinDivert handle: {}", e);
                // Try one more cache flush on failure
                flush_wfp_cache();
                Err(e)
            }
        }
    }

    /// Updates the handle with a new filter, only reopening if the filter changed.
    ///
    /// # Arguments
    ///
    /// * `filter` - The new filter string
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the handle was updated
    /// * `Ok(false)` - If no update was needed (filter unchanged)
    /// * `Err(WinDivertError)` - If handle update failed
    pub fn update_filter(&mut self, filter: &str) -> Result<bool, WinDivertError> {
        let needs_update = match &self.current_config {
            Some(config) => config.filter != filter,
            None => true,
        };

        if needs_update {
            let config = self
                .current_config
                .clone()
                .map(|mut c| {
                    c.filter = filter.to_string();
                    c
                })
                .unwrap_or_else(|| HandleConfig::with_filter(filter));

            self.open(config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Closes the current handle if one is open.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the handle was closed successfully or no handle was open
    /// * `Err(WinDivertError)` - If closing the handle failed
    pub fn close(&mut self) -> Result<(), WinDivertError> {
        if let Some(mut handle) = self.handle.take() {
            debug!("Closing WinDivert handle");
            handle.close(CloseAction::Nothing)?;
            self.current_config = None;
            flush_wfp_cache();
        }
        Ok(())
    }

    /// Returns a reference to the underlying WinDivert handle.
    ///
    /// # Returns
    ///
    /// * `Some(&WinDivert)` - If a handle is open
    /// * `None` - If no handle is open
    pub fn handle(&self) -> Option<&WinDivert<NetworkLayer>> {
        self.handle.as_ref()
    }

    /// Returns a mutable reference to the underlying WinDivert handle.
    ///
    /// # Returns
    ///
    /// * `Some(&mut WinDivert)` - If a handle is open
    /// * `None` - If no handle is open
    pub fn handle_mut(&mut self) -> Option<&mut WinDivert<NetworkLayer>> {
        self.handle.as_mut()
    }
}

impl Default for HandleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HandleManager {
    fn drop(&mut self) {
        if let Err(e) = self.close() {
            error!("Error closing WinDivert handle on drop: {}", e);
        }
    }
}

/// Flushes the Windows Filtering Platform (WFP) cache.
///
/// This is a workaround for WinDivert caching issues. It opens and
/// immediately closes a handle with a no-match filter to clear stale state.
pub fn flush_wfp_cache() {
    // Try multiple priorities to ensure thorough cache flush
    for priority in [0, 1000, -1000] {
        if let Ok(mut handle) =
            WinDivert::<NetworkLayer>::network("false", priority, WinDivertFlags::new())
        {
            let _ = handle.close(CloseAction::Nothing);
            debug!("Flushed WFP cache with priority {}", priority);
        }
    }
}

/// Creates a WinDivert filter that excludes Tauri app ports.
///
/// Takes a user-provided filter and adds conditions to exclude traffic
/// on ports used by the Tauri app to prevent disrupting the app's functionality.
///
/// # Arguments
///
/// * `user_filter` - Optional user-provided filter string
///
/// # Returns
///
/// The complete filter string with Tauri port exclusions, or None if no filter provided
pub fn construct_filter_with_exclusions(user_filter: &Option<String>) -> Option<String> {
    user_filter.as_ref()?;

    // Use localPort/remotePort which work for both TCP and UDP
    let tauri_exclusion = format!("localPort != {0} and remotePort != {0}", TAURI_PORT);

    Some(match user_filter {
        Some(filter) if !filter.is_empty() => {
            // Fix common mistakes: "outbound and inbound" is impossible (packet can't be both)
            // But "(outbound and X) or (inbound and Y)" is valid
            let filter_lower = filter.to_lowercase();
            let corrected_filter = if filter_lower.contains("outbound and inbound")
                || filter_lower.contains("inbound and outbound")
            {
                log::warn!("Filter '{}' is invalid: a packet cannot be both outbound AND inbound. Using 'true' to capture all traffic.", filter);
                "true".to_string()
            } else {
                filter.clone()
            };
            format!("({}) and {}", corrected_filter, tauri_exclusion)
        }
        _ => tauri_exclusion,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_config_default() {
        let config = HandleConfig::default();
        assert_eq!(config.filter, "true");
        assert_eq!(config.priority, DEFAULT_PRIORITY);
        assert!(config.recv_only);
        assert!(config.exclude_tauri_port);
    }

    #[test]
    fn test_handle_config_builder() {
        let config = HandleConfig::with_filter("tcp")
            .priority(100)
            .recv_only(false)
            .exclude_tauri_port(false);

        assert_eq!(config.filter, "tcp");
        assert_eq!(config.priority, 100);
        assert!(!config.recv_only);
        assert!(!config.exclude_tauri_port);
    }

    #[test]
    fn test_build_filter_with_exclusions() {
        let config = HandleConfig::with_filter("tcp.DstPort == 80");
        let filter = config.build_filter();
        assert!(filter.contains("tcp.DstPort == 80"));
        assert!(filter.contains("1420"));
    }

    #[test]
    fn test_build_filter_without_exclusions() {
        let config = HandleConfig::with_filter("tcp").exclude_tauri_port(false);
        let filter = config.build_filter();
        assert_eq!(filter, "tcp");
    }

    #[test]
    fn test_construct_filter_with_exclusions_none() {
        assert!(construct_filter_with_exclusions(&None).is_none());
    }

    #[test]
    fn test_construct_filter_with_exclusions_some() {
        let filter = construct_filter_with_exclusions(&Some("tcp".to_string()));
        assert!(filter.is_some());
        assert!(filter.unwrap().contains("tcp"));
    }
}
