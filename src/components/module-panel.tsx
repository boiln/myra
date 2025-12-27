import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useNetworkStore } from "@/lib/stores/network";
import { ModuleRow } from "@/components/module-row";
import { useDebounce } from "@/hooks/use-debounce";
import { ModuleInfo } from "@/types";
import { useEffect } from "react";
import { Loader2 } from "lucide-react";

export function ModulePanel() {
    const {
        isActive,
        manipulationStatus,
        updateModuleSettings,
        toggleDirection,
        applyModuleSettings,
        loadStatus,
    } = useNetworkStore();

    const modules = manipulationStatus.modules;

    useEffect(() => {
        loadStatus();
    }, [loadStatus]);

    const debouncedSettingChange = useDebounce(
        async (module: ModuleInfo, setting: string, value: number) => {
            try {
                const newConfig = { ...module.config, [setting]: value };
                await updateModuleSettings(module.name || "", newConfig);
            } catch (error) {
                console.error("Error updating setting:", error);
            }
        },
        300
    );

    const handleModuleToggle = async (module: ModuleInfo) => {
        try {
            await applyModuleSettings(module.name || "", !module.enabled);
        } catch (error) {
            console.error("Error toggling module:", error);
        }
    };

    const handleSettingChange = (module: ModuleInfo, setting: string, value: number) => {
        debouncedSettingChange(module, setting, value);
    };

    const handleDirectionToggle = async (module: ModuleInfo, direction: "inbound" | "outbound") => {
        try {
            await toggleDirection(module.name || "", direction);
        } catch (error) {
            console.error("Error updating direction:", error);
        }
    };

    return (
        <div className="relative z-10 flex flex-col">
            <Card className="border-border bg-card/90">
                <CardHeader className="rounded-t-lg bg-card/90 pb-2">
                    <div className="flex">
                        <CardTitle className="text-lg text-foreground">Modules</CardTitle>
                    </div>
                </CardHeader>
                <CardContent className="bg-card/90 px-3 py-2">
                    {modules.length === 0 ? (
                        <div className="flex h-32 items-center justify-center">
                            <Loader2 className="h-6 w-6 animate-spin text-primary/70" />
                        </div>
                    ) : (
                        <div className="flex flex-col">
                            {modules.map((module) => (
                                <ModuleRow
                                    key={module.name}
                                    module={module}
                                    isActive={isActive}
                                    onModuleToggle={handleModuleToggle}
                                    onDirectionToggle={handleDirectionToggle}
                                    onSettingChange={handleSettingChange}
                                />
                            ))}
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}
