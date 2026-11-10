import React, { useState, useEffect } from "react";
import {
  X,
  Sparkles,
  Loader2,
  BookOpen,
  ArrowRight,
  FileText,
  Lightbulb,
  GitCompare,
  HelpCircle,
  AlertCircle,
  ExternalLink,
} from "lucide-react";
import type { ConceptRecord } from "@/types/tags";
import type { ConceptExplorationResult } from "@/types/graph";
import { GraphService } from "@/services/graph/graphService";

interface ConceptExplorationModalProps {
  concept: ConceptRecord | null;
  isOpen: boolean;
  onClose: () => void;
  onOpenDocument: (documentId: string) => void;
  allConcepts?: ConceptRecord[];
}

export const ConceptExplorationModal: React.FC<ConceptExplorationModalProps> = ({
  concept,
  isOpen,
  onClose,
  onOpenDocument,
  allConcepts = [],
}) => {
  const [activeQueryType, setActiveQueryType] = useState<string>("explain");
  const [customQuestion, setCustomQuestion] = useState("");
  const [compareConceptId, setCompareConceptId] = useState<string>("");
  const [result, setResult] = useState<ConceptExplorationResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Available comparison candidates (excluding current concept)
  const otherConcepts = allConcepts.filter((c) => c.id !== concept?.id);

  const runExploration = async (queryType: string, customPrompt?: string, compareId?: string) => {
    if (!concept) return;
    try {
      setIsLoading(true);
      setError(null);
      setActiveQueryType(queryType);

      const res = await GraphService.exploreConcept(
        concept.id,
        queryType,
        customPrompt,
        compareId || undefined
      );
      setResult(res);
    } catch (err) {
      console.error("Failed to explore concept:", err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen && concept) {
      setResult(null);
      setError(null);
      setActiveQueryType("explain");
      runExploration("explain");
    }
  }, [isOpen, concept?.id]);

  if (!isOpen || !concept) return null;

  const handleCustomSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (customQuestion.trim()) {
      runExploration("custom", customQuestion.trim());
    }
  };

  const handleCompareSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const targetId = e.target.value;
    setCompareConceptId(targetId);
    if (targetId) {
      runExploration("compare", undefined, targetId);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
      <div className="fluent-card w-full max-w-3xl max-h-[85vh] flex flex-col bg-[var(--bg-elevated)] border border-[var(--border-strong)] shadow-[var(--shadow-dialog)] overflow-hidden">
        {/* Header */}
        <div className="p-4 border-b border-[var(--border-subtle)] flex items-center justify-between shrink-0">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-[var(--accent-primary)]/15 border border-[var(--accent-primary)]/30 flex items-center justify-center text-[var(--accent-primary)]">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-semibold text-[var(--text-primary)]">
                  Explore with ECHO
                </h2>
                <span className="fluent-badge text-[10px] py-0.2 px-2 bg-[var(--accent-primary)]/10 text-[var(--accent-primary)] font-mono">
                  {concept.name}
                </span>
                <span className="fluent-badge text-[9px] uppercase px-1.5 py-0.2">
                  {concept.category}
                </span>
              </div>
              <p className="text-xs text-[var(--text-secondary)]">
                AI-powered grounded synthesis scoped strictly to your personal documents.
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

        {/* Action Prompt Chips */}
        <div className="p-3 bg-[var(--bg-subtle)] border-b border-[var(--border-subtle)] flex flex-wrap items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={() => runExploration("explain")}
            disabled={isLoading}
            className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeQueryType === "explain"
                ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <Lightbulb className="w-3 h-3" />
            <span>Explain Concept</span>
          </button>

          <button
            type="button"
            onClick={() => runExploration("related")}
            disabled={isLoading}
            className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeQueryType === "related"
                ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <ArrowRight className="w-3 h-3" />
            <span>Related Topics</span>
          </button>

          <button
            type="button"
            onClick={() => runExploration("documents")}
            disabled={isLoading}
            className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeQueryType === "documents"
                ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <BookOpen className="w-3 h-3" />
            <span>Source Perspectives</span>
          </button>

          <button
            type="button"
            onClick={() => runExploration("evidence")}
            disabled={isLoading}
            className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeQueryType === "evidence"
                ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <FileText className="w-3 h-3" />
            <span>Direct Evidence Quotes</span>
          </button>

          <button
            type="button"
            onClick={() => runExploration("next_steps")}
            disabled={isLoading}
            className={`fluent-badge text-xs py-1 px-2.5 rounded-full cursor-pointer transition-colors border flex items-center gap-1.5 ${activeQueryType === "next_steps"
                ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
                : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
          >
            <HelpCircle className="w-3 h-3" />
            <span>Next Steps to Learn</span>
          </button>

          {/* Compare dropdown */}
          {otherConcepts.length > 0 && (
            <div className="flex items-center gap-1">
              <GitCompare className="w-3 h-3 text-[var(--text-tertiary)]" />
              <select
                value={compareConceptId}
                onChange={handleCompareSelect}
                disabled={isLoading}
                aria-label="Compare with another concept"
                className="fluent-input text-xs py-0.5 px-2 h-7 bg-[var(--bg-card)] border-[var(--border-default)] rounded-[var(--radius-sm)]"
              >
                <option value="">Compare with...</option>
                {otherConcepts.slice(0, 15).map((oc) => (
                  <option key={oc.id} value={oc.id}>
                    {oc.name}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>

        {/* Content Body */}
        <div className="p-4 flex-1 overflow-y-auto space-y-4 min-h-[300px]">
          {isLoading ? (
            <div className="h-64 flex flex-col items-center justify-center space-y-2 text-center">
              <Loader2 className="w-6 h-6 animate-spin text-[var(--accent-primary)]" />
              <span className="text-xs text-[var(--text-secondary)]">
                Synthesizing grounded answer using Gemini and document citations...
              </span>
            </div>
          ) : error ? (
              <div className="p-4 rounded-[var(--radius-md)] bg-[var(--status-danger)]/10 text-[var(--status-danger)] border border-[var(--status-danger)]/30 text-xs flex flex-col gap-3">
                <div className="flex items-start gap-2.5">
                  <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
                  <div className="space-y-1">
                    <p className="font-semibold">Unable to explore concept</p>
                    <p className="text-[11px] opacity-90 font-mono break-all">{error}</p>
                  </div>
                </div>
                <div className="flex items-center gap-2 pt-1 border-t border-[var(--status-danger)]/20">
                  <button
                    type="button"
                    onClick={() => runExploration(activeQueryType)}
                    className="fluent-button text-xs py-1 px-3 bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-primary)] border border-[var(--border-default)]"
                  >
                    Retry Exploration
                  </button>
                </div>
              </div>
              ) : result ? (
              <div className="space-y-4">
                {/* Answer Content */}
                <div className="p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2 text-xs leading-relaxed text-[var(--text-primary)]">
                  <div className="whitespace-pre-wrap">{result.response_text}</div>

                  {result.insufficient_context && (
                    <div className="pt-2 border-t border-[var(--border-subtle)] text-[11px] text-amber-400 flex items-center gap-1.5">
                      <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                      <span>
                        Notice: Limited document passages were found for this specific topic. No unsupported facts were invented.
                      </span>
                    </div>
                  )}
                </div>

                {/* Connected Concepts Tags */}
                {result.related_concepts.length > 0 && (
                  <div className="space-y-1.5">
                    <span className="text-[11px] font-semibold text-[var(--text-secondary)] uppercase">
                      Connected Concepts in Graph ({result.related_concepts.length})
                    </span>
                    <div className="flex flex-wrap gap-1.5">
                      {result.related_concepts.map((rc, idx) => (
                        <span
                          key={idx}
                          className="fluent-badge text-[11px] py-0.5 px-2 bg-[var(--bg-card)] border border-[var(--border-default)] text-[var(--text-primary)] font-mono"
                        >
                          {rc}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {/* Citations & Evidence Passages */}
                {result.citations.length > 0 && (
                  <div className="space-y-2 pt-2 border-t border-[var(--border-subtle)]">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
                        <BookOpen className="w-3.5 h-3.5 text-emerald-400" />
                        <span>Authentic Supporting Citations ({result.citations.length})</span>
                      </span>
                      <span className="text-[10px] text-[var(--text-tertiary)]">
                        Grounded in local document chunks
                      </span>
                    </div>

                    <div className="space-y-2">
                      {result.citations.map((c, i) => (
                        <div
                          key={i}
                          className="p-2.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1.5 hover:border-[var(--border-strong)] transition-colors"
                        >
                          <div className="flex items-center justify-between gap-2">
                            <span className="text-xs font-medium text-[var(--text-primary)] flex items-center gap-1.5">
                              <FileText className="w-3 h-3 text-[var(--accent-primary)]" />
                              <span>{c.filename}</span>
                              <span className="text-[10px] text-[var(--text-tertiary)] font-mono">
                                (Chunk #{c.chunk_index})
                              </span>
                            </span>

                            <button
                              type="button"
                              onClick={() => onOpenDocument(c.document_id)}
                              className="text-[10px] text-[var(--accent-primary)] hover:underline flex items-center gap-1 cursor-pointer"
                            >
                              <span>View Source</span>
                              <ExternalLink className="w-2.5 h-2.5" />
                            </button>
                          </div>

                          <p className="text-[11px] text-[var(--text-secondary)] italic leading-relaxed pl-2 border-l-2 border-[var(--accent-primary)]/40">
                            "{c.snippet}"
                          </p>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
          ) : null}
            </div>

        {/* Custom Question Footer */}
          <form
            onSubmit={handleCustomSubmit}
            className="p-3 bg-[var(--bg-subtle)] border-t border-[var(--border-subtle)] flex items-center gap-2 shrink-0"
          >
            <input
              type="text"
              value={customQuestion}
              onChange={(e) => setCustomQuestion(e.target.value)}
              placeholder={`Ask a specific question about ${concept.name}...`}
              className="fluent-input text-xs py-1.5 px-3 flex-1 h-8"
              disabled={isLoading}
            />
            <button
              type="submit"
              disabled={isLoading || !customQuestion.trim()}
              className="fluent-button text-xs py-1.5 px-3 bg-[var(--accent-primary)] text-white hover:opacity-90 flex items-center gap-1.5 shrink-0"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Ask</span>
            </button>
          </form>
        </div>
      </div>
      );
};

