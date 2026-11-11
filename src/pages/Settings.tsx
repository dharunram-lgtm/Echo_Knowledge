import React, { useState, useEffect } from "react";
import {
  Key,
  Moon,
  Sun,
  Laptop,
  Shield,
  HardDrive,
  Info,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Trash2,
  Eye,
  EyeOff,
  ExternalLink,
  Cpu,
  Layers,
  Sparkles,
  RefreshCw,
} from "lucide-react";
import { useTheme } from "@/hooks/useTheme";
import { useAppInfo } from "@/hooks/useAppInfo";
import { AIService } from "@/services/ai/aiService";
import type {
  EmbeddingCompatibility,
  ModelConfig,
  SafeProviderStatus,
} from "@/types/ai";

export const Settings: React.FC = () => {
  const { theme, setTheme } = useTheme();
  const appInfo = useAppInfo();

  // AI Provider State
  const [selectedProvider, setSelectedProvider] = useState<string>("gemini");
  const [apiKeyInput, setApiKeyInput] = useState<string>("");
  const [showApiKey, setShowApiKey] = useState<boolean>(false);
  const [status, setStatus] = useState<SafeProviderStatus | null>(null);
  const [modelConfig, setModelConfig] = useState<ModelConfig | null>(null);
  const [compatibility, setCompatibility] = useState<EmbeddingCompatibility | null>(null);
  const [lastConnectionTest, setLastConnectionTest] = useState<string | null>(null);
  const [autoIndexEnabled, setAutoIndexEnabled] = useState<boolean>(true);
  const [isUpdatingAutoIndex, setIsUpdatingAutoIndex] = useState(false);

  // Operation state
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [isReindexing, setIsReindexing] = useState(false);
  const [feedback, setFeedback] = useState<{
    type: "success" | "error" | "info";
    message: string;
  } | null>(null);

  useEffect(() => {
    loadAIStatus();
  }, []);

  const loadAIStatus = async () => {
    try {
      const [provStatus, config, autoIndex] = await Promise.all([
        AIService.getProviderStatus(),
        AIService.getModelConfig(),
        AIService.getAutoIndexSetting().catch(() => true),
      ]);
      setStatus(provStatus);
      setModelConfig(config);
      setSelectedProvider(provStatus.provider);
      setAutoIndexEnabled(autoIndex);

      if (provStatus.is_configured) {
        try {
          const compat = await AIService.checkEmbeddingCompatibility();
          setCompatibility(compat);
        } catch {
          // Non-critical background compatibility check
        }
      }
    } catch (err) {
      console.error("Failed to load AI provider status:", err);
    }
  };

  const handleToggleAutoIndex = async () => {
    try {
      setIsUpdatingAutoIndex(true);
      const nextVal = !autoIndexEnabled;
      await AIService.setAutoIndexSetting(nextVal);
      setAutoIndexEnabled(nextVal);
      setFeedback({
        type: "info",
        message: `Automatic startup indexing ${nextVal ? "enabled" : "disabled"}.`,
      });
    } catch (err) {
      setFeedback({
        type: "error",
        message: "Failed to update auto-index setting: " + String(err),
      });
    } finally {
      setIsUpdatingAutoIndex(false);
    }
  };

  const handleSaveKey = async () => {
    if (!apiKeyInput.trim()) {
      setFeedback({
        type: "error",
        message: "Please enter an API key to save.",
      });
      return;
    }

    try {
      setIsSaving(true);
      setFeedback(null);
      await AIService.saveApiKey(selectedProvider, apiKeyInput.trim());
      setApiKeyInput("");
      setFeedback({
        type: "success",
        message: `API key securely saved to Windows Credential Manager for ${selectedProvider.toUpperCase()}.`,
      });
      await loadAIStatus();
    } catch (err) {
      setFeedback({
        type: "error",
        message: `Failed to save API key: ${String(err)}`,
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    try {
      setIsTesting(true);
      setFeedback(null);
      const keyToTest = apiKeyInput.trim() || undefined;
      const success = await AIService.testConnection(selectedProvider, keyToTest);
      if (success) {
        const timeStr = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
        setLastConnectionTest(timeStr);
        setFeedback({
          type: "success",
          message: `Successfully connected to ${selectedProvider.toUpperCase()} API! Connection verified at ${timeStr}.`,
        });
      }
      await loadAIStatus();
    } catch (err) {
      setFeedback({
        type: "error",
        message: `Connection test failed: ${String(err)}`,
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleRefreshModels = async () => {
    try {
      setIsDiscovering(true);
      setFeedback(null);
      const models = await AIService.discoverModels(selectedProvider);
      const timeStr = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
      setLastConnectionTest(timeStr);
      setFeedback({
        type: "success",
        message: `Discovered ${models.length} active models directly from ${selectedProvider.toUpperCase()} (verified at ${timeStr}).`,
      });
      await loadAIStatus();
    } catch (err) {
      setFeedback({
        type: "error",
        message: `Model discovery failed: ${String(err)}`,
      });
    } finally {
      setIsDiscovering(false);
    }
  };

  const handleRemoveKey = async () => {
    const confirmed = window.confirm(
      `Are you sure you want to remove the saved API key for ${selectedProvider.toUpperCase()} from Windows Credential Manager?`
    );
    if (!confirmed) return;

    try {
      setIsRemoving(true);
      setFeedback(null);
      await AIService.removeApiKey(selectedProvider);
      setApiKeyInput("");
      setFeedback({
        type: "info",
        message: `Removed ${selectedProvider.toUpperCase()} key from Windows Credential Manager. Cloud AI operations are disabled.`,
      });
      await loadAIStatus();
    } catch (err) {
      setFeedback({
        type: "error",
        message: `Failed to remove key: ${String(err)}`,
      });
    } finally {
      setIsRemoving(false);
    }
  };

  const handleUpdateModel = async (
    field: "generation_model" | "embedding_model",
    value: string
  ) => {
    if (!modelConfig) return;
    const updated = { ...modelConfig, [field]: value };
    setModelConfig(updated);
    try {
      await AIService.updateModelConfig(updated);
      setFeedback({
        type: "success",
        message: `Updated ${field === "generation_model" ? "generation" : "embedding"} model to ${value}.`,
      });
      await loadAIStatus();
    } catch (err) {
      setFeedback({
        type: "error",
        message: `Failed to update model configuration: ${String(err)}`,
      });
    }
  };

  const handleSelectProvider = async (provider: string) => {
    setSelectedProvider(provider);
    if (!modelConfig) return;
    try {
      await AIService.updateModelConfig({ ...modelConfig, provider });
      await loadAIStatus();
    } catch (err) {
      setFeedback({ type: "error", message: `Failed to switch provider: ${String(err)}` });
    }
  };

  const handleOpenExternalUrl = async (url: string) => {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch (err) {
      console.error("Failed to open external URL with Tauri opener:", err);
      try {
        window.open(url, "_blank", "noopener,noreferrer");
      } catch (fallbackErr) {
        console.error("Fallback window.open also failed:", fallbackErr);
        setFeedback({
          type: "error",
          message: `Unable to open browser link: ${String(err)}`,
        });
      }
    }
  };

  const handleReindexAll = async () => {
    try {
      setIsReindexing(true);
      setFeedback(null);
      const result = await AIService.indexAllDocuments();
      setFeedback({
        type: "success",
        message: `Re-indexed ${result.successful.length} of ${result.total_processed} documents with active embedding model '${modelConfig?.embedding_model}'.`,
      });
      await loadAIStatus();
    } catch (err) {
      setFeedback({
        type: "error",
        message: `Failed to re-index documents: ${String(err)}`,
      });
    } finally {
      setIsReindexing(false);
    }
  };

  const discoveredGenModels = status?.discovered_models?.filter((m) => m.supports_generation) ?? [];
  const discoveredEmbModels = status?.discovered_models?.filter((m) => m.supports_embedding) ?? [];

  return (
    <div className="p-6 space-y-6 max-w-4xl mx-auto">
      {/* Header */}
      <div className="pb-2 border-b border-[var(--border-subtle)]">
        <h1 className="text-xl font-semibold tracking-tight text-[var(--text-primary)]">
          Settings & Preferences
        </h1>
        <p className="text-xs text-[var(--text-secondary)]">
          Manage application appearance, BYOK AI provider credentials, and local privacy boundaries.
        </p>
      </div>

      {/* AI Provider Section (BYOK) */}
      <div className="fluent-card p-5 space-y-5">
        <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
          <div className="flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-[var(--accent-primary)]" />
            <h2 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
              AI Provider & Intelligence (BYOK)
            </h2>
          </div>
          <div className="flex items-center gap-2">
            {status?.is_configured ? (
              <span className="fluent-badge text-[10px] bg-[var(--status-success)]/10 text-[var(--status-success)] border-[var(--status-success)]/30">
                <CheckCircle2 className="w-3 h-3 inline mr-1" />
                Key Configured in Windows Keyring
              </span>
            ) : (
              <span className="fluent-badge text-[10px] bg-amber-500/10 text-amber-500 border-amber-500/30">
                <AlertCircle className="w-3 h-3 inline mr-1" />
                No Key Configured
              </span>
            )}
            {lastConnectionTest && (
              <span className="text-[10px] text-[var(--text-tertiary)] hidden sm:inline">
                Verified at {lastConnectionTest}
              </span>
            )}
          </div>
        </div>

        {/* Feedback Alert */}
        {feedback && (
          <div
            className={`p-3 rounded-[var(--radius-md)] flex items-start gap-2.5 text-xs ${feedback.type === "success"
              ? "bg-[var(--status-success)]/10 border border-[var(--status-success)]/30 text-[var(--status-success)]"
              : feedback.type === "error"
                ? "bg-[var(--status-danger)]/10 border border-[var(--status-danger)]/30 text-[var(--status-danger)]"
                : "bg-[var(--bg-subtle)] border border-[var(--border-default)] text-[var(--text-primary)]"
              }`}
          >
            {feedback.type === "success" ? (
              <CheckCircle2 className="w-4 h-4 shrink-0 mt-0.5" />
            ) : (
              <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
            )}
            <div className="flex-1 min-w-0">
              <span className="break-words">{feedback.message}</span>
            </div>
            <button
              type="button"
              onClick={() => setFeedback(null)}
              className="text-[var(--text-tertiary)] hover:text-[var(--text-primary)] cursor-pointer text-xs"
            >
              Dismiss
            </button>
          </div>
        )}

        {/* Discovery error alert if any */}
        {status?.last_error && !feedback && (
          <div className="p-3 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-amber-500 text-xs flex items-start gap-2">
            <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="font-medium">Model Discovery Notice</p>
              <p className="text-[11px] opacity-90 break-words">{status.last_error}</p>
            </div>
          </div>
        )}

        {/* Provider Selector */}
        <div className="space-y-2">
          <label className="text-xs font-medium text-[var(--text-secondary)]">
            Select AI Provider
          </label>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            <button
              type="button"
              onClick={() => void handleSelectProvider("gemini")}
              className={`fluent-button justify-start p-3 text-xs ${selectedProvider === "gemini" ? "fluent-button-primary" : ""
                }`}
            >
              <Cpu className="w-3.5 h-3.5" />
              <div className="text-left">
                <div className="font-semibold">Google Gemini</div>
                <div className="text-[10px] opacity-80">Full Support (Dynamic Discovery)</div>
              </div>
            </button>

            <button
              type="button"
              onClick={() => void handleSelectProvider("groq")}
              className={`fluent-button justify-start p-3 text-xs ${selectedProvider === "groq" ? "fluent-button-primary" : ""}`}
            >
              <Sparkles className="w-3.5 h-3.5" />
              <div className="text-left">
                <div className="font-semibold">Groq</div>
                <div className="text-[10px] opacity-80">Fast Generation (Gemini Embeddings)</div>
              </div>
            </button>

            <button
              type="button"
              disabled
              className="fluent-button justify-start p-3 text-xs opacity-50 cursor-not-allowed"
              title="Planned for future update"
            >
              <Layers className="w-3.5 h-3.5" />
              <div className="text-left">
                <div className="font-semibold">OpenAI</div>
                <div className="text-[10px] opacity-80">Planned Extension</div>
              </div>
            </button>

            <button
              type="button"
              disabled
              className="fluent-button justify-start p-3 text-xs opacity-50 cursor-not-allowed"
              title="Planned for future update"
            >
              <HardDrive className="w-3.5 h-3.5" />
              <div className="text-left">
                <div className="font-semibold">Ollama</div>
                <div className="text-[10px] opacity-80">Planned Local Provider</div>
              </div>
            </button>
          </div>
        </div>

        {/* API Key Input Field */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-xs font-medium text-[var(--text-secondary)] flex items-center gap-1.5">
              <Key className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
              <span>{selectedProvider.toUpperCase()} API Key</span>
            </label>
            <button
              type="button"
              onClick={(e) => {
                e.preventDefault();
                const url =
                  selectedProvider === "groq"
                    ? "https://console.groq.com/keys"
                    : "https://aistudio.google.com/apikey";
                void handleOpenExternalUrl(url);
              }}
              className="text-[11px] text-[var(--accent-primary)] hover:underline inline-flex items-center gap-1 cursor-pointer focus:outline-none focus-visible:ring-1 focus-visible:ring-[var(--accent-primary)] rounded-[var(--radius-xs)]"
              title={`Open ${selectedProvider === "groq" ? "Groq Console" : "Google AI Studio"} in external browser`}
            >
              <span>Get API key from {selectedProvider === "groq" ? "Groq Console" : "Google AI Studio"}</span>
              <ExternalLink className="w-3 h-3" />
            </button>
          </div>

          <div className="relative">
            <input
              type={showApiKey ? "text" : "password"}
              value={apiKeyInput}
              onChange={(e) => setApiKeyInput(e.target.value)}
              placeholder={
                status?.is_configured
                  ? "•••••••••••••••••••••••••••••••• (Key saved in Windows Keyring. Enter new key to replace)"
                : `Enter your ${selectedProvider.toUpperCase()} API key`
              }
              className="fluent-input text-xs pr-10 py-2.5 font-mono"
              autoComplete="off"
              spellCheck={false}
            />
            <button
              type="button"
              onClick={() => setShowApiKey(!showApiKey)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)] hover:text-[var(--text-primary)] cursor-pointer"
              title={showApiKey ? "Mask key" : "Show key"}
            >
              {showApiKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
            </button>
          </div>
          <p className="text-[11px] text-[var(--text-tertiary)] flex items-center gap-1">
            <Shield className="w-3 h-3 text-[var(--status-success)]" />
            <span>
              Keys are stored securely in Windows Credential Manager. Keys are never saved to SQLite, never written to disk, and never logged.
            </span>
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-wrap items-center gap-2 pt-1">
          <button
            type="button"
            onClick={handleSaveKey}
            disabled={isSaving || !apiKeyInput.trim()}
            className="fluent-button fluent-button-primary text-xs"
          >
            {isSaving ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : null}
            <span>Save to Windows Keyring</span>
          </button>

          <button
            type="button"
            onClick={handleTestConnection}
            disabled={isTesting || (!status?.is_configured && !apiKeyInput.trim())}
            className="fluent-button text-xs"
            title={`Sends a verification request to the official ${selectedProvider.toUpperCase()} API endpoint`}
          >
            {isTesting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : null}
            <span>Test Connection</span>
          </button>

          {status?.is_configured && (
            <button
              type="button"
              onClick={handleRefreshModels}
              disabled={isDiscovering}
              className="fluent-button text-xs"
              title="Queries https://generativelanguage.googleapis.com/v1beta/models to discover all active models"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isDiscovering ? "animate-spin" : ""}`} />
              <span>Refresh Models</span>
            </button>
          )}

          {status?.is_configured && (
            <button
              type="button"
              onClick={handleRemoveKey}
              disabled={isRemoving}
              className="fluent-button text-xs text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
              title="Deletes the saved key from Windows Credential Manager"
            >
              {isRemoving ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Trash2 className="w-3.5 h-3.5" />}
              <span>Remove Key</span>
            </button>
          )}
        </div>

        {/* Embedding Compatibility Alert */}
        {compatibility && !compatibility.is_compatible && (
          <div className="p-3.5 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-xs space-y-2">
            <div className="flex items-start gap-2">
              <AlertCircle className="w-4 h-4 text-amber-500 shrink-0 mt-0.5" />
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-amber-500">Embedding Model Mismatch Detected</p>
                <p className="text-[11px] text-[var(--text-secondary)] mt-0.5 leading-relaxed">
                  {compatibility.message}
                </p>
              </div>
            </div>
            <div className="pt-1 flex items-center gap-2">
              <button
                type="button"
                onClick={handleReindexAll}
                disabled={isReindexing}
                className="fluent-button fluent-button-primary text-xs"
              >
                {isReindexing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <RefreshCw className="w-3.5 h-3.5" />}
                <span>Re-index All Documents ({modelConfig?.embedding_model})</span>
              </button>
            </div>
          </div>
        )}

        {/* Dynamic Model Selection Options */}
        {status?.is_configured && modelConfig && (
          <div className="pt-4 border-t border-[var(--border-subtle)] space-y-5">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-xs font-semibold text-[var(--text-primary)]">
                  Active {selectedProvider.toUpperCase()} Models
                </h3>
                <p className="text-[11px] text-[var(--text-tertiary)]">
                  Select text generation and vector embedding models independently. Groq provides generation while Gemini remains the embedding engine.
                </p>
              </div>
              <button
                type="button"
                onClick={handleRefreshModels}
                disabled={isDiscovering}
                className="fluent-button text-[11px] py-1 px-2.5"
                title="Discover newly released models"
              >
                <RefreshCw className={`w-3 h-3 ${isDiscovering ? "animate-spin" : ""}`} />
                <span>Sync with {selectedProvider.toUpperCase()}</span>
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {/* Generation Model Selection */}
              <div className="space-y-2 p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)]">
                <div className="flex items-center justify-between">
                  <label className="text-xs font-medium text-[var(--text-primary)] flex items-center gap-1.5">
                    <Sparkles className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
                    <span>Generation Model (Q&A & RAG)</span>
                  </label>
                  <span className="text-[10px] text-[var(--text-tertiary)] bg-[var(--bg-default)] px-2 py-0.5 rounded border border-[var(--border-subtle)]">
                    Supports generateContent
                  </span>
                </div>

                <select
                  value={modelConfig.generation_model}
                  onChange={(e) => handleUpdateModel("generation_model", e.target.value)}
                  className="fluent-input text-xs py-2 bg-[var(--bg-default)] w-full"
                >
                  {status.supported_generation_models.map((m) => {
                    const discovered = discoveredGenModels.find((dm) => dm.id === m);
                    return (
                      <option key={m} value={m}>
                        {discovered ? `${discovered.display_name} (${m})` : m}{" "}
                        {m === "gemini-3.5-flash" ? "★ (Recommended Default)" : ""}
                      </option>
                    );
                  })}
                </select>

                <div className="text-[11px] text-[var(--text-tertiary)] flex items-center justify-between pt-1">
                  <span>Selected: <strong className="text-[var(--text-secondary)] font-mono">{modelConfig.generation_model}</strong></span>
                  <span className="fluent-badge text-[9px] bg-emerald-500/10 text-emerald-500 border-emerald-500/30">
                    Active
                  </span>
                </div>
                <p className="text-[10px] text-[var(--text-tertiary)] leading-relaxed">
                  Generates document-grounded answers with source citations inside Ask ECHO.
                </p>
              </div>

              {/* Embedding Model Selection */}
              <div className="space-y-2 p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)]">
                <div className="flex items-center justify-between">
                  <label className="text-xs font-medium text-[var(--text-primary)] flex items-center gap-1.5">
                    <Layers className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
                    <span>Embedding Model (Vector Search)</span>
                  </label>
                  <span className="text-[10px] text-[var(--text-tertiary)] bg-[var(--bg-default)] px-2 py-0.5 rounded border border-[var(--border-subtle)]">
                    Supports embedContent
                  </span>
                </div>

                <select
                  value={modelConfig.embedding_model}
                  onChange={(e) => handleUpdateModel("embedding_model", e.target.value)}
                  className="fluent-input text-xs py-2 bg-[var(--bg-default)] w-full"
                >
                  {status.supported_embedding_models.map((m) => {
                    const discovered = discoveredEmbModels.find((dm) => dm.id === m);
                    return (
                      <option key={m} value={m}>
                        {discovered ? `${discovered.display_name} (${m})` : m}{" "}
                        {m === "gemini-embedding-001" ? "★ (Recommended Default)" : ""}
                      </option>
                    );
                  })}
                </select>

                <div className="text-[11px] text-[var(--text-tertiary)] flex items-center justify-between pt-1">
                  <span>Selected: <strong className="text-[var(--text-secondary)] font-mono">{modelConfig.embedding_model}</strong></span>
                  <span className="fluent-badge text-[9px] bg-emerald-500/10 text-emerald-500 border-emerald-500/30">
                    Active
                  </span>
                </div>
                <p className="text-[10px] text-[var(--text-tertiary)] leading-relaxed">
                  Generates high-dimensional vector representations stored in local SQLite database.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Automatic Knowledge Indexing Toggle */}
        <div className="pt-4 border-t border-[var(--border-subtle)] space-y-2">
          <div className="flex items-center justify-between gap-4">
            <div className="space-y-0.5">
              <div className="text-xs font-semibold text-[var(--text-primary)]">
                Automatically Index Documents on Application Startup
              </div>
              <p className="text-[11px] text-[var(--text-tertiary)] max-w-lg leading-relaxed">
                Asynchronously checks all documents when ECHO starts and indexes unindexed, modified, or eligible documents in the background without blocking the UI.
              </p>
            </div>
            <button
              type="button"
              onClick={handleToggleAutoIndex}
              disabled={isUpdatingAutoIndex}
              className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${autoIndexEnabled ? "bg-[var(--accent-primary)]" : "bg-[var(--border-default)]"
                }`}
              title={autoIndexEnabled ? "Disable automatic startup indexing" : "Enable automatic startup indexing"}
            >
              <span
                aria-hidden="true"
                className={`pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${autoIndexEnabled ? "translate-x-4" : "translate-x-0"
                  }`}
              />
            </button>
          </div>
        </div>

        {/* Transparent Privacy & Data Flow Disclosure */}
        <div className="p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-2 text-xs">
          <div className="font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
            <Shield className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
            <span>Local Boundary & Privacy Transparency</span>
          </div>
          <ul className="list-disc pl-4 space-y-1 text-[11px] text-[var(--text-secondary)] leading-relaxed">
            <li>
              <strong>Local Storage:</strong> All original files, SQLite database tables, document chunks, conversation histories, and embedding vectors stay strictly on your local PC.
            </li>
            <li>
              <strong>Cloud AI Processing:</strong> Only relevant extracted document excerpts and your query are sent to the selected text provider; Gemini remains the embedding service for vector search.
            </li>
            <li>
              <strong>Zero Telemetry:</strong> ECHO has no third-party telemetry, tracking, or developer proxies. Requests go directly from your desktop to the selected provider APIs.
            </li>
            <li>
              <strong>Revocation:</strong> Removing your key immediately disables all cloud AI functionality.
            </li>
          </ul>
        </div>
      </div>

      {/* Appearance Section */}
      <div className="fluent-card p-5 space-y-4">
        <div className="flex items-center gap-2">
          <Moon className="w-4 h-4 text-[var(--accent-primary)]" />
          <h2 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
            Appearance & Theme
          </h2>
        </div>
        <div className="grid grid-cols-3 gap-3">
          <button
            type="button"
            onClick={() => setTheme("dark")}
            className={`fluent-button justify-start p-3 text-xs ${theme === "dark" ? "fluent-button-primary" : ""
              }`}
          >
            <Moon className="w-3.5 h-3.5" />
            <div className="text-left">
              <div className="font-semibold">Dark</div>
              <div className="text-[10px] opacity-80">Fluent dark aesthetic</div>
            </div>
          </button>

          <button
            type="button"
            onClick={() => setTheme("light")}
            className={`fluent-button justify-start p-3 text-xs ${theme === "light" ? "fluent-button-primary" : ""
              }`}
          >
            <Sun className="w-3.5 h-3.5" />
            <div className="text-left">
              <div className="font-semibold">Light</div>
              <div className="text-[10px] opacity-80">Clean high contrast</div>
            </div>
          </button>

          <button
            type="button"
            onClick={() => setTheme("system")}
            className={`fluent-button justify-start p-3 text-xs ${theme === "system" ? "fluent-button-primary" : ""
              }`}
          >
            <Laptop className="w-3.5 h-3.5" />
            <div className="text-left">
              <div className="font-semibold">System</div>
              <div className="text-[10px] opacity-80">Match Windows theme</div>
            </div>
          </button>
        </div>
      </div>

      {/* System Information Card */}
      <div className="fluent-card p-5 space-y-3">
        <div className="flex items-center gap-2 pb-2 border-b border-[var(--border-subtle)]">
          <Info className="w-4 h-4 text-[var(--accent-primary)]" />
          <h2 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
            System & Storage Details
          </h2>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
          <div>
            <span className="text-[var(--text-secondary)]">Application Name:</span>{" "}
            <span className="font-medium text-[var(--text-primary)]">{appInfo.name}</span>
          </div>
          <div>
            <span className="text-[var(--text-secondary)]">Version:</span>{" "}
            <span className="font-medium text-[var(--text-primary)]">{appInfo.version}</span>
          </div>
          <div>
            <span className="text-[var(--text-secondary)]">Desktop Framework:</span>{" "}
            <span className="font-medium text-[var(--text-primary)]">Tauri 2 (Rust)</span>
          </div>
          <div>
            <span className="text-[var(--text-secondary)]">Frontend Engine:</span>{" "}
            <span className="font-medium text-[var(--text-primary)]">React 19 + TypeScript + Vite</span>
          </div>
          <div className="sm:col-span-2">
            <span className="text-[var(--text-secondary)]">Database Location:</span>{" "}
            <span className="font-mono text-[11px] text-[var(--text-tertiary)] break-all">
              %APPDATA%\com.echo.desktop\ECHO\database\echo.db
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Settings;
