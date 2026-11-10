import React, { useState, useEffect, useCallback } from "react";
import {
  FileText,
  Cpu,
  CheckCircle2,
  ArrowRight,
  FileSpreadsheet,
  FileCode,
  Sparkles,
  Layers,
  AlertCircle,
  Clock,
  RefreshCw,
  Loader2,
} from "lucide-react";
import { DocumentService } from "@/services/documents/documentService";
import { AIService } from "@/services/ai/aiService";
import { RetryService } from "@/services/ai/retryService";
import type { DocumentSummary, StorageInitResponse, StorageStats } from "@/types/documents";
import type { IndexingOverview, SafeProviderStatus } from "@/types/ai";
import type { RetryOverview } from "@/types/retry";
import { formatBytes } from "@/utils";
import { useRouter } from "@/router";

export const Dashboard: React.FC = () => {
  const { navigate } = useRouter();
  const [stats, setStats] = useState<StorageStats>({
    total_documents: 0,
    total_size_bytes: 0,
    pdf_count: 0,
    md_count: 0,
    txt_count: 0,
    last_imported_at: null,
  });
  const [indexing, setIndexing] = useState<IndexingOverview | null>(null);
  const [retryOverview, setRetryOverview] = useState<RetryOverview | null>(null);
  const [aiStatus, setAiStatus] = useState<SafeProviderStatus | null>(null);
  const [recentDocs, setRecentDocs] = useState<DocumentSummary[]>([]);
  const [initInfo, setInitInfo] = useState<StorageInitResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRetryingFailed, setIsRetryingFailed] = useState(false);
  const [isPausingResuming, setIsPausingResuming] = useState(false);

      const loadDashboardData = useCallback(async () => {
        try {
          const [storageStats, indexOverview, providerStatus, retryStats, docs, initRes] = await Promise.all([
            DocumentService.getStorageStats(),
            AIService.getIndexingStatus().catch(() => null),
            AIService.getProviderStatus().catch(() => null),
            RetryService.getRetryOverview().catch(() => null),
            DocumentService.getDocuments(),
            DocumentService.initializeStorage().catch(() => null),
          ]);

      setStats(storageStats);
      setIndexing(indexOverview);
      setAiStatus(providerStatus);
      setRetryOverview(retryStats);
      setRecentDocs(docs.slice(0, 5));
      setInitInfo(initRes);
    } catch (err) {
      console.error("Failed to load dashboard storage data:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadDashboardData();

    // Listen to background indexing and tagging events for real-time reactivity
    let unlistenIdx: (() => void) | undefined;
    let unlistenTag: (() => void) | undefined;

      import("@tauri-apps/api/event").then(({ listen }) => {
        listen("indexing:status_changed", () => {
          loadDashboardData();
        }).then((unlisten) => {
          unlistenIdx = unlisten;
        }).catch(() => { });

        listen("tagging:status_changed", () => {
          loadDashboardData();
        }).then((unlisten) => {
          unlistenTag = unlisten;
        }).catch(() => { });
      }).catch(() => { });

      return () => {
        if (unlistenIdx) unlistenIdx();
        if (unlistenTag) unlistenTag();
      };
    }, [loadDashboardData]);

  const handleRetryFailed = async () => {
    try {
      setIsRetryingFailed(true);
      await RetryService.retryAllFailed();
      await loadDashboardData();
    } catch (err) {
      console.error("Failed to retry indexing:", err);
    } finally {
      setIsRetryingFailed(false);
    }
  };

const handleTogglePauseRetries = async () => {
  try {
    setIsPausingResuming(true);
    if (retryOverview?.is_globally_paused) {
      await RetryService.resumeAllRetries();
    } else {
      await RetryService.pauseAllRetries();
    }
    await loadDashboardData();
  } catch (err) {
    console.error("Failed to toggle retries:", err);
  } finally {
    setIsPausingResuming(false);
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

return (
  <div className="p-6 space-y-6 max-w-6xl mx-auto">
    {/* Header Overview */}
    <div className="space-y-1">
      <h1 className="text-xl font-semibold tracking-tight text-[var(--text-primary)]">
        System Overview
      </h1>
      <p className="text-xs text-[var(--text-secondary)]">
        Your Personal AI Knowledge Operating System — Local storage & AI RAG status.
      </p>
    </div>

    {/* Metric Cards Grid */}
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {/* Total Documents Metric */}
      <div className="fluent-card p-4 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-[var(--text-secondary)] font-medium">
            Total Documents
          </span>
          <FileText className="w-4 h-4 text-[var(--accent-primary)]" />
        </div>
        <div className="text-2xl font-bold text-[var(--text-primary)] tracking-tight">
          {isLoading ? "..." : stats.total_documents}
        </div>
        <div className="text-[11px] text-[var(--text-tertiary)] flex items-center gap-1.5">
          <span>{stats.pdf_count} PDF</span>
          <span>•</span>
          <span>{stats.md_count} MD</span>
          <span>•</span>
          <span>{stats.txt_count} TXT</span>
        </div>
      </div>

      {/* AI Indexed Documents & Chunks */}
      <div className="fluent-card p-4 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-[var(--text-secondary)] font-medium">
            AI Indexed Documents
          </span>
          <Sparkles className="w-4 h-4 text-[var(--accent-primary)]" />
        </div>
        <div className="text-2xl font-bold text-[var(--text-primary)] tracking-tight">
          {isLoading ? "..." : `${indexing?.indexed_documents || 0} / ${stats.total_documents}`}
        </div>
        <p className="text-[11px] text-[var(--text-tertiary)]">
          {indexing?.total_chunks || 0} chunks extracted
        </p>
      </div>

      {/* Local Vector Embeddings */}
      <div className="fluent-card p-4 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-[var(--text-secondary)] font-medium">
            Vector Embeddings
          </span>
          <Layers className="w-4 h-4 text-[var(--accent-primary)]" />
        </div>
        <div className="text-2xl font-bold text-[var(--text-primary)] tracking-tight">
          {isLoading ? "..." : indexing?.total_embeddings || 0}
        </div>
        <p className="text-[11px] text-[var(--text-tertiary)]">
          Stored locally in SQLite (768-dim)
        </p>
      </div>

      {/* Active AI Provider Status */}
      <div className="fluent-card p-4 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-[var(--text-secondary)] font-medium">
            AI Provider (BYOK)
          </span>
          <Cpu className="w-4 h-4 text-[var(--accent-primary)]" />
        </div>
        <div className="flex items-center gap-1.5 pt-1">
          {aiStatus?.is_configured ? (
            <span className="text-xs font-semibold text-[var(--status-success)] flex items-center gap-1">
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>Connected (Gemini)</span>
            </span>
          ) : (
            <span className="text-xs font-semibold text-amber-500 flex items-center gap-1">
              <AlertCircle className="w-3.5 h-3.5" />
              <span>Not Configured</span>
            </span>
          )}
        </div>
        <p className="text-[11px] text-[var(--text-tertiary)] truncate">
          {aiStatus?.is_configured
            ? aiStatus.active_generation_model
            : "Set API key in Settings"}
        </p>
      </div>
    </div>

    {/* Phase 3.2 Automatic Knowledge Indexing Card */}
    <div className="fluent-card p-5 space-y-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 pb-2 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-[var(--accent-primary)]" />
          <h2 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
            Automatic Knowledge Indexing
          </h2>
        </div>
        <div className="flex items-center gap-2">
          {indexing?.currently_processing_documents ? (
            <span className="fluent-badge text-[10px] bg-blue-500/10 text-blue-500 border-blue-500/30 flex items-center gap-1">
              <Loader2 className="w-3 h-3 animate-spin" />
              <span>Indexing {indexing.currently_processing_documents} doc(s)...</span>
            </span>
          ) : indexing?.pending_documents ? (
            <span className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30 flex items-center gap-1">
              <Clock className="w-3 h-3" />
              <span>{indexing.pending_documents} pending</span>
            </span>
          ) : (
            <span className="fluent-badge text-[10px] bg-emerald-500/10 text-emerald-500 border-emerald-500/30 flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" />
              <span>Up to date</span>
            </span>
          )}

            <button
              type="button"
              onClick={handleTogglePauseRetries}
              disabled={isPausingResuming}
              className={`fluent-button text-xs py-1 px-2.5 cursor-pointer ${retryOverview?.is_globally_paused
                  ? "bg-amber-500/10 text-amber-500 border-amber-500/30 hover:bg-amber-500/20"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-subtle)]"
                }`}
              title={retryOverview?.is_globally_paused ? "Resume automatic background retries" : "Pause automatic retries"}
            >
              {isPausingResuming ? (
                <Loader2 className="w-3 h-3 animate-spin" />
              ) : retryOverview?.is_globally_paused ? (
                <RefreshCw className="w-3 h-3" />
              ) : (
                <Clock className="w-3 h-3" />
              )}
              <span>{retryOverview?.is_globally_paused ? "Resume Retries" : "Pause Retries"}</span>
            </button>

          {((indexing && indexing.failed_documents > 0) || (retryOverview && (retryOverview.failed_indexing_count > 0 || retryOverview.failed_tagging_count > 0 || retryOverview.retry_wait_indexing_count > 0 || retryOverview.retry_wait_tagging_count > 0))) && (
            <button
              type="button"
              onClick={handleRetryFailed}
              disabled={isRetryingFailed}
              className="fluent-button text-xs py-1 px-2.5 text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10 cursor-pointer"
              title="Force immediate retry for all failed or waiting operations"
            >
              {isRetryingFailed ? (
                <Loader2 className="w-3 h-3 animate-spin" />
              ) : (
                <RefreshCw className="w-3 h-3" />
              )}
              <span>Retry All Failed</span>
            </button>
          )}
        </div>
      </div>

      {/* Status Metric Badges */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
          <span className="text-[10px] text-[var(--text-tertiary)] uppercase font-semibold">Total</span>
          <div className="text-base font-bold text-[var(--text-primary)]">
            {indexing?.total_documents ?? stats.total_documents}
          </div>
          <span className="text-[10px] text-[var(--text-tertiary)]">Documents</span>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
          <span className="text-[10px] text-emerald-500 uppercase font-semibold">Indexed</span>
          <div className="text-base font-bold text-emerald-500">
            {indexing?.indexed_documents ?? 0}
          </div>
          <span className="text-[10px] text-[var(--text-tertiary)]">Ready for Q&A</span>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
          <span className="text-[10px] text-amber-500 uppercase font-semibold">Pending</span>
          <div className="text-base font-bold text-amber-500">
            {indexing?.pending_documents ?? 0}
          </div>
          <span className="text-[10px] text-[var(--text-tertiary)]">Queued</span>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
          <span className="text-[10px] text-blue-500 uppercase font-semibold">Indexing</span>
          <div className="text-base font-bold text-blue-500">
            {indexing?.currently_processing_documents ?? 0}
          </div>
          <span className="text-[10px] text-[var(--text-tertiary)]">In progress</span>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
          <span className="text-[10px] text-rose-500 uppercase font-semibold">Failed</span>
          <div className="text-base font-bold text-rose-500">
            {indexing?.failed_documents ?? 0}
          </div>
          <span className="text-[10px] text-[var(--text-tertiary)]">Errors</span>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
          <span className="text-[10px] text-[var(--text-secondary)] uppercase font-semibold">Awaiting Key</span>
          <div className="text-base font-bold text-[var(--text-primary)]">
            {indexing?.awaiting_key_documents ?? 0}
          </div>
          <span className="text-[10px] text-[var(--text-tertiary)]">Need API key</span>
        </div>
      </div>

      {/* Last Indexing Time Note */}
      <div className="flex flex-wrap items-center justify-between text-[11px] text-[var(--text-tertiary)] pt-1">
        <span className="flex items-center gap-1.5">
          <Clock className="w-3.5 h-3.5" />
          <span>
            Last successful index:{" "}
            <strong className="text-[var(--text-secondary)]">
              {indexing?.last_successful_indexing_time
                ? new Date(indexing.last_successful_indexing_time).toLocaleString()
                : "Never"}
            </strong>
          </span>
        </span>
        <span>Automatic startup indexing inspects and embeds modified/new files asynchronously.</span>
      </div>
    </div>

    {/* Phase 3 Diagnostics Banner */}
    <div className="fluent-card p-5 space-y-3 border-l-4 border-l-[var(--accent-primary)]">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-[var(--accent-primary)]" />
          <h2 className="text-sm font-semibold text-[var(--text-primary)]">
            Phase 3: AI Intelligence, Semantic Search & RAG Active
          </h2>
        </div>
        <span className="fluent-badge text-[10px] text-[var(--accent-primary)]">
          Local SQLite Vector Search
        </span>
      </div>
      <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
        ECHO's personal knowledge assistant is ready. Documents can be embedded into local SQLite chunks and queried semantically through Ask ECHO with verified source citations.
      </p>

      {initInfo && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 pt-1 text-[11px] text-[var(--text-secondary)] font-mono">
          <div className="p-2 rounded bg-[var(--bg-subtle)] border border-[var(--border-subtle)] truncate" title={initInfo.db_path}>
            Database: {initInfo.db_path}
          </div>
          <div className="p-2 rounded bg-[var(--bg-subtle)] border border-[var(--border-subtle)] truncate" title={initInfo.documents_dir}>
            Documents: {initInfo.documents_dir}
          </div>
        </div>
      )}
    </div>

    {/* Recent Documents Section */}
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
          Recently Imported Documents
        </h2>
        <button
          type="button"
          onClick={() => navigate("my-knowledge")}
          className="text-xs text-[var(--accent-primary)] hover:underline inline-flex items-center gap-1 cursor-pointer"
        >
          <span>View All Knowledge</span>
          <ArrowRight className="w-3 h-3" />
        </button>
      </div>

      {recentDocs.length === 0 ? (
        <div className="fluent-card p-8 text-center space-y-2">
          <FileText className="w-8 h-8 text-[var(--text-tertiary)] mx-auto" />
          <p className="text-xs text-[var(--text-secondary)]">No documents imported yet.</p>
          <button
            type="button"
            onClick={() => navigate("my-knowledge")}
            className="fluent-button fluent-button-primary text-xs"
          >
            Import Documents
          </button>
        </div>
      ) : (
        <div className="fluent-card divide-y divide-[var(--border-subtle)] overflow-hidden">
          {recentDocs.map((doc) => (
            <div
              key={doc.id}
              onClick={() => navigate("my-knowledge", { previewDocumentId: doc.id })}
              className="p-3 flex items-center justify-between hover:bg-[var(--bg-card-hover)] transition-colors cursor-pointer"
            >
              <div className="flex items-center gap-3 min-w-0">
                <div className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] shrink-0">
                  {renderFileIcon(doc.file_type)}
                </div>
                <div className="min-w-0">
                  <span className="text-xs font-medium text-[var(--text-primary)] truncate block">
                    {doc.filename}
                  </span>
                  <span className="text-[10px] text-[var(--text-tertiary)]">
                    {formatBytes(doc.file_size)} • {doc.text_length.toLocaleString()} characters
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <span className="fluent-badge text-[10px] uppercase font-mono">
                  {doc.file_type}
                </span>
                <ArrowRight className="w-3 h-3 text-[var(--text-tertiary)]" />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  </div>
);
};

export default Dashboard;
