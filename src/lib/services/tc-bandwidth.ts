/**
 * Traffic Control (NetLimiter-style) bandwidth limiting service
 * 
 * This provides true OS-level bandwidth limiting at the socket layer,
 * operating like NetLimiter does.
 */

import { invoke } from "@tauri-apps/api/core";

export type TcDirection = "inbound" | "outbound" | "both";

export interface TcBandwidthStatus {
    active: boolean;
    limit_kbps: number;
    direction: string;
}

/**
 * Start the bandwidth limiter
 * 
 * @param limitKbps - Bandwidth limit in KB/s
 * @param direction - Direction to limit: "inbound", "outbound", or "both"
 * @returns Promise resolving to success message
 */
export async function startTcBandwidth(
    limitKbps: number,
    direction: TcDirection = "inbound"
): Promise<string> {
    return await invoke<string>("start_tc_bandwidth", {
        limitKbps,
        direction,
    });
}

/**
 * Stop the Traffic Control bandwidth limiter
 * 
 * @returns Promise resolving to success message
 */
export async function stopTcBandwidth(): Promise<string> {
    return await invoke<string>("stop_tc_bandwidth");
}

/**
 * Get the current status of the TC bandwidth limiter
 * 
 * @returns Promise resolving to current status
 */
export async function getTcBandwidthStatus(): Promise<TcBandwidthStatus> {
    return await invoke<TcBandwidthStatus>("get_tc_bandwidth_status");
}

