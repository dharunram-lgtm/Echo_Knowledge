use chrono::Utc;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::ai::GeminiClient;
use crate::ai::indexing::{prepare_chunks, save_index_data};
use crate::commands::ai::AIState;
use crate::database::{self, AppStorage, DocumentIndexingStatusRecord, DocumentRecord};
use crate::security;

pub struct IndexingCoordinator {
    pub active_jobs: Arc<Mutex<HashSet<String>>>,
    pub is_running: Arc<AtomicBool>,
    pub active_tagging_jobs: Arc<Mutex<HashSet<String>>>,
    pub is_tagging_running: Arc<AtomicBool>,
    pub is_shutdown: Arc<AtomicBool>,
}

impl IndexingCoordinator {
    pub fn new() -> Self {
        Self {
            active_jobs: Arc::new(Mutex::new(HashSet::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            active_tagging_jobs: Arc::new(Mutex::new(HashSet::new())),
            is_tagging_running: Arc::new(AtomicBool::new(false)),
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Determines whether a document requires indexing or re-indexing
    pub fn check_document_needs_indexing(
        conn: &rusqlite::Connection,
        doc: &DocumentRecord,
        active_embedding_model: &str,
    ) -> bool {
        let status_opt = database::get_indexing_status(conn, &doc.id).unwrap_or(None);

        match status_opt {
            None => true,
            Some(status) => {
                // If previously marked as pending, indexing (interrupted), or failed -> needs indexing
                if status.status == "pending"
                    || status.status == "indexing"
                    || status.status == "failed"
                {
                    return true;
                }

                // If awaiting key, check if key is now configured
                if status.status == "awaiting_key" {
                    return security::has_api_key("gemini").unwrap_or(false);
                }

                // If content hash has changed, document text was updated -> needs re-indexing
                if status.content_hash != doc.content_hash {
                    return true;
                }

                // If model has changed -> needs re-indexing with new model
                let active_clean = GeminiClient::clean_model_id(active_embedding_model);
                let stored_model = status.model.as_deref().unwrap_or("");
                let stored_clean = GeminiClient::clean_model_id(stored_model);
                if stored_clean != active_clean && stored_model != active_embedding_model {
                    return true;
                }

                // If document text exists but zero embeddings are recorded -> needs indexing
                if !doc.extracted_text.trim().is_empty() && status.embeddings_count == 0 {
                    return true;
                }

                // Verify actual embeddings exist in SQLite table
                let has_embeddings = database::is_document_indexed(conn, &doc.id).unwrap_or(false);
                if !doc.extracted_text.trim().is_empty() && !has_embeddings {
                    return true;
                }

                false
            }
        }
    }

    /// Determines whether a document requires AI tagging or re-tagging
    pub fn check_document_needs_tagging(
        conn: &rusqlite::Connection,
        doc: &DocumentRecord,
    ) -> bool {
        let trimmed = doc.extracted_text.trim();
        if trimmed.is_empty() || trimmed.starts_with("[Notice:") {
            return false;
        }

        let status_opt = database::get_tagging_status(conn, &doc.id).unwrap_or(None);
        match status_opt {
            None => true,
            Some(status) => {
                // Re-enqueue if pending, untagged, failed, or previously awaiting key
                if status.status == "pending"
                    || status.status == "untagged"
                    || status.status == "failed"
                    || status.status == "awaiting_key"
                {
                    return true;
                }
                // Re-tag if document content changed
                if status.content_hash != doc.content_hash {
                    return true;
                }
                // Check if document has 0 AI tags despite a "tagged" or other status
                let ai_tags_count = database::get_ai_tags_count_for_document(conn, &doc.id)
                    .unwrap_or(0);
                if ai_tags_count == 0 {
                    return true;
                }
                false
            }
        }
    }
}

/// Spawns startup indexing & tagging inspection and kicks off background processing
pub fn start_startup_indexing(app: &AppHandle) {
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        // Small delay to allow main window to present and UI to render first
        std::thread::sleep(Duration::from_millis(600));

        let storage = match app_handle.try_state::<AppStorage>() {
            Some(s) => s,
            None => return,
        };
        let ai = match app_handle.try_state::<AIState>() {
            Some(a) => a,
            None => return,
        };

        // 1. Check if automatic indexing on startup is enabled (default: true)
        let auto_index_enabled = {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => return,
            };
            database::get_app_setting(&conn, "auto_index_on_startup")
                .unwrap_or(None)
                .map(|val| val != "false")
                .unwrap_or(true)
        };

        let conf = match ai.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => return,
        };

        let embedding_key_available = security::has_api_key("gemini").unwrap_or(false);
        let text_generation_key_available = security::has_api_key(&conf.provider).unwrap_or(false);

        // 2a. Inspect all documents for Indexing
        let has_docs_to_index: bool = {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => return,
            };
            let all_docs = match database::get_all_documents(&conn) {
                Ok(d) => d,
                Err(_) => return,
            };

            let mut count = 0;
            for doc_summary in all_docs {
                if let Ok(Some(full_doc)) = database::get_document_by_id(&conn, &doc_summary.id) {
                    if IndexingCoordinator::check_document_needs_indexing(
                        &conn,
                        &full_doc,
                        &conf.embedding_model,
                    ) {
                        let initial_status = if !embedding_key_available {
                            "awaiting_key"
                        } else {
                            "pending"
                        };

                        let err_msg = if !embedding_key_available {
                            Some("Gemini API key is not configured. Configure Gemini in Settings for embeddings.")
                        } else {
                            None
                        };

                        let now = Utc::now().to_rfc3339();
                        let rec = DocumentIndexingStatusRecord {
                            document_id: full_doc.id.clone(),
                            status: initial_status.to_string(),
                            error_message: err_msg.map(|s| s.to_string()),
                            content_hash: full_doc.content_hash.clone(),
                            model: Some(conf.embedding_model.clone()),
                            dimensions: None,
                            chunks_count: 0,
                            embeddings_count: 0,
                            last_attempt_at: Some(now.clone()),
                            indexed_at: None,
                            attempt_count: 0,
                            next_retry_at: None,
                            retry_state: if !embedding_key_available { "awaiting_key".to_string() } else { "pending".to_string() },
                            last_successful_attempt_at: None,
                            last_error: None,
                        };
                        let _ = database::upsert_indexing_status(&conn, &rec);
                        if embedding_key_available {
                            count += 1;
                        }
                    }
                }
            }
            count > 0
        };

        // 2b. Inspect all documents for AI Tagging & Backfill
        let has_docs_to_tag: bool = {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => return,
            };
            let all_docs = match database::get_all_documents(&conn) {
                Ok(d) => d,
                Err(_) => return,
            };

            let mut count = 0;
            for doc_summary in all_docs {
                if let Ok(Some(full_doc)) = database::get_document_by_id(&conn, &doc_summary.id) {
                    if IndexingCoordinator::check_document_needs_tagging(&conn, &full_doc) {
                        let tag_status = if !text_generation_key_available {
                            "awaiting_key"
                        } else {
                            "pending"
                        };
                        let err_msg = if !text_generation_key_available {
                            Some("Text-generation provider API key is not configured. Configure the selected provider in Settings.")
                        } else {
                            None
                        };
                        let now = Utc::now().to_rfc3339();
                        let tag_rec = database::DocumentTaggingStatusRecord {
                            document_id: full_doc.id.clone(),
                            status: tag_status.to_string(),
                            error_message: err_msg.map(|s| s.to_string()),
                            content_hash: full_doc.content_hash.clone(),
                            tags_count: 0,
                            last_attempt_at: Some(now),
                            tagged_at: None,
                            attempt_count: 0,
                            next_retry_at: None,
                            retry_state: if !text_generation_key_available { "awaiting_key".to_string() } else { "pending".to_string() },
                            last_successful_attempt_at: None,
                            last_error: None,
                        };
                        let _ = database::upsert_tagging_status(&conn, &tag_rec);
                        if text_generation_key_available {
                            count += 1;
                        }
                    }
                }
            }
            count > 0
        };

