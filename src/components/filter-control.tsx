import React, { useState, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { useNetworkStore } from "@/lib/stores/network";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

export function FilterControl() {
    const [localFilter, setLocalFilter] = useState<string>("");
    const [isFocused, setIsFocused] = useState(false);
    const { isActive, filter, isUpdatingFilter, updateFilter } = useNetworkStore();

    useEffect(() => {
        setLocalFilter(filter);
    }, [filter]);

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setLocalFilter(e.target.value);
    };

    const applyFilterChange = async () => {
        if (localFilter === filter || isActive) return;
        await updateFilter(localFilter);
    };

    const handleFilterKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter" && !isActive) {
            e.preventDefault();
            applyFilterChange();
        }
    };

    const handleFocus = () => {
        setIsFocused(true);
    };

    const handleBlur = () => {
        setIsFocused(false);

        if (localFilter === filter || isActive) return;

        applyFilterChange();
    };

    return (
        <div className="flex flex-1 items-center">
            <Tooltip>
                <TooltipTrigger asChild>
                    <Label
                        htmlFor="filter"
                        className="mr-2 cursor-help whitespace-nowrap text-sm font-medium text-foreground"
                    >
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
            <div className="relative flex-1">
                <Input
                    id="filter"
                    value={localFilter}
                    onChange={handleInputChange}
                    onKeyDown={handleFilterKeyDown}
                    onFocus={handleFocus}
                    onBlur={handleBlur}
                    placeholder="outbound"
                    className={cn(
                        "h-9 text-sm transition-all duration-200",
                        "border-border bg-background text-foreground",
                        "hover:border-border focus:border-primary focus:ring-1 focus:ring-primary/30",
                        isActive && "cursor-not-allowed opacity-70",
                        isFocused && "border-primary/50"
                    )}
                    disabled={isUpdatingFilter || isActive}
                />
                {isUpdatingFilter && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        className="absolute right-2 top-1/2 -translate-y-1/2"
                    >
                        <Loader2 className="h-4 w-4 animate-spin text-primary/70" />
                    </motion.div>
                )}
            </div>
        </div>
    );
}
