import { Card, CardContent } from "@/components/ui/card";
import { ClassicModuleRow } from "@/components/classic-module-row";
import { useClassicStore } from "@/lib/stores/classic-store";
import { ClassicModuleInfo } from "@/types/classic";
import { Loader2 } from "lucide-react";

export function ClassicModulePanel() {

    const { modules, isLoading, updateModuleConfig, toggleModule, toggleDirection } =

        useClassicStore();

    const handleModuleToggle = async (module: ClassicModuleInfo) => {
        try {
            await toggleModule(module.name);
        } catch (error) {
            console.error("Error toggling classic module:", error);
        }
    };

    const handleDirectionToggle = async (

        module: ClassicModuleInfo,
        direction: "inbound" | "outbound"
    ) => {
        try {
            await toggleDirection(module.name, direction);
        } catch (error) {
            console.error("Error toggling direction:", error);
        }
    };

    const handleSettingChange = async (

        module: ClassicModuleInfo,
        setting: string,
        value: number | boolean
    ) => {
        try {
            await updateModuleConfig(module.name, { [setting]: value });
        } catch (error) {
            console.error("Error updating classic setting:", error);
        }
    };

    return (

        <div className="relative z-10 flex flex-col">
            <Card className="border-border bg-card/90">
                <CardContent className="bg-card/90 px-3 py-2">
                    {isLoading ? (
                        <div className="flex h-32 items-center justify-center">
                            <Loader2 className="size-6 animate-spin text-primary/70" />
                        </div>
                    ) : (
                        <div className="flex flex-col">
                            {modules.map((module) => (
                                <ClassicModuleRow
                                    key={module.name}
                                    module={module}
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
