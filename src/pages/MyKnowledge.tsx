import React, { useState, useEffect, useCallback, useRef, useMemo } from "react";
import {
  Files,
  Plus,
  Search,
  Trash2,
  Eye,
  FileText,
  FileCode,
  FileSpreadsheet,
  AlertCircle,
  CheckCircle2,
  RefreshCw,
  Loader2,
  X,
  Sparkles,
  Clock,
  Key,
  AlertTriangle,
  Tag,
  Filter,
  RotateCcw,
} from "lucide-react";
import type {
  DocumentRecord,
  DocumentSearchResult,
  DocumentSummary,
  DuplicateNotice,
} from "@/types/documents";
import type { IndexingOverview, DocumentIndexStatus } from "@/types/ai";
import type { DocumentTagRecord, DocumentTaggingStatusRecord, TagRecord } from "@/types/tags";
import { DocumentService } from "@/services/documents/documentService";
import { AIService } from "@/services/ai/aiService";
import { TagService } from "@/services/tags/tagService";
import { RetryService } from "@/services/ai/retryService";
import { DocumentPreviewModal } from "@/components/documents/DocumentPreviewModal";
import { DuplicateNoticeModal } from "@/components/documents/DuplicateNoticeModal";
import { DeleteConfirmModal } from "@/components/documents/DeleteConfirmModal";
import { formatBytes } from "@/utils";
import { useRouter } from "@/router";

