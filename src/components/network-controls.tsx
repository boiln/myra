import { motion } from "framer-motion";
import { FilterTargetSelector } from "@/components/filter-target-selector";
import { ToggleButton } from "@/components/toggle-button";

export function NetworkControls() {
    return (
        <motion.div
            initial={{ opacity: 0.9, y: 5 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.2 }}
            className="relative z-20 rounded-lg border border-border bg-card/90 p-2 shadow-sm backdrop-blur-sm"
        >
            <div className="flex items-center gap-3">
                <div className="flex-1">
                    <FilterTargetSelector />
                </div>
                <ToggleButton />
            </div>
        </motion.div>
    );
}
