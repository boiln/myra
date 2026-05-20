import { ThemeToggle } from "@/components/ui/theme-toggle";
import { useNetworkStore } from "@/lib/stores/network";
import { useActiveTimer } from "@/hooks/use-active-timer";
import { ModeSelector } from "@/components/mode-selector";
import { ManipulationMode } from "@/lib/stores/mode-store";

interface HeaderProps {
    mode: ManipulationMode;
    onModeChange: (mode: ManipulationMode) => void;
}

export function Header({ mode, onModeChange }: HeaderProps) {
    const { isActive, manipulationStatus } = useNetworkStore();
    const activeModules = manipulationStatus.modules.filter((m) => m.enabled);
    const showTimer = isActive && activeModules.length > 0;
    const { formattedTime } = useActiveTimer(showTimer);

    return (
        <header className="sticky top-0 z-10 border-b border-border/40 bg-background/80 backdrop-blur-md backdrop-saturate-150 transition-colors">
            <div className="container flex h-9 items-center justify-between px-2">
                <div className="flex items-center gap-3">
                    <h1 className="text-base font-semibold tracking-tight">Myra</h1>
                    <ModeSelector mode={mode} onModeChange={onModeChange} disabled={isActive} />
                </div>

                {/* Active Timer - centered */}
                {showTimer && (
                    <div className="absolute left-1/2 flex -translate-x-1/2 items-center gap-2">
                        <span className="font-mono text-lg font-semibold tabular-nums text-green-500">
                            {formattedTime}
                        </span>
                    </div>
                )}

                <ThemeToggle />
            </div>
        </header>
    );
}