        // Notify frontend of initial statuses
        let _ = app_handle.emit("indexing:status_changed", ());
        let _ = app_handle.emit("tagging:status_changed", ());

        // Start background supervisor for persistent automatic retries
        start_retry_supervisor(app_handle.clone());

        // 3a. Start background indexing if enabled and key available
        if auto_index_enabled && embedding_key_available && has_docs_to_index {
            let app_idx = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                run_background_indexing_worker(app_idx, Vec::new()).await;
            });
        }

        // 3b. Start background tagging backfill if key available
        if text_generation_key_available && has_docs_to_tag {
            let app_tag = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                run_background_tagging_worker(app_tag).await;
            });
        }
    });
}

/// Enqueues a single document for immediate background indexing (e.g., after document import)
pub fn enqueue_document_for_indexing(app: &AppHandle, doc_id: &str) {
    let app_handle = app.clone();
    let document_id = doc_id.to_string();

    tauri::async_runtime::spawn(async move {
        let storage = match app_handle.try_state::<AppStorage>() {
            Some(s) => s,
            None => return,
        };
        let ai = match app_handle.try_state::<AIState>() {
            Some(a) => a,
            None => return,
        };

        let conf = match ai.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => return,
        };

        let api_key_available = security::has_api_key("gemini").unwrap_or(false);

        // Mark as pending or awaiting_key in DB
        {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => return,
            };
            if let Ok(Some(doc)) = database::get_document_by_id(&conn, &document_id) {
                let status_str = if api_key_available {
                    "pending"
                } else {
                    "awaiting_key"
                };
                let err_msg = if api_key_available {
                    None
                } else {
                    Some("Gemini API key is not configured. Configure Gemini in Settings for embeddings.")
                };
                let now = Utc::now().to_rfc3339();
                let rec = DocumentIndexingStatusRecord {
                    document_id: doc.id.clone(),
                    status: status_str.to_string(),
                    error_message: err_msg.map(|s| s.to_string()),
                    content_hash: doc.content_hash.clone(),
                    model: Some(conf.embedding_model.clone()),
                    dimensions: None,
                    chunks_count: 0,
                    embeddings_count: 0,
                    last_attempt_at: Some(now),
                    indexed_at: None,
                    attempt_count: 0,
                    next_retry_at: None,
                    retry_state: if !api_key_available { "awaiting_key".to_string() } else { "pending".to_string() },
                    last_successful_attempt_at: None,
                    last_error: None,
                };
                let _ = database::upsert_indexing_status(&conn, &rec);
            }
        }

        let _ = app_handle.emit("indexing:status_changed", ());

        if api_key_available {
            run_background_indexing_worker(app_handle, vec![document_id]).await;
        }
    });
}

