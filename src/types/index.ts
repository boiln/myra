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
    delay?: DelayOptions;
    throttle?: ThrottleOptions;
    reorder?: ReorderOptions;
    tamper?: TamperOptions;
    duplicate?: DuplicateOptions;
    bandwidth?: BandwidthOptions;
}

export interface DropOptions {
    probability: number;
    duration_ms: number;
}

export interface DelayOptions {
    probability: number;
    duration_ms: number;
}

export interface ThrottleOptions {
    probability: number;
    duration_ms: number;
    throttle_ms?: number;
}

export interface ReorderOptions {
    probability: number;
    duration_ms: number;
    max_delay?: number;
}

export interface TamperOptions {
    probability: number;
    duration_ms: number;
}

export interface DuplicateOptions {
    probability: number;
    count: number;
    duration_ms: number;
}

export interface BandwidthOptions {
    probability: number;
    limit_kbps: number;
    duration_ms: number;
}

export interface ProcessingStatus {
    running: boolean;
    modules: ModuleInfo[];
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
