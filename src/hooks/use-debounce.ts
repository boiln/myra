import { useRef } from "react";

export function useDebounce(callback: Function, delay: number) {
    const timeoutRef = useRef<number | null>(null);

    return (...args: any[]) => {
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }

        timeoutRef.current = window.setTimeout(() => {
            callback(...args);
        }, delay);
    };
}
