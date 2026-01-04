import { useTapStore } from "@/lib/stores/tap-store";
import { MyraCheckbox } from "@/components/ui/myra-checkbox";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";

export function TapControl() {
    const { settings, setEnabled, setIntervalMs, setDurationMs } = useTapStore();

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
                    type="number"
                    min={100}
                    max={60000}
                    step={100}
                    value={settings.intervalMs}
                    onChange={(e) => {
                        const value = parseInt(e.target.value, 10);
                        if (!isNaN(value) && value >= 100) {
                            setIntervalMs(value);
                        }
                    }}
                    className="h-7 w-20 px-2 text-xs"
                    disabled={!settings.enabled}
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
                    type="number"
                    min={50}
                    max={10000}
                    step={50}
                    value={settings.durationMs}
                    onChange={(e) => {
                        const value = parseInt(e.target.value, 10);
                        if (!isNaN(value) && value >= 50) {
                            setDurationMs(value);
                        }
                    }}
                    className="h-7 w-20 px-2 text-xs"
                    disabled={!settings.enabled}
                />
                <span className="text-xs text-foreground/70">ms</span>
            </div>
        </div>
    );
}
