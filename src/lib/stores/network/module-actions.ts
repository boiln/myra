import { StateCreator } from "zustand";
import { NetworkStore } from "@/lib/stores/network/types";
import { ManipulationService } from "@/lib/services/manipulation";
import { ModuleConfig, PacketManipulationSettings } from "@/types";

export const createModuleSlice: StateCreator<
    NetworkStore,
    [],
    [],
    Pick<
        NetworkStore,
        "updateModuleConfig" | "updateModuleSettings" | "toggleDirection" | "applyModuleSettings"
    >
> = (set, get) => ({
    updateModuleConfig: async (moduleName: string, config: Record<string, any>) => {
        const newSettings: PacketManipulationSettings = {
            ...(await ManipulationService.getSettings()),
        };

        const probability = Math.max(0.0001, (config.chance || 100) / 100);
        const duration_ms = config.duration_ms || 0;

        switch (moduleName) {
            case "lag":
                newSettings.lag = {
                    probability,
                    delay_ms: config.duration_ms || 100,  // UI duration_ms is the lag time
                    duration_ms: 0,  // Effect duration (0 = infinite)
                };
                break;

            case "drop":
                newSettings.drop = {
                    probability,
                    duration_ms,
                };
                break;

            case "throttle":
                newSettings.throttle = {
                    probability,
                    duration_ms,
                };
                break;

            case "duplicate":
                newSettings.duplicate = {
                    probability,
                    count: config.count || 1,
                    duration_ms,
                };
                break;

            case "bandwidth":
                newSettings.bandwidth = {
                    probability,
                    limit: config.limit_kbps || 500,  // Map UI limit_kbps to Rust limit
                    duration_ms,
                    use_wfp: config.use_wfp ?? false,
                };
                break;

            case "tamper":
                newSettings.tamper = {
                    probability,
                    duration_ms,
                };
                break;

            case "reorder":
                newSettings.reorder = {
                    probability,
                    duration_ms,
                    max_delay: config.throttle_ms || 100,
                };
                break;

            case "burst":
                // Use preserved release_delay_us from settings if available
                const preservedReleaseDelay = newSettings.burst_release_delay_us ?? config.release_delay_us ?? 500;
                newSettings.burst = {
                    probability,
                    buffer_ms: config.buffer_ms ?? 0,
                    keepalive_ms: config.keepalive_ms ?? 0,
                    release_delay_us: preservedReleaseDelay,
                    duration_ms,
                };
                // Also update top-level preserved value
                newSettings.burst_release_delay_us = preservedReleaseDelay;
                break;
        }

        try {
            await ManipulationService.updateSettings(newSettings, get().isActive);
            await get().loadStatus();
        } catch (error) {
            console.error("Failed to update module config:", error);
        }
    },

    updateModuleSettings: async (moduleName: string, config: ModuleConfig) => {
        const { manipulationStatus } = get();
        const moduleIndex = manipulationStatus.modules.findIndex((m) => m.name === moduleName);

        if (moduleIndex === -1) return;

        const updatedModules = [...manipulationStatus.modules];

        updatedModules[moduleIndex] = {
            ...updatedModules[moduleIndex],
            config: {
                ...updatedModules[moduleIndex].config,
                ...config,
            },
        };

        set({
            manipulationStatus: {
                ...manipulationStatus,
                modules: updatedModules,
            },
        });

        try {
            await ManipulationService.updateSettings(await get().buildSettings(), get().isActive);
        } catch (error) {
            console.error("Failed to update module settings:", error);
        }
    },

    toggleDirection: async (moduleName: string, direction: "inbound" | "outbound") => {
        const { manipulationStatus } = get();
        const moduleIndex = manipulationStatus.modules.findIndex((m) => m.name === moduleName);

        if (moduleIndex === -1) return;

        const module = manipulationStatus.modules[moduleIndex];

        const newConfig = {
            ...module.config,
            [direction]: !module.config[direction],
        };

        await get().updateModuleSettings(moduleName, newConfig);
    },

    applyModuleSettings: async (moduleName: string, enabled: boolean) => {
        const { manipulationStatus } = get();
        const moduleIndex = manipulationStatus.modules.findIndex((m) => m.name === moduleName);

        if (moduleIndex === -1) return;

        const updatedModules = [...manipulationStatus.modules];

        updatedModules[moduleIndex] = {
            ...updatedModules[moduleIndex],
            enabled,
            config: {
                ...updatedModules[moduleIndex].config,
                enabled,
            },
        };

        set({
            manipulationStatus: {
                ...manipulationStatus,
                modules: updatedModules,
            },
        });

        try {
            await ManipulationService.updateSettings(await get().buildSettings(), get().isActive);
        } catch (error) {
            console.error("Failed to apply module settings:", error);
        }
    },
});
