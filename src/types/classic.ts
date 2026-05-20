/**
 * Classic Mode Types
 *
 * Classic mode provides deterministic, timer-based network manipulation
 * as opposed to Standard mode's probabilistic packet-level approach.
 *
 * Key differences:
 * - Standard: Per-packet probability (e.g., 10% chance each packet is affected)
 * - Classic: Time-window based (e.g., buffer all packets for X ms, then release/drop)
 */
export type ClassicModuleName =
    | "classic_latency"
    | "classic_drop"
    | "classic_throttle"
    | "classic_reorder"
    | "classic_tamper"
    | "classic_bandwidth";

export interface ClassicModuleBase {

    enabled: boolean;
    inbound: boolean;
    outbound: boolean;
    chance: number; // 0-100 (percentage with 0.01% precision internally)

}

/**
 * Latency Module (Classic)
 * Holds packets for a fixed duration before releasing them.
 * Unlike Standard lag which applies per-packet delay, this buffers
 * all matching packets and releases them after the delay expires.
 */
export interface ClassicLatencyOptions extends ClassicModuleBase {

    delay_ms: number; // Fixed delay in milliseconds (0-15000)

}

/**
 * Packet Drop Module (Classic)
 * Probabilistically drops packets immediately (no buffering).
 */
export interface ClassicDropOptions extends ClassicModuleBase {

    // Uses base chance field for drop probability

}

/**
 * Throttle Module (Classic)
 * Buffers packets for a time window, then either releases them
 * as a burst or drops them entirely.
 *
 * Key insight: When drop_on_release=false, this creates a "burst" effect
 * where packets accumulate and release all at once.
 */
export interface ClassicThrottleOptions extends ClassicModuleBase {

    window_ms: number; // Time window for buffering (0-1000ms)
    drop_on_release: boolean; // true = drop buffered packets, false = release as burst
    max_buffer: number; // Max packets to buffer (default: 1000)

}

/**
 * Reorder Module (Classic)
 * Swaps adjacent packets to create out-of-order delivery.
 * Can hold a single packet for up to N cycles waiting for more packets.
 */
export interface ClassicReorderOptions extends ClassicModuleBase {

    max_hold_cycles: number; // How many cycles to hold a lone packet (default: 10)

}

/**
 * Tamper Module (Classic)
 * XORs packet payload data with a rotating pattern.
 * Small packets get full payload tampered, larger packets get ~25% of middle section.
 */
export interface ClassicTamperOptions extends ClassicModuleBase {

    recalc_checksum: boolean; // Whether to recalculate checksums after tampering

}

/**
 * Bandwidth Limit Module (Classic)
 * Rate-limits by bytes per second using a token bucket algorithm.
 * Excess packets are buffered up to a limit, then dropped.
 */
export interface ClassicBandwidthOptions extends ClassicModuleBase {

    limit_kbps: number; // Bandwidth limit in KB/s (0.1-420)
    max_buffer: number; // Max packets to buffer (default: 6000)

}

/**
 * Classic Mode Settings
 * All classic modules with their configurations
 */
export interface ClassicModeSettings {

    latency?: ClassicLatencyOptions;
    drop?: ClassicDropOptions;
    throttle?: ClassicThrottleOptions;
    reorder?: ClassicReorderOptions;
    tamper?: ClassicTamperOptions;
    bandwidth?: ClassicBandwidthOptions;

}

/**
 * Classic Module Info for UI rendering
 */
export interface ClassicModuleInfo {

    name: ClassicModuleName;
    display_name: string;
    description: string;
    enabled: boolean;
    config: ClassicModuleBase & Record<string, unknown>;

}

/**
 * Default configurations for Classic modules
 */
