//! Module registry for automatic module discovery and processing.
//!
//! This module provides a registry pattern that eliminates boilerplate when
//! adding new packet manipulation modules. Instead of modifying 10+ files,
//! you only need to:
//!
//! 1. Create your module file with options, state, and implementation
//! 2. Register it in the registry
//!
//! # Example: Adding a new "jitter" module
//!
//! ```rust,ignore
//! // 1. Create settings/jitter.rs with JitterOptions
//! // 2. Create network/modules/jitter.rs with JitterModule
//! // 3. Add to registry in this file:
//!
//! registry.register(ModuleEntry {
//!     name: "jitter",
//!     display_name: "Packet Jitter",
//!     get_options: |s| s.jitter.as_ref(),
//!     process: |packets, settings, state, stats, effect_start, has_packets| {
//!         process_module(&JitterModule, settings.jitter.as_ref(), packets,
//!                        &mut state.jitter, effect_start, stats, has_packets)
//!     },
//! });
//! ```

use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::burst::flush_buffer;
use crate::network::modules::stats::PacketProcessingStatistics;
use crate::network::modules::traits::{ModuleContext, ModuleOptions, PacketModule};
use crate::network::modules::{
    BandwidthModule, BurstModule, DropModule, DuplicateModule, LagModule, ReorderModule,
    TamperModule, ThrottleModule,
};
use crate::network::processing::module_state::ModuleProcessingState;
use crate::settings::Settings;
use crate::utils::is_effect_active;
use log::info;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Entry for a registered module in the registry.
pub struct ModuleEntry {
    /// Unique identifier for this module
    pub name: &'static str,
    /// Human-readable display name
    pub display_name: &'static str,
    /// Order in which this module should be processed (lower = earlier)
    pub order: u32,
    /// Whether this module needs special handling (like burst flush)
    pub needs_special_handling: bool,
}

/// Information about all registered modules.
pub const MODULES: &[ModuleEntry] = &[
    ModuleEntry {
        name: "drop",
        display_name: "Packet Drop",
        order: 10,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "lag",
        display_name: "Packet Lag",
        order: 20,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "throttle",
        display_name: "Throttle",
        order: 30,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "reorder",
        display_name: "Packet Reorder",
        order: 40,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "tamper",
        display_name: "Packet Tamper",
        order: 50,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "duplicate",
        display_name: "Packet Duplicate",
        order: 60,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "bandwidth",
        display_name: "Bandwidth Limit",
        order: 70,
        needs_special_handling: false,
    },
    ModuleEntry {
        name: "burst",
        display_name: "Burst (Lag Switch)",
        order: 80,
        needs_special_handling: true, // Needs buffer flush on disable
    },
];

/// Get all module names as a slice.
pub fn module_names() -> impl Iterator<Item = &'static str> {
    MODULES.iter().map(|m| m.name)
}

/// Get the total number of registered modules.
pub const fn module_count() -> usize {
    MODULES.len()
}

/// Find a module by name.
pub fn find_module(name: &str) -> Option<&'static ModuleEntry> {
    MODULES.iter().find(|m| m.name == name)
}

/// Checks if a specific module is enabled in settings.
pub fn is_module_enabled(settings: &Settings, name: &str) -> bool {
    match name {
        "drop" => settings.drop.as_ref().is_some_and(|o| o.enabled),
        "lag" => settings.lag.as_ref().is_some_and(|o| o.enabled),
        "throttle" => settings.throttle.as_ref().is_some_and(|o| o.enabled),
        "reorder" => settings.reorder.as_ref().is_some_and(|o| o.enabled),
        "tamper" => settings.tamper.as_ref().is_some_and(|o| o.enabled),
        "duplicate" => settings.duplicate.as_ref().is_some_and(|o| o.enabled),
        "bandwidth" => settings.bandwidth.as_ref().is_some_and(|o| o.enabled),
        "burst" => settings.burst.as_ref().is_some_and(|o| o.enabled),
        _ => false,
    }
}

/// Returns true if any module is currently enabled.
pub fn has_any_enabled(settings: &Settings) -> bool {
    MODULES.iter().any(|m| is_module_enabled(settings, m.name))
}

/// Returns a list of currently enabled module names.
pub fn get_enabled_modules(settings: &Settings) -> Vec<&'static str> {
    MODULES
        .iter()
        .filter(|m| is_module_enabled(settings, m.name))
        .map(|m| m.name)
        .collect()
}

