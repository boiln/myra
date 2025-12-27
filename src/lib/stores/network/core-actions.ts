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

            // Create modules array from settings
            const modules: ModuleInfo[] = [
                {
                    name: "delay",
                    display_name: "Delay",
                    enabled: !!settings.delay,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.delay ? Math.round(settings.delay.probability * 100) : 100,
                        enabled: !!settings.delay,
                        duration_ms: settings.delay?.duration_ms || 1000,
                    },
                },
                {
                    name: "drop",
                    display_name: "Freeze",
                    enabled: !!settings.drop,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.drop ? Math.round(settings.drop.probability * 100) : 100,
                        enabled: !!settings.drop,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "throttle",
                    display_name: "Throttle",
                    enabled: !!settings.throttle,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.throttle
                            ? Math.round(settings.throttle.probability * 100)
                            : 100,
                        enabled: !!settings.throttle,
                        throttle_ms: settings.throttle?.throttle_ms || 30,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "duplicate",
                    display_name: "Duplicate",
                    enabled: !!settings.duplicate,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.duplicate
                            ? Math.round(settings.duplicate.probability * 100)
                            : 100,
                        enabled: !!settings.duplicate,
                        count: settings.duplicate?.count || 2,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "bandwidth",
                    display_name: "Bandwidth",
                    enabled: !!settings.bandwidth,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.bandwidth
                            ? Math.round(settings.bandwidth.probability * 100)
                            : 100,
                        enabled: !!settings.bandwidth,
                        limit_kbps: settings.bandwidth?.limit_kbps || 500,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "tamper",
                    display_name: "Tamper",
                    enabled: !!settings.tamper,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.tamper
                            ? Math.round(settings.tamper.probability * 100)
                            : 100,
                        enabled: !!settings.tamper,
                        duration_ms: 0, // 0 = infinite effect duration
                    },
                },
                {
                    name: "reorder",
                    display_name: "Reorder",
                    enabled: !!settings.reorder,
                    config: {
                        inbound: true,
                        outbound: true,
                        chance: settings.reorder
                            ? Math.round(settings.reorder.probability * 100)
                            : 100,
                        enabled: !!settings.reorder,
                        throttle_ms: settings.reorder?.max_delay || 100,
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
        const { isActive, filter } = get();

        try {
            set({ isTogglingActive: true });
            set({ isActive: !isActive });

            if (isActive) {
                await ManipulationService.stopProcessing();
            }

            if (!isActive) {
                const settings = await ManipulationService.getSettings();
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
