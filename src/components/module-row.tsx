import React, { ChangeEvent } from "react";
import { ModuleInfo } from "@/types";
import { MyraCheckbox } from "@/components/ui/myra-checkbox";
import { Input } from "@/components/ui/input";
import { InfinityInput } from "@/components/ui/infinity-input";
import { Label } from "@/components/ui/label";
import { HotkeyBadge } from "@/components/hotkey-badge";

// Map module names to hotkey actions
const MODULE_HOTKEY_ACTIONS: Record<string, string> = {
    drop: "toggleDrop",
    lag: "toggleLag",
    throttle: "toggleThrottle",
    duplicate: "toggleDuplicate",
    bandwidth: "toggleBandwidth",
    corruption: "toggleCorruption",
    reorder: "toggleReorder",
    burst: "toggleBurst",
};

interface ModuleRowProps {
    module: ModuleInfo;
    isActive: boolean;
    onModuleToggle: (module: ModuleInfo) => Promise<void>;
    onDirectionToggle: (module: ModuleInfo, direction: "inbound" | "outbound") => Promise<void>;
    onSettingChange: (module: ModuleInfo, setting: string, value: number) => void;
    onBooleanSettingChange?: (module: ModuleInfo, setting: string, value: boolean) => void;
}

export function ModuleRow({
    module,
    isActive,
    onModuleToggle,
    onDirectionToggle,
    onSettingChange,
    onBooleanSettingChange,
}: ModuleRowProps) {
    const [inputValues, setInputValues] = React.useState<Record<string, string>>({});

    // Handle input change - allow any valid number format during typing
    const handleInputChange = (
        e: ChangeEvent<HTMLInputElement>,
        setting: string,
        min: number,
        max: number,
        isInteger = false
    ) => {
        const input = e.target.value;

        // Always update the display value
        setInputValues((prev) => ({ ...prev, [setting]: input }));

        // Allow empty input
        if (input === "") {
            return;
        }

        // Allow partial number input during typing (like "0.", ".", "-", etc.)
        if (!/^-?\d*\.?\d*$/.test(input)) return;
        if (input === "." || input === "-" || input === "-." || input.endsWith(".")) return;

        const parsed = isInteger ? parseInt(input, 10) : parseFloat(input);

        if (isNaN(parsed)) return;

        // Don't clamp during typing - just update with parsed value
        // Clamping happens on blur
        onSettingChange(module, setting, parsed);
    };

    // Handle blur - clamp value to valid range
    const handleInputBlur = (setting: string, min: number, max: number, isInteger = false) => {
        const input = inputValues[setting];

        if (input === undefined || input === "") {
            // Reset to min if empty
            setInputValues((prev) => ({ ...prev, [setting]: min.toString() }));
            onSettingChange(module, setting, min);
            return;
        }

        const parsed = isInteger ? parseInt(input, 10) : parseFloat(input);

        if (isNaN(parsed)) {
            // Reset to min if invalid
            setInputValues((prev) => ({ ...prev, [setting]: min.toString() }));
            onSettingChange(module, setting, min);
            return;
        }

        // Clamp to valid range
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

        const defaults: Record<string, string> = {
            chance: "100",
            duration_ms: module.name === "lag" ? "1000" : "0",
            throttle_ms: module.name === "throttle" ? "30" : "100",
            count: "2",
            limit_kbps: "500",
            buffer_ms: "0",
            keepalive_ms: "0",
            release_delay_us: "500",
        };

        return defaults[setting] ?? "";
    };

    return (
        <div key={module.name} className="flex items-center gap-x-3 py-2 first:pt-0.5 last:pb-0.5">
            {/* Active Indicator */}
            <span
                className={`h-2 w-2 shrink-0 rounded-full ${
                    module.enabled && isActive ? "animate-pulse bg-green-500" : "bg-transparent"
                }`}
                title={module.enabled && isActive ? "This module is active" : ""}
            ></span>

            {/* Module Enable Checkbox */}
            <div className="flex shrink-0 items-center gap-1.5">
                <MyraCheckbox
                    id={`${module.name}-enable`}
                    checked={module.enabled}
                    onCheckedChange={() => onModuleToggle(module)}
                    label={module.display_name}
                    labelClassName="text-sm font-medium text-foreground w-[70px]"
                />
                {MODULE_HOTKEY_ACTIONS[module.name] && (
                    <HotkeyBadge action={MODULE_HOTKEY_ACTIONS[module.name]} />
                )}
            </div>

            {/* Module-specific inputs - right after module name */}
            <div className="flex shrink-0 items-center gap-2">
                {module.name === "lag" && (
                    <div className="flex items-center gap-1">
                        <Label
                            htmlFor={`${module.name}-time`}
                            className="whitespace-nowrap text-xs text-foreground/70"
                        >
                            Time(ms):
                        </Label>
                        <Input
                            id={`${module.name}-time`}
                            value={getDisplayValue("duration_ms")}
                            onChange={(e) => handleInputChange(e, "duration_ms", 0, 999999)}
                            onBlur={() => handleInputBlur("duration_ms", 0, 999999)}
                            className="h-6 w-[50px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                            disabled={!module.enabled}
                            type="text"
                            inputMode="numeric"
                        />
                    </div>
                )}
                {module.name === "throttle" && (
                    <>
                        <div className="flex items-center gap-1">
                            <Label
                                htmlFor={`${module.name}-throttle-time`}
                                className="whitespace-nowrap text-xs text-foreground/70"
                            >
                                Time(ms):
                            </Label>
                            <Input
                                id={`${module.name}-throttle-time`}
                                value={getDisplayValue("throttle_ms")}
                                onChange={(e) =>
                                    handleInputChange(e, "throttle_ms", 1, 60000, true)
                                }
                                onBlur={() => handleInputBlur("throttle_ms", 1, 60000, true)}
                                className="h-6 w-[50px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                                disabled={!module.enabled}
                                type="text"
                                inputMode="numeric"
                            />
                        </div>
                        <MyraCheckbox
                            id={`${module.name}-freeze-mode`}
                            checked={module.config.freeze_mode ?? false}
                            onCheckedChange={(checked) =>
                                onBooleanSettingChange?.(module, "freeze_mode", checked === true)
                            }
                            disabled={!module.enabled}
                            label="Freeze"
                            labelClassName={`text-xs text-foreground ${!module.enabled ? "opacity-50" : ""}`}
                        />
                    </>
                )}
                {module.name === "duplicate" && (
                    <div className="flex items-center gap-1">
                        <Label
                            htmlFor={`${module.name}-count`}
                            className="whitespace-nowrap text-xs text-foreground/70"
                        >
                            Count:
                        </Label>
                        <Input
                            id={`${module.name}-count`}
                            value={getDisplayValue("count")}
                            onChange={(e) => handleInputChange(e, "count", 1, 9999, true)}
                            onBlur={() => handleInputBlur("count", 1, 9999, true)}
                            className="h-6 w-[50px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                            disabled={!module.enabled}
                            type="text"
                            inputMode="numeric"
                        />
                    </div>
                )}
                {module.name === "bandwidth" && (
                    <>
                        <div className="flex items-center gap-1">
                            <Label
                                htmlFor={`${module.name}-limit`}
                                className="whitespace-nowrap text-xs text-foreground/70"
                            >
                                Limit(KB/s):
                            </Label>
                            <Input
                                id={`${module.name}-limit`}
                                value={getDisplayValue("limit_kbps")}
                                onChange={(e) =>
                                    handleInputChange(e, "limit_kbps", 0.1, 100000, false)
                                }
                                onBlur={() => handleInputBlur("limit_kbps", 0.1, 100000, false)}
                                className="h-6 w-[55px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                                disabled={!module.enabled}
                                type="text"
                                inputMode="decimal"
                                step="0.1"
                            />
                        </div>
                        <MyraCheckbox
                            id={`${module.name}-use-wfp`}
                            checked={module.config.use_wfp ?? false}
                            onCheckedChange={(checked) =>
                                onBooleanSettingChange?.(module, "use_wfp", checked === true)
                            }
                            disabled={!module.enabled}
                            label="WFP"
                            labelClassName={`text-xs text-foreground ${!module.enabled ? "opacity-50" : ""}`}
                        />
                    </>
                )}
                {module.name === "reorder" && (
                    <>
                        <div className="flex items-center gap-1">
                            <Label
                                htmlFor={`${module.name}-max-delay`}
                                className="whitespace-nowrap text-xs text-foreground/70"
                            >
                                Max(ms):
                            </Label>
                            <Input
                                id={`${module.name}-max-delay`}
                                value={getDisplayValue("throttle_ms")}
                                onChange={(e) =>
                                    handleInputChange(e, "throttle_ms", 1, 60000, true)
                                }
                                onBlur={() => handleInputBlur("throttle_ms", 1, 60000, true)}
                                className="h-6 w-[50px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                                disabled={!module.enabled}
                                type="text"
                                inputMode="numeric"
                            />
                        </div>
                    </>
                )}
                {module.name === "burst" && (
                    <>
                        <div className="flex items-center gap-1">
                            <Label
                                htmlFor={`${module.name}-buffer`}
                                className="whitespace-nowrap text-xs text-foreground/70"
                            >
                                Buffer(ms):
                            </Label>
                            <InfinityInput
                                id={`${module.name}-buffer`}
                                value={getDisplayValue("buffer_ms")}
                                onChange={(e) => handleInputChange(e, "buffer_ms", 0, 10000, true)}
                                className="w-[45px]"
                                disabled={!module.enabled}
                                infinityLabel="∞"
                                editPlaceholder="ms"
                            />
                        </div>
                        <div className="flex items-center gap-1">
                            <Label
                                htmlFor={`${module.name}-release-delay`}
                                className="whitespace-nowrap text-xs text-foreground/70"
                            >
                                Replay(μs):
                            </Label>
                            <Input
                                id={`${module.name}-release-delay`}
                                value={getDisplayValue("release_delay_us")}
                                onChange={(e) =>
                                    handleInputChange(e, "release_delay_us", 0, 50000, true)
                                }
                                onBlur={() => handleInputBlur("release_delay_us", 0, 50000, true)}
                                className="h-6 w-[50px] rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary"
                                disabled={!module.enabled}
                                type="text"
                                inputMode="numeric"
                            />
                        </div>
                        <MyraCheckbox
                            id={`${module.name}-reverse`}
                            checked={module.config.reverse ?? false}
                            onCheckedChange={() =>
                                onBooleanSettingChange?.(module, "reverse", !module.config.reverse)
                            }
                            disabled={!module.enabled}
                            label="Reverse"
                            labelClassName={`text-xs text-foreground ${!module.enabled ? "opacity-50" : ""}`}
                        />
                    </>
                )}
            </div>

            {/* Spacer to push right controls to the right */}
            <div className="flex-1" />

            {/* Direction Controls - docked right */}
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

            {/* Chance - docked right */}
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

            {/* Duration - docked right (not for lag since it uses Time above) */}
            {module.name !== "lag" && (
                <div className="flex items-center gap-1">
                    <Label
                        htmlFor={`${module.name}-duration`}
                        className="whitespace-nowrap text-xs text-foreground/70"
                    >
                        Dur(ms):
                    </Label>
                    <InfinityInput
                        id={`${module.name}-duration`}
                        value={getDisplayValue("duration_ms")}
                        onChange={(e) => handleInputChange(e, "duration_ms", 0, 999999)}
                        className="w-[45px]"
                        disabled={!module.enabled}
                        infinityLabel="∞"
                        editPlaceholder="ms"
                    />
                </div>
            )}
        </div>
    );
}