export const CLASSIC_MODULE_DEFAULTS: Record<ClassicModuleName, ClassicModuleInfo> = {

    classic_latency: {
        name: "classic_latency",
        display_name: "Latency",
        description: "Hold packets for a fixed duration before release",
        enabled: false,
        config: {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100,
            delay_ms: 100,
        },
    },
    classic_drop: {
        name: "classic_drop",
        display_name: "Packet Drop",
        description: "Probabilistically discard packets",
        enabled: false,
        config: {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100,
        },
    },
    classic_throttle: {
        name: "classic_throttle",
        display_name: "Throttle",
        description: "Buffer packets then release or drop",
        enabled: false,
        config: {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100,
            window_ms: 30,
            drop_on_release: false,
            max_buffer: 1000,
        },
    },
    classic_reorder: {
        name: "classic_reorder",
        display_name: "Reorder",
        description: "Swap adjacent packets for out-of-order delivery",
        enabled: false,
        config: {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100,
            max_hold_cycles: 10,
        },
    },
    classic_tamper: {
        name: "classic_tamper",
        display_name: "Tamper",
        description: "Corrupt packet payload data",
        enabled: false,
        config: {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 10,
            recalc_checksum: true,
        },
    },
    classic_bandwidth: {
        name: "classic_bandwidth",
        display_name: "Rate Limit",
        description: "Limit throughput by bytes per second",
        enabled: false,
        config: {
            enabled: false,
            inbound: true,
            outbound: true,
            chance: 100,
            limit_kbps: 115,
            max_buffer: 6000,
        },
    },

};

/**
 * Module buffer limits from reverse engineering
 */
export const CLASSIC_BUFFER_LIMITS = {

    latency: 15000, // Emergency release on overflow
    bandwidth: 6000, // DROP on overflow
    throttle: 1000, // Release or drop (configurable)
    reorder: 1, // Single packet hold

} as const;

/**
 * Backend settings format for Classic mode.
 * This matches the Rust ClassicSettings struct.
 */
export interface ClassicBackendSettings {

    latency?: {
        enabled: boolean;
        inbound: boolean;
        outbound: boolean;
        chance: number;
        delay_ms: number;
    };
    drop?: {
        enabled: boolean;
        inbound: boolean;
        outbound: boolean;
        chance: number;
    };
    throttle?: {
        enabled: boolean;
        inbound: boolean;
        outbound: boolean;
        chance: number;
        window_ms: number;
        drop_on_release: boolean;
        max_buffer: number;
    };
    reorder?: {
        enabled: boolean;
        inbound: boolean;
        outbound: boolean;
        chance: number;
        max_hold_cycles: number;
    };
    tamper?: {
        enabled: boolean;
        inbound: boolean;
        outbound: boolean;
        chance: number;
        recalc_checksum: boolean;
    };
    bandwidth?: {
        enabled: boolean;
        inbound: boolean;
        outbound: boolean;
        chance: number;
        limit_kbps: number;
        max_buffer: number;
    };

}

/**
 * Convert frontend module array to backend settings format.
 */
export function modulesToBackendSettings(modules: ClassicModuleInfo[]): ClassicBackendSettings {

    const settings: ClassicBackendSettings = {};

    for (const module of modules) {
        const config = module.config;

        switch (module.name) {
            case "classic_latency":
                settings.latency = {
                    enabled: module.enabled,
                    inbound: config.inbound,
                    outbound: config.outbound,
                    chance: config.chance,
                    delay_ms: (config.delay_ms as number) ?? 100,
                };
                break;
            case "classic_drop":
                settings.drop = {
                    enabled: module.enabled,
                    inbound: config.inbound,
                    outbound: config.outbound,
                    chance: config.chance,
                };
                break;
            case "classic_throttle":
                settings.throttle = {
                    enabled: module.enabled,
                    inbound: config.inbound,
                    outbound: config.outbound,
                    chance: config.chance,
                    window_ms: (config.window_ms as number) ?? 30,
                    drop_on_release: (config.drop_on_release as boolean) ?? false,
                    max_buffer: (config.max_buffer as number) ?? 1000,
                };
                break;
            case "classic_reorder":
                settings.reorder = {
                    enabled: module.enabled,
                    inbound: config.inbound,
                    outbound: config.outbound,
                    chance: config.chance,
                    max_hold_cycles: (config.max_hold_cycles as number) ?? 10,
                };
                break;
            case "classic_tamper":
                settings.tamper = {
                    enabled: module.enabled,
                    inbound: config.inbound,
                    outbound: config.outbound,
                    chance: config.chance,
                    recalc_checksum: (config.recalc_checksum as boolean) ?? true,
                };
                break;
            case "classic_bandwidth":
                settings.bandwidth = {
                    enabled: module.enabled,
                    inbound: config.inbound,
                    outbound: config.outbound,
                    chance: config.chance,
                    limit_kbps: (config.limit_kbps as number) ?? 115,
                    max_buffer: (config.max_buffer as number) ?? 6000,
                };
                break;
        }
    }

    return settings;

}

