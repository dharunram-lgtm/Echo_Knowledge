use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: String,
    pub filename: String,
    pub original_path: String,
    pub stored_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub extracted_text: String,
    pub content_hash: String,
    pub imported_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: String,
    pub filename: String,
    pub original_path: String,
    pub stored_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub content_hash: String,
    pub imported_at: String,
    pub updated_at: String,
    pub text_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchResult {
    pub id: String,
    pub filename: String,
    pub file_type: String,
    pub file_size: i64,
    pub imported_at: String,
    pub snippet: String,
    pub match_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_documents: i64,
    pub total_size_bytes: i64,
    pub pdf_count: i64,
    pub md_count: i64,
    pub txt_count: i64,
    pub last_imported_at: Option<String>,
}

pub fn insert_document(conn: &Connection, doc: &DocumentRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO documents (
            id, filename, original_path, stored_path, file_type,
            file_size, extracted_text, content_hash, imported_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
        params![
            doc.id,
            doc.filename,
            doc.original_path,
            doc.stored_path,
            doc.file_type,
            doc.file_size,
            doc.extracted_text,
            doc.content_hash,
            doc.imported_at,
            doc.updated_at
        ],
    )?;
    Ok(())
}

pub fn find_by_content_hash(
    conn: &Connection,
    hash: &str,
) -> Result<Option<DocumentSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, original_path, stored_path, file_type, file_size, content_hash, imported_at, updated_at, LENGTH(extracted_text)
         FROM documents WHERE content_hash = ?1;",
    )?;

    let mut rows = stmt.query(params![hash])?;

    if let Some(row) = rows.next()? {
        Ok(Some(DocumentSummary {
            id: row.get(0)?,
            filename: row.get(1)?,
            original_path: row.get(2)?,
            stored_path: row.get(3)?,
            file_type: row.get(4)?,
            file_size: row.get(5)?,
            content_hash: row.get(6)?,
            imported_at: row.get(7)?,
            updated_at: row.get(8)?,
            text_length: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_all_documents(conn: &Connection) -> Result<Vec<DocumentSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, original_path, stored_path, file_type, file_size, content_hash, imported_at, updated_at, LENGTH(extracted_text)
         FROM documents
         ORDER BY imported_at DESC;",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(DocumentSummary {
            id: row.get(0)?,
            filename: row.get(1)?,
            original_path: row.get(2)?,
            stored_path: row.get(3)?,
            file_type: row.get(4)?,
            file_size: row.get(5)?,
            content_hash: row.get(6)?,
            imported_at: row.get(7)?,
            updated_at: row.get(8)?,
            text_length: row.get(9)?,
        })
    })?;

    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

pub fn get_document_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<DocumentRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, filename, original_path, stored_path, file_type, file_size, extracted_text, content_hash, imported_at, updated_at
         FROM documents WHERE id = ?1;",
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(DocumentRecord {
            id: row.get(0)?,
            filename: row.get(1)?,
            original_path: row.get(2)?,
            stored_path: row.get(3)?,
            file_type: row.get(4)?,
            file_size: row.get(5)?,
            extracted_text: row.get(6)?,
            content_hash: row.get(7)?,
            imported_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn search_documents(
    conn: &Connection,
    query: &str,
) -> Result<Vec<DocumentSearchResult>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let pattern = format!("%{}%", trimmed);

    let mut stmt = conn.prepare(
        "SELECT id, filename, file_type, file_size, imported_at, extracted_text
         FROM documents
         WHERE filename LIKE ?1 OR extracted_text LIKE ?1
         ORDER BY imported_at DESC
         LIMIT 50;",
    )?;

    let query_lower = trimmed.to_lowercase();

    let rows = stmt.query_map(params![pattern], |row| {
        let id: String = row.get(0)?;
        let filename: String = row.get(1)?;
        let file_type: String = row.get(2)?;
        let file_size: i64 = row.get(3)?;
        let imported_at: String = row.get(4)?;
        let extracted_text: String = row.get(5)?;

        let filename_matched = filename.to_lowercase().contains(&query_lower);
        let content_matched = extracted_text.to_lowercase().contains(&query_lower);

        let match_type = if filename_matched && content_matched {
            "both".to_string()
        } else if filename_matched {
            "filename".to_string()
        } else {
            "content".to_string()
        };

        let snippet = extract_search_snippet(&extracted_text, &query_lower);

        Ok(DocumentSearchResult {
            id,
            filename,
            file_type,
            file_size,
            imported_at,
            snippet,
            match_type,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

fn extract_search_snippet(text: &str, query_lower: &str) -> String {
    let lower_text = text.to_lowercase();
    if let Some(idx) = lower_text.find(query_lower) {
        let start = if idx > 60 { idx - 60 } else { 0 };
        let end = (idx + query_lower.len() + 90).min(text.len());
        
        let mut snippet = String::new();
        if start > 0 {
            snippet.push_str("...");
        }
        snippet.push_str(&text[start..end]);
        if end < text.len() {
            snippet.push_str("...");
        }
        snippet.replace(['\r', '\n'], " ")
    } else {
        // Fallback to start of text
        let end = text.len().min(120);
        let mut snippet = text[..end].replace(['\r', '\n'], " ");
        if text.len() > 120 {
            snippet.push_str("...");
        }
        snippet
    }
}

pub fn delete_document(conn: &Connection, id: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT stored_path FROM documents WHERE id = ?1;")?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        let stored_path: String = row.get(0)?;
        conn.execute("DELETE FROM documents WHERE id = ?1;", params![id])?;
        Ok(Some(stored_path))
    } else {
        Ok(None)
    }
}

pub fn get_storage_stats(conn: &Connection) -> Result<StorageStats> {
    let total_documents: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents;", [], |row| row.get(0))
        .unwrap_or(0);

    let total_size_bytes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM documents;",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pdf_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE file_type = 'pdf';",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let md_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE file_type = 'md';",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let txt_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM documents WHERE file_type = 'txt';",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let last_imported_at: Option<String> = conn
        .query_row(
            "SELECT imported_at FROM documents ORDER BY imported_at DESC LIMIT 1;",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(StorageStats {
        total_documents,
        total_size_bytes,
        pdf_count,
        md_count,
        txt_count,
        last_imported_at,
    })
}

