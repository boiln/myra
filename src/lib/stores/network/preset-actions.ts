import { StateCreator } from "zustand";
import { NetworkStore } from "@/lib/stores/network/types";
import { ManipulationService } from "@/lib/services/manipulation";
import { DEFAULT_PRESET_NAME } from "@/lib/stores/network/constants";
import { useHotkeyStore } from "@/lib/stores/hotkey-store";
import { useTapStore } from "@/lib/stores/tap-store";

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
            const filterTarget = get().filterTarget;
            
            // Get hotkey bindings
            const hotkeyBindings = useHotkeyStore.getState().bindings;
            const hotkeys = Object.values(hotkeyBindings).map((binding) => ({
                action: binding.action,
                shortcut: binding.shortcut,
                enabled: binding.enabled,
            }));
            
            // Get tap settings
            const tapSettings = useTapStore.getState().settings;
            const tap = {
                enabled: tapSettings.enabled,
                interval_ms: tapSettings.intervalMs,
                duration_ms: tapSettings.durationMs,
            };
            
            await ManipulationService.updateSettings(settings, get().isActive);
            await ManipulationService.updateFilter(filter);
            await ManipulationService.saveConfig(name, filterTarget || undefined, hotkeys, tap);
            await get().loadPresets();
            set({ currentPreset: name });
        } catch (error) {
            console.error("Failed to save preset:", error);
        }
    },

    loadPreset: async (name: string) => {
        try {
            const response = await ManipulationService.loadConfig(name);

            if (!response) return;

            await ManipulationService.updateSettings(response.settings, get().isActive);

            if (response.filter) {
                await ManipulationService.updateFilter(response.filter);
            }

            // Restore filter target if present
            if (response.filter_target) {
                // Direction is now radio-style (only one can be active)
                const inbound = response.filter_target.include_inbound ?? false;
                const outbound = response.filter_target.include_outbound ?? true;
                // If both true or both undefined, default to outbound only
                const finalInbound = inbound && !outbound;
                const finalOutbound = !finalInbound;
                
                set({
                    filterTarget: {
                        mode: response.filter_target.mode as
                            | "all"
                            | "process"
                            | "device"
                            | "custom",
                        processId: response.filter_target.process_id,
                        processName: response.filter_target.process_name,
                        deviceIp: response.filter_target.device_ip,
                        deviceName: response.filter_target.device_name,
                        customFilter: response.filter_target.custom_filter,
                        includeInbound: finalInbound,
                        includeOutbound: finalOutbound,
                    },
                });
            }

            // Restore hotkey bindings if present
            if (response.hotkeys && response.hotkeys.length > 0) {
                await useHotkeyStore.getState().restoreBindings(response.hotkeys);
            }

            // Restore tap settings if present (but always default enabled to false for safety)
            if (response.tap) {
                useTapStore.getState().updateSettings({
                    enabled: false,  // Always start with tap disabled for safety
                    intervalMs: response.tap.interval_ms,
                    durationMs: response.tap.duration_ms,
                });
            }

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

            if (get().currentPreset !== name) return;

            set({ currentPreset: null });
        } catch (error) {
            console.error("Failed to delete preset:", error);
        }
    },

    initializeDefaultPreset: async () => {
        try {
            const configs = await ManipulationService.listConfigs();

            if (configs.includes(DEFAULT_PRESET_NAME)) {
                // Default config exists - LOAD it on startup
                await get().loadPreset(DEFAULT_PRESET_NAME);
                return;
            }

            // No default config exists - create one from current settings
            const settings = await ManipulationService.getSettings();
            const filter = await ManipulationService.getFilter();
            const filterTarget = get().filterTarget;
            
            // Get tap settings (always save with enabled: false by default)
            const tapSettings = useTapStore.getState().settings;
            const tap = {
                enabled: false,  // Always default to disabled
                interval_ms: tapSettings.intervalMs,
                duration_ms: tapSettings.durationMs,
            };

            await ManipulationService.updateSettings(settings, get().isActive);
            await ManipulationService.updateFilter(filter);
            await ManipulationService.saveConfig(DEFAULT_PRESET_NAME, filterTarget || undefined, undefined, tap);

            await get().loadPresets();
        } catch (error) {
            console.error("Failed to initialize default preset:", error);
        }
    },
});
