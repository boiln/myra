import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { register, unregister, isRegistered } from "@tauri-apps/plugin-global-shortcut";
import { useHotkeyStore, HotkeyBinding } from "../hotkey-store";

vi.mock("@tauri-apps/plugin-global-shortcut");

describe("HotkeyStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(register).mockResolvedValue(undefined);
        vi.mocked(unregister).mockResolvedValue(undefined);
        vi.mocked(isRegistered).mockResolvedValue(false);

        // Reset store to default state
        useHotkeyStore.setState({
            bindings: {
                toggleFilter: { action: "toggleFilter", shortcut: "F9", enabled: true },
                toggleDrop: { action: "toggleDrop", shortcut: null, enabled: false },
                toggleLag: { action: "toggleLag", shortcut: null, enabled: false },
                toggleThrottle: { action: "toggleThrottle", shortcut: null, enabled: false },
                toggleDuplicate: { action: "toggleDuplicate", shortcut: null, enabled: false },
                toggleBandwidth: { action: "toggleBandwidth", shortcut: null, enabled: false },
                toggleCorruption: { action: "toggleCorruption", shortcut: null, enabled: false },
                toggleReorder: { action: "toggleReorder", shortcut: null, enabled: false },
                toggleBurst: { action: "toggleBurst", shortcut: null, enabled: false },
            },
            isRecording: null,
        });
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    describe("initial state", () => {
        it("should have default bindings", () => {
            const state = useHotkeyStore.getState();
            expect(state.bindings.toggleFilter.shortcut).toBe("F9");
            expect(state.bindings.toggleFilter.enabled).toBe(true);
        });

        it("should not be recording by default", () => {
            const state = useHotkeyStore.getState();
            expect(state.isRecording).toBeNull();
        });

        it("should have all module toggles defined", () => {
            const state = useHotkeyStore.getState();
            expect(state.bindings.toggleDrop).toBeDefined();
            expect(state.bindings.toggleLag).toBeDefined();
            expect(state.bindings.toggleThrottle).toBeDefined();
            expect(state.bindings.toggleDuplicate).toBeDefined();
            expect(state.bindings.toggleBandwidth).toBeDefined();
            expect(state.bindings.toggleCorruption).toBeDefined();
            expect(state.bindings.toggleReorder).toBeDefined();
            expect(state.bindings.toggleBurst).toBeDefined();
        });
    });

    describe("startRecording", () => {
        it("should set isRecording to the action name", () => {
            useHotkeyStore.getState().startRecording("toggleDrop");
            expect(useHotkeyStore.getState().isRecording).toBe("toggleDrop");
        });
    });

    describe("stopRecording", () => {
        it("should clear isRecording", () => {
            useHotkeyStore.setState({ isRecording: "toggleDrop" });
            useHotkeyStore.getState().stopRecording();
            expect(useHotkeyStore.getState().isRecording).toBeNull();
        });
    });

    describe("setBinding", () => {
        it("should update binding with new shortcut", async () => {
            await useHotkeyStore.getState().setBinding("toggleDrop", "F10");

            const state = useHotkeyStore.getState();
            expect(state.bindings.toggleDrop.shortcut).toBe("F10");
            expect(state.bindings.toggleDrop.enabled).toBe(true);
        });

        it("should clear recording state after setting binding", async () => {
            useHotkeyStore.setState({ isRecording: "toggleDrop" });
            await useHotkeyStore.getState().setBinding("toggleDrop", "F10");

            expect(useHotkeyStore.getState().isRecording).toBeNull();
        });

        it("should handle clearing a shortcut", async () => {
            // First set a shortcut
            await useHotkeyStore.getState().setBinding("toggleDrop", "F10");
            // Then clear it
            await useHotkeyStore.getState().setBinding("toggleDrop", null);

            const state = useHotkeyStore.getState();
            expect(state.bindings.toggleDrop.shortcut).toBeNull();
            expect(state.bindings.toggleDrop.enabled).toBe(false);
        });

        it("should unregister old shortcut when changing to new one", async () => {
            // Register handlers first so the shortcut gets tracked
            await useHotkeyStore.getState().registerAllHotkeys({
                toggleDrop: vi.fn(),
            });

            // Set initial shortcut
            await useHotkeyStore.getState().setBinding("toggleDrop", "F10");
            vi.clearAllMocks();

            // Change to new shortcut - now F10 should be in registeredShortcuts
            await useHotkeyStore.getState().setBinding("toggleDrop", "F11");

            expect(unregister).toHaveBeenCalledWith("F10");
        });
    });

    describe("toggleBinding", () => {
        it("should toggle enabled state", async () => {
            // Set up a binding first
            useHotkeyStore.setState({
                bindings: {
                    ...useHotkeyStore.getState().bindings,
                    toggleDrop: { action: "toggleDrop", shortcut: "F10", enabled: true },
                },
            });

            await useHotkeyStore.getState().toggleBinding("toggleDrop");

            expect(useHotkeyStore.getState().bindings.toggleDrop.enabled).toBe(false);
        });

        it("should not toggle if no shortcut is set", async () => {
            await useHotkeyStore.getState().toggleBinding("toggleDrop");

            // Should remain unchanged
            expect(useHotkeyStore.getState().bindings.toggleDrop.enabled).toBe(false);
            expect(unregister).not.toHaveBeenCalled();
        });

        it("should unregister shortcut when disabling", async () => {
            useHotkeyStore.setState({
                bindings: {
                    ...useHotkeyStore.getState().bindings,
                    toggleDrop: { action: "toggleDrop", shortcut: "F10", enabled: true },
                },
            });

            await useHotkeyStore.getState().toggleBinding("toggleDrop");

            expect(unregister).toHaveBeenCalledWith("F10");
        });
    });

    describe("registerAllHotkeys", () => {
        it("should register all enabled hotkeys", async () => {
            useHotkeyStore.setState({
                bindings: {
                    toggleFilter: { action: "toggleFilter", shortcut: "F9", enabled: true },
                    toggleDrop: { action: "toggleDrop", shortcut: "F10", enabled: true },
                    toggleLag: { action: "toggleLag", shortcut: null, enabled: false },
                },
            });

            const handlers = {
                toggleFilter: vi.fn(),
                toggleDrop: vi.fn(),
            };

            await useHotkeyStore.getState().registerAllHotkeys(handlers);

            expect(register).toHaveBeenCalledWith("F9", expect.any(Function));
            expect(register).toHaveBeenCalledWith("F10", expect.any(Function));
        });

        it("should not register disabled hotkeys", async () => {
            useHotkeyStore.setState({
                bindings: {
                    toggleFilter: { action: "toggleFilter", shortcut: "F9", enabled: false },
                },
            });

            const handlers = {
                toggleFilter: vi.fn(),
            };

            await useHotkeyStore.getState().registerAllHotkeys(handlers);

            expect(register).not.toHaveBeenCalled();
        });

        it("should not register hotkeys without handlers", async () => {
            useHotkeyStore.setState({
                bindings: {
                    toggleFilter: { action: "toggleFilter", shortcut: "F9", enabled: true },
                },
            });

            await useHotkeyStore.getState().registerAllHotkeys({});

            expect(register).not.toHaveBeenCalled();
        });

        it("should skip already registered shortcuts", async () => {
            vi.mocked(isRegistered).mockResolvedValue(true);

            useHotkeyStore.setState({
                bindings: {
                    toggleFilter: { action: "toggleFilter", shortcut: "F9", enabled: true },
                },
            });

            await useHotkeyStore.getState().registerAllHotkeys({
                toggleFilter: vi.fn(),
            });

            expect(register).not.toHaveBeenCalled();
        });
    });

    describe("unregisterAllHotkeys", () => {
        it("should unregister all shortcuts", async () => {
            // First register some hotkeys
            vi.mocked(isRegistered).mockResolvedValue(false);
            useHotkeyStore.setState({
                bindings: {
                    toggleFilter: { action: "toggleFilter", shortcut: "F9", enabled: true },
                    toggleDrop: { action: "toggleDrop", shortcut: "F10", enabled: true },
                },
            });

            await useHotkeyStore.getState().registerAllHotkeys({
                toggleFilter: vi.fn(),
                toggleDrop: vi.fn(),
            });

            vi.clearAllMocks();

            await useHotkeyStore.getState().unregisterAllHotkeys();

            expect(unregister).toHaveBeenCalledWith("F9");
            expect(unregister).toHaveBeenCalledWith("F10");
        });
    });

    describe("restoreBindings", () => {
        it("should restore bindings from array", async () => {
            const bindingsToRestore = [
                { action: "toggleFilter", shortcut: "F1", enabled: true },
                { action: "toggleDrop", shortcut: "F2", enabled: true },
            ];

            await useHotkeyStore.getState().restoreBindings(bindingsToRestore);

            const state = useHotkeyStore.getState();
            expect(state.bindings.toggleFilter.shortcut).toBe("F1");
            expect(state.bindings.toggleDrop.shortcut).toBe("F2");
        });

        it("should unregister all existing hotkeys before restoring", async () => {
            // Register some hotkeys first
            vi.mocked(isRegistered).mockResolvedValue(false);
            await useHotkeyStore.getState().registerAllHotkeys({
                toggleFilter: vi.fn(),
            });

            vi.clearAllMocks();

            await useHotkeyStore
                .getState()
                .restoreBindings([{ action: "toggleFilter", shortcut: "F1", enabled: true }]);

            expect(unregister).toHaveBeenCalled();
        });
    });
});
