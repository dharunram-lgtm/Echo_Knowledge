use tauri::State;

use crate::ai::tagging::normalize_tag_name;
use crate::commands::ai::AIState;
use crate::database::{self, AppStorage, ConceptRecord, DocumentTagRecord, TagRecord};
use crate::security;

#[tauri::command]
pub fn get_document_tags(
    storage: State<AppStorage>,
    document_id: String,
) -> Result<Vec<DocumentTagRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_tags_for_document(&conn, &document_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_tags(storage: State<AppStorage>) -> Result<Vec<TagRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_all_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_concepts(storage: State<AppStorage>) -> Result<Vec<ConceptRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_all_concepts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_manual_tag(
    storage: State<AppStorage>,
    document_id: String,
    tag_name: String,
    category: Option<String>,
) -> Result<DocumentTagRecord, String> {
    let trimmed = tag_name.trim();
    if trimmed.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }

    let cat = category.unwrap_or_else(|| "concept".to_string());
    let (display_name, canonical_id) = normalize_tag_name(trimmed);

    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    // 1. Upsert tag
    let tag_id = database::upsert_tag(&conn, &display_name, &canonical_id, &cat)
        .map_err(|e| format!("Failed to create tag: {}", e))?;

    // 2. Link tag to document (is_ai_generated = false, confidence = 1.0)
    database::add_tag_to_document(
        &conn,
        &document_id,
        &tag_id,
        false,
        1.0,
        Some("Manually added by user"),
    )
    .map_err(|e| format!("Failed to link tag to document: {}", e))?;

    // 3. Upsert concept & link concept for Knowledge Graph continuity
    let concept_id = database::upsert_concept(
        &conn,
        &display_name,
        &canonical_id,
        &cat,
        Some("User-defined concept"),
    )
    .map_err(|e| format!("Failed to create concept: {}", e))?;

    let _ = database::add_concept_to_document(
        &conn,
        &document_id,
        &concept_id,
        1.0,
        Some("User-defined concept"),
    );

    // Return the newly created record
    let all_tags =
        database::get_tags_for_document(&conn, &document_id).map_err(|e| e.to_string())?;

    all_tags
        .into_iter()
        .find(|t| t.tag_id == tag_id)
        .ok_or_else(|| "Failed to retrieve newly added tag".to_string())
}

#[tauri::command]
pub fn remove_document_tag(
    storage: State<AppStorage>,
    document_id: String,
    tag_id: String,
) -> Result<bool, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::remove_tag_from_document(&conn, &document_id, &tag_id)
        .map_err(|e| format!("Failed to remove tag from document: {}", e))
}

#[tauri::command]
pub async fn retag_document(
    storage: State<'_, AppStorage>,
    ai: State<'_, AIState>,
    document_id: String,
) -> Result<Vec<DocumentTagRecord>, String> {
    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let api_key = security::get_api_key(&conf.provider)?.ok_or_else(|| {
        "Gemini API key is not configured. Please set your key in Settings.".to_string()
    })?;

    let doc = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let d = database::get_document_by_id(&conn, &document_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Document not found: {}", document_id))?;
        let _ = database::update_tagging_status(&conn, &document_id, "tagging", None, 0);
        d
    };

    if doc.extracted_text.trim().is_empty() {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let _ = database::update_tagging_status(&conn, &document_id, "tagged", None, 0);
        return Ok(Vec::new());
    }

    let extracted = match crate::ai::provider::dispatch_extract_tags(
        &ai.gemini,
        &ai.groq,
        &conf.provider,
        &api_key,
        &conf.generation_model,
        &doc.filename,
        &doc.extracted_text,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            let conn = storage.conn.lock().map_err(|err| err.to_string())?;
            let _ = database::update_tagging_status(&conn, &document_id, "failed", Some(&e), 0);
            return Err(e);
        }
    };

    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    crate::ai::tagging::save_extracted_tags(&conn, &doc, &extracted)
}

#[tauri::command]
pub fn get_documents_by_tags(
    storage: State<AppStorage>,
    tag_names: Vec<String>,
) -> Result<Vec<String>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_documents_matching_all_tags(&conn, &tag_names)
        .map_err(|e| format!("Failed to filter documents by tags: {}", e))
}

#[tauri::command]
pub fn rename_tag(
    storage: State<AppStorage>,
    tag_id: String,
    new_name: String,
) -> Result<TagRecord, String> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err("New tag name cannot be empty".to_string());
    }

    let (display_name, canonical_id) = normalize_tag_name(trimmed);
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    database::rename_tag(&conn, &tag_id, &display_name, &canonical_id)
        .map_err(|e| format!("Failed to rename tag: {}", e))?;

    let all = database::get_all_tags(&conn).map_err(|e| e.to_string())?;
    all.into_iter()
        .find(|t| t.id == tag_id)
        .ok_or_else(|| "Renamed tag not found".to_string())
}

#[tauri::command]
pub fn get_all_tagging_statuses(
    storage: State<AppStorage>,
) -> Result<Vec<database::DocumentTaggingStatusRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_all_tagging_statuses(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn retry_failed_tagging(
    app: tauri::AppHandle,
    storage: State<'_, AppStorage>,
) -> Result<usize, String> {
    let pending_count: usize = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT document_id FROM document_tagging_status WHERE status IN ('failed', 'awaiting_key');")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        for r in rows {
            if let Ok(id) = r {
                ids.push(id);
            }
        }
        for id in &ids {
            let _ = database::update_tagging_status(&conn, id, "pending", None, 0);
        }
        ids.len()
    };

    if pending_count > 0 {
        use tauri::Emitter;
        let _ = app.emit("tagging:status_changed", ());
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            crate::ai::coordinator::run_background_tagging_worker(app_handle).await;
        });
    }

    Ok(pending_count)
}

#[tauri::command]
pub async fn retry_tagging_for_document(
    app: tauri::AppHandle,
    document_id: String,
) -> Result<(), String> {
    crate::ai::coordinator::enqueue_document_for_tagging(&app, &document_id);
    Ok(())
}
