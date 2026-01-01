//! Trait definitions for packet manipulation modules.
//!
//! This module provides a unified interface for all packet manipulation
//! modules, enabling consistent behavior and easier extensibility.

use crate::error::{MyraError, Result};
use crate::network::core::PacketData;
use crate::network::modules::stats::PacketProcessingStatistics;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Trait for module options that can be enabled/disabled.
pub trait ModuleOptions {
    /// Returns whether this module is enabled.
    fn is_enabled(&self) -> bool;
}

/// Context passed to packet modules during processing.
///
/// Contains shared state and timing information needed by modules
/// to determine if effects should be applied.
pub struct ModuleContext<'a, 'b> {
    /// Statistics tracker for all modules
    pub statistics: &'a Arc<RwLock<PacketProcessingStatistics>>,
    /// Whether there are packets to process
    pub has_packets: bool,
    /// Reference to effect start time for duration tracking
    pub effect_start: &'b mut Instant,
}

impl<'a, 'b> ModuleContext<'a, 'b> {
    /// Acquires a write lock on the statistics, returning a Result instead of panicking.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module requesting the lock (for error messages)
    ///
    /// # Returns
    ///
    /// A write guard to the statistics or a MyraError if the lock is poisoned.
    pub fn write_stats(
        &self,
        module_name: &str,
    ) -> Result<std::sync::RwLockWriteGuard<'_, PacketProcessingStatistics>> {
        self.statistics
            .write()
            .map_err(|_| MyraError::stats_lock(module_name))
    }
}

/// Trait for packet manipulation modules.
///
/// All packet manipulation modules (drop, delay, throttle, etc.) implement
/// this trait to provide a consistent interface for packet processing.
///
/// # Example
///
/// ```rust
/// struct MyModule;
///
/// impl PacketModule for MyModule {
///     type Options = MyOptions;
///     type State = MyState;
///
///     fn name(&self) -> &'static str {
///         "my_module"
///     }
///
///     fn process<'a>(
///         &self,
///         packets: &mut Vec<PacketData<'a>>,
///         options: &Self::Options,
///         state: &mut Self::State,
///         ctx: &mut ModuleContext,
///     ) -> Result<()> {
///         // Implementation
///         Ok(())
///     }
/// }
/// ```
pub trait PacketModule {
    /// Configuration options for this module - must implement ModuleOptions
    type Options: ModuleOptions;

    /// Persistent state maintained between processing calls
    type State;

    /// Returns the unique name identifier for this module
    fn name(&self) -> &'static str;

    /// Returns the human-readable display name for this module
    fn display_name(&self) -> &'static str {
        self.name()
    }

    /// Process packets according to module-specific logic.
    ///
    /// # Arguments
    ///
    /// * `packets` - The packets to process (may be modified in place)
    /// * `options` - Module configuration options
    /// * `state` - Mutable module state persisted between calls
    /// * `ctx` - Processing context with shared resources
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `MyraError` if processing fails.
    fn process<'a>(
        &self,
        packets: &mut Vec<PacketData<'a>>,
        options: &Self::Options,
        state: &mut Self::State,
        ctx: &mut ModuleContext,
    ) -> Result<()>;

    /// Returns the duration setting from options, if applicable.
    /// Returns 0 for infinite duration.
    fn get_duration_ms(&self, options: &Self::Options) -> u64;

    /// Check if the module should skip processing based on options.
    /// Override this for modules with skip conditions (e.g., bandwidth=0).
    fn should_skip(&self, _options: &Self::Options) -> bool {
        false
    }
}
