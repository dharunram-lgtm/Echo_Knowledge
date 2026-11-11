import React from "react";
import { AlertCircle, X } from "lucide-react";
import type { DuplicateNotice } from "@/types/documents";

interface DuplicateNoticeModalProps {
  duplicates: DuplicateNotice[];
  isOpen: boolean;
  onClose: () => void;
}

export const DuplicateNoticeModal: React.FC<DuplicateNoticeModalProps> = ({
  duplicates,
  isOpen,
  onClose,
}) => {
  if (!isOpen || duplicates.length === 0) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-150">
      <div className="fluent-card w-full max-w-xl flex flex-col bg-[var(--bg-elevated)] border border-[var(--border-strong)] shadow-[var(--shadow-dialog)] rounded-[var(--radius-xl)] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--border-subtle)] bg-[var(--bg-card)]">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-full bg-[var(--status-warning)]/10 text-[var(--status-warning)] flex items-center justify-center">
              <AlertCircle className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-[var(--text-primary)]">
                Duplicate Documents Detected
              </h2>
              <p className="text-[11px] text-[var(--text-tertiary)]">
                ECHO uses cryptographic SHA-256 hashes to prevent redundant storage.
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="fluent-button-subtle p-1.5 rounded-[var(--radius-md)] cursor-pointer text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* List of Duplicates */}
        <div className="p-6 space-y-3 max-h-[60vh] overflow-y-auto bg-[var(--bg-app)]">
          <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
            The following {duplicates.length === 1 ? "file was" : `${duplicates.length} files were`} skipped because identical file content already exists in your local knowledge repository:
          </p>

          <div className="space-y-2">
            {duplicates.map((dup, idx) => (
              <div
                key={idx}
                className="fluent-card p-3.5 space-y-2 bg-[var(--bg-card)] border border-[var(--border-default)]"
              >
                <div className="flex items-center justify-between">
                  <span className="text-xs font-semibold text-[var(--text-primary)] truncate">
                    {dup.filename}
                  </span>
                  <span className="fluent-badge text-[10px] text-[var(--status-warning)]">
                    Duplicate Skipped
                  </span>
                </div>
                <div className="text-[11px] text-[var(--text-secondary)] space-y-1">
                  <div>
                    Already imported as:{" "}
                    <strong className="text-[var(--text-primary)]">{dup.existing_filename}</strong>
                  </div>
                  <div>
                    Imported on: {new Date(dup.imported_at).toLocaleString()}
                  </div>
                  <div className="font-mono text-[10px] text-[var(--text-tertiary)] truncate">
                    Hash: {dup.content_hash}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-[var(--border-subtle)] bg-[var(--bg-card)] flex items-center justify-end">
          <button
            type="button"
            onClick={onClose}
            className="fluent-button fluent-button-primary text-xs py-1 px-4"
          >
            Understood
          </button>
        </div>
      </div>
    </div>
  );
};

export default DuplicateNoticeModal;
