import { useEffect, useReducer } from "react";
import {
    startTcBandwidth,
    stopTcBandwidth,
    getTcBandwidthStatus,
    TcDirection,
    TcBandwidthStatus,
} from "@/lib/services/tc-bandwidth";
import { MyraCheckbox } from "./ui/myra-checkbox";

interface State {
    enabled: boolean;
    limitKbps: number;
    direction: TcDirection;
    status: TcBandwidthStatus | null;
    error: string | null;
    loading: boolean;
}

type Action =
    | { type: "setLimit"; value: number }
    | { type: "setDirection"; value: TcDirection }
    | { type: "statusFetched"; status: TcBandwidthStatus }
    | { type: "operationStart" }
    | { type: "operationEnd"; enabled?: boolean; error?: string | null };

const initialState: State = {
    enabled: false,
    limitKbps: 1.0,
    direction: "inbound",
    status: null,
    error: null,
    loading: false,
};

function reducer(state: State, action: Action): State {
    switch (action.type) {
        case "setLimit":
            return { ...state, limitKbps: action.value };

        case "setDirection":
            return { ...state, direction: action.value };

        case "statusFetched":
            return { ...state, status: action.status, enabled: action.status.active };

        case "operationStart":
            return { ...state, loading: true, error: null };

        case "operationEnd":
            return {
                ...state,
                loading: false,
                ...(action.enabled !== undefined ? { enabled: action.enabled } : {}),
                ...(action.error !== undefined ? { error: action.error } : {}),
            };
        default:
            return state;
    }
}

export function TcBandwidthControl() {
    const [state, dispatch] = useReducer(reducer, initialState);
    const { enabled, limitKbps, direction, status, error, loading } = state;

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const s = await getTcBandwidthStatus();
                dispatch({ type: "statusFetched", status: s });
            } catch (e) {
                console.error("Failed to get TC status:", e);
            }
        };
        fetchStatus();
        const interval = setInterval(fetchStatus, 2000);

        return () => clearInterval(interval);
    }, []);

    const handleToggle = async (checked: boolean) => {
        dispatch({ type: "operationStart" });

        try {
            if (!checked) {
                await stopTcBandwidth();
                dispatch({ type: "operationEnd", enabled: false });
                return;
            }

            await startTcBandwidth(limitKbps, direction);
            dispatch({ type: "operationEnd", enabled: true });
        } catch (e: any) {
            dispatch({ type: "operationEnd", error: e.toString() });
        }
    };

    const handleApply = async () => {
        if (!enabled) return;

        dispatch({ type: "operationStart" });

        try {
            // Stop and restart with new settings
            await stopTcBandwidth();
            await startTcBandwidth(limitKbps, direction);
            dispatch({ type: "operationEnd" });
        } catch (e: any) {
            dispatch({ type: "operationEnd", error: e.toString() });
        }
    };

    return (
        <div className="rounded-lg border border-zinc-700 bg-zinc-900/50 p-4">
            <div className="mb-3 flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <h3 className="text-sm font-medium text-zinc-200">NetLimiter Mode</h3>
                    <span className="text-xs text-zinc-500">(Traffic Control)</span>
                </div>
                <MyraCheckbox
                    checked={enabled}
                    onCheckedChange={handleToggle}
                    disabled={loading}
                    label={enabled ? "Active" : "Inactive"}
                />
            </div>
            <p className="mb-3 text-xs text-zinc-500">
                Bandwidth limiting with packet pacing. Small packets (ACKs/keepalives) pass through
                to maintain connection.
            </p>
            <div className="mb-3 flex items-center gap-3">
                <div className="w-24">
                    <label htmlFor="tc-limit" className="mb-1 block text-xs text-zinc-400">
                        Limit (KB/s)
                    </label>
                    <input
                        id="tc-limit"
                        type="number"
                        min={0.1}
                        max={9999}
                        step={0.1}
                        value={limitKbps}
                        onChange={(e) =>
                            dispatch({
                                type: "setLimit",
                                value: Math.max(0.1, parseFloat(e.target.value) || 0.1),
                            })
                        }
                        className="w-full rounded border border-zinc-700 bg-zinc-800 px-2 py-1 text-sm text-zinc-200"
                    />
                </div>
                <div className="flex-1">
                    <label htmlFor="tc-direction" className="mb-1 block text-xs text-zinc-400">
                        Direction
                    </label>
                    <select
                        id="tc-direction"
                        value={direction}
                        onChange={(e) =>
                            dispatch({ type: "setDirection", value: e.target.value as TcDirection })
                        }
                        className="w-full rounded border border-zinc-700 bg-zinc-800 px-2 py-1 text-sm text-zinc-200"
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
                        className="mt-5 rounded bg-blue-600 px-3 py-1 text-sm hover:bg-blue-500 disabled:opacity-50"
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
            {error && <div className="mt-2 text-xs text-red-400">Error: {error}</div>}
        </div>
    );
}
