import React from "react";
import { FolderTree, FolderPlus, Tag, Bookmark } from "lucide-react";

export const Collections: React.FC = () => {
  return (
    <div className="p-6 space-y-6 max-w-6xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between pb-2 border-b border-[var(--border-subtle)]">
        <div>
          <h1 className="text-xl font-semibold tracking-tight text-[var(--text-primary)]">
            Collections & Notebooks
          </h1>
          <p className="text-xs text-[var(--text-secondary)]">
            Organize documents into thematic workspaces, custom tags, and projects.
          </p>
        </div>
        <button
          type="button"
          disabled
          className="fluent-button fluent-button-primary text-xs opacity-60 cursor-not-allowed"
          title="Create collection feature enabled in Phase 2"
        >
          <FolderPlus className="w-3.5 h-3.5" />
          <span>New Collection</span>
        </button>
      </div>

      {/* Grid of Structural Category Placeholders */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="fluent-card p-5 space-y-3 opacity-80">
          <div className="w-9 h-9 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] flex items-center justify-center text-[var(--accent-primary)]">
            <Bookmark className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-xs font-semibold text-[var(--text-primary)]">Pinned Notebooks</h3>
            <p className="text-[11px] text-[var(--text-secondary)] mt-0.5">
              Quick access to frequently referenced research and documents.
            </p>
          </div>
          <span className="fluent-badge text-[10px]">0 Notebooks</span>
        </div>

        <div className="fluent-card p-5 space-y-3 opacity-80">
          <div className="w-9 h-9 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] flex items-center justify-center text-[var(--accent-primary)]">
            <Tag className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-xs font-semibold text-[var(--text-primary)]">Taxonomy & Tags</h3>
            <p className="text-[11px] text-[var(--text-secondary)] mt-0.5">
              Hierarchical categorization automatically suggested by ECHO or user-defined.
            </p>
          </div>
          <span className="fluent-badge text-[10px]">0 Tags</span>
        </div>

        <div className="fluent-card p-5 space-y-3 opacity-80">
          <div className="w-9 h-9 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] flex items-center justify-center text-[var(--accent-primary)]">
            <FolderTree className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-xs font-semibold text-[var(--text-primary)]">Topic Clusters</h3>
            <p className="text-[11px] text-[var(--text-secondary)] mt-0.5">
              Semantic clusters discovered through embedding vector analysis.
            </p>
          </div>
          <span className="fluent-badge text-[10px]">0 Clusters</span>
        </div>
      </div>
    </div>
  );
};

export default Collections;

