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
            toggleFreeze: () => {
                console.log("Hotkey: Toggle Freeze");
                toggleModule("drop", "Freeze");
            },
            toggleDelay: () => {
                console.log("Hotkey: Toggle Delay");
                toggleModule("delay", "Delay");
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
            toggleTamper: () => {
                console.log("Hotkey: Toggle Tamper");
                toggleModule("tamper", "Tamper");
            },
            toggleReorder: () => {
                console.log("Hotkey: Toggle Reorder");
                toggleModule("reorder", "Reorder");
            },
        };

        registerAllHotkeys(handlers);

        return () => {
            unregisterAllHotkeys();
        };
    }, [registerAllHotkeys, unregisterAllHotkeys, toggleActive, toggleModule]);
}
