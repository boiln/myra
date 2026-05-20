import React, { ChangeEvent } from "react";
import { ClassicModuleInfo, ClassicModuleName } from "@/types/classic";
import { MyraCheckbox } from "@/components/ui/myra-checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Info } from "lucide-react";

interface ClassicModuleRowProps {
    module: ClassicModuleInfo;
    onModuleToggle: (module: ClassicModuleInfo) => Promise<void>;
    onDirectionToggle: (
        module: ClassicModuleInfo,
        direction: "inbound" | "outbound"
    ) => Promise<void>;
    onSettingChange: (module: ClassicModuleInfo, setting: string, value: number | boolean) => void;
}

// Module-specific parameter configurations
const MODULE_PARAMS: Record<
    ClassicModuleName,
    Array<{
        key: string;
        label: string;
        min: number;
        max: number;
        isInteger?: boolean;
        tooltip?: string;
    }>
> = {
    classic_latency: [
        {
            key: "delay_ms",
            label: "Delay(ms)",
            min: 0,
            max: 15000,
            isInteger: true,
            tooltip: "Fixed delay before packet release",
        },
    ],
    classic_drop: [],
    classic_throttle: [
        {
            key: "window_ms",
            label: "Window(ms)",
            min: 0,
            max: 1000,
            isInteger: true,
            tooltip: "Time window for buffering packets",
        },
    ],
    classic_reorder: [
        {
            key: "max_hold_cycles",
            label: "Hold Cycles",
            min: 1,
            max: 100,
            isInteger: true,
            tooltip: "Max cycles to hold a single packet",
        },
    ],
    classic_tamper: [],
    classic_bandwidth: [
        {
            key: "limit_kbps",
            label: "Limit(KB/s)",
            min: 0.1,
            max: 10000,
            isInteger: false,
            tooltip: "Bandwidth limit in kilobytes per second",
        },
    ],
};

// Boolean toggles for specific modules
const MODULE_TOGGLES: Record<
    ClassicModuleName,
    Array<{ key: string; label: string; tooltip?: string }>
> = {
    classic_latency: [],
    classic_drop: [],
    classic_throttle: [
        {
            key: "drop_on_release",
            label: "Drop",
            tooltip: "Drop buffered packets instead of releasing them",
        },
    ],
    classic_reorder: [],
    classic_tamper: [
        {
            key: "recalc_checksum",
            label: "Fix Checksum",
            tooltip: "Recalculate checksums after tampering",
        },
    ],
    classic_bandwidth: [],
};

