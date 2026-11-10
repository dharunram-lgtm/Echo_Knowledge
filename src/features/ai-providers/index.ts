/**
 * AI Providers Feature Module
 *
 * Manages BYO (Bring-Your-Own) API provider configurations.
 * Note: API keys and credentials are never stored in frontend code.
 * They are securely managed via Rust backend IPC calls into the OS credential store.
 */

export type SupportedProviderId = "openai" | "anthropic" | "gemini" | "ollama";

export interface ProviderManifest {
  id: SupportedProviderId;
  name: string;
  description: string;
  isLocal: boolean;
  requiresApiKey: boolean;
  defaultBaseUrl?: string;
  defaultModels: string[];
}

export const SUPPORTED_PROVIDERS: ProviderManifest[] = [
  {
    id: "openai",
    name: "OpenAI",
    description: "GPT-4o, o3-mini and text-embedding-3 models.",
    isLocal: false,
    requiresApiKey: true,
    defaultModels: ["gpt-4o", "gpt-4o-mini", "text-embedding-3-small"],
  },
  {
    id: "anthropic",
    name: "Anthropic Claude",
    description: "Claude 3.7 Sonnet, Claude 3.5 Haiku models.",
    isLocal: false,
    requiresApiKey: true,
    defaultModels: ["claude-3-7-sonnet", "claude-3-5-haiku"],
  },
  {
    id: "gemini",
    name: "Google Gemini",
    description: "Gemini 2.5 Pro, Flash, and text-embedding-004.",
    isLocal: false,
    requiresApiKey: true,
    defaultModels: ["gemini-2.5-flash", "gemini-2.5-pro", "text-embedding-004"],
  },
  {
    id: "ollama",
    name: "Ollama (Local LLM)",
    description: "100% offline local inference without API keys.",
    isLocal: true,
    requiresApiKey: false,
    defaultBaseUrl: "http://localhost:11434",
    defaultModels: ["llama3.2", "mistral", "nomic-embed-text"],
  },
];

