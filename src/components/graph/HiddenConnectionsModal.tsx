import React, { useState } from "react";
import {
  X,
  Sparkles,
  Loader2,
  Check,
  Ban,
  ArrowRight,
  FileText,
  HelpCircle,
  ExternalLink,
} from "lucide-react";
import type { SuggestedRelationship, ConceptRelationshipRecord } from "@/types/graph";
import { GraphService } from "@/services/graph/graphService";
import { RELATIONSHIP_COLOR_MAP } from "./RelationshipEdge";

interface HiddenConnectionsModalProps {
  isOpen: boolean;
  onClose: () => void;
  suggestions: SuggestedRelationship[];
  onAccept: (rel: ConceptRelationshipRecord) => void;
  onReject: (id: string) => void;
  onRefresh: () => void;
  onOpenDocument: (documentId: string) => void;
}

export const HiddenConnectionsModal: React.FC<HiddenConnectionsModalProps> = ({
  isOpen,
  onClose,
  suggestions,
  onAccept,
  onReject,
  onRefresh,
  onOpenDocument,
}) => {
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [processingId, setProcessingId] = useState<string | null>(null);
  const [filterStatus, setFilterStatus] = useState<"suggested" | "accepted" | "rejected" | "all">("suggested");

  if (!isOpen) return null;

  const handleDiscover = async () => {
    try {
      setIsDiscovering(true);
      await GraphService.discoverHiddenConnections();
      onRefresh();
    } catch (err) {
      console.error("Failed to discover hidden connections:", err);
    } finally {
      setIsDiscovering(false);
    }
  };

  const handleAccept = async (id: string) => {
    try {
      setProcessingId(id);
      const newConfirmedRel = await GraphService.acceptSuggestedRelationship(id);
      onAccept(newConfirmedRel);
    } catch (err) {
      console.error("Failed to accept suggestion:", err);
    } finally {
      setProcessingId(null);
    }
  };

  const handleReject = async (id: string) => {
    try {
      setProcessingId(id);
      await GraphService.rejectSuggestedRelationship(id);
      onReject(id);
    } catch (err) {
      console.error("Failed to reject suggestion:", err);
    } finally {
      setProcessingId(null);
    }
  };

  const filteredList = suggestions.filter((s) => {
    if (filterStatus === "all") return true;
    return s.status === filterStatus;
  });

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
      <div className="fluent-card w-full max-w-3xl max-h-[85vh] flex flex-col bg-[var(--bg-elevated)] border border-[var(--border-strong)] shadow-[var(--shadow-dialog)] overflow-hidden">
        {/* Header */}
        <div className="p-4 border-b border-[var(--border-subtle)] flex items-center justify-between shrink-0">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-cyan-500/15 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-semibold text-[var(--text-primary)]">
                  Hidden Connection Discovery
                </h2>
                <span className="fluent-badge text-[10px] py-0.2 px-2 bg-cyan-500/10 text-cyan-400 font-mono">
                  Phase 4.3 Human-in-the-Loop
                </span>
              </div>
              <p className="text-xs text-[var(--text-secondary)]">
                AI proposes unlinked semantic relationships. Only connections you approve enter the verified graph.
              </p>
            </div>
          </div>

          <button
            type="button"
            onClick={onClose}
            className="fluent-button p-1.5 text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
            title="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Filter bar & Discover Action */}
        <div className="p-3 bg-[var(--bg-subtle)] border-b border-[var(--border-subtle)] flex items-center justify-between gap-3 shrink-0">
          <div className="flex items-center gap-1.5">
            <button
              type="button"
              onClick={() => setFilterStatus("suggested")}
              className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border ${filterStatus === "suggested"
                  ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                  : "bg-[var(--bg-card)] text-[var(--text-secondary)] border-[var(--border-default)]"
                }`}
            >
              Pending Approval ({suggestions.filter((s) => s.status === "suggested").length})
            </button>
            <button
              type="button"
              onClick={() => setFilterStatus("accepted")}
              className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border ${filterStatus === "accepted"
                  ? "bg-emerald-600 text-white border-emerald-600"
                  : "bg-[var(--bg-card)] text-[var(--text-secondary)] border-[var(--border-default)]"
                }`}
            >
              Accepted ({suggestions.filter((s) => s.status === "accepted").length})
            </button>
            <button
              type="button"
              onClick={() => setFilterStatus("rejected")}
              className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border ${filterStatus === "rejected"
                  ? "bg-rose-600 text-white border-rose-600"
                  : "bg-[var(--bg-card)] text-[var(--text-secondary)] border-[var(--border-default)]"
                }`}
            >
              Rejected ({suggestions.filter((s) => s.status === "rejected").length})
            </button>
          </div>

          <button
            type="button"
            onClick={handleDiscover}
            disabled={isDiscovering}
            className="fluent-button text-xs py-1.5 px-3 bg-[var(--accent-primary)] text-white hover:opacity-90 flex items-center gap-1.5 cursor-pointer shadow-sm"
          >
            {isDiscovering ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Sparkles className="w-3.5 h-3.5" />
            )}
            <span>{isDiscovering ? "Analyzing Concepts..." : "Discover More"}</span>
          </button>
        </div>

        {/* Suggestions List */}
        <div className="p-4 flex-1 overflow-y-auto space-y-3 min-h-[350px]">
          {filteredList.length === 0 ? (
            <div className="h-64 flex flex-col items-center justify-center space-y-2.5 text-center p-6">
              <div className="w-10 h-10 rounded-full bg-[var(--bg-subtle)] border border-[var(--border-default)] flex items-center justify-center text-[var(--text-tertiary)]">
                <HelpCircle className="w-5 h-5" />
              </div>
              <h3 className="text-xs font-semibold text-[var(--text-primary)]">
                No {filterStatus} connections
              </h3>
              <p className="text-[11px] text-[var(--text-secondary)] max-w-sm">
                Click "Discover More" above to analyze co-occurrences, concept descriptions, and shared documents for latent connections.
              </p>
            </div>
          ) : (
            filteredList.map((item) => {
              const style = RELATIONSHIP_COLOR_MAP[item.relationship_type] || RELATIONSHIP_COLOR_MAP.RELATED_TO;
              const isProcessing = processingId === item.id;

              return (
                <div
                  key={item.id}
                  className="p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] hover:border-[var(--border-strong)] transition-all space-y-2.5"
                >
                  {/* Top Bar: Connection flow & Type Badge */}
                  <div className="flex items-center justify-between gap-2 flex-wrap">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-bold text-[var(--text-primary)]">
                        {item.source_concept_name || item.source_concept_id}
                      </span>
                      <ArrowRight className="w-3.5 h-3.5 text-[var(--text-tertiary)]" />
                      <span
                        className="fluent-badge text-[10px] font-semibold py-0.5 px-2 border"
                        style={{
                          backgroundColor: style.bg,
                          color: style.text,
                          borderColor: style.border,
                        }}
                      >
                        {item.relationship_type}
                      </span>
                      <ArrowRight className="w-3.5 h-3.5 text-[var(--text-tertiary)]" />
                      <span className="text-xs font-bold text-[var(--text-primary)]">
                        {item.target_concept_name || item.target_concept_id}
                      </span>
                    </div>

                    <div className="flex items-center gap-1.5">
                      <span className="text-[10px] font-mono text-cyan-400 bg-cyan-500/10 border border-cyan-500/30 px-1.5 py-0.5 rounded">
                        {(item.confidence * 100).toFixed(0)}% Confidence
                      </span>
                      {item.status === "accepted" && (
                        <span className="text-[10px] text-emerald-400 font-semibold uppercase">
                          ✓ Confirmed
                        </span>
                      )}
                      {item.status === "rejected" && (
                        <span className="text-[10px] text-rose-400 font-semibold uppercase">
                          ✕ Rejected
                        </span>
                      )}
                    </div>
                  </div>

                  {/* Explanation */}
                  <p className="text-xs text-[var(--text-primary)] leading-relaxed">
                    {item.explanation}
                  </p>

                  {/* Grounded Evidence Quote */}
                  <div className="p-2 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] space-y-1">
                    <div className="flex items-center justify-between text-[10px] text-[var(--text-tertiary)]">
                      <span className="flex items-center gap-1">
                        <FileText className="w-3 h-3 text-[var(--accent-primary)]" />
                        <span>Source Document: {item.source_document_name || "Extracted Evidence"}</span>
                      </span>
                      {item.source_document_id && (
                        <button
                          type="button"
                          onClick={() => onOpenDocument(item.source_document_id!)}
                          className="text-[var(--accent-primary)] hover:underline flex items-center gap-1 cursor-pointer"
                        >
                          <span>Open</span>
                          <ExternalLink className="w-2.5 h-2.5" />
                        </button>
                      )}
                    </div>
                    <p className="text-[11px] text-[var(--text-secondary)] italic">
                      "{item.evidence}"
                    </p>
                  </div>

                  {/* Decision Action Buttons (only for suggested status) */}
                  {item.status === "suggested" && (
                    <div className="flex items-center justify-end gap-2 pt-1 border-t border-[var(--border-subtle)]">
                      <button
                        type="button"
                        onClick={() => handleReject(item.id)}
                        disabled={isProcessing}
                        className="fluent-button text-xs py-1 px-3 text-rose-400 hover:bg-rose-500/10 border-rose-500/30 flex items-center gap-1.5 cursor-pointer"
                      >
                        <Ban className="w-3 h-3" />
                        <span>Reject</span>
                      </button>

                      <button
                        type="button"
                        onClick={() => handleAccept(item.id)}
                        disabled={isProcessing}
                        className="fluent-button text-xs py-1 px-3 bg-emerald-600 hover:bg-emerald-500 text-white flex items-center gap-1.5 cursor-pointer shadow-sm"
                      >
                        {isProcessing ? (
                          <Loader2 className="w-3 h-3 animate-spin" />
                        ) : (
                          <Check className="w-3 h-3" />
                        )}
                        <span>Approve & Connect</span>
                      </button>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};