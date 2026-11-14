import React, { useState, useEffect, useCallback } from "react";
import {
  Globe,
  Power,
  RotateCw,
  Trash2,
  Copy,
  Eye,
  EyeOff,
  Plus,
  Shield,
  CheckCircle2,
  AlertCircle,
  Loader2,
  HelpCircle,
} from "lucide-react";
import { WebClipperService } from "@/services/webClipper/webClipperService";
import { WebClipperConfirmModal } from "./WebClipperConfirmModal";
import type {
  ClipperPairingInfo,
  AuthorizedExtensionInfo,
} from "@/types/webClipper";

export const WebClipperSettingsSection: React.FC = () => {
  const [pairingInfo, setPairingInfo] = useState<ClipperPairingInfo | null>(
    null
  );
  const [loading, setLoading] = useState<boolean>(true);
  const [revealedToken, setRevealedToken] = useState<string | null>(null);
  const [isRevealingToken, setIsRevealingToken] = useState<boolean>(false);
  const [copiedToken, setCopiedToken] = useState<boolean>(false);
  const [testingConnection, setTestingConnection] = useState<boolean>(false);
  const [connectionStatus, setConnectionStatus] = useState<
    "idle" | "success" | "error"
  >("idle");

  // Pair new browser state
  const [newExtId, setNewExtId] = useState<string>("");
  const [newBrowserName, setNewBrowserName] = useState<string>("Brave Browser");
  const [isAuthorizing, setIsAuthorizing] = useState<boolean>(false);
  const [showPairingGuide, setShowPairingGuide] = useState<boolean>(false);

  // Feedback banner
  const [feedback, setFeedback] = useState<{
    type: "success" | "error" | "info";
    message: string;
  } | null>(null);

  // Modal confirmations
  const [modalState, setModalState] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    confirmLabel: string;
    action: () => Promise<void>;
    isLoading: boolean;
  }>({
    isOpen: false,
    title: "",
    message: "",
    confirmLabel: "",
    action: async () => { },
    isLoading: false,
  });

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      const info = await WebClipperService.getPairingInfo();
      setPairingInfo(info);
    } catch (err) {
      console.error("Failed to load clipper pairing info:", err);
      setFeedback({
        type: "error",
        message: "Failed to query Web Clipper status: " + String(err),
      });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleToggleServer = async () => {
    if (!pairingInfo) return;
    try {
      const nextState = !pairingInfo.is_running;
      await WebClipperService.setServerState(nextState);
      setPairingInfo((prev) =>
        prev ? { ...prev, is_running: nextState } : null
      );
      setFeedback({
        type: "info",
        message: `Web Clipper listener ${nextState ? "started" : "stopped"}.`,
      });
      setConnectionStatus("idle");
    } catch (err) {
      setFeedback({
        type: "error",
        message: "Failed to toggle server: " + String(err),
      });
    }
  };

  const handleTestConnection = async () => {
    try {
      setTestingConnection(true);
      const res = await WebClipperService.testConnection();
      setConnectionStatus(res.reachable ? "success" : "error");
      setFeedback({
        type: res.reachable ? "success" : "error",
        message: res.message,
      });
    } catch (err) {
      setConnectionStatus("error");
      setFeedback({
        type: "error",
        message: "Connection test error: " + String(err),
      });
    } finally {
      setTestingConnection(false);
    }
  };

  const handleToggleRevealToken = async () => {
    if (revealedToken) {
      // Hide and clear from memory
      setRevealedToken(null);
    } else {
      // Explicit reveal: fetch on-demand from OS Credential Manager
      try {
        setIsRevealingToken(true);
        const token = await WebClipperService.revealToken();
        setRevealedToken(token);
      } catch (err) {
        setFeedback({
          type: "error",
          message: "Failed to retrieve token from Credential Manager: " + String(err),
        });
      } finally {
        setIsRevealingToken(false);
      }
    }
  };

  const handleCopyToken = async () => {
    try {
      let tokenToCopy = revealedToken;
      if (!tokenToCopy) {
        tokenToCopy = await WebClipperService.revealToken();
      }
      if (!tokenToCopy) return;

      await navigator.clipboard.writeText(tokenToCopy);
      setCopiedToken(true);
      setTimeout(() => setCopiedToken(false), 2000);
      setFeedback({
        type: "success",
        message: "Pairing token copied to clipboard.",
      });
    } catch (err) {
      setFeedback({
        type: "error",
        message: "Failed to copy token: " + String(err),
      });
    }
  };

  const promptRegenerateToken = () => {
    setModalState({
      isOpen: true,
      title: "Regenerate Pairing Token?",
      message:
        "Regenerating the 256-bit pairing token will immediately revoke and disconnect all currently authorized browser extensions. You will need to pair your browsers again with the new token.",
      confirmLabel: "Regenerate & Disconnect All",
      isLoading: false,
      action: async () => {
        try {
          setModalState((prev) => ({ ...prev, isLoading: true }));
          await WebClipperService.regenerateToken();
          setRevealedToken(null);
          await loadData();
          setFeedback({
            type: "success",
            message:
              "New pairing token generated. All browser pairings invalidated.",
          });
        } catch (err) {
          setFeedback({
            type: "error",
            message: "Failed to regenerate token: " + String(err),
          });
        } finally {
          setModalState((prev) => ({ ...prev, isOpen: false, isLoading: false }));
        }
      },
    });
  };

  const promptRevokeToken = () => {
    setModalState({
      isOpen: true,
      title: "Revoke Pairing Token?",
      message:
        "Revoking the pairing token removes it from Windows Credential Manager and deauthorizes all browser installations. Web notes cannot be saved until you re-enable or regenerate a token.",
      confirmLabel: "Revoke Token",
      isLoading: false,
      action: async () => {
        try {
          setModalState((prev) => ({ ...prev, isLoading: true }));
          await WebClipperService.revokeToken();
          setRevealedToken(null);
          await loadData();
          setFeedback({
            type: "info",
            message: "Pairing token and all browser pairings revoked.",
          });
        } catch (err) {
          setFeedback({
            type: "error",
            message: "Failed to revoke token: " + String(err),
          });
        } finally {
          setModalState((prev) => ({ ...prev, isOpen: false, isLoading: false }));
        }
      },
    });
  };

  const handleAuthorizeNewExtension = async (e: React.FormEvent) => {
    e.preventDefault();
    const cleanId = newExtId.trim().toLowerCase();

    if (cleanId.length !== 32 || !/^[a-p]{32}$/.test(cleanId)) {
      setFeedback({
        type: "error",
        message:
          "Invalid Extension ID format. Chromium extension IDs must be exactly 32 lowercase letters between a and p.",
      });
      return;
    }

    try {
      setIsAuthorizing(true);
      await WebClipperService.authorizeExtension(cleanId, newBrowserName);
      setNewExtId("");
      await loadData();
      setFeedback({
        type: "success",
        message: `Extension ID authorized for ${newBrowserName}!`,
      });
    } catch (err) {
      setFeedback({
        type: "error",
        message: "Failed to authorize extension: " + String(err),
      });
    } finally {
      setIsAuthorizing(false);
    }
  };

  const promptRevokeExtension = (ext: AuthorizedExtensionInfo) => {
    setModalState({
      isOpen: true,
      title: `Revoke ${ext.browser || "Browser"} Pairing?`,
      message: `Are you sure you want to deauthorize extension ID ${ext.extension_id}? This browser will no longer be able to save Web Notes into ECHO Knowledge.`,
      confirmLabel: "Revoke Authorization",
      isLoading: false,
      action: async () => {
        try {
          setModalState((prev) => ({ ...prev, isLoading: true }));
          await WebClipperService.deauthorizeExtension(ext.extension_id);
          await loadData();
          setFeedback({
            type: "info",
            message: `Authorization for ${ext.browser || "browser"} revoked.`,
          });
        } catch (err) {
          setFeedback({
            type: "error",
            message: "Failed to revoke browser: " + String(err),
          });
        } finally {
          setModalState((prev) => ({ ...prev, isOpen: false, isLoading: false }));
        }
      },
    });
  };

  const promptRevokeAllExtensions = () => {
    setModalState({
      isOpen: true,
      title: "Revoke All Paired Browsers?",
      message:
        "Are you sure you want to disconnect all browser installations? All extensions will require re-pairing in Settings before they can clip again.",
      confirmLabel: "Revoke All Browsers",
      isLoading: false,
      action: async () => {
        try {
          setModalState((prev) => ({ ...prev, isLoading: true }));
          await WebClipperService.deauthorizeExtension("*");
          await loadData();
          setFeedback({
            type: "info",
            message: "All browser pairings have been revoked.",
          });
        } catch (err) {
          setFeedback({
            type: "error",
            message: "Failed to revoke all browsers: " + String(err),
          });
        } finally {
          setModalState((prev) => ({ ...prev, isOpen: false, isLoading: false }));
        }
      },
    });
  };

  return (
    <div className="fluent-card p-5 space-y-5">
      {/* Header */}
      <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2.5">
          <Globe className="w-5 h-5 text-[var(--accent-primary)]" />
          <div>
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">
              Web Clipper Integration
            </h2>
            <p className="text-[11px] text-[var(--text-secondary)]">
              Save articles, excerpts, and bookmarks from Brave, Chrome, and Edge directly into ECHO.
            </p>
          </div>
        </div>

        {/* Server Toggle Button */}
        <div className="flex items-center gap-2">
          {pairingInfo?.is_running ? (
            <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-medium bg-[var(--status-success)]/15 text-[var(--status-success)]">
              <span className="w-2 h-2 rounded-full bg-[var(--status-success)] animate-pulse" />
              Active (Port {pairingInfo.port})
            </span>
          ) : (
            <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-medium bg-[var(--text-tertiary)]/15 text-[var(--text-tertiary)]">
              <span className="w-2 h-2 rounded-full bg-[var(--text-tertiary)]" />
              Stopped
            </span>
          )}

          <button
            type="button"
            onClick={handleToggleServer}
            disabled={loading}
            className={`fluent-button text-xs px-3 py-1.5 ${pairingInfo?.is_running
              ? "text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
              : "fluent-button-primary"
              }`}
          >
            <Power className="w-3.5 h-3.5 mr-1" />
            {pairingInfo?.is_running ? "Stop Server" : "Start Server"}
          </button>
        </div>
      </div>

      {/* Feedback Banner */}
      {feedback && (
        <div
          className={`p-3 rounded-[var(--radius-md)] text-xs flex items-center justify-between gap-2 ${feedback.type === "success"
            ? "bg-[var(--status-success)]/10 text-[var(--status-success)] border border-[var(--status-success)]/20"
            : feedback.type === "error"
              ? "bg-[var(--status-danger)]/10 text-[var(--status-danger)] border border-[var(--status-danger)]/20"
              : "bg-[var(--accent-primary)]/10 text-[var(--accent-primary)] border border-[var(--accent-primary)]/20"
            }`}
        >
          <div className="flex items-center gap-2">
            {feedback.type === "success" && (
              <CheckCircle2 className="w-4 h-4 shrink-0" />
            )}
            {feedback.type === "error" && (
              <AlertCircle className="w-4 h-4 shrink-0" />
            )}
            {feedback.type === "info" && <Shield className="w-4 h-4 shrink-0" />}
            <span>{feedback.message}</span>
          </div>
          <button
            type="button"
            onClick={() => setFeedback(null)}
            className="text-[11px] opacity-70 hover:opacity-100"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Connection & Port Diagnostics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)]">
          <div className="text-[11px] text-[var(--text-secondary)] font-medium">
            Active Port
          </div>
          <div className="text-sm font-semibold text-[var(--text-primary)] mt-1 flex items-center justify-between">
            <span>{pairingInfo?.port || 23119}</span>
            <span className="text-[10px] text-[var(--text-tertiary)] font-normal">
              {pairingInfo?.port === 23119
                ? "Preferred (23119)"
                : "Fallback Active (23120)"}
            </span>
          </div>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)]">
          <div className="text-[11px] text-[var(--text-secondary)] font-medium">
            Loopback Interface
          </div>
          <div className="text-sm font-semibold text-[var(--text-primary)] mt-1">
            127.0.0.1 (Localhost Only)
          </div>
        </div>

        <div className="p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] flex items-center justify-between">
          <div>
            <div className="text-[11px] text-[var(--text-secondary)] font-medium">
              Diagnostics Test
            </div>
            <div className="text-xs text-[var(--text-primary)] mt-1">
              {connectionStatus === "success" && (
                <span className="text-[var(--status-success)] flex items-center gap-1 font-medium">
                  <CheckCircle2 className="w-3.5 h-3.5" /> Reachable
                </span>
              )}
              {connectionStatus === "error" && (
                <span className="text-[var(--status-danger)] flex items-center gap-1 font-medium">
                  <AlertCircle className="w-3.5 h-3.5" /> Unreachable
                </span>
              )}
              {connectionStatus === "idle" && (
                <span className="text-[var(--text-tertiary)]">Not tested</span>
              )}
            </div>
          </div>
          <button
            type="button"
            onClick={handleTestConnection}
            disabled={testingConnection}
            className="fluent-button text-xs px-2.5 py-1"
          >
            {testingConnection ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              "Test Link"
            )}
          </button>
        </div>
      </div>

      {/* Pairing Token Card */}
      <div className="space-y-3 pt-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <Shield className="w-4 h-4 text-[var(--accent-primary)]" />
            <h3 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
              Pairing Security Credentials
            </h3>
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={promptRegenerateToken}
              disabled={loading}
              className="fluent-button text-xs px-2.5 py-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              title="Generates a new token and revokes all paired browsers"
            >
              <RotateCw className="w-3 h-3 mr-1" />
              Regenerate
            </button>
            <button
              type="button"
              onClick={promptRevokeToken}
              disabled={loading}
              className="fluent-button text-xs px-2.5 py-1 text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
              title="Revokes pairing credentials completely"
            >
              <Trash2 className="w-3 h-3 mr-1" />
              Revoke
            </button>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative flex-1">
            <input
              type={revealedToken ? "text" : "password"}
              readOnly
              value={
                revealedToken
                  ? revealedToken
                  : pairingInfo?.token_configured
                    ? "••••••••••••••••••••••••••••••••"
                    : "No token configured"
              }
              className="fluent-input font-mono text-xs pr-10 w-full"
            />
            <button
              type="button"
              onClick={handleToggleRevealToken}
              disabled={isRevealingToken || !pairingInfo?.token_configured}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
              title={revealedToken ? "Hide token" : "Reveal token"}
            >
              {isRevealingToken ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : revealedToken ? (
                <EyeOff className="w-3.5 h-3.5" />
              ) : (
                <Eye className="w-3.5 h-3.5" />
              )}
            </button>
          </div>

          <button
            type="button"
            onClick={handleCopyToken}
            disabled={!pairingInfo?.token_configured}
            className="fluent-button text-xs px-3 py-2 shrink-0"
          >
            <Copy className="w-3.5 h-3.5 mr-1" />
            {copiedToken ? "Copied!" : "Copy Token"}
          </button>
        </div>
        <p className="text-[11px] text-[var(--text-tertiary)]">
          Stored securely in Windows Credential Manager. Plaintext is retrieved on-demand only when you explicitly reveal or copy it.
        </p>
      </div>

      {/* Multi-Browser Authorizations Section */}
      <div className="space-y-3 pt-3 border-t border-[var(--border-subtle)]">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
              Authorized Browsers ({pairingInfo?.authorized_extensions?.length || 0})
            </h3>
            <p className="text-[11px] text-[var(--text-secondary)]">
              Manage individual extension IDs allowed to submit Web Notes.
            </p>
          </div>

          {pairingInfo && pairingInfo.authorized_extensions.length > 1 && (
            <button
              type="button"
              onClick={promptRevokeAllExtensions}
              className="fluent-button text-xs text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10 px-2.5 py-1"
            >
              Revoke All
            </button>
          )}
        </div>

        {/* Browser List */}
        {pairingInfo?.authorized_extensions &&
          pairingInfo.authorized_extensions.length > 0 ? (
          <div className="space-y-2">
            {pairingInfo.authorized_extensions.map((ext) => (
              <div
                key={ext.extension_id}
                className="flex items-center justify-between p-3 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)]"
              >
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-semibold text-[var(--text-primary)]">
                      {ext.browser || "Chromium Browser"}
                    </span>
                    <span className="px-2 py-0.5 rounded text-[10px] bg-[var(--status-success)]/15 text-[var(--status-success)] font-medium">
                      Authorized
                    </span>
                  </div>
                  <div className="font-mono text-[11px] text-[var(--text-secondary)]">
                    ID: {ext.extension_id}
                  </div>
                  <div className="text-[10px] text-[var(--text-tertiary)]">
                    Paired: {new Date(ext.paired_at).toLocaleDateString()} at{" "}
                    {new Date(ext.paired_at).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                    })}
                  </div>
                </div>

                <button
                  type="button"
                  onClick={() => promptRevokeExtension(ext)}
                  className="fluent-button text-xs px-2.5 py-1 text-[var(--status-danger)] hover:bg-[var(--status-danger)]/10"
                >
                  <Trash2 className="w-3.5 h-3.5 mr-1" />
                  Revoke
                </button>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-6 border border-dashed border-[var(--border-subtle)] rounded-[var(--radius-md)]">
            <Globe className="w-7 h-7 text-[var(--text-tertiary)] mx-auto mb-2 opacity-50" />
            <div className="text-xs font-medium text-[var(--text-secondary)]">
              No browsers currently authorized
            </div>
            <div className="text-[11px] text-[var(--text-tertiary)] mt-0.5">
              Add your extension ID below to permit capture requests.
            </div>
          </div>
        )}

        {/* Add / Pair Browser Form */}
        <form
          onSubmit={handleAuthorizeNewExtension}
          className="p-3.5 rounded-[var(--radius-md)] bg-[var(--bg-subtle)] border border-[var(--border-subtle)] space-y-3"
        >
          <div className="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
            <Plus className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
            Authorize Another Browser
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
            <div>
              <label className="text-[11px] font-medium text-[var(--text-secondary)] block mb-1">
                Browser
              </label>
              <select
                value={newBrowserName}
                onChange={(e) => setNewBrowserName(e.target.value)}
                className="fluent-input text-xs w-full"
              >
                <option value="Brave Browser">Brave Browser</option>
                <option value="Microsoft Edge">Microsoft Edge</option>
                <option value="Google Chrome">Google Chrome</option>
                <option value="Other Chromium">Other Chromium</option>
              </select>
            </div>

            <div className="sm:col-span-2">
              <label className="text-[11px] font-medium text-[var(--text-secondary)] block mb-1">
                Extension ID (32 Characters)
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="e.g. ceeecehafdiielifgklginjhfdoacfif"
                  value={newExtId}
                  onChange={(e) => setNewExtId(e.target.value)}
                  className="fluent-input font-mono text-xs flex-1"
                />
                <button
                  type="submit"
                  disabled={isAuthorizing || !newExtId.trim()}
                  className="fluent-button fluent-button-primary text-xs px-3.5 shrink-0"
                >
                  {isAuthorizing ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    "Authorize"
                  )}
                </button>
              </div>
            </div>
          </div>

          {/* Quick Help Link */}
          <div className="flex items-center justify-between text-[11px] pt-1">
            <button
              type="button"
              onClick={() => setShowPairingGuide((prev) => !prev)}
              className="text-[var(--accent-primary)] hover:underline flex items-center gap-1"
            >
              <HelpCircle className="w-3.5 h-3.5" />
              {showPairingGuide ? "Hide pairing instructions" : "How to find your Extension ID?"}
            </button>
          </div>

          {showPairingGuide && (
            <div className="p-3 bg-[var(--bg-elevated)] rounded border border-[var(--border-subtle)] text-[11px] text-[var(--text-secondary)] space-y-1.5">
              <p className="font-semibold text-[var(--text-primary)]">
                Step-by-step pairing:
              </p>
              <ol className="list-decimal list-inside space-y-1">
                <li>
                  Open your browser extensions manager (
                  <code className="bg-[var(--bg-subtle)] px-1 py-0.5 rounded font-mono">
                    brave://extensions
                  </code>
                  ,{" "}
                  <code className="bg-[var(--bg-subtle)] px-1 py-0.5 rounded font-mono">
                    edge://extensions
                  </code>
                  , or{" "}
                  <code className="bg-[var(--bg-subtle)] px-1 py-0.5 rounded font-mono">
                    chrome://extensions
                  </code>
                  ).
                </li>
                <li>
                  Toggle <strong>Developer Mode</strong> in the top-right corner.
                </li>
                <li>
                  Locate <strong>ECHO Web Clipper</strong> and copy the 32-character ID.
                </li>
                <li>
                  Paste the ID above and click <strong>Authorize</strong>.
                </li>
                <li>
                  In your extension popup, click Settings (&apos;gear&apos; icon) and paste your
                  Pairing Token from above.
                </li>
              </ol>
            </div>
          )}
        </form>
      </div>

      {/* Confirmation Modal */}
      <WebClipperConfirmModal
        isOpen={modalState.isOpen}
        title={modalState.title}
        message={modalState.message}
        confirmLabel={modalState.confirmLabel}
        onConfirm={modalState.action}
        onCancel={() =>
          setModalState((prev) => ({ ...prev, isOpen: false, isLoading: false }))
        }
        isLoading={modalState.isLoading}
      />
    </div>
  );
};

export default WebClipperSettingsSection;
