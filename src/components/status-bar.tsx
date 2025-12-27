import { Database, FileText, Monitor, Gamepad2, Globe, Code } from "lucide-react";
import { useNetworkStore } from "@/lib/stores/network";
import { FilterTarget } from "@/types";

const StatusIndicator = ({ isActive }: { isActive: boolean }) => (
    <div className="flex items-center space-x-1.5">
        <span className={isActive ? "text-green-500" : "text-foreground/70"}>
            {isActive ? "Filtering" : "Ready"}
        </span>
    </div>
);

const TargetIndicator = ({ target }: { target: FilterTarget | null }) => {
    if (!target) return null;

    const getTargetIcon = () => {
        switch (target.mode) {
            case "all":
                return <Globe className="h-3 w-3 text-muted-foreground" />;
            case "process":
                return <Monitor className="h-3 w-3 text-muted-foreground" />;
            case "device":
                return <Gamepad2 className="h-3 w-3 text-muted-foreground" />;
            case "custom":
                return <Code className="h-3 w-3 text-muted-foreground" />;
        }
    };

    const getTargetLabel = () => {
        switch (target.mode) {
            case "all":
                return "All";
            case "process":
                return target.processName || `PID ${target.processId}`;
            case "device":
                return target.deviceName
                    ? `${target.deviceIp} (${target.deviceName})`
                    : target.deviceIp;
            case "custom":
                return "Custom";
        }
    };

    return (
        <div className="flex items-center space-x-1.5">
            {getTargetIcon()}
            <span className="text-muted-foreground">Target:</span>
            <span className="text-foreground/70">{getTargetLabel()}</span>
        </div>
    );
};

const PresetIndicator = ({ preset }: { preset: string | null }) => {
    if (!preset) return null;

    return (
        <div className="flex items-center space-x-1.5">
            <FileText className="h-3 w-3 text-muted-foreground" />
            <span className="text-muted-foreground">Preset:</span>
            <span className="text-foreground/70">{preset}</span>
        </div>
    );
};

const ModulesIndicator = ({ activeModules }: { activeModules: { display_name: string }[] }) => {
    if (activeModules.length === 0) return null;

    return (
        <div className="flex items-center space-x-1.5">
            <Database className="h-3 w-3 text-muted-foreground" />
            <span className="text-muted-foreground">Modules:</span>
            <span className="text-foreground/70">
                {activeModules.map((m) => m.display_name).join(", ")}
            </span>
        </div>
    );
};

export function StatusBar() {
    const { isActive, manipulationStatus, currentPreset, filterTarget } = useNetworkStore();
    const modules = manipulationStatus.modules;
    const activeModules = modules.filter((m) => m.enabled);

    return (
        <div className="fixed bottom-0 left-0 right-0 z-50 flex h-6 items-center justify-between border-t border-border/30 bg-background/60 px-3 text-xs backdrop-blur-md backdrop-saturate-150">
            <div className="flex items-center space-x-3">
                <StatusIndicator isActive={isActive} />
                <TargetIndicator target={filterTarget} />
                <PresetIndicator preset={currentPreset} />
                <ModulesIndicator activeModules={activeModules} />
            </div>
        </div>
    );
}
