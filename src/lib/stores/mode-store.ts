import { create } from "zustand";

export type ManipulationMode = "standard" | "classic";

interface ModeStore {

    mode: ManipulationMode;
    setMode: (mode: ManipulationMode) => void;

}

export const useModeStore = create<ModeStore>()((set) => ({
    mode: "standard",
    setMode: (mode) => set({ mode }),
}));
