//! Classic mode packet processor.
//!
//! Orchestrates all Classic mode modules in the correct order.
use crate::network::classic::state::ClassicProcessingState;
use crate::network::classic::{bandwidth, drop, latency, reorder, tamper, throttle};
use crate::network::core::PacketData;
use crate::settings::classic::ClassicSettings;

/// Process packets through all enabled Classic mode modules.
///
/// Module processing order:
/// 1. Drop - Remove packets first (no point processing dropped packets)
/// 2. Latency - Hold packets for fixed duration
/// 3. Throttle - Buffer packets for time window
/// 4. Reorder - Swap adjacent packets
/// 5. Tamper - Corrupt packet data
/// 6. Bandwidth - Rate limit output
pub fn process_classic_packets<'a>(
    packets: &mut Vec<PacketData<'a>>,
    settings: &ClassicSettings,
    state: &mut ClassicProcessingState,
) {

    // 1. Drop module
    if let Some(opts) = &settings.drop {
        if opts.enabled {
            drop::process_drop(packets, opts);
        }
    }

    // 2. Latency module
    if let Some(opts) = &settings.latency {
        if opts.enabled {
            latency::process_latency(packets, opts, &mut state.latency);
        }
    }

    // 3. Throttle module
    if let Some(opts) = &settings.throttle {
        if opts.enabled {
            throttle::process_throttle(packets, opts, &mut state.throttle);
        }
    }

    // 4. Reorder module
    if let Some(opts) = &settings.reorder {
        if opts.enabled {
            reorder::process_reorder(packets, opts, &mut state.reorder);
        }
    }

    // 5. Tamper module
    if let Some(opts) = &settings.tamper {
        if opts.enabled {
            tamper::process_tamper(packets, opts, &mut state.tamper);
        }
    }

    // 6. Bandwidth module
    if let Some(opts) = &settings.bandwidth {
        if opts.enabled {
            bandwidth::process_bandwidth(packets, opts, &mut state.bandwidth);
        }
    }

}
