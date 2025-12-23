import { motion } from "framer-motion";
import { FilterControl } from "@/components/filter-control";
import { ToggleButton } from "@/components/toggle-button";

export function NetworkControls() {
    return (
        <motion.div
            initial={{ opacity: 0.9, y: 5 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.2 }}
            className="rounded-lg border border-border bg-card/90 p-2 shadow-sm backdrop-blur-sm"
        >
            <div className="flex items-center justify-between gap-3">
                <FilterControl />
                <ToggleButton />
            </div>
        </motion.div>
    );
}
