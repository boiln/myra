import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
    register,
    unregister,
    isRegistered,
} from "@tauri-apps/plugin-global-shortcut";

export interface HotkeyBinding {
    action: string;
    shortcut: string | null;
    enabled: boolean;
}

export interface HotkeyState {
    bindings: Record<string, HotkeyBinding>;
    isRecording: string | null; // action name being recorded, or null
}

export interface HotkeyActions {
    setBinding: (action: string, shortcut: string | null) => Promise<void>;
    toggleBinding: (action: string) => Promise<void>;
    startRecording: (action: string) => void;
    stopRecording: () => void;
    registerAllHotkeys: (handlers: Record<string, () => void>) => Promise<void>;
    unregisterAllHotkeys: () => Promise<void>;
    restoreBindings: (bindings: { action: string; shortcut: string | null; enabled: boolean }[]) => Promise<void>;
}

const DEFAULT_BINDINGS: Record<string, HotkeyBinding> = {
    toggleFilter: {
        action: "toggleFilter",
        shortcut: "F9",
        enabled: true,
    },
    toggleDrop: {
        action: "toggleDrop",
        shortcut: null,
        enabled: false,
    },
    toggleLag: {
        action: "toggleLag",
        shortcut: null,
        enabled: false,
    },
    toggleThrottle: {
        action: "toggleThrottle",
        shortcut: null,
        enabled: false,
    },
    toggleDuplicate: {
        action: "toggleDuplicate",
        shortcut: null,
        enabled: false,
    },
    toggleBandwidth: {
        action: "toggleBandwidth",
        shortcut: null,
        enabled: false,
    },
    toggleTamper: {
        action: "toggleTamper",
        shortcut: null,
        enabled: false,
    },
    toggleReorder: {
        action: "toggleReorder",
        shortcut: null,
        enabled: false,
    },
    toggleBurst: {
        action: "toggleBurst",
        shortcut: null,
        enabled: false,
    },
};

// Track registered shortcuts to avoid double-registration
let registeredShortcuts: Set<string> = new Set();
let currentHandlers: Record<string, () => void> = {};
let lastTriggerTime: Record<string, number> = {};
const DEBOUNCE_MS = 300; // Prevent rapid-fire triggers