/// Enqueues a single document for immediate background AI tagging
pub fn enqueue_document_for_tagging(app: &AppHandle, doc_id: &str) {
    let app_handle = app.clone();
    let document_id = doc_id.to_string();

    tauri::async_runtime::spawn(async move {
        let storage = match app_handle.try_state::<AppStorage>() {
            Some(s) => s,
            None => return,
        };
        let ai = match app_handle.try_state::<AIState>() {
            Some(a) => a,
            None => return,
        };

        let conf = match ai.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => return,
        };

        let api_key_available = security::has_api_key(&conf.provider).unwrap_or(false);

        // Mark as pending or awaiting_key in DB
        {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => return,
            };
            if let Ok(Some(doc)) = database::get_document_by_id(&conn, &document_id) {
                let status_str = if api_key_available {
                    "pending"
                } else {
                    "awaiting_key"
                };
                let err_msg = if api_key_available {
                    None
                } else {
                    Some("Selected text-generation provider API key is not configured. Configure it in Settings.")
                };
                let now = Utc::now().to_rfc3339();
                let tag_rec = database::DocumentTaggingStatusRecord {
                    document_id: doc.id.clone(),
                    status: status_str.to_string(),
                    error_message: err_msg.map(|s| s.to_string()),
                    content_hash: doc.content_hash.clone(),
                    tags_count: 0,
                    last_attempt_at: Some(now),
                    tagged_at: None,
                    attempt_count: 0,
                    next_retry_at: None,
                    retry_state: if !api_key_available { "awaiting_key".to_string() } else { "pending".to_string() },
                    last_successful_attempt_at: None,
                    last_error: None,
                };
                let _ = database::upsert_tagging_status(&conn, &tag_rec);
            }
        }

        let _ = app_handle.emit("tagging:status_changed", ());

        if api_key_available {
            run_background_tagging_worker(app_handle).await;
        }
    });
}

