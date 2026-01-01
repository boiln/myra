import { invoke } from "@tauri-apps/api/core";
import {
    FilterTarget,
    LoadConfigResponse,
    PacketManipulationSettings,
    ProcessingStatus,
} from "@/types";

export const ManipulationService = {
    async startProcessing(settings: PacketManipulationSettings, filter?: string): Promise<void> {
        // The backend `start_processing` command expects settings directly
        return invoke("start_processing", { settings, filter });
    },

    async stopProcessing(): Promise<void> {
        return invoke("stop_processing");
    },

    async getStatus(): Promise<ProcessingStatus> {
        return invoke("get_status");
    },

    async updateSettings(settings: PacketManipulationSettings): Promise<void> {
        // Create the modules array from settings
        const modules = this.createModulesFromSettings(settings);
        return invoke("update_settings", { modules });
    },

    // Helper function to convert settings to modules array
    // Always sends all modules with their settings, using enabled field to track active state
    createModulesFromSettings(settings: PacketManipulationSettings): any[] {
        const modules = [];

        // Delay module - always include
        const delay = settings.delay || { enabled: false, inbound: true, outbound: true, probability: 1, duration_ms: 1000 };
        modules.push({
            name: "delay",
            display_name: "Delay",
            enabled: delay.enabled ?? false,
            config: {
                inbound: delay.inbound ?? true,
                outbound: delay.outbound ?? true,
                chance: delay.probability * 100,
                enabled: delay.enabled ?? false,
                duration_ms: delay.duration_ms,
            },
            params: null,
        });

        // Freeze module (Drop packets) - always include
        const drop = settings.drop || { enabled: false, inbound: true, outbound: true, probability: 1, duration_ms: 0 };
        modules.push({
            name: "drop",
            display_name: "Freeze",
            enabled: drop.enabled ?? false,
            config: {
                inbound: drop.inbound ?? true,
                outbound: drop.outbound ?? true,
                chance: drop.probability * 100,
                enabled: drop.enabled ?? false,
                duration_ms: drop.duration_ms,
            },
            params: null,
        });

        // Throttle module - always include
        const throttle = settings.throttle || { enabled: false, inbound: true, outbound: true, probability: 1, duration_ms: 0, throttle_ms: 300 };
        modules.push({
            name: "throttle",
            display_name: "Throttle",
            enabled: throttle.enabled ?? false,
            config: {
                inbound: throttle.inbound ?? true,
                outbound: throttle.outbound ?? true,
                chance: throttle.probability * 100,
                enabled: throttle.enabled ?? false,
                duration_ms: throttle.duration_ms,
                throttle_ms: throttle.throttle_ms,
            },
            params: null,
        });

        // Duplicate module - always include
        const duplicate = settings.duplicate || { enabled: false, inbound: true, outbound: true, probability: 1, count: 2, duration_ms: 0 };
        modules.push({
            name: "duplicate",
            display_name: "Duplicate",
            enabled: duplicate.enabled ?? false,
            config: {
                inbound: duplicate.inbound ?? true,
                outbound: duplicate.outbound ?? true,
                chance: duplicate.probability * 100,
                enabled: duplicate.enabled ?? false,
                duration_ms: duplicate.duration_ms,
                count: duplicate.count,
            },
            params: null,
        });

        // Bandwidth module - always include
        const bandwidth = settings.bandwidth || { enabled: false, inbound: true, outbound: true, probability: 1, limit_kbps: 50, duration_ms: 0 };
        modules.push({
            name: "bandwidth",
            display_name: "Bandwidth",
            enabled: bandwidth.enabled ?? false,
            config: {
                inbound: bandwidth.inbound ?? true,
                outbound: bandwidth.outbound ?? true,
                chance: bandwidth.probability * 100,
                enabled: bandwidth.enabled ?? false,
                duration_ms: bandwidth.duration_ms,
                limit_kbps: bandwidth.limit_kbps,
            },
            params: null,
        });

        // Tamper module - always include
        const tamper = settings.tamper || { enabled: false, inbound: true, outbound: true, probability: 1, duration_ms: 0 };
        modules.push({
            name: "tamper",
            display_name: "Tamper",
            enabled: tamper.enabled ?? false,
            config: {
                inbound: tamper.inbound ?? true,
                outbound: tamper.outbound ?? true,
                chance: tamper.probability * 100,
                enabled: tamper.enabled ?? false,
                duration_ms: tamper.duration_ms,
            },
            params: null,
        });

        // Reorder module - always include
        const reorder = settings.reorder || { enabled: false, inbound: true, outbound: true, probability: 1, duration_ms: 0, max_delay: 1000 };
        modules.push({
            name: "reorder",
            display_name: "Reorder",
            enabled: reorder.enabled ?? false,
            config: {
                inbound: reorder.inbound ?? true,
                outbound: reorder.outbound ?? true,
                chance: reorder.probability * 100,
                enabled: reorder.enabled ?? false,
                duration_ms: reorder.duration_ms,
                throttle_ms: reorder.max_delay,
            },
            params: null,
        });

        // Burst module (lag switch) - always include
        const burst = settings.burst || { 
            enabled: false, 
            inbound: true,
            outbound: true,
            probability: 1, 
            buffer_ms: 0, 
            duration_ms: 0, 
            keepalive_ms: 0, 
            release_delay_us: settings.burst_release_delay_us ?? 500 
        };
        modules.push({
            name: "burst",
            display_name: "Burst",
            enabled: burst.enabled ?? false,
            config: {
                inbound: burst.inbound ?? true,
                outbound: burst.outbound ?? true,
                chance: burst.probability * 100,
                enabled: burst.enabled ?? false,
                duration_ms: burst.duration_ms,
                buffer_ms: burst.buffer_ms,
                keepalive_ms: burst.keepalive_ms,
                release_delay_us: burst.release_delay_us,
            },
            params: null,
        });

        return modules;
    },

    async getSettings(): Promise<PacketManipulationSettings> {
        return invoke("get_settings");
    },

    async updateFilter(filter: string | null): Promise<void> {
        return invoke("update_filter", { filter });
    },

    async getFilter(): Promise<string | null> {
        return invoke("get_filter");
    },

    async saveConfig(
        name: string,
        filterTarget?: FilterTarget,
        hotkeys?: { action: string; shortcut: string | null; enabled: boolean }[]
    ): Promise<void> {
        return invoke("save_config", { name, filterTarget, hotkeys });
    },

    async loadConfig(name: string): Promise<LoadConfigResponse> {
        return invoke("load_config", { name });
    },

    async listConfigs(): Promise<string[]> {
        return invoke("list_configs");
    },

    async deleteConfig(name: string): Promise<void> {
        return invoke("delete_config", { name });
    },
};
