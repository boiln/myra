import React, { ChangeEvent, useState, useEffect, useRef } from "react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface InfinityInputProps {
    id: string;
    value: string | number;
    onChange: (e: ChangeEvent<HTMLInputElement>) => void;
    disabled?: boolean;
    className?: string;
    /** The label to show when value is 0 (infinity mode) */
    infinityLabel?: string;
    /** Placeholder when clicking on infinity to edit */
    editPlaceholder?: string;
}

/**
 * An input that displays ∞ when the value is 0, indicating infinite/manual mode.
 * Clicking on the infinity symbol allows editing to set a specific value.
 * Setting to 0 or empty returns to infinity mode.
 */
export function InfinityInput({
    id,
    value,
    onChange,
    disabled = false,
    className,
    infinityLabel = "∞",
    editPlaceholder = "0",
}: InfinityInputProps) {
    const numValue = typeof value === "string" ? parseFloat(value) || 0 : value;
    const isInfinity = numValue === 0;
    const [isEditing, setIsEditing] = useState(false);
    const [localValue, setLocalValue] = useState(isInfinity ? "" : value.toString());
    const inputRef = useRef<HTMLInputElement>(null);

    // Sync local value when prop changes (but not during editing)
    useEffect(() => {
        if (!isEditing) {
            setLocalValue(isInfinity ? "" : value.toString());
        }
    }, [value, isInfinity, isEditing]);

    // Focus input when entering edit mode
    useEffect(() => {
        if (isEditing && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [isEditing]);

    const handleInfinityClick = () => {
        if (disabled) return;
        setIsEditing(true);
        setLocalValue("");
    };

    const handleInputChange = (e: ChangeEvent<HTMLInputElement>) => {
        const input = e.target.value;
        setLocalValue(input);
        
        // Forward the change event
        onChange(e);
    };

    const handleBlur = () => {
        setIsEditing(false);
        // If empty or 0, show infinity
        if (localValue === "" || localValue === "0") {
            setLocalValue("");
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Escape") {
            setIsEditing(false);
            setLocalValue(isInfinity ? "" : value.toString());
        } else if (e.key === "Enter") {
            setIsEditing(false);
        }
    };

    // Show infinity display when value is 0 and not editing
    if (isInfinity && !isEditing) {
        return (
            <button
                type="button"
                onClick={handleInfinityClick}
                disabled={disabled}
                className={cn(
                    "flex h-6 items-center justify-center rounded border border-border bg-background/80 text-sm font-medium text-foreground/80 transition-colors",
                    "hover:border-primary/50 hover:text-primary",
                    "focus:outline-none focus:ring-1 focus:ring-primary",
                    disabled && "cursor-not-allowed opacity-50",
                    className
                )}
                title="Click to set a specific duration (0 = infinite)"
            >
                {infinityLabel}
            </button>
        );
    }

    return (
        <Input
            ref={inputRef}
            id={id}
            value={isEditing ? localValue : (isInfinity ? "" : value.toString())}
            onChange={handleInputChange}
            onBlur={handleBlur}
            onKeyDown={handleKeyDown}
            className={cn(
                "h-6 rounded border-border bg-background/80 px-1 text-center text-sm text-foreground focus:border-primary",
                className
            )}
            disabled={disabled}
            type="text"
            inputMode="numeric"
            placeholder={editPlaceholder}
        />
    );
}
