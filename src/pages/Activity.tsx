import React from "react";
import { Activity as ActivityIcon, Clock, CheckCircle2, ShieldCheck } from "lucide-react";

export const Activity: React.FC = () => {
  return (
    <div className="p-6 space-y-6 max-w-6xl mx-auto">
      {/* Header */}
      <div className="pb-2 border-b border-[var(--border-subtle)]">
        <h1 className="text-xl font-semibold tracking-tight text-[var(--text-primary)]">
          System Activity & Audit Log
        </h1>
        <p className="text-xs text-[var(--text-secondary)]">
          Audit trail of indexing, AI query sessions, and synchronization operations.
        </p>
      </div>

      {/* Activity Timeline Structural Placeholder */}
      <div className="fluent-card p-5 space-y-4">
        <div className="flex items-center gap-2">
          <Clock className="w-4 h-4 text-[var(--text-secondary)]" />
          <h2 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
            Recent System Events
          </h2>
        </div>

        <div className="space-y-3">
          <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] flex items-start gap-3">
            <CheckCircle2 className="w-4 h-4 text-[var(--status-success)] shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between text-xs font-medium text-[var(--text-primary)]">
                <span>ECHO Desktop Runtime Initialized</span>
                <span className="text-[10px] text-[var(--text-tertiary)] font-mono">Current Session</span>
              </div>
              <p className="text-[11px] text-[var(--text-secondary)] mt-0.5">
                Core shell loaded, IPC communication channels established with safe capability profile.
              </p>
            </div>
          </div>

          <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] flex items-start gap-3 opacity-80">
            <ShieldCheck className="w-4 h-4 text-[var(--accent-primary)] shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between text-xs font-medium text-[var(--text-primary)]">
                <span>Security Boundaries Enforced</span>
                <span className="text-[10px] text-[var(--text-tertiary)] font-mono">System Startup</span>
              </div>
              <p className="text-[11px] text-[var(--text-secondary)] mt-0.5">
                Restricted capability schema active. Arbitrary shell and raw disk access blocked.
              </p>
            </div>
          </div>
        </div>
      </div>

      <div className="flex items-center justify-center p-6 text-[11px] text-[var(--text-tertiary)]">
        <ActivityIcon className="w-3.5 h-3.5 mr-1.5" />
        <span>Granular event logging will capture vector indexing and sync transactions in future phases.</span>
      </div>
    </div>
  );
};

export default Activity;

