import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Monitor } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useNetworkStore } from "@/lib/stores/network";
import { ProcessInfo } from "@/types";
import { ProcessSelector } from "@/components/ui/process-selector";
import { MyraCheckbox } from "@/components/ui/myra-checkbox";

interface FilterTargetSelectorProps {
    disabled?: boolean;
}

export function FilterTargetSelector({ disabled }: FilterTargetSelectorProps) {
    const { isActive, filter, updateFilter, filterTarget, setFilterTarget } = useNetworkStore();

    // Local filter state - syncs with store
    const [localFilter, setLocalFilter] = useState(filter || "outbound");
    const [filterError, setFilterError] = useState<string | null>(null);

    // Process selection state
    const [processes, setProcesses] = useState<ProcessInfo[]>([]);
    const [loadingProcesses, setLoadingProcesses] = useState(false);
    const [selectedProcess, setSelectedProcess] = useState<string>(
        filterTarget?.processId?.toString() || ""
    );

    // Direction toggles
    const [includeInbound, setIncludeInbound] = useState(filterTarget?.includeInbound ?? false);
    const [includeOutbound, setIncludeOutbound] = useState(filterTarget?.includeOutbound ?? true);

    // Debounce timer for auto-apply
    const filterTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    // Track previous process mode
    const prevProcessRef = useRef<string | null>(selectedProcess);

    // Flag to skip auto-apply when syncing from preset load
    const skipAutoApplyRef = useRef(false);

    // Sync local filter with store filter
    useEffect(() => {
        if (filter && filter !== localFilter) {
            setLocalFilter(filter);
        }
    }, [filter]);

    // Sync state when filterTarget changes (e.g., from loading preset)
    useEffect(() => {
        if (filterTarget) {
            // Skip auto-apply when syncing from preset - the filter was already set
            skipAutoApplyRef.current = true;

            if (filterTarget.processId) {
                setSelectedProcess(filterTarget.processId.toString());
            } else {
                setSelectedProcess("");
            }
            setIncludeInbound(filterTarget.includeInbound ?? false);
            setIncludeOutbound(filterTarget.includeOutbound ?? true);

            // Reset the skip flag after state updates propagate
            setTimeout(() => {
                skipAutoApplyRef.current = false;
            }, 200);
        }
    }, [filterTarget]);

    // Load processes
    const loadProcesses = useCallback(async () => {
        setLoadingProcesses(true);
        try {
            const result = await invoke<ProcessInfo[]>("list_processes");
            setProcesses(result);
        } catch (error) {
            console.error("Failed to load processes:", error);
        } finally {
            setLoadingProcesses(false);
        }
    }, []);

    // Load processes on mount
    useEffect(() => {
        loadProcesses();
    }, [loadProcesses]);

    // Validate filter with backend
    const validateFilter = useCallback(async (filterStr: string): Promise<boolean> => {
        try {
            const isValid = await invoke<boolean>("validate_filter", { filter: filterStr });
            if (isValid) {
                setFilterError(null);
                return true;
            }
            return false;
        } catch (error) {
            setFilterError(error as string);
            return false;
        }
    }, []);

    // Build filter string from current state
    const buildFilterString = useCallback(async (): Promise<string> => {
        let baseFilter = "";

        // Build direction part
        const dirFilter =
            includeInbound && includeOutbound ? "true" : includeInbound ? "inbound" : "outbound";

        // If process is selected, build process filter
        if (selectedProcess) {
            const pid = parseInt(selectedProcess);

            // Stop flow tracking if we had a different process
            if (prevProcessRef.current && prevProcessRef.current !== selectedProcess) {
                await invoke("stop_flow_tracking").catch(() => {});
            }
            prevProcessRef.current = selectedProcess;

            // Start flow tracking for this process
            await invoke("start_flow_tracking", { pid }).catch((e) =>
                console.warn("Flow tracking start failed:", e)
            );

            // Get process filter
            baseFilter = await invoke<string>("build_process_filter", {
                pid,
                includeInbound,
                includeOutbound,
            });

            // Try to get flow-based filter if available
            const flowFilter = await invoke<string | null>("get_flow_filter").catch(() => null);
            if (flowFilter) {
                baseFilter = flowFilter;
            }

            // Update filter target
            const process = processes.find((p) => p.pid === pid);
            setFilterTarget({
                mode: "process",
                processId: pid,
                processName: process?.name,
                includeInbound,
                includeOutbound,
            });
        } else {
            // Stop flow tracking if we were tracking
            if (prevProcessRef.current) {
                await invoke("stop_flow_tracking").catch(() => {});
                prevProcessRef.current = null;
            }

            baseFilter = dirFilter;
            setFilterTarget({
                mode: "all",
                includeInbound,
                includeOutbound,
            });
        }

        return baseFilter;
    }, [selectedProcess, includeInbound, includeOutbound, processes, setFilterTarget]);

    // Auto-apply filter when dependencies change
    useEffect(() => {
        // Skip if filtering is active or if we're syncing from a preset load
        if (isActive || skipAutoApplyRef.current) return;

        const applyFilter = async () => {
            const newFilter = await buildFilterString();
            setLocalFilter(newFilter);

            // Validate before applying
            const isValid = await validateFilter(newFilter);
            if (isValid) {
                await updateFilter(newFilter);
            }
        };

        // Debounce the filter application
        if (filterTimeoutRef.current) {
            clearTimeout(filterTimeoutRef.current);
        }
        filterTimeoutRef.current = setTimeout(applyFilter, 150);

        return () => {
            if (filterTimeoutRef.current) {
                clearTimeout(filterTimeoutRef.current);
            }
        };
    }, [
        selectedProcess,
        includeInbound,
        includeOutbound,
        isActive,
        buildFilterString,
        validateFilter,
        updateFilter,
    ]);

    // Handle manual filter input change
    const handleFilterChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const newFilter = e.target.value;
        setLocalFilter(newFilter);
        setFilterError(null);
    };

    // Handle filter blur - validate and apply
    const handleFilterBlur = async () => {
        if (isActive || localFilter === filter) return;

        const isValid = await validateFilter(localFilter);
        if (isValid) {
            await updateFilter(localFilter);
        }
    };

    // Handle filter keydown
    const handleFilterKeyDown = async (e: React.KeyboardEvent) => {
        if (e.key === "Enter" && !isActive) {
            e.preventDefault();
            const isValid = await validateFilter(localFilter);
            if (isValid) {
                await updateFilter(localFilter);
            }
        }
    };

    // Handle direction change
    const handleDirectionChange = (direction: "inbound" | "outbound", checked: boolean) => {
        if (direction === "inbound") {
            // Ensure at least one is selected
            if (!checked && !includeOutbound) {
                setIncludeOutbound(true);
            }
            setIncludeInbound(checked);
        } else {
            // Ensure at least one is selected
            if (!checked && !includeInbound) {
                setIncludeInbound(true);
            }
            setIncludeOutbound(checked);
        }
    };

    // Handle process change
    const handleProcessChange = (value: string) => {
        setSelectedProcess(value);
    };

    return (
        <div className="flex flex-col gap-2">
            {/* Main row: Filter input + Process selector */}
            <div className="flex items-center gap-2">
                {/* Filter input */}
                <div className="flex flex-1 items-center gap-2">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Label className="shrink-0 cursor-help text-sm font-medium">
                                Filter:
                            </Label>
                        </TooltipTrigger>
                        <TooltipContent side="bottom" className="max-w-xs">
                            <p className="text-xs">
                                WinDivert filter syntax. Examples:
                                <br />• <code>outbound</code> - outgoing traffic only
                                <br />• <code>inbound</code> - incoming traffic only
                                <br />• <code>true</code> - all traffic
                                <br />• <code>tcp.DstPort == 80</code> - HTTP traffic
                            </p>
                        </TooltipContent>
                    </Tooltip>
                    <Input
                        value={localFilter}
                        onChange={handleFilterChange}
                        onBlur={handleFilterBlur}
                        onKeyDown={handleFilterKeyDown}
                        placeholder="outbound"
                        className={cn(
                            "h-8 flex-1 font-mono text-sm",
                            filterError && "border-red-500",
                            isActive && "cursor-not-allowed opacity-60"
                        )}
                        disabled={disabled || isActive}
                    />
                </div>

                {/* Direction toggles */}
                <div className="flex items-center gap-2">
                    <MyraCheckbox
                        id="filter-inbound"
                        checked={includeInbound}
                        onCheckedChange={(checked) => handleDirectionChange("inbound", !!checked)}
                        disabled={disabled || isActive}
                        label="In"
                        labelClassName="text-xs"
                    />
                    <MyraCheckbox
                        id="filter-outbound"
                        checked={includeOutbound}
                        onCheckedChange={(checked) => handleDirectionChange("outbound", !!checked)}
                        disabled={disabled || isActive}
                        label="Out"
                        labelClassName="text-xs"
                    />
                </div>

                {/* Separator */}
                <div className="h-6 w-px bg-border" />

                {/* Process selector */}
                <div className="flex items-center gap-1.5">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Monitor className="h-4 w-4 shrink-0 text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Target a specific process</p>
                        </TooltipContent>
                    </Tooltip>
                    <ProcessSelector
                        processes={processes}
                        value={selectedProcess}
                        onValueChange={handleProcessChange}
                        disabled={disabled || isActive || loadingProcesses}
                        placeholder="All processes"
                        className="w-[160px]"
                    />
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={loadProcesses}
                        disabled={loadingProcesses || isActive}
                    >
                        <RefreshCw
                            className={cn("h-3.5 w-3.5", loadingProcesses && "animate-spin")}
                        />
                    </Button>
                </div>
            </div>

            {/* Error message */}
            {filterError && <p className="text-xs text-red-500">{filterError}</p>}
        </div>
    );
}
