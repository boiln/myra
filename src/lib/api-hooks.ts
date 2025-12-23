import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import { invokeCommand } from "./tauri-api";

/**
 * Hook for making API calls to the Tauri backend
 * @param command The command to invoke
 * @param args The arguments to pass to the command
 * @param deps The dependencies to watch for changes
 * @returns An object containing the data, loading state, error, and a refetch function
 */
export function useApiCall<T>(
    command: string,
    args: Record<string, unknown> = {},
    deps: any[] = []
) {
    const [data, setData] = useState<T | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    const fetchData = useCallback(async () => {
        setLoading(true);
        try {
            const result = await invokeCommand<T>(command, args);
            setData(result);
            setError(null);
        } catch (err) {
            console.error(`Error invoking ${command}:`, err);
            setError(err instanceof Error ? err : new Error(String(err)));
        } finally {
            setLoading(false);
        }
    }, [command, ...Object.values(args), ...deps]);

    useEffect(() => {
        fetchData();
    }, [fetchData]);

    return { data, loading, error, refetch: fetchData };
}

/**
 * Hook for listening to Tauri events
 * @param event The event to listen for
 * @returns An object containing the event data
 */
export function useEventListener<T>(event: string) {
    const [data, setData] = useState<T | null>(null);

    useEffect(() => {
        let unlistenFn: () => void;

        const setupListener = async () => {
            try {
                unlistenFn = await listen<T>(event, (e) => {
                    setData(e.payload as T);
                });
            } catch (err) {
                console.error(`Error setting up event listener for ${event}:`, err);
            }
        };

        setupListener();

        return () => {
            if (unlistenFn) {
                unlistenFn();
            }
        };
    }, [event]);

    return { data };
}

/**
 * Hook for making API calls to the Tauri backend with real-time updates
 * @param command The command to invoke
 * @param eventName The event name to listen for updates
 * @param args The arguments to pass to the command
 * @returns An object containing the data, loading state, error, and a refetch function
 */
export function useRealTimeData<T>(
    command: string,
    eventName: string,
    args: Record<string, unknown> = {}
) {
    const { data: initialData, loading, error, refetch } = useApiCall<T>(command, args);
    const { data: updateData } = useEventListener<T>(eventName);
    const [data, setData] = useState<T | null>(null);

    useEffect(() => {
        if (initialData) {
            setData(initialData);
        }
    }, [initialData]);

    useEffect(() => {
        if (updateData) {
            setData(updateData);
        }
    }, [updateData]);

    return { data, loading, error, refetch };
}
