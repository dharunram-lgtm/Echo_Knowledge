import React from "react";
import {
  Search,
  X,
  ZoomIn,
  ZoomOut,
  Maximize2,
  RotateCcw,
  SlidersHorizontal,
} from "lucide-react";
import type { RelationshipType } from "@/types/graph";
import { RELATIONSHIP_COLOR_MAP } from "./RelationshipEdge";

interface GraphToolbarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  selectedType: RelationshipType | "ALL";
  onSelectType: (type: RelationshipType | "ALL") => void;
  confidenceThreshold: number;
  onConfidenceChange: (threshold: number) => void;
  connectedOnly: boolean;
  onConnectedOnlyToggle: (connectedOnly: boolean) => void;
  maxNodes: number;
  onMaxNodesChange: (limit: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitView: () => void;
  onResetLayout: () => void;
  onClearFilters: () => void;
  isFiltersActive: boolean;
}

const AVAILABLE_TYPES: { type: RelationshipType | "ALL"; label: string }[] = [
  { type: "ALL", label: "All Types" },
  { type: "USES", label: "Uses" },
  { type: "IS_A", label: "Is A" },
  { type: "PART_OF", label: "Part Of" },
  { type: "RELATED_TO", label: "Related To" },
  { type: "IMPLEMENTS", label: "Implements" },
  { type: "REQUIRES", label: "Requires" },
  { type: "COMPARED_WITH", label: "Compared With" },
  { type: "INCLUDES", label: "Includes" },
];

export const GraphToolbar: React.FC<GraphToolbarProps> = ({
  searchQuery,
  onSearchChange,
  selectedType,
  onSelectType,
  confidenceThreshold,
  onConfidenceChange,
  connectedOnly,
  onConnectedOnlyToggle,
  maxNodes,
  onMaxNodesChange,
  onZoomIn,
  onZoomOut,
  onFitView,
  onResetLayout,
  onClearFilters,
  isFiltersActive,
}) => {
  return (
    <div className="fluent-card p-3 space-y-2.5 bg-[var(--bg-elevated)] border border-[var(--border-default)] shadow-[var(--shadow-rest)]">
      {/* Top Row: Search + Viewport Controls + Reset Layout */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
        {/* Concept Search Input */}
        <div className="relative flex-1 max-w-md">
          <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)]" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search concepts or evidence quotes..."
            className="fluent-input pl-9 pr-8 text-xs py-1.5 w-full h-8"
          />
          {searchQuery && (
            <button
              type="button"
              onClick={() => onSearchChange("")}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
              title="Clear search"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>

        {/* Canvas Controls */}
        <div className="flex items-center gap-1.5 self-end sm:self-auto">
          <button
            type="button"
            onClick={onZoomIn}
            className="fluent-button p-1.5 text-xs h-8 w-8 flex items-center justify-center"
            title="Zoom in (or use mouse wheel)"
          >
            <ZoomIn className="w-3.5 h-3.5" />
          </button>
          <button
            type="button"
            onClick={onZoomOut}
            className="fluent-button p-1.5 text-xs h-8 w-8 flex items-center justify-center"
            title="Zoom out (or use mouse wheel)"
          >
            <ZoomOut className="w-3.5 h-3.5" />
          </button>
          <button
            type="button"
            onClick={onFitView}
            className="fluent-button p-1.5 text-xs h-8 w-8 flex items-center justify-center"
            title="Fit all visible nodes to view"
          >
            <Maximize2 className="w-3.5 h-3.5" />
          </button>
          <button
            type="button"
            onClick={onResetLayout}
            className="fluent-button text-xs py-1 px-2.5 h-8 flex items-center gap-1.5"
            title="Recalculate force-directed layout positions"
          >
            <RotateCcw className="w-3 h-3" />
            <span className="hidden md:inline">Reset Layout</span>
          </button>
        </div>
      </div>

      {/* Bottom Row: Filters (Relationship Type, Confidence, Connected Only, Limit) */}
      <div className="flex flex-wrap items-center justify-between gap-3 pt-1 border-t border-[var(--border-subtle)] text-xs">
        {/* Type Filter Buttons */}
        <div className="flex flex-wrap items-center gap-1">
          {AVAILABLE_TYPES.map((item) => {
            const isSelected = selectedType === item.type;
            const style = item.type !== "ALL" ? RELATIONSHIP_COLOR_MAP[item.type] : null;

            return (
              <button
                key={item.type}
                type="button"
                onClick={() => onSelectType(item.type)}
                className={`fluent-badge text-[10px] py-1 px-2 rounded-full cursor-pointer transition-all border ${isSelected
                    ? "bg-[var(--accent-primary)] text-white border-[var(--accent-primary)] font-semibold shadow-sm"
                    : "bg-[var(--bg-subtle)] text-[var(--text-secondary)] border-[var(--border-subtle)] hover:border-[var(--border-strong)]"
                  }`}
                style={
                  isSelected && style
                    ? { backgroundColor: style.stroke, borderColor: style.stroke, color: "#000000" }
                    : undefined
                }
              >
                {item.label}
              </button>
            );
          })}
        </div>

        {/* Secondary Filters Bar */}
        <div className="flex flex-wrap items-center gap-3">
          {/* Confidence Slider */}
          <div className="flex items-center gap-1.5 text-[11px] text-[var(--text-secondary)]">
            <SlidersHorizontal className="w-3 h-3 text-[var(--text-tertiary)]" />
            <span>Min Confidence:</span>
            <input
              type="range"
              min="60"
              max="100"
              step="5"
              value={confidenceThreshold}
              onChange={(e) => onConfidenceChange(Number(e.target.value))}
              className="w-20 accent-[var(--accent-primary)] cursor-pointer"
              title={`Minimum confidence: ${confidenceThreshold}%`}
            />
            <span className="font-mono text-[10px] text-[var(--text-primary)] w-7 text-right">
              {confidenceThreshold}%
            </span>
          </div>

          {/* Connected Only Checkbox */}
          <label className="flex items-center gap-1.5 text-[11px] text-[var(--text-secondary)] cursor-pointer select-none">
            <input
              type="checkbox"
              checked={connectedOnly}
              onChange={(e) => onConnectedOnlyToggle(e.target.checked)}
              className="rounded-[var(--radius-xs)] accent-[var(--accent-primary)]"
            />
            <span>Connected only</span>
          </label>

          {/* Node Limit Dropdown */}
          <div className="flex items-center gap-1 text-[11px] text-[var(--text-secondary)]">
            <span className="text-[var(--text-tertiary)]">Limit:</span>
            <select
              value={maxNodes}
              onChange={(e) => onMaxNodesChange(Number(e.target.value))}
              className="fluent-input text-[11px] py-0.5 px-1.5 h-6.5 bg-[var(--bg-card)] border-[var(--border-default)] rounded-[var(--radius-sm)]"
            >
              <option value="50">50</option>
              <option value="100">100</option>
              <option value="200">200</option>
              <option value="500">500</option>
            </select>
          </div>

          {/* Clear Filters Action */}
          {isFiltersActive && (
            <button
              type="button"
              onClick={onClearFilters}
              className="text-[10px] text-[var(--accent-primary)] hover:underline cursor-pointer"
            >
              Clear Filters
            </button>
          )}
        </div>
      </div>
    </div>
  );
};