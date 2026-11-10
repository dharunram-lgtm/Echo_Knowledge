/**
 * AI Service Contracts
 *
 * Defines typed interfaces for calling AI completion and embedding endpoints
 * via safe Tauri backend IPC.
 */

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface CompletionRequest {
  providerId: string;
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  maxTokens?: number;
}

export interface CompletionResponse {
  content: string;
  finishReason?: string;
  model: string;
}

export interface AIServiceContract {
  checkProviderHealth(providerId: string): Promise<boolean>;
  generateCompletion(request: CompletionRequest): Promise<CompletionResponse>;
}

