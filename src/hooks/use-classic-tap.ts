import { useEffect, useRef } from "react";
import { useTapStore } from "@/lib/stores/tap-store";
import { useClassicStore } from "@/lib/stores/classic-store";
import { ClassicModuleName } from "@/types/classic";

/**
 * Hook that manages the "tap" feature for Classic mode.
 * Periodically disables Classic modules briefly to create a pulsing effect.
 *
 * When tap is enabled and Classic processing is active:
 * - Every `intervalMs`, temporarily disable all enabled Classic modules
 * - Keep them disabled for `durationMs`
 * - Then re-enable them
 */
export function useClassicTap() {

    const { settings, isTapping, setIsTapping } = useTapStore();
    const { isProcessing, toggleModule } = useClassicStore();

    const intervalRef = useRef<NodeJS.Timeout | null>(null);
    const tapTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const savedModulesRef = useRef<ClassicModuleName[]>([]);
    const isTappingRef = useRef(false); // Use ref to avoid effect re-runs
    const lastTapEndedAtRef = useRef<number>(0);

    // Keep ref in sync with state
    useEffect(() => {
        isTappingRef.current = isTapping;
    }, [isTapping]);

    // Effect to manage the tap interval for Classic mode
    // Uses refs internally to track tap state without causing effect re-runs
    useEffect(() => {

        const shouldRun = settings.enabled && isProcessing;
        console.log(
            "[ClassicTap] Effect: shouldRun=",
            shouldRun,
            "enabled=",
            settings.enabled,
            "isProcessing=",
            isProcessing
        );
        // Helper to get enabled modules from store
        const getEnabledModules = (): ClassicModuleName[] => {

            const names: ClassicModuleName[] = [];
            const modules = useClassicStore.getState().modules;

            for (const m of modules) {
                if (m.enabled) names.push(m.name);
            }

            return names;

        };
        // Restore modules helper
        const restoreModules = async () => {

            const modulesToRestore = savedModulesRef.current;

            if (modulesToRestore.length === 0) return;
            console.log("[ClassicTap] Restoring modules:", modulesToRestore);

            try {
                await Promise.all(modulesToRestore.map((moduleName) => toggleModule(moduleName)));
            } catch (error) {
                console.error("Error restoring modules:", error);
            } finally {
                isTappingRef.current = false;
                setIsTapping(false);
                savedModulesRef.current = [];
            }

        };

        if (!shouldRun) {
            // Clean up
            if (intervalRef.current) {
                clearInterval(intervalRef.current);
                intervalRef.current = null;
            }
            if (tapTimeoutRef.current) {
                clearTimeout(tapTimeoutRef.current);
                tapTimeoutRef.current = null;
            }
            // Restore modules if we were in a tap when disabled
            if (savedModulesRef.current.length > 0) {
                restoreModules();
            }
            return;
        }
        // Don't restart interval if it's already running with same settings
        if (intervalRef.current) {
            console.log("[ClassicTap] Effect: Interval already running");
            return;
        }
        const { durationMs, cooldownMs = 1200, intervalMs } = settings;
        const runTapCycle = async () => {

            // Don't start if we're already tapping
            if (isTappingRef.current) {
                console.log("[ClassicTap] runTapCycle: Already tapping, skipping");
                return;
            }
            // Check cooldown
            const timeSinceLastTap = Date.now() - lastTapEndedAtRef.current;

            if (timeSinceLastTap < cooldownMs) {
                console.log("[ClassicTap] runTapCycle: In cooldown, skipping");
                return;
            }
            // Check if we have enabled modules
            const enabledModules = getEnabledModules();

            if (enabledModules.length === 0) {
                console.log("[ClassicTap] runTapCycle: No enabled modules, skipping");
                return;
            }
            console.log(
                "[ClassicTap] runTapCycle: Starting tap - disabling modules:",
                enabledModules
            );
            // Start tap (disable modules)
            savedModulesRef.current = enabledModules;
            isTappingRef.current = true;
            setIsTapping(true);

            try {
                await Promise.all(enabledModules.map((moduleName) => toggleModule(moduleName)));
                console.log("[ClassicTap] runTapCycle: Modules disabled successfully");
            } catch (error) {
                console.error("Error disabling modules:", error);
                isTappingRef.current = false;
                setIsTapping(false);
                savedModulesRef.current = [];
                return;
            }
            // Schedule end of tap
            tapTimeoutRef.current = setTimeout(async () => {

                const modulesToRestore = [...savedModulesRef.current];
                console.log("[ClassicTap] Tap duration ended, re-enabling:", modulesToRestore);

                if (modulesToRestore.length === 0) {
                    isTappingRef.current = false;
                    setIsTapping(false);
                    return;
                }

                try {
                    await Promise.all(
                        modulesToRestore.map((moduleName) => toggleModule(moduleName))
                    );
                    console.log("[ClassicTap] Modules re-enabled successfully");
                } catch (error) {
                    console.error("Error re-enabling modules:", error);
                } finally {
                    isTappingRef.current = false;
                    setIsTapping(false);
                    savedModulesRef.current = [];
                    lastTapEndedAtRef.current = Date.now();
                    tapTimeoutRef.current = null;
                }

            }, durationMs);

        };
        console.log(
            "[ClassicTap] Effect: Starting interval with intervalMs=",
            intervalMs,
            "durationMs=",
            durationMs
        );
        intervalRef.current = setInterval(runTapCycle, intervalMs);
        // Run first cycle after a brief delay to let things settle
        setTimeout(runTapCycle, 100);

        return () => {

            console.log("[ClassicTap] Effect cleanup");

            if (intervalRef.current) {
                clearInterval(intervalRef.current);
                intervalRef.current = null;
            }

            if (tapTimeoutRef.current) {
                clearTimeout(tapTimeoutRef.current);
                tapTimeoutRef.current = null;
            }

        };
        // Note: toggleModule and setIsTapping are stable references from zustand stores
        // eslint-disable-next-line react-hooks/exhaustive-deps

    }, [
        settings.enabled,
        settings.intervalMs,
        settings.durationMs,
        settings.cooldownMs,
        isProcessing,
    ]);

    // Cleanup on unmount
    useEffect(() => {

        return () => {

            savedModulesRef.current = [];

            if (intervalRef.current) clearInterval(intervalRef.current);

            if (tapTimeoutRef.current) clearTimeout(tapTimeoutRef.current);

        };

    }, []);

    return {
        isTapping,
        settings,
    };

}
