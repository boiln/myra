import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "@/styles/globals.css";
import "@/styles/blue.css";
import "@/styles/red.css";
import "@/styles/green.css";
import "@/styles/dark.css";
import "@/styles/light.css";
import { ThemeProvider } from "@/providers/theme-provider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <ThemeProvider>
            <App />
        </ThemeProvider>
    </React.StrictMode>
);
