import React, { ChangeEvent, useEffect, useState } from "react";
import { useTapStore } from "@/lib/stores/tap-store";
import { MyraCheckbox } from "@/components/ui/myra-checkbox";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";

export function TapControl() {
    const {
        settings,
        setEnabled,
        setIntervalMs,
        setDurationMs,
        setAutoEnabled,
        setMinBufferForTap,
        setCooldownMs,
    } = useTapStore();

    // Local, lenient input state (matches module inputs behavior)
    const [inputValues, setInputValues] = useState<Record<string, string>>({});

    const getDisplayValue = (key: string, fallback: number) => {
        if (key in inputValues) return inputValues[key];
        return (fallback ?? 0).toString();
    };

    const handleInputChange = (
        e: ChangeEvent<HTMLInputElement>,
        key: "intervalMs" | "durationMs" | "minBufferForTap" | "cooldownMs",
        isInteger = true
    ) => {
        const input = e.target.value;
        setInputValues((prev) => ({ ...prev, [key]: input }));

        // Allow empty and partial numeric input during typing
        if (input === "") return;
        if (!/^-?\d*\.?\d*$/.test(input)) return;
        if (input === "." || input === "-" || input === "-." || input.endsWith(".")) return;

        const parsed = isInteger ? parseInt(input, 10) : parseFloat(input);
        if (isNaN(parsed)) return;

        // Do not clamp while typing; update store with parsed value
        switch (key) {
            case "intervalMs":
                setIntervalMs(parsed);
                break;
            case "durationMs":
                setDurationMs(parsed);
                break;
            case "minBufferForTap":
                setMinBufferForTap(parsed);
                break;
            case "cooldownMs":
                setCooldownMs(parsed);
                break;
        }
    };

    const handleInputBlur = (
        key: "intervalMs" | "durationMs" | "minBufferForTap" | "cooldownMs",
        min: number,
        max: number,
        isInteger = true
    ) => {
        const input = inputValues[key];
        const parsed = isInteger ? parseInt(input ?? "", 10) : parseFloat(input ?? "");

        let clamped: number;
        if (isNaN(parsed)) {
            clamped = min;
        } else {
            clamped = Math.max(min, Math.min(max, parsed));
        }

        setInputValues((prev) => ({ ...prev, [key]: clamped.toString() }));

        switch (key) {
            case "intervalMs":
                setIntervalMs(clamped);
                break;
            case "durationMs":
                setDurationMs(clamped);
                break;
            case "minBufferForTap":
                setMinBufferForTap(clamped);
                break;
            case "cooldownMs":
                setCooldownMs(clamped);
                break;
        }
    };

    // Sync UI inputs when TAP settings change (e.g., after loading a preset)
    useEffect(() => {
        setInputValues({
            intervalMs: (settings.intervalMs ?? 0).toString(),
            durationMs: (settings.durationMs ?? 0).toString(),
            minBufferForTap: (settings.minBufferForTap ?? 200).toString(),
            cooldownMs: (settings.cooldownMs ?? 1200).toString(),
        });
    }, [settings.intervalMs, settings.durationMs, settings.minBufferForTap, settings.cooldownMs]);

    return (
        <div className="flex items-center gap-3 rounded-lg border border-border bg-card/90 p-2 shadow-sm backdrop-blur-sm">
            {/* Tap enable checkbox */}
            <MyraCheckbox
                id="tap-enabled"
                checked={settings.enabled}
                onCheckedChange={setEnabled}
                label="Tap"
                labelClassName="text-sm font-medium text-foreground"
            />

            {/* Interval setting */}
            <div className="flex items-center gap-1">
                <Label
                    htmlFor="tap-interval"
                    className="whitespace-nowrap text-xs text-foreground/70"
                >
                    Every:
                </Label>
                <Input
                    id="tap-interval"
                    value={getDisplayValue("intervalMs", settings.intervalMs)}
                    onChange={(e) => handleInputChange(e, "intervalMs", true)}
                    onBlur={() => handleInputBlur("intervalMs", 100, 60000, true)}
                    className="h-7 w-20 px-2 text-center text-xs"
                    disabled={!settings.enabled}
                    type="text"
                    inputMode="numeric"
                />
                <span className="text-xs text-foreground/70">ms</span>
            </div>

            {/* Duration setting */}
            <div className="flex items-center gap-1">
                <Label
                    htmlFor="tap-duration"
                    className="whitespace-nowrap text-xs text-foreground/70"
                >
                    Off for:
                </Label>
                <Input
                    id="tap-duration"
                    value={getDisplayValue("durationMs", settings.durationMs)}
                    onChange={(e) => handleInputChange(e, "durationMs", true)}
                    onBlur={() => handleInputBlur("durationMs", 50, 10000, true)}
                    className="h-7 w-20 px-2 text-center text-xs"
                    disabled={!settings.enabled}
                    type="text"
                    inputMode="numeric"
                />
                <span className="text-xs text-foreground/70">ms</span>
            </div>

            {/* Auto mode toggle */}
            <MyraCheckbox
                id="tap-auto-enabled"
                checked={settings.autoEnabled ?? false}
                onCheckedChange={(v) => setAutoEnabled(v === true)}
                label="Auto"
                labelClassName="text-sm font-medium text-foreground"
                disabled={!settings.enabled}
            />

            {/* Auto thresholds */}
            <div className="flex items-center gap-1">
                <Label
                    htmlFor="tap-buffer-threshold"
                    className="whitespace-nowrap text-xs text-foreground/70"
                >
                    Buffer≥
                </Label>
                <Input
                    id="tap-buffer-threshold"
                    value={getDisplayValue("minBufferForTap", settings.minBufferForTap ?? 200)}
                    onChange={(e) => handleInputChange(e, "minBufferForTap", true)}
                    onBlur={() => handleInputBlur("minBufferForTap", 10, 100000, true)}
                    className="h-7 w-20 px-2 text-center text-xs"
                    disabled={!settings.enabled || !(settings.autoEnabled ?? false)}
                    type="text"
                    inputMode="numeric"
                />
            </div>

            <div className="flex items-center gap-1">
                <Label
                    htmlFor="tap-cooldown"
                    className="whitespace-nowrap text-xs text-foreground/70"
                >
                    Cooldown:
                </Label>
                <Input
                    id="tap-cooldown"
                    value={getDisplayValue("cooldownMs", settings.cooldownMs ?? 1200)}
                    onChange={(e) => handleInputChange(e, "cooldownMs", true)}
                    onBlur={() => handleInputBlur("cooldownMs", 100, 10000, true)}
                    className="h-7 w-20 px-2 text-center text-xs"
                    disabled={!settings.enabled || !(settings.autoEnabled ?? false)}
                    type="text"
                    inputMode="numeric"
                />
                <span className="text-xs text-foreground/70">ms</span>
            </div>
        </div>
    );
}
