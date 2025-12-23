import { Component, ErrorInfo, ReactNode } from "react";
import { Button } from "@/components/ui/button";

interface Props {
    children?: ReactNode;
}

interface State {
    hasError: boolean;
    error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error("Uncaught error:", error, errorInfo);
    }

    public render() {
        if (this.state.hasError) {
            return (
                <div className="flex min-h-screen flex-col items-center justify-center gap-4">
                    <h2 className="text-xl font-semibold">Something went wrong!</h2>
                    <p className="text-sm text-muted-foreground">
                        {this.state.error?.message || "An unexpected error occurred"}
                    </p>
                    <Button variant="outline" onClick={() => this.setState({ hasError: false })}>
                        Try again
                    </Button>
                </div>
            );
        }

        return this.props.children;
    }
}
