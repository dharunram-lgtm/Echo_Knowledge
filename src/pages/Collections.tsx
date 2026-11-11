/**
 * Collections Phase 1 — My Collections page.
 *
 * Layout:
 *   LEFT PANEL  — collection list, search, document counts, create button
 *   MAIN PANEL  — selected collection detail, documents, add/remove actions
 *
 * State is loaded from SQLite via Rust commands through CollectionService.
 * collections:changed events keep the view fresh after mutations.
 */

import React, {
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";
import {
  AlertCircle,
  BookOpen,
  Check,
  FilePlus2,
  FolderOpen,
  FolderPlus,
  Loader2,
  Pencil,
  Plus,
  Search,
  Trash2,
  X,
} from "lucide-react";
import { CollectionService } from "@/services/collections/collectionService";
import { DocumentService } from "@/services/documents/documentService";
import type { CollectionRecord, CollectionDocumentRecord } from "@/types/collections";
import type { DocumentSummary } from "@/types/documents";

// ─── Utility helpers ──────────────────────────────────────────────────────────

function formatError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  try { return JSON.stringify(error) ?? "Unknown error"; } catch { return "Unknown error"; }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function fileTypeLabel(ft: string): string {
  if (ft === "pdf") return "PDF";
  if (ft === "md") return "Markdown";
  if (ft === "txt") return "Text";
  return ft.toUpperCase();
}

// ─── Sub-components ───────────────────────────────────────────────────────────

interface InlineAlertProps {
  message: string;
  onDismiss?: () => void;
}
const InlineAlert: React.FC<InlineAlertProps> = ({ message, onDismiss }) => (
  <div
    role="alert"
    className="flex items-start gap-2 rounded-[var(--radius-md)] border border-[var(--status-danger)]/40 bg-[var(--status-danger)]/5 p-3 text-xs text-[var(--status-danger)]"
  >
    <AlertCircle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
    <span className="flex-1">{message}</span>
    {onDismiss && (
      <button
        type="button"
        onClick={onDismiss}
        className="shrink-0 opacity-70 hover:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-primary)]"
        aria-label="Dismiss error"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    )}
  </div>
);

interface CollectionListItemProps {
  collection: CollectionRecord;
  isSelected: boolean;
  onSelect: () => void;
}
const CollectionListItem: React.FC<CollectionListItemProps> = ({
  collection,
  isSelected,
  onSelect,
}) => (
  <button
    type="button"
    onClick={onSelect}
    aria-current={isSelected ? "true" : undefined}
    className={`flex w-full items-center justify-between gap-2 rounded-[var(--radius-md)] px-3 py-2 text-left text-sm transition-colors duration-100
      focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-primary)]
      ${
        isSelected
          ? "bg-[var(--accent-primary)]/15 font-medium text-[var(--text-primary)]"
          : "text-[var(--text-secondary)] hover:bg-[var(--bg-active)] hover:text-[var(--text-primary)]"
      }`}
  >
    <span className="flex min-w-0 items-center gap-2">
      <FolderOpen className="h-4 w-4 shrink-0 text-[var(--accent-primary)]" />
      <span className="truncate">{collection.name}</span>
    </span>
    <span className="shrink-0 rounded-full bg-[var(--bg-subtle)] px-1.5 py-0.5 text-[10px] font-medium text-[var(--text-secondary)]">
      {collection.document_count}
    </span>
  </button>
);

interface DocumentRowProps {
  doc: CollectionDocumentRecord;
  onRemove: () => void;
  isRemoving: boolean;
}
const DocumentRow: React.FC<DocumentRowProps> = ({ doc, onRemove, isRemoving }) => (
  <li className="flex items-center gap-3 rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--bg-card)] px-3 py-2.5">
    <BookOpen className="h-4 w-4 shrink-0 text-[var(--accent-primary)]" />
    <div className="min-w-0 flex-1">
      <p className="truncate text-sm font-medium text-[var(--text-primary)]">
        {doc.filename}
      </p>
      <p className="mt-0.5 text-[10px] text-[var(--text-secondary)]">
        {fileTypeLabel(doc.file_type)} · {formatBytes(doc.file_size)}
      </p>
    </div>
    <button
      type="button"
      onClick={onRemove}
      disabled={isRemoving}
      title={`Remove "${doc.filename}" from this collection`}
      aria-label={`Remove ${doc.filename} from collection`}
      className="shrink-0 rounded p-1 text-[var(--text-secondary)] opacity-0 transition-opacity duration-100 hover:text-[var(--status-danger)] focus:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--status-danger)] group-hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-40"
    >
      {isRemoving ? (
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
      ) : (
        <Trash2 className="h-3.5 w-3.5" />
      )}
    </button>
  </li>
);

// ─── Create / Edit collection form ────────────────────────────────────────────

interface CollectionFormProps {
  initial?: { name: string; description: string };
  submitLabel: string;
  onSubmit: (name: string, description: string) => Promise<void>;
  onCancel: () => void;
  isSubmitting: boolean;
  submitError: string | null;
}

const CollectionForm: React.FC<CollectionFormProps> = ({
  initial,
  submitLabel,
  onSubmit,
  onCancel,
  isSubmitting,
  submitError,
}) => {
  const nameId = useId();
  const descId = useId();
  const [name, setName] = useState(initial?.name ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const nameRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    nameRef.current?.focus();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSubmit(name, description);
  };

  return (
    <form onSubmit={(e) => void handleSubmit(e)} className="space-y-3" noValidate>
      <div>
        <label
          htmlFor={nameId}
          className="mb-1 block text-xs font-medium text-[var(--text-secondary)]"
        >
          Name <span aria-hidden="true" className="text-[var(--status-danger)]">*</span>
        </label>
        <input
          ref={nameRef}
          id={nameId}
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          maxLength={120}
          required
          disabled={isSubmitting}
          placeholder="e.g. Research Papers"
          className="w-full rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-input)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-primary)]/30 disabled:opacity-60"
        />
      </div>

      <div>
        <label
          htmlFor={descId}
          className="mb-1 block text-xs font-medium text-[var(--text-secondary)]"
        >
          Description <span className="text-[var(--text-muted)]">(optional)</span>
        </label>
        <textarea
          id={descId}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={2}
          disabled={isSubmitting}
          placeholder="What is this collection for?"
          className="w-full resize-none rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-input)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-primary)]/30 disabled:opacity-60"
        />
      </div>

      {submitError && <InlineAlert message={submitError} />}

      <div className="flex items-center justify-end gap-2 pt-1">
        <button
          type="button"
          onClick={onCancel}
          disabled={isSubmitting}
          className="fluent-button text-xs disabled:cursor-not-allowed disabled:opacity-60"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={isSubmitting || !name.trim()}
          className="fluent-button fluent-button-primary text-xs disabled:cursor-not-allowed disabled:opacity-60"
        >
          {isSubmitting ? (
            <><Loader2 className="h-3.5 w-3.5 animate-spin" /><span>Saving…</span></>
          ) : (
            <><Check className="h-3.5 w-3.5" /><span>{submitLabel}</span></>
          )}
        </button>
      </div>
    </form>
  );
};

// ─── Add documents picker ─────────────────────────────────────────────────────

interface AddDocumentsPickerProps {
  collectionId: string;
  memberIds: Set<string>;
  onAdd: (ids: string[]) => Promise<void>;
  onClose: () => void;
}

const AddDocumentsPicker: React.FC<AddDocumentsPickerProps> = ({
  collectionId: _collectionId,
  memberIds,
  onAdd,
  onClose,
}) => {
  const [documents, setDocuments] = useState<DocumentSummary[] | null>(null);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [loadError, setLoadError] = useState<string | null>(null);
  const [isAdding, setIsAdding] = useState(false);
  const [addError, setAddError] = useState<string | null>(null);
  const searchId = useId();

  useEffect(() => {
    DocumentService.getDocuments()
      .then(setDocuments)
      .catch((err: unknown) => setLoadError(formatError(err)));
  }, []);

  const filtered = (documents ?? []).filter((d) =>
    d.filename.toLowerCase().includes(search.toLowerCase())
  );

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleAdd = async () => {
    if (selected.size === 0) return;
    setIsAdding(true);
    setAddError(null);
    try {
      await onAdd([...selected]);
      onClose();
    } catch (err: unknown) {
      setAddError(formatError(err));
    } finally {
      setIsAdding(false);
    }
  };

  const availableDocs = filtered.filter((d) => !memberIds.has(d.id));

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Add documents to collection"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
    >
      <div className="flex w-full max-w-lg flex-col gap-4 rounded-[var(--radius-lg)] border border-[var(--border-default)] bg-[var(--bg-card)] p-5 shadow-[var(--shadow-elevated)]">
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-sm font-semibold text-[var(--text-primary)]">
            Add documents
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-primary)]"
            aria-label="Close"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="relative">
          <Search className="absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]" aria-hidden="true" />
          <input
            id={searchId}
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search documents…"
            aria-label="Search documents to add"
            className="w-full rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-input)] py-2 pl-9 pr-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-primary)]/30"
          />
        </div>

        <div className="max-h-72 overflow-y-auto rounded-[var(--radius-md)] border border-[var(--border-subtle)]">
          {loadError ? (
            <p className="p-4 text-center text-xs text-[var(--status-danger)]">{loadError}</p>
          ) : documents === null ? (
            <div className="flex items-center justify-center gap-2 p-6 text-xs text-[var(--text-secondary)]">
              <Loader2 className="h-4 w-4 animate-spin" /> Loading…
            </div>
          ) : availableDocs.length === 0 ? (
            <p className="p-4 text-center text-xs text-[var(--text-secondary)]">
              {documents.length === 0
                ? "No documents imported yet. Import documents in My Knowledge first."
                : search
                  ? "No documents match your search."
                  : "All documents are already in this collection."}
            </p>
          ) : (
            <ul className="divide-y divide-[var(--border-subtle)]">
              {availableDocs.map((doc) => {
                const isSel = selected.has(doc.id);
                return (
                  <li key={doc.id}>
                    <button
                      type="button"
                      onClick={() => toggle(doc.id)}
                      className={`flex w-full items-center gap-3 px-3 py-2.5 text-left text-sm transition-colors hover:bg-[var(--bg-active)] focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-[var(--accent-primary)]
                        ${isSel ? "bg-[var(--accent-primary)]/10" : ""}`}
                      aria-pressed={isSel}
                    >
                      <span
                        className={`flex h-4 w-4 shrink-0 items-center justify-center rounded border transition-colors
                          ${isSel ? "border-[var(--accent-primary)] bg-[var(--accent-primary)]" : "border-[var(--border-default)] bg-[var(--bg-input)]"}`}
                        aria-hidden="true"
                      >
                        {isSel && <Check className="h-3 w-3 text-white" />}
                      </span>
                      <span className="min-w-0 flex-1">
                        <span className="block truncate font-medium text-[var(--text-primary)]">{doc.filename}</span>
                        <span className="block text-[10px] text-[var(--text-secondary)]">
                          {fileTypeLabel(doc.file_type)} · {formatBytes(doc.file_size)}
                        </span>
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </div>

        {addError && <InlineAlert message={addError} />}

        <div className="flex items-center justify-between gap-2">
          <span className="text-xs text-[var(--text-secondary)]">
            {selected.size > 0 ? `${selected.size} selected` : ""}
          </span>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              disabled={isAdding}
              className="fluent-button text-xs disabled:cursor-not-allowed disabled:opacity-60"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => void handleAdd()}
              disabled={isAdding || selected.size === 0}
              className="fluent-button fluent-button-primary text-xs disabled:cursor-not-allowed disabled:opacity-60"
            >
              {isAdding ? (
                <><Loader2 className="h-3.5 w-3.5 animate-spin" /><span>Adding…</span></>
              ) : (
                <><Plus className="h-3.5 w-3.5" /><span>Add {selected.size > 0 ? `(${selected.size})` : ""}</span></>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

// ─── Delete confirmation dialog ───────────────────────────────────────────────

interface DeleteConfirmProps {
  collectionName: string;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
  isDeleting: boolean;
  deleteError: string | null;
}

const DeleteConfirm: React.FC<DeleteConfirmProps> = ({
  collectionName,
  onConfirm,
  onCancel,
  isDeleting,
  deleteError,
}) => (
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Confirm delete collection"
    className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
  >
    <div className="flex w-full max-w-sm flex-col gap-4 rounded-[var(--radius-lg)] border border-[var(--border-default)] bg-[var(--bg-card)] p-5 shadow-[var(--shadow-elevated)]">
      <div className="flex items-start gap-3">
        <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--status-danger)]/10 text-[var(--status-danger)]">
          <Trash2 className="h-4 w-4" />
        </span>
        <div>
          <h2 className="text-sm font-semibold text-[var(--text-primary)]">
            Delete collection
          </h2>
          <p className="mt-1 text-xs text-[var(--text-secondary)]">
            This will permanently delete <strong className="text-[var(--text-primary)]">{collectionName}</strong> and remove all its document memberships. The documents themselves will not be deleted.
          </p>
        </div>
      </div>

      {deleteError && <InlineAlert message={deleteError} />}

      <div className="flex items-center justify-end gap-2">
        <button
          type="button"
          onClick={onCancel}
          disabled={isDeleting}
          className="fluent-button text-xs disabled:cursor-not-allowed disabled:opacity-60"
        >
          Cancel
        </button>
        <button
          type="button"
          onClick={() => void onConfirm()}
          disabled={isDeleting}
          className="fluent-button text-xs font-medium text-[var(--status-danger)] ring-1 ring-inset ring-[var(--status-danger)]/40 hover:bg-[var(--status-danger)]/10 focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--status-danger)] disabled:cursor-not-allowed disabled:opacity-60"
        >
          {isDeleting ? (
            <><Loader2 className="h-3.5 w-3.5 animate-spin" /><span>Deleting…</span></>
          ) : (
            <><Trash2 className="h-3.5 w-3.5" /><span>Delete</span></>
          )}
        </button>
      </div>
    </div>
  </div>
);

// ─── Main page ────────────────────────────────────────────────────────────────

type Panel =
  | { kind: "none" }
  | { kind: "create" }
  | { kind: "edit"; collection: CollectionRecord }
  | { kind: "deleteConfirm"; collection: CollectionRecord }
  | { kind: "addDocuments" };

export const Collections: React.FC = () => {
  // ── Collection list state ──────────────────────────────────────────────────
  const [collections, setCollections] = useState<CollectionRecord[] | null>(null);
  const [listError, setListError] = useState<string | null>(null);
  const [isLoadingList, setIsLoadingList] = useState(true);
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  // ── Selected collection / documents state ──────────────────────────────────
  const [documents, setDocuments] = useState<CollectionDocumentRecord[] | null>(null);
  const [docsError, setDocsError] = useState<string | null>(null);
  const [isLoadingDocs, setIsLoadingDocs] = useState(false);

  // ── Mutation state ─────────────────────────────────────────────────────────
  const [removingDocId, setRemovingDocId] = useState<string | null>(null);
  const [panel, setPanel] = useState<Panel>({ kind: "none" });
  const [formSubmitting, setFormSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const searchId = useId();

  // ── Load all collections ───────────────────────────────────────────────────
  const loadCollections = useCallback(async () => {
    setIsLoadingList(true);
    try {
      const data = await CollectionService.listCollections();
      setCollections(data);
      setListError(null);
    } catch (err: unknown) {
      setListError(formatError(err));
    } finally {
      setIsLoadingList(false);
    }
  }, []);

  // ── Load documents for a collection ───────────────────────────────────────
  const loadDocuments = useCallback(async (id: string) => {
    setIsLoadingDocs(true);
    setDocuments(null);
    setDocsError(null);
    try {
      const data = await CollectionService.getCollectionDocuments(id);
      setDocuments(data);
    } catch (err: unknown) {
      setDocsError(formatError(err));
    } finally {
      setIsLoadingDocs(false);
    }
  }, []);

  // ── Initial load ───────────────────────────────────────────────────────────
  useEffect(() => {
    void loadCollections();
  }, [loadCollections]);

  // ── Reload documents when selection changes ────────────────────────────────
  useEffect(() => {
    if (selectedId) void loadDocuments(selectedId);
    else { setDocuments(null); setDocsError(null); }
  }, [selectedId, loadDocuments]);

  // ── collections:changed event listener ────────────────────────────────────
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    import("@tauri-apps/api/event")
      .then(({ listen }) =>
        listen("collections:changed", () => {
          void loadCollections().then(() => {
            if (selectedId) void loadDocuments(selectedId);
          });
        })
      )
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      })
      .catch(() => {});

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [loadCollections, loadDocuments, selectedId]);

  // ── Deselect if selected collection is deleted ─────────────────────────────
  useEffect(() => {
    if (!collections || selectedId === null) return;
    if (!collections.some((c) => c.id === selectedId)) {
      setSelectedId(null);
    }
  }, [collections, selectedId]);

  // ── Derived values ─────────────────────────────────────────────────────────
  const filteredCollections = (collections ?? []).filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  );

  const selectedCollection =
    selectedId !== null
      ? (collections ?? []).find((c) => c.id === selectedId) ?? null
      : null;

  const memberIds = new Set((documents ?? []).map((d) => d.id));

  // ── Mutation handlers ──────────────────────────────────────────────────────
  const handleCreate = async (name: string, description: string) => {
    setFormSubmitting(true);
    setFormError(null);
    try {
      const created = await CollectionService.createCollection({
        name,
        description: description.trim() || null,
      });
      await loadCollections();
      setSelectedId(created.id);
      setPanel({ kind: "none" });
    } catch (err: unknown) {
      setFormError(formatError(err));
    } finally {
      setFormSubmitting(false);
    }
  };

  const handleEdit = async (name: string, description: string) => {
    if (panel.kind !== "edit") return;
    setFormSubmitting(true);
    setFormError(null);
    try {
      await CollectionService.updateCollection({
        id: panel.collection.id,
        name,
        description: description.trim() || null,
      });
      await loadCollections();
      setPanel({ kind: "none" });
    } catch (err: unknown) {
      setFormError(formatError(err));
    } finally {
      setFormSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (panel.kind !== "deleteConfirm") return;
    setIsDeleting(true);
    setDeleteError(null);
    try {
      await CollectionService.deleteCollection(panel.collection.id);
      await loadCollections();
      setPanel({ kind: "none" });
    } catch (err: unknown) {
      setDeleteError(formatError(err));
    } finally {
      setIsDeleting(false);
    }
  };

  const handleAddDocuments = async (ids: string[]) => {
    if (!selectedId) return;
    await CollectionService.addDocumentsToCollection(selectedId, ids);
    await loadDocuments(selectedId);
    await loadCollections();
  };

  const handleRemoveDocument = async (docId: string) => {
    if (!selectedId) return;
    setRemovingDocId(docId);
    try {
      await CollectionService.removeDocumentsFromCollection(selectedId, [docId]);
      await loadDocuments(selectedId);
      await loadCollections();
    } catch (err: unknown) {
      console.error("Failed to remove document from collection:", err);
    } finally {
      setRemovingDocId(null);
    }
  };

  const closePanel = () => {
    setPanel({ kind: "none" });
    setFormError(null);
    setDeleteError(null);
  };

  // ── Render ─────────────────────────────────────────────────────────────────
  return (
    <>
      {/* Modals */}
      {panel.kind === "addDocuments" && selectedCollection && (
        <AddDocumentsPicker
          collectionId={selectedCollection.id}
          memberIds={memberIds}
          onAdd={handleAddDocuments}
          onClose={closePanel}
        />
      )}

      {panel.kind === "deleteConfirm" && (
        <DeleteConfirm
          collectionName={panel.collection.name}
          onConfirm={handleDelete}
          onCancel={closePanel}
          isDeleting={isDeleting}
          deleteError={deleteError}
        />
      )}

      {/* Page layout */}
      <div className="flex h-full min-h-0 overflow-hidden">
        {/* ── LEFT PANEL ── */}
        <aside
          aria-label="My Collections"
          className="flex w-64 shrink-0 flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-sidebar)]"
        >
          {/* Header */}
          <div className="flex items-center justify-between gap-2 border-b border-[var(--border-subtle)] px-4 py-3">
            <h1 className="text-sm font-semibold text-[var(--text-primary)]">
              My Collections
            </h1>
            <button
              type="button"
              onClick={() => { setPanel({ kind: "create" }); setFormError(null); }}
              title="Create a new collection"
              aria-label="Create collection"
              className="rounded p-1 text-[var(--text-secondary)] hover:bg-[var(--bg-active)] hover:text-[var(--accent-primary)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-primary)]"
            >
              <FolderPlus className="h-4 w-4" />
            </button>
          </div>

          {/* Search */}
          <div className="px-3 py-2">
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]" aria-hidden="true" />
              <input
                id={searchId}
                type="search"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search…"
                aria-label="Search collections"
                className="w-full rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--bg-input)] py-1.5 pl-8 pr-3 text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-primary)]/30"
              />
            </div>
          </div>

          {/* Create form (inline) */}
          {panel.kind === "create" && (
            <div className="border-b border-[var(--border-subtle)] bg-[var(--bg-card)] px-3 pb-3 pt-2">
              <p className="mb-2 text-xs font-semibold text-[var(--accent-primary)]">
                New collection
              </p>
              <CollectionForm
                submitLabel="Create"
                onSubmit={handleCreate}
                onCancel={closePanel}
                isSubmitting={formSubmitting}
                submitError={formError}
              />
            </div>
          )}

          {/* Collection list */}
          <nav
            aria-label="Collections list"
            className="flex-1 overflow-y-auto px-2 py-1"
          >
            {isLoadingList && collections === null ? (
              <div className="flex items-center justify-center gap-2 py-8 text-xs text-[var(--text-secondary)]">
                <Loader2 className="h-4 w-4 animate-spin" /> Loading…
              </div>
            ) : listError ? (
              <div className="p-2">
                <InlineAlert
                  message={listError}
                  onDismiss={() => { setListError(null); void loadCollections(); }}
                />
              </div>
            ) : filteredCollections.length === 0 ? (
              <div className="px-2 py-6 text-center">
                <FolderOpen className="mx-auto h-8 w-8 text-[var(--text-muted)]" />
                <p className="mt-2 text-xs text-[var(--text-secondary)]">
                  {search
                    ? "No collections match your search."
                    : "No collections yet. Create your first one above."}
                </p>
              </div>
            ) : (
              <ul className="space-y-0.5" role="listbox" aria-label="Collections">
                {filteredCollections.map((c) => (
                  <li key={c.id} role="option" aria-selected={c.id === selectedId}>
                    <CollectionListItem
                      collection={c}
                      isSelected={c.id === selectedId}
                      onSelect={() => {
                        setSelectedId(c.id);
                        closePanel();
                      }}
                    />
                  </li>
                ))}
              </ul>
            )}
          </nav>

          {/* Footer count */}
          {!isLoadingList && !listError && (
            <div className="border-t border-[var(--border-subtle)] px-4 py-2">
              <p className="text-[10px] text-[var(--text-muted)]">
                {(collections ?? []).length}{" "}
                {(collections ?? []).length === 1 ? "collection" : "collections"}
              </p>
            </div>
          )}
        </aside>

        {/* ── MAIN PANEL ── */}
        <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
          {selectedCollection === null ? (
            /* Empty state — no selection */
            <div className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
              <span className="flex h-14 w-14 items-center justify-center rounded-[var(--radius-lg)] border border-[var(--border-subtle)] bg-[var(--bg-subtle)] text-[var(--text-muted)]">
                <FolderOpen className="h-7 w-7" />
              </span>
              <div className="max-w-xs space-y-1">
                <p className="text-sm font-semibold text-[var(--text-primary)]">
                  Select a collection
                </p>
                <p className="text-xs text-[var(--text-secondary)]">
                  Choose a collection from the sidebar, or create a new one to get started.
                </p>
              </div>
              <button
                type="button"
                onClick={() => { setPanel({ kind: "create" }); setFormError(null); }}
                className="fluent-button fluent-button-primary text-xs"
              >
                <FolderPlus className="h-3.5 w-3.5" />
                <span>New collection</span>
              </button>
            </div>
          ) : (
            <div className="flex h-full min-h-0 flex-col">
              {/* Collection header */}
              <header className="flex items-start gap-3 border-b border-[var(--border-subtle)] bg-[var(--bg-card)] px-6 py-4">
                {panel.kind === "edit" && panel.collection.id === selectedCollection.id ? (
                  <div className="flex-1">
                    <p className="mb-2 text-xs font-semibold text-[var(--accent-primary)]">
                      Edit collection
                    </p>
                    <CollectionForm
                      initial={{
                        name: panel.collection.name,
                        description: panel.collection.description ?? "",
                      }}
                      submitLabel="Save"
                      onSubmit={handleEdit}
                      onCancel={closePanel}
                      isSubmitting={formSubmitting}
                      submitError={formError}
                    />
                  </div>
                ) : (
                  <>
                    <div className="min-w-0 flex-1">
                      <h2 className="truncate text-base font-semibold text-[var(--text-primary)]">
                        {selectedCollection.name}
                      </h2>
                      {selectedCollection.description && (
                        <p className="mt-0.5 text-sm text-[var(--text-secondary)]">
                          {selectedCollection.description}
                        </p>
                      )}
                      <p className="mt-1.5 text-xs text-[var(--text-muted)]">
                        {selectedCollection.document_count}{" "}
                        {selectedCollection.document_count === 1 ? "document" : "documents"}
                      </p>
                    </div>
                    <div className="flex shrink-0 items-center gap-1.5">
                      <button
                        type="button"
                        onClick={() =>
                          setPanel({
                            kind: "edit",
                            collection: selectedCollection,
                          })
                        }
                        title="Rename or edit this collection"
                        aria-label="Edit collection"
                        className="fluent-button text-xs"
                      >
                        <Pencil className="h-3.5 w-3.5" />
                        <span>Edit</span>
                      </button>
                      <button
                        type="button"
                        onClick={() =>
                          setPanel({
                            kind: "deleteConfirm",
                            collection: selectedCollection,
                          })
                        }
                        title="Delete this collection"
                        aria-label="Delete collection"
                        className="fluent-button text-xs text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                        <span>Delete</span>
                      </button>
                    </div>
                  </>
                )}
              </header>

              {/* Add documents toolbar */}
              <div className="flex items-center justify-between gap-3 border-b border-[var(--border-subtle)] px-6 py-2.5">
                <p className="text-xs text-[var(--text-secondary)]">
                  {isLoadingDocs
                    ? "Loading documents…"
                    : docsError
                      ? "Failed to load documents."
                      : `${(documents ?? []).length} document${(documents ?? []).length === 1 ? "" : "s"} in this collection`}
                </p>
                <button
                  type="button"
                  onClick={() => setPanel({ kind: "addDocuments" })}
                  disabled={isLoadingDocs}
                  className="fluent-button fluent-button-primary text-xs disabled:cursor-not-allowed disabled:opacity-60"
                >
                  <FilePlus2 className="h-3.5 w-3.5" />
                  <span>Add documents</span>
                </button>
              </div>

              {/* Document list */}
              <div className="flex-1 overflow-y-auto px-6 py-4">
                {isLoadingDocs ? (
                  <div className="flex items-center justify-center gap-2 py-12 text-xs text-[var(--text-secondary)]">
                    <Loader2 className="h-4 w-4 animate-spin" /> Loading documents…
                  </div>
                ) : docsError ? (
                  <InlineAlert
                    message={docsError}
                    onDismiss={() => { setDocsError(null); void loadDocuments(selectedId!); }}
                  />
                ) : (documents ?? []).length === 0 ? (
                  <div className="flex flex-col items-center gap-3 py-16 text-center">
                    <span className="flex h-12 w-12 items-center justify-center rounded-[var(--radius-lg)] border border-[var(--border-subtle)] bg-[var(--bg-subtle)] text-[var(--text-muted)]">
                      <BookOpen className="h-6 w-6" />
                    </span>
                    <div className="max-w-xs space-y-1">
                      <p className="text-sm font-semibold text-[var(--text-primary)]">
                        No documents yet
                      </p>
                      <p className="text-xs text-[var(--text-secondary)]">
                        Add existing documents from your ECHO library to this collection.
                      </p>
                    </div>
                    <button
                      type="button"
                      onClick={() => setPanel({ kind: "addDocuments" })}
                      className="fluent-button fluent-button-primary text-xs"
                    >
                      <FilePlus2 className="h-3.5 w-3.5" />
                      <span>Add documents</span>
                    </button>
                  </div>
                ) : (
                  <ul className="group/list space-y-1.5">
                    {(documents ?? []).map((doc) => (
                      <DocumentRow
                        key={doc.id}
                        doc={doc}
                        onRemove={() => void handleRemoveDocument(doc.id)}
                        isRemoving={removingDocId === doc.id}
                      />
                    ))}
                  </ul>
                )}
              </div>
            </div>
          )}
        </main>
      </div>
    </>
  );
};

export default Collections;
