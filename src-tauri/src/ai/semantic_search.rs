use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::provider::ContextChunk;
use super::vector::cosine_similarity;
use crate::database::StoredEmbeddingWithChunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub document_id: String,
    pub filename: String,
    pub file_type: String,
    pub chunk_id: String,
    pub chunk_index: usize,
    pub chunk_text: String,
    pub similarity: f32,
    pub snippet: String,
}

/// Executes cosine-similarity ranking over stored embeddings using the query vector
pub fn rank_embeddings(
    stored_embeddings: &[StoredEmbeddingWithChunk],
    query_vector: &[f32],
    top_k: usize,
    min_similarity: f32,
) -> Vec<ContextChunk> {
    let empty_set = HashSet::new();
    rank_embeddings_tag_aware(
        stored_embeddings,
        query_vector,
        top_k,
        min_similarity,
        &empty_set,
    )
}

/// Executes tag-aware cosine-similarity ranking over stored embeddings with relevance boost for tagged candidate documents
pub fn rank_embeddings_tag_aware(
    stored_embeddings: &[StoredEmbeddingWithChunk],
    query_vector: &[f32],
    top_k: usize,
    min_similarity: f32,
    prioritized_doc_ids: &HashSet<String>,
) -> Vec<ContextChunk> {
    let mut scored: Vec<(f32, &StoredEmbeddingWithChunk)> = stored_embeddings
        .iter()
        .map(|item| {
            let mut sim = cosine_similarity(&item.embedding, query_vector);
            // If this chunk belongs to a document matching a query tag or concept, apply a 15% relevance boost
            if prioritized_doc_ids.contains(&item.document_id) {
                sim = (sim * 1.15).min(1.0);
            }
            (sim, item)
        })
        .filter(|(sim, _)| *sim >= min_similarity)
        .collect();

    // Sort descending by similarity
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .take(top_k)
        .map(|(sim, item)| ContextChunk {
            document_id: item.document_id.clone(),
            filename: item.filename.clone(),
            file_type: item.file_type.clone(),
            chunk_id: item.chunk_id.clone(),
            chunk_index: item.chunk_index,
            chunk_text: item.chunk_text.clone(),
            similarity: sim,
        })
        .collect()
}

/// Formats a semantic search result for client presentation
pub fn context_to_search_results(chunks: Vec<ContextChunk>) -> Vec<SemanticSearchResult> {
    chunks
        .into_iter()
        .map(|c| {
            let snippet = if c.chunk_text.chars().count() > 240 {
                let s: String = c.chunk_text.chars().take(240).collect();
                format!("{}...", s.trim())
            } else {
                c.chunk_text.clone()
            };

            SemanticSearchResult {
                document_id: c.document_id,
                filename: c.filename,
                file_type: c.file_type,
                chunk_id: c.chunk_id,
                chunk_index: c.chunk_index,
                chunk_text: c.chunk_text,
                similarity: (c.similarity * 100.0).round() / 100.0,
                snippet,
            }
        })
        .collect()
}
