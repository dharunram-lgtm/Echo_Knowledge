import React, { memo } from "react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
  type Edge,
} from "@xyflow/react";
import type { ConceptRelationshipRecord } from "@/types/graph";

export interface RelationshipEdgeData extends Record<string, unknown> {
  relationship: ConceptRelationshipRecord;
  isSelected?: boolean;
  isDimmed?: boolean;
}

export type RelationshipEdgeType = Edge<RelationshipEdgeData, "relationshipEdge">;

export const RELATIONSHIP_COLOR_MAP: Record<string, { stroke: string; bg: string; text: string; border: string }> = {
  USES: {
    stroke: "#4cc2ff",
    bg: "rgba(76, 194, 255, 0.12)",
    text: "#4cc2ff",
    border: "rgba(76, 194, 255, 0.4)",
  },
  IS_A: {
    stroke: "#6ccb5f",
    bg: "rgba(108, 203, 95, 0.12)",
    text: "#6ccb5f",
    border: "rgba(108, 203, 95, 0.4)",
  },
  PART_OF: {
    stroke: "#94a3b8",
    bg: "rgba(148, 163, 184, 0.12)",
    text: "#cbd5e1",
    border: "rgba(148, 163, 184, 0.4)",
  },
  RELATED_TO: {
    stroke: "#a1a1aa",
    bg: "rgba(161, 161, 170, 0.12)",
    text: "#d4d4d8",
    border: "rgba(161, 161, 170, 0.4)",
  },
  IMPLEMENTS: {
    stroke: "#2dd4bf",
    bg: "rgba(45, 212, 191, 0.12)",
    text: "#2dd4bf",
    border: "rgba(45, 212, 191, 0.4)",
  },
  REQUIRES: {
    stroke: "#f59e0b",
    bg: "rgba(245, 158, 11, 0.12)",
    text: "#fbbf24",
    border: "rgba(245, 158, 11, 0.4)",
  },
  COMPARED_WITH: {
    stroke: "#f43f5e",
    bg: "rgba(244, 63, 94, 0.12)",
    text: "#fda4af",
    border: "rgba(244, 63, 94, 0.4)",
  },
  INCLUDES: {
    stroke: "#06b6d4",
    bg: "rgba(6, 182, 212, 0.12)",
    text: "#67e8f9",
    border: "rgba(6, 182, 212, 0.4)",
  },
};

export const RelationshipEdge: React.FC<EdgeProps<RelationshipEdgeType>> = memo((props) => {
  const {
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    data,
    selected,
    markerEnd,
  } = props;

  const relationship = data?.relationship;
  const isSelected = selected || data?.isSelected;
  const isDimmed = data?.isDimmed;

  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const relType = relationship?.relationship_type || "RELATED_TO";
  const styling = RELATIONSHIP_COLOR_MAP[relType] || RELATIONSHIP_COLOR_MAP.RELATED_TO;

  const strokeWidth = isSelected ? 3 : 1.75;
  const strokeColor = isSelected ? "#ffffff" : styling.stroke;
  const opacity = isDimmed ? 0.15 : isSelected ? 1 : 0.75;

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        style={{
          stroke: strokeColor,
          strokeWidth,
          opacity,
          transition: "stroke 0.2s, stroke-width 0.2s, opacity 0.2s",
        }}
      />
      {relationship && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: "absolute",
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              pointerEvents: isDimmed ? "none" : "all",
              opacity,
              transition: "opacity 0.2s, transform 0.2s",
            }}
            className="nodrag nopan"
          >
            <div
              className={`fluent-badge text-[9px] font-semibold py-0.5 px-1.5 rounded-full border shadow-sm flex items-center gap-1 cursor-pointer select-none transition-all ${isSelected
                  ? "bg-white text-black border-white ring-2 ring-white/50 scale-105 z-30"
                  : "hover:scale-105 hover:border-[var(--border-strong)] z-10"
                }`}
              style={{
                backgroundColor: isSelected ? "#ffffff" : styling.bg,
                color: isSelected ? "#000000" : styling.text,
                borderColor: isSelected ? "#ffffff" : styling.border,
              }}
              title={`${relType} (${(relationship.confidence * 100).toFixed(0)}% confidence)\n"${relationship.evidence}"`}
            >
              <span>{relType}</span>
              <span className="opacity-75 font-mono text-[8px]">
                {(relationship.confidence * 100).toFixed(0)}%
              </span>
            </div>
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
});

RelationshipEdge.displayName = "RelationshipEdge";