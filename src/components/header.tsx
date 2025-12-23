import { ThemeToggle } from "@/components/ui/theme-toggle";

export function Header() {
    return (
        <header className="sticky top-0 z-10 border-b border-border/40 bg-background/80 backdrop-blur-md backdrop-saturate-150 transition-colors">
            <div className="container flex h-9 items-center justify-between px-2">
                <div className="flex items-center gap-1.5">
                    <h1 className="text-base font-bold tracking-tight">Myra</h1>
                </div>
                <ThemeToggle />
            </div>
        </header>
    );
}
