import { invoke } from "@tauri-apps/api/core";
import {
    FilterTarget,
    LoadConfigResponse,
    PacketManipulationSettings,
    ProcessingStatus,
} from "@/types";

// Track WFP throttle state to avoid duplicate start/stop calls
let wfpThrottleActive = false;

export const ManipulationService = {
    async startProcessing(settings: PacketManipulationSettings, filter?: string): Promise<void> {
        // Start the filtering first
        await invoke("start_processing", { settings, filter });
        // Then start WFP throttle if needed (filtering is now active)
        await this.handleWfpThrottle(settings, true);
    },

    async stopProcessing(): Promise<void> {
        // Also stop WFP throttle if active
        if (wfpThrottleActive) {
            await this.stopWfpThrottle();
        }
        return invoke("stop_processing");
    },

    async getStatus(): Promise<ProcessingStatus> {
        return invoke("get_status");
    },

    async updateSettings(settings: PacketManipulationSettings, isFilteringActive: boolean = false): Promise<void> {
        // Handle WFP throttle based on bandwidth settings AND filtering state
        await this.handleWfpThrottle(settings, isFilteringActive);
        
        // Create the modules array from settings
        const modules = this.createModulesFromSettings(settings);
        return invoke("update_settings", { modules });
    },

    // Handle WFP throttle based on bandwidth settings - only starts when filtering is active
    async handleWfpThrottle(settings: PacketManipulationSettings, isFilteringActive: boolean): Promise<void> {
        const bandwidth = settings.bandwidth;
        // WFP should only be active when: bandwidth enabled + use_wfp checked + filtering is running
        const shouldBeActive = bandwidth?.enabled && bandwidth?.use_wfp && isFilteringActive;
        
        if (shouldBeActive && !wfpThrottleActive) {
            // Start WFP throttle
            const direction = bandwidth.inbound && bandwidth.outbound ? "both" 
                : bandwidth.inbound ? "inbound" 
                : "outbound";
            await this.startWfpThrottle(bandwidth.limit || 1, direction);
        } else if (!shouldBeActive && wfpThrottleActive) {
            // Stop WFP throttle
            await this.stopWfpThrottle();
        } else if (shouldBeActive && wfpThrottleActive) {
            // Update: restart with new settings
            const direction = bandwidth!.inbound && bandwidth!.outbound ? "both" 
                : bandwidth!.inbound ? "inbound" 
                : "outbound";
            await this.stopWfpThrottle();
            await this.startWfpThrottle(bandwidth!.limit || 1, direction);
        }
    },

    async startWfpThrottle(limitKbps: number, direction: string): Promise<void> {
        try {
            await invoke("start_tc_bandwidth", { limitKbps, direction });
            wfpThrottleActive = true;
        } catch (e) {
            console.error("Failed to start WFP throttle:", e);
        }
    },

    async stopWfpThrottle(): Promise<void> {
        try {
            await invoke("stop_tc_bandwidth");
            wfpThrottleActive = false;
        } catch (e) {
            console.error("Failed to stop WFP throttle:", e);
        }
    },

    // Helper function to convert settings to modules array
    // Always sends all modules with their settings, using enabled field to track active state
    createModulesFromSettings(settings: PacketManipulationSettings): any[] {
        const modules = [];

        // Lag module - always include
        const lag = settings.lag || { enabled: false, inbound: true, outbound: true, probability: 1, delay_ms: 1000, duration_ms: 0 };
        modules.push({
            name: "lag",
            display_name: "Lag",
            enabled: lag.enabled ?? false,
            config: {
                inbound: lag.inbound ?? true,
                outbound: lag.outbound ?? true,
                chance: lag.probability * 100,
                enabled: lag.enabled ?? false,
                duration_ms: lag.delay_ms,  // Use lag_ms as the time value sent to backend
            },
            params: null,
        });

        // Drop module - always include
        const drop = settings.drop || { enabled: false, inbound: true, outbound: true, probability: 1, duration_ms: 0 };
        modules.push({
            name: "drop",
            display_name: "Drop",
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
        const bandwidth = settings.bandwidth || { enabled: false, inbound: true, outbound: true, probability: 1, limit: 50, duration_ms: 0, use_wfp: false };
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
                limit_kbps: bandwidth.limit,  // Map Rust 'limit' to frontend 'limit_kbps'
                use_wfp: bandwidth.use_wfp ?? false,
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
                lag_bypass: settings.lag_bypass ?? false,  // MGO2 bypass mode
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
        // Convert camelCase to snake_case for Rust
        const rustFilterTarget = filterTarget ? {
            mode: filterTarget.mode,
            process_id: filterTarget.processId,
            process_name: filterTarget.processName,
            device_ip: filterTarget.deviceIp,
            device_name: filterTarget.deviceName,
            custom_filter: filterTarget.customFilter,
            include_inbound: filterTarget.includeInbound ?? true,
            include_outbound: filterTarget.includeOutbound ?? true,
        } : undefined;
        
        return invoke("save_config", { name, filterTarget: rustFilterTarget, hotkeys });
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
