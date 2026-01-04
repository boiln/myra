import { useEffect, useRef, useCallback } from "react";
import { useTapStore } from "@/lib/stores/tap-store";
import { useNetworkStore } from "@/lib/stores/network";
import { ManipulationService } from "@/lib/services/manipulation";

/**
 * Hook that manages the "tap" feature - periodically disabling modules briefly
 * to create a pulsing/tapping effect for the lag.
 * 
 * When tap is enabled and manipulation is active:
 * - Every `intervalSeconds`, temporarily disable all enabled modules
 * - Keep them disabled for `durationSeconds`
 * - Then re-enable them
 * 
 * Uses updateSettings instead of startProcessing for lighter updates.
 */
export function useTap() {
    const { settings, isTapping, setIsTapping } = useTapStore();
    const { isActive, manipulationStatus, buildSettings } = useNetworkStore();
    
    const intervalRef = useRef<NodeJS.Timeout | null>(null);
    const tapTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const savedModulesRef = useRef<string[]>([]);  // Module names that were enabled before tap
    const isRunningRef = useRef(false);  // Prevent overlapping operations
    
    // Get list of currently enabled modules
    const getEnabledModules = useCallback(() => {
        return manipulationStatus.modules
            .filter((m) => m.enabled)
            .map((m) => m.name);
    }, [manipulationStatus.modules]);

    // Disable all enabled modules temporarily
    const startTap = useCallback(async () => {
        if (isRunningRef.current) return;
        
        const enabledModules = getEnabledModules();
        if (enabledModules.length === 0) return;

        isRunningRef.current = true;
        
        // Save which modules were enabled
        savedModulesRef.current = enabledModules;
        setIsTapping(true);

        try {
            // Build settings with all previously enabled modules now disabled
            const currentSettings = buildSettings();
            const disabledSettings = { ...currentSettings };

            // Disable all modules
            for (const moduleName of enabledModules) {
                const key = moduleName as keyof typeof disabledSettings;
                if (disabledSettings[key] && typeof disabledSettings[key] === 'object') {
                    (disabledSettings[key] as any).enabled = false;
                }
            }

            // Use updateSettings instead of startProcessing - much lighter operation
            await ManipulationService.updateSettings(disabledSettings, true);
        } catch (error) {
            console.error("Error starting tap:", error);
        } finally {
            isRunningRef.current = false;
        }
    }, [getEnabledModules, buildSettings, setIsTapping]);

    // Re-enable modules after tap duration
    const endTap = useCallback(async () => {
        const modulesToRestore = savedModulesRef.current;
        if (modulesToRestore.length === 0) {
            setIsTapping(false);
            return;
        }

        try {
            // Build settings with saved modules re-enabled
            const currentSettings = buildSettings();
            const restoredSettings = { ...currentSettings };

            // Re-enable the modules that were previously enabled
            for (const moduleName of modulesToRestore) {
                const key = moduleName as keyof typeof restoredSettings;
                if (restoredSettings[key] && typeof restoredSettings[key] === 'object') {
                    (restoredSettings[key] as any).enabled = true;
                }
            }

            // Use updateSettings instead of startProcessing - much lighter operation
            await ManipulationService.updateSettings(restoredSettings, true);
        } catch (error) {
            console.error("Error ending tap:", error);
        } finally {
            setIsTapping(false);
            savedModulesRef.current = [];
        }
    }, [buildSettings, setIsTapping]);

    // Main tap cycle
    const runTapCycle = useCallback(async () => {
        // Don't tap if already tapping or if an operation is running
        if (isTapping || isRunningRef.current) return;

        await startTap();

        // Schedule end of tap
        tapTimeoutRef.current = setTimeout(async () => {
            await endTap();
        }, settings.durationMs);
    }, [isTapping, startTap, endTap, settings.durationMs]);

    // Setup/cleanup interval
    useEffect(() => {
        // Clear existing intervals
        if (intervalRef.current) {
            clearInterval(intervalRef.current);
            intervalRef.current = null;
        }
        if (tapTimeoutRef.current) {
            clearTimeout(tapTimeoutRef.current);
            tapTimeoutRef.current = null;
        }

        // Only run if tap is enabled and manipulation is active
        const shouldRun = settings.enabled && isActive;
        
        if (!shouldRun) {
            // If we were in a tap, restore modules
            if (isTapping && savedModulesRef.current.length > 0) {
                endTap();
            }
            return;
        }

        // Check if any modules are enabled
        const enabledModules = getEnabledModules();
        if (enabledModules.length === 0) {
            return;
        }

        // Start the interval - run tap cycle every intervalMs
        intervalRef.current = setInterval(() => {
            runTapCycle();
        }, settings.intervalMs);

        // Cleanup on unmount or when dependencies change
        return () => {
            if (intervalRef.current) {
                clearInterval(intervalRef.current);
                intervalRef.current = null;
            }
            if (tapTimeoutRef.current) {
                clearTimeout(tapTimeoutRef.current);
                tapTimeoutRef.current = null;
            }
        };
    }, [
        settings.enabled,
        settings.intervalMs,
        settings.durationMs,
        isActive,
        isTapping,
        getEnabledModules,
        runTapCycle,
        endTap,
    ]);

    return {
        isTapping,
        settings,
    };
}
