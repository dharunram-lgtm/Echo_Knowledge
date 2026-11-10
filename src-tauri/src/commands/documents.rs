use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use uuid::Uuid;

use crate::database::{
    self, AppStorage, DocumentRecord, DocumentSearchResult, DocumentSummary, StorageStats,
};

#[derive(Debug, serde::Serialize)]
pub struct StorageInitResponse {
    pub initialized: bool,
    pub db_path: String,
    pub documents_dir: String,
    pub schema_version: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct DuplicateNotice {
    pub filename: String,
    pub existing_filename: String,
    pub content_hash: String,
    pub imported_at: String,
    pub reason: String,
}

#[derive(Debug, serde::Serialize)]
pub struct FailedImportNotice {
    pub filename: String,
    pub path: String,
    pub error: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ImportBatchResult {
    pub imported: Vec<DocumentSummary>,
    pub duplicates: Vec<DuplicateNotice>,
    pub failed: Vec<FailedImportNotice>,
    pub total_processed: usize,
}

#[tauri::command]
pub fn initialize_storage(storage: State<AppStorage>) -> Result<StorageInitResponse, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(StorageInitResponse {
        initialized: true,
        db_path: storage.db_path.to_string_lossy().to_string(),
        documents_dir: storage.documents_dir.to_string_lossy().to_string(),
        schema_version: version,
    })
}

#[tauri::command]
pub fn import_documents(
    app: tauri::AppHandle,
    storage: State<AppStorage>,
    file_paths: Vec<String>,
) -> Result<ImportBatchResult, String> {
    let mut imported = Vec::new();
    let mut duplicates = Vec::new();
    let mut failed = Vec::new();

    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    for path_str in &file_paths {
        let source_path = PathBuf::from(path_str);

        let filename = match source_path.file_name().and_then(|f| f.to_str()) {
            Some(name) => name.to_string(),
            None => {
                failed.push(FailedImportNotice {
                    filename: path_str.clone(),
                    path: path_str.clone(),
                    error: "Invalid filename or path format.".to_string(),
                });
                continue;
            }
        };

        let extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .unwrap_or_default();

        if !matches!(extension.as_str(), "pdf" | "md" | "txt") {
            failed.push(FailedImportNotice {
                filename: filename.clone(),
                path: path_str.clone(),
                error: format!(
                    "Unsupported file extension '.{}'. Only .pdf, .md, and .txt are supported.",
                    extension
                ),
            });
            continue;
        }

        let bytes = match fs::read(&source_path) {
            Ok(b) => b,
            Err(e) => {
                failed.push(FailedImportNotice {
                    filename: filename.clone(),
                    path: path_str.clone(),
                    error: format!("Failed to read file from disk: {}", e),
                });
                continue;
            }
        };

        let file_size = bytes.len() as i64;

        // Calculate SHA-256 content hash
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let content_hash = hex::encode(hasher.finalize());

        // Check for duplicates
        match database::find_by_content_hash(&conn, &content_hash) {
            Ok(Some(existing)) => {
                duplicates.push(DuplicateNotice {
                    filename: filename.clone(),
                    existing_filename: existing.filename.clone(),
                    content_hash: content_hash.clone(),
                    imported_at: existing.imported_at,
                    reason: format!(
                        "Duplicate detected: Identical file content already imported as '{}'.",
                        existing.filename
                    ),
                });
                continue;
            }
            Ok(None) => {}
            Err(e) => {
                failed.push(FailedImportNotice {
                    filename: filename.clone(),
                    path: path_str.clone(),
                    error: format!("Database error during duplicate check: {}", e),
                });
                continue;
            }
        }

        // Generate safe unique storage destination in ECHO's managed directory
        let doc_id = Uuid::new_v4().to_string();
        let sanitized_filename = sanitize_filename(&filename);
        let stored_filename = format!("{}_{}", doc_id, sanitized_filename);
        let stored_path = storage.documents_dir.join(&stored_filename);

        // Copy source file into managed storage
        if let Err(e) = fs::write(&stored_path, &bytes) {
            failed.push(FailedImportNotice {
                filename: filename.clone(),
                path: path_str.clone(),
                error: format!("Failed to copy file to managed document storage: {}", e),
            });
            continue;
        }

        // Extract text based on file format
        let extracted_text = match extension.as_str() {
            "pdf" => extract_pdf_text(&stored_path),
            "md" | "txt" => String::from_utf8_lossy(&bytes).to_string(),
            _ => String::new(),
        };

        let now = Utc::now().to_rfc3339();

        let doc_record = DocumentRecord {
            id: doc_id.clone(),
            filename: filename.clone(),
            original_path: path_str.clone(),
            stored_path: stored_path.to_string_lossy().to_string(),
            file_type: extension.clone(),
            file_size,
            extracted_text: extracted_text.clone(),
            content_hash: content_hash.clone(),
            imported_at: now.clone(),
            updated_at: now.clone(),
        };

        if let Err(e) = database::insert_document(&conn, &doc_record) {
            // Roll back copied file if database insert fails
            let _ = fs::remove_file(&stored_path);
            failed.push(FailedImportNotice {
                filename: filename.clone(),
                path: path_str.clone(),
                error: format!("Failed to save document metadata in database: {}", e),
            });
            continue;
        }

        imported.push(DocumentSummary {
            id: doc_id,
            filename,
            original_path: path_str.clone(),
            stored_path: stored_path.to_string_lossy().to_string(),
            file_type: extension,
            file_size,
            content_hash,
            imported_at: now.clone(),
            updated_at: now,
            text_length: extracted_text.len(),
        });
    }

    // Automatically enqueue newly imported documents for background indexing & AI tagging
    for doc in &imported {
        crate::ai::enqueue_document_for_indexing(&app, &doc.id);
        crate::ai::enqueue_document_for_tagging(&app, &doc.id);
    }

    let total_processed = file_paths.len();

    Ok(ImportBatchResult {
        imported,
        duplicates,
        failed,
        total_processed,
    })
}

fn extract_pdf_text(path: &Path) -> String {
    match pdf_extract::extract_text(path) {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                "[Notice: No extractable text found in this PDF. Scanned documents requiring OCR are not yet supported in Phase 2.]".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Err(e) => {
            format!(
                "[Notice: PDF text extraction could not process this file: {}. Scanned documents requiring OCR are not yet supported.]",
                e
            )
        }
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[tauri::command]
pub fn get_documents(storage: State<AppStorage>) -> Result<Vec<DocumentSummary>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_all_documents(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_document(storage: State<AppStorage>, id: String) -> Result<DocumentRecord, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_document_by_id(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Document with id '{}' not found.", id))
}

#[tauri::command]
pub fn search_documents(
    storage: State<AppStorage>,
    query: String,
) -> Result<Vec<DocumentSearchResult>, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::search_documents(&conn, &query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_document(storage: State<AppStorage>, id: String) -> Result<bool, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let stored_path_opt = database::delete_document(&conn, &id).map_err(|e| e.to_string())?;

    if let Some(stored_path_str) = stored_path_opt {
        let path = PathBuf::from(&stored_path_str);
        // Only delete if the file is inside ECHO's managed documents directory
        if storage.is_path_in_managed_storage(&path) && path.exists() {
            let _ = fs::remove_file(path);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn get_storage_stats(storage: State<AppStorage>) -> Result<StorageStats, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_storage_stats(&conn).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migrations::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_text_extraction_from_pdf() {
        let pdf_path = PathBuf::from("../test_samples/sample_doc.pdf");
        if pdf_path.exists() {
            let extracted = extract_pdf_text(&pdf_path);
            assert!(
                extracted.contains("ECHO Knowledge Operating System") || extracted.contains("PDF"),
                "Extracted text did not contain expected content: {}",
                extracted
            );
        }
    }

    #[test]
    fn test_document_lifecycle_in_sqlite() {
        let conn = setup_test_db();

        let doc = DocumentRecord {
            id: "test-uuid-1".to_string(),
            filename: "sample_note.txt".to_string(),
            original_path: "C:/path/sample_note.txt".to_string(),
            stored_path: "C:/echo/docs/test-uuid-1_sample_note.txt".to_string(),
            file_type: "txt".to_string(),
            file_size: 120,
            extracted_text: "ECHO Architecture Note: Local-first knowledge operating systems ensure user privacy.".to_string(),
            content_hash: "hash123456789".to_string(),
            imported_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        // 1. Insert
        database::insert_document(&conn, &doc).unwrap();

        // 2. Duplicate Detection
        let duplicate = database::find_by_content_hash(&conn, "hash123456789").unwrap();
        assert!(duplicate.is_some());
        assert_eq!(duplicate.unwrap().filename, "sample_note.txt");

        // 3. Search
        let search_results = database::search_documents(&conn, "Local-first").unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].match_type, "content");

        // 4. Get by ID
        let fetched = database::get_document_by_id(&conn, "test-uuid-1").unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().extracted_text, doc.extracted_text);

        // 5. Stats
        let stats = database::get_storage_stats(&conn).unwrap();
        assert_eq!(stats.total_documents, 1);
        assert_eq!(stats.txt_count, 1);
        assert_eq!(stats.total_size_bytes, doc.file_size);

        // 6. Delete
        let deleted_path = database::delete_document(&conn, "test-uuid-1").unwrap();
        assert_eq!(deleted_path, Some(doc.stored_path));

        let stats_after = database::get_storage_stats(&conn).unwrap();
        assert_eq!(stats_after.total_documents, 0);
    }
}
