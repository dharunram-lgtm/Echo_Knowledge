import React from "react";
import { Sun, Moon, HardDrive, ShieldCheck } from "lucide-react";
import { useTheme } from "@/hooks/useTheme";
import type { AppRoute } from "@/types/navigation";
import { GlobalSearch } from "./GlobalSearch";

interface TitleBarProps {
  currentRoute: AppRoute;
  title: string;
}

export const TitleBar: React.FC<TitleBarProps> = ({ title }) => {
  const { resolvedTheme, toggleTheme } = useTheme();

  return (
    <header className="relative z-30 h-11 shrink-0 flex items-center justify-between px-4 border-b border-[var(--border-subtle)] bg-[var(--bg-titlebar)] select-none">
      {/* Page Title & Breadcrumb */}
      <div className="flex items-center gap-2 min-w-0">
        <span className="text-xs font-semibold text-[var(--text-primary)] tracking-tight">
          {title}
        </span>
        <span className="text-[10px] text-[var(--text-tertiary)] font-mono">
          / local-mode
        </span>
      </div>

      {/* Global Quick Search - Connected to SQLite backend */}
      <div className="hidden sm:flex items-center max-w-md w-full mx-4 select-text">
        <GlobalSearch />
      </div>

      {/* Action Controls & Indicators */}
          <div className="flex items-center gap-2 shrink-0">
            <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-[var(--radius-sm)] bg-[var(--bg-subtle)] text-[10px] text-[var(--text-secondary)] border border-[var(--border-subtle)]">
              <HardDrive className="w-3 h-3 text-[var(--accent-primary)]" />
              <span>Local Storage</span>
            </div>

            <div className="hidden md:flex items-center gap-1.5 px-2 py-0.5 rounded-[var(--radius-sm)] bg-[var(--bg-subtle)] text-[10px] text-[var(--text-secondary)] border border-[var(--border-subtle)]">
              <ShieldCheck className="w-3 h-3 text-[var(--status-success)]" />
              <span>Zero Telemetry</span>
            </div>

            {/* Theme Mode Toggle */}
            <button
              type="button"
              onClick={toggleTheme}
              className="fluent-button-subtle p-1.5 rounded-[var(--radius-md)] cursor-pointer"
              title={`Switch to ${resolvedTheme === "dark" ? "light" : "dark"} mode`}
              aria-label="Toggle theme"
            >
              {resolvedTheme === "dark" ? (
                <Sun className="w-4 h-4 text-[var(--text-secondary)]" />
              ) : (
                <Moon className="w-4 h-4 text-[var(--text-secondary)]" />
              )}
            </button>
          </div>
      </header>
    );
  };

  export default TitleBar;

