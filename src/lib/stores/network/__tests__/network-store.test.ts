import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useNetworkStore } from "../index";
import type { ModuleInfo } from "@/types";

vi.mock("@tauri-apps/api/core");

// Mock modules for testing
const createMockModules = (): ModuleInfo[] => [
    {
        name: "lag",
        display_name: "Lag",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 1000,
        },
    },
    {
        name: "drop",
        display_name: "Drop",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
        },
    },
    {
        name: "throttle",
        display_name: "Throttle",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
            throttle_ms: 300,
            freeze_mode: false,
        },
    },
    {
        name: "duplicate",
        display_name: "Duplicate",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
            count: 2,
        },
    },
    {
        name: "bandwidth",
        display_name: "Bandwidth",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
            limit_kbps: 100,
            use_wfp: false,
        },
    },
    {
        name: "corruption",
        display_name: "Corruption",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
        },
    },
    {
        name: "reorder",
        display_name: "Reorder",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
            throttle_ms: 100,
        },
    },
    {
        name: "burst",
        display_name: "Burst",
        enabled: false,
        config: {
            inbound: true,
            outbound: true,
            chance: 100,
            enabled: false,
            duration_ms: 0,
            buffer_ms: 0,
            keepalive_ms: 0,
            release_delay_us: 500,
            reverse: false,
        },
    },
];

describe("NetworkStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Reset store to initial state
        useNetworkStore.setState({
            isActive: false,
            filter: "outbound",
            filterTarget: { mode: "all" },
            manipulationStatus: {
                active: false,
                filter: "",
                modules: createMockModules(),
            },
            isUpdatingFilter: false,
            isTogglingActive: false,
            presets: [],
            loadingPresets: false,
            currentPreset: null,
        });
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    describe("initial state", () => {
        it("should have correct initial values", () => {
            const state = useNetworkStore.getState();
            expect(state.isActive).toBe(false);
            expect(state.filter).toBe("outbound");
            expect(state.filterTarget?.mode).toBe("all");
        });
    });

    describe("buildSettings", () => {
        it("should build settings from modules correctly", () => {
            const modules = createMockModules();
            // Enable lag module
            modules[0].enabled = true;
            modules[0].config.chance = 50;
            modules[0].config.duration_ms = 500;

            useNetworkStore.setState({
                manipulationStatus: { active: false, filter: "", modules },
            });

            const settings = useNetworkStore.getState().buildSettings();

            expect(settings.lag).toBeDefined();
            expect(settings.lag?.enabled).toBe(true);
            expect(settings.lag?.probability).toBe(0.5); // chance / 100
            expect(settings.lag?.delay_ms).toBe(500);
        });

        it("should preserve throttle freeze_mode in settings", () => {
            const modules = createMockModules();
            const throttleModule = modules.find((m) => m.name === "throttle")!;
            throttleModule.enabled = true;
            throttleModule.config.freeze_mode = true;

            useNetworkStore.setState({
                manipulationStatus: { active: false, filter: "", modules },
            });

            const settings = useNetworkStore.getState().buildSettings();

            expect(settings.throttle?.freeze_mode).toBe(true);
        });

        it("should preserve burst reverse mode in settings", () => {
            const modules = createMockModules();
            const burstModule = modules.find((m) => m.name === "burst")!;
            burstModule.enabled = true;
            burstModule.config.reverse = true;

            useNetworkStore.setState({
                manipulationStatus: { active: false, filter: "", modules },
            });

            const settings = useNetworkStore.getState().buildSettings();

            expect(settings.burst?.reverse).toBe(true);
        });

        it("should map duplicate count correctly", () => {
            const modules = createMockModules();
            const duplicateModule = modules.find((m) => m.name === "duplicate")!;
            duplicateModule.enabled = true;
            duplicateModule.config.count = 5;

            useNetworkStore.setState({
                manipulationStatus: { active: false, filter: "", modules },
            });

            const settings = useNetworkStore.getState().buildSettings();

            expect(settings.duplicate?.count).toBe(5);
        });

        it("should map bandwidth limit correctly", () => {
            const modules = createMockModules();
            const bandwidthModule = modules.find((m) => m.name === "bandwidth")!;
            bandwidthModule.enabled = true;
            bandwidthModule.config.limit_kbps = 200;
            bandwidthModule.config.use_wfp = true;

            useNetworkStore.setState({
                manipulationStatus: { active: false, filter: "", modules },
            });

            const settings = useNetworkStore.getState().buildSettings();

            expect(settings.bandwidth?.limit).toBe(200);
            expect(settings.bandwidth?.use_wfp).toBe(true);
        });
    });

    describe("setFilterTarget", () => {
        it("should update filter target", () => {
            const store = useNetworkStore.getState();
            store.setFilterTarget({ mode: "process", processId: 1234, processName: "test.exe" });

            const newState = useNetworkStore.getState();
            expect(newState.filterTarget?.mode).toBe("process");
            expect(newState.filterTarget?.processId).toBe(1234);
            expect(newState.filterTarget?.processName).toBe("test.exe");
        });
    });

    describe("updateFilter", () => {
        it("should call invoke and update filter", async () => {
            // Mock all the calls that updateFilter might make
            vi.mocked(invoke).mockImplementation(async (cmd: string) => {
                if (cmd === "update_filter") return undefined;
                if (cmd === "get_status")
                    return { running: false, modules: createMockModules() };
                return undefined;
            });

            await useNetworkStore.getState().updateFilter("tcp.DstPort == 80");

            expect(invoke).toHaveBeenCalledWith("update_filter", { filter: "tcp.DstPort == 80" });
            expect(useNetworkStore.getState().filter).toBe("tcp.DstPort == 80");
        });
    });

    describe("toggleActive", () => {
        it("should start processing when inactive", async () => {
            vi.mocked(invoke).mockImplementation(async (cmd: string) => {
                if (cmd === "start_processing") return undefined;
                if (cmd === "get_status")
                    return { running: true, modules: createMockModules() };
                return undefined;
            });

            useNetworkStore.setState({ isActive: false });
            await useNetworkStore.getState().toggleActive();

            expect(invoke).toHaveBeenCalledWith("start_processing", expect.anything());
        });

        it("should stop processing when active", async () => {
            vi.mocked(invoke).mockImplementation(async (cmd: string) => {
                if (cmd === "stop_processing") return undefined;
                if (cmd === "get_status")
                    return { running: false, modules: createMockModules() };
                return undefined;
            });

            useNetworkStore.setState({ isActive: true });
            await useNetworkStore.getState().toggleActive();

            expect(invoke).toHaveBeenCalledWith("stop_processing");
        });
    });

    describe("loadPresets", () => {
        it("should load presets from backend", async () => {
            vi.mocked(invoke).mockResolvedValue(["preset1", "preset2"]);

            await useNetworkStore.getState().loadPresets();

            expect(invoke).toHaveBeenCalledWith("list_configs");
            expect(useNetworkStore.getState().presets).toEqual(["preset1", "preset2"]);
        });
    });

    describe("deletePreset", () => {
        it("should delete preset and reload list", async () => {
            vi.mocked(invoke).mockImplementation(async (cmd: string) => {
                if (cmd === "delete_config") return undefined;
                if (cmd === "list_configs") return ["remaining-preset"];
                return undefined;
            });

            useNetworkStore.setState({
                presets: ["preset-to-delete", "remaining-preset"],
            });

            await useNetworkStore.getState().deletePreset("preset-to-delete");

            expect(invoke).toHaveBeenCalledWith("delete_config", { name: "preset-to-delete" });
            expect(invoke).toHaveBeenCalledWith("list_configs");
        });
    });
});
