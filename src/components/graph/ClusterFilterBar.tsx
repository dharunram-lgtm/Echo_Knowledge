import React from "react";
import { Layers, RotateCcw } from "lucide-react";
import type { KnowledgeCluster } from "@/types/graph";

interface ClusterFilterBarProps {
  clusters: KnowledgeCluster[];
  selectedClusterId: string | null;
  onSelectCluster: (clusterId: string | null) => void;
  onRecomputeClusters: () => void;
  isRecomputing?: boolean;
}

export const ClusterFilterBar: React.FC<ClusterFilterBarProps> = ({
  clusters,
  selectedClusterId,
  onSelectCluster,
  onRecomputeClusters,
  isRecomputing = false,
}) => {
  if (clusters.length === 0) return null;

  return (
    <div className="flex items-center justify-between gap-2 px-3 py-2 bg-[var(--bg-elevated)] border border-[var(--border-default)] rounded-[var(--radius-md)] overflow-x-auto text-xs shadow-sm">
      <div className="flex items-center gap-1.5 shrink-0 pr-2 border-r border-[var(--border-subtle)]">
        <Layers className="w-3.5 h-3.5 text-emerald-400" />
        <span className="font-semibold text-[var(--text-secondary)]">Clusters:</span>
      </div>

      <div className="flex items-center gap-1.5 flex-1 overflow-x-auto py-0.5 scrollbar-thin">
        {/* All Clusters reset button */}
        <button
          type="button"
          onClick={() => onSelectCluster(null)}
          className={`fluent-badge text-[11px] py-1 px-2.5 rounded-full cursor-pointer transition-colors border shrink-0 ${
            selectedClusterId === null
              ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)]"
              : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
          }`}
        >
          All Clusters
        </button>

        {/* Cluster Pills */}
        {clusters.map((cl) => {
          const isSelected = selectedClusterId === cl.id;
          return (
            <button
              key={cl.id}
              type="button"
              onClick={() => onSelectCluster(isSelected ? null : cl.id)}
              className={`fluent-badge text-[11px] py-1 px-2.5 rounded-full cursor-pointer transition-all border flex items-center gap-1.5 shrink-0 ${
                isSelected
                  ? "text-white shadow-sm ring-1 ring-white/30"
                  : "bg-[var(--bg-card)] hover:bg-[var(--bg-card-hover)] text-[var(--text-secondary)] border-[var(--border-default)]"
              }`}
              style={{
                backgroundColor: isSelected ? cl.color : undefined,
                borderColor: isSelected ? cl.color : undefined,
              }}
              title={cl.description || cl.name}
            >
              <span
                className="w-2 h-2 rounded-full shrink-0"
                style={{ backgroundColor: isSelected ? "#ffffff" : cl.color }}
              />
              <span className="truncate max-w-[140px] font-medium">{cl.name}</span>
              <span className="text-[10px] opacity-75 font-mono">({cl.size})</span>
            </button>
          );
        })}
      </div>

      <button
        type="button"
        onClick={onRecomputeClusters}
        disabled={isRecomputing}
        className="fluent-button p-1 text-[var(--text-tertiary)] hover:text-[var(--text-primary)] shrink-0"
        title="Recompute clusters using graph community detection"
      >
        <RotateCcw className={`w-3.5 h-3.5 ${isRecomputing ? "animate-spin" : ""}`} />
      </button>
    </div>
  );
};

