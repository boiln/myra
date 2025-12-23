import { invoke } from "@tauri-apps/api/core";
import { PacketManipulationSettings, ProcessingStatus } from "@/types";

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
    createModulesFromSettings(settings: PacketManipulationSettings): any[] {
        const modules = [];

        // Freeze module (Drop packets)
        if (settings.drop) {
            modules.push({
                name: "drop",
                display_name: "Freeze",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.drop.probability * 100,
                    enabled: true,
                    duration_ms: settings.drop.duration_ms,
                },
                params: null,
            });
        }

        // Delay module
        if (settings.delay) {
            modules.push({
                name: "delay",
                display_name: "Delay",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.delay.probability * 100,
                    enabled: true,
                    duration_ms: settings.delay.duration_ms, // Delay time
                },
                params: null,
            });
        }

        // Throttle module
        if (settings.throttle) {
            modules.push({
                name: "throttle",
                display_name: "Throttle",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.throttle.probability * 100,
                    enabled: true,
                    duration_ms: settings.throttle.duration_ms,
                    throttle_ms: settings.throttle.throttle_ms,
                },
                params: null,
            });
        }

        // Duplicate module
        if (settings.duplicate) {
            modules.push({
                name: "duplicate",
                display_name: "Duplicate",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.duplicate.probability * 100,
                    enabled: true,
                    duration_ms: settings.duplicate.duration_ms,
                    count: settings.duplicate.count,
                },
                params: null,
            });
        }

        // Bandwidth module
        if (settings.bandwidth) {
            modules.push({
                name: "bandwidth",
                display_name: "Bandwidth",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.bandwidth.probability * 100,
                    enabled: true,
                    duration_ms: settings.bandwidth.duration_ms,
                    limit_kbps: settings.bandwidth.limit_kbps,
                },
                params: null,
            });
        }

        // Tamper module
        if (settings.tamper) {
            modules.push({
                name: "tamper",
                display_name: "Tamper",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.tamper.probability * 100,
                    enabled: true,
                    duration_ms: settings.tamper.duration_ms,
                },
                params: null,
            });
        }

        // Reorder module
        if (settings.reorder) {
            modules.push({
                name: "reorder",
                display_name: "Reorder",
                enabled: true,
                config: {
                    inbound: true,
                    outbound: true,
                    chance: settings.reorder.probability * 100,
                    enabled: true,
                    duration_ms: settings.reorder.duration_ms,
                    throttle_ms: settings.reorder.max_delay,
                },
                params: null,
            });
        }

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

    async saveConfig(name: string): Promise<void> {
        return invoke("save_config", { name });
    },

    async loadConfig(name: string): Promise<PacketManipulationSettings> {
        return invoke("load_config", { name });
    },

    async listConfigs(): Promise<string[]> {
        return invoke("list_configs");
    },

    async deleteConfig(name: string): Promise<void> {
        return invoke("delete_config", { name });
    },
};
