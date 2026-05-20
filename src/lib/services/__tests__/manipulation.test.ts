import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ManipulationService } from "../manipulation";
import type { PacketManipulationSettings } from "@/types";

vi.mock("@tauri-apps/api/core");

describe("ManipulationService", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(invoke).mockResolvedValue(undefined);
    });

    describe("createModulesFromSettings", () => {
        it("should create lag module with correct defaults", () => {
            const settings: PacketManipulationSettings = {
                lag: {
                    enabled: true,
                    inbound: true,
                    outbound: false,
                    probability: 0.5,
                    delay_ms: 500,
                    duration_ms: 0,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const lagModule = modules.find((m) => m.name === "lag");

            expect(lagModule).toBeDefined();
            expect(lagModule?.enabled).toBe(true);
            expect(lagModule?.config.inbound).toBe(true);
            expect(lagModule?.config.outbound).toBe(false);
            expect(lagModule?.config.chance).toBe(50); // probability * 100
            expect(lagModule?.config.duration_ms).toBe(500); // delay_ms maps to duration_ms
        });

        it("should create drop module with correct config", () => {
            const settings: PacketManipulationSettings = {
                drop: {
                    enabled: true,
                    inbound: false,
                    outbound: true,
                    probability: 0.25,
                    duration_ms: 1000,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const dropModule = modules.find((m) => m.name === "drop");

            expect(dropModule).toBeDefined();
            expect(dropModule?.enabled).toBe(true);
            expect(dropModule?.config.inbound).toBe(false);
            expect(dropModule?.config.outbound).toBe(true);
            expect(dropModule?.config.chance).toBe(25);
            expect(dropModule?.config.duration_ms).toBe(1000);
        });

        it("should create throttle module with freeze_mode preserved", () => {
            const settings: PacketManipulationSettings = {
                throttle: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    duration_ms: 0,
                    throttle_ms: 500,
                    freeze_mode: true, // This should be preserved
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const throttleModule = modules.find((m) => m.name === "throttle");

            expect(throttleModule).toBeDefined();
            expect(throttleModule?.config.freeze_mode).toBe(true);
            expect(throttleModule?.config.throttle_ms).toBe(500);
        });

        it("should default freeze_mode to false when not specified", () => {
            const settings: PacketManipulationSettings = {
                throttle: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    duration_ms: 0,
                    throttle_ms: 300,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const throttleModule = modules.find((m) => m.name === "throttle");

            expect(throttleModule?.config.freeze_mode).toBe(false);
        });

        it("should create duplicate module with count parameter", () => {
            const settings: PacketManipulationSettings = {
                duplicate: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    count: 5,
                    duration_ms: 0,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const duplicateModule = modules.find((m) => m.name === "duplicate");

            expect(duplicateModule).toBeDefined();
            expect(duplicateModule?.config.count).toBe(5);
        });

        it("should create burst module with reverse option preserved", () => {
            const settings: PacketManipulationSettings = {
                burst: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    buffer_ms: 1000,
                    duration_ms: 0,
                    keepalive_ms: 500,
                    release_delay_us: 1000,
                    reverse: true, // This should be preserved
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const burstModule = modules.find((m) => m.name === "burst");

            expect(burstModule).toBeDefined();
            expect(burstModule?.config.reverse).toBe(true);
            expect(burstModule?.config.buffer_ms).toBe(1000);
            expect(burstModule?.config.keepalive_ms).toBe(500);
            expect(burstModule?.config.release_delay_us).toBe(1000);
        });

        it("should default reverse to false when not specified", () => {
            const settings: PacketManipulationSettings = {
                burst: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    buffer_ms: 1000,
                    duration_ms: 0,
                    keepalive_ms: 500,
                    release_delay_us: 1000,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const burstModule = modules.find((m) => m.name === "burst");

            expect(burstModule?.config.reverse).toBe(false);
        });

        it("should create bandwidth module with WFP option", () => {
            const settings: PacketManipulationSettings = {
                bandwidth: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    limit: 100,
                    duration_ms: 0,
                    use_wfp: true,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const bandwidthModule = modules.find((m) => m.name === "bandwidth");

            expect(bandwidthModule).toBeDefined();
            expect(bandwidthModule?.config.limit_kbps).toBe(100);
            expect(bandwidthModule?.config.use_wfp).toBe(true);
        });

        it("should create all 8 modules even with empty settings", () => {
            const settings: PacketManipulationSettings = {};

            const modules = ManipulationService.createModulesFromSettings(settings);

            expect(modules).toHaveLength(8);
            expect(modules.map((m) => m.name)).toEqual([
                "lag",
                "drop",
                "throttle",
                "duplicate",
                "bandwidth",
                "corruption",
                "reorder",
                "burst",
            ]);
        });

        it("should set all modules as disabled by default", () => {
            const settings: PacketManipulationSettings = {};

            const modules = ManipulationService.createModulesFromSettings(settings);

            modules.forEach((module) => {
                expect(module.enabled).toBe(false);
                expect(module.config.enabled).toBe(false);
            });
        });

        it("should include lag_bypass in burst module from root settings", () => {
            const settings: PacketManipulationSettings = {
                lag_bypass: true,
                burst: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    buffer_ms: 1000,
                    duration_ms: 0,
                    keepalive_ms: 500,
                    release_delay_us: 1000,
                },
            };

            const modules = ManipulationService.createModulesFromSettings(settings);
            const burstModule = modules.find((m) => m.name === "burst");

            expect(burstModule?.config.lag_bypass).toBe(true);
        });
    });

    describe("startProcessing", () => {
        it("should call invoke with correct parameters", async () => {
            const settings: PacketManipulationSettings = {
                drop: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    duration_ms: 0,
                },
            };
            const filter = "outbound";

            await ManipulationService.startProcessing(settings, filter);

            expect(invoke).toHaveBeenCalledWith("start_processing", { settings, filter });
        });

        it("should handle WFP throttle when bandwidth with WFP is enabled", async () => {
            const settings: PacketManipulationSettings = {
                bandwidth: {
                    enabled: true,
                    inbound: true,
                    outbound: true,
                    probability: 1,
                    limit: 100,
                    duration_ms: 0,
                    use_wfp: true,
                },
            };

            await ManipulationService.startProcessing(settings, "outbound");

            expect(invoke).toHaveBeenCalledWith("start_processing", expect.anything());
            expect(invoke).toHaveBeenCalledWith("start_tc_bandwidth", {
                limitKbps: 100,
                direction: "both",
            });
        });
    });

    describe("stopProcessing", () => {
        it("should call invoke to stop processing", async () => {
            await ManipulationService.stopProcessing();

            expect(invoke).toHaveBeenCalledWith("stop_processing");
        });
    });

    describe("updateFilter", () => {
        it("should call invoke with the filter", async () => {
            await ManipulationService.updateFilter("tcp.DstPort == 80");

            expect(invoke).toHaveBeenCalledWith("update_filter", { filter: "tcp.DstPort == 80" });
        });

        it("should handle null filter", async () => {
            await ManipulationService.updateFilter(null);

            expect(invoke).toHaveBeenCalledWith("update_filter", { filter: null });
        });
    });

    describe("config operations", () => {
        it("should save config with correct parameters", async () => {
            const filterTarget = {
                mode: "process" as const,
                processId: 1234,
                processName: "test.exe",
                includeInbound: true,
                includeOutbound: true,
            };
            const hotkeys = [{ action: "toggle", shortcut: "Ctrl+1", enabled: true }];
            const tap = { enabled: true, interval_ms: 100, duration_ms: 50 };

            await ManipulationService.saveConfig("test-config", filterTarget, hotkeys, tap);

            expect(invoke).toHaveBeenCalledWith("save_config", {
                name: "test-config",
                filterTarget: {
                    mode: "process",
                    process_id: 1234,
                    process_name: "test.exe",
                    device_ip: undefined,
                    device_name: undefined,
                    custom_filter: undefined,
                    include_inbound: true,
                    include_outbound: true,
                },
                hotkeys,
                tap,
            });
        });

        it("should load config by name", async () => {
            vi.mocked(invoke).mockResolvedValue({
                settings: {},
                filter: "outbound",
            });

            const result = await ManipulationService.loadConfig("test-config");

            expect(invoke).toHaveBeenCalledWith("load_config", { name: "test-config" });
            expect(result.filter).toBe("outbound");
        });

        it("should list configs", async () => {
            vi.mocked(invoke).mockResolvedValue(["config1", "config2"]);

            const result = await ManipulationService.listConfigs();

            expect(invoke).toHaveBeenCalledWith("list_configs");
            expect(result).toEqual(["config1", "config2"]);
        });

        it("should delete config by name", async () => {
            await ManipulationService.deleteConfig("test-config");

            expect(invoke).toHaveBeenCalledWith("delete_config", { name: "test-config" });
        });
    });
});
