import React from "react";
import { CheckCircle2, Shield, CloudOff, Cpu } from "lucide-react";
import { useAppInfo } from "@/hooks/useAppInfo";

export const StatusBar: React.FC = () => {
  const appInfo = useAppInfo();

  return (
    <footer className="h-6 shrink-0 flex items-center justify-between px-3 text-[11px] border-t border-[var(--border-subtle)] bg-[var(--bg-statusbar)] text-[var(--text-tertiary)] select-none">
      <div className="flex items-center gap-4">
        {/* Readiness Indicator */}
        <div className="flex items-center gap-1.5 text-[var(--status-success)]">
          <CheckCircle2 className="w-3 h-3" />
          <span className="font-medium text-[var(--text-secondary)]">ECHO Ready</span>
        </div>

        {/* Phase Indicator */}
        <div className="hidden sm:flex items-center gap-1">
          <span className="text-[var(--text-secondary)]">{appInfo.phase}</span>
        </div>

        {/* Sandbox Indicator */}
        <div className="hidden md:flex items-center gap-1">
          <Shield className="w-3 h-3 text-[var(--text-tertiary)]" />
          <span>Restricted Sandbox</span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        {/* Cloud Sync State */}
        <div className="flex items-center gap-1">
          <CloudOff className="w-3 h-3 text-[var(--text-tertiary)]" />
          <span>Drive Sync: Unconfigured</span>
        </div>

        {/* AI Provider State */}
        <div className="hidden sm:flex items-center gap-1">
          <Cpu className="w-3 h-3 text-[var(--text-tertiary)]" />
          <span>Provider: None</span>
        </div>

        {/* Platform Info */}
        <div className="font-mono text-[10px] text-[var(--text-tertiary)]">
          v{appInfo.version} ({appInfo.platform})
        </div>
      </div>
    </footer>
  );
};

export default StatusBar;