/// Core background tagging worker: processes all documents with status = 'pending' in SQLite
pub async fn run_background_tagging_worker(app: AppHandle) {
    let storage = match app.try_state::<AppStorage>() {
        Some(s) => s,
        None => return,
    };
    let ai = match app.try_state::<AIState>() {
        Some(a) => a,
        None => return,
    };
    let coordinator = match app.try_state::<IndexingCoordinator>() {
        Some(c) => c,
        None => return,
    };

    if coordinator.is_tagging_running.swap(true, Ordering::SeqCst) {
        // Already running; running instance will drain all 'pending' records
        return;
    }

    loop {
        if coordinator.is_shutdown.load(Ordering::SeqCst) {
            break;
        }

        // Query the next pending document from SQLite
        let next_doc_id: Option<String> = {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => break,
            };
            conn.query_row(
                "SELECT document_id FROM document_tagging_status WHERE status = 'pending' ORDER BY last_attempt_at ASC LIMIT 1;",
                [],
                |row| row.get(0),
            ).ok()
        };

        let doc_id = match next_doc_id {
            Some(id) => id,
            None => break, // All pending tagging jobs completed
        };

        // Mark active
        {
            let mut active = coordinator.active_tagging_jobs.lock().unwrap();
            active.insert(doc_id.clone());
        }

        let conf = match ai.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => break,
        };

        let api_key = match security::get_api_key(&conf.provider) {
            Ok(Some(k)) if !k.trim().is_empty() => k,
            _ => {
                let conn = storage.conn.lock().unwrap();
                let _ = database::update_tagging_status(
                    &conn,
                    &doc_id,
                    "awaiting_key",
                    Some("Selected text-generation provider API key is not configured. Configure it in Settings."),
                    0,
                );
                coordinator.active_tagging_jobs.lock().unwrap().remove(&doc_id);
                let _ = app.emit("tagging:status_changed", ());
                break;
            }
        };

        // Transition status to 'tagging'
        {
            let conn = storage.conn.lock().unwrap();
            let _ = database::update_tagging_status(&conn, &doc_id, "tagging", None, 0);
        }
        let _ = app.emit("tagging:status_changed", ());

        let doc_opt = {
            let conn = storage.conn.lock().unwrap();
            database::get_document_by_id(&conn, &doc_id).unwrap_or(None)
        };

        let doc = match doc_opt {
            Some(d) => d,
            None => {
                coordinator.active_tagging_jobs.lock().unwrap().remove(&doc_id);
                continue;
            }
        };

        let trimmed_text = doc.extracted_text.trim();
        if trimmed_text.is_empty() || trimmed_text.starts_with("[Notice:") {
            let conn = storage.conn.lock().unwrap();
            let _ = database::update_tagging_status(&conn, &doc_id, "tagged", None, 0);
            coordinator.active_tagging_jobs.lock().unwrap().remove(&doc_id);
            let _ = app.emit("tagging:status_changed", ());
            continue;
        }

        let tagging_result = crate::ai::provider::dispatch_extract_tags(
            &ai.gemini,
            &ai.groq,
            &conf.provider,
            &api_key,
            &conf.generation_model,
            &doc.filename,
            &doc.extracted_text,
        )
        .await;

        match tagging_result {
            Ok(tags) => {
                {
                    let conn = storage.conn.lock().unwrap();
                    let tags_count = tags.len();
                    let _ = crate::ai::tagging::save_extracted_tags(&conn, &doc, &tags);
                    let _ = database::update_tagging_retry_state(
                        &conn,
                        &doc_id,
                        "succeeded",
                        "tagged",
                        0,
                        None,
                        None,
                        tags_count,
                    );
                }
                coordinator.active_tagging_jobs.lock().unwrap().remove(&doc_id);
                let _ = app.emit("tagging:status_changed", ());

                // Auto-discover relationships for this document if concepts exist
                let app_rel = app.clone();
                let doc_id_c = doc.id.clone();
                let doc_filename_c = doc.filename.clone();
                let doc_text_c = doc.extracted_text.clone();
                let api_key_c = api_key.clone();
                let gen_model_c = conf.generation_model.clone();
                let gemini_client = ai.gemini.clone();
                let groq_client = ai.groq.clone();
                let provider_c = conf.provider.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(st) = app_rel.try_state::<AppStorage>() {
                        let concepts_opt = {
                            if let Ok(conn) = st.conn.lock() {
                                database::get_concepts_for_document(&conn, &doc_id_c).ok()
                            } else {
                                None
                            }
                        };

                        if let Some(concepts) = concepts_opt {
                            if concepts.len() >= 2 {
                                if let Ok(raw_rels) = crate::ai::provider::dispatch_extract_relationships(
                                    &gemini_client,
                                    &groq_client,
                                    &provider_c,
                                    &api_key_c,
                                    &gen_model_c,
                                    &doc_filename_c,
                                    &doc_text_c,
                                    &concepts,
                                ).await {
                                    if !raw_rels.is_empty() {
                                        if let Ok(conn) = st.conn.lock() {
                                            let _ = crate::ai::graph::save_discovered_relationships(
                                                &conn,
                                                &doc_id_c,
                                                &doc_filename_c,
                                                &concepts,
                                                &raw_rels,
                                            );
                                        }
                                        let _ = app_rel.emit("relationships:changed", ());
                                    }
                                }
                            }
                        }
                    }
                });

                // Bounded rate: short pause between documents
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(err_msg) => {
                let (category, retry_after) = crate::ai::retry::classify_error(None, &err_msg);
                let sanitized = if err_msg.contains("429") || err_msg.to_lowercase().contains("quota") {
                    "API rate limit or quota exceeded (429). Tagging will retry shortly.".to_string()
                } else if err_msg.to_lowercase().contains("network")
                    || err_msg.to_lowercase().contains("dns")
                    || err_msg.to_lowercase().contains("connection")
                {
                    "Network connection offline or unreachable. Will retry when connected.".to_string()
                } else {
                    err_msg
                };

                let conn = storage.conn.lock().unwrap();
                let current_status = database::get_tagging_status(&conn, &doc_id).ok().flatten();
                let attempts = current_status.map(|s| s.attempt_count).unwrap_or(0) + 1;

                if category == crate::ai::retry::ErrorCategory::Permanent {
                    let _ = database::update_tagging_retry_state(
                        &conn,
                        &doc_id,
                        "permanently_failed",
                        "failed",
                        attempts,
                        None,
                        Some(&sanitized),
                        0,
                    );
                } else {
                    let (next_retry, _sec) = crate::ai::retry::calculate_next_retry(
                        attempts,
                        &category,
                        retry_after,
                    );
                    let _ = database::update_tagging_retry_state(
                        &conn,
                        &doc_id,
                        "retry_wait",
                        "failed",
                        attempts,
                        Some(&next_retry),
                        Some(&sanitized),
                        0,
                    );
                }

                coordinator.active_tagging_jobs.lock().unwrap().remove(&doc_id);
                let _ = app.emit("tagging:status_changed", ());

                if category == crate::ai::retry::ErrorCategory::RateLimit {
                    std::thread::sleep(Duration::from_secs(3));
                }
            }
        }
    }

    coordinator.is_tagging_running.store(false, Ordering::SeqCst);
    let _ = app.emit("tagging:status_changed", ());
}

