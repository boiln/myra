import { Button } from "@/components/ui/button";
import { Loader2, Play, StopCircle } from "lucide-react";
import { motion } from "framer-motion";
import { useNetworkStore } from "@/lib/stores/network";
import { cn } from "@/lib/utils";
import { HotkeyBadge } from "@/components/hotkey-badge";

export function ToggleButton() {
    const { isActive, isTogglingActive, toggleActive } = useNetworkStore();

    return (
        <div className="flex items-center gap-2">
            <motion.div
                whileTap={{ scale: 0.97 }}
                animate={{
                    boxShadow: isActive
                        ? [
                              "0 0 0 rgba(225, 29, 72, 0)",
                              "0 0 10px rgba(225, 29, 72, 0.3)",
                              "0 0 0 rgba(225, 29, 72, 0)",
                          ]
                        : [
                              "0 0 0 rgba(5, 150, 105, 0)",
                              "0 0 10px rgba(5, 150, 105, 0.3)",
                              "0 0 0 rgba(5, 150, 105, 0)",
                          ],
                }}
                transition={{
                    boxShadow: {
                        repeat: Infinity,
                        duration: 2.5,
                        ease: "easeInOut",
                    },
                }}
                className="overflow-hidden rounded-md"
            >
                <Button
                    onClick={toggleActive}
                    disabled={isTogglingActive}
                    className={cn(
                        "min-w-[100px] font-medium shadow-md transition-colors duration-300",
                        isActive
                            ? "bg-rose-600 text-white shadow-rose-500/20 hover:bg-rose-700 dark:bg-rose-700 dark:shadow-rose-900/30 dark:hover:bg-rose-600"
                            : "bg-emerald-600 text-white shadow-emerald-500/20 hover:bg-emerald-700 dark:bg-emerald-700 dark:shadow-emerald-900/30 dark:hover:bg-emerald-600",
                        "border border-transparent hover:border-white/10"
                    )}
                >
                    {isTogglingActive ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                    ) : isActive ? (
                        <StopCircle className="h-4 w-4" />
                    ) : (
                        <Play className="h-4 w-4" />
                    )}
                    {isActive ? "Stop" : "Start"}
                </Button>
            </motion.div>
            <HotkeyBadge action="toggleFilter" />
        </div>
    );
}
