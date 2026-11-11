use tauri::{AppHandle, Emitter, State};

use crate::database::{
    self, AppStorage, CollectionDocumentRecord, CollectionRecord,
};

const COLLECTIONS_CHANGED_EVENT: &str = "collections:changed";

fn emit_collections_changed(app: &AppHandle) {
    let _ = app.emit(COLLECTIONS_CHANGED_EVENT, ());
}

#[tauri::command]
pub fn list_collections(
    storage: State<'_, AppStorage>,
) -> Result<Vec<CollectionRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::list_collections(&conn)
}

#[tauri::command]
pub fn get_collection(
    storage: State<'_, AppStorage>,
    id: String,
) -> Result<CollectionRecord, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_collection(&conn, &id)?
        .ok_or_else(|| format!("Collection with id '{}' not found.", id))
}

#[tauri::command]
pub fn create_collection(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<CollectionRecord, String> {
    let collection = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::create_collection(&conn, &name, description, color, icon)?
    };

    emit_collections_changed(&app);
    Ok(collection)
}

#[tauri::command]
pub fn update_collection(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    id: String,
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<CollectionRecord, String> {
    let collection = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::update_collection(&conn, &id, &name, description, color, icon)?
    };

    emit_collections_changed(&app);
    Ok(collection)
}

#[tauri::command]
pub fn delete_collection(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    id: String,
) -> Result<bool, String> {
    let deleted = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::delete_collection(&conn, &id)?
    };

    if !deleted {
        return Err(format!("Collection with id '{}' not found.", id));
    }

    emit_collections_changed(&app);
    Ok(true)
}

#[tauri::command]
pub fn add_documents_to_collection(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    collection_id: String,
    document_ids: Vec<String>,
) -> Result<CollectionRecord, String> {
    let collection = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::add_documents_to_collection(&conn, &collection_id, &document_ids)?
    };

    emit_collections_changed(&app);
    Ok(collection)
}

#[tauri::command]
pub fn remove_documents_from_collection(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    collection_id: String,
    document_ids: Vec<String>,
) -> Result<CollectionRecord, String> {
    let collection = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::remove_documents_from_collection(&conn, &collection_id, &document_ids)?
    };

    emit_collections_changed(&app);
    Ok(collection)
}

#[tauri::command]
pub fn get_collection_documents(
    storage: State<'_, AppStorage>,
    collection_id: String,
) -> Result<Vec<CollectionDocumentRecord>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_collection_documents(&conn, &collection_id)
}