/// Generic module processor that handles common logic.
///
/// This function wraps the module-specific processing with:
/// - Enabled check
/// - Duration-based auto-disable
/// - Skip conditions
/// - Effect start time reset
pub fn process_module<M>(
    module: &M,
    options: Option<&M::Options>,
    packets: &mut Vec<PacketData<'_>>,
    state: &mut M::State,
    effect_start: &mut Instant,
    statistics: &Arc<RwLock<PacketProcessingStatistics>>,
    has_packets: bool,
) -> Result<()>
where
    M: PacketModule,
{
    let Some(opts) = options else {
        return Ok(());
    };

    if !opts.is_enabled() {
        return Ok(());
    }

    // Check duration-based disable
    let duration = module.get_duration_ms(opts);
    if duration > 0 && !is_effect_active(duration, *effect_start) {
        return Ok(());
    }

    // Check module-specific skip conditions
    if module.should_skip(opts) {
        return Ok(());
    }

    // Reset effect start time if this is the first packet
    if has_packets && *effect_start == Instant::now() {
        *effect_start = Instant::now();
    }

    let mut ctx = ModuleContext {
        statistics,
        has_packets,
        effect_start,
    };

    module.process(packets, opts, state, &mut ctx)
}

/// Process all registered modules in order.
///
/// This is the main entry point that replaces the manual `process_module` calls
/// in processor.rs. It handles all modules automatically based on the registry.
pub fn process_all_modules(
    settings: &Settings,
    packets: &mut Vec<PacketData<'_>>,
    state: &mut ModuleProcessingState,
    statistics: &Arc<RwLock<PacketProcessingStatistics>>,
) -> Result<()> {
    let has_packets = !packets.is_empty();

    // Process each module in order
    process_module(
        &DropModule,
        settings.drop.as_ref(),
        packets,
        &mut (),
        &mut state.effect_start_times.drop,
        statistics,
        has_packets,
    )?;

    process_module(
        &LagModule,
        settings.lag.as_ref(),
        packets,
        &mut state.lag,
        &mut state.effect_start_times.lag,
        statistics,
        has_packets,
    )?;

    process_module(
        &ThrottleModule,
        settings.throttle.as_ref(),
        packets,
        &mut state.throttle,
        &mut state.effect_start_times.throttle,
        statistics,
        has_packets,
    )?;

    process_module(
        &ReorderModule,
        settings.reorder.as_ref(),
        packets,
        &mut state.reorder,
        &mut state.effect_start_times.reorder,
        statistics,
        has_packets,
    )?;

    process_module(
        &TamperModule,
        settings.tamper.as_ref(),
        packets,
        &mut (),
        &mut state.effect_start_times.tamper,
        statistics,
        has_packets,
    )?;

    process_module(
        &DuplicateModule,
        settings.duplicate.as_ref(),
        packets,
        &mut (),
        &mut state.effect_start_times.duplicate,
        statistics,
        has_packets,
    )?;

    process_module(
        &BandwidthModule,
        settings.bandwidth.as_ref(),
        packets,
        &mut state.bandwidth,
        &mut state.effect_start_times.bandwidth,
        statistics,
        has_packets,
    )?;

    // Special handling for burst module - flush buffer when disabled
    let burst_enabled = settings.burst.as_ref().is_some_and(|b| b.enabled);
    if state.burst_was_enabled && !burst_enabled {
        let buffer_size = state.burst.buffer.len();
        let reverse = settings.burst.as_ref().is_some_and(|b| b.reverse);

        info!(
            "BURST DISABLED: Flushing {} buffered packets (reverse={})",
            buffer_size, reverse
        );

        // SAFETY: We need to transmute the lifetime because the buffer holds
        // PacketData with a different lifetime than the current packets vec.
        // This is safe because we immediately drain and process all packets.
        let buffer: &mut VecDeque<(PacketData<'_>, Instant)> =
            unsafe { std::mem::transmute(&mut state.burst.buffer) };
        flush_buffer(packets, buffer, &mut state.burst.cycle_start, reverse);
    }
    state.burst_was_enabled = burst_enabled;

    process_module(
        &BurstModule,
        settings.burst.as_ref(),
        packets,
        &mut state.burst,
        &mut state.effect_start_times.burst,
        statistics,
        has_packets,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_count() {
        assert_eq!(module_count(), 8);
    }

    #[test]
    fn test_find_module() {
        let drop = find_module("drop");
        assert!(drop.is_some());
        assert_eq!(drop.unwrap().display_name, "Packet Drop");

        let invalid = find_module("nonexistent");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_module_names() {
        let names: Vec<_> = module_names().collect();
        assert!(names.contains(&"drop"));
        assert!(names.contains(&"lag"));
        assert!(names.contains(&"burst"));
    }
}
