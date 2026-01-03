export interface CaptureStatus {
    active: boolean;
    filter: string;
}

export interface ModuleConfig {
    inbound: boolean;
    outbound: boolean;
    chance: number;
    enabled: boolean;
    duration_ms: number;
    throttle_ms?: number;
    limit_kbps?: number;
    count?: number;
    buffer_ms?: number;
    keepalive_ms?: number;
    release_delay_us?: number;
    drop?: boolean;
    max_buffer?: number;
    lag_bypass?: boolean;
    freeze_mode?: boolean;
    use_wfp?: boolean;
    passthrough_threshold?: number;
    reverse?: boolean;
}

export interface ModuleInfo {
    name: string;
    display_name: string;
    enabled: boolean;
    config: ModuleConfig;
    params?: ModuleParams;
}

export interface ModuleParams {
    lag_time?: number;
}

export interface ManipulationStatus {
    active: boolean;
    filter: string;
    modules: ModuleInfo[];
}

export interface Preset {
    name: string;
    description?: string;
    settings: PacketManipulationSettings;
    filter: string;
    createdAt?: string;
}

export interface Config {
    default_filter: string;
    module_configs: Record<string, ModuleConfig>;
}

export interface PacketManipulationSettings {
    drop?: DropOptions;
    lag?: LagOptions;
    throttle?: ThrottleOptions;
    reorder?: ReorderOptions;
    tamper?: TamperOptions;
    duplicate?: DuplicateOptions;
    bandwidth?: BandwidthOptions;
    burst?: BurstOptions;
    burst_release_delay_us?: number;
    lag_bypass?: boolean;  // MGO2 bypass - swap IPs on send failure
}

export interface DropOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    duration_ms: number;
}

export interface LagOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    lag_ms: number;      // The actual lag time in ms
    duration_ms: number;   // Effect duration (0 = infinite)
}

export interface ThrottleOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    duration_ms: number;
    throttle_ms?: number;
    drop?: boolean;        // If true, drop buffered packets; if false, release them
    max_buffer?: number;   // Max packets to buffer (default 2000)
    freeze_mode?: boolean; // If true, disable cooldown for death loop effect (may DC faster)
}

export interface ReorderOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    duration_ms: number;
    max_delay?: number;
    reverse?: boolean;
}

export interface TamperOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    duration_ms: number;
}

export interface DuplicateOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    count: number;
    duration_ms: number;
}

export interface BandwidthOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    limit: number;          // Bandwidth limit in KB/s (matches Rust 'limit' field)
    duration_ms: number;
    /** Passthrough packets smaller than this size (bytes) to keep connection alive. Default: 64 */
    passthrough_threshold?: number;
    /** Use WFP token bucket algorithm (like NetLimiter) instead of inline packet pacing */
    use_wfp?: boolean;
}

export interface BurstOptions {
    enabled?: boolean;
    inbound?: boolean;
    outbound?: boolean;
    probability: number;
    buffer_ms: number;
    duration_ms: number;
    keepalive_ms: number;
    release_delay_us: number;
    reverse?: boolean;
}

export interface ProcessingStatus {
    running: boolean;
    modules: ModuleInfo[];
}

// Filter target types for the filter selector
export type FilterTargetMode = "all" | "process" | "device" | "custom";

export interface FilterTarget {
    mode: FilterTargetMode;
    processId?: number;
    processName?: string;
    deviceIp?: string;
    deviceName?: string;
    customFilter?: string;
    includeInbound?: boolean;
    includeOutbound?: boolean;
}

export interface ProcessInfo {
    pid: number;
    name: string;
    path?: string;
    window_title?: string;
    icon?: string;
}

export interface LoadConfigResponse {
    settings: PacketManipulationSettings;
    filter?: string;
    filter_target?: {
        mode: string;
        process_id?: number;
        process_name?: string;
        device_ip?: string;
        device_name?: string;
        custom_filter?: string;
        include_inbound?: boolean;
        include_outbound?: boolean;
    };
    hotkeys?: {
        action: string;
        shortcut: string | null;
        enabled: boolean;
    }[];
}

export interface NetworkDevice {
    ip: string;
    mac?: string;
    hostname?: string;
    device_type?: string;
}

declare global {
    interface Window {
        __TAURI__: {
            event: {
                listen: (
                    event: string,
                    callback: (event: { payload: any }) => void
                ) => Promise<() => void>;
            };
        };
    }
}
