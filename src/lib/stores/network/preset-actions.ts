import { StateCreator } from "zustand";
import { NetworkStore } from "@/lib/stores/network/types";
import { ManipulationService } from "@/lib/services/manipulation";
import { DEFAULT_PRESET_NAME } from "@/lib/stores/network/constants";

export const createPresetSlice: StateCreator<
    NetworkStore,
    [],
    [],
    Pick<
        NetworkStore,
        "loadPresets" | "savePreset" | "loadPreset" | "deletePreset" | "initializeDefaultPreset"
    >
> = (set, get) => ({
    loadPresets: async () => {
        try {
            set({ loadingPresets: true });
            const presets = await ManipulationService.listConfigs();
            set({ presets });
        } catch (error) {
            console.error("Failed to load presets:", error);
        } finally {
            set({ loadingPresets: false });
        }
    },

    savePreset: async (name: string) => {
        try {
            const settings = await ManipulationService.getSettings();
            const filter = await ManipulationService.getFilter();
            await ManipulationService.updateSettings(settings);
            await ManipulationService.updateFilter(filter);
            await ManipulationService.saveConfig(name);
            await get().loadPresets();
            set({ currentPreset: name });
        } catch (error) {
            console.error("Failed to save preset:", error);
        }
    },

    loadPreset: async (name: string) => {
        try {
            const settings = await ManipulationService.loadConfig(name);
            if (!settings) return;

            await ManipulationService.updateSettings(settings);
            // Get the current filter since loadConfig doesn't return it
            const filter = await ManipulationService.getFilter();
            await ManipulationService.updateFilter(filter);

            // Refresh state
            await get().loadStatus();
            set({ currentPreset: name });
        } catch (error) {
            console.error("Failed to load preset:", error);
        }
    },

    deletePreset: async (name: string) => {
        if (name === DEFAULT_PRESET_NAME) return;

        try {
            await ManipulationService.deleteConfig(name);
            await get().loadPresets();

            if (get().currentPreset === name) {
                set({ currentPreset: null });
            }
        } catch (error) {
            console.error("Failed to delete preset:", error);
        }
    },

    initializeDefaultPreset: async () => {
        try {
            const configs = await ManipulationService.listConfigs();
            const defaultExists = configs.includes(DEFAULT_PRESET_NAME);

            if (!defaultExists) {
                const settings = await ManipulationService.getSettings();
                const filter = await ManipulationService.getFilter();
                // First update the settings and filter
                await ManipulationService.updateSettings(settings);
                await ManipulationService.updateFilter(filter);
                // Then save the config
                await ManipulationService.saveConfig(DEFAULT_PRESET_NAME);
                await get().loadPresets();
            }
        } catch (error) {
            console.error("Failed to initialize default preset:", error);
        }
    },
});