/// Core background indexing worker with bounded concurrency, rate limit backoff, and deduplication
pub async fn run_background_indexing_worker(app: AppHandle, _initial_doc_ids: Vec<String>) {
    let storage = match app.try_state::<AppStorage>() {
        Some(s) => s,
        None => return,
    };
    let ai = match app.try_state::<AIState>() {
        Some(a) => a,
        None => return,
    };

    let coordinator = match app.try_state::<IndexingCoordinator>() {
        Some(c) => c,
        None => return,
    };

    if coordinator.is_running.swap(true, Ordering::SeqCst) {
        // Already running worker, items in pending status will be drained in loop
        return;
    }

    loop {
        if coordinator.is_shutdown.load(Ordering::SeqCst) {
            break;
        }

        // Query next pending document from SQLite
        let next_doc_id: Option<String> = {
            let conn = match storage.conn.lock() {
                Ok(c) => c,
                Err(_) => break,
            };
            conn.query_row(
                "SELECT document_id FROM document_indexing_status WHERE status = 'pending' ORDER BY last_attempt_at ASC LIMIT 1;",
                [],
                |row| row.get(0),
            ).ok()
        };

        let doc_id = match next_doc_id {
            Some(id) => id,
            None => break, // All pending indexing jobs completed
        };

        // Deduplication check: avoid duplicate jobs for same document
        {
            let mut active = coordinator.active_jobs.lock().unwrap();
            if active.contains(&doc_id) {
                continue;
            }
            active.insert(doc_id.clone());
        }

        let conf = match ai.config.lock() {
            Ok(c) => c.clone(),
            Err(_) => break,
        };

        let api_key = match security::get_api_key(&conf.provider) {
            Ok(Some(k)) if !k.trim().is_empty() => k,
            _ => {
                // Key was removed or missing
                let conn = storage.conn.lock().unwrap();
                let _ = database::update_status(
                    &conn,
                    &doc_id,
                    "awaiting_key",
                    Some("Gemini API key is not configured. Add your key in Settings."),
                );
                coordinator.active_jobs.lock().unwrap().remove(&doc_id);
                let _ = app.emit("indexing:status_changed", ());
                break;
            }
        };

        // 1. Set status to 'indexing'
        {
            let conn = storage.conn.lock().unwrap();
            let _ = database::update_status(&conn, &doc_id, "indexing", None);
        }
        let _ = app.emit("indexing:status_changed", ());

        // 2. Fetch doc and prepare chunks (brief DB lock)
        let (doc_opt, chunks) = {
            let conn = storage.conn.lock().unwrap();
            let d_opt = database::get_document_by_id(&conn, &doc_id).unwrap_or(None);
            let c = match &d_opt {
                Some(d) => prepare_chunks(d),
                None => Vec::new(),
            };
            (d_opt, c)
        };

        let doc = match doc_opt {
            Some(d) => d,
            None => {
                coordinator.active_jobs.lock().unwrap().remove(&doc_id);
                continue;
            }
        };

        if chunks.is_empty() {
            let conn = storage.conn.lock().unwrap();
            let _ = save_index_data(&conn, &doc, &conf.embedding_model, &[], &[]);
            coordinator.active_jobs.lock().unwrap().remove(&doc_id);
            let _ = app.emit("indexing:status_changed", ());
            continue;
        }

        // 3. Generate embeddings over network (NO DB lock held)
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.chunk_text.clone()).collect();
        let embedding_result: std::result::Result<Vec<Vec<f32>>, String> = ai
            .gemini
            .generate_embeddings(&api_key, &conf.embedding_model, &chunk_texts)
            .await;

        match embedding_result {
            Ok(vectors) => {
                // 4. Save index data into SQLite
                {
                    let conn = storage.conn.lock().unwrap();
                    let _ = save_index_data(&conn, &doc, &conf.embedding_model, &chunks, &vectors);
                    let _ = database::update_indexing_retry_state(
                        &conn,
                        &doc_id,
                        "succeeded",
                        "indexed",
                        0,
                        None,
                        None,
                    );
                }
                coordinator.active_jobs.lock().unwrap().remove(&doc_id);
                let _ = app.emit("indexing:status_changed", ());

                // Bounded rate: short pause between documents
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(err_msg) => {
                let (category, retry_after) = crate::ai::retry::classify_error(None, &err_msg);
                let sanitized =
                    if err_msg.contains("429") || err_msg.to_lowercase().contains("quota") {
                        "API rate limit or quota exceeded (429). Indexing will retry shortly."
                            .to_string()
                    } else if err_msg.to_lowercase().contains("network")
                        || err_msg.to_lowercase().contains("dns")
                    {
                        "Network connection offline or unreachable. Will retry when connected."
                            .to_string()
                    } else {
                        err_msg
                    };

                let conn = storage.conn.lock().unwrap();
                let current_status = database::get_indexing_status(&conn, &doc_id).ok().flatten();
                let attempts = current_status.map(|s| s.attempt_count).unwrap_or(0) + 1;

                if category == crate::ai::retry::ErrorCategory::Permanent {
                    let _ = database::update_indexing_retry_state(
                        &conn,
                        &doc_id,
                        "permanently_failed",
                        "failed",
                        attempts,
                        None,
                        Some(&sanitized),
                    );
                } else {
                    let (next_retry, _sec) = crate::ai::retry::calculate_next_retry(
                        attempts,
                        &category,
                        retry_after,
                    );
                    let _ = database::update_indexing_retry_state(
                        &conn,
                        &doc_id,
                        "retry_wait",
                        "failed",
                        attempts,
                        Some(&next_retry),
                        Some(&sanitized),
                    );
                }

                coordinator.active_jobs.lock().unwrap().remove(&doc_id);
                let _ = app.emit("indexing:status_changed", ());

                if category == crate::ai::retry::ErrorCategory::RateLimit {
                    std::thread::sleep(Duration::from_secs(4));
                }
            }
        }
    }

    coordinator.is_running.store(false, Ordering::SeqCst);
    let _ = app.emit("indexing:status_changed", ());
}

