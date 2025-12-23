import * as React from "react";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

interface MyraCheckboxProps {
    id?: string;
    label?: string;
    checked?: boolean;
    onCheckedChange?: (checked: boolean) => void;
    disabled?: boolean;
    className?: string;
    labelClassName?: string;
    isHighlighted?: boolean;
}

export function MyraCheckbox({
    id,
    label,
    checked = false,
    onCheckedChange,
    disabled = false,
    className,
    labelClassName,
    isHighlighted = false,
}: MyraCheckboxProps) {
    const uniqueId = React.useId();
    const checkboxId = id || uniqueId;

    return (
        <div className="relative flex items-center">
            <div className="relative">
                {/* Hide the original input but keep it accessible */}
                <input
                    type="checkbox"
                    id={checkboxId}
                    checked={checked}
                    onChange={(e) => onCheckedChange && onCheckedChange(e.target.checked)}
                    disabled={disabled}
                    className="sr-only"
                />

                {/* Custom checkbox UI */}
                <label
                    htmlFor={checkboxId}
                    className={cn(
                        "flex h-4 w-4 items-center justify-center rounded-[3px] border-2 border-border",
                        "transition-all duration-200 ease-in-out",
                        checked && "bg-primary",
                        disabled && "cursor-not-allowed opacity-50",
                        !disabled && "cursor-pointer",
                        isHighlighted && checked && "ring-2 ring-primary/30",
                        className
                    )}
                >
                    {/* SVG Checkmark - much more reliable than CSS */}
                    <svg
                        className={cn(
                            "h-3 w-3 text-primary-foreground",
                            "transition-transform duration-200 ease-in-out",
                            checked ? "scale-100" : "scale-0"
                        )}
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="3"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <polyline points="20 6 9 17 4 12" />
                    </svg>
                </label>
            </div>

            {/* Label */}
            {label && (
                <Label
                    htmlFor={checkboxId}
                    className={cn(
                        "ml-2 cursor-pointer text-sm font-medium transition-colors",
                        disabled && "cursor-not-allowed opacity-50",
                        labelClassName
                    )}
                >
                    {label}
                </Label>
            )}
        </div>
    );
}
