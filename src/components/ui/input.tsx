import * as React from "react";

import { cn } from "@/lib/utils";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    ref?: React.Ref<HTMLInputElement>;
}

function Input({ className, type, ref, ...props }: InputProps) {

    return (
        <input
            ref={ref}
            type={type}
            className={cn(
                "flex h-10 w-full rounded-md border border-input/50 bg-background px-3 py-2 text-sm ring-offset-background",
                "file:border-0 file:bg-transparent file:text-sm file:font-medium",
                "placeholder:text-muted-foreground",
                "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus-visible:ring-offset-0",
                "disabled:cursor-not-allowed disabled:opacity-50",
                "transition-colors duration-200 hover:border-primary/50",
                className
            )}
            {...props}
        />
    );

}

export { Input };
