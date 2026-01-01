import { useEffect, useCallback } from "react";
import { useHotkeyStore } from "@/lib/stores/hotkey-store";
import { cn } from "@/lib/utils";

interface HotkeyBadgeProps {
    action: string;
    className?: string;
}

export function HotkeyBadge({ action, className }: HotkeyBadgeProps) {
    const { bindings, isRecording, setBinding, startRecording, stopRecording } = useHotkeyStore();
    const binding = bindings[action];
    const isRecordingThis = isRecording === action;

    const handleKeyDown = useCallback(
        (e: KeyboardEvent) => {
            if (!isRecordingThis) return;

            e.preventDefault();
            e.stopPropagation();

            const key = normalizeKey(e.key);
            
            if (key === "Escape") {
                stopRecording();
                return;
            }
            
            if (key === "Backspace" || key === "Delete") {
                setBinding(action, null);
                return;
            }
            
            if (key === "Control" || key === "Alt" || key === "Shift" || key === "Meta") {
                return;
            }

            // F-keys don't need modifiers
            if (/^F\d+$/.test(key)) {
                setBinding(action, key);
                return;
            }

            // Build modifier prefix
            const modifier = e.ctrlKey ? "Ctrl" : e.altKey ? "Alt" : e.shiftKey ? "Shift" : null;
            
            if (modifier) {
                setBinding(action, `${modifier}+${key}`);
                return;
            }
            
            setBinding(action, key);
        },
        [isRecordingThis, action, setBinding, stopRecording]
    );

    const handleClickOutside = useCallback(
        (e: MouseEvent) => {
            if (!isRecordingThis) return;
            
            const target = e.target as HTMLElement;
            if (target.closest(`[data-hotkey-action="${action}"]`)) return;
            
            stopRecording();
        },
        [isRecordingThis, action, stopRecording]
    );

    const handleClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        
        if (isRecordingThis) {
            stopRecording();
            return;
        }
        
        startRecording(action);
    };

    useEffect(() => {
        if (!isRecordingThis) return;
        
        window.addEventListener("keydown", handleKeyDown);
        window.addEventListener("mousedown", handleClickOutside);
        
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
            window.removeEventListener("mousedown", handleClickOutside);
        };
    }, [isRecordingThis, handleKeyDown, handleClickOutside]);

    const displayText = isRecordingThis ? "..." : binding?.shortcut || "[-]";
    const title = getTitle(isRecordingThis, binding?.shortcut);
    const buttonClass = getButtonClass(isRecordingThis, binding?.shortcut);

    return (
        <button
            data-hotkey-action={action}
            onClick={handleClick}
            className={cn(
                "inline-flex items-center justify-center rounded px-1.5 py-0.5 font-mono text-[10px] transition-all",
                buttonClass,
                className
            )}
            title={title}
        >
            {displayText}
        </button>
    );
}

function normalizeKey(key: string): string {
    if (key === " ") return "Space";
    if (key.startsWith("Arrow")) return key.replace("Arrow", "");
    if (key.length === 1) return key.toUpperCase();
    return key;
}

function getTitle(isRecording: boolean, shortcut?: string | null): string {
    if (isRecording) return "Press a key (Esc to cancel, Del to clear)";
    if (shortcut) return `Hotkey: ${shortcut} (click to change)`;
    return "Click to set hotkey";
}

function getButtonClass(isRecording: boolean, shortcut?: string | null): string {
    if (isRecording) return "animate-pulse bg-primary text-primary-foreground";
    if (shortcut) return "bg-muted/80 text-muted-foreground hover:bg-muted";
    return "bg-muted/50 text-muted-foreground/60 hover:bg-muted/80 hover:text-muted-foreground";
}
