import "@/styles/globals.css";
import { Outlet } from "react-router-dom";
import { ErrorBoundary } from "@/components/error-boundary";

export default function RootLayout() {
    return (
        <div className="min-h-screen bg-background font-sans antialiased">
            <div className="relative flex min-h-screen flex-col">
                <div className="flex-1">
                    <ErrorBoundary>
                        <Outlet />
                    </ErrorBoundary>
                </div>
            </div>
        </div>
    );
}