export function ClassicModuleRow({
    module,
    onModuleToggle,
    onDirectionToggle,
    onSettingChange,
}: ClassicModuleRowProps) {
    const [inputValues, setInputValues] = React.useState<Record<string, string>>({});

    const params = MODULE_PARAMS[module.name] || [];
    const toggles = MODULE_TOGGLES[module.name] || [];

    const handleInputChange = (
        e: ChangeEvent<HTMLInputElement>,
        setting: string,
        _min: number,
        _max: number,
        isInteger = false
    ) => {
        const input = e.target.value;
        setInputValues((prev) => ({ ...prev, [setting]: input }));

        if (input === "") return;
        if (!/^-?\d*\.?\d*$/.test(input)) return;
        if (input === "." || input === "-" || input === "-.") return;

        const parsed = isInteger ? parseInt(input, 10) : parseFloat(input);
        if (isNaN(parsed)) return;

        onSettingChange(module, setting, parsed);
    };

    const handleInputBlur = (setting: string, min: number, max: number, isInteger = false) => {
        const input = inputValues[setting];

        if (input === undefined || input === "") {
            setInputValues((prev) => ({ ...prev, [setting]: min.toString() }));
            onSettingChange(module, setting, min);
            return;
        }

        const parsed = isInteger ? parseInt(input, 10) : parseFloat(input);

        if (isNaN(parsed)) {
            setInputValues((prev) => ({ ...prev, [setting]: min.toString() }));
            onSettingChange(module, setting, min);
            return;
        }

        let clamped = parsed;
        if (parsed < min) clamped = min;
        if (parsed > max) clamped = max;

        if (clamped !== parsed) {
            setInputValues((prev) => ({ ...prev, [setting]: clamped.toString() }));
            onSettingChange(module, setting, clamped);
        }
    };

    const getDisplayValue = (setting: string) => {
        if (setting in inputValues) return inputValues[setting];
        const value = module.config[setting as keyof typeof module.config];
        if (value !== undefined && value !== null) return value.toString();
        return "";
    };

    return (
        <div className="flex items-center gap-x-3 py-2 first:pt-0.5 last:pb-0.5">
            {/* Module Enable Checkbox */}
            <div className="flex shrink-0 items-center gap-1.5">
                <MyraCheckbox
                    id={`${module.name}-enable`}
                    checked={module.enabled}
                    onCheckedChange={() => onModuleToggle(module)}
                    label={module.display_name}
                    labelClassName="text-sm font-medium text-foreground w-[80px]"
                />
                <Tooltip>
                    <TooltipTrigger asChild>
                        <Info className="size-3.5 text-muted-foreground/60 hover:text-muted-foreground" />
                    </TooltipTrigger>
                    <TooltipContent side="top" className="max-w-xs">
                        <p className="text-xs">{module.description}</p>
                    </TooltipContent>
                </Tooltip>
            </div>

            {/* Module-specific numeric inputs */}
            <div className="flex shrink-0 items-center gap-2">
                {params.map((param) => (
                    <div key={param.key} className="flex items-center gap-1">
                        <Label
                            htmlFor={`${module.name}-${param.key}`}
                            className="whitespace-nowrap text-xs text-foreground/70"
                        >
                            {param.label}:
                        </Label>
                        <Input
                            id={`${module.name}-${param.key}`}
                            value={getDisplayValue(param.key)}
                            onChange={(e) =>
                                handleInputChange(
                                    e,
                                    param.key,
                                    param.min,
                                    param.max,
                                    param.isInteger
                                )
                            }
                            onBlur={() =>
                                handleInputBlur(param.key, param.min, param.max, param.isInteger)
                            }
                            className="h-6 w-[55px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                            disabled={!module.enabled}
                            type="text"
                            inputMode={param.isInteger ? "numeric" : "decimal"}
                        />
                    </div>
                ))}

                {/* Module-specific boolean toggles */}
                {toggles.map((toggle) => (
                    <Tooltip key={toggle.key}>
                        <TooltipTrigger asChild>
                            <div>
                                <MyraCheckbox
                                    id={`${module.name}-${toggle.key}`}
                                    checked={
                                        (module.config[
                                            toggle.key as keyof typeof module.config
                                        ] as boolean) ?? false
                                    }
                                    onCheckedChange={(checked) =>
                                        onSettingChange(module, toggle.key, checked === true)
                                    }
                                    disabled={!module.enabled}
                                    label={toggle.label}
                                    labelClassName={`text-xs text-foreground ${!module.enabled ? "opacity-50" : ""}`}
                                />
                            </div>
                        </TooltipTrigger>
                        {toggle.tooltip && (
                            <TooltipContent side="top">
                                <p className="text-xs">{toggle.tooltip}</p>
                            </TooltipContent>
                        )}
                    </Tooltip>
                ))}
            </div>

            {/* Spacer */}
            <div className="flex-1" />

            {/* Direction Controls */}
            <div className="flex items-center gap-2">
                <MyraCheckbox
                    id={`${module.name}-inbound`}
                    checked={module.config.inbound}
                    onCheckedChange={() => onDirectionToggle(module, "inbound")}
                    disabled={!module.enabled}
                    label="In"
                    labelClassName={`text-xs text-foreground ${!module.enabled ? "opacity-50" : ""}`}
                />
                <MyraCheckbox
                    id={`${module.name}-outbound`}
                    checked={module.config.outbound}
                    onCheckedChange={() => onDirectionToggle(module, "outbound")}
                    disabled={!module.enabled}
                    label="Out"
                    labelClassName={`text-xs text-foreground ${!module.enabled ? "opacity-50" : ""}`}
                />
            </div>

            {/* Chance */}
            <div className="flex items-center gap-1">
                <Label
                    htmlFor={`${module.name}-chance`}
                    className="whitespace-nowrap text-xs text-foreground/70"
                >
                    %:
                </Label>
                <Input
                    id={`${module.name}-chance`}
                    value={getDisplayValue("chance")}
                    onChange={(e) => handleInputChange(e, "chance", 0, 100)}
                    onBlur={() => handleInputBlur("chance", 0, 100)}
                    className="h-6 w-[40px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                    disabled={!module.enabled}
                    type="text"
                    inputMode="decimal"
                />
            </div>
        </div>
    );
}
