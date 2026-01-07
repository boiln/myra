import { useEffect, useCallback } from "react";
import { useHotkeyStore } from "@/lib/stores/hotkey-store";
import { useNetworkStore } from "@/lib/stores/network";
import { toast } from "sonner";

export function useHotkeys() {
    const { registerAllHotkeys, unregisterAllHotkeys } = useHotkeyStore();
    const { toggleActive, applyModuleSettings, manipulationStatus } = useNetworkStore();

    const getModuleEnabled = useCallback(
        (moduleName: string) => {
            const module = manipulationStatus.modules.find((m) => m.name === moduleName);
            return module?.enabled ?? false;
        },
        [manipulationStatus.modules]
    );

    const toggleModule = useCallback(
        async (moduleName: string, displayName: string) => {
            const currentEnabled = getModuleEnabled(moduleName);
            await applyModuleSettings(moduleName, !currentEnabled);
            toast.success(`${displayName} ${currentEnabled ? "disabled" : "enabled"}`, {
                duration: 1500,
            });
        },
        [getModuleEnabled, applyModuleSettings]
    );

    useEffect(() => {
        const handlers: Record<string, () => void> = {
            toggleFilter: () => {
                console.log("Hotkey: Toggle Filter");
                toggleActive();
            },
            toggleDrop: () => {
                console.log("Hotkey: Toggle Drop");
                toggleModule("drop", "Drop");
            },
            toggleLag: () => {
                console.log("Hotkey: Toggle Lag");
                toggleModule("lag", "Lag");
            },
            toggleThrottle: () => {
                console.log("Hotkey: Toggle Throttle");
                toggleModule("throttle", "Throttle");
            },
            toggleDuplicate: () => {
                console.log("Hotkey: Toggle Duplicate");
                toggleModule("duplicate", "Duplicate");
            },
            toggleBandwidth: () => {
                console.log("Hotkey: Toggle Bandwidth");
                toggleModule("bandwidth", "Bandwidth");
            },
            toggleCorruption: () => {
                console.log("Hotkey: Toggle Corruption");
                toggleModule("corruption", "Corruption");
            },
            toggleReorder: () => {
                console.log("Hotkey: Toggle Reorder");
                toggleModule("reorder", "Reorder");
            },
            toggleBurst: () => {
                console.log("Hotkey: Toggle Burst");
                toggleModule("burst", "Burst");
            },
        };

        registerAllHotkeys(handlers);

        return () => {
            unregisterAllHotkeys();
        };
    }, [registerAllHotkeys, unregisterAllHotkeys, toggleActive, toggleModule]);
}
