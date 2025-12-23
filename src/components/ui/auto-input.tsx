import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { Loader2 } from "lucide-react";
import { useEffect, useState, forwardRef } from "react";
import { motion } from "framer-motion";

export interface AutoInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    onValueChange?: (value: string) => void;
    isLoading?: boolean;
    debounceMs?: number;
    containerClassName?: string;
}

export const AutoInput = forwardRef<HTMLInputElement, AutoInputProps>(
    (
        {
            onValueChange,
            isLoading = false,
            className,
            containerClassName,
            value: propValue,
            onChange,
            debounceMs = 150,
            ...props
        },
        ref
    ) => {
        const [value, setValue] = useState(propValue || "");
        const [isFocused, setIsFocused] = useState(false);

        useEffect(() => {
            if (propValue === undefined || propValue === value) return;

            setValue(propValue);
        }, [propValue]);

        useEffect(() => {
            if (value === propValue) return;

            const timer = setTimeout(() => {
                onValueChange?.(value as string);
            }, debounceMs);

            return () => clearTimeout(timer);
        }, [value, debounceMs, onValueChange, propValue]);

        const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
            setValue(e.target.value);
            onChange?.(e);
        };

        return (
            <div className={cn("relative", containerClassName)}>
                <Input
                    ref={ref}
                    className={cn(
                        "transition-all duration-200",
                        isFocused && "shadow-[0_0_0_1px] shadow-primary/30",
                        className
                    )}
                    value={value}
                    onChange={handleChange}
                    onFocus={() => setIsFocused(true)}
                    onBlur={() => setIsFocused(false)}
                    {...props}
                />
                {isLoading && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        className="absolute right-2 top-1/2 -translate-y-1/2"
                    >
                        <Loader2 className="h-4 w-4 animate-spin text-primary/70" />
                    </motion.div>
                )}
            </div>
        );
    }
);

AutoInput.displayName = "AutoInput";
