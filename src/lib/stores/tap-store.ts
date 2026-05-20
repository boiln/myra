import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface TapSettings {
    enabled: boolean;
    intervalMs: number; // How often to tap (every X ms)
    durationMs: number; // How long to keep modules off (X ms)
    autoEnabled?: boolean; // Auto mode: trigger taps based on buffer pressure
    minBufferForTap?: number; // Threshold of buffered/lagged packets to trigger
    cooldownMs?: number; // Minimum quiet period between auto taps
}

interface TapState {
    settings: TapSettings;
    isTapping: boolean; // Currently in a tap (modules temporarily off)
}

interface TapActions {
    setEnabled: (enabled: boolean) => void;
    setIntervalMs: (ms: number) => void;
    setDurationMs: (ms: number) => void;
    setIsTapping: (isTapping: boolean) => void;
    updateSettings: (settings: Partial<TapSettings>) => void;
    setAutoEnabled: (enabled: boolean) => void;
    setMinBufferForTap: (n: number) => void;
    setCooldownMs: (ms: number) => void;
}

type TapStore = TapState & TapActions;

const DEFAULT_SETTINGS: TapSettings = {
    enabled: false,
    intervalMs: 3000, // Every 3000ms (3 seconds)
    durationMs: 600, // Off for 600ms (0.6 seconds)
    autoEnabled: false,
    minBufferForTap: 200,
    cooldownMs: 1200,
};

export const useTapStore = create<TapStore>()(
    persist(
        (set) => ({
            settings: DEFAULT_SETTINGS,
            isTapping: false,

            setEnabled: (enabled) =>
                set((state) => ({
                    settings: { ...state.settings, enabled },
                })),

            setIntervalMs: (intervalMs) =>
                set((state) => ({
                    settings: { ...state.settings, intervalMs },
                })),

            setDurationMs: (durationMs) =>
                set((state) => ({
                    settings: { ...state.settings, durationMs },
                })),

            setIsTapping: (isTapping) => set({ isTapping }),

            updateSettings: (newSettings) =>
                set((state) => ({
                    settings: { ...state.settings, ...newSettings },
                })),

            setAutoEnabled: (enabled) =>
                set((state) => ({ settings: { ...state.settings, autoEnabled: enabled } })),

            setMinBufferForTap: (n) =>
                set((state) => ({ settings: { ...state.settings, minBufferForTap: n } })),

            setCooldownMs: (ms) =>
                set((state) => ({ settings: { ...state.settings, cooldownMs: ms } })),
        }),
        {
            name: "myra-tap-settings",
            partialize: (state) => ({ settings: state.settings }),
        }
    )
);
