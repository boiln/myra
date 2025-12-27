import {
    FilterTarget,
    ManipulationStatus,
    ModuleConfig,
    PacketManipulationSettings,
} from "@/types";

export interface NetworkState {
    // Status
    isActive: boolean;
    filter: string;
    filterTarget: FilterTarget | null;
    manipulationStatus: ManipulationStatus;
    isUpdatingFilter: boolean;
    isTogglingActive: boolean;
    presets: string[];
    loadingPresets: boolean;
    currentPreset: string | null;
}

export interface NetworkActions {
    // Core actions
    toggleActive: () => Promise<void>;
    updateFilter: (newFilter: string) => Promise<void>;
    setFilterTarget: (target: FilterTarget) => void;
    loadStatus: () => Promise<void>;

    // Module actions
    updateModuleConfig: (moduleName: string, config: Record<string, any>) => Promise<void>;
    updateModuleSettings: (moduleName: string, config: ModuleConfig) => Promise<void>;
    toggleDirection: (moduleName: string, direction: "inbound" | "outbound") => Promise<void>;
    applyModuleSettings: (moduleName: string, enabled: boolean) => Promise<void>;

    // Preset actions
    loadPresets: () => Promise<void>;
    savePreset: (name: string) => Promise<void>;
    loadPreset: (name: string) => Promise<void>;
    deletePreset: (name: string) => Promise<void>;
    initializeDefaultPreset: () => Promise<void>;

    // Utils
    buildSettings: () => PacketManipulationSettings;
}

export type NetworkStore = NetworkState & NetworkActions;
