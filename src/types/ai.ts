export type AIProviderId = "gemini" | "groq" | "openai" | "ollama";

export interface DiscoveredModel {
  id: string;
  full_name: string;
  display_name: string;
  description: string;
  supported_methods: string[];
  supports_generation: boolean;
  supports_embedding: boolean;
}

export interface EmbeddingCompatibility {
  is_compatible: boolean;
  active_model: string;
  stored_models: string[];
  total_embeddings: number;
  mismatched_count: number;
  message: string;
}

export interface SafeProviderStatus {
  provider: string;
  is_configured: boolean;
  active_generation_model: string;
  active_embedding_model: string;
  supported_generation_models: string[];
  supported_embedding_models: string[];
  supported_providers: string[];
  discovered_models?: DiscoveredModel[];
  last_error?: string | null;
}

export interface ModelConfig {
  provider: string;
  generation_model: string;
  embedding_model: string;
  temperature: number;
  top_k_retrieval: number;
  min_similarity: number;
}

export interface DocumentIndexResult {
  document_id: string;
  filename: string;
  chunks_count: number;
  embeddings_count: number;
  status: "indexed" | "empty" | "failed";
  error?: string | null;
}

export interface BatchIndexResult {
  total_processed: number;
  successful: DocumentIndexResult[];
  failed: DocumentIndexResult[];
}

export type IndexingState = "unindexed" | "pending" | "indexing" | "indexed" | "failed" | "awaiting_key";

export interface DocumentIndexStatus {
  document_id: string;
  filename: string;
  file_type: string;
  is_indexed: boolean;
  status: IndexingState;
  chunk_count: number;
  embedding_count: number;
  error_message?: string | null;
  last_attempt_time?: string | null;
  attempt_count?: number;
  next_retry_at?: string | null;
  retry_state?: string;
  last_successful_attempt_at?: string | null;
  last_error?: string | null;
}

export interface IndexingOverview {
  total_documents: number;
  indexed_documents: number;
  pending_documents: number;
  currently_processing_documents: number;
  failed_documents: number;
  awaiting_key_documents: number;
  total_chunks: number;
  total_embeddings: number;
  last_successful_indexing_time?: string | null;
  documents: DocumentIndexStatus[];
}

export interface SemanticSearchResult {
  document_id: string;
  filename: string;
  file_type: string;
  chunk_id: string;
  chunk_index: number;
  chunk_text: string;
  similarity: number;
  snippet: string;
}

export interface Citation {
  document_id: string;
  filename: string;
  file_type: string;
  chunk_id: string;
  chunk_index: number;
  snippet: string;
  similarity: number;
}

export interface AskEchoResponse {
  conversation_id: string;
  answer: string;
  citations: Citation[];
  insufficient_context: boolean;
  sources_used: number;
}

export interface Conversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

export interface ConversationMessage {
  id: string;
  conversation_id: string;
  role: "user" | "assistant";
  content: string;
  citations_json?: string | null;
  created_at: string;
}
