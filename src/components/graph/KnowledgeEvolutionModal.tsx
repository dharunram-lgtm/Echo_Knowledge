import React, { useState, useEffect } from "react";
import {
  X,
  History,
  FileText,
  Network,
  Share2,
  AlertTriangle,
  Clock,
  ExternalLink,
  Loader2,
} from "lucide-react";
import type { TimelineEvent, PotentialConflict } from "@/types/graph";
import { GraphService } from "@/services/graph/graphService";

interface KnowledgeEvolutionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenDocument: (documentId: string) => void;
  onFocusConcept: (conceptId: string) => void;
}

export const KnowledgeEvolutionModal: React.FC<KnowledgeEvolutionModalProps> = ({
  isOpen,
  onClose,
  onOpenDocument,
  onFocusConcept,
}) => {
  const [activeTab, setActiveTab] = useState<"timeline" | "conflicts">("timeline");
  const [timelineEvents, setTimelineEvents] = useState<TimelineEvent[]>([]);
  const [conflicts, setConflicts] = useState<PotentialConflict[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadData();
    }
  }, [isOpen]);

  const loadData = async () => {
    try {
      setIsLoading(true);
      const [events, detectedConflicts] = await Promise.all([
        GraphService.getKnowledgeEvolutionTimeline().catch(() => []),
        GraphService.detectPotentialConflicts().catch(() => []),
      ]);
      setTimelineEvents(events);
      setConflicts(detectedConflicts);
    } catch (err) {
      console.error("Failed to load knowledge evolution data:", err);
    } finally {
      setIsLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
      <div className="fluent-card w-full max-w-4xl max-h-[85vh] flex flex-col bg-[var(--bg-elevated)] border border-[var(--border-strong)] shadow-[var(--shadow-dialog)] overflow-hidden">
        {/* Header */}
        <div className="p-4 border-b border-[var(--border-subtle)] flex items-center justify-between shrink-0">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
              <History className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-semibold text-[var(--text-primary)]">
                  Knowledge Evolution & Conflict Analysis
                </h2>
                <span className="fluent-badge text-[10px] py-0.2 px-2 bg-emerald-500/10 text-emerald-400 font-mono">
                  Phase 4.3 Provenance
                </span>
              </div>
              <p className="text-xs text-[var(--text-secondary)]">
                Track historical knowledge growth, document contributions, and inspect diverging concept evidence.
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

        {/* Tab Selector */}
        <div className="p-2.5 bg-[var(--bg-subtle)] border-b border-[var(--border-subtle)] flex items-center gap-2 shrink-0">
          <button
            type="button"
            onClick={() => setActiveTab("timeline")}
            className={`fluent-badge text-xs py-1.5 px-3 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeTab === "timeline"
                ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                : "bg-[var(--bg-card)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <Clock className="w-3.5 h-3.5" />
            <span>Development Timeline ({timelineEvents.length})</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab("conflicts")}
            className={`fluent-badge text-xs py-1.5 px-3 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeTab === "conflicts"
                ? "bg-amber-600 text-white border-amber-600"
                : "bg-[var(--bg-card)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
            <span>Potential Divergences / Conflicts ({conflicts.length})</span>
          </button>
        </div>

        {/* Tab Body */}
        <div className="p-4 flex-1 overflow-y-auto space-y-3 min-h-[350px]">
          {isLoading ? (
            <div className="h-64 flex flex-col items-center justify-center space-y-2 text-center">
              <Loader2 className="w-6 h-6 animate-spin text-[var(--accent-primary)]" />
              <span className="text-xs text-[var(--text-secondary)]">Loading knowledge chronology...</span>
            </div>
          ) : activeTab === "timeline" ? (
            /* Timeline View */
            timelineEvents.length === 0 ? (
              <div className="p-8 text-center text-xs text-[var(--text-secondary)]">
                No evolution history recorded yet.
              </div>
            ) : (
              <div className="relative pl-6 space-y-4 before:absolute before:left-2.5 before:top-2 before:bottom-2 before:w-0.5 before:bg-[var(--border-subtle)]">
                {timelineEvents.map((ev) => {
                  const isDoc = ev.event_type === "document_imported";
                  const isConcept = ev.event_type === "concept_discovered";
                  const isRel = ev.event_type === "relationship_added";

                  return (
                    <div key={ev.id} className="relative group">
                      {/* Node Bullet */}
                      <div
                        className={`absolute -left-6 top-1 w-3 h-3 rounded-full border-2 bg-[var(--bg-app)] ${isDoc
                            ? "border-blue-400"
                            : isConcept
                              ? "border-emerald-400"
                              : "border-cyan-400"
                          }`}
                      />

                      <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] hover:border-[var(--border-strong)] transition-all space-y-1">
                        <div className="flex items-center justify-between gap-2">
                          <div className="flex items-center gap-1.5">
                            {isDoc && <FileText className="w-3.5 h-3.5 text-blue-400" />}
                            {isConcept && <Network className="w-3.5 h-3.5 text-emerald-400" />}
                            {isRel && <Share2 className="w-3.5 h-3.5 text-cyan-400" />}
                            <span className="text-xs font-semibold text-[var(--text-primary)]">
                              {ev.title}
                            </span>
                          </div>

                          <span className="text-[10px] text-[var(--text-tertiary)] font-mono">
                            {ev.timestamp.replace("T", " ").slice(0, 16)}
                          </span>
                        </div>

                        <p className="text-xs text-[var(--text-secondary)]">
                          {ev.description}
                        </p>

                        <div className="flex items-center gap-3 pt-1 text-[10px]">
                          {ev.document_id && (
                            <button
                              type="button"
                              onClick={() => {
                                onOpenDocument(ev.document_id!);
                                onClose();
                              }}
                              className="text-[var(--accent-primary)] hover:underline flex items-center gap-1 cursor-pointer"
                            >
                              <span>View Document</span>
                              <ExternalLink className="w-2.5 h-2.5" />
                            </button>
                          )}
                          {ev.concept_id && (
                            <button
                              type="button"
                              onClick={() => {
                                onFocusConcept(ev.concept_id!);
                                onClose();
                              }}
                              className="text-emerald-400 hover:underline flex items-center gap-1 cursor-pointer"
                            >
                              <span>Focus Concept in Graph</span>
                              <ExternalLink className="w-2.5 h-2.5" />
                            </button>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )
          ) : (
            /* Conflicts / Divergence View */
            conflicts.length === 0 ? (
              <div className="p-8 text-center text-xs text-[var(--text-secondary)] space-y-2">
                <AlertTriangle className="w-8 h-8 text-[var(--text-tertiary)] mx-auto opacity-40" />
                <h4 className="text-xs font-semibold text-[var(--text-primary)]">
                  No Contradictory Evidence Detected
                </h4>
                <p className="text-[11px] text-[var(--text-tertiary)] max-w-sm mx-auto">
                  When different documents present divergent or opposing claims regarding the same concept, ECHO surfaces them here for human review.
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="p-3 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-xs text-amber-300 flex items-start gap-2">
                  <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5" />
                  <span>
                    ECHO identifies potential evidence divergences across your documents. ECHO never silently decides which statement is correct; inspect the source excerpts below to evaluate context.
                  </span>
                </div>

                {conflicts.map((c) => (
                  <div
                    key={c.id}
                    className="p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-3"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-bold text-[var(--text-primary)]">
                          Concept: {c.concept_name}
                        </span>
                        <span className="fluent-badge text-[9px] py-0.2 px-2 bg-amber-500/15 text-amber-400 border-amber-500/30 uppercase font-mono">
                          Potential Divergence
                        </span>
                      </div>

                      <button
                        type="button"
                        onClick={() => {
                          onFocusConcept(c.concept_id);
                          onClose();
                        }}
                        className="text-[10px] text-[var(--accent-primary)] hover:underline flex items-center gap-1 cursor-pointer"
                      >
                        <span>Focus in Graph</span>
                        <ExternalLink className="w-2.5 h-2.5" />
                      </button>
                    </div>

                    <p className="text-xs text-[var(--text-secondary)]">
                      {c.explanation}
                    </p>

                    {/* Side-by-side or stacked statement comparison */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                      {/* Statement A */}
                      <div className="p-2.5 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] space-y-1.5">
                        <div className="flex items-center justify-between text-[10px] text-[var(--text-tertiary)]">
                          <span className="font-medium text-[var(--text-primary)]">
                            Source A: {c.doc_a_filename}
                          </span>
                          {c.doc_a_id && (
                            <button
                              type="button"
                              onClick={() => onOpenDocument(c.doc_a_id!)}
                              className="text-[var(--accent-primary)] hover:underline"
                            >
                              Inspect
                            </button>
                          )}
                        </div>
                        <p className="text-[11px] text-[var(--text-secondary)] italic leading-relaxed pl-2 border-l-2 border-blue-400">
                          "{c.statement_a}"
                        </p>
                      </div>

                      {/* Statement B */}
                      <div className="p-2.5 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] space-y-1.5">
                        <div className="flex items-center justify-between text-[10px] text-[var(--text-tertiary)]">
                          <span className="font-medium text-[var(--text-primary)]">
                            Source B: {c.doc_b_filename}
                          </span>
                          {c.doc_b_id && (
                            <button
                              type="button"
                              onClick={() => onOpenDocument(c.doc_b_id!)}
                              className="text-[var(--accent-primary)] hover:underline"
                            >
                              Inspect
                            </button>
                          )}
                        </div>
                        <p className="text-[11px] text-[var(--text-secondary)] italic leading-relaxed pl-2 border-l-2 border-cyan-400">
                          "{c.statement_b}"
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )
          )}
        </div>
      </div>
    </div>
  );
};