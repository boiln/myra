export const themes = [
    {
        id: "blue",
        name: "Blue",
        color: "hsl(230 80% 70%)",
        bgColor: "rgba(47, 53, 128, 0.3)",
    },
    {
        id: "red",
        name: "Red",
        color: "hsl(0 80% 70%)",
        bgColor:
            "radial-gradient(circle at 15% 15%, rgba(255, 100, 100, 0.15) 0%, transparent 40%), radial-gradient(circle at 85% 85%, rgba(255, 100, 100, 0.15) 0%, transparent 40%), rgba(128, 47, 47, 0.3)",
    },
    {
        id: "green",
        name: "Green",
        color: "hsl(160 80% 70%)",
        bgColor: "rgba(47, 128, 98, 0.3)",
    },
    {
        id: "dark",
        name: "Dark",
        color: "hsl(220 15% 20%)",
        bgColor: "rgba(25, 28, 41, 0.3)",
    },
    {
        id: "light",
        name: "Light",
        color: "hsl(0 0% 90%)",
        bgColor: "rgba(255, 255, 255, 0.35)",
    },
] as const;

export type ThemeId = (typeof themes)[number]["id"];
export type Theme = (typeof themes)[number];