/// Periodic supervisor that triggers pending or due retries
pub fn start_retry_supervisor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Sleep 5 seconds between retry checks
            std::thread::sleep(Duration::from_secs(5));

            let storage = match app.try_state::<AppStorage>() {
                Some(s) => s,
                None => continue,
            };
            let coordinator = match app.try_state::<IndexingCoordinator>() {
                Some(c) => c,
                None => continue,
            };

            if coordinator.is_shutdown.load(Ordering::SeqCst) {
                break;
            }

            let is_paused = {
                let conn = match storage.conn.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                database::get_app_setting(&conn, "retries_paused")
                    .unwrap_or(None)
                    .map(|v| v == "true")
                    .unwrap_or(false)
            };

            if is_paused {
                continue;
            }

            let now = Utc::now().to_rfc3339();

            // 1. Check due indexing retries
            let mut trigger_indexing = false;
            {
                let conn = match storage.conn.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Ok(due_ids) = database::get_due_indexing_retries(&conn, &now) {
                    for id in due_ids {
                        let _ = conn.execute(
                            "UPDATE document_indexing_status SET retry_state = 'pending', status = 'pending', next_retry_at = NULL WHERE document_id = ?1;",
                            rusqlite::params![id],
                        );
                        trigger_indexing = true;
                    }
                }
            }

            // 2. Check due tagging retries
            let mut trigger_tagging = false;
            {
                let conn = match storage.conn.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Ok(due_ids) = database::get_due_tagging_retries(&conn, &now) {
                    for id in due_ids {
                        let _ = conn.execute(
                            "UPDATE document_tagging_status SET retry_state = 'pending', status = 'pending', next_retry_at = NULL WHERE document_id = ?1;",
                            rusqlite::params![id],
                        );
                        trigger_tagging = true;
                    }
                }
            }

            if trigger_indexing {
                let _ = app.emit("indexing:status_changed", ());
                let app_idx = app.clone();
                tauri::async_runtime::spawn(async move {
                    run_background_indexing_worker(app_idx, Vec::new()).await;
                });
            }

            if trigger_tagging {
                let _ = app.emit("tagging:status_changed", ());
                let app_tag = app.clone();
                tauri::async_runtime::spawn(async move {
                    run_background_tagging_worker(app_tag).await;
                });
            }
        }
    });
}

