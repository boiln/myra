import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useNetworkStore } from "@/lib/stores/network";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import { useEffect } from "react";
import { toast } from "sonner";
import { create } from "zustand";
import { QuickPresets } from "./quick-presets";

interface PresetUIState {
    presetName: string;
    saveDialogOpen: boolean;
    isLoading: boolean;
    showPresetInfo: boolean;
    setPresetName: (name: string) => void;
    setSaveDialogOpen: (open: boolean) => void;
    setIsLoading: (loading: boolean) => void;
    setShowPresetInfo: (show: boolean) => void;
    resetState: () => void;
}

const usePresetUIStore = create<PresetUIState>((set) => ({
    presetName: "",
    saveDialogOpen: false,
    isLoading: false,
    showPresetInfo: false,
    setPresetName: (name) => set({ presetName: name }),
    setSaveDialogOpen: (open) => set({ saveDialogOpen: open }),
    setIsLoading: (loading) => set({ isLoading: loading }),
    setShowPresetInfo: (show) => set({ showPresetInfo: show }),
    resetState: () =>
        set({
            presetName: "",
            saveDialogOpen: false,
            isLoading: false,
            showPresetInfo: false,
        }),
}));

export function PresetManager() {
    const { loadPresets, presets, currentPreset, loadPreset, deletePreset, savePreset } =
        useNetworkStore();
    const {
        presetName,
        saveDialogOpen,
        setPresetName,
        isLoading,
        setIsLoading,
        setShowPresetInfo,
        setSaveDialogOpen,
    } = usePresetUIStore();

    useEffect(() => {
        loadPresets();
    }, [loadPresets]);

    const handleOpenSaveDialog = () => {
        setPresetName(currentPreset || "default");
    };

    const handleKeyDown = async (e: React.KeyboardEvent) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            await handleSavePreset();
        }
    };

    const handleSavePreset = async () => {
        if (!presetName.trim()) {
            toast.error("Please enter a preset name", { dismissible: true });
            return;
        }

        setIsLoading(true);

        try {
            await savePreset(presetName);
            toast.success(`Preset "${presetName}" saved`, { dismissible: true });

            await loadPresets();
            setPresetName("");
            setShowPresetInfo(true);
            setSaveDialogOpen(false);

            setTimeout(() => setShowPresetInfo(false), 5000);
        } catch (error) {
            toast.error(`Failed to save preset: ${error}`, { dismissible: true });
        } finally {
            setIsLoading(false);
        }
    };

    const handleLoadPreset = async (name: string) => {
        setIsLoading(true);

        try {
            await loadPreset(name);
            toast.success(`Preset "${name}" loaded`, { dismissible: true });
        } catch (error) {
            toast.error(`Failed to load preset: ${error}`, { dismissible: true });
        } finally {
            setIsLoading(false);
        }
    };

    const handleDeletePreset = async (name: string) => {
        if (!name) return;
        if (!confirm(`Are you sure you want to delete "${name}"? This action cannot be undone.`))
            return;

        setIsLoading(true);

        try {
            await deletePreset(name);
            toast.success(`Preset "${name}" deleted`, { dismissible: true });
            await loadPresets();
        } catch (error) {
            toast.error(`Failed to delete preset: ${error}`, { dismissible: true });
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <Card className="border-border bg-card/90">
            <CardContent className="p-2">
                <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">Configs:</span>
                    <span className="text-sm font-medium">{currentPreset || "default"}</span>
                    <div className="flex-1" />
                    <QuickPresets />
                    <Dialog open={saveDialogOpen} onOpenChange={setSaveDialogOpen}>
                        <DialogTrigger asChild>
                            <Button
                                variant="outline"
                                size="sm"
                                className="h-7 px-2"
                                disabled={isLoading}
                                onClick={handleOpenSaveDialog}
                            >
                                Save
                            </Button>
                        </DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>Save Preset</DialogTitle>
                                <DialogDescription>
                                    Save current settings as a preset.
                                </DialogDescription>
                            </DialogHeader>
                            <div className="py-3">
                                <Label htmlFor="preset-name" className="mb-1.5 block">
                                    Config Name
                                </Label>
                                <Input
                                    id="preset-name"
                                    value={presetName}
                                    onChange={(e) => setPresetName(e.target.value)}
                                    onKeyDown={handleKeyDown}
                                    placeholder="Enter config name"
                                    className="w-full border-input/50 bg-background/80 hover:border-primary/40 focus:border-primary focus:ring-1 focus:ring-primary/30"
                                    autoFocus
                                    onFocus={(e) => e.target.select()}
                                />
                            </div>
                            <DialogFooter>
                                <Button
                                    onClick={handleSavePreset}
                                    disabled={!presetName.trim() || isLoading}
                                    className="bg-secondary text-secondary-foreground hover:bg-secondary/90"
                                >
                                    {isLoading ? "Saving .." : "Save"}
                                </Button>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                variant="outline"
                                size="sm"
                                className="h-7 px-2"
                                disabled={presets.length === 0 || isLoading}
                            >
                                Load
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                            {presets.map((preset) => (
                                <DropdownMenuItem
                                    key={preset}
                                    onClick={() => handleLoadPreset(preset)}
                                >
                                    {preset}
                                </DropdownMenuItem>
                            ))}
                        </DropdownMenuContent>
                    </DropdownMenu>
                    <Button
                        variant="outline"
                        size="sm"
                        className="h-7 px-2"
                        onClick={() => handleDeletePreset(currentPreset || "")}
                        disabled={!currentPreset || isLoading}
                    >
                        Delete
                    </Button>
                </div>
            </CardContent>
        </Card>
    );
}