/**
 * Convert backend settings to frontend module array.
 */
export function backendSettingsToModules(settings: ClassicBackendSettings): ClassicModuleInfo[] {

    const modules: ClassicModuleInfo[] = [];
    const defaults = CLASSIC_MODULE_DEFAULTS;

    // Latency
    const latency = settings.latency;

    modules.push({
        ...defaults.classic_latency,
        enabled: latency?.enabled ?? false,
        config: {
            enabled: latency?.enabled ?? false,
            inbound: latency?.inbound ?? true,
            outbound: latency?.outbound ?? true,
            chance: latency?.chance ?? 100,
            delay_ms: latency?.delay_ms ?? 100,
        },
    });

    // Drop
    const drop = settings.drop;

    modules.push({
        ...defaults.classic_drop,
        enabled: drop?.enabled ?? false,
        config: {
            enabled: drop?.enabled ?? false,
            inbound: drop?.inbound ?? true,
            outbound: drop?.outbound ?? true,
            chance: drop?.chance ?? 10,
        },
    });

    // Throttle
    const throttle = settings.throttle;

    modules.push({
        ...defaults.classic_throttle,
        enabled: throttle?.enabled ?? false,
        config: {
            enabled: throttle?.enabled ?? false,
            inbound: throttle?.inbound ?? true,
            outbound: throttle?.outbound ?? true,
            chance: throttle?.chance ?? 10,
            window_ms: throttle?.window_ms ?? 30,
            drop_on_release: throttle?.drop_on_release ?? false,
            max_buffer: throttle?.max_buffer ?? 1000,
        },
    });

    // Reorder
    const reorder = settings.reorder;

    modules.push({
        ...defaults.classic_reorder,
        enabled: reorder?.enabled ?? false,
        config: {
            enabled: reorder?.enabled ?? false,
            inbound: reorder?.inbound ?? true,
            outbound: reorder?.outbound ?? true,
            chance: reorder?.chance ?? 10,
            max_hold_cycles: reorder?.max_hold_cycles ?? 10,
        },
    });

    // Tamper
    const tamper = settings.tamper;

    modules.push({
        ...defaults.classic_tamper,
        enabled: tamper?.enabled ?? false,
        config: {
            enabled: tamper?.enabled ?? false,
            inbound: tamper?.inbound ?? true,
            outbound: tamper?.outbound ?? true,
            chance: tamper?.chance ?? 10,
            recalc_checksum: tamper?.recalc_checksum ?? true,
        },
    });

    // Bandwidth
    const bandwidth = settings.bandwidth;

    modules.push({
        ...defaults.classic_bandwidth,
        enabled: bandwidth?.enabled ?? false,
        config: {
            enabled: bandwidth?.enabled ?? false,
            inbound: bandwidth?.inbound ?? true,
            outbound: bandwidth?.outbound ?? true,
            chance: bandwidth?.chance ?? 100,
            limit_kbps: bandwidth?.limit_kbps ?? 115,
            max_buffer: bandwidth?.max_buffer ?? 6000,
        },
    });

    return modules;

}
