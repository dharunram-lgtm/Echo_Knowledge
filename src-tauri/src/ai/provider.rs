use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Gemini,
    Groq,
    OpenAI,
    Ollama,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::Groq => "groq",
            Self::OpenAI => "openai",
            Self::Ollama => "ollama",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "groq" => Self::Groq,
            "openai" => Self::OpenAI,
            "ollama" => Self::Ollama,
            _ => Self::Gemini,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub generation_model: String,
    pub embedding_model: String,
    pub temperature: f32,
    pub top_k_retrieval: usize,
    pub min_similarity: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "gemini".to_string(),
            generation_model: "gemini-3.5-flash".to_string(),
            embedding_model: "gemini-embedding-001".to_string(),
            temperature: 0.2,
            top_k_retrieval: 5,
            min_similarity: 0.35,
        }
    }
}

/// Returns the default text-generation model for the configured provider.
/// The provider identifier is authoritative; model names are never used to
/// infer which provider is active.
pub fn default_generation_model_for(provider: &str) -> &'static str {
    if provider.trim().eq_ignore_ascii_case("groq") {
        crate::ai::groq::DEFAULT_GROQ_GENERATION_MODEL
    } else {
        crate::ai::gemini::DEFAULT_GENERATION_MODEL
    }
}

/// Detects a generation-model/provider mismatch using the known model
/// catalogues for each provider. This intentionally checks model ownership
/// rather than inferring the provider from a model-name prefix.
pub fn generation_model_conflicts_with_provider(provider: &str, model: &str) -> bool {
    let provider = provider.trim().to_lowercase();
    let retired_gemini_model =
        matches!(model, "gemini-1.5-flash" | "gemini-2.0-flash");

    match provider.as_str() {
        "groq" => {
            retired_gemini_model
                || crate::ai::gemini::SUPPORTED_GENERATION_MODELS.contains(&model)
        }
        "gemini" => {
            retired_gemini_model
                || crate::ai::groq::SUPPORTED_GROQ_MODELS.contains(&model)
        }
        _ => false,
    }
}

/// Repairs stale or cross-provider generation-model selections while keeping
/// the configured provider as the single source of truth.
pub fn normalize_generation_model(config: &mut ModelConfig) -> bool {
    if generation_model_conflicts_with_provider(&config.provider, &config.generation_model) {
        config.generation_model = default_generation_model_for(&config.provider).to_string();
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub id: String,
    pub full_name: String,
    pub display_name: String,
    pub description: String,
    pub supported_methods: Vec<String>,
    pub supports_generation: bool,
    pub supports_embedding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCompatibility {
    pub is_compatible: bool,
    pub active_model: String,
    pub stored_models: Vec<String>,
    pub total_embeddings: usize,
    pub mismatched_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeProviderStatus {
    pub provider: String,
    pub is_configured: bool,
    pub active_generation_model: String,
    pub active_embedding_model: String,
    pub supported_generation_models: Vec<String>,
    pub supported_embedding_models: Vec<String>,
    pub supported_providers: Vec<String>,
    pub discovered_models: Vec<DiscoveredModel>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    pub document_id: String,
    pub filename: String,
    pub file_type: String,
    pub chunk_id: String,
    pub chunk_index: usize,
    pub chunk_text: String,
    pub similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub document_id: String,
    pub filename: String,
    pub file_type: String,
    pub chunk_id: String,
    pub chunk_index: usize,
    pub snippet: String,
    pub similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswer {
    pub answer: String,
    pub citations: Vec<Citation>,
    pub insufficient_context: bool,
}

/// Reusable provider abstraction for text generation capabilities
pub trait TextGenerationProvider: Send + Sync {
    fn generate_grounded_answer(
        &self,
        api_key: &str,
        model: &str,
        question: &str,
        context_chunks: &[ContextChunk],
        conversation_history: &[crate::database::ConversationMessage],
    ) -> impl std::future::Future<Output = Result<GroundedAnswer, String>> + Send;

    fn list_models(
        &self,
        api_key: &str,
    ) -> impl std::future::Future<Output = Result<Vec<DiscoveredModel>, String>> + Send;

    fn test_connection(
        &self,
        api_key: &str,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send;
}

/// Dispatches grounded Q&A generation to the selected text provider (Gemini or Groq)
pub async fn dispatch_grounded_answer(
    gemini: &crate::ai::gemini::GeminiClient,
    groq: &crate::ai::groq::GroqClient,
    provider: &str,
    api_key: &str,
    model: &str,
    question: &str,
    context_chunks: &[ContextChunk],
    conversation_history: &[crate::database::ConversationMessage],
) -> Result<GroundedAnswer, String> {
    match provider.trim().to_lowercase().as_str() {
        "groq" => {
            groq.generate_grounded_answer(
                api_key,
                model,
                question,
                context_chunks,
                conversation_history,
            )
            .await
        }
        _ => {
            gemini
                .generate_grounded_answer(
                    api_key,
                    model,
                    question,
                    context_chunks,
                    conversation_history,
                )
                .await
        }
    }
}

/// Dispatches automatic tagging to the selected text provider (Gemini or Groq)
pub async fn dispatch_extract_tags(
    gemini: &crate::ai::gemini::GeminiClient,
    groq: &crate::ai::groq::GroqClient,
    provider: &str,
    api_key: &str,
    model: &str,
    filename: &str,
    text: &str,
) -> Result<Vec<crate::ai::tagging::service::ExtractedTag>, String> {
    match provider.trim().to_lowercase().as_str() {
        "groq" => groq.extract_tags(api_key, model, filename, text).await,
        _ => {
            crate::ai::tagging::service::extract_tags_with_gemini(
                gemini, api_key, model, filename, text,
            )
            .await
        }
    }
}

/// Dispatches relationship discovery to the selected text provider (Gemini or Groq)
pub async fn dispatch_extract_relationships(
    gemini: &crate::ai::gemini::GeminiClient,
    groq: &crate::ai::groq::GroqClient,
    provider: &str,
    api_key: &str,
    model: &str,
    filename: &str,
    text: &str,
    concepts: &[crate::database::DocumentConceptRecord],
) -> Result<Vec<crate::ai::graph::discovery::RawRelationshipItem>, String> {
    match provider.trim().to_lowercase().as_str() {
        "groq" => {
            groq.extract_relationships(api_key, model, filename, text, concepts)
                .await
        }
        _ => {
            crate::ai::graph::discovery::extract_relationships_with_gemini(
                gemini,
                api_key,
                model,
                filename,
                text,
                concepts,
            )
            .await
        }
    }
}

