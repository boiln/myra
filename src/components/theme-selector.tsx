import { Palette } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { useTheme } from "@/providers/theme-provider";
import { themes } from "@/types/theme";

export function ThemeSelector() {
    const { theme, setTheme } = useTheme();

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 rounded-full hover:bg-accent/20 focus-visible:ring-0 focus-visible:ring-offset-0"
                >
                    <Palette className="h-4 w-4 text-accent" />
                    <span className="sr-only">Toggle theme</span>
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-40 bg-card/80 backdrop-blur-md">
                {themes.map((t) => (
                    <DropdownMenuItem
                        key={t.id}
                        onClick={() => setTheme(t.id)}
                        className={cn(
                            "group flex items-center gap-3 rounded-sm px-2 py-1.5 text-sm text-foreground transition-colors",
                            theme === t.id ? "bg-accent/20" : "hover:bg-accent/10"
                        )}
                    >
                        <div
                            className={cn(
                                "h-3 w-3 rounded-full transition-all duration-300",
                                theme === t.id
                                    ? "scale-110 shadow-[0_0_8px_var(--glow-color)]"
                                    : "group-hover:scale-105 group-hover:shadow-[0_0_5px_var(--glow-color)]"
                            )}
                            style={
                                {
                                    background: t.color,
                                    "--glow-color": t.color,
                                } as React.CSSProperties
                            }
                        />
                        <span className={cn(theme === t.id ? "font-medium" : "font-normal")}>
                            {t.name}
                        </span>
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
