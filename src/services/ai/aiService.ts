import type {
  AskEchoResponse,
  BatchIndexResult,
  Conversation,
  ConversationMessage,
  DiscoveredModel,
  DocumentIndexResult,
  EmbeddingCompatibility,
  IndexingOverview,
  ModelConfig,
  SafeProviderStatus,
  SemanticSearchResult,
} from "@/types/ai";

export class AIService {
  /**
   * Retrieves safe provider status (provider, models, whether key is configured)
   * Never exposes the API key string to the frontend.
   */
  public static async getProviderStatus(): Promise<SafeProviderStatus> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<SafeProviderStatus>("get_ai_status");
  }

  /**
   * Securely saves an API key to Windows Credential Manager via backend
   */
  public static async saveApiKey(provider: string, apiKey: string): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<void>("save_api_key", { provider, apiKey });
  }

  /**
   * Removes an API key from Windows Credential Manager
   */
  public static async removeApiKey(provider: string): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("remove_api_key", { provider });
  }

  /**
   * Tests connection with the provider using either a transient key or saved key
   */
  public static async testConnection(provider: string, apiKey?: string): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("test_ai_connection", { provider, apiKey });
  }

  /**
   * Dynamically discovers models available to the configured API key
   */
  public static async discoverModels(provider?: string): Promise<DiscoveredModel[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<DiscoveredModel[]>("discover_models", { provider });
  }

  /**
   * Checks whether stored document embeddings match the active embedding model
   */
  public static async checkEmbeddingCompatibility(): Promise<EmbeddingCompatibility> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<EmbeddingCompatibility>("check_embedding_compatibility");
  }

  /**
   * Retrieves active model configuration
   */
  public static async getModelConfig(): Promise<ModelConfig> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<ModelConfig>("get_model_config");
  }

  /**
   * Updates model configuration in backend
   */
  public static async updateModelConfig(config: ModelConfig): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<void>("update_model_config", { config });
  }

  /**
   * Indexes a single document: chunks text, generates embeddings, saves to SQLite
   */
  public static async indexDocument(documentId: string): Promise<DocumentIndexResult> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<DocumentIndexResult>("index_document", { documentId });
  }

  /**
   * Batch indexes all unindexed documents
   */
  public static async indexAllDocuments(): Promise<BatchIndexResult> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<BatchIndexResult>("index_all_documents");
  }

  /**
   * Retrieves overall indexing status and per-document stats
   */
  public static async getIndexingStatus(): Promise<IndexingOverview> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<IndexingOverview>("get_indexing_status");
  }

  /**
   * Executes local vector cosine-similarity search against stored document chunks
   */
  public static async semanticSearch(
    query: string,
    limit?: number
  ): Promise<SemanticSearchResult[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<SemanticSearchResult[]>("semantic_search", { query, limit });
  }

  /**
   * Executes grounded RAG question answering across personal documents
   */
  public static async askEcho(
    question: string,
    conversationId?: string | null
  ): Promise<AskEchoResponse> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<AskEchoResponse>("ask_echo", {
      question,
      conversationId: conversationId || null,
    });
  }

  /**
   * Lists all local conversations
   */
  public static async listConversations(): Promise<Conversation[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Conversation[]>("list_conversations");
  }

  /**
   * Creates a new conversation
   */
  public static async createConversation(title?: string): Promise<Conversation> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Conversation>("create_conversation", { title: title || null });
  }

  /**
   * Retrieves all messages for a conversation
   */
  public static async getConversationMessages(
    conversationId: string
  ): Promise<ConversationMessage[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<ConversationMessage[]>("get_conversation_messages", { conversationId });
  }

  /**
   * Deletes a conversation and its messages
   */
  public static async deleteConversation(conversationId: string): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("delete_conversation", { conversationId });
  }

  /**
   * Retrieves startup auto-indexing configuration setting
   */
  public static async getAutoIndexSetting(): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("get_auto_index_setting");
  }

  /**
   * Updates startup auto-indexing configuration setting
   */
  public static async setAutoIndexSetting(enabled: boolean): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<void>("set_auto_index_setting", { enabled });
  }

  /**
   * Retries indexing for all failed documents
   */
  public static async retryFailedIndexing(): Promise<BatchIndexResult> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<BatchIndexResult>("retry_failed_indexing");
  }

  /**
   * Clears all messages in a conversation
   */
  public static async clearConversation(conversationId: string): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("clear_conversation", { conversationId });
  }
}

export default AIService;

