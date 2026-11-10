use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use super::gemini::GeminiClient;
use super::groq::GroqClient;
use super::provider::{Citation, GroundedAnswer, ModelConfig, dispatch_grounded_answer};
use super::semantic_search::rank_embeddings_tag_aware;
use crate::database::{ConversationMessage, StoredEmbeddingWithChunk};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskEchoResponse {
    pub conversation_id: String,
    pub answer: String,
    pub citations: Vec<Citation>,
    pub insufficient_context: bool,
    pub sources_used: usize,
}

/// Core RAG logic executed without holding the database mutex lock:
/// 1. Embeds user question using the embedding engine (Gemini).
/// 2. Performs tag-aware local vector similarity ranking against stored embeddings.
/// 3. Sends prompt + context to selected provider (Gemini or Groq).
/// 4. Returns answer and validated citations.
pub async fn execute_rag_pipeline(
    gemini: &GeminiClient,
    groq: &GroqClient,
    embedding_api_key: &str,
    text_gen_api_key: &str,
    model_config: &ModelConfig,
    conversation_id: &str,
    question: &str,
    stored_embeddings: &[StoredEmbeddingWithChunk],
    conversation_history: &[ConversationMessage],
    prioritized_doc_ids: &HashSet<String>,
) -> Result<AskEchoResponse, String> {
    let trimmed_q = question.trim();

    // 1. Generate query embedding (embeddings are decoupled from text generation)
    let query_embeddings = gemini
        .generate_embeddings(
            embedding_api_key,
            &model_config.embedding_model,
            &[trimmed_q.to_string()],
        )
        .await?;

    let query_vec = match query_embeddings.into_iter().next() {
        Some(v) => v,
        None => return Err("Failed to generate embedding for query".to_string()),
    };

    // 2. Filter stored embeddings to ensure strict compatibility with the active embedding model and dimension
    let active_clean = GeminiClient::clean_model_id(&model_config.embedding_model);
    let compatible_embeddings: Vec<StoredEmbeddingWithChunk> = stored_embeddings
        .iter()
        .filter(|e| {
            let stored_clean = GeminiClient::clean_model_id(&e.model);
            (stored_clean == active_clean || e.model == model_config.embedding_model)
                && e.embedding.len() == query_vec.len()
        })
        .cloned()
        .collect();

    // 3. Rank relevant chunks using compatible embeddings and tag awareness
    let context_chunks = rank_embeddings_tag_aware(
        &compatible_embeddings,
        &query_vec,
        model_config.top_k_retrieval,
        model_config.min_similarity,
        prioritized_doc_ids,
    );

    // 4. Generate grounded answer via selected provider (Gemini or Groq)
    let grounded_answer: GroundedAnswer = dispatch_grounded_answer(
        gemini,
        groq,
        &model_config.provider,
        text_gen_api_key,
        &model_config.generation_model,
        trimmed_q,
        &context_chunks,
        conversation_history,
    )
    .await?;

    Ok(AskEchoResponse {
        conversation_id: conversation_id.to_string(),
        answer: grounded_answer.answer,
        citations: grounded_answer.citations,
        insufficient_context: grounded_answer.insufficient_context,
        sources_used: context_chunks.len(),
    })
}
