import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import {
    ClassicModuleInfo,
    ClassicModuleName,
    CLASSIC_MODULE_DEFAULTS,
    modulesToBackendSettings,
    backendSettingsToModules,
    ClassicBackendSettings,
} from "@/types/classic";

interface ClassicStore {
    // State
    modules: ClassicModuleInfo[];
    isLoading: boolean;
    isActive: boolean;
    isProcessing: boolean;

    // Actions
    initialize: () => void;
    initializeFromBackend: (settings: ClassicBackendSettings) => void;
    toggleModule: (moduleName: ClassicModuleName) => Promise<void>;
    toggleDirection: (
        moduleName: ClassicModuleName,
        direction: "inbound" | "outbound"
    ) => Promise<void>;
    updateModuleConfig: (
        moduleName: ClassicModuleName,
        config: Partial<Record<string, unknown>>
    ) => Promise<void>;
    setActive: (active: boolean) => void;
    startProcessing: (filter?: string) => Promise<void>;
    stopProcessing: () => Promise<void>;
    syncToBackend: () => Promise<void>;
    getBackendSettings: () => ClassicBackendSettings;
}

export const useClassicStore = create<ClassicStore>()((set, get) => ({
    // Initial state
    modules: [],
    isLoading: true,
    isActive: false,
    isProcessing: false,
    // Initialize with default modules
    initialize: () => {
        const defaultModules = Object.values(CLASSIC_MODULE_DEFAULTS);
        set({ modules: defaultModules, isLoading: false });
    },
    // Initialize from backend settings (for config loading)
    initializeFromBackend: (settings: ClassicBackendSettings) => {
        const modules = backendSettingsToModules(settings);
        set({ modules, isLoading: false });
    },
    // Toggle module enabled state
    toggleModule: async (moduleName: ClassicModuleName) => {

        const { modules, isProcessing, syncToBackend } = get();
        const updatedModules = modules.map((m) =>
            m.name === moduleName
                ? {
                      ...m,
                      enabled: !m.enabled,
                      config: { ...m.config, enabled: !m.enabled },
                  }
                : m
        );
        set({ modules: updatedModules });
        // Sync to backend if processing is active
        if (isProcessing) {
            await syncToBackend();
        }

    },
    // Toggle inbound/outbound direction
    toggleDirection: async (moduleName: ClassicModuleName, direction: "inbound" | "outbound") => {

        const { modules, isProcessing, syncToBackend } = get();
        const updatedModules = modules.map((m) =>
            m.name === moduleName
                ? {
                      ...m,
                      config: {
                          ...m.config,
                          [direction]: !m.config[direction],
                      },
                  }
                : m
        );
        set({ modules: updatedModules });
        // Sync to backend if processing is active
        if (isProcessing) {
            await syncToBackend();
        }

    },
    // Update module configuration
    updateModuleConfig: async (
        moduleName: ClassicModuleName,
        config: Partial<Record<string, unknown>>
    ) => {

        const { modules, isProcessing, syncToBackend } = get();
        const updatedModules = modules.map((m) =>
            m.name === moduleName
                ? {
                      ...m,
                      config: { ...m.config, ...config },
                  }
                : m
        );
        set({ modules: updatedModules });
        // Sync to backend if processing is active
        if (isProcessing) {
            await syncToBackend();
        }

    },
    // Set active state (when filtering starts/stops)
    setActive: (active: boolean) => {
        set({ isActive: active });
    },
    // Start Classic mode packet processing
    startProcessing: async (filter?: string) => {

        const { modules } = get();
        const settings = modulesToBackendSettings(modules);

        try {
            await invoke("start_classic_processing", { settings, filter });
            set({ isProcessing: true, isActive: true });
        } catch (error) {
            console.error("Failed to start Classic processing:", error);
            throw error;
        }

    },
    // Stop Classic mode packet processing
    stopProcessing: async () => {

        try {
            await invoke("stop_classic_processing");
            set({ isProcessing: false });
        } catch (error) {
            console.error("Failed to stop Classic processing:", error);
            throw error;
        }

    },
    // Sync current module settings to backend
    syncToBackend: async () => {

        const { modules, isProcessing } = get();

        if (!isProcessing) return;
        const settings = modulesToBackendSettings(modules);

        try {
            await invoke("update_classic_settings", { settings });
        } catch (error) {
            console.error("Failed to sync Classic settings:", error);
            throw error;
        }

    },
    // Get current settings in backend format (for config saving)
    getBackendSettings: () => {
        const { modules } = get();

        return modulesToBackendSettings(modules);
    },
}));
