import React, { useState, useEffect, useRef } from "react";
import {
  Sparkles,
  Send,
  Plus,
  Trash2,
  FileText,
  FileCode,
  FileSpreadsheet,
  AlertCircle,
  ExternalLink,
  Loader2,
  Key,
  ShieldCheck,
  CheckCircle2,
  MessageSquare,
  RotateCcw,
} from "lucide-react";
import { AIService } from "@/services/ai/aiService";
import { DocumentService } from "@/services/documents/documentService";
import { DocumentPreviewModal } from "@/components/documents/DocumentPreviewModal";
import { useRouter } from "@/router";
import type { Citation, Conversation, ConversationMessage, SafeProviderStatus, IndexingOverview } from "@/types/ai";
import type { DocumentRecord } from "@/types/documents";

export const AskEcho: React.FC = () => {
  const { navigate } = useRouter();

  // State
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [inputQuery, setInputQuery] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [providerStatus, setProviderStatus] = useState<SafeProviderStatus | null>(null);
  const [indexingOverview, setIndexingOverview] = useState<IndexingOverview | null>(null);
  const [errorBanner, setErrorBanner] = useState<string | null>(null);

  // Document preview state
  const [previewDoc, setPreviewDoc] = useState<DocumentRecord | null>(null);
  const [isPreviewOpen, setIsPreviewOpen] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Load initial status and conversations
  useEffect(() => {
    initChat();

    let unlistenFn: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen("indexing:status_changed", () => {
        AIService.getIndexingStatus().then(setIndexingOverview).catch(() => { });
      }).then((unlisten) => {
        unlistenFn = unlisten;
      }).catch(() => { });
    }).catch(() => { });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const initChat = async () => {
    try {
      const [pStatus, convs, idxOverview] = await Promise.all([
        AIService.getProviderStatus(),
        AIService.listConversations(),
        AIService.getIndexingStatus().catch(() => null),
      ]);
      setProviderStatus(pStatus);
      setConversations(convs);
      setIndexingOverview(idxOverview);

      if (convs.length > 0) {
        selectConversation(convs[0].id);
      }
    } catch (err) {
      console.error("Failed to initialize chat:", err);
    }
  };

  const selectConversation = async (convId: string) => {
    try {
      setActiveConversationId(convId);
      setErrorBanner(null);
      const msgs = await AIService.getConversationMessages(convId);
      setMessages(msgs);
    } catch (err) {
      console.error("Failed to load conversation messages:", err);
    }
  };

  const handleNewConversation = async () => {
    try {
      const conv = await AIService.createConversation("New Research Chat");
      setConversations((prev) => [conv, ...prev]);
      setActiveConversationId(conv.id);
      setMessages([]);
      setErrorBanner(null);
      textareaRef.current?.focus();
    } catch (err) {
      console.error("Failed to create conversation:", err);
    }
  };

  const handleDeleteConversation = async (convId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const confirmed = window.confirm("Are you sure you want to delete this conversation?");
    if (!confirmed) return;

    try {
      await AIService.deleteConversation(convId);
      const remaining = conversations.filter((c) => c.id !== convId);
      setConversations(remaining);
      if (activeConversationId === convId) {
        if (remaining.length > 0) {
          selectConversation(remaining[0].id);
        } else {
          setActiveConversationId(null);
          setMessages([]);
        }
      }
    } catch (err) {
      console.error("Failed to delete conversation:", err);
    }
  };

  const handleClearConversation = async () => {
    if (!activeConversationId) return;
    const confirmed = window.confirm("Are you sure you want to clear all messages in this conversation?");
    if (!confirmed) return;

    try {
      await AIService.clearConversation(activeConversationId);
      setMessages([]);
    } catch (err) {
      console.error("Failed to clear conversation:", err);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isLoading]);

  const handleSendMessage = async () => {
    const q = inputQuery.trim();
    if (!q || isLoading) return;

    if (!providerStatus?.is_configured) {
      setErrorBanner("Google Gemini API key is not configured. Please set your key in Settings.");
      return;
    }

    // Optimistically add user message to list
    const tempUserMsg: ConversationMessage = {
      id: "temp-" + Date.now(),
      conversation_id: activeConversationId || "temp",
      role: "user",
      content: q,
      citations_json: null,
      created_at: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, tempUserMsg]);
    setInputQuery("");
    setIsLoading(true);
    setErrorBanner(null);

    try {
      const response = await AIService.askEcho(q, activeConversationId);

      // If a new conversation was created on the fly
      if (!activeConversationId || activeConversationId !== response.conversation_id) {
        setActiveConversationId(response.conversation_id);
        const updatedConvs = await AIService.listConversations();
        setConversations(updatedConvs);
      }

      // Refresh actual stored messages from SQLite
      const updatedMessages = await AIService.getConversationMessages(response.conversation_id);
      setMessages(updatedMessages);
    } catch (err) {
      console.error("Failed to generate answer:", err);
      setErrorBanner(String(err));
    } finally {
      setIsLoading(false);
      textareaRef.current?.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  const openDocumentPreview = async (documentId: string) => {
    try {
      const doc = await DocumentService.getDocument(documentId);
      setPreviewDoc(doc);
      setIsPreviewOpen(true);
    } catch (err) {
      console.error("Failed to load document for citation preview:", err);
    }
  };

  const renderFileIcon = (fileType: string) => {
    switch (fileType.toLowerCase()) {
      case "pdf":
        return <FileSpreadsheet className="w-3.5 h-3.5 text-rose-500 shrink-0" />;
      case "md":
        return <FileCode className="w-3.5 h-3.5 text-cyan-400 shrink-0" />;
      default:
        return <FileText className="w-3.5 h-3.5 text-[var(--accent-primary)] shrink-0" />;
    }
  };

  return (
    <div className="h-full flex overflow-hidden">
      {/* Document Preview Modal */}
      <DocumentPreviewModal
        document={previewDoc}
        isOpen={isPreviewOpen}
        onClose={() => setIsPreviewOpen(false)}
      />

      {/* Conversations Sidebar */}
      <aside className="w-64 shrink-0 border-r border-[var(--border-subtle)] bg-[var(--bg-subtle)]/50 flex flex-col hidden md:flex">
        <div className="p-3 border-b border-[var(--border-subtle)] flex items-center justify-between">
          <span className="text-xs font-semibold text-[var(--text-primary)]">
            Research Chats
          </span>
          <button
            type="button"
            onClick={handleNewConversation}
            className="fluent-button p-1.5 text-xs"
            title="New Conversation"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Conversation List */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {conversations.length === 0 ? (
            <div className="p-4 text-center text-xs text-[var(--text-tertiary)]">
              No conversations yet.
            </div>
          ) : (
            conversations.map((c) => {
              const isActive = c.id === activeConversationId;
              return (
                <div
                  key={c.id}
                  onClick={() => selectConversation(c.id)}
                  className={`group relative p-2.5 rounded-[var(--radius-md)] cursor-pointer text-xs flex items-center justify-between transition-colors ${isActive
                    ? "bg-[var(--bg-card)] border border-[var(--border-default)] shadow-xs text-[var(--text-primary)] font-medium"
                    : "text-[var(--text-secondary)] hover:bg-[var(--bg-active)]"
                    }`}
                >
                  <div className="flex items-center gap-2 min-w-0 pr-2">
                    <MessageSquare className="w-3.5 h-3.5 shrink-0 text-[var(--text-tertiary)]" />
                    <span className="truncate">{c.title}</span>
                  </div>
                  <button
                    type="button"
                    onClick={(e) => handleDeleteConversation(c.id, e)}
                    className="opacity-0 group-hover:opacity-100 p-1 text-[var(--text-tertiary)] hover:text-[var(--status-danger)] transition-opacity"
                    title="Delete Chat"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              );
            })
          )}
        </div>
      </aside>

      {/* Main Chat Interface Stage */}
      <main className="flex-1 flex flex-col min-w-0 h-full bg-[var(--bg-app)]">
        {/* Chat Stage Header */}
        <div className="h-11 shrink-0 px-4 border-b border-[var(--border-subtle)] bg-[var(--bg-titlebar)] flex items-center justify-between">
          <div className="flex items-center gap-2 min-w-0">
            <Sparkles className="w-4 h-4 text-[var(--accent-primary)] shrink-0" />
            <h1 className="text-xs font-semibold text-[var(--text-primary)] truncate">
              Ask ECHO — Grounded Document Retrieval & Q&A
            </h1>
          </div>

          <div className="flex items-center gap-2">
            {providerStatus?.is_configured ? (
              <span className="fluent-badge text-[10px] bg-[var(--status-success)]/10 text-[var(--status-success)] border-[var(--status-success)]/30">
                <CheckCircle2 className="w-3 h-3 inline mr-1" />
                {providerStatus.active_generation_model}
              </span>
            ) : (
              <button
                type="button"
                onClick={() => navigate("settings")}
                className="fluent-button p-1.5 text-xs text-amber-500 border-amber-500/30 bg-amber-500/5 hover:bg-amber-500/10"
              >
                <Key className="w-3 h-3" />
                <span>Configure API Key</span>
              </button>
            )}

            {activeConversationId && messages.length > 0 && (
              <button
                type="button"
                onClick={handleClearConversation}
                className="fluent-button-subtle p-1.5 text-xs rounded hover:bg-[var(--bg-active)]"
                title="Clear current chat"
              >
                <RotateCcw className="w-3.5 h-3.5 text-[var(--text-tertiary)]" />
              </button>
            )}
          </div>
        </div>

        {/* Error Alert Banner */}
        {errorBanner && (
          <div className="m-4 p-3 rounded-[var(--radius-md)] bg-[var(--status-danger)]/10 border border-[var(--status-danger)]/30 text-[var(--status-danger)] text-xs flex items-center justify-between">
            <div className="flex items-center gap-2">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span>{errorBanner}</span>
            </div>
            <button
              type="button"
              onClick={() => setErrorBanner(null)}
              className="text-xs underline hover:no-underline cursor-pointer"
            >
              Dismiss
            </button>
          </div>
        )}

        {/* Background Indexing Notification Banner */}
        {indexingOverview && (indexingOverview.currently_processing_documents > 0 || indexingOverview.pending_documents > 0) && (
          <div className="mx-4 mt-3 p-3 rounded-[var(--radius-md)] bg-blue-500/10 border border-blue-500/30 text-blue-400 text-xs flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin shrink-0 text-blue-400" />
              <span>
                Automatic document indexing is running in the background (
                {indexingOverview.currently_processing_documents > 0 ? `${indexingOverview.currently_processing_documents} indexing, ` : ""}
                {indexingOverview.pending_documents} pending). Newly indexed knowledge will be automatically included in search once complete.
              </span>
            </div>
          </div>
        )}

        {/* Scrollable Message History Area */}
        <div className="flex-1 overflow-y-auto p-4 sm:p-6 space-y-5">
          {messages.length === 0 ? (
            /* Empty State */
            <div className="h-full flex flex-col items-center justify-center text-center max-w-md mx-auto space-y-4 p-6">
              <div className="w-12 h-12 rounded-[var(--radius-xl)] bg-[var(--bg-subtle)] border border-[var(--border-default)] flex items-center justify-center text-[var(--accent-primary)]">
                <Sparkles className="w-6 h-6" />
              </div>
              <div className="space-y-1">
                <h2 className="text-sm font-semibold text-[var(--text-primary)]">
                  Ask questions across your documents
                </h2>
                <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                  ECHO retrieves relevant passages from your local SQLite document repository and generates grounded answers citing specific sources.
                </p>
              </div>

              {!providerStatus?.is_configured ? (
                <div className="p-3.5 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-amber-500 text-xs space-y-2 max-w-sm">
                  <p>A Google Gemini API key is required for semantic indexing and answer generation.</p>
                  <button
                    type="button"
                    onClick={() => navigate("settings")}
                    className="fluent-button fluent-button-primary text-xs w-full justify-center"
                  >
                    <Key className="w-3 h-3" />
                    <span>Configure API Key in Settings</span>
                  </button>
                </div>
              ) : (
                <div className="space-y-2 w-full pt-2">
                  <span className="text-[11px] text-[var(--text-tertiary)] uppercase tracking-wider font-semibold">
                    Sample questions to try:
                  </span>
                  <div className="flex flex-col gap-1.5 text-xs">
                    {[
                      "What is ECHO's local storage and database architecture?",
                      "What does the BLUE LANTERN document explain?",
                      "Summarize the key test concepts in my documents.",
                    ].map((sample, idx) => (
                      <button
                        key={idx}
                        type="button"
                        onClick={() => {
                          setInputQuery(sample);
                          textareaRef.current?.focus();
                        }}
                        className="fluent-card p-2.5 text-left text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--accent-primary)] transition-all cursor-pointer"
                      >
                        {sample}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          ) : (
            /* Render Messages */
            messages.map((msg) => {
              const isUser = msg.role === "user";
              let citations: Citation[] = [];
              if (msg.citations_json) {
                try {
                  citations = JSON.parse(msg.citations_json);
                } catch {
                  citations = [];
                }
              }

              return (
                <div
                  key={msg.id}
                  className={`flex flex-col ${isUser ? "items-end" : "items-start"} space-y-1.5`}
                >
                  <div
                    className={`max-w-2xl p-3.5 rounded-[var(--radius-lg)] text-xs leading-relaxed ${isUser
                      ? "bg-[var(--accent-primary)] text-[var(--accent-foreground)] rounded-br-xs"
                      : "fluent-card rounded-bl-xs text-[var(--text-primary)] whitespace-pre-wrap"
                      }`}
                  >
                    {msg.content}
                  </div>

                  {/* Render Verified Source Citations */}
                  {!isUser && citations.length > 0 && (
                    <div className="max-w-2xl w-full pt-1 space-y-1.5">
                      <div className="text-[10px] font-semibold text-[var(--text-tertiary)] uppercase tracking-wider flex items-center gap-1.5">
                        <ShieldCheck className="w-3 h-3 text-[var(--status-success)]" />
                        <span>Sources Cited from Local Documents ({citations.length}):</span>
                      </div>

                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                        {citations.map((c, i) => (
                          <div
                            key={c.chunk_id || i}
                            onClick={() => openDocumentPreview(c.document_id)}
                            className="fluent-card p-2.5 space-y-1 cursor-pointer hover:border-[var(--accent-primary)] transition-colors group"
                            title="Click to view full original document"
                          >
                            <div className="flex items-center justify-between gap-1.5">
                              <div className="flex items-center gap-1.5 min-w-0">
                                {renderFileIcon(c.file_type)}
                                <span className="font-semibold text-[11px] text-[var(--text-primary)] truncate group-hover:text-[var(--accent-primary)]">
                                  {c.filename}
                                </span>
                              </div>
                              <span className="text-[9px] font-mono px-1 py-0.5 rounded bg-[var(--bg-subtle)] text-[var(--text-tertiary)] border border-[var(--border-subtle)] shrink-0">
                                Chunk #{c.chunk_index + 1}
                              </span>
                            </div>

                            <p className="text-[10px] text-[var(--text-secondary)] line-clamp-2 leading-snug font-mono bg-[var(--bg-subtle)]/70 p-1 rounded-[var(--radius-xs)] border border-[var(--border-subtle)]/50">
                              "{c.snippet}"
                            </p>

                            <div className="flex items-center justify-between pt-0.5 text-[9px] text-[var(--text-tertiary)]">
                              <span>Match: {(c.similarity * 100).toFixed(0)}%</span>
                              <span className="flex items-center gap-0.5 text-[var(--accent-primary)] opacity-0 group-hover:opacity-100 transition-opacity">
                                <span>Preview</span>
                                <ExternalLink className="w-2.5 h-2.5" />
                              </span>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              );
            })
          )}

          {/* Thinking / Loading indicator */}
          {isLoading && (
            <div className="flex items-center gap-2 p-3 text-xs text-[var(--text-secondary)] fluent-card max-w-sm rounded-bl-xs">
              <Loader2 className="w-4 h-4 animate-spin text-[var(--accent-primary)]" />
              <span>Retrieving local document passages & generating answer...</span>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Message Input Composer */}
        <div className="p-4 border-t border-[var(--border-subtle)] bg-[var(--bg-titlebar)]">
          <div className="relative max-w-3xl mx-auto flex items-end gap-2 fluent-card p-2 bg-[var(--bg-card)]">
            <textarea
              ref={textareaRef}
              rows={2}
              value={inputQuery}
              onChange={(e) => setInputQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isLoading}
              placeholder={
                providerStatus?.is_configured
                  ? "Ask a question about your documents... (Enter to send, Shift+Enter for newline)"
                  : "Please configure your Gemini API key in Settings to ask questions"
              }
              className="fluent-input resize-none text-xs border-0 border-b-0 focus:border-0 bg-transparent py-1.5 focus:bg-transparent"
            />
            <button
              type="button"
              onClick={handleSendMessage}
              disabled={isLoading || !inputQuery.trim()}
              className="fluent-button fluent-button-primary p-2 shrink-0 rounded-[var(--radius-md)] disabled:opacity-40"
              title="Send question (Enter)"
            >
              {isLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Send className="w-4 h-4" />
              )}
            </button>
          </div>
          <div className="flex items-center justify-between max-w-3xl mx-auto pt-1.5 px-1 text-[10px] text-[var(--text-tertiary)]">
            <span>Answers synthesized strictly from your local files</span>
            <span>Zero cloud sync • Key stored in Windows Keyring</span>
          </div>
        </div>
      </main>
    </div>
  );
};

export default AskEcho;
