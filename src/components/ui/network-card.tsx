import { cn } from "@/lib/utils";
import { motion, HTMLMotionProps } from "framer-motion";
import React, { useState } from "react";

interface NetworkCardProps extends Omit<
    HTMLMotionProps<"div">,
    "initial" | "animate" | "transition" | "whileTap"
> {
    isActive?: boolean;
    isFeatured?: boolean;
    children: React.ReactNode;
}

export const NetworkCard = React.forwardRef<HTMLDivElement, NetworkCardProps>(
    ({ isActive = false, isFeatured = false, children, className, ...props }, ref) => {
        const [isHovered, setIsHovered] = useState(false);

        return (
            <motion.div
                ref={ref}
                className={cn(
                    "relative rounded-lg border border-border/30 bg-background/60 p-3",
                    "backdrop-blur-[2px] transition-all duration-150",
                    isActive && "border-primary/40 bg-primary/5",
                    isFeatured && !isActive && "border-border/50",
                    isHovered && !isActive && "border-border/60 bg-background/70",
                    className
                )}
                initial={{ opacity: 0.9, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.2 }}
                onMouseEnter={() => setIsHovered(true)}
                onMouseLeave={() => setIsHovered(false)}
                whileTap={{ scale: 0.995 }}
                {...props}
            >
                {isActive && (
                    <motion.div
                        className="absolute inset-0 rounded-lg"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        transition={{ duration: 0.3 }}
                    >
                        <div className="absolute inset-0 rounded-lg bg-gradient-to-r from-primary/10 to-transparent opacity-30" />
                        <div className="absolute inset-0 rounded-lg shadow-[inset_0_0_0_1px] shadow-primary/20" />
                    </motion.div>
                )}
                {children}
            </motion.div>
        );
    }
);

NetworkCard.displayName = "NetworkCard";
