import { Component, type ErrorInfo, type ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  public override state: State = {
    hasError: false,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public override componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error in ECHO UI:", error, errorInfo);
  }

  public override render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center h-full p-8 text-center space-y-4">
          <div className="p-3 rounded-full bg-[var(--bg-subtle)] border border-[var(--border-default)] text-[var(--status-danger)]">
            <AlertTriangle className="w-8 h-8" />
          </div>
          <h2 className="text-base font-semibold text-[var(--text-primary)]">
            Something went wrong in this view
          </h2>
          <p className="text-xs text-[var(--text-secondary)] max-w-sm">
            {this.state.error?.message || "An unexpected rendering error occurred."}
          </p>
          <button
            type="button"
            onClick={() => this.setState({ hasError: false })}
            className="fluent-button fluent-button-primary text-xs"
          >
            <RefreshCw className="w-3.5 h-3.5 mr-1" />
            <span>Try Again</span>
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
