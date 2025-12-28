import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { Search, ChevronDown, Gamepad2, Monitor } from "lucide-react";
import { cn } from "@/lib/utils";
import { ProcessInfo } from "@/types";
import { ProcessIcon } from "@/components/ui/process-icon";
import { Input } from "@/components/ui/input";

interface ProcessSelectorProps {
    processes: ProcessInfo[];
    value: string;
    onValueChange: (value: string) => void;
    disabled?: boolean;
    placeholder?: string;
}

export function ProcessSelector({
    processes,
    value,
    onValueChange,
    disabled,
    placeholder = "Select a process ..",
}: ProcessSelectorProps) {
    const [isOpen, setIsOpen] = useState(false);
    const [search, setSearch] = useState("");
    const [highlightedIndex, setHighlightedIndex] = useState(0);
    const containerRef = useRef<HTMLDivElement>(null);
    const listRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLInputElement>(null);

    const filteredProcesses = useMemo(() => {
        const searchLower = search.toLowerCase();
        return processes.filter(
            (p) =>
                p.name.toLowerCase().includes(searchLower) ||
                p.path?.toLowerCase().includes(searchLower)
        );
    }, [processes, search]);

    const { gameProcesses, otherProcesses, allFiltered } = useMemo(() => {
        const games = filteredProcesses.filter((p) => {
            const name = p.name.toLowerCase();
            const path = p.path?.toLowerCase() || "";
            return (
                path.includes("steam") ||
                path.includes("steamapps") ||
                path.includes("epic games") ||
                path.includes("riot") ||
                path.includes("battle.net") ||
                path.includes("origin") ||
                path.includes("ubisoft") ||
                path.includes("rockstar") ||
                path.includes("ea games") ||
                name.includes("game") ||
                name.endsWith("-win64-shipping.exe") ||
                name.endsWith("-win32-shipping.exe")
            );
        });

        const others = filteredProcesses.filter((p) => !games.includes(p));

        const all = [...games, ...others];

        return { gameProcesses: games, otherProcesses: others, allFiltered: all };
    }, [filteredProcesses]);

    const selectedProcess = processes.find((p) => p.pid.toString() === value);

    const handleKeyDown = useCallback(
        (e: React.KeyboardEvent) => {
            if (!isOpen) {
                if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
                    e.preventDefault();
                    setIsOpen(true);
                    setTimeout(() => inputRef.current?.focus(), 0);
                }
                return;
            }

            switch (e.key) {
                case "ArrowDown":
                    e.preventDefault();
                    setHighlightedIndex((prev) => Math.min(prev + 1, allFiltered.length - 1));
                    break;
                case "ArrowUp":
                    e.preventDefault();
                    setHighlightedIndex((prev) => Math.max(prev - 1, 0));
                    break;
                case "Enter":
                    e.preventDefault();
                    if (allFiltered[highlightedIndex]) {
                        onValueChange(allFiltered[highlightedIndex].pid.toString());
                        setIsOpen(false);
                        setSearch("");
                    }
                    break;
                case "Escape":
                    e.preventDefault();
                    setIsOpen(false);
                    setSearch("");
                    break;
                case "Home":
                    e.preventDefault();
                    setHighlightedIndex(0);
                    break;
                case "End":
                    e.preventDefault();
                    setHighlightedIndex(allFiltered.length - 1);
                    break;
                default:
                    // Type-ahead: jump to first process starting with the typed letter
                    if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
                        const char = e.key.toLowerCase();
                        const currentIndex = highlightedIndex;

                        // Find next process starting with this letter after current position
                        let foundIndex = allFiltered.findIndex(
                            (p, i) => i > currentIndex && p.name.toLowerCase().startsWith(char)
                        );

                        // If not found after current, search from beginning
                        if (foundIndex === -1) {
                            foundIndex = allFiltered.findIndex((p) =>
                                p.name.toLowerCase().startsWith(char)
                            );
                        }

                        if (foundIndex !== -1) {
                            setHighlightedIndex(foundIndex);
                        }
                    }
                    break;
            }
        },
        [isOpen, highlightedIndex, allFiltered, onValueChange]
    );

    // Scroll highlighted item into view
    useEffect(() => {
        if (isOpen && listRef.current) {
            const highlightedEl = listRef.current.querySelector(
                `[data-index="${highlightedIndex}"]`
            );
            highlightedEl?.scrollIntoView({ block: "nearest" });
        }
    }, [highlightedIndex, isOpen]);

    // Close on outside click
    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
                setIsOpen(false);
                setSearch("");
            }
        };

        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    // Reset highlight when search changes
    useEffect(() => {
        setHighlightedIndex(0);
    }, [search]);

    // Reset highlight when opening
    useEffect(() => {
        if (isOpen) {
            // Try to highlight the currently selected item
            const selectedIndex = allFiltered.findIndex((p) => p.pid.toString() === value);
            setHighlightedIndex(selectedIndex >= 0 ? selectedIndex : 0);
        }
    }, [isOpen, value, allFiltered]);

    return (
        <div ref={containerRef} className="relative w-full">
            {/* Trigger Button */}
            <button
                type="button"
                onClick={() => {
                    if (!disabled) {
                        setIsOpen(!isOpen);
                        if (!isOpen) {
                            setTimeout(() => inputRef.current?.focus(), 0);
                        }
                    }
                }}
                onKeyDown={handleKeyDown}
                disabled={disabled}
                className={cn(
                    "flex h-9 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm",
                    "ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
                    "disabled:cursor-not-allowed disabled:opacity-50",
                    isOpen && "ring-2 ring-ring ring-offset-2"
                )}
            >
                <div className="flex min-w-0 flex-1 items-center gap-2">
                    {selectedProcess ? (
                        <>
                            <ProcessIcon
                                icon={selectedProcess.icon}
                                name={selectedProcess.name}
                                className="h-4 w-4 flex-shrink-0"
                            />
                            <span className="truncate">{selectedProcess.name}</span>
                        </>
                    ) : (
                        <span className="text-muted-foreground">{placeholder}</span>
                    )}
                </div>
                <ChevronDown className="h-4 w-4 opacity-50" />
            </button>

            {/* Dropdown */}
            {isOpen && (
                <div
                    className={cn(
                        "absolute left-0 z-[100] mt-1 w-full min-w-[280px] rounded-md border border-border bg-background shadow-xl",
                        "animate-in fade-in-0 zoom-in-95"
                    )}
                >
                    {/* Search Input */}
                    <div className="border-b border-border bg-background p-2">
                        <div className="relative">
                            <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                            <Input
                                ref={inputRef}
                                value={search}
                                onChange={(e) => setSearch(e.target.value)}
                                onKeyDown={handleKeyDown}
                                placeholder="Search or type letter to jump .."
                                className="h-8 pl-8 text-sm"
                                autoFocus
                            />
                        </div>
                        <p className="mt-1.5 text-[10px] text-muted-foreground">
                            Type a letter to jump • ↑↓ navigate • Enter select
                        </p>
                    </div>

                    {/* Process List */}
                    <div
                        ref={listRef}
                        className="max-h-[200px] overflow-y-auto rounded-b-md bg-background p-1"
                        onKeyDown={handleKeyDown}
                    >
                        {allFiltered.length === 0 ? (
                            <div className="px-2 py-4 text-center text-sm text-muted-foreground">
                                No processes found
                            </div>
                        ) : (
                            <>
                                {/* Games Section */}
                                {gameProcesses.length > 0 && (
                                    <>
                                        <div className="flex items-center gap-1 px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                                            <Gamepad2 className="h-3 w-3" />
                                            Games & Launchers
                                        </div>
                                        {gameProcesses.map((p) => {
                                            const index = allFiltered.indexOf(p);
                                            return (
                                                <ProcessItem
                                                    key={p.pid}
                                                    process={p}
                                                    isSelected={p.pid.toString() === value}
                                                    isHighlighted={index === highlightedIndex}
                                                    dataIndex={index}
                                                    onClick={() => {
                                                        onValueChange(p.pid.toString());
                                                        setIsOpen(false);
                                                        setSearch("");
                                                    }}
                                                    onMouseEnter={() => setHighlightedIndex(index)}
                                                />
                                            );
                                        })}
                                    </>
                                )}

                                {/* Other Processes Section */}
                                {otherProcesses.length > 0 && (
                                    <>
                                        <div className="flex items-center gap-1 px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                                            <Monitor className="h-3 w-3" />
                                            All Processes
                                        </div>
                                        {otherProcesses.map((p) => {
                                            const index = allFiltered.indexOf(p);
                                            return (
                                                <ProcessItem
                                                    key={p.pid}
                                                    process={p}
                                                    isSelected={p.pid.toString() === value}
                                                    isHighlighted={index === highlightedIndex}
                                                    dataIndex={index}
                                                    onClick={() => {
                                                        onValueChange(p.pid.toString());
                                                        setIsOpen(false);
                                                        setSearch("");
                                                    }}
                                                    onMouseEnter={() => setHighlightedIndex(index)}
                                                />
                                            );
                                        })}
                                    </>
                                )}
                            </>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

interface ProcessItemProps {
    process: ProcessInfo;
    isSelected: boolean;
    isHighlighted: boolean;
    dataIndex: number;
    onClick: () => void;
    onMouseEnter: () => void;
}

function ProcessItem({
    process,
    isSelected,
    isHighlighted,
    dataIndex,
    onClick,
    onMouseEnter,
}: ProcessItemProps) {
    return (
        <button
            type="button"
            data-index={dataIndex}
            onClick={onClick}
            onMouseEnter={onMouseEnter}
            className={cn(
                "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none",
                isHighlighted && "bg-accent text-accent-foreground",
                isSelected && "font-medium"
            )}
        >
            <ProcessIcon
                icon={process.icon}
                name={process.name}
                className="h-4 w-4 flex-shrink-0"
            />
            <span className="min-w-0 flex-1 truncate text-left">{process.name}</span>
            <span className="flex-shrink-0 font-mono text-[10px] text-muted-foreground">
                {process.pid}
            </span>
            {isSelected && <span className="flex-shrink-0 text-primary">✓</span>}
        </button>
    );
}
