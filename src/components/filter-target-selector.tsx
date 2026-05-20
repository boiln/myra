import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Monitor, ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useNetworkStore } from "@/lib/stores/network";
import { ProcessInfo } from "@/types";
import { ProcessSelector } from "@/components/ui/process-selector";
import { MyraCheckbox } from "@/components/ui/myra-checkbox";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
    DropdownMenuLabel,
    DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { ManipulationService } from "@/lib/services/manipulation";

interface FilterTargetSelectorProps {
    disabled?: boolean;
}

export function FilterTargetSelector({ disabled }: FilterTargetSelectorProps) {
    const { isActive, filter, updateFilter, filterTarget, setFilterTarget, isInitialized } =
        useNetworkStore();

    // Grouped UI state for the filter input field
    const [filterUi, setFilterUi] = useState<{
        localFilter: string;
        error: string | null;
        history: string[];
    }>({
        localFilter: filter || "outbound",
        error: null,
        history: [],
    });
    const { localFilter, error: filterError, history: previousFilters } = filterUi;

    // Grouped process-list state
    const [processList, setProcessList] = useState<{
        list: ProcessInfo[];
        loading: boolean;
    }>({ list: [], loading: false });
    const { list: processes, loading: loadingProcesses } = processList;

    // Grouped filter-target state
    const [target, setTarget] = useState<{
        selectedProcess: string;
        includeInbound: boolean;
        includeOutbound: boolean;
    }>({
        selectedProcess: filterTarget?.processId?.toString() || "",
        includeInbound: filterTarget?.includeInbound ?? false,
        includeOutbound: filterTarget?.includeOutbound ?? true,
    });
    const { selectedProcess, includeInbound, includeOutbound } = target;

    // Debounce timer for auto-apply
    const filterTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    // Track previous process mode
    const prevProcessRef = useRef<string | null>(selectedProcess);

    // Flag to skip auto-apply when syncing from preset load
    const skipAutoApplyRef = useRef(false);

    // Sync local filter with store filter
    useEffect(() => {
        if (filter && filter !== localFilter) {
            setFilterUi((s) => ({ ...s, localFilter: filter }));
        }
    }, [filter]);

    // Sync state when filterTarget changes (e.g., from loading preset)
    useEffect(() => {
        if (filterTarget) {
            // Skip auto-apply when syncing from preset - the filter was already set
            skipAutoApplyRef.current = true;

            setTarget({
                selectedProcess: filterTarget.processId ? filterTarget.processId.toString() : "",
                includeInbound: filterTarget.includeInbound ?? false,
                includeOutbound: filterTarget.includeOutbound ?? true,
            });

            // Reset the skip flag after state updates propagate
            const t = setTimeout(() => {
                skipAutoApplyRef.current = false;
            }, 200);

            return () => clearTimeout(t);
        }
    }, [filterTarget]);

    // Load processes
    const loadProcesses = useCallback(async () => {
        setProcessList((s) => ({ ...s, loading: true }));
        try {
            const result = await invoke<ProcessInfo[]>("list_processes");
            setProcessList({ list: result, loading: false });
        } catch (error) {
            console.error("Failed to load processes:", error);
            setProcessList((s) => ({ ...s, loading: false }));
        }
    }, []);

    // Load processes on mount
    useEffect(() => {
        loadProcesses();
        // Load filter history
        ManipulationService.getFilterHistory()
            .then((list) => setFilterUi((s) => ({ ...s, history: list ?? [] })))
            .catch(() => setFilterUi((s) => ({ ...s, history: [] })));
    }, [loadProcesses]);

    // Validate filter with backend
    const validateFilter = useCallback(async (filterStr: string): Promise<boolean> => {
        try {
            const isValid = await invoke<boolean>("validate_filter", { filter: filterStr });
            if (isValid) {
                setFilterUi((s) => ({ ...s, error: null }));
                return true;
            }

            return false;
        } catch (error) {
            setFilterUi((s) => ({ ...s, error: error as string }));
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
        // Skip if filtering is active, syncing from preset, or app not yet initialized
        if (isActive || skipAutoApplyRef.current || !isInitialized) return;

        const applyFilter = async () => {
            const newFilter = await buildFilterString();

            // If a specific filter is already set in store and differs,
            // do not auto-override it with a generic direction filter.
            if (filter && filter !== newFilter) {
                return;
            }

            setFilterUi((s) => ({ ...s, localFilter: newFilter }));

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
    }, [selectedProcess, includeInbound, includeOutbound, isActive, isInitialized, filter]);

    // Handle manual filter input change
    const handleFilterChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const newFilter = e.target.value;
        setFilterUi((s) => ({ ...s, localFilter: newFilter, error: null }));
    };

    // Handle filter blur - validate and apply
    const handleFilterBlur = async () => {
        if (isActive || localFilter === filter) return;

        const isValid = await validateFilter(localFilter);
        if (isValid) {
            await updateFilter(localFilter);
            // Refresh history after successful update
            try {
                const list = await ManipulationService.getFilterHistory();
                setFilterUi((s) => ({ ...s, history: list ?? [] }));
            } catch {}
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
            setTarget((s) => ({
                ...s,
                includeInbound: checked,
                // Ensure at least one is selected
                includeOutbound: !checked && !s.includeOutbound ? true : s.includeOutbound,
            }));
        } else {
            setTarget((s) => ({
                ...s,
                includeOutbound: checked,
                // Ensure at least one is selected
                includeInbound: !checked && !s.includeInbound ? true : s.includeInbound,
            }));
        }
    };

    // Handle process change
    const handleProcessChange = (value: string) => {
        setTarget((s) => ({ ...s, selectedProcess: value }));
    };

    const applyPreviousFilter = async (value: string) => {
        if (isActive) return; // Do not change while active
        const ok = await validateFilter(value);
        if (!ok) return;
        setFilterUi((s) => ({ ...s, localFilter: value }));
        await updateFilter(value);
        try {
            const list = await ManipulationService.getFilterHistory();
            setFilterUi((s) => ({ ...s, history: list ?? [] }));
        } catch {}
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
                    {/* Previous filters dropdown */}
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                variant="outline"
                                size="icon"
                                className="size-8"
                                disabled={disabled || isActive}
                                title="Previous filters"
                            >
                                <ChevronDown className="size-4" />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="start" className="min-w-[240px]">
                            <DropdownMenuLabel>Recent Filters</DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            {previousFilters.length === 0 ? (
                                <DropdownMenuItem disabled>No recent filters</DropdownMenuItem>
                            ) : (
                                previousFilters.map((f) => (
                                    <DropdownMenuItem
                                        key={f}
                                        onClick={() => applyPreviousFilter(f)}
                                        className="font-mono text-xs"
                                    >
                                        {f}
                                    </DropdownMenuItem>
                                ))
                            )}
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                                onClick={async () => {
                                    if (disabled || isActive) return;
                                    try {
                                        await ManipulationService.clearFilterHistory();
                                        setFilterUi((s) => ({ ...s, history: [] }));
                                    } catch {}
                                }}
                                className="text-xs text-muted-foreground"
                                disabled={disabled || isActive || previousFilters.length === 0}
                            >
                                Clear history
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
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
                            <Monitor className="size-4 shrink-0 text-muted-foreground" />
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
                        className="size-8"
                        onClick={loadProcesses}
                        disabled={loadingProcesses || isActive}
                    >
                        <RefreshCw className={cn("size-3.5", loadingProcesses && "animate-spin")} />
                    </Button>
                </div>
            </div>

            {/* Error message */}
            {filterError && <p className="text-xs text-red-500">{filterError}</p>}
        </div>
    );
}
