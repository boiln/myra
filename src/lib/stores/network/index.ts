import { create } from "zustand";
import { NetworkStore } from "@/lib/stores/network/types";
import { initialState } from "@/lib/stores/network/constants";
import { createCoreSlice } from "@/lib/stores/network/core-actions";
import { createModuleSlice } from "@/lib/stores/network/module-actions";
import { createPresetSlice } from "@/lib/stores/network/preset-actions";
import { ModuleInfo, PacketManipulationSettings } from "@/types";

// Build settings from modules - always includes all modules with enabled flag
const buildSettings = (modules: ModuleInfo[]) => {
    const settings: PacketManipulationSettings = {};

    modules.forEach((module) => {
        // Always include settings, with enabled flag to track active state
        switch (module.name) {
            case "delay":
                settings.delay = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                };
                break;

            case "drop":
                settings.drop = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                };
                break;

            case "throttle":
                settings.throttle = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    throttle_ms: module.config.throttle_ms || 30,
                };
                break;

            case "duplicate":
                settings.duplicate = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    count: module.config.count || 2,
                };
                break;

            case "bandwidth":
                settings.bandwidth = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    limit_kbps: module.config.limit_kbps || 100,
                };
                break;

            case "tamper":
                settings.tamper = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                };
                break;

            case "reorder":
                settings.reorder = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    max_delay: module.config.throttle_ms || 100,
                };
                break;

            case "burst":
                settings.burst = {
                    enabled: module.enabled,
                    inbound: module.config.inbound,
                    outbound: module.config.outbound,
                    probability: module.config.chance / 100,
                    duration_ms: module.config.duration_ms,
                    buffer_ms: module.config.buffer_ms ?? 0,
                    keepalive_ms: module.config.keepalive_ms ?? 0,
                    release_delay_us: module.config.release_delay_us ?? 500,
                };
                break;
        }
    });

    // Always include burst_release_delay_us
    const burstModule = modules.find((m) => m.name === "burst");
    if (burstModule) {
        settings.burst_release_delay_us = burstModule.config.release_delay_us ?? 500;
    }

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
