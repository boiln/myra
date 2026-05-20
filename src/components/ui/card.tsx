import * as React from "react";

import { cn } from "@/lib/utils";

function Card({ className, ref, ...props }: React.ComponentProps<"div">) {
    return (
        <div
            ref={ref}
            className={cn(
                "rounded-lg border bg-card text-card-foreground shadow-sm transition-all hover:shadow-md",
                className
            )}
            {...props}
        />
    );
}

function CardHeader({ className, ref, ...props }: React.ComponentProps<"div">) {
    return <div ref={ref} className={cn("flex flex-col gap-1.5 p-6", className)} {...props} />;
}

function CardTitle({ className, ref, ...props }: React.ComponentProps<"div">) {
    return (
        <div
            ref={ref}
            className={cn("text-2xl font-semibold leading-none tracking-tight", className)}
            {...props}
        />
    );
}

function CardDescription({ className, ref, ...props }: React.ComponentProps<"div">) {
    return <div ref={ref} className={cn("text-sm text-muted-foreground", className)} {...props} />;
}

function CardContent({ className, ref, ...props }: React.ComponentProps<"div">) {
    return <div ref={ref} className={cn("p-6 pt-0", className)} {...props} />;
}

function CardFooter({ className, ref, ...props }: React.ComponentProps<"div">) {
    return <div ref={ref} className={cn("flex items-center p-6 pt-0", className)} {...props} />;
}

export { Card, CardHeader, CardFooter, CardTitle, CardDescription, CardContent };
