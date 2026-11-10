import React, { useState, useEffect, useRef, useCallback } from "react";
import {
  Search,
  Loader2,
  X,
  FileText,
  FileCode,
  FileSpreadsheet,
} from "lucide-react";
import type { DocumentSearchResult } from "@/types/documents";
import { DocumentService } from "@/services/documents/documentService";
import { useRouter } from "@/router";
import { formatBytes } from "@/utils";

interface HighlightMatchProps {
  text: string;
  query: string;
}

const HighlightMatch: React.FC<HighlightMatchProps> = ({ text, query }) => {
  if (!query.trim()) return <>{text}</>;

  const trimmedQuery = query.trim();
  const escaped = trimmedQuery.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const parts = text.split(new RegExp(`(${escaped})`, "gi"));

  return (
    <>
      {parts.map((part, index) =>
        part.toLowerCase() === trimmedQuery.toLowerCase() ? (
          <mark
            key={index}
            className="bg-[var(--accent-primary)]/30 text-[var(--text-primary)] font-semibold rounded-xs px-0.5"
          >
            {part}
          </mark>
        ) : (
          <span key={index}>{part}</span>
        )
      )}
    </>
  );
};

export const GlobalSearch: React.FC = () => {
  const { navigate } = useRouter();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<DocumentSearchResult[] | null>(null);
  const [isSearching, setIsSearching] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);

  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Debounced search (250ms) querying SQLite via DocumentService
  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed) {
      setResults(null);
      setIsSearching(false);
      setIsOpen(false);
      return;
    }

    setIsSearching(true);
    const timer = setTimeout(async () => {
      try {
        const data = await DocumentService.searchDocuments(trimmed);
        setResults(data);
        setSelectedIndex(0);
        setIsOpen(true);
      } catch (err) {
        console.error("Global search query failed:", err);
        setResults([]);
        setIsOpen(true);
      } finally {
        setIsSearching(false);
      }
    }, 250);

    return () => clearTimeout(timer);
  }, [query]);

  // Click outside listener to close dropdown
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Global Ctrl+K / Cmd+K listener to focus global search
  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
        if (query.trim() && results !== null) {
          setIsOpen(true);
        }
      }
    };
    window.addEventListener("keydown", handleGlobalKeyDown);
    return () => window.removeEventListener("keydown", handleGlobalKeyDown);
  }, [query, results]);

  const handleSelectResult = useCallback(
    (result: DocumentSearchResult) => {
      setIsOpen(false);
      setSelectedIndex(0);
      navigate("my-knowledge", { previewDocumentId: result.id });
      inputRef.current?.blur();
    },
    [navigate]
  );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!isOpen && results && results.length > 0) {
        setIsOpen(true);
      } else if (results && results.length > 0) {
        setSelectedIndex((prev) => (prev + 1) % results.length);
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (results && results.length > 0) {
        setSelectedIndex((prev) => (prev - 1 + results.length) % results.length);
      }
    } else if (e.key === "Enter") {
      if (isOpen && results && results.length > 0) {
        e.preventDefault();
        const target = results[selectedIndex] ?? results[0];
        handleSelectResult(target);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      setIsOpen(false);
      inputRef.current?.blur();
    }
  };

  const renderFileIcon = (fileType: string) => {
    switch (fileType.toLowerCase()) {
      case "pdf":
        return <FileSpreadsheet className="w-4 h-4 text-rose-500 shrink-0 mt-0.5" />;
      case "md":
        return <FileCode className="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />;
      default:
        return <FileText className="w-4 h-4 text-[var(--accent-primary)] shrink-0 mt-0.5" />;
    }
  };

  const getMatchLabel = (matchType: "filename" | "content" | "both") => {
    switch (matchType) {
      case "both":
        return "Filename & Content";
      case "content":
        return "Content";
      case "filename":
        return "Filename";
    }
  };

  return (
    <div ref={containerRef} className="relative w-full select-text">
      <div className="relative w-full">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-[var(--text-tertiary)] pointer-events-none" />
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => {
            if (query.trim() && results !== null) {
              setIsOpen(true);
            }
          }}
          onKeyDown={handleKeyDown}
          placeholder="Search documents by filename or text... (Ctrl + K)"
          title="Search documents stored locally in SQLite"
          className="fluent-input text-xs pl-8 pr-14 py-1 bg-[var(--bg-subtle)] hover:bg-[var(--bg-card)] focus:bg-[var(--bg-card)] text-[var(--text-primary)] cursor-text select-text"
          autoComplete="off"
          spellCheck={false}
          role="combobox"
          aria-expanded={isOpen}
          aria-autocomplete="list"
          aria-haspopup="listbox"
        />

        {/* Loading Indicator */}
        {isSearching && (
          <Loader2 className="absolute right-7 top-1/2 -translate-y-1/2 w-3.5 h-3.5 animate-spin text-[var(--accent-primary)] pointer-events-none" />
        )}

        {/* Clear Button */}
        {query && (
          <button
            type="button"
            onClick={() => {
              setQuery("");
              setResults(null);
              setIsOpen(false);
              inputRef.current?.focus();
            }}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)] hover:text-[var(--text-primary)] cursor-pointer p-0.5"
            title="Clear search"
            aria-label="Clear global search"
          >
            <X className="w-3 h-3" />
          </button>
        )}
      </div>

      {/* Dropdown Flyout */}
      {isOpen && (
        <div
          role="listbox"
          aria-label="Document search results"
          className="absolute left-0 right-0 top-full mt-1.5 z-50 bg-[var(--bg-card)] border border-[var(--border-default)] rounded-[var(--radius-lg)] shadow-2xl shadow-black/50 overflow-hidden flex flex-col max-h-96"
        >
          {isSearching && (!results || results.length === 0) ? (
            <div className="p-4 flex items-center justify-center gap-2 text-xs text-[var(--text-secondary)] select-none">
              <Loader2 className="w-3.5 h-3.5 animate-spin text-[var(--accent-primary)]" />
              <span>Searching local documents...</span>
            </div>
          ) : results && results.length > 0 ? (
            <>
              {/* Results List */}
              <div className="overflow-y-auto max-h-80 divide-y divide-[var(--border-subtle)]/60">
                {results.map((item, index) => {
                  const isSelected = index === selectedIndex;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      role="option"
                      aria-selected={isSelected}
                      onClick={() => handleSelectResult(item)}
                      onMouseEnter={() => setSelectedIndex(index)}
                      className={`w-full text-left p-3 flex items-start gap-3 transition-colors cursor-pointer ${
                        isSelected
                          ? "bg-[var(--bg-active)] border-l-2 border-l-[var(--accent-primary)] pl-2.5"
                          : "hover:bg-[var(--bg-active)]"
                      }`}
                    >
                      {renderFileIcon(item.file_type)}

                      <div className="flex-1 min-w-0 space-y-1">
                        {/* Filename & Meta Badges */}
                        <div className="flex items-center justify-between gap-2">
                          <span className="font-medium text-xs text-[var(--text-primary)] truncate">
                            <HighlightMatch text={item.filename} query={query} />
                          </span>

                          <div className="flex items-center gap-1.5 shrink-0">
                            <span className="text-[9px] font-mono uppercase px-1.5 py-0.5 rounded-[var(--radius-xs)] bg-[var(--bg-subtle)] text-[var(--text-secondary)] border border-[var(--border-subtle)]">
                              {item.file_type}
                            </span>
                            <span className="text-[10px] text-[var(--text-tertiary)]">
                              {formatBytes(item.file_size)}
                            </span>
                          </div>
                        </div>

                        {/* Matching Snippet */}
                        {item.snippet && (
                          <div className="text-[11px] text-[var(--text-secondary)] line-clamp-2 leading-relaxed bg-[var(--bg-subtle)]/70 px-2 py-1 rounded-[var(--radius-xs)] border border-[var(--border-subtle)]/50 font-mono break-all">
                            <HighlightMatch text={item.snippet} query={query} />
                          </div>
                        )}

                        {/* Match Scope Badge */}
                        <div className="flex items-center gap-1 text-[10px] text-[var(--text-tertiary)]">
                          <span>Matched in:</span>
                          <span className="text-[var(--text-secondary)] font-medium">
                            {getMatchLabel(item.match_type)}
                          </span>
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>

              {/* Dropdown Footer */}
              <div className="px-3 py-1.5 bg-[var(--bg-subtle)]/80 border-t border-[var(--border-subtle)] flex items-center justify-between text-[10px] text-[var(--text-tertiary)] select-none">
                <span>
                  Found <strong>{results.length}</strong> matching document{results.length !== 1 ? "s" : ""}
                </span>
                <span className="hidden sm:inline">
                  Press <kbd className="px-1 py-0.5 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] font-mono text-[9px]">Enter</kbd> to preview, <kbd className="px-1 py-0.5 rounded bg-[var(--bg-card)] border border-[var(--border-subtle)] font-mono text-[9px]">Esc</kbd> to close
                </span>
              </div>
            </>
          ) : results && results.length === 0 ? (
            <div className="p-6 text-center space-y-1.5 select-none">
              <p className="text-xs font-medium text-[var(--text-primary)]">
                No documents found matching "{query}"
              </p>
              <p className="text-[11px] text-[var(--text-tertiary)] max-w-xs mx-auto">
                No matches in document filenames or extracted text. Check your spelling or try another keyword.
              </p>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
};

export default GlobalSearch;

