import { useState, useMemo } from "react";
import { Monitor } from "lucide-react";

interface ProcessIconProps {
    icon?: string;
    name: string;
    className?: string;
}

/**
 * Renders a process icon from base64 data or falls back to a default icon
 */
export function ProcessIcon({
    icon,
    name,
    className = "size-4",
}: ProcessIconProps) {

    const [imageError, setImageError] = useState(false);

    // Compute the image URL from the raw icon data. Pure derivation -> useMemo
    // avoids the cascading setState pattern that useEffect required.
    const imageUrl = useMemo<string | null>(() => {

        if (!icon) return null;
        // Already a PNG data URL
        if (icon.startsWith("data:image/png")) return icon;
        // Check if it's our raw format: data:image/raw;width=32;height=32;base64, ..
        if (!icon.startsWith("data:image/raw;")) return null;

        try {
            const base64Part = icon.split("base64,")[1];
            if (!base64Part) return null;
            // Decode base64 to binary
            const binaryString = atob(base64Part);
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            // Create canvas and draw RGBA data
            const canvas = document.createElement("canvas");
            canvas.width = 32;
            canvas.height = 32;
            const ctx = canvas.getContext("2d");
            if (!ctx) return null;
            const imageData = ctx.createImageData(32, 32);
            imageData.data.set(bytes);
            ctx.putImageData(imageData, 0, 0);
            return canvas.toDataURL("image/png");
        } catch (e) {
            console.error("Failed to process icon:", e);
            return null;
        }

    }, [icon]);

    if (!imageUrl || imageError) {
        return <Monitor className={className} />;
    }

    return (
        <img
            src={imageUrl}
            alt={`${name} icon`}
            className={className}
            onError={() => setImageError(true)}
            style={{ imageRendering: "pixelated" }}
        />
    );

}
