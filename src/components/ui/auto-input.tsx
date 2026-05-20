import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { Loader2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { LazyMotion, domAnimation, m } from "framer-motion";

export interface AutoInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    onValueChange?: (value: string) => void;
    isLoading?: boolean;
    debounceMs?: number;
    containerClassName?: string;
    ref?: React.Ref<HTMLInputElement>;
}

export function AutoInput({
    onValueChange,
    isLoading = false,
    className,
    containerClassName,
    value: propValue,
    onChange,
    debounceMs = 150,
    ref,
    ...props
}: AutoInputProps) {
    const [value, setValue] = useState(propValue || "");
    const [isFocused, setIsFocused] = useState(false);

    // Keep latest onValueChange callback in a ref so the debounce effect
    // doesn't re-fire when the parent passes a fresh function each render.
    const onValueChangeRef = useRef(onValueChange);
    useEffect(() => {
        onValueChangeRef.current = onValueChange;
    }, [onValueChange]);

    useEffect(() => {
        if (propValue === undefined || propValue === value) return;

        setValue(propValue);
    }, [propValue]);

    useEffect(() => {
        if (value === propValue) return;

        const timer = setTimeout(() => {
            onValueChangeRef.current?.(value as string);
        }, debounceMs);

        return () => clearTimeout(timer);
    }, [value, debounceMs, propValue]);

    const updateValueAndNotify = (e: React.ChangeEvent<HTMLInputElement>) => {
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
                onChange={updateValueAndNotify}
                onFocus={() => setIsFocused(true)}
                onBlur={() => setIsFocused(false)}
                {...props}
            />
            {isLoading && (
                <LazyMotion features={domAnimation}>
                    <m.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        className="absolute right-2 top-1/2 -translate-y-1/2"
                    >
                        <Loader2 className="size-4 animate-spin text-primary/70" />
                    </m.div>
                </LazyMotion>
            )}
        </div>
    );
}
