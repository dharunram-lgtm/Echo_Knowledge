import React, { useEffect, useRef } from "react";
import { AlertTriangle, Loader2, Trash2, X } from "lucide-react";

export interface DeleteConfirmModalProps {
  isOpen: boolean;
  filename: string;
  onConfirm: () => Promise<void> | void;
  onCancel: () => void;
  isDeleting: boolean;
  error?: string | null;
}

export const DeleteConfirmModal: React.FC<DeleteConfirmModalProps> = ({
  isOpen,
  filename,
  onConfirm,
  onCancel,
  isDeleting,
  error,
}) => {
  const modalRef = useRef<HTMLDivElement>(null);
  const cancelBtnRef = useRef<HTMLButtonElement>(null);

  // Focus trap and keyboard handlers
  useEffect(() => {
    if (!isOpen) return;

    // Focus cancel button by default for safety
    const timer = setTimeout(() => {
      cancelBtnRef.current?.focus();
    }, 50);

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (!isDeleting) {
          e.preventDefault();
          onCancel();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      clearTimeout(timer);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, isDeleting, onCancel]);

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="delete-dialog-title"
      aria-describedby="delete-dialog-desc"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-150"
      onClick={(e) => {
        // Backdrop click dismisses only if clicking the overlay itself, not inside modal
        if (e.target === e.currentTarget && !isDeleting) {
          onCancel();
        }
      }}
    >
      <div
        ref={modalRef}
        className="flex w-full max-w-md flex-col gap-4 rounded-[var(--radius-lg)] border border-[var(--status-danger)]/40 bg-[var(--bg-elevated)] p-6 shadow-[var(--shadow-dialog)]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3">
            <span
              className="flex h-10 w-10 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--status-danger)]/15 text-[var(--status-danger)]"
              aria-hidden="true"
            >
              <AlertTriangle className="h-5 w-5" />
            </span>
            <div className="space-y-1">
              <h2
                id="delete-dialog-title"
                className="text-base font-semibold text-[var(--text-primary)]"
              >
                Delete document permanently?
              </h2>
              <p
                id="delete-dialog-desc"
                className="text-xs text-[var(--text-secondary)] leading-relaxed"
              >
                Are you sure you want to delete{" "}
                <strong className="text-[var(--text-primary)] font-medium">"{filename}"</strong>? This
                action permanently removes the document from local storage, extracted text, chunks and
                embeddings.
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onCancel}
            disabled={isDeleting}
            className="rounded p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-primary)] disabled:opacity-50 disabled:cursor-not-allowed"
            aria-label="Close"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="rounded-[var(--radius-md)] bg-[var(--status-danger)]/10 border border-[var(--status-danger)]/20 p-3 text-[11px] text-[var(--status-danger)] space-y-1">
          <p className="font-semibold">⚠️ Permanent destructive action:</p>
          <ul className="list-disc list-inside space-y-0.5 text-[10.5px]">
            <li>Removes the stored copy and parsed plain text.</li>
            <li>Deletes vector embeddings and semantic search indexing data.</li>
            <li>Removes document memberships across all collections.</li>
          </ul>
        </div>

        {error && (
          <div
            role="alert"
            className="rounded-[var(--radius-md)] bg-[var(--status-danger)]/15 border border-[var(--status-danger)]/30 p-2.5 text-xs text-[var(--status-danger)]"
          >
            {error}
          </div>
        )}

        <div className="flex items-center justify-end gap-2.5 pt-2">
          <button
            ref={cancelBtnRef}
            type="button"
            onClick={onCancel}
            disabled={isDeleting}
            className="fluent-button text-xs py-1.5 px-3.5 disabled:cursor-not-allowed disabled:opacity-60"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => void onConfirm()}
            disabled={isDeleting}
            className="fluent-button text-xs py-1.5 px-3.5 font-medium text-[var(--accent-foreground)] bg-[var(--status-danger)] border border-[var(--status-danger)] hover:opacity-90 focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--status-danger)] disabled:cursor-not-allowed disabled:opacity-60"
          >
            {isDeleting ? (
              <>
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                <span>Deleting…</span>
              </>
            ) : (
              <>
                <Trash2 className="h-3.5 w-3.5" />
                <span>Permanently Delete</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
