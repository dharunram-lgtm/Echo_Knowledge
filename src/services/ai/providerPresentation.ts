import type { SafeProviderStatus } from "@/types/ai";

export type ProviderConnectionState =
  | "loading"
  | "not_configured"
  | "disconnected"
  | "error"
  | "connected";

export interface ProviderPresentation {
  state: ProviderConnectionState;
  label: string;
  detail: string;
  detailTitle?: string;
}

const PROVIDER_DISPLAY_NAMES: Record<string, string> = {
  gemini: "Gemini",
  groq: "Groq",
  openai: "OpenAI",
  ollama: "Ollama",
};

/**
 * Resolves the display name from the configured provider identifier.
 * Provider identity never comes from the model ID or its prefix.
 */
export function getProviderDisplayName(
  provider: string | null | undefined,
): string {
  const id = provider?.trim().toLowerCase() ?? "";
  if (!id) return "";
  return (
    PROVIDER_DISPLAY_NAMES[id] ?? id.charAt(0).toUpperCase() + id.slice(1)
  );
}

/**
 * Derives the dashboard provider card presentation from the backend's
 * SafeProviderStatus, which is the single source of truth for the active
 * provider, model, credential state, and last connection error.
 */
export function presentProviderStatus(
  status: SafeProviderStatus | null,
  isLoading: boolean,
  loadError?: string | null,
): ProviderPresentation {
  if (isLoading) {
    return {
      state: "loading",
      label: "Checking provider...",
      detail: "Reading provider configuration",
    };
  }

  if (loadError) {
    return {
      state: "error",
      label: "Connection error",
      detail: "Unable to read provider status",
      detailTitle: loadError,
    };
  }

  if (!status || !status.provider?.trim()) {
    return {
      state: "not_configured",
      label: "Not configured",
      detail: "Set API key in Settings",
    };
  }

  const providerName = getProviderDisplayName(status.provider);
  const model = status.active_generation_model?.trim() || "No model selected";

  if (!status.is_configured) {
    return {
      state: "disconnected",
      label: "Disconnected",
      detail: model,
      detailTitle: `API key is not configured for ${providerName}`,
    };
  }

  if (status.last_error) {
    return {
      state: "error",
      label: "Connection error",
      detail: model,
      detailTitle: status.last_error,
    };
  }

  return {
    state: "connected",
    label: `Connected (${providerName})`,
    detail: model,
  };
}
