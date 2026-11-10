import React, { useState, useEffect } from "react";
import {
  X,
  BarChart3,
  Network,
  Layers,
  FileText,
  Loader2,
  TrendingUp,
} from "lucide-react";
import type { KnowledgeInsights } from "@/types/graph";
import { GraphService } from "@/services/graph/graphService";

interface KnowledgeInsightsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onFocusConcept: (conceptId: string) => void;
  onOpenDocument: (documentId: string) => void;
  onSelectCluster?: (clusterId: string) => void;
}

export const KnowledgeInsightsModal: React.FC<KnowledgeInsightsModalProps> = ({
  isOpen,
  onClose,
  onFocusConcept,
  onOpenDocument,
  onSelectCluster,
}) => {
  const [insights, setInsights] = useState<KnowledgeInsights | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadInsights();
    }
  }, [isOpen]);

  const loadInsights = async () => {
    try {
      setIsLoading(true);
      const data = await GraphService.getKnowledgeInsights();
      setInsights(data);
    } catch (err) {
      console.error("Failed to load knowledge insights:", err);
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
            <div className="w-8 h-8 rounded-lg bg-[var(--accent-primary)]/15 border border-[var(--accent-primary)]/30 flex items-center justify-center text-[var(--accent-primary)]">
              <BarChart3 className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-semibold text-[var(--text-primary)]">
                  Knowledge Intelligence Insights
                </h2>
                <span className="fluent-badge text-[10px] py-0.2 px-2 bg-[var(--accent-primary)]/10 text-[var(--accent-primary)] font-mono">
                  Phase 4.3 Analytics
                </span>
              </div>
              <p className="text-xs text-[var(--text-secondary)]">
                Live metrics, hub concepts, and community distributions derived from your local repository.
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

        {/* Content Body */}
        <div className="p-4 flex-1 overflow-y-auto space-y-4 min-h-[350px]">
          {isLoading || !insights ? (
            <div className="h-64 flex flex-col items-center justify-center space-y-2 text-center">
              <Loader2 className="w-6 h-6 animate-spin text-[var(--accent-primary)]" />
              <span className="text-xs text-[var(--text-secondary)]">
                Aggregating graph analytics from SQLite...
              </span>
            </div>
          ) : (
            <>
              {/* Primary Metric Tiles */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
                  <span className="text-[10px] uppercase font-semibold text-[var(--text-tertiary)] block">
                    Total Concepts
                  </span>
                  <span className="text-xl font-bold text-[var(--text-primary)]">
                    {insights.total_concepts}
                  </span>
                </div>

                <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
                  <span className="text-[10px] uppercase font-semibold text-cyan-400 block">
                    Confirmed Edges
                  </span>
                  <span className="text-xl font-bold text-cyan-400">
                    {insights.total_confirmed_relationships}
                  </span>
                </div>

                <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
                  <span className="text-[10px] uppercase font-semibold text-emerald-400 block">
                    Knowledge Clusters
                  </span>
                  <span className="text-xl font-bold text-emerald-400">
                    {insights.total_clusters}
                  </span>
                </div>

                <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1">
                  <span className="text-[10px] uppercase font-semibold text-amber-400 block">
                    Hidden Suggestions
                  </span>
                  <span className="text-xl font-bold text-amber-400">
                    {insights.total_suggested_relationships}
                  </span>
                </div>
              </div>

              {/* Grid: Most Connected Concepts & Largest Clusters */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Most Connected Concepts (Hubs) */}
                <div className="fluent-card p-3.5 bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2.5">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
                      <Network className="w-3.5 h-3.5 text-cyan-400" />
                      <span>Most Connected Hubs</span>
                    </span>
                    <span className="text-[10px] text-[var(--text-tertiary)]">Degree</span>
                  </div>

                  {insights.most_connected_concepts.length === 0 ? (
                    <div className="p-3 text-center text-xs text-[var(--text-tertiary)]">
                      No connected hubs yet.
                    </div>
                  ) : (
                    <div className="space-y-1.5">
                      {insights.most_connected_concepts.map(([id, name, deg]) => (
                        <div
                          key={id}
                          onClick={() => {
                            onFocusConcept(id);
                            onClose();
                          }}
                          className="p-2 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] hover:border-[var(--accent-primary)] cursor-pointer flex items-center justify-between transition-colors"
                        >
                          <span className="text-xs font-medium text-[var(--text-primary)] truncate">
                            {name}
                          </span>
                          <span className="text-[11px] font-mono font-bold text-cyan-400">
                            {deg} edges
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Largest Knowledge Clusters */}
                <div className="fluent-card p-3.5 bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2.5">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
                      <Layers className="w-3.5 h-3.5 text-emerald-400" />
                      <span>Largest Clusters</span>
                    </span>
                    <span className="text-[10px] text-[var(--text-tertiary)]">Members</span>
                  </div>

                  {insights.largest_clusters.length === 0 ? (
                    <div className="p-3 text-center text-xs text-[var(--text-tertiary)]">
                      No clusters detected yet.
                    </div>
                  ) : (
                    <div className="space-y-1.5">
                      {insights.largest_clusters.map(([id, name, count, color]) => (
                        <div
                          key={id}
                          onClick={() => {
                            if (onSelectCluster) onSelectCluster(id);
                            onClose();
                          }}
                          className="p-2 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] hover:border-[var(--border-strong)] cursor-pointer flex items-center justify-between transition-colors"
                        >
                          <div className="flex items-center gap-2 truncate">
                            <span
                              className="w-2.5 h-2.5 rounded-full shrink-0"
                              style={{ backgroundColor: color }}
                            />
                            <span className="text-xs font-medium text-[var(--text-primary)] truncate">
                              {name}
                            </span>
                          </div>
                          <span className="text-[11px] font-mono text-[var(--text-secondary)]">
                            {count} concepts
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              {/* Grid: Top Contributing Documents & Isolated Concepts */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Documents Contributing the Most Concepts */}
                <div className="fluent-card p-3.5 bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2.5">
                  <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
                    <FileText className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
                    <span>Top Concept Sources</span>
                  </span>

                  {insights.top_source_documents.length === 0 ? (
                    <div className="p-3 text-center text-xs text-[var(--text-tertiary)]">
                      No document associations found.
                    </div>
                  ) : (
                    <div className="space-y-1.5">
                      {insights.top_source_documents.map(([id, filename, count]) => (
                        <div
                          key={id}
                          onClick={() => {
                            onOpenDocument(id);
                            onClose();
                          }}
                          className="p-2 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] hover:border-[var(--accent-primary)] cursor-pointer flex items-center justify-between transition-colors"
                        >
                          <span className="text-xs font-medium text-[var(--text-primary)] truncate">
                            {filename}
                          </span>
                          <span className="text-[10px] font-mono text-[var(--accent-primary)]">
                            {count} concepts
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Isolated Concepts Needing Connections */}
                <div className="fluent-card p-3.5 bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2.5">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
                      <TrendingUp className="w-3.5 h-3.5 text-amber-400" />
                      <span>Isolated Concepts (0 Connections)</span>
                    </span>
                    <span className="text-[10px] text-[var(--text-tertiary)]">
                      {insights.isolated_concepts.length} total
                    </span>
                  </div>

                  {insights.isolated_concepts.length === 0 ? (
                    <div className="p-3 text-center text-xs text-emerald-400">
                      All concepts are connected into the knowledge graph!
                    </div>
                  ) : (
                    <div className="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto">
                      {insights.isolated_concepts.map(([id, name]) => (
                        <button
                          key={id}
                          type="button"
                          onClick={() => {
                            onFocusConcept(id);
                            onClose();
                          }}
                          className="fluent-badge text-[11px] py-0.5 px-2 bg-[var(--bg-card)] border border-[var(--border-default)] hover:border-[var(--accent-primary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] cursor-pointer transition-colors"
                        >
                          {name}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