export const useHotkeyStore = create<HotkeyState & HotkeyActions>()(
    persist(
        (set, get) => ({
            bindings: DEFAULT_BINDINGS,
            isRecording: null,

            setBinding: async (action: string, shortcut: string | null) => {
                const { bindings } = get();
                const oldBinding = bindings[action];

                // Unregister old shortcut if it existed
                if (oldBinding?.shortcut && registeredShortcuts.has(oldBinding.shortcut)) {
                    try {
                        await unregister(oldBinding.shortcut);
                        registeredShortcuts.delete(oldBinding.shortcut);
                    } catch (e) {
                        console.error("Failed to unregister old shortcut:", e);
                    }
                }

                // Update binding
                const newBinding: HotkeyBinding = {
                    action,
                    shortcut,
                    enabled: shortcut !== null,
                };

                set({
                    bindings: { ...bindings, [action]: newBinding },
                    isRecording: null,
                });

                // Register new shortcut if we have a handler
                if (shortcut && newBinding.enabled && currentHandlers[action]) {
                    try {
                        const alreadyRegistered = await isRegistered(shortcut);
                        if (!alreadyRegistered) {
                            await register(shortcut, currentHandlers[action]);
                            registeredShortcuts.add(shortcut);
                        }
                    } catch (e) {
                        console.error("Failed to register new shortcut:", e);
                    }
                }
            },

            toggleBinding: async (action: string) => {
                const { bindings } = get();
                const binding = bindings[action];
                if (!binding || !binding.shortcut) return;

                const newEnabled = !binding.enabled;

                set({
                    bindings: {
                        ...bindings,
                        [action]: { ...binding, enabled: newEnabled },
                    },
                });

                if (newEnabled && currentHandlers[action]) {
                    try {
                        const alreadyRegistered = await isRegistered(binding.shortcut);
                        if (!alreadyRegistered) {
                            await register(binding.shortcut, currentHandlers[action]);
                            registeredShortcuts.add(binding.shortcut);
                        }
                    } catch (e) {
                        console.error("Failed to register shortcut:", e);
                    }
                } else if (!newEnabled) {
                    try {
                        await unregister(binding.shortcut);
                        registeredShortcuts.delete(binding.shortcut);
                    } catch (e) {
                        console.error("Failed to unregister shortcut:", e);
                    }
                }
            },

            startRecording: (action: string) => {
                set({ isRecording: action });
            },

            stopRecording: () => {
                set({ isRecording: null });
            },

            registerAllHotkeys: async (handlers: Record<string, () => void>) => {
                // Wrap handlers with debounce and recording check
                const wrappedHandlers: Record<string, () => void> = {};
                for (const [action, handler] of Object.entries(handlers)) {
                    wrappedHandlers[action] = () => {
                        // Don't fire hotkeys while recording
                        if (get().isRecording) {
                            console.log(`Hotkey ${action} blocked - recording mode`);
                            return;
                        }
                        
                        // Debounce rapid triggers
                        const now = Date.now();
                        if (lastTriggerTime[action] && now - lastTriggerTime[action] < DEBOUNCE_MS) {
                            console.log(`Hotkey ${action} debounced`);
                            return;
                        }
                        lastTriggerTime[action] = now;
                        
                        handler();
                    };
                }
                
                currentHandlers = wrappedHandlers;
                const { bindings } = get();

                for (const [action, binding] of Object.entries(bindings)) {
                    if (binding.shortcut && binding.enabled && wrappedHandlers[action]) {
                        try {
                            const alreadyRegistered = await isRegistered(binding.shortcut);
                            if (!alreadyRegistered) {
                                await register(binding.shortcut, wrappedHandlers[action]);
                                registeredShortcuts.add(binding.shortcut);
                                console.log(`Registered hotkey: ${binding.shortcut} for ${action}`);
                            }
                        } catch (e) {
                            console.error(`Failed to register ${binding.shortcut}:`, e);
                        }
                    }
                }
            },

            unregisterAllHotkeys: async () => {
                for (const shortcut of registeredShortcuts) {
                    try {
                        await unregister(shortcut);
                    } catch (e) {
                        console.error(`Failed to unregister ${shortcut}:`, e);
                    }
                }
                registeredShortcuts.clear();
            },

            restoreBindings: async (bindings: { action: string; shortcut: string | null; enabled: boolean }[]) => {
                // Unregister all current hotkeys first
                for (const shortcut of registeredShortcuts) {
                    try {
                        await unregister(shortcut);
                    } catch (e) {
                        console.error(`Failed to unregister ${shortcut}:`, e);
                    }
                }
                registeredShortcuts.clear();

                // Build new bindings record
                const newBindings: Record<string, HotkeyBinding> = { ...get().bindings };
                for (const binding of bindings) {
                    newBindings[binding.action] = {
                        action: binding.action,
                        shortcut: binding.shortcut,
                        enabled: binding.enabled,
                    };
                }

                set({ bindings: newBindings });

                // Re-register enabled hotkeys
                for (const binding of Object.values(newBindings)) {
                    if (binding.shortcut && binding.enabled && currentHandlers[binding.action]) {
                        try {
                            const alreadyRegistered = await isRegistered(binding.shortcut);
                            if (!alreadyRegistered) {
                                await register(binding.shortcut, currentHandlers[binding.action]);
                                registeredShortcuts.add(binding.shortcut);
                                console.log(`Restored hotkey: ${binding.shortcut} for ${binding.action}`);
                            }
                        } catch (e) {
                            console.error(`Failed to register ${binding.shortcut}:`, e);
                        }
                    }
                }
            },
        }),
        {
            name: "myra-hotkeys",
            partialize: (state) => ({ bindings: state.bindings }),
        }
    )
);
