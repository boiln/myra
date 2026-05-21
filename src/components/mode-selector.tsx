import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { ManipulationMode } from "@/lib/stores/mode-store";

interface ModeSelectorProps {
    mode: ManipulationMode;
    onModeChange: (mode: ManipulationMode) => void;
    disabled?: boolean;
}

export function ModeSelector({ mode, onModeChange, disabled }: ModeSelectorProps) {

    return (
        <Tabs
            value={mode}
            onValueChange={(v) => onModeChange(v as ManipulationMode)}
            className="w-auto"
        >
            <TabsList className="h-7 bg-muted/60">
                <Tooltip>
                    <TooltipTrigger asChild>
                        <TabsTrigger
                            value="standard"
                            disabled={disabled}
                            className="h-5 px-2.5 text-xs data-[state=active]:bg-background"
                        >
                            Standard
                        </TabsTrigger>
                    </TooltipTrigger>
                    <TooltipContent side="bottom">
                        <p className="text-xs">
                            Per-packet probabilistic manipulation with duration controls
                        </p>
                    </TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <TabsTrigger
                            value="classic"
                            disabled={disabled}
                            className="h-5 px-2.5 text-xs data-[state=active]:bg-background"
                        >
                            Classic
                        </TabsTrigger>
                    </TooltipTrigger>
                    <TooltipContent side="bottom">
                        <p className="text-xs">
                            Timer-based deterministic manipulation with fixed windows
                        </p>
                    </TooltipContent>
                </Tooltip>
            </TabsList>
        </Tabs>
    );

}
