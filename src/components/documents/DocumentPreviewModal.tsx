import React, { useState } from "react";
import {
  X,
  FileText,
  Copy,
  Check,
  Calendar,
  HardDrive,
  Hash,
  FolderOpen,
  Info,
} from "lucide-react";
import type { DocumentRecord } from "@/types/documents";
import { formatBytes } from "@/utils";

interface DocumentPreviewModalProps {
  document: DocumentRecord | null;
  isOpen: boolean;
  onClose: () => void;
}

export const DocumentPreviewModal: React.FC<DocumentPreviewModalProps> = ({
  document,
  isOpen,
  onClose,
}) => {
  const [copied, setCopied] = useState(false);

  if (!isOpen || !document) return null;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(document.extracted_text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy text:", err);
    }
  };

  const isScannedWarning = document.extracted_text.includes(
    "Scanned documents requiring OCR are not yet supported"
  );

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="preview-document-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-150"
    >
      <div className="fluent-card w-full max-w-3xl max-h-[88vh] flex flex-col bg-[var(--bg-elevated)] border border-[var(--border-strong)] shadow-[var(--shadow-dialog)] rounded-[var(--radius-xl)] overflow-hidden">
        {/* Modal Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--border-subtle)] bg-[var(--bg-card)]">
          <div className="flex items-center gap-3 min-w-0">
            <div className="w-9 h-9 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-default)] flex items-center justify-center text-[var(--accent-primary)] shrink-0">
              <FileText className="w-5 h-5" />
            </div>
            <div className="min-w-0">
              <h2 id="preview-document-title" className="text-sm font-semibold text-[var(--text-primary)] truncate">
                {document.filename}
              </h2>
              <div className="flex items-center gap-2 text-[11px] text-[var(--text-tertiary)]">
                <span className="uppercase font-mono font-medium text-[var(--accent-primary)]">
                  {document.file_type}
                </span>
                <span>•</span>
                <span>{formatBytes(document.file_size)}</span>
                <span>•</span>
                <span>{document.extracted_text.length.toLocaleString()} characters extracted</span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleCopy}
              className="fluent-button text-xs py-1 px-2.5"
              title="Copy extracted text to clipboard"
            >
              {copied ? (
                <>
                  <Check className="w-3.5 h-3.5 text-[var(--status-success)]" />
                  <span className="text-[var(--status-success)]">Copied</span>
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5" />
                  <span>Copy Text</span>
                </>
              )}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="fluent-button-subtle p-1.5 rounded-[var(--radius-md)] cursor-pointer text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              aria-label="Close preview"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Metadata Strip */}
        <div className="px-6 py-2.5 bg-[var(--bg-subtle)] border-b border-[var(--border-subtle)] text-[11px] grid grid-cols-1 sm:grid-cols-2 gap-2 text-[var(--text-secondary)]">
          <div className="flex items-center gap-1.5 truncate" title={document.original_path}>
            <FolderOpen className="w-3.5 h-3.5 text-[var(--text-tertiary)] shrink-0" />
            <span className="truncate">Original: {document.original_path}</span>
          </div>
          <div className="flex items-center gap-1.5 truncate">
            <Calendar className="w-3.5 h-3.5 text-[var(--text-tertiary)] shrink-0" />
            <span>Imported: {new Date(document.imported_at).toLocaleString()}</span>
          </div>
          <div className="flex items-center gap-1.5 truncate" title={document.content_hash}>
            <Hash className="w-3.5 h-3.5 text-[var(--text-tertiary)] shrink-0" />
            <span className="font-mono text-[10px] truncate">SHA-256: {document.content_hash}</span>
          </div>
          <div className="flex items-center gap-1.5 truncate" title={document.stored_path}>
            <HardDrive className="w-3.5 h-3.5 text-[var(--text-tertiary)] shrink-0" />
            <span className="truncate">Managed: Local ECHO Vault</span>
          </div>
        </div>

        {/* OCR / Scanned Warning Notice if applicable */}
        {isScannedWarning && (
          <div className="mx-6 mt-4 p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-strong)] flex items-start gap-2.5 text-xs text-[var(--text-secondary)]">
            <Info className="w-4 h-4 text-[var(--status-warning)] shrink-0 mt-0.5" />
            <div>
              <strong className="text-[var(--text-primary)]">Scanned Document Detected:</strong>
              <p className="text-[11px] text-[var(--text-tertiary)] mt-0.5">
                This PDF contains bitmap images without an embedded text layer. Optical Character Recognition (OCR) is scheduled for a future release; text has not been invented.
              </p>
            </div>
          </div>
        )}

        {/* Extracted Text Content Viewport */}
        <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-app)]">
          <div className="fluent-card p-5 bg-[var(--bg-card)]">
            <pre className="whitespace-pre-wrap font-sans text-xs leading-relaxed text-[var(--text-primary)] select-text break-words">
              {document.extracted_text || (
                <span className="text-[var(--text-tertiary)] italic">
                  No text content could be extracted from this document.
                </span>
              )}
            </pre>
          </div>
        </div>

        {/* Modal Footer */}
        <div className="px-6 py-3 border-t border-[var(--border-subtle)] bg-[var(--bg-card)] flex items-center justify-between text-xs">
          <span className="text-[11px] text-[var(--text-tertiary)]">
            Persisted securely in local SQLite database
          </span>
          <button
            type="button"
            onClick={onClose}
            className="fluent-button text-xs py-1 px-4"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default DocumentPreviewModal;