/// Global Retry Overview helper
pub fn get_retry_overview(conn: &rusqlite::Connection) -> rusqlite::Result<crate::ai::retry::RetryOverview> {
    let is_globally_paused = database::get_app_setting(conn, "retries_paused")
        .unwrap_or(None)
        .map(|v| v == "true")
        .unwrap_or(false);

    let count_state = |table: &str, state: &str| -> usize {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM {} WHERE retry_state = ?1;", table),
            rusqlite::params![state],
            |r| r.get(0),
        ).unwrap_or(0)
    };

    Ok(crate::ai::retry::RetryOverview {
        is_globally_paused,
        pending_indexing_count: count_state("document_indexing_status", "pending") + count_state("document_indexing_status", "processing"),
        retry_wait_indexing_count: count_state("document_indexing_status", "retry_wait"),
        succeeded_indexing_count: count_state("document_indexing_status", "succeeded"),
        failed_indexing_count: count_state("document_indexing_status", "permanently_failed"),
        pending_tagging_count: count_state("document_tagging_status", "pending") + count_state("document_tagging_status", "processing"),
        retry_wait_tagging_count: count_state("document_tagging_status", "retry_wait"),
        succeeded_tagging_count: count_state("document_tagging_status", "succeeded"),
        failed_tagging_count: count_state("document_tagging_status", "permanently_failed"),
    })
}
