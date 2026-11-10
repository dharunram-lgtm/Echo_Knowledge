import React, { memo } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Network, Cpu, BookOpen } from "lucide-react";
import type { ConceptRecord } from "@/types/tags";

export interface ConceptNodeData extends Record<string, unknown> {
  concept: ConceptRecord;
  degree: number;
  isSelected: boolean;
  isNeighbor: boolean;
  isDimmed: boolean;
  isSearchMatch: boolean;
  clusterColor?: string;
  isClusterMatch?: boolean;
}

export type ConceptNodeType = Node<ConceptNodeData, "conceptNode">;

export const ConceptNode: React.FC<NodeProps<ConceptNodeType>> = memo(({ data }) => {
  const {
    concept,
    degree,
    isSelected,
    isNeighbor,
    isDimmed,
    isSearchMatch,
    clusterColor,
    isClusterMatch,
  } = data;

  const renderIcon = (category: string) => {
    switch (category?.toLowerCase()) {
      case "technology":
        return <Cpu className="w-3.5 h-3.5 text-cyan-400" />;
      case "concept":
        return <Network className="w-3.5 h-3.5 text-[var(--accent-primary)]" />;
      default:
        return <BookOpen className="w-3.5 h-3.5 text-zinc-400" />;
    }
  };

  const getCategoryBadgeClass = (category: string) => {
    switch (category?.toLowerCase()) {
      case "technology":
        return "bg-cyan-500/10 text-cyan-400 border-cyan-500/30";
      case "concept":
        return "bg-blue-500/10 text-blue-400 border-blue-500/30";
      default:
        return "bg-zinc-500/10 text-zinc-300 border-zinc-500/30";
    }
  };

  return (
    <div
      className={`relative min-w-[150px] max-w-[220px] rounded-[var(--radius-lg)] p-2.5 transition-all duration-200 select-none cursor-pointer border ${isSelected
          ? "bg-[var(--bg-elevated)] border-[var(--accent-primary)] shadow-[var(--shadow-dialog)] ring-2 ring-[var(--accent-primary)]/50 scale-[1.03] z-30"
          : isClusterMatch
  ? "bg-[var(--bg-elevated)] shadow-[var(--shadow-flyout)] ring-2 scale-[1.02] z-25"
  : isNeighbor
    ? "bg-[var(--bg-card)] border-emerald-500/60 shadow-[var(--shadow-flyout)] ring-1 ring-emerald-500/30 z-20"
    : isSearchMatch
      ? "bg-[var(--bg-card)] border-amber-500/80 shadow-[var(--shadow-flyout)] ring-2 ring-amber-500/40 z-20"
      : "bg-[var(--bg-card)] border-[var(--border-default)] hover:border-[var(--border-strong)] hover:bg-[var(--bg-card-hover)] shadow-[var(--shadow-rest)] z-10"
      } ${ isDimmed ? "opacity-25 grayscale-[30%] pointer-events-none" : "opacity-100" } `}
      style={isClusterMatch && clusterColor ? { borderColor: clusterColor } : undefined}
    >
      {/* Multi-directional handles for clean connection angles */}
      <Handle type="target" position={Position.Top} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="source" position={Position.Top} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="target" position={Position.Bottom} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="source" position={Position.Bottom} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="target" position={Position.Left} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="source" position={Position.Left} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="target" position={Position.Right} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />
      <Handle type="source" position={Position.Right} className="!w-2 !h-2 !bg-[var(--border-strong)] !border-none opacity-0" />

      {/* Node Header */}
      <div className="flex items-center justify-between gap-1.5 pb-1">
        <div className="flex items-center gap-1.5 min-w-0">
          <div className="p-1 rounded-[var(--radius-sm)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] shrink-0">
            {renderIcon(concept.category)}
          </div>
          <span className="text-xs font-semibold text-[var(--text-primary)] truncate" title={concept.name}>
            {concept.name}
          </span>
          {clusterColor && (
            <span
              className="w-1.5 h-1.5 rounded-full shrink-0"
              style={{ backgroundColor: clusterColor }}
              title="Knowledge Cluster Member"
            />
          )}
        </div>

        {degree > 0 && (
          <span
            className="fluent-badge text-[9px] px-1 py-0.2 bg-[var(--bg-subtle)] text-[var(--text-tertiary)] border-[var(--border-subtle)] shrink-0 font-mono"
            title={`${ degree } connected relationship(s)`}
          >
            {degree}
          </span>
        )}
      </div>

      {/* Category Tag */}
      <div className="flex items-center justify-between gap-1 mt-1">
        <span className={`fluent - badge text - [9px] uppercase px - 1.5 py - 0.2 border ${ getCategoryBadgeClass(concept.category) } `}>
          {concept.category}
        </span>
        {isNeighbor && (
          <span className="text-[9px] text-emerald-400 font-medium tracking-tight">
            Connected
          </span>
        )}
      </div>

      {/* Description Snippet if available */}
      {concept.description && (
        <p className="text-[10px] text-[var(--text-tertiary)] truncate mt-1 italic border-t border-[var(--border-subtle)] pt-1">
          {concept.description}
        </p>
      )}
    </div>
  );
});

ConceptNode.displayName = "ConceptNode";