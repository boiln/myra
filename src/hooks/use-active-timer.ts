import { useState, useEffect, useRef } from "react";

/**
 * Custom hook that tracks elapsed time when active.
 * Returns formatted time string (MM:SS or HH:MM:SS for longer durations).
 * 
 * @param isActive - Whether the timer should be running
 * @returns Object with elapsed time in ms and formatted time string
 */
export function useActiveTimer(isActive: boolean) {
    const [elapsedMs, setElapsedMs] = useState(0);
    const startTimeRef = useRef<number | null>(null);
    const intervalRef = useRef<NodeJS.Timeout | null>(null);

    useEffect(() => {
        if (!isActive) {
            // Stop timer and reset
            if (intervalRef.current) {
                clearInterval(intervalRef.current);
                intervalRef.current = null;
            }
            startTimeRef.current = null;
            setElapsedMs(0);
            return;
        }

        // Start or resume timer
        if (startTimeRef.current === null) {
            startTimeRef.current = Date.now();
        }

        intervalRef.current = setInterval(() => {
            if (startTimeRef.current !== null) {
                setElapsedMs(Date.now() - startTimeRef.current);
            }
        }, 100); // Update every 100ms for smooth display

        return () => {
            if (intervalRef.current) {
                clearInterval(intervalRef.current);
            }
        };
    }, [isActive]);

    // Format elapsed time like Google timer (with tenths of seconds)
    const formatTime = (ms: number): string => {
        const totalSeconds = Math.floor(ms / 1000);
        const tenths = Math.floor((ms % 1000) / 100);
        const hours = Math.floor(totalSeconds / 3600);
        const minutes = Math.floor((totalSeconds % 3600) / 60);
        const seconds = totalSeconds % 60;

        if (hours > 0) {
            return `${hours}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}.${tenths}`;
        }
        return `${minutes}:${seconds.toString().padStart(2, "0")}.${tenths}`;
    };

    return {
        elapsedMs,
        formattedTime: formatTime(elapsedMs),
    };
}
