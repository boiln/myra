import { invoke } from "@tauri-apps/api/core";

/**
 * Invoke a Tauri command
 * @param command The command to invoke
 * @param args The arguments to pass to the command
 * @returns A promise that resolves to the result of the command
 */
export async function invokeCommand<T>(
    command: string,
    args: Record<string, unknown> = {}
): Promise<T> {
    try {
        return await invoke<T>(command, args);
    } catch (error) {
        console.error(`Error invoking command ${command}:`, error);
        throw error;
    }
}
