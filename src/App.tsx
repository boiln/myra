import { useEffect } from "react";
import { LazyMotion, domAnimation } from "framer-motion";
import { Header } from "@/components/header";
import { ModulePanel } from "@/components/module-panel";
import { ClassicModulePanel } from "@/components/classic-module-panel";
import { NetworkControls } from "@/components/network-controls";
import { TapControl } from "@/components/tap-control";
import { StatusBar } from "@/components/status-bar";
import { useNetworkStore } from "@/lib/stores/network";
import { useClassicStore } from "@/lib/stores/classic-store";
import { useModeStore } from "@/lib/stores/mode-store";
import { ToastProvider } from "@/components/providers/toast-provider";
import { PresetManager } from "@/components/presets/preset-manager";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useHotkeys } from "@/hooks/use-hotkeys";
import { useTap } from "@/hooks/use-tap";
import { useClassicTap } from "@/hooks/use-classic-tap";

function App() {
    const mode = useModeStore((state) => state.mode);
    const setMode = useModeStore((state) => state.setMode);

    const loadStatus = useNetworkStore((state) => state.loadStatus);
    const loadPresets = useNetworkStore((state) => state.loadPresets);
    const initializeDefaultPreset = useNetworkStore((state) => state.initializeDefaultPreset);

    const initializeClassic = useClassicStore((state) => state.initialize);

    // Initialize global hotkeys
    useHotkeys();

    // Initialize tap feature for both modes
    useTap();
    useClassicTap();

    useEffect(() => {
        const initialize = async () => {
            await loadStatus();
            await initializeDefaultPreset();
            initializeClassic();
        };

        initialize();

        return () => {
            // Cleanup function
        };
    }, [loadStatus, loadPresets, initializeDefaultPreset, initializeClassic]);

    return (
        <LazyMotion features={domAnimation}>
            <TooltipProvider>
                <div className="flex h-screen min-h-[520px] min-w-[800px] flex-col bg-muted/30">
                    <Header mode={mode} onModeChange={setMode} />
                    <main className="flex-1 overflow-y-auto px-2 py-1 pb-8">
                        <div className="mx-auto flex w-full flex-col gap-2 xl:max-w-7xl">
                            <div className="flex flex-col gap-1.5">
                                <PresetManager />
                                <NetworkControls />
                            </div>
                            <TapControl />
                            {mode === "standard" ? <ModulePanel /> : <ClassicModulePanel />}
                        </div>
                    </main>
                    <StatusBar />
                    <ToastProvider />
                </div>
            </TooltipProvider>
        </LazyMotion>
    );
}

export default App;
