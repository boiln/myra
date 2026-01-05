import { useState, useEffect } from "react";
import { startTcBandwidth, stopTcBandwidth, getTcBandwidthStatus, TcDirection, TcBandwidthStatus } from "@/lib/services/tc-bandwidth";
import { MyraCheckbox } from "./ui/myra-checkbox";

export function TcBandwidthControl() {
    const [enabled, setEnabled] = useState(false);
    const [limitKbps, setLimitKbps] = useState(1.0);
    const [direction, setDirection] = useState<TcDirection>("inbound");
    const [status, setStatus] = useState<TcBandwidthStatus | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const s = await getTcBandwidthStatus();
                setStatus(s);
                setEnabled(s.active);
            } catch (e) {
                console.error("Failed to get TC status:", e);
            }
        };
        
        fetchStatus();
        const interval = setInterval(fetchStatus, 2000);
        return () => clearInterval(interval);
    }, []);

    const handleToggle = async (checked: boolean) => {
        setLoading(true);
        setError(null);
        
        try {
            if (!checked) {
                await stopTcBandwidth();
                setEnabled(false);
                return;
            }

            await startTcBandwidth(limitKbps, direction);
            setEnabled(true);
        } catch (e: any) {
            setError(e.toString());
        } finally {
            setLoading(false);
        }
    };

    const handleApply = async () => {
        if (!enabled) return;
        
        setLoading(true);
        setError(null);
        
        try {
            // Stop and restart with new settings
            await stopTcBandwidth();
            await startTcBandwidth(limitKbps, direction);
        } catch (e: any) {
            setError(e.toString());
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="border border-zinc-700 rounded-lg p-4 bg-zinc-900/50">
            <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                    <h3 className="text-sm font-medium text-zinc-200">
                        NetLimiter Mode
                    </h3>
                    <span className="text-xs text-zinc-500">(Traffic Control)</span>
                </div>
                <MyraCheckbox
                    checked={enabled}
                    onCheckedChange={handleToggle}
                    disabled={loading}
                    label={enabled ? "Active" : "Inactive"}
                />
            </div>
            
            <p className="text-xs text-zinc-500 mb-3">
                Bandwidth limiting with packet pacing. Small packets (ACKs/keepalives) pass through to maintain connection.
            </p>
            
            <div className="flex gap-3 items-center mb-3">
                <div className="w-24">
                    <label className="text-xs text-zinc-400 block mb-1">Limit (KB/s)</label>
                    <input
                        type="number"
                        min={0.1}
                        max={9999}
                        step={0.1}
                        value={limitKbps}
                        onChange={(e) => setLimitKbps(Math.max(0.1, parseFloat(e.target.value) || 0.1))}
                        className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-200"
                    />
                </div>
                
                <div className="flex-1">
                    <label className="text-xs text-zinc-400 block mb-1">Direction</label>
                    <select
                        value={direction}
                        onChange={(e) => setDirection(e.target.value as TcDirection)}
                        className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-200"
                    >
                        <option value="inbound">Inbound (Download)</option>
                        <option value="outbound">Outbound (Upload)</option>
                        <option value="both">Both</option>
                    </select>
                </div>
                
                {enabled && (
                    <button
                        onClick={handleApply}
                        disabled={loading}
                        className="mt-5 px-3 py-1 text-sm bg-blue-600 hover:bg-blue-500 rounded disabled:opacity-50"
                    >
                        Apply
                    </button>
                )}
            </div>
            
            {status && status.active && (
                <div className="text-xs text-green-400">
                    ✓ Active: {status.limit_kbps} KB/s ({status.direction})
                </div>
            )}
            
            {error && (
                <div className="text-xs text-red-400 mt-2">
                    Error: {error}
                </div>
            )}
        </div>
    );
}

