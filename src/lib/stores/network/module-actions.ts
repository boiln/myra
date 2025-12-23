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

        switch (moduleName) {
            case "delay":
                if (!newSettings.delay) {
                    newSettings.delay = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        duration_ms: config.duration_ms || 100,
                    };
                } else {
                    newSettings.delay.probability = Math.max(0.0001, (config.chance || 100) / 100);
                    newSettings.delay.duration_ms = config.duration_ms || 100;
                }
                break;
            case "drop":
                if (!newSettings.drop) {
                    newSettings.drop = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        duration_ms: config.duration_ms || 0,
                    };
                } else {
                    newSettings.drop.probability = Math.max(0.0001, (config.chance || 100) / 100);
                    newSettings.drop.duration_ms = config.duration_ms || 0;
                }
                break;
            case "throttle":
                if (!newSettings.throttle) {
                    newSettings.throttle = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        duration_ms: config.duration_ms || 0,
                    };
                } else {
                    newSettings.throttle.probability = Math.max(
                        0.0001,
                        (config.chance || 100) / 100
                    );
                    newSettings.throttle.duration_ms = config.duration_ms || 0;
                }
                break;
            case "duplicate":
                if (!newSettings.duplicate) {
                    newSettings.duplicate = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        count: config.count || 1,
                        duration_ms: config.duration_ms || 0,
                    };
                } else {
                    newSettings.duplicate.probability = Math.max(
                        0.0001,
                        (config.chance || 100) / 100
                    );
                    newSettings.duplicate.count = config.count || 1;
                    newSettings.duplicate.duration_ms = config.duration_ms || 0;
                }
                break;
            case "bandwidth":
                if (!newSettings.bandwidth) {
                    newSettings.bandwidth = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        limit_kbps: config.limit_kbps || 500,
                        duration_ms: config.duration_ms || 0,
                    };
                } else {
                    newSettings.bandwidth.probability = Math.max(
                        0.0001,
                        (config.chance || 100) / 100
                    );
                    newSettings.bandwidth.limit_kbps = config.limit_kbps || 500;
                    newSettings.bandwidth.duration_ms = config.duration_ms || 0;
                }
                break;
            case "tamper":
                if (!newSettings.tamper) {
                    newSettings.tamper = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        duration_ms: config.duration_ms || 0,
                    };
                } else {
                    newSettings.tamper.probability = Math.max(0.0001, (config.chance || 100) / 100);
                    newSettings.tamper.duration_ms = config.duration_ms || 0;
                }
                break;
            case "reorder":
                if (!newSettings.reorder) {
                    newSettings.reorder = {
                        probability: Math.max(0.0001, (config.chance || 100) / 100),
                        duration_ms: config.duration_ms || 0,
                        max_delay: config.throttle_ms || 100,
                    };
                } else {
                    newSettings.reorder.probability = Math.max(
                        0.0001,
                        (config.chance || 100) / 100
                    );
                    newSettings.reorder.duration_ms = config.duration_ms || 0;
                    newSettings.reorder.max_delay = config.throttle_ms || 100;
                }
                break;
        }

        try {
            await ManipulationService.updateSettings(newSettings);
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
            await ManipulationService.updateSettings(await get().buildSettings());
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
            await ManipulationService.updateSettings(await get().buildSettings());
        } catch (error) {
            console.error("Failed to apply module settings:", error);
        }
    },
});
