import React from "react";
import {
  X,
  Share2,
  BookOpen,
  FileText,
  Trash2,
  ArrowRight,
  ExternalLink,
  Target,
  Sparkles,
  Info,
} from "lucide-react";
import type {
  ConceptRelationshipRecord,
  ConceptNeighborRecord,
  ConceptDocumentRecord,
} from "@/types/graph";
import type { ConceptRecord } from "@/types/tags";
import { RELATIONSHIP_COLOR_MAP } from "./RelationshipEdge";

interface GraphInspectorPanelProps {
  selectedConcept: ConceptRecord | null;
  selectedEdge: ConceptRelationshipRecord | null;
  neighbors: ConceptNeighborRecord[];
  conceptDocs: ConceptDocumentRecord[];
  onClose: () => void;
  onSelectConcept: (conceptId: string) => void;
  onOpenDocument: (documentId: string) => void;
  onDeleteRelationship: (relationshipId: string) => void;
  onFocusConcept?: (conceptId: string) => void;
  onExploreConcept?: (concept: ConceptRecord) => void;
}

export const GraphInspectorPanel: React.FC<GraphInspectorPanelProps> = ({
  selectedConcept,
  selectedEdge,
  neighbors,
  conceptDocs,
  onClose,
  onSelectConcept,
  onOpenDocument,
  onDeleteRelationship,
  onFocusConcept,
  onExploreConcept,
}) => {
  if (!selectedConcept && !selectedEdge) {
    return (
      <div className="fluent-card p-5 h-full flex flex-col items-center justify-center text-center space-y-3 bg-[var(--bg-elevated)] border border-[var(--border-strong)]">
        <div className="w-10 h-10 rounded-full bg-[var(--bg-subtle)] border border-[var(--border-default)] flex items-center justify-center text-[var(--accent-primary)]">
          <Info className="w-5 h-5" />
        </div>
        <div className="space-y-1">
          <h3 className="text-xs font-semibold text-[var(--text-primary)]">
            Inspector Ready
          </h3>
          <p className="text-[11px] text-[var(--text-secondary)] max-w-xs leading-relaxed">
            Click any concept node or connecting relationship on the canvas to inspect evidence, confidence scores, and source documents.
          </p>
        </div>
      </div>
    );
  }

  // --- Edge Inspection View ---
  if (selectedEdge) {
    const relType = selectedEdge.relationship_type;
    const style = RELATIONSHIP_COLOR_MAP[relType] || RELATIONSHIP_COLOR_MAP.RELATED_TO;

    return (
      <div className="fluent-card p-4 h-full flex flex-col space-y-4 bg-[var(--bg-elevated)] border border-[var(--border-strong)] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
          <div className="flex items-center gap-2">
            <span
              className="fluent-badge text-[10px] font-semibold py-0.5 px-2 border"
              style={{
                backgroundColor: style.bg,
                color: style.text,
                borderColor: style.border,
              }}
            >
              {relType}
            </span>
            <span className="text-xs font-semibold text-[var(--text-primary)]">
              Relationship Evidence
            </span>
          </div>

          <button
            type="button"
            onClick={onClose}
            className="fluent-button p-1 text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
            title="Close inspector"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Concept Connection Banner */}
        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2">
          <div className="text-[10px] text-[var(--text-tertiary)] uppercase font-semibold">
            Connected Concepts
          </div>
          <div className="flex items-center justify-between gap-2">
            <button
              type="button"
              onClick={() => onSelectConcept(selectedEdge.source_concept_id)}
              className="text-xs font-semibold text-[var(--text-primary)] hover:underline truncate max-w-[130px] text-left cursor-pointer"
              title={selectedEdge.source_concept_name || "Source concept"}
            >
              {selectedEdge.source_concept_name || "Source"}
            </button>

            <ArrowRight className="w-3.5 h-3.5 text-[var(--text-tertiary)] shrink-0" />

            <button
              type="button"
              onClick={() => onSelectConcept(selectedEdge.target_concept_id)}
              className="text-xs font-semibold text-[var(--text-primary)] hover:underline truncate max-w-[130px] text-right cursor-pointer"
              title={selectedEdge.target_concept_name || "Target concept"}
            >
              {selectedEdge.target_concept_name || "Target"}
            </button>
          </div>
        </div>

        {/* Confidence Score */}
        <div className="space-y-1.5">
          <div className="flex items-center justify-between text-xs">
            <span className="text-[var(--text-secondary)] font-medium">Confidence Score</span>
            <span className="font-mono font-bold text-[var(--text-primary)]">
              {(selectedEdge.confidence * 100).toFixed(0)}%
            </span>
          </div>
          <div className="w-full h-1.5 rounded-full bg-[var(--bg-subtle)] overflow-hidden">
            <div
              className="h-full rounded-full transition-all duration-300"
              style={{
                width: `${Math.min(100, Math.max(0, selectedEdge.confidence * 100))}%`,
                backgroundColor: style.stroke,
              }}
            />
          </div>
        </div>

        {/* Supporting Evidence Quote */}
        <div className="space-y-1.5">
          <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
            <Sparkles className="w-3.5 h-3.5 text-amber-400" />
            <span>Grounded Source Evidence</span>
          </span>
          <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-card)] border border-[var(--border-subtle)] text-xs text-[var(--text-secondary)] italic leading-relaxed select-text">
            "{selectedEdge.evidence}"
          </div>
        </div>

        {/* Source Document Reference */}
        {selectedEdge.source_document_id && (
          <div className="space-y-1.5">
            <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
              <FileText className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
              <span>Source Document</span>
            </span>
            <div className="p-2.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] flex items-center justify-between gap-2">
              <div className="min-w-0">
                <span className="text-xs font-medium text-[var(--text-primary)] truncate block">
                  {selectedEdge.source_document_name || "Document"}
                </span>
                <span className="text-[10px] text-[var(--text-tertiary)] font-mono">
                  Verified in local SQLite repository
                </span>
              </div>
              <button
                type="button"
                onClick={() => onOpenDocument(selectedEdge.source_document_id!)}
                className="fluent-button text-xs py-1 px-2.5 shrink-0 flex items-center gap-1"
                title="Open document content preview"
              >
                <ExternalLink className="w-3 h-3" />
                <span>View</span>
              </button>
            </div>
          </div>
        )}

        {/* Metadata Footer */}
        <div className="pt-2 border-t border-[var(--border-subtle)] flex items-center justify-between text-[10px] text-[var(--text-tertiary)]">
          <span className="font-mono uppercase">Method: {selectedEdge.discovery_method}</span>
          <button
            type="button"
            onClick={() => onDeleteRelationship(selectedEdge.id)}
            className="text-[var(--text-tertiary)] hover:text-[var(--status-danger)] flex items-center gap-1 cursor-pointer"
            title="Delete this relationship"
          >
            <Trash2 className="w-3 h-3" />
            <span>Delete</span>
          </button>
        </div>
      </div>
    );
  }

  // --- Concept Inspection View ---
  return (
    <div className="fluent-card p-4 h-full flex flex-col space-y-4 bg-[var(--bg-elevated)] border border-[var(--border-strong)] overflow-y-auto">
      {/* Header */}
      <div className="flex items-start justify-between gap-2 pb-3 border-b border-[var(--border-subtle)]">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-semibold text-[var(--text-primary)] truncate">
              {selectedConcept!.name}
            </h2>
            <span className="fluent-badge text-[9px] uppercase px-1.5 py-0.2">
              {selectedConcept!.category}
            </span>
          </div>
          {selectedConcept!.description && (
            <p className="text-xs text-[var(--text-secondary)] mt-1 leading-relaxed">
              {selectedConcept!.description}
            </p>
          )}
          <span className="text-[10px] text-[var(--text-tertiary)] font-mono mt-0.5 block truncate">
            Canonical: {selectedConcept!.canonical_id}
          </span>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          {onFocusConcept && (
            <button
              type="button"
              onClick={() => onFocusConcept(selectedConcept!.id)}
              className="fluent-button p-1 text-[var(--text-tertiary)] hover:text-[var(--accent-primary)]"
              title="Center graph on this concept"
            >
              <Target className="w-3.5 h-3.5" />
            </button>
          )}
          <button
            type="button"
            onClick={onClose}
            className="fluent-button p-1 text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
            title="Close inspector"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* AI Concept Exploration Action */}
      {onExploreConcept && (
        <button
          type="button"
          onClick={() => onExploreConcept(selectedConcept!)}
          className="fluent-button text-xs py-2 px-3 bg-[var(--accent-primary)] text-white hover:opacity-90 flex items-center justify-center gap-2 cursor-pointer shadow-sm w-full font-medium"
          title="Open concept-focused AI exploration"
        >
          <Sparkles className="w-3.5 h-3.5" />
          <span>Explore with ECHO</span>
        </button>
      )}

      {/* Connection Metrics */}
      <div className="grid grid-cols-3 gap-2">
        <div className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] text-center">
          <span className="text-[9px] text-[var(--text-tertiary)] uppercase font-semibold block">Total</span>
          <span className="text-sm font-bold text-[var(--text-primary)]">{neighbors.length}</span>
        </div>
        <div className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] text-center">
          <span className="text-[9px] text-cyan-400 uppercase font-semibold block">Outgoing</span>
          <span className="text-sm font-bold text-cyan-400">
            {neighbors.filter((n) => n.direction === "outgoing").length}
          </span>
        </div>
        <div className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] text-center">
          <span className="text-[9px] text-emerald-400 uppercase font-semibold block">Incoming</span>
          <span className="text-sm font-bold text-emerald-400">
            {neighbors.filter((n) => n.direction === "incoming").length}
          </span>
        </div>
      </div>

      {/* Connected Concepts Section */}
      <div className="space-y-2">
        <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center justify-between">
          <span className="flex items-center gap-1.5">
            <Share2 className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
            <span>Connected Concepts ({neighbors.length})</span>
          </span>
        </span>

        {neighbors.length === 0 ? (
          <div className="p-4 text-center rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] text-xs text-[var(--text-tertiary)]">
            No direct semantic relationships recorded for this concept yet.
          </div>
        ) : (
          <div className="space-y-2 max-h-56 overflow-y-auto pr-1">
            {neighbors.map((nbr) => {
              const style = RELATIONSHIP_COLOR_MAP[nbr.relationship_type] || RELATIONSHIP_COLOR_MAP.RELATED_TO;
              return (
                <div
                  key={nbr.relationship_id}
                  className="p-2.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-1.5"
                >
                  <div className="flex items-center justify-between gap-1">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <span
                        className="fluent-badge text-[9px] font-semibold py-0.2 px-1.5 border shrink-0"
                        style={{
                          backgroundColor: style.bg,
                          color: style.text,
                          borderColor: style.border,
                        }}
                      >
                        {nbr.relationship_type}
                      </span>
                      <button
                        type="button"
                        onClick={() => onSelectConcept(nbr.neighbor_concept_id)}
                        className="text-xs font-medium text-[var(--text-primary)] hover:underline truncate text-left cursor-pointer"
                        title={nbr.neighbor_concept_name}
                      >
                        {nbr.neighbor_concept_name}
                      </button>
                    </div>

                    <div className="flex items-center gap-1 shrink-0">
                      <span className="text-[9px] text-[var(--text-tertiary)] font-mono">
                        {(nbr.confidence * 100).toFixed(0)}%
                      </span>
                      <button
                        type="button"
                        onClick={() => onDeleteRelationship(nbr.relationship_id)}
                        className="text-[var(--text-tertiary)] hover:text-[var(--status-danger)] p-0.5 cursor-pointer"
                        title="Delete relationship"
                      >
                        <Trash2 className="w-2.5 h-2.5" />
                      </button>
                    </div>
                  </div>

                  {/* Evidence quote */}
                  <p className="text-[10px] text-[var(--text-secondary)] italic bg-[var(--bg-card)] p-1.5 rounded-[var(--radius-sm)] border border-[var(--border-subtle)] leading-relaxed">
                    "{nbr.evidence}"
                  </p>

                  {nbr.source_document_name && nbr.source_document_id && (
                    <button
                      type="button"
                      onClick={() => onOpenDocument(nbr.source_document_id!)}
                      className="text-[10px] text-[var(--accent-primary)] hover:underline flex items-center gap-1 pt-0.5"
                    >
                      <FileText className="w-2.5 h-2.5" />
                      <span className="truncate max-w-[240px]">Source: {nbr.source_document_name}</span>
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Connected Documents Section */}
      <div className="space-y-2 pt-2 border-t border-[var(--border-subtle)]">
        <span className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
          <BookOpen className="w-3.5 h-3.5 text-emerald-400" />
          <span>Referenced Documents ({conceptDocs.length})</span>
        </span>

        {conceptDocs.length === 0 ? (
          <div className="p-3 text-center text-xs text-[var(--text-tertiary)]">
            No document associations recorded.
          </div>
        ) : (
          <div className="space-y-1.5 max-h-44 overflow-y-auto pr-1">
            {conceptDocs.map((doc) => (
              <div
                key={doc.document_id}
                onClick={() => onOpenDocument(doc.document_id)}
                className="p-2 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] hover:bg-[var(--bg-card-hover)] cursor-pointer transition-colors space-y-1"
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-1.5 min-w-0">
                    <FileText className="w-3 h-3 text-[var(--accent-primary)] shrink-0" />
                    <span className="text-xs font-medium text-[var(--text-primary)] truncate">
                      {doc.document_filename}
                    </span>
                  </div>
                  <span className="text-[9px] text-[var(--text-tertiary)] font-mono shrink-0">
                    {(doc.confidence * 100).toFixed(0)}%
                  </span>
                </div>
                {doc.evidence && (
                  <p className="text-[10px] text-[var(--text-tertiary)] italic line-clamp-2">
                    "{doc.evidence}"
                  </p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
