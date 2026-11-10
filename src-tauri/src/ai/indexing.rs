use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ai::chunking::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, chunk_text};
use crate::database::{self, DocumentEmbedding, DocumentIndexingStatusRecord, DocumentRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexResult {
    pub document_id: String,
    pub filename: String,
    pub chunks_count: usize,
    pub embeddings_count: usize,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchIndexResult {
    pub total_processed: usize,
    pub successful: Vec<DocumentIndexResult>,
    pub failed: Vec<DocumentIndexResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexStatus {
    pub document_id: String,
    pub filename: String,
    pub file_type: String,
    pub is_indexed: bool,
    pub status: String, // 'indexed', 'pending', 'indexing', 'failed', 'awaiting_key'
    pub error: Option<String>,
    pub chunk_count: usize,
    pub embedding_count: usize,
    pub model: Option<String>,
    pub last_attempt_at: Option<String>,
    pub indexed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingOverview {
    pub total_documents: i64,
    pub indexed_documents: i64,
    pub pending_documents: i64,
    pub currently_processing_documents: i64,
    pub failed_documents: i64,
    pub awaiting_key_documents: i64,
    pub total_chunks: i64,
    pub total_embeddings: i64,
    pub last_successful_indexing_time: Option<String>,
    pub documents: Vec<DocumentIndexStatus>,
}

/// Prepares chunks and text payloads for a document without holding the DB lock
pub fn prepare_chunks(doc: &DocumentRecord) -> Vec<database::DocumentChunk> {
    chunk_text(
        &doc.id,
        &doc.extracted_text,
        DEFAULT_CHUNK_SIZE,
        DEFAULT_CHUNK_OVERLAP,
    )
}

/// Writes prepared chunks, embeddings, and indexing status record into SQLite
pub fn save_index_data(
    conn: &Connection,
    doc: &DocumentRecord,
    model: &str,
    chunks: &[database::DocumentChunk],
    vectors: &[Vec<f32>],
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    database::delete_embeddings_for_document(conn, &doc.id).map_err(|e| e.to_string())?;
    database::delete_chunks_for_document(conn, &doc.id).map_err(|e| e.to_string())?;

    if chunks.is_empty() {
        let status_rec = DocumentIndexingStatusRecord {
            document_id: doc.id.clone(),
            status: "empty".to_string(),
            error_message: None,
            content_hash: doc.content_hash.clone(),
            model: Some(model.to_string()),
            dimensions: None,
            chunks_count: 0,
            embeddings_count: 0,
            last_attempt_at: Some(now.clone()),
            indexed_at: Some(now.clone()),
            attempt_count: 0,
            next_retry_at: None,
            retry_state: "succeeded".to_string(),
            last_successful_attempt_at: Some(now),
            last_error: None,
        };
        database::upsert_indexing_status(conn, &status_rec).map_err(|e| e.to_string())?;
        return Ok(());
    }

    database::insert_chunks(conn, chunks).map_err(|e| e.to_string())?;

    let dimensions = vectors.first().map(|v| v.len());

    for (idx, vec) in vectors.iter().enumerate() {
        let emb = DocumentEmbedding {
            id: Uuid::new_v4().to_string(),
            chunk_id: chunks[idx].id.clone(),
            document_id: doc.id.clone(),
            model: model.to_string(),
            dimensions: vec.len(),
            embedding: vec.clone(),
            created_at: now.clone(),
        };
        database::insert_embedding(conn, &emb).map_err(|e| e.to_string())?;
    }

    let status_rec = DocumentIndexingStatusRecord {
        document_id: doc.id.clone(),
        status: "indexed".to_string(),
        error_message: None,
        content_hash: doc.content_hash.clone(),
        model: Some(model.to_string()),
        dimensions,
        chunks_count: chunks.len(),
        embeddings_count: vectors.len(),
        last_attempt_at: Some(now.clone()),
        indexed_at: Some(now.clone()),
        attempt_count: 0,
        next_retry_at: None,
        retry_state: "succeeded".to_string(),
        last_successful_attempt_at: Some(now),
        last_error: None,
    };
    database::upsert_indexing_status(conn, &status_rec).map_err(|e| e.to_string())?;

    Ok(())
}

/// Retrieves overall indexing metrics and per-document indexing status
pub fn get_indexing_overview(conn: &Connection) -> Result<IndexingOverview, String> {
    let all_docs = database::get_all_documents(conn).map_err(|e| e.to_string())?;
    let counts = database::get_indexing_counts(conn).map_err(|e| e.to_string())?;
    let all_statuses = database::get_all_indexing_statuses(conn).map_err(|e| e.to_string())?;

    let total_chunks = database::get_total_chunks(conn).map_err(|e| e.to_string())?;
    let total_embeddings = database::get_embeddings_count(conn).map_err(|e| e.to_string())?;

    let mut doc_statuses = Vec::with_capacity(all_docs.len());

    for doc in all_docs {
        let status_opt = all_statuses.get(&doc.id);
        let chunks = database::get_chunks_for_document(conn, &doc.id).unwrap_or_default();
        let is_idx = status_opt.map(|s| s.status == "indexed").unwrap_or(false);

        let status_str = status_opt
            .map(|s| s.status.clone())
            .unwrap_or_else(|| "pending".to_string());
        let error = status_opt.and_then(|s| s.error_message.clone());
        let model = status_opt.and_then(|s| s.model.clone());
        let last_attempt_at = status_opt.and_then(|s| s.last_attempt_at.clone());
        let indexed_at = status_opt.and_then(|s| s.indexed_at.clone());

        doc_statuses.push(DocumentIndexStatus {
            document_id: doc.id,
            filename: doc.filename,
            file_type: doc.file_type,
            is_indexed: is_idx,
            status: status_str,
            error,
            chunk_count: chunks.len(),
            embedding_count: if is_idx { chunks.len() } else { 0 },
            model,
            last_attempt_at,
            indexed_at,
        });
    }

    Ok(IndexingOverview {
        total_documents: counts.total,
        indexed_documents: counts.indexed,
        pending_documents: counts.pending,
        currently_processing_documents: counts.indexing,
        failed_documents: counts.failed,
        awaiting_key_documents: counts.awaiting_key,
        total_chunks,
        total_embeddings,
        last_successful_indexing_time: counts.last_indexed_at,
        documents: doc_statuses,
    })
}
