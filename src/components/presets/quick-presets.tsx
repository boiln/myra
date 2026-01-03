import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useNetworkStore } from "@/lib/stores/network";
import { PacketManipulationSettings } from "@/types";
import { Zap } from "lucide-react";
import { toast } from "sonner";

interface QuickPreset {
    name: string;
    description: string;
    settings: PacketManipulationSettings;
    filter?: string;
}

const QUICK_PRESETS: QuickPreset[] = [
    {
        name: "Network Throttle",
        description: "300ms inbound throttle with freeze mode",
        settings: {
            throttle: {
                enabled: true,
                inbound: true,
                outbound: false,
                probability: 1,
                duration_ms: 0,
                throttle_ms: 300,
                drop: false,
                freeze_mode: true,
            },
        },
        filter: "inbound",
    },
];

export function QuickPresets() {
    const { manipulationStatus, loadStatus, isActive } = useNetworkStore();

    const applyQuickPreset = async (preset: QuickPreset) => {
        try {
            const { ManipulationService } = await import("@/lib/services/manipulation");

            const currentModules = manipulationStatus.modules;
            const newSettings: PacketManipulationSettings = {};

            // Disable all modules by default
            currentModules.forEach((module) => {
                switch (module.name) {
                    case "lag":
                        newSettings.lag = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            lag_ms: 1000,
                            duration_ms: 0,
                        };
                        break;
                    case "drop":
                        newSettings.drop = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            duration_ms: 0,
                        };
                        break;
                    case "throttle":
                        newSettings.throttle = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            duration_ms: 0,
                            throttle_ms: 300,
                        };
                        break;
                    case "duplicate":
                        newSettings.duplicate = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            count: 2,
                            duration_ms: 0,
                        };
                        break;
                    case "bandwidth":
                        newSettings.bandwidth = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            limit: 500,
                            duration_ms: 0,
                        };
                        break;
                    case "tamper":
                        newSettings.tamper = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            duration_ms: 0,
                        };
                        break;
                    case "reorder":
                        newSettings.reorder = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            duration_ms: 0,
                            max_delay: 100,
                        };
                        break;
                    case "burst":
                        newSettings.burst = {
                            enabled: false,
                            inbound: true,
                            outbound: true,
                            probability: 1,
                            buffer_ms: 0,
                            duration_ms: 0,
                            keepalive_ms: 0,
                            release_delay_us: 500,
                        };
                        break;
                }
            });

            // Apply preset settings on top
            Object.assign(newSettings, preset.settings);

            await ManipulationService.updateSettings(newSettings, isActive);
            
            // Apply filter if specified
            if (preset.filter) {
                await ManipulationService.updateFilter(preset.filter);
            }
            
            await loadStatus();

            toast.success(`Applied "${preset.name}"`, { dismissible: true });
        } catch (error) {
            toast.error(`Failed to apply preset: ${error}`, { dismissible: true });
        }
    };

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="outline"
                    size="sm"
                    className="h-7 gap-1.5 px-2"
                >
                    <Zap className="h-3.5 w-3.5" />
                    Quick
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="w-56">
                {QUICK_PRESETS.map((preset) => (
                    <DropdownMenuItem
                        key={preset.name}
                        onClick={() => applyQuickPreset(preset)}
                        className="flex flex-col items-start gap-0.5 py-2 focus:text-foreground"
                    >
                        <span className="font-medium">{preset.name}</span>
                        <span className="text-xs opacity-70">
                            {preset.description}
                        </span>
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
