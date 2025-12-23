import { create } from "zustand";
import { NetworkStore } from "@/lib/stores/network/types";
import { initialState } from "@/lib/stores/network/constants";
import { createCoreSlice } from "@/lib/stores/network/core-actions";
import { createModuleSlice } from "@/lib/stores/network/module-actions";
import { createPresetSlice } from "@/lib/stores/network/preset-actions";
import { ModuleInfo, PacketManipulationSettings } from "@/types";

const buildSettings = (modules: ModuleInfo[]) => {
    const settings: PacketManipulationSettings = {};

    modules.forEach((module) => {
        if (!module.enabled) return;

        switch (module.name) {
            case "delay":
                settings.delay = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                };
                break;

            case "drop":
                settings.drop = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                };
                break;

            case "throttle":
                settings.throttle = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    throttle_ms: module.config.throttle_ms || 30,
                };
                break;

            case "duplicate":
                settings.duplicate = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    count: module.config.count || 2,
                };
                break;

            case "bandwidth":
                settings.bandwidth = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    limit_kbps: module.config.limit_kbps || 100,
                };
                break;

            case "tamper":
                settings.tamper = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                };
                break;

            case "reorder":
                settings.reorder = {
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    max_delay: module.config.throttle_ms || 100,
                };
                break;
        }
    });

    return settings;
};

interface NetworkStoreWithUtils extends NetworkStore {
    buildSettings: () => PacketManipulationSettings;
}

export const useNetworkStore = create<NetworkStoreWithUtils>()((...a) => ({
    ...initialState,
    ...createCoreSlice(...a),
    ...createModuleSlice(...a),
    ...createPresetSlice(...a),
    buildSettings: () => buildSettings(a[1]().manipulationStatus.modules),
}));
