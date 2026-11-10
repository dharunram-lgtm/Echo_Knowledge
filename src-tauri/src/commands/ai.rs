use std::sync::Mutex;
use tauri::State;

use crate::ai::{
    self, BatchIndexResult, DiscoveredModel, DocumentIndexResult, EmbeddingCompatibility,
    GeminiClient, GroqClient, IndexingOverview, ModelConfig, SUPPORTED_EMBEDDING_MODELS,
    SUPPORTED_GENERATION_MODELS, SUPPORTED_GROQ_MODELS, SafeProviderStatus, SemanticSearchResult,
};
use crate::database::{self, AppStorage, Conversation, ConversationMessage};
use crate::security;

pub struct AIState {
    pub config: Mutex<ModelConfig>,
    pub gemini: GeminiClient,
    pub groq: GroqClient,
}

impl AIState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ModelConfig::default()),
            gemini: GeminiClient::new(),
            groq: GroqClient::new(),
        }
    }
}

#[tauri::command]
pub async fn get_ai_status(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
) -> Result<SafeProviderStatus, String> {
    // 1. Sync config with SQLite app_settings
    let mut conf = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        if let Ok(Some(saved_json)) = database::get_app_setting(&conn, "model_config") {
            if let Ok(saved_conf) = serde_json::from_str::<ModelConfig>(&saved_json) {
                let mut c = ai.config.lock().map_err(|e| e.to_string())?;
                *c = saved_conf;
                c.clone()
            } else {
                ai.config.lock().map_err(|e| e.to_string())?.clone()
            }
        } else {
            ai.config.lock().map_err(|e| e.to_string())?.clone()
        }
    };

    // Auto-migrate obsolete gemini-1.5-flash or retired models
    let mut config_dirty = false;
    if conf.generation_model == "gemini-1.5-flash" || conf.generation_model == "gemini-2.0-flash" {
        conf.generation_model = "gemini-3.5-flash".to_string();
        config_dirty = true;
    }
    if conf.embedding_model == "text-embedding-004" {
        conf.embedding_model = "gemini-embedding-001".to_string();
        config_dirty = true;
    }
    if conf.provider.eq_ignore_ascii_case("groq")
        && conf.generation_model.starts_with("gemini")
    {
        conf.generation_model = crate::ai::DEFAULT_GROQ_GENERATION_MODEL.to_string();
        config_dirty = true;
    }
    if conf.provider.eq_ignore_ascii_case("gemini")
        && !conf.generation_model.starts_with("gemini")
    {
        conf.generation_model = crate::ai::DEFAULT_GENERATION_MODEL.to_string();
        config_dirty = true;
    }
    if config_dirty {
        let mut c = ai.config.lock().map_err(|e| e.to_string())?;
        *c = conf.clone();
        if let Ok(json_str) = serde_json::to_string(&conf) {
            let conn = storage.conn.lock().map_err(|e| e.to_string())?;
            let _ = database::set_app_setting(&conn, "model_config", &json_str);
        }
    }

    let is_configured = security::has_api_key(&conf.provider).unwrap_or(false);

    // Discover models using the configured text provider. Embeddings remain Gemini-backed.
    let mut discovered_models = Vec::new();
    let mut last_error = None;

    if is_configured {
        if let Ok(Some(key)) = security::get_api_key(&conf.provider) {
            let result = match conf.provider.trim().to_lowercase().as_str() {
                "groq" => ai.groq.list_models(&key).await,
                _ => ai.gemini.list_models(&key).await,
            };
            match result {
                Ok(models) => {
                    discovered_models = models;
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
    }

    // Dynamic model lists based on discovery or fallback to verified supported lists
    let dynamic_gen_models: Vec<String> = if !discovered_models.is_empty() {
        discovered_models
            .iter()
            .filter(|m| m.supports_generation)
            .map(|m| m.id.clone())
            .collect()
    } else {
        match conf.provider.trim().to_lowercase().as_str() {
            "groq" => SUPPORTED_GROQ_MODELS.iter().map(|s| s.to_string()).collect(),
            _ => SUPPORTED_GENERATION_MODELS.iter().map(|s| s.to_string()).collect(),
        }
    };

    let dynamic_emb_models: Vec<String> = if !discovered_models.is_empty() {
        discovered_models
            .iter()
            .filter(|m| m.supports_embedding)
            .map(|m| m.id.clone())
            .collect()
    } else {
        // Groq intentionally has no embedding API; Gemini remains the embedding engine.
        SUPPORTED_EMBEDDING_MODELS
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    Ok(SafeProviderStatus {
        provider: conf.provider,
        is_configured,
        active_generation_model: conf.generation_model,
        active_embedding_model: conf.embedding_model,
        supported_generation_models: dynamic_gen_models,
        supported_embedding_models: dynamic_emb_models,
        supported_providers: vec![
            "gemini".to_string(),
            "groq".to_string(),
            "openai (planned)".to_string(),
            "ollama (planned)".to_string(),
        ],
        discovered_models,
        last_error,
    })
}

#[tauri::command]
pub fn save_api_key(provider: String, api_key: String) -> Result<(), String> {
    security::store_api_key(&provider, &api_key)
}

#[tauri::command]
pub fn remove_api_key(provider: String) -> Result<bool, String> {
    security::delete_api_key(&provider)
}

#[tauri::command]
pub async fn test_ai_connection(
    ai: State<'_, AIState>,
    provider: String,
    api_key: Option<String>,
) -> Result<bool, String> {
    let key_to_test = match api_key.filter(|k| !k.trim().is_empty()) {
        Some(k) => k,
        None => security::get_api_key(&provider)?
            .ok_or_else(|| "No API key configured for this provider".to_string())?,
    };

    match provider.to_lowercase().as_str() {
        "gemini" => ai.gemini.test_connection(&key_to_test).await,
        "groq" => ai.groq.test_connection(&key_to_test).await,
        "openai" => Err("OpenAI provider integration is planned for a future update".to_string()),
        "ollama" => {
            Err("Ollama local provider integration is planned for a future update".to_string())
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn discover_models(
    ai: State<'_, AIState>,
    provider: Option<String>,
) -> Result<Vec<DiscoveredModel>, String> {
    let prov = provider.unwrap_or_else(|| "gemini".to_string());
    let api_key = security::get_api_key(&prov)?.ok_or_else(|| {
        format!("{} API key is not configured. Please save your API key in Settings before discovering models.", prov.to_uppercase())
    })?;

    match prov.trim().to_lowercase().as_str() {
        "groq" => ai.groq.list_models(&api_key).await,
        "gemini" => ai.gemini.list_models(&api_key).await,
        _ => Err(format!("Unsupported provider: {}", prov)),
    }
}

#[tauri::command]
pub fn check_embedding_compatibility(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
) -> Result<EmbeddingCompatibility, String> {
    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    let all_embeddings = database::get_all_embeddings(&conn).map_err(|e| e.to_string())?;
    let total_embeddings = all_embeddings.len();

    let active_clean = GeminiClient::clean_model_id(&conf.embedding_model);
    let mut stored_models: Vec<String> = all_embeddings
        .iter()
        .map(|e| e.model.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    stored_models.sort();

    let mismatched_count = all_embeddings
        .iter()
        .filter(|e| {
            let stored_clean = GeminiClient::clean_model_id(&e.model);
            stored_clean != active_clean && e.model != conf.embedding_model
        })
        .count();

    let is_compatible = mismatched_count == 0;
    let message = if is_compatible {
        if total_embeddings == 0 {
            "No document embeddings generated yet.".to_string()
        } else {
            format!(
                "All {} stored embeddings match active model '{}'.",
                total_embeddings, conf.embedding_model
            )
        }
    } else {
        format!(
            "Embedding model mismatch: {} of {} stored embeddings were generated using other models ({}). Switching embedding models requires re-indexing documents to ensure search accuracy.",
            mismatched_count,
            total_embeddings,
            stored_models.join(", ")
        )
    };

    Ok(EmbeddingCompatibility {
        is_compatible,
        active_model: conf.embedding_model,
        stored_models,
        total_embeddings,
        mismatched_count,
        message,
    })
}

#[tauri::command]
pub fn get_model_config(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
) -> Result<ModelConfig, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    if let Ok(Some(saved_json)) = database::get_app_setting(&conn, "model_config") {
        if let Ok(mut saved_conf) = serde_json::from_str::<ModelConfig>(&saved_json) {
            if saved_conf.generation_model == "gemini-1.5-flash"
                || saved_conf.generation_model == "gemini-2.0-flash"
            {
                saved_conf.generation_model = "gemini-3.5-flash".to_string();
            }
            if saved_conf.embedding_model == "text-embedding-004" {
                saved_conf.embedding_model = "gemini-embedding-001".to_string();
            }
            let mut conf = ai.config.lock().map_err(|e| e.to_string())?;
            *conf = saved_conf.clone();
            return Ok(saved_conf);
        }
    }

    let mut conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    if conf.generation_model == "gemini-1.5-flash" || conf.generation_model == "gemini-2.0-flash" {
        conf.generation_model = "gemini-3.5-flash".to_string();
    }
    if conf.embedding_model == "text-embedding-004" {
        conf.embedding_model = "gemini-embedding-001".to_string();
    }
    Ok(conf)
}

#[tauri::command]
pub fn update_model_config(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
    config: ModelConfig,
) -> Result<(), String> {
    {
        let mut conf = ai.config.lock().map_err(|e| e.to_string())?;
        *conf = config.clone();
    }

    let json_str = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "model_config", &json_str).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn index_document(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
    document_id: String,
) -> Result<DocumentIndexResult, String> {
    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let api_key = security::get_api_key("gemini")?.ok_or_else(|| {
        "Gemini API key is not configured. Configure Gemini for embeddings in Settings.".to_string()
    })?;

    // 1. Fetch document and prepare chunks without holding lock across network calls
    let (doc, chunks) = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let d = database::get_document_by_id(&conn, &document_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Document not found: {}", document_id))?;
        let c = ai::indexing::prepare_chunks(&d);
        (d, c)
    };

    if chunks.is_empty() {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        ai::indexing::save_index_data(&conn, &doc, &conf.embedding_model, &[], &[])?;
        return Ok(DocumentIndexResult {
            document_id: doc.id,
            filename: doc.filename,
            chunks_count: 0,
            embeddings_count: 0,
            status: "empty".to_string(),
            error: None,
        });
    }

    // 2. Generate embeddings via network (no DB lock held)
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.chunk_text.clone()).collect();
    let vectors = match ai
        .gemini
        .generate_embeddings(&api_key, &conf.embedding_model, &chunk_texts)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            let conn = storage.conn.lock().map_err(|err| err.to_string())?;
            let _ = database::update_status(&conn, &doc.id, "failed", Some(&e));
            return Ok(DocumentIndexResult {
                document_id: doc.id,
                filename: doc.filename,
                chunks_count: chunks.len(),
                embeddings_count: 0,
                status: "failed".to_string(),
                error: Some(e),
            });
        }
    };

    // 3. Save chunks and embeddings into SQLite
    {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        ai::indexing::save_index_data(&conn, &doc, &conf.embedding_model, &chunks, &vectors)?;
    }

    Ok(DocumentIndexResult {
        document_id: doc.id,
        filename: doc.filename,
        chunks_count: chunks.len(),
        embeddings_count: vectors.len(),
        status: "indexed".to_string(),
        error: None,
    })
}

#[tauri::command]
pub async fn index_all_documents(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
) -> Result<BatchIndexResult, String> {
    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let api_key = security::get_api_key("gemini")?.ok_or_else(|| {
        "Gemini API key is not configured. Configure Gemini for embeddings in Settings.".to_string()
    })?;

    // Fetch documents
    let docs = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let summaries = database::get_all_documents(&conn).map_err(|e| e.to_string())?;
        let mut full_docs = Vec::new();
        for s in summaries {
            if let Some(doc) =
                database::get_document_by_id(&conn, &s.id).map_err(|e| e.to_string())?
            {
                full_docs.push(doc);
            }
        }
        full_docs
    };

    let total_processed = docs.len();
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for doc in docs {
        let chunks = ai::indexing::prepare_chunks(&doc);

        if chunks.is_empty() {
            let conn = storage.conn.lock().map_err(|e| e.to_string())?;
            let _ = ai::indexing::save_index_data(&conn, &doc, &conf.embedding_model, &[], &[]);
            successful.push(DocumentIndexResult {
                document_id: doc.id,
                filename: doc.filename,
                chunks_count: 0,
                embeddings_count: 0,
                status: "empty".to_string(),
                error: None,
            });
            continue;
        }

        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.chunk_text.clone()).collect();
        match ai
            .gemini
            .generate_embeddings(&api_key, &conf.embedding_model, &chunk_texts)
            .await
        {
            Ok(vectors) => {
                let conn = storage.conn.lock().map_err(|e| e.to_string())?;
                let _ = ai::indexing::save_index_data(
                    &conn,
                    &doc,
                    &conf.embedding_model,
                    &chunks,
                    &vectors,
                );
                successful.push(DocumentIndexResult {
                    document_id: doc.id,
                    filename: doc.filename,
                    chunks_count: chunks.len(),
                    embeddings_count: vectors.len(),
                    status: "indexed".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                failed.push(DocumentIndexResult {
                    document_id: doc.id,
                    filename: doc.filename,
                    chunks_count: chunks.len(),
                    embeddings_count: 0,
                    status: "failed".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    Ok(BatchIndexResult {
        total_processed,
        successful,
        failed,
    })
}

#[tauri::command]
pub fn get_indexing_status(storage: State<AppStorage>) -> Result<IndexingOverview, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    ai::indexing::get_indexing_overview(&conn)
}

#[tauri::command]
pub fn get_auto_index_setting(storage: State<AppStorage>) -> Result<bool, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let val =
        database::get_app_setting(&conn, "auto_index_on_startup").map_err(|e| e.to_string())?;
    Ok(val.map(|v| v != "false").unwrap_or(true))
}

#[tauri::command]
pub fn set_auto_index_setting(storage: State<AppStorage>, enabled: bool) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let val_str = if enabled { "true" } else { "false" };
    database::set_app_setting(&conn, "auto_index_on_startup", val_str)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn retry_failed_indexing(
    app: tauri::AppHandle,
    storage: State<AppStorage>,
) -> Result<usize, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT document_id FROM document_indexing_status WHERE status IN ('failed', 'awaiting_key', 'pending');")
        .map_err(|e| e.to_string())?;

    let doc_ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let count = doc_ids.len();
    drop(stmt);
    drop(conn);

    if !doc_ids.is_empty() {
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            crate::ai::coordinator::run_background_indexing_worker(app_clone, doc_ids).await;
        });
    }

    Ok(count)
}

#[tauri::command]
pub async fn semantic_search(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SemanticSearchResult>, String> {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let api_key = security::get_api_key("gemini")?.ok_or_else(|| {
        "Gemini API key is not configured. Configure Gemini for embeddings in Settings.".to_string()
    })?;

    let top_k = limit.unwrap_or(conf.top_k_retrieval);

    // 1. Generate query embedding (no DB lock held)
    let query_vecs = ai
        .gemini
        .generate_embeddings(&api_key, &conf.embedding_model, &[trimmed.clone()])
        .await?;

    let query_vec = match query_vecs.into_iter().next() {
        Some(v) => v,
        None => return Err("Failed to generate query embedding".to_string()),
    };

    // 2. Fetch all embeddings briefly
    let stored = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::get_all_embeddings(&conn).map_err(|e| e.to_string())?
    };

    // Filter to ensure only compatible embeddings of matching model/dimensions are ranked
    let active_clean = GeminiClient::clean_model_id(&conf.embedding_model);
    let compatible_stored: Vec<_> = stored
        .into_iter()
        .filter(|e| {
            let stored_clean = GeminiClient::clean_model_id(&e.model);
            (stored_clean == active_clean || e.model == conf.embedding_model)
                && e.embedding.len() == query_vec.len()
        })
        .collect();

    // Extract candidate documents matching query tags/concepts
    let prioritized_doc_ids = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let mut set = std::collections::HashSet::new();
        let lower_q = trimmed.to_lowercase();
        if let Ok(tags) = database::get_all_tags(&conn) {
            for t in tags {
                if lower_q.contains(&t.normalized_name) || lower_q.contains(&t.name.to_lowercase()) {
                    if let Ok(docs) = database::get_documents_by_tag(&conn, &t.name) {
                        set.extend(docs);
                    }
                }
            }
        }
        set
    };

    let ranked = ai::semantic_search::rank_embeddings_tag_aware(
        &compatible_stored,
        &query_vec,
        top_k,
        conf.min_similarity,
        &prioritized_doc_ids,
    );
    Ok(ai::semantic_search::context_to_search_results(ranked))
}

#[tauri::command]
pub async fn ask_echo(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
    conversation_id: Option<String>,
    question: String,
) -> Result<ai::AskEchoResponse, String> {
    let trimmed_q = question.trim().to_string();
    if trimmed_q.is_empty() {
        return Err("Question cannot be empty".to_string());
    }

    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let embedding_api_key = security::get_api_key("gemini")?.ok_or_else(|| {
        "Gemini API key is not configured. Configure Gemini for embeddings in Settings.".to_string()
    })?;
    let text_gen_api_key = security::get_api_key(&conf.provider)?.ok_or_else(|| {
        format!("{} API key is not configured. Please set it in Settings.", conf.provider.to_uppercase())
    })?;

    // 1. Ensure conversation exists and persist user message (brief DB lock)
    let (active_conv_id, stored_embeddings, history, prioritized_doc_ids) = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;

        let conv_id = match conversation_id {
            Some(ref id) if !id.trim().is_empty() => {
                if database::get_conversation(&conn, id)
                    .map_err(|e| e.to_string())?
                    .is_none()
                {
                    let title = if trimmed_q.len() > 36 {
                        format!("{}...", &trimmed_q[..36])
                    } else {
                        trimmed_q.clone()
                    };
                    let c = database::create_conversation(&conn, Some(&title))
                        .map_err(|e| e.to_string())?;
                    c.id
                } else {
                    id.clone()
                }
            }
            _ => {
                let title = if trimmed_q.len() > 36 {
                    format!("{}...", &trimmed_q[..36])
                } else {
                    trimmed_q.clone()
                };
                let c = database::create_conversation(&conn, Some(&title))
                    .map_err(|e| e.to_string())?;
                c.id
            }
        };

        // Add user message
        database::add_message(&conn, &conv_id, "user", &trimmed_q, None)
            .map_err(|e| format!("Failed to save user message: {}", e))?;

        // Extract matching tags and concepts for query prioritization
        let mut prioritized = std::collections::HashSet::new();
        let lower_q = trimmed_q.to_lowercase();
        if let Ok(all_tags) = database::get_all_tags(&conn) {
            for t in all_tags {
                if lower_q.contains(&t.normalized_name) || lower_q.contains(&t.name.to_lowercase()) {
                    if let Ok(docs) = database::get_documents_by_tag(&conn, &t.name) {
                        prioritized.extend(docs);
                    }
                }
            }
        }
        if let Ok(all_concepts) = database::get_all_concepts(&conn) {
            for c in all_concepts {
                if lower_q.contains(&c.canonical_id) || lower_q.contains(&c.name.to_lowercase()) {
                    if let Ok(mut stmt) = conn.prepare("SELECT document_id FROM document_concepts WHERE concept_id = ?1") {
                        if let Ok(rows) = stmt.query_map([&c.id], |r| r.get(0)) {
                            for r in rows.flatten() {
                                prioritized.insert(r);
                            }
                        }
                    }
                }
            }
        }

        let embeddings = database::get_all_embeddings(&conn).map_err(|e| e.to_string())?;
        let msgs = database::get_conversation_messages(&conn, &conv_id).unwrap_or_default();

        (conv_id, embeddings, msgs, prioritized)
    };

    // 2. Run RAG pipeline (no DB lock held)
    let response = ai::rag::execute_rag_pipeline(
        &ai.gemini,
        &ai.groq,
        &embedding_api_key,
        &text_gen_api_key,
        &conf,
        &active_conv_id,
        &trimmed_q,
        &stored_embeddings,
        &history,
        &prioritized_doc_ids,
    )
    .await?;

    // 3. Persist assistant message (brief DB lock)
    {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let citations_json = serde_json::to_string(&response.citations).ok();
        database::add_message(
            &conn,
            &active_conv_id,
            "assistant",
            &response.answer,
            citations_json.as_deref(),
        )
        .map_err(|e| format!("Failed to save assistant response: {}", e))?;
    }

    Ok(response)
}

#[tauri::command]
pub fn list_conversations(storage: State<AppStorage>) -> Result<Vec<Conversation>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::list_conversations(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_conversation(
    storage: State<AppStorage>,
    title: Option<String>,
) -> Result<Conversation, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::create_conversation(&conn, title.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_conversation_messages(
    storage: State<AppStorage>,
    conversation_id: String,
) -> Result<Vec<ConversationMessage>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_conversation_messages(&conn, &conversation_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_conversation(
    storage: State<AppStorage>,
    conversation_id: String,
) -> Result<bool, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::delete_conversation(&conn, &conversation_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_conversation(
    storage: State<AppStorage>,
    conversation_id: String,
) -> Result<bool, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::clear_conversation_messages(&conn, &conversation_id).map_err(|e| e.to_string())
}
