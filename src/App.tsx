import { useEffect } from "react";
import { Header } from "@/components/header";
import { ModulePanel } from "@/components/module-panel";
import { NetworkControls } from "@/components/network-controls";
import { StatusBar } from "@/components/status-bar";
import { useNetworkStore } from "@/lib/stores/network";
import { ToastProvider } from "@/components/providers/toast-provider";
import { PresetManager } from "@/components/presets/preset-manager";
import { TooltipProvider } from "@/components/ui/tooltip";

function App() {
    const loadStatus = useNetworkStore((state) => state.loadStatus);
    const loadPresets = useNetworkStore((state) => state.loadPresets);
    const initializeDefaultPreset = useNetworkStore((state) => state.initializeDefaultPreset);

    useEffect(() => {
        const initialize = async () => {
            await loadStatus();
            await initializeDefaultPreset();
        };

        initialize();

        return () => {
            // Cleanup function
        };
    }, [loadStatus, loadPresets, initializeDefaultPreset]);

    return (
        <TooltipProvider>
            <div className="flex min-h-[520px] min-w-[800px] flex-col bg-muted/30">
                <Header />
                <main className="flex-1 overflow-y-auto px-2 py-1">
                    <div className="mx-auto flex w-full flex-col space-y-2 xl:max-w-7xl">
                        <div className="space-y-1.5">
                            <PresetManager />
                            <NetworkControls />
                        </div>
                        <ModulePanel />
                    </div>
                </main>
                <StatusBar />
                <ToastProvider />
            </div>
        </TooltipProvider>
    );
}

export default App;
