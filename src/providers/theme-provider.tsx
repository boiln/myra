import React, { createContext, useContext, useEffect, useState } from "react";

import { ThemeId, themes } from "@/types/theme";

interface ThemeContextType {
    theme: ThemeId;
    setTheme: (theme: ThemeId) => void;
}

const defaultTheme = themes[0].id;

const ThemeContext = createContext<ThemeContextType>({
    theme: defaultTheme,
    setTheme: () => null,
});

export function ThemeProvider({ children }: { children: React.ReactNode }) {
    const [theme, setTheme] = useState<ThemeId>(() => {
        // Check if theme exists in localStorage
        const savedTheme = localStorage.getItem("theme") as ThemeId | null;
        return savedTheme && themes.some((t) => t.id === savedTheme) ? savedTheme : defaultTheme;
    });

    useEffect(() => {
        document.documentElement.classList.remove(...themes.map((t) => t.id));
        document.body.classList.remove(...themes.map((t) => t.id));

        document.documentElement.setAttribute("data-theme", theme);
        document.documentElement.classList.add(theme);
        document.body.classList.add(theme);

        document.body.style.display = "none";
        document.body.offsetHeight;
        document.body.style.display = "";

        localStorage.setItem("theme", theme);
    }, [theme]);

    return <ThemeContext.Provider value={{ theme, setTheme }}>{children}</ThemeContext.Provider>;
}

export const useTheme = () => {
    const context = useContext(ThemeContext);
    if (!context) {
        throw new Error("useTheme must be used within a ThemeProvider");
    }
    return context;
};
