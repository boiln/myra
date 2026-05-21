import { NetworkState } from "@/lib/stores/network/types";

export const DEFAULT_PRESET_NAME = "default";
export const DEFAULT_FILTER = "outbound";

export const initialState: NetworkState = {
    isActive: false,
    filter: DEFAULT_FILTER,
    filterTarget: { mode: "all" },
    manipulationStatus: {
        active: false,
        filter: "",
        modules: [],
    },
    isUpdatingFilter: false,
    isTogglingActive: false,
    presets: [],
    loadingPresets: false,
    currentPreset: null,
    isInitialized: false,
};
