import { StateCreator } from "zustand";
import { NetworkStore } from "@/lib/stores/network/types";
import { ManipulationService } from "@/lib/services/manipulation";
import { DEFAULT_FILTER } from "@/lib/stores/network/constants";
import { FilterTarget, ModuleInfo } from "@/types";

export const createCoreSlice: StateCreator<
    NetworkStore,
    [],
    [],
    Pick<NetworkStore, "toggleActive" | "updateFilter" | "setFilterTarget" | "loadStatus">
> = (set, get) => ({
    loadStatus: async () => {
        try {
            const status = await ManipulationService.getStatus();
            const currentFilter = await ManipulationService.getFilter();
            const settings = await ManipulationService.getSettings();

            // Get existing modules to preserve inbound/outbound settings
            const { manipulationStatus: existingStatus } = get();
            const existingModules = existingStatus.modules;

            // Helper to get existing direction settings for a module
            const getExistingDirections = (moduleName: string) => {
                const existing = existingModules.find((m) => m.name === moduleName);
                return {
                    inbound: existing?.config.inbound ?? true,
                    outbound: existing?.config.outbound ?? true,
                };
            };

            // Create modules array from settings, preserving direction settings
            const modules: ModuleInfo[] = [
                {
                    name: "lag",
                    display_name: "Lag",
                    enabled: settings.lag?.enabled ?? false,
                    config: {
                        inbound: settings.lag?.inbound ?? getExistingDirections("lag").inbound,
                        outbound: settings.lag?.outbound ?? getExistingDirections("lag").outbound,
                        chance: settings.lag ? Math.round(settings.lag.probability * 100) : 100,
                        enabled: settings.lag?.enabled ?? false,
                        duration_ms: settings.lag?.delay_ms || 1000,
                    },
                },
                {
                    name: "drop",
                    display_name: "Drop",
                    enabled: settings.drop?.enabled ?? false,
                    config: {
                        inbound: settings.drop?.inbound ?? getExistingDirections("drop").inbound,
                        outbound: settings.drop?.outbound ?? getExistingDirections("drop").outbound,
                        chance: settings.drop ? Math.round(settings.drop.probability * 100) : 100,
                        enabled: settings.drop?.enabled ?? false,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "throttle",
                    display_name: "Throttle",
                    enabled: settings.throttle?.enabled ?? false,
                    config: {
                        inbound: settings.throttle?.inbound ?? getExistingDirections("throttle").inbound,
                        outbound: settings.throttle?.outbound ?? getExistingDirections("throttle").outbound,
                        chance: settings.throttle
                            ? Math.round(settings.throttle.probability * 100)
                            : 100,
                        enabled: settings.throttle?.enabled ?? false,
                        throttle_ms: settings.throttle?.throttle_ms || 30,
                        freeze_mode: settings.throttle?.freeze_mode ?? false,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "duplicate",
                    display_name: "Duplicate",
                    enabled: settings.duplicate?.enabled ?? false,
                    config: {
                        inbound: settings.duplicate?.inbound ?? getExistingDirections("duplicate").inbound,
                        outbound: settings.duplicate?.outbound ?? getExistingDirections("duplicate").outbound,
                        chance: settings.duplicate
                            ? Math.round(settings.duplicate.probability * 100)
                            : 100,
                        enabled: settings.duplicate?.enabled ?? false,
                        count: settings.duplicate?.count || 2,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "bandwidth",
                    display_name: "Bandwidth",
                    enabled: settings.bandwidth?.enabled ?? false,
                    config: {
                        inbound: settings.bandwidth?.inbound ?? getExistingDirections("bandwidth").inbound,
                        outbound: settings.bandwidth?.outbound ?? getExistingDirections("bandwidth").outbound,
                        chance: settings.bandwidth
                            ? Math.round(settings.bandwidth.probability * 100)
                            : 100,
                        enabled: settings.bandwidth?.enabled ?? false,
                        limit_kbps: settings.bandwidth?.limit || 500,
                        use_wfp: settings.bandwidth?.use_wfp ?? false,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "tamper",
                    display_name: "Tamper",
                    enabled: settings.tamper?.enabled ?? false,
                    config: {
                        inbound: settings.tamper?.inbound ?? getExistingDirections("tamper").inbound,
                        outbound: settings.tamper?.outbound ?? getExistingDirections("tamper").outbound,
                        chance: settings.tamper
                            ? Math.round(settings.tamper.probability * 100)
                            : 100,
                        enabled: settings.tamper?.enabled ?? false,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "reorder",
                    display_name: "Reorder",
                    enabled: settings.reorder?.enabled ?? false,
                    config: {
                        inbound: settings.reorder?.inbound ?? getExistingDirections("reorder").inbound,
                        outbound: settings.reorder?.outbound ?? getExistingDirections("reorder").outbound,
                        chance: settings.reorder
                            ? Math.round(settings.reorder.probability * 100)
                            : 100,
                        enabled: settings.reorder?.enabled ?? false,
                        throttle_ms: settings.reorder?.max_delay || 100,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "burst",
                    display_name: "Burst",
                    enabled: settings.burst?.enabled ?? false,
                    config: {
                        inbound: settings.burst?.inbound ?? getExistingDirections("burst").inbound,
                        outbound: settings.burst?.outbound ?? getExistingDirections("burst").outbound,
                        chance: settings.burst
                            ? Math.round(settings.burst.probability * 100)
                            : 100,
                        enabled: settings.burst?.enabled ?? false,
                        buffer_ms: settings.burst?.buffer_ms ?? 0,
                        keepalive_ms: settings.burst?.keepalive_ms ?? 0,
                        // Use burst_release_delay_us from top-level settings (persists even when burst disabled)
                        release_delay_us: settings.burst_release_delay_us ?? settings.burst?.release_delay_us ?? 500,
                        reverse: settings.burst?.reverse ?? false,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
            ];

            // Preserve current filter if backend returns null - don't reset to default
            const { filter: existingFilter } = get();
            const newFilter = currentFilter || existingFilter || DEFAULT_FILTER;

            set({
                isActive: status.running,
                filter: newFilter,
                manipulationStatus: {
                    active: status.running,
                    filter: newFilter,
                    modules: modules,
                },
            });
        } catch (error) {
            console.error("Failed to load status:", error);
        }
    },

    toggleActive: async () => {
        const { isActive, filter, buildSettings } = get();

        try {
            set({ isTogglingActive: true });
            set({ isActive: !isActive });

            if (isActive) {
                await ManipulationService.stopProcessing();
            }

            if (!isActive) {
                // Use buildSettings() to get current UI state instead of backend state
                // This preserves boolean settings like freeze_mode, use_wfp, reverse
                const settings = buildSettings();
                await ManipulationService.startProcessing(settings, filter);
            }

            await get().loadStatus();
        } catch (error) {
            console.error("Failed to toggle active state:", error);
            set({ isActive: isActive });
        } finally {
            set({ isTogglingActive: false });
        }
    },

    updateFilter: async (newFilter: string) => {
        const { isActive } = get();

        try {
            set({ isUpdatingFilter: true });

            await ManipulationService.updateFilter(newFilter || DEFAULT_FILTER);
            set({ filter: newFilter || DEFAULT_FILTER });

            if (!isActive) {
                await get().loadStatus();
                return;
            }

            const settings = await ManipulationService.getSettings();
            await ManipulationService.stopProcessing();
            await ManipulationService.startProcessing(settings, newFilter);

            await get().loadStatus();
        } catch (error) {
            console.error("Failed to update filter:", error);
        } finally {
            set({ isUpdatingFilter: false });
        }
    },

    setFilterTarget: (target: FilterTarget) => {
        set({ filterTarget: target });
    },
});
