use crate::commands::ai::AIState;
use crate::database::{
    self, AppStorage, ConceptDocumentRecord, ConceptNeighborRecord, ConceptRelationshipRecord,
    GraphStats,
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_all_relationships(
    storage: State<'_, AppStorage>,
    limit: Option<usize>,
) -> Result<Vec<ConceptRelationshipRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_all_concept_relationships(&conn, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_concept_neighbors(
    storage: State<'_, AppStorage>,
    concept_id: String,
) -> Result<Vec<ConceptNeighborRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_concept_neighbors(&conn, &concept_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_concept_documents(
    storage: State<'_, AppStorage>,
    concept_id: String,
) -> Result<Vec<ConceptDocumentRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_concept_documents(&conn, &concept_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_document_relationships(
    storage: State<'_, AppStorage>,
    document_id: String,
) -> Result<Vec<ConceptRelationshipRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_document_relationships(&conn, &document_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_concept_relationship(
    storage: State<'_, AppStorage>,
    id: String,
) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::delete_concept_relationship(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_graph_stats(storage: State<'_, AppStorage>) -> Result<GraphStats, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_graph_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_node_positions(
    storage: State<'_, AppStorage>,
    positions: Vec<database::NodePositionInput>,
) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::save_node_positions(&conn, &positions).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_node_positions(
    storage: State<'_, AppStorage>,
) -> Result<Vec<database::ConceptNodePositionRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_all_node_positions(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_graph_viewport(
    storage: State<'_, AppStorage>,
    viewport: database::GraphViewportRecord,
) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::save_graph_viewport(&conn, &viewport).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_graph_viewport(
    storage: State<'_, AppStorage>,
) -> Result<Option<database::GraphViewportRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_graph_viewport(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn discover_document_relationships(
    app: AppHandle,
    document_id: String,
) -> Result<Vec<ConceptRelationshipRecord>, String> {
    let storage = app
        .try_state::<AppStorage>()
        .ok_or_else(|| "Storage not found".to_string())?;
    let ai = app
        .try_state::<AIState>()
        .ok_or_else(|| "AI state not found".to_string())?;

    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let api_key = crate::security::get_api_key(&conf.provider)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API key not configured".to_string())?;

    // 1. Read document & concepts within short lock
    let (filename, text, concepts) = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        let doc = database::get_document_by_id(&conn, &document_id)
            .map_err(|e| format!("Document not found: {}", e))?
            .ok_or_else(|| format!("Document {} not found", document_id))?;
        let concepts = database::get_concepts_for_document(&conn, &document_id)
            .map_err(|e| format!("Failed to load concepts: {}", e))?;
        (doc.filename, doc.extracted_text, concepts)
    };

    if concepts.len() < 2 {
        return Ok(Vec::new());
    }

    // 2. Pure network call with NO db lock held
    let raw_rels = crate::ai::provider::dispatch_extract_relationships(
        &ai.gemini,
        &ai.groq,
        &conf.provider,
        &api_key,
        &conf.generation_model,
        &filename,
        &text,
        &concepts,
    )
    .await?;

    // 3. Save discovered relationships
    let saved = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        crate::ai::graph::save_discovered_relationships(
            &conn,
            &document_id,
            &filename,
            &concepts,
            &raw_rels,
        )?
    };

    Ok(saved)
}

#[tauri::command]
pub async fn discover_all_relationships(app: AppHandle) -> Result<usize, String> {
    let storage = app
        .try_state::<AppStorage>()
        .ok_or_else(|| "Storage not found".to_string())?;

    // Get list of all doc IDs within brief lock
    let doc_ids: Vec<String> = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::get_all_documents(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|d| d.id)
            .collect()
    };

    let mut total_discovered = 0;
    for doc_id in doc_ids {
        if let Ok(rels) = discover_document_relationships(app.clone(), doc_id).await {
            total_discovered += rels.len();
        }
    }

    Ok(total_discovered)
}

#[tauri::command]
pub fn get_knowledge_clusters(
    storage: State<'_, AppStorage>,
) -> Result<Vec<database::KnowledgeClusterRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_knowledge_clusters(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recompute_clusters(
    storage: State<'_, AppStorage>,
) -> Result<Vec<database::KnowledgeClusterRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::compute_and_save_clusters(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_suggested_relationships(
    storage: State<'_, AppStorage>,
    status: Option<String>,
) -> Result<Vec<database::SuggestedRelationshipRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_suggested_relationships(&conn, status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn discover_hidden_connections(
    app: AppHandle,
) -> Result<Vec<database::SuggestedRelationshipRecord>, String> {
    crate::ai::graph::discover_hidden_connections(&app).await
}

#[tauri::command]
pub fn accept_suggested_relationship(
    storage: State<'_, AppStorage>,
    id: String,
) -> Result<database::ConceptRelationshipRecord, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::accept_suggested_relationship(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reject_suggested_relationship(
    storage: State<'_, AppStorage>,
    id: String,
) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::reject_suggested_relationship(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn explore_concept(
    app: AppHandle,
    concept_id: String,
    query_type: String,
    custom_prompt: Option<String>,
    compare_concept_id: Option<String>,
) -> Result<database::ConceptExplorationResult, String> {
    crate::ai::graph::explore_concept_with_echo(
        &app,
        &concept_id,
        &query_type,
        custom_prompt.as_deref(),
        compare_concept_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub fn get_knowledge_evolution_timeline(
    storage: State<'_, AppStorage>,
) -> Result<Vec<database::TimelineEventRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_knowledge_evolution_timeline(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_potential_conflicts(
    storage: State<'_, AppStorage>,
) -> Result<Vec<database::PotentialConflictRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::detect_potential_conflicts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_knowledge_insights(
    storage: State<'_, AppStorage>,
) -> Result<database::KnowledgeInsightsRecord, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_knowledge_insights(&conn).map_err(|e| e.to_string())
}