export const MyKnowledge: React.FC = () => {
  const { previewDocumentId, clearPreviewDocumentId } = useRouter();
  const [documents, setDocuments] = useState<DocumentSummary[]>([]);
  const [searchResults, setSearchResults] = useState<DocumentSearchResult[] | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isImporting, setIsImporting] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Phase 3 Indexing State
  const [indexingOverview, setIndexingOverview] = useState<IndexingOverview | null>(null);
  const [isBatchIndexing, setIsBatchIndexing] = useState(false);
  const [indexingDocId, setIndexingDocId] = useState<string | null>(null);

  // Modals & Feedback
  const [previewDoc, setPreviewDoc] = useState<DocumentRecord | null>(null);
  const [isPreviewOpen, setIsPreviewOpen] = useState(false);
  const [duplicates, setDuplicates] = useState<DuplicateNotice[]>([]);
  const [isDuplicateModalOpen, setIsDuplicateModalOpen] = useState(false);
  const [notification, setNotification] = useState<{
    type: "success" | "error" | "info";
    message: string;
  } | null>(null);

  // Deletion state
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; filename: string } | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const deleteTriggerRef = useRef<HTMLButtonElement | null>(null);

  // Phase 3.3 Tags State
  const [documentTagsMap, setDocumentTagsMap] = useState<Record<string, DocumentTagRecord[]>>({});
  const [taggingStatusMap, setTaggingStatusMap] = useState<Record<string, DocumentTaggingStatusRecord>>({});
  const [allTags, setAllTags] = useState<TagRecord[]>([]);
  const [selectedTagFilter, setSelectedTagFilter] = useState<string | null>(null);
  const [activeTagInputDocId, setActiveTagInputDocId] = useState<string | null>(null);
  const [manualTagInput, setManualTagInput] = useState<string>("");
  const [isRetaggingDocId, setIsRetaggingDocId] = useState<string | null>(null);
  const [isRetryingTagging, setIsRetryingTagging] = useState<boolean>(false);

  const filteredDocuments = useMemo(() => {
    if (!selectedTagFilter) return documents;
    return documents.filter((doc) => {
      const docTags = documentTagsMap[doc.id] || [];
      return docTags.some((t) => t.name.toLowerCase() === selectedTagFilter.toLowerCase());
    });
  }, [documents, selectedTagFilter, documentTagsMap]);

  const failedTaggingCount = useMemo(() => {
    return Object.values(taggingStatusMap).filter((s) => s.status === "failed").length;
  }, [taggingStatusMap]);

  const fetchDocuments = useCallback(async () => {
    try {
      setIsLoading(true);
      const [docs, indexStatus, globalTags, taggingStatuses] = await Promise.all([
        DocumentService.getDocuments(),
        AIService.getIndexingStatus().catch(() => null),
        TagService.getAllTags().catch(() => []),
        TagService.getAllTaggingStatuses().catch(() => []),
      ]);
      setDocuments(docs);
      setIndexingOverview(indexStatus);
      setAllTags(globalTags);

      const statusMap: Record<string, DocumentTaggingStatusRecord> = {};
      for (const st of taggingStatuses) {
        statusMap[st.document_id] = st;
      }
      setTaggingStatusMap(statusMap);

      // Fetch tags for all current documents
      const tagsPairs = await Promise.all(
        docs.map(async (d) => [d.id, await TagService.getDocumentTags(d.id).catch(() => [])] as const)
      );
      const map: Record<string, DocumentTagRecord[]> = {};
      for (const [id, t] of tagsPairs) {
        map[id] = t;
      }
      setDocumentTagsMap(map);
    } catch (err) {
      console.error("Failed to load documents:", err);
      setNotification({
        type: "error",
        message: "Failed to connect to local database. " + String(err),
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDocuments();

    let unlistenIndexing: (() => void) | undefined;
    let unlistenTagging: (() => void) | undefined;

    import("@tauri-apps/api/event").then(({ listen }) => {
      listen("indexing:status_changed", () => {
        fetchDocuments();
      }).then((unlisten) => {
        unlistenIndexing = unlisten;
      }).catch(() => { });

      listen("tagging:status_changed", () => {
        fetchDocuments();
      }).then((unlisten) => {
        unlistenTagging = unlisten;
      }).catch(() => { });
    }).catch(() => { });

    return () => {
      if (unlistenIndexing) unlistenIndexing();
      if (unlistenTagging) unlistenTagging();
    };
  }, [fetchDocuments]);

  const handleAddManualTag = async (docId: string) => {
    const trimmed = manualTagInput.trim();
    if (!trimmed) return;

    try {
      const newTag = await TagService.addManualTag(docId, trimmed);
      setDocumentTagsMap((prev) => ({
        ...prev,
        [docId]: [...(prev[docId] || []).filter((t) => t.tag_id !== newTag.tag_id), newTag],
      }));
      setManualTagInput("");
      setActiveTagInputDocId(null);
      // Refresh global tags
      TagService.getAllTags().then(setAllTags).catch(() => { });
    } catch (err) {
      console.error("Failed to add manual tag:", err);
      setNotification({
        type: "error",
        message: "Failed to add tag: " + String(err),
      });
    }
  };

  const handleRemoveTag = async (docId: string, tagId: string) => {
    try {
      await TagService.removeDocumentTag(docId, tagId);
      setDocumentTagsMap((prev) => ({
        ...prev,
        [docId]: (prev[docId] || []).filter((t) => t.tag_id !== tagId),
      }));
      // Refresh global tags
      TagService.getAllTags().then(setAllTags).catch(() => { });
    } catch (err) {
      console.error("Failed to remove tag:", err);
      setNotification({
        type: "error",
        message: "Failed to remove tag: " + String(err),
      });
    }
  };

  const handleRetag = async (docId: string, filename: string) => {
    try {
      setIsRetaggingDocId(docId);
      setNotification(null);
      const updatedTags = await TagService.retagDocument(docId);
      setDocumentTagsMap((prev) => ({
        ...prev,
        [docId]: updatedTags,
      }));
      TagService.getAllTags().then(setAllTags).catch(() => { });
      setNotification({
        type: "success",
        message: `Successfully extracted ${updatedTags.length} AI tags for "${filename}"!`,
      });
    } catch (err) {
      console.error("Failed to re-tag document:", err);
      setNotification({
        type: "error",
        message: `Failed to generate AI tags for "${filename}": ${String(err)}`,
      });
    } finally {
      setIsRetaggingDocId(null);
    }
  };

  const handleRetryFailed = async () => {
    try {
      setIsBatchIndexing(true);
      setNotification(null);
      const res = await AIService.retryFailedIndexing();
      if (res.failed.length > 0) {
        setNotification({
          type: "error",
          message: `Retried indexing: ${res.successful.length} succeeded, but ${res.failed.length} failed.`,
        });
      } else {
        setNotification({
          type: "success",
          message: `Successfully indexed ${res.successful.length} previously failed document(s)!`,
        });
      }
      await fetchDocuments();
    } catch (err) {
      setNotification({
        type: "error",
        message: "Failed to retry indexing: " + String(err),
      });
    } finally {
      setIsBatchIndexing(false);
    }
  };

  const handleRetryTagging = async (docId: string) => {
    try {
      setNotification(null);
      await TagService.retryTaggingForDocument(docId);
      setNotification({
        type: "info",
        message: "Retrying tag extraction in background...",
      });
      await fetchDocuments();
    } catch (err) {
      setNotification({
        type: "error",
        message: "Failed to retry tagging: " + String(err),
      });
    }
  };

  const handleRetryAllFailedTagging = async () => {
    try {
      setIsRetryingTagging(true);
      setNotification(null);
      const count = await TagService.retryFailedTagging();
      setNotification({
        type: "info",
        message: `Enqueued ${count} failed tagging job(s) for background processing.`,
      });
      await fetchDocuments();
    } catch (err) {
      setNotification({
        type: "error",
        message: "Failed to retry tagging: " + String(err),
      });
    } finally {
      setIsRetryingTagging(false);
    }
  };

  const handleRetryIndexing = async (docId: string) => {
    try {
      await RetryService.retryDocumentOperation(docId, "indexing");
      await fetchDocuments();
    } catch (err) {
      console.error("Failed to retry indexing:", err);
    }
  };

  const handleCancelIndexingRetry = async (docId: string) => {
    try {
      await RetryService.cancelDocumentRetry(docId, "indexing");
      await fetchDocuments();
    } catch (err) {
      console.error("Failed to cancel indexing retry:", err);
    }
  };

  const handleCancelTaggingRetry = async (docId: string) => {
    try {
      await RetryService.cancelDocumentRetry(docId, "tagging");
      await fetchDocuments();
    } catch (err) {
      console.error("Failed to cancel tagging retry:", err);
    }
  };

  // Debounced / reactive search
  useEffect(() => {
    if (!searchQuery.trim()) {
      setSearchResults(null);
      setIsSearching(false);
      return;
    }

    const timer = setTimeout(async () => {
      try {
        setIsSearching(true);
        const results = await DocumentService.searchDocuments(searchQuery);
        setSearchResults(results);
      } catch (err) {
        console.error("Search query failed:", err);
      } finally {
        setIsSearching(false);
      }
    }, 250);

    return () => clearTimeout(timer);
  }, [searchQuery]);

  // Keyboard shortcut: Escape clears page search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && searchQuery) {
        setSearchQuery("");
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [searchQuery]);

  const handleImport = async () => {
    try {
      setIsImporting(true);
      setNotification(null);

      const result = await DocumentService.pickAndImportDocuments();
      if (!result) return;

      if (result.imported.length > 0) {
        setNotification({
          type: "success",
          message: `Successfully imported ${result.imported.length} document${result.imported.length > 1 ? "s" : ""
            } into local storage.`,
        });
        await fetchDocuments();
      }

      if (result.duplicates.length > 0) {
        setDuplicates(result.duplicates);
        setIsDuplicateModalOpen(true);
      }

      if (result.failed.length > 0) {
        const errors = result.failed.map((f) => `${f.filename}: ${f.error}`).join(" | ");
        setNotification({
          type: "error",
          message: `Some documents could not be imported: ${errors}`,
        });
      }
    } catch (err) {
      console.error("Import error:", err);
      setNotification({
        type: "error",
        message: "Failed to import selected files: " + String(err),
      });
    } finally {
      setIsImporting(false);
    }
  };

  const handlePreview = useCallback(async (id: string) => {
    try {
      const fullDoc = await DocumentService.getDocument(id);
      setPreviewDoc(fullDoc);
      setIsPreviewOpen(true);
    } catch (err) {
      console.error("Failed to retrieve document text:", err);
      setNotification({
        type: "error",
        message: "Failed to load document text: " + String(err),
      });
    }
  }, []);

  // Automatically open preview when navigating from Global Search or Citations
  useEffect(() => {
    if (previewDocumentId) {
      handlePreview(previewDocumentId);
      clearPreviewDocumentId();
    }
  }, [previewDocumentId, handlePreview, clearPreviewDocumentId]);

  const openDeleteModal = (
    id: string,
    filename: string,
    triggerElement?: HTMLButtonElement | null
  ) => {
    deleteTriggerRef.current = triggerElement || null;
    setDeleteError(null);
    setDeleteTarget({ id, filename });
  };

  const closeDeleteModal = () => {
    if (deletingId) return; // Prevent closing while in-flight
    setDeleteTarget(null);
    setDeleteError(null);
    // Return focus to the button that triggered the modal
    if (deleteTriggerRef.current) {
      deleteTriggerRef.current.focus();
      deleteTriggerRef.current = null;
    }
  };

  const handleConfirmDelete = async () => {
    if (!deleteTarget || deletingId) return;

    const { id, filename } = deleteTarget;
    try {
      setDeletingId(id);
      setDeleteError(null);
      await DocumentService.deleteDocument(id);
      setNotification({
        type: "info",
        message: `Deleted "${filename}" and its associated embeddings from local storage.`,
      });
      setDeleteTarget(null);
      deleteTriggerRef.current = null;
      await fetchDocuments();
      if (searchResults) {
        setSearchResults((prev) => (prev ? prev.filter((r) => r.id !== id) : null));
      }
    } catch (err) {
      console.error("Failed to delete document:", err);
      setDeleteError("Failed to delete document: " + String(err));
      setNotification({
        type: "error",
        message: "Failed to delete document: " + String(err),
      });
    } finally {
      setDeletingId(null);
    }
  };

  // Phase 3: Index Single Document
  const handleIndexSingle = async (documentId: string, filename: string) => {
    try {
      setIndexingDocId(documentId);
      setNotification(null);

      const result = await AIService.indexDocument(documentId);
      if (result.error) {
        setNotification({
          type: "error",
          message: `Indexing failed for "${filename}": ${result.error}`,
        });
      } else {
        setNotification({
          type: "success",
          message: `Successfully indexed "${filename}" into ${result.chunks_count} chunks and generated vector embeddings!`,
        });
      }
      await fetchDocuments();
    } catch (err) {
      setNotification({
        type: "error",
        message: `Failed to index "${filename}": ${String(err)}`,
      });
    } finally {
      setIndexingDocId(null);
    }
  };

  // Phase 3: Batch Index All Unindexed Documents
  const handleIndexAll = async () => {
    try {
      setIsBatchIndexing(true);
      setNotification(null);

      const result = await AIService.indexAllDocuments();
      const succCount = result.successful.length;
      const failCount = result.failed.length;

      if (failCount > 0) {
        const errors = result.failed.map((f) => `${f.filename}: ${f.error}`).join(" | ");
        setNotification({
          type: "error",
          message: `Indexed ${succCount} documents, but ${failCount} failed: ${errors}`,
        });
      } else {
        setNotification({
          type: "success",
          message: `Successfully indexed all ${succCount} documents! Ready for semantic retrieval and grounded Ask ECHO Q&A.`,
        });
      }
      await fetchDocuments();
    } catch (err) {
      setNotification({
        type: "error",
        message: `Batch indexing failed: ${String(err)}`,
      });
    } finally {
      setIsBatchIndexing(false);
    }
  };

  const renderFileIcon = (fileType: string) => {
    switch (fileType.toLowerCase()) {
      case "pdf":
        return <FileSpreadsheet className="w-4 h-4 text-rose-500" />;
      case "md":
        return <FileCode className="w-4 h-4 text-cyan-400" />;
      default:
        return <FileText className="w-4 h-4 text-[var(--accent-primary)]" />;
    }
  };

  const getDocIndexStatus = (docId: string) => {
    return indexingOverview?.documents.find((d) => d.document_id === docId);
  };

  const renderIndexBadge = (idxStatus?: DocumentIndexStatus) => {
    if (!idxStatus) {
      return (
        <span className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30">
          Not Indexed
        </span>
      );
    }

    if (idxStatus.retry_state === "retry_wait") {
      return (
        <div className="inline-flex items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
          <span
            className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30 cursor-help flex items-center gap-1"
            title={`Temporary indexing failure: ${idxStatus.error_message || idxStatus.last_error || "Transient error"}. Auto-retry attempt #${idxStatus.attempt_count || 1}${idxStatus.next_retry_at ? ` scheduled for ${new Date(idxStatus.next_retry_at).toLocaleTimeString()}` : ""}`}
          >
            <Clock className="w-2.5 h-2.5 animate-pulse" />
            <span>Retry Wait (#{idxStatus.attempt_count || 1})</span>
          </span>
          <button
            type="button"
            onClick={() => handleRetryIndexing(idxStatus.document_id)}
            className="text-[10px] text-[var(--accent-primary)] hover:underline cursor-pointer"
            title="Force immediate retry"
          >
            Retry Now
          </button>
          <button
            type="button"
            onClick={() => handleCancelIndexingRetry(idxStatus.document_id)}
            className="text-[10px] text-[var(--text-tertiary)] hover:text-[var(--status-danger)] cursor-pointer"
            title="Cancel auto-retry"
          >
            <X className="w-2.5 h-2.5" />
          </button>
        </div>
      );
    }

    if (idxStatus.retry_state === "paused") {
      return (
        <span className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30">
          <Clock className="w-2.5 h-2.5 inline mr-1" />
          Retries Paused
        </span>
      );
    }

    if (idxStatus.retry_state === "cancelled") {
      return (
        <div className="inline-flex items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
          <span className="fluent-badge text-[10px] bg-[var(--bg-subtle)] text-[var(--text-tertiary)] border-[var(--border-subtle)]">
            Retry Stopped
          </span>
          <button
            type="button"
            onClick={() => handleRetryIndexing(idxStatus.document_id)}
            className="text-[10px] text-[var(--accent-primary)] hover:underline cursor-pointer"
          >
            Resume
          </button>
        </div>
      );
    }

    switch (idxStatus.status) {
      case "indexed":
        return (
          <span className="fluent-badge text-[10px] bg-[var(--status-success)]/10 text-[var(--status-success)] border-[var(--status-success)]/30">
            <Sparkles className="w-2.5 h-2.5 inline mr-1" />
            Indexed ({idxStatus.chunk_count} chunk{idxStatus.chunk_count !== 1 ? "s" : ""})
          </span>
        );
      case "indexing":
        return (
          <span className="fluent-badge text-[10px] bg-blue-500/10 text-blue-500 border-blue-500/30">
            <Loader2 className="w-2.5 h-2.5 inline mr-1 animate-spin" />
            Indexing...
          </span>
        );
      case "pending":
        return (
          <span className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30">
            <Clock className="w-2.5 h-2.5 inline mr-1" />
            Pending Startup Index
          </span>
        );
      case "failed":
        return (
          <div className="inline-flex items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
            <span
              className="fluent-badge text-[10px] bg-[var(--status-danger)]/10 text-[var(--status-danger)] border-[var(--status-danger)]/30 cursor-help"
              title={idxStatus.error_message || idxStatus.last_error || "Indexing failed"}
            >
              <AlertTriangle className="w-2.5 h-2.5 inline mr-1" />
              Failed
            </span>
            <button
              type="button"
              onClick={() => handleRetryIndexing(idxStatus.document_id)}
              className="text-[10px] text-[var(--accent-primary)] hover:underline cursor-pointer"
              title="Retry indexing"
            >
              Retry
            </button>
          </div>
        );
      case "awaiting_key":
        return (
          <span
            className="fluent-badge text-[10px] bg-[var(--bg-subtle)] text-[var(--text-secondary)] border-[var(--border-subtle)] cursor-help"
            title="Requires configured AI Provider API Key to index"
          >
            <Key className="w-2.5 h-2.5 inline mr-1" />
            Awaiting AI Key
          </span>
        );
      default:
        return idxStatus.is_indexed ? (
          <span className="fluent-badge text-[10px] bg-[var(--status-success)]/10 text-[var(--status-success)] border-[var(--status-success)]/30">
            <Sparkles className="w-2.5 h-2.5 inline mr-1" />
            Indexed ({idxStatus.chunk_count} chunks)
          </span>
        ) : (
          <span className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30">
            Not Indexed
          </span>
        );
    }
  };

  const unindexedCount = documents.filter((d) => !getDocIndexStatus(d.id)?.is_indexed).length;

  return (
    <div className="p-6 space-y-6 max-w-6xl mx-auto">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold tracking-tight text-[var(--text-primary)]">
            My Knowledge
          </h1>
          <p className="text-xs text-[var(--text-secondary)]">
            Local document library with SQLite-backed text extraction & AI vector embeddings.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={fetchDocuments}
            disabled={isLoading || isImporting || isBatchIndexing}
            className="fluent-button p-2 text-xs"
            title="Refresh list"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? "animate-spin" : ""}`} />
          </button>

          {indexingOverview && indexingOverview.failed_documents > 0 && (
            <button
              type="button"
              onClick={handleRetryFailed}
              disabled={isBatchIndexing || isImporting}
              className="fluent-button text-xs py-1.5 px-3 text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
              title="Retry indexing for failed documents"
            >
              <AlertTriangle className="w-3.5 h-3.5" />
              <span>Retry Failed ({indexingOverview.failed_documents})</span>
              <span>Retry Failed Indexing ({indexingOverview.failed_documents})</span>
            </button>
          )}

          {failedTaggingCount > 0 && (
            <button
              type="button"
              onClick={handleRetryAllFailedTagging}
              disabled={isRetryingTagging}
              className="fluent-button text-xs py-1.5 px-3 text-rose-400 hover:bg-rose-500/10"
              title="Retry tag extraction for failed documents"
            >
              {isRetryingTagging ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : (
                <RotateCcw className="w-3.5 h-3.5" />
              )}
              <span>Retry Failed Tagging ({failedTaggingCount})</span>
            </button>
          )}

          {documents.length > 0 && (
            <button
              type="button"
              onClick={handleIndexAll}
              disabled={isBatchIndexing || isImporting}
              className="fluent-button text-xs"
              title="Generate vector embeddings for documents"
            >
              {isBatchIndexing ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin text-[var(--accent-primary)]" />
                  <span>Indexing Documents...</span>
                </>
              ) : (
                <>
                  <Sparkles className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
                  <span>
                    {unindexedCount > 0 ? `Index Documents (${unindexedCount} pending)` : "Reindex All"}
                  </span>
                </>
              )}
            </button>
          )}

          <button
            type="button"
            onClick={handleImport}
            disabled={isImporting || isBatchIndexing}
            className="fluent-button fluent-button-primary text-xs"
          >
            {isImporting ? (
              <>
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                <span>Importing...</span>
              </>
            ) : (
              <>
                <Plus className="w-3.5 h-3.5" />
                <span>Import Documents</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Notifications */}
      {notification && (
        <div
          className={`fluent-card p-3.5 flex items-center justify-between gap-3 text-xs ${notification.type === "success"
            ? "border-[var(--status-success)]/40 bg-[var(--status-success)]/5 text-[var(--status-success)]"
            : notification.type === "error"
              ? "border-[var(--status-danger)]/40 bg-[var(--status-danger)]/5 text-[var(--status-danger)]"
              : "border-[var(--accent-primary)]/40 bg-[var(--accent-primary)]/5 text-[var(--accent-primary)]"
            }`}
        >
          <div className="flex items-center gap-2">
            {notification.type === "success" ? (
              <CheckCircle2 className="w-4 h-4 shrink-0" />
            ) : (
              <AlertCircle className="w-4 h-4 shrink-0" />
            )}
            <span>{notification.message}</span>
          </div>
          <button
            type="button"
            onClick={() => setNotification(null)}
            className="fluent-button-subtle p-1 rounded hover:bg-black/10 cursor-pointer"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Search Input Bar */}
      <div className="relative select-text">
        <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-tertiary)] pointer-events-none" />
        <input
          ref={searchInputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search documents by filename or full extracted text content..."
          className="fluent-input text-xs pl-10 pr-10 py-2.5 cursor-text select-text"
          autoComplete="off"
          spellCheck={false}
        />
        {isSearching ? (
          <Loader2 className="absolute right-3.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 animate-spin text-[var(--accent-primary)] pointer-events-none" />
        ) : searchQuery ? (
          <button
            type="button"
            onClick={() => setSearchQuery("")}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)] hover:text-[var(--text-primary)] cursor-pointer p-1"
            title="Clear search"
            aria-label="Clear document search"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        ) : null}
      </div>

      {/* Phase 3.3 Concept & Tag Filter Pills */}
      {allTags.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5 pt-0.5">
          <span className="text-[11px] text-[var(--text-tertiary)] flex items-center gap-1 font-medium mr-1">
            <Filter className="w-3 h-3" />
            <span>Filter by topic / concept:</span>
          </span>
          {allTags.map((tag) => {
            const isSelected = selectedTagFilter?.toLowerCase() === tag.name.toLowerCase();
            return (
              <button
                key={tag.id}
                type="button"
                onClick={() => setSelectedTagFilter(isSelected ? null : tag.name)}
                className={`fluent-badge text-[11px] py-0.5 px-2.5 cursor-pointer transition-all ${isSelected
                  ? "bg-[var(--accent-primary)] text-[var(--accent-foreground)] border-[var(--accent-primary)] font-semibold shadow-xs"
                  : "hover:bg-[var(--bg-card-hover)] hover:border-[var(--border-strong)] text-[var(--text-secondary)]"
                  }`}
              >
                <span>{tag.name}</span>
              </button>
            );
          })}
          {selectedTagFilter && (
            <button
              type="button"
              onClick={() => setSelectedTagFilter(null)}
              className="text-[11px] text-[var(--accent-primary)] hover:underline ml-2 cursor-pointer flex items-center gap-0.5"
            >
              <span>Clear filter</span>
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
      )}

      {/* Main Content Stage */}
      {isLoading ? (
        <div className="fluent-card p-12 flex flex-col items-center justify-center text-center space-y-3">
          <Loader2 className="w-6 h-6 animate-spin text-[var(--accent-primary)]" />
          <span className="text-xs text-[var(--text-secondary)]">
            Loading documents from SQLite...
          </span>
        </div>
      ) : searchResults !== null ? (
        /* Search Results View */
        <div className="space-y-3">
          <div className="flex items-center justify-between text-xs text-[var(--text-secondary)]">
            <span>
              Found <strong>{searchResults.length}</strong> matching document{searchResults.length !== 1 ? "s" : ""}
            </span>
            <button
              type="button"
              onClick={() => setSearchQuery("")}
              className="text-[var(--accent-primary)] hover:underline text-[11px]"
            >
              Clear Search
            </button>
          </div>

          {searchResults.length === 0 ? (
            <div className="fluent-card p-12 text-center flex flex-col items-center justify-center space-y-2">
              <Search className="w-8 h-8 text-[var(--text-tertiary)]" />
              <h3 className="text-sm font-semibold text-[var(--text-primary)]">
                No matching results found
              </h3>
              <p className="text-xs text-[var(--text-secondary)]">
                No document filename or extracted text matches "{searchQuery}".
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {searchResults.map((result) => {
                const idxStatus = getDocIndexStatus(result.id);
                return (
                  <div
                    key={result.id}
                    className="fluent-card p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3 hover:border-[var(--border-strong)]"
                  >
                    <div className="flex items-start gap-3 min-w-0 flex-1">
                      <div className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] shrink-0 mt-0.5">
                        {renderFileIcon(result.file_type)}
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-xs font-semibold text-[var(--text-primary)] truncate">
                            {result.filename}
                          </span>
                          <span className="fluent-badge text-[10px] uppercase font-mono">
                            {result.file_type}
                          </span>
                          {renderIndexBadge(idxStatus)}
                          <span className="text-[10px] text-[var(--text-tertiary)]">
                            matched via {result.match_type}
                          </span>
                        </div>
                        <p className="text-[11px] text-[var(--text-secondary)] mt-1 font-mono bg-[var(--bg-subtle)] p-2 rounded-[var(--radius-sm)] border border-[var(--border-subtle)] leading-relaxed select-text">
                          {result.snippet}
                        </p>
                        {documentTagsMap[result.id] && documentTagsMap[result.id].length > 0 && (
                          <div className="flex flex-wrap items-center gap-1 mt-1.5">
                            {documentTagsMap[result.id].map((t) => (
                              <span
                                key={t.tag_id}
                                className={`fluent-badge text-[9px] py-0.5 px-1.5 flex items-center gap-0.5 ${t.is_ai_generated
                                  ? "bg-[var(--accent-primary)]/10 text-[var(--accent-primary)] border-[var(--accent-primary)]/20"
                                  : "bg-purple-500/10 text-purple-400 border-purple-500/20"
                                  }`}
                              >
                                {t.is_ai_generated ? <Sparkles className="w-2 h-2" /> : <Tag className="w-2 h-2" />}
                                <span>{t.name}</span>
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="flex items-center gap-2 shrink-0 sm:self-center">
                      <button
                        type="button"
                        onClick={() => handlePreview(result.id)}
                        className="fluent-button text-xs py-1 px-2.5"
                        title="Preview extracted content"
                      >
                        <Eye className="w-3.5 h-3.5" />
                        <span>Preview</span>
                      </button>
                      <button
                        type="button"
                        onClick={(e) => openDeleteModal(result.id, result.filename, e.currentTarget)}
                        disabled={deletingId === result.id}
                        className="fluent-button text-xs py-1 px-2 text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
                        title="Delete document"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      ) : documents.length === 0 ? (
        /* Empty State */
        <div className="fluent-card p-12 text-center flex flex-col items-center justify-center space-y-4">
          <div className="w-14 h-14 rounded-[var(--radius-xl)] bg-[var(--bg-subtle)] border border-[var(--border-default)] flex items-center justify-center text-[var(--accent-primary)] shadow-[var(--shadow-rest)]">
            <Files className="w-7 h-7" />
          </div>
          <div className="space-y-1">
            <h2 className="text-base font-semibold text-[var(--text-primary)]">
              Your knowledge repository is empty
            </h2>
            <p className="text-xs text-[var(--text-secondary)] max-w-md mx-auto leading-relaxed">
              Import PDF, Markdown (.md), or Plaintext (.txt) files to establish your local knowledge base.
              Files are stored locally in SQLite with full text extraction and AI embeddings.
            </p>
          </div>
          <button
            type="button"
            onClick={handleImport}
            disabled={isImporting}
            className="fluent-button fluent-button-primary text-xs py-2 px-4"
          >
            <Plus className="w-4 h-4 mr-1.5" />
            <span>Import Your First Documents</span>
          </button>
        </div>
      ) : (
        /* Documents Table / Card List */
        <div className="space-y-2">
          <div className="flex items-center justify-between text-xs text-[var(--text-secondary)] px-1">
            <span>
              {selectedTagFilter ? (
                <>
                  Showing <strong>{filteredDocuments.length}</strong> of <strong>{documents.length}</strong> document{documents.length !== 1 ? "s" : ""} tagged with{" "}
                  <strong className="text-[var(--accent-primary)] font-semibold">"{selectedTagFilter}"</strong>
                </>
              ) : (
                <>
                  <strong>{documents.length}</strong> document{documents.length !== 1 ? "s" : ""} stored locally •{" "}
                  <strong className="text-[var(--text-primary)]">
                    {indexingOverview?.indexed_documents || 0}
                  </strong>{" "}
                  indexed for AI Q&A
                </>
              )}
            </span>
            <span className="text-[11px] text-[var(--text-tertiary)]">
              Total: {formatBytes(documents.reduce((acc, d) => acc + d.file_size, 0))}
            </span>
          </div>

          {
            filteredDocuments.length === 0 ? (
              <div className="fluent-card p-8 text-center space-y-2">
                <Tag className="w-6 h-6 text-[var(--text-tertiary)] mx-auto" />
                <p className="text-xs text-[var(--text-secondary)]">
                  No documents matching tag "{selectedTagFilter}".
                </p>
                <button
                  type="button"
                  onClick={() => setSelectedTagFilter(null)}
                  className="fluent-button text-xs py-1 px-3 mx-auto"
                >
                  Clear tag filter
                </button>
              </div>
            ) : (
              <div className="fluent-card divide-y divide-[var(--border-subtle)] overflow-hidden">
                {filteredDocuments.map((doc) => {
                  const idxStatus = getDocIndexStatus(doc.id);
                  const isThisIndexing = indexingDocId === doc.id;
                  const isThisRetagging = isRetaggingDocId === doc.id;
                  const docTags = documentTagsMap[doc.id] || [];
                  const docTagStatus = taggingStatusMap[doc.id];

                  return (
                    <div
                      key={doc.id}
                      className="p-3.5 flex flex-col sm:flex-row sm:items-start justify-between gap-3 hover:bg-[var(--bg-card-hover)] transition-colors"
                    >
                      <div className="flex items-start gap-3 min-w-0 flex-1">
                        <div className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] shrink-0 mt-0.5">
                          {renderFileIcon(doc.file_type)}
                        </div>
                        <div className="min-w-0 flex-1 space-y-1">
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="text-xs font-semibold text-[var(--text-primary)] truncate">
                              {doc.filename}
                            </span>
                            <span className="fluent-badge text-[10px] uppercase font-mono">
                              {doc.file_type}
                            </span>
                            {renderIndexBadge(idxStatus)}
                          </div>
                          <div className="flex items-center gap-2 text-[11px] text-[var(--text-tertiary)]">
                            <span>{formatBytes(doc.file_size)}</span>
                            <span>•</span>
                            <span>{doc.text_length.toLocaleString()} chars</span>
                            <span>•</span>
                            <span>Imported {new Date(doc.imported_at).toLocaleDateString()}</span>
                          </div>

                          {/* Phase 3.3 Document Tags Area */}
                          <div className="flex flex-wrap items-center gap-1.5 pt-1.5">
                            <Tag className="w-3 h-3 text-[var(--text-tertiary)] shrink-0" />

                            {/* Render Extracted Tags */}
                            {docTags.map((tag) => (
                              <span
                                key={tag.tag_id}
                                className={`fluent-badge text-[10px] py-0.5 px-2 flex items-center gap-1 group transition-colors ${tag.is_ai_generated
                                  ? "bg-[var(--accent-primary)]/10 text-[var(--accent-primary)] border-[var(--accent-primary)]/30"
                                  : "bg-purple-500/10 text-purple-400 border-purple-500/30"
                                  }`}
                                title={
                                  tag.is_ai_generated
                                    ? `AI Tag (Confidence: ${(tag.confidence * 100).toFixed(0)}%)${tag.evidence ? ` • Evidence: "${tag.evidence}"` : ""
                                    }`
                                    : "Manual User Tag"
                                }
                              >
                                {tag.is_ai_generated ? (
                                  <Sparkles className="w-2.5 h-2.5 shrink-0" />
                                ) : (
                                  <Tag className="w-2.5 h-2.5 shrink-0" />
                                )}
                                <span
                                  className="cursor-pointer hover:underline font-medium"
                                  onClick={() =>
                                    setSelectedTagFilter(
                                      selectedTagFilter?.toLowerCase() === tag.name.toLowerCase()
                                        ? null
                                        : tag.name
                                    )
                                  }
                                >
                                  {tag.name}
                                </span>
                                <button
                                  type="button"
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    handleRemoveTag(doc.id, tag.tag_id);
                                  }}
                                  className="opacity-40 group-hover:opacity-100 hover:text-[var(--status-danger)] cursor-pointer ml-0.5"
                                  title={`Remove tag "${tag.name}"`}
                                >
                                  <X className="w-2.5 h-2.5" />
                                </button>
                              </span>
                            ))}

                            {/* Tagging Status Badges (Pending, In Progress, Failed, Awaiting Key) */}
                            {docTagStatus?.status === "tagging" || isThisRetagging ? (
                              <span className="fluent-badge text-[10px] py-0.5 px-2 bg-blue-500/10 text-blue-400 border-blue-500/30 flex items-center gap-1">
                                <Loader2 className="w-2.5 h-2.5 animate-spin shrink-0" />
                                <span>Tagging in progress...</span>
                              </span>
                            ) : docTagStatus?.status === "pending" ? (
                              <span className="fluent-badge text-[10px] py-0.5 px-2 bg-amber-500/10 text-amber-400 border-amber-500/30 flex items-center gap-1">
                                <Clock className="w-2.5 h-2.5 shrink-0" />
                                <span>Pending tagging</span>
                              </span>
                            ) : docTagStatus?.retry_state === "retry_wait" ? (
                              <div className="flex items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
                                <span
                                  className="fluent-badge text-[10px] py-0.5 px-2 bg-amber-500/10 text-amber-400 border-amber-500/30 flex items-center gap-1"
                                  title={`Tagging transient failure: ${docTagStatus.error_message || docTagStatus.last_error || "Error"}. Auto-retry #${docTagStatus.attempt_count}${docTagStatus.next_retry_at ? ` at ${new Date(docTagStatus.next_retry_at).toLocaleTimeString()}` : ""}`}
                                >
                                  <Clock className="w-2.5 h-2.5 animate-pulse shrink-0" />
                                  <span>Tag Retry Wait (#{docTagStatus.attempt_count})</span>
                                </span>
                                <button
                                  type="button"
                                  onClick={() => handleRetryTagging(doc.id)}
                                  className="text-[10px] text-[var(--accent-primary)] hover:underline cursor-pointer"
                                  title="Force immediate tag extraction"
                                >
                                  Retry Now
                                </button>
                                <button
                                  type="button"
                                  onClick={() => handleCancelTaggingRetry(doc.id)}
                                  className="text-[10px] text-[var(--text-tertiary)] hover:text-[var(--status-danger)] cursor-pointer"
                                  title="Stop automatic retries"
                                >
                                  <X className="w-2.5 h-2.5" />
                                </button>
                              </div>
                            ) : docTagStatus?.retry_state === "paused" ? (
                              <span className="fluent-badge text-[10px] py-0.5 px-2 bg-amber-500/10 text-amber-500 border-amber-500/30 flex items-center gap-1">
                                <Clock className="w-2.5 h-2.5 shrink-0" />
                                <span>Tagging Paused</span>
                              </span>
                            ) : docTagStatus?.retry_state === "cancelled" ? (
                              <div className="flex items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
                                <span className="fluent-badge text-[10px] py-0.5 px-1.5 bg-[var(--bg-subtle)] text-[var(--text-tertiary)] border-[var(--border-subtle)]">
                                  Tag Retry Stopped
                                </span>
                                <button
                                  type="button"
                                  onClick={() => handleRetryTagging(doc.id)}
                                  className="text-[10px] text-[var(--accent-primary)] hover:underline cursor-pointer"
                                >
                                  Resume
                                </button>
                              </div>
                            ) : docTagStatus?.status === "awaiting_key" ? (
                              <span
                                className="fluent-badge text-[10px] py-0.5 px-2 bg-amber-500/10 text-amber-500 border-amber-500/30 flex items-center gap-1"
                                title="Configure Gemini API Key in Settings to extract tags"
                              >
                                <Key className="w-2.5 h-2.5 shrink-0" />
                                <span>Awaiting AI key</span>
                              </span>
                            ) : docTagStatus?.status === "failed" ? (
                              <div className="flex items-center gap-1">
                                <span
                                  className="fluent-badge text-[10px] py-0.5 px-2 bg-rose-500/10 text-rose-400 border-rose-500/30 flex items-center gap-1"
                                  title={docTagStatus.error_message || docTagStatus.last_error || "Tagging failed"}
                                >
                                  <AlertTriangle className="w-2.5 h-2.5 shrink-0" />
                                  <span>Tagging failed</span>
                                </span>
                                <button
                                  type="button"
                                  onClick={() => handleRetryTagging(doc.id)}
                                  className="fluent-badge text-[10px] py-0.5 px-1.5 text-[var(--accent-primary)] hover:bg-[var(--accent-primary)]/10 cursor-pointer flex items-center gap-0.5"
                                  title="Retry tag extraction"
                                >
                                  <RotateCcw className="w-2.5 h-2.5 shrink-0" />
                                  <span>Retry</span>
                                </button>
                              </div>
                            ) : docTags.length === 0 ? (
                              <span className="text-[10px] text-[var(--text-tertiary)] italic mr-1">
                                No tags extracted
                              </span>
                            ) : null}

                            {/* Inline Add Manual Tag */}
                            {activeTagInputDocId === doc.id ? (
                              <div className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                                <input
                                  type="text"
                                  value={manualTagInput}
                                  onChange={(e) => setManualTagInput(e.target.value)}
                                  onKeyDown={(e) => {
                                    if (e.key === "Enter") handleAddManualTag(doc.id);
                                    if (e.key === "Escape") setActiveTagInputDocId(null);
                                  }}
                                  placeholder="Tag name..."
                                  className="fluent-input text-[10px] py-0.5 px-1.5 w-24 h-5.5"
                                  autoFocus
                                />
                                <button
                                  type="button"
                                  onClick={() => handleAddManualTag(doc.id)}
                                  className="fluent-button p-1 text-[10px] h-5.5"
                                  title="Add tag"
                                >
                                  <Plus className="w-2.5 h-2.5" />
                                </button>
                                <button
                                  type="button"
                                  onClick={() => setActiveTagInputDocId(null)}
                                  className="p-0.5 text-[var(--text-tertiary)] hover:text-[var(--text-primary)] cursor-pointer"
                                  title="Cancel"
                                >
                                  <X className="w-2.5 h-2.5" />
                                </button>
                              </div>
                            ) : (
                              <button
                                type="button"
                                onClick={() => {
                                  setActiveTagInputDocId(doc.id);
                                  setManualTagInput("");
                                }}
                                className="fluent-badge text-[10px] py-0.5 px-1.5 border-dashed border-[var(--border-default)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)] hover:border-[var(--accent-primary)] transition-colors cursor-pointer"
                                title="Add manual tag"
                              >
                                <Plus className="w-2.5 h-2.5 inline mr-0.5" />
                                Tag
                              </button>
                            )}

                            {/* Re-tag AI Button */}
                            <button
                              type="button"
                              onClick={() => handleRetag(doc.id, doc.filename)}
                              disabled={isThisRetagging}
                              className="fluent-badge text-[10px] py-0.5 px-1.5 bg-transparent border-transparent text-[var(--text-tertiary)] hover:text-[var(--accent-primary)] transition-colors cursor-pointer ml-0.5"
                              title="Re-extract AI tags for this document"
                            >
                              {isThisRetagging ? (
                                <Loader2 className="w-2.5 h-2.5 animate-spin inline mr-0.5" />
                              ) : (
                                <Sparkles className="w-2.5 h-2.5 inline mr-0.5 text-[var(--accent-primary)]" />
                              )}
                              Extract Tags
                            </button>
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center gap-2 shrink-0 sm:self-center">
                        <button
                          type="button"
                          onClick={() => handleIndexSingle(doc.id, doc.filename)}
                          disabled={isThisIndexing || isBatchIndexing}
                          className="fluent-button text-xs py-1 px-2.5"
                          title={idxStatus?.is_indexed ? "Reindex document chunks & embeddings" : "Index document for AI Q&A"}
                        >
                          {isThisIndexing ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin text-[var(--accent-primary)]" />
                          ) : (
                            <Sparkles className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
                          )}
                          <span>{idxStatus?.is_indexed ? "Reindex" : "Index"}</span>
                        </button>

                        <button
                          type="button"
                          onClick={() => handlePreview(doc.id)}
                          className="fluent-button text-xs py-1 px-2.5"
                          title="Preview extracted content"
                        >
                          <Eye className="w-3.5 h-3.5" />
                          <span>Preview</span>
                        </button>

                        <button
                          type="button"
                          onClick={(e) => openDeleteModal(doc.id, doc.filename, e.currentTarget)}
                          disabled={deletingId === doc.id || isThisIndexing}
                          className="fluent-button text-xs py-1 px-2 text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
                          title="Delete document"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
        </div>
      )}

      {/* Modals */}
      <DocumentPreviewModal
        document={previewDoc}
        isOpen={isPreviewOpen}
        onClose={() => setIsPreviewOpen(false)}
      />

      <DuplicateNoticeModal
        duplicates={duplicates}
        isOpen={isDuplicateModalOpen}
        onClose={() => setIsDuplicateModalOpen(false)}
      />

      <DeleteConfirmModal
        isOpen={Boolean(deleteTarget)}
        filename={deleteTarget?.filename || ""}
        onConfirm={handleConfirmDelete}
        onCancel={closeDeleteModal}
        isDeleting={Boolean(deletingId)}
        error={deleteError}
      />
    </div>
  );
};

export default MyKnowledge;
