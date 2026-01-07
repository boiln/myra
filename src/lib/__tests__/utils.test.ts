import { describe, it, expect } from "vitest";
import { cn } from "../utils";

describe("cn utility", () => {
    it("should merge class names", () => {
        const result = cn("foo", "bar");
        expect(result).toBe("foo bar");
    });

    it("should handle conditional classes", () => {
        const isActive = true;
        const result = cn("base", isActive && "active");
        expect(result).toBe("base active");
    });

    it("should exclude falsy values", () => {
        const result = cn("base", false && "excluded", null, undefined, "included");
        expect(result).toBe("base included");
    });

    it("should merge tailwind classes correctly", () => {
        // twMerge should override conflicting classes
        const result = cn("p-4", "p-2");
        expect(result).toBe("p-2");
    });

    it("should handle object syntax", () => {
        const result = cn({
            "text-red-500": true,
            "text-blue-500": false,
        });
        expect(result).toBe("text-red-500");
    });

    it("should handle array syntax", () => {
        const result = cn(["foo", "bar"]);
        expect(result).toBe("foo bar");
    });

    it("should handle mixed syntax", () => {
        const result = cn("base", ["array1", "array2"], { conditional: true });
        expect(result).toBe("base array1 array2 conditional");
    });

    it("should handle empty input", () => {
        const result = cn();
        expect(result).toBe("");
    });
});
