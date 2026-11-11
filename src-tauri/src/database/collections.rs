use chrono::Utc;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

const COLLECTION_SELECT: &str = "SELECT c.id, c.name, c.description, c.color, c.icon, c.created_at, c.updated_at,
        (SELECT COUNT(*) FROM collection_documents cd WHERE cd.collection_id = c.id)
    FROM collections c";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub document_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDocumentRecord {
    pub id: String,
    pub filename: String,
    pub original_path: String,
    pub stored_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub content_hash: String,
    pub imported_at: String,
    pub updated_at: String,
    pub text_length: i64,
    pub added_at: String,
}

fn db_error(context: &str, error: rusqlite::Error) -> String {
    format!("{}: {}", context, error)
}

fn normalize_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Collection name cannot be empty.".to_string());
    }
    if trimmed.chars().count() > 120 {
        return Err("Collection name must be 120 characters or fewer.".to_string());
    }
    Ok(trimmed.to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_metadata(label: &str, value: Option<String>) -> Result<Option<String>, String> {
    let normalized = normalize_optional(value);
    if let Some(ref value) = normalized {
        if value.chars().count() > 32 {
            return Err(format!("{} must be 32 characters or fewer.", label));
        }
    }
    Ok(normalized)
}

fn collection_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CollectionRecord> {
    Ok(CollectionRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        color: row.get(3)?,
        icon: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        document_count: row.get(7)?,
    })
}

pub fn list_collections(conn: &Connection) -> Result<Vec<CollectionRecord>, String> {
    let sql = format!(
        "{} ORDER BY c.name COLLATE NOCASE ASC, c.id ASC;",
        COLLECTION_SELECT
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("Failed to prepare collection list query", e))?;

    let rows = stmt
        .query_map([], collection_from_row)
        .map_err(|e| db_error("Failed to query collections", e))?;

    let mut collections = Vec::new();
    for row in rows {
        collections.push(row.map_err(|e| db_error("Failed to read collection record", e))?);
    }
    Ok(collections)
}

pub fn get_collection(
    conn: &Connection,
    id: &str,
) -> Result<Option<CollectionRecord>, String> {
    let sql = format!("{} WHERE c.id = ?1;", COLLECTION_SELECT);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("Failed to prepare collection lookup", e))?;

    let mut rows = stmt
        .query(params![id])
        .map_err(|e| db_error("Failed to look up collection", e))?;

    match rows
        .next()
        .map_err(|e| db_error("Failed to read collection", e))?
    {
        Some(row) => Ok(Some(
            collection_from_row(row).map_err(|e| db_error("Failed to read collection record", e))?,
        )),
        None => Ok(None),
    }
}

fn require_collection(conn: &Connection, id: &str) -> Result<CollectionRecord, String> {
    get_collection(conn, id)?.ok_or_else(|| "Collection not found.".to_string())
}

pub fn create_collection(
    conn: &Connection,
    name: &str,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<CollectionRecord, String> {
    let name = normalize_name(name)?;
    let description = normalize_optional(description);
    let color = normalize_metadata("Collection color", color)?;
    let icon = normalize_metadata("Collection icon", icon)?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO collections (id, name, description, color, icon, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
        params![id, name, description, color, icon, now, now],
    )
    .map_err(|e| db_error("Failed to create collection", e))?;

    require_collection(conn, &id)
}

pub fn update_collection(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<CollectionRecord, String> {
    let name = normalize_name(name)?;
    let description = normalize_optional(description);
    let color = normalize_metadata("Collection color", color)?;
    let icon = normalize_metadata("Collection icon", icon)?;

    require_collection(conn, id)?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE collections
         SET name = ?1, description = ?2, color = ?3, icon = ?4, updated_at = ?5
         WHERE id = ?6;",
        params![name, description, color, icon, now, id],
    )
    .map_err(|e| db_error("Failed to update collection", e))?;

    require_collection(conn, id)
}

pub fn delete_collection(conn: &Connection, id: &str) -> Result<bool, String> {
    let changed = conn
        .execute("DELETE FROM collections WHERE id = ?1;", params![id])
        .map_err(|e| db_error("Failed to delete collection", e))?;

    Ok(changed > 0)
}

fn normalize_document_ids(document_ids: &[String]) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for document_id in document_ids {
        let trimmed = document_id.trim();
        if trimmed.is_empty() {
            return Err("Document ID cannot be empty.".to_string());
        }
        if seen.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }

    Ok(normalized)
}

pub fn add_documents_to_collection(
    conn: &Connection,
    collection_id: &str,
    document_ids: &[String],
) -> Result<CollectionRecord, String> {
    require_collection(conn, collection_id)?;

    let document_ids = normalize_document_ids(document_ids)?;
    if document_ids.is_empty() {
        return require_collection(conn, collection_id);
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("Failed to start collection update transaction", e))?;

    let found_documents = {
        let placeholders = std::iter::repeat("?")
            .take(document_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT id FROM documents WHERE id IN ({});", placeholders);
        let mut stmt = tx
            .prepare(&sql)
            .map_err(|e| db_error("Failed to prepare document validation query", e))?;
        let rows = stmt
            .query_map(
                params_from_iter(document_ids.iter().map(|id| id.as_str())),
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| db_error("Failed to validate documents", e))?;

        let mut found = HashSet::new();
        for row in rows {
            found.insert(
                row.map_err(|e| db_error("Failed to read validated document", e))?,
            );
        }
        found
    };

    let missing = document_ids
        .iter()
        .filter(|id| !found_documents.contains(*id))
        .cloned()
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        return Err(format!(
            "Document not found: {}. No documents were added.",
            missing.join(", ")
        ));
    }

    let now = Utc::now().to_rfc3339();
    for document_id in &document_ids {
        tx.execute(
            "INSERT OR IGNORE INTO collection_documents (collection_id, document_id, added_at)
             VALUES (?1, ?2, ?3);",
            params![collection_id, document_id, now],
        )
        .map_err(|e| {
            db_error(
                &format!("Failed to add document '{}' to collection", document_id),
                e,
            )
        })?;
    }

    tx.execute(
        "UPDATE collections SET updated_at = ?1 WHERE id = ?2;",
        params![now, collection_id],
    )
    .map_err(|e| db_error("Failed to update collection timestamp", e))?;

    tx.commit()
        .map_err(|e| db_error("Failed to commit collection update", e))?;

    require_collection(conn, collection_id)
}

pub fn remove_documents_from_collection(
    conn: &Connection,
    collection_id: &str,
    document_ids: &[String],
) -> Result<CollectionRecord, String> {
    require_collection(conn, collection_id)?;

    let document_ids = normalize_document_ids(document_ids)?;
    if document_ids.is_empty() {
        return require_collection(conn, collection_id);
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("Failed to start collection update transaction", e))?;

    for document_id in &document_ids {
        tx.execute(
            "DELETE FROM collection_documents
             WHERE collection_id = ?1 AND document_id = ?2;",
            params![collection_id, document_id],
        )
        .map_err(|e| {
            db_error(
                &format!(
                    "Failed to remove document '{}' from collection",
                    document_id
                ),
                e,
            )
        })?;
    }

    let now = Utc::now().to_rfc3339();
    tx.execute(
        "UPDATE collections SET updated_at = ?1 WHERE id = ?2;",
        params![now, collection_id],
    )
    .map_err(|e| db_error("Failed to update collection timestamp", e))?;

    tx.commit()
        .map_err(|e| db_error("Failed to commit collection update", e))?;

    require_collection(conn, collection_id)
}

pub fn get_collection_documents(
    conn: &Connection,
    collection_id: &str,
) -> Result<Vec<CollectionDocumentRecord>, String> {
    require_collection(conn, collection_id)?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.filename, d.original_path, d.stored_path, d.file_type, d.file_size,
                    d.content_hash, d.imported_at, d.updated_at, LENGTH(d.extracted_text), cd.added_at
             FROM collection_documents cd
             JOIN documents d ON d.id = cd.document_id
             WHERE cd.collection_id = ?1
             ORDER BY d.filename COLLATE NOCASE ASC, d.id ASC;",
        )
        .map_err(|e| db_error("Failed to prepare collection documents query", e))?;

    let rows = stmt
        .query_map(params![collection_id], |row| {
            Ok(CollectionDocumentRecord {
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
                added_at: row.get(10)?,
            })
        })
        .map_err(|e| db_error("Failed to query collection documents", e))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(row.map_err(|e| db_error("Failed to read collection document", e))?);
    }
    Ok(documents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations::run_migrations;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn schema_version(conn: &Connection) -> i32 {
        conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap()
    }

    fn table_exists(conn: &Connection, table: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1;",
            params![table],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            > 0
    }

    fn insert_document(conn: &Connection, id: &str, filename: &str) {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO documents (
                id, filename, original_path, stored_path, file_type, file_size,
                extracted_text, content_hash, imported_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
            params![
                id,
                filename,
                format!("C:/source/{}", filename),
                format!("C:/echo/{}", filename),
                "txt",
                64,
                "Collection test document content.",
                format!("hash-{}", id),
                now,
                now
            ],
        )
        .unwrap();
    }

    fn test_collection(conn: &Connection, name: &str) -> CollectionRecord {
        create_collection(
            conn,
            name,
            Some("A test collection".to_string()),
            Some("blue".to_string()),
            Some("folder".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_fresh_database_migrates_to_v9_and_is_idempotent() {
        let conn = setup_db();

        assert_eq!(schema_version(&conn), 9);
        assert!(table_exists(&conn, "collections"));
        assert!(table_exists(&conn, "collection_documents"));

        insert_document(&conn, "doc-1", "one.txt");
        let collection = test_collection(&conn, "Research");

        run_migrations(&conn).unwrap();

        assert_eq!(schema_version(&conn), 9);
        assert_eq!(list_collections(&conn).unwrap().len(), 1);
        assert_eq!(
            get_collection(&conn, &collection.id)
                .unwrap()
                .unwrap()
                .document_count,
            0
        );
    }

    #[test]
    fn test_simulated_v8_database_upgrades_without_losing_documents() {
        let conn = setup_db();
        insert_document(&conn, "doc-v8", "legacy.txt");

        conn.execute_batch(
            "DROP TABLE collection_documents;
             DROP TABLE collections;
             PRAGMA user_version = 8;",
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        assert_eq!(schema_version(&conn), 9);
        assert!(table_exists(&conn, "collections"));
        assert!(table_exists(&conn, "collection_documents"));

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let collection = test_collection(&conn, "Upgraded");
        add_documents_to_collection(&conn, &collection.id, &["doc-v8".to_string()]).unwrap();
        assert_eq!(
            get_collection_documents(&conn, &collection.id)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_collection_lifecycle_and_counts() {
        let conn = setup_db();
        insert_document(&conn, "doc-a", "alpha.txt");
        insert_document(&conn, "doc-b", "beta.txt");

        let collection = test_collection(&conn, "  Project Alpha  ");
        assert_eq!(collection.name, "Project Alpha");
        assert_eq!(collection.document_count, 0);

        let updated = update_collection(
            &conn,
            &collection.id,
            "Project Alpha Updated",
            Some("Updated description".to_string()),
            Some("violet".to_string()),
            Some("book".to_string()),
        )
        .unwrap();
        assert_eq!(updated.name, "Project Alpha Updated");
        assert_eq!(updated.description.as_deref(), Some("Updated description"));
        assert_eq!(updated.color.as_deref(), Some("violet"));
        assert_eq!(updated.icon.as_deref(), Some("book"));

        let with_docs = add_documents_to_collection(
            &conn,
            &collection.id,
            &[
                "doc-a".to_string(),
                "doc-b".to_string(),
                "doc-a".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(with_docs.document_count, 2);

        let docs = get_collection_documents(&conn, &collection.id).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].filename, "alpha.txt");
        assert_eq!(docs[1].filename, "beta.txt");

        let after_remove = remove_documents_from_collection(
            &conn,
            &collection.id,
            &["doc-a".to_string()],
        )
        .unwrap();
        assert_eq!(after_remove.document_count, 1);

        assert!(delete_collection(&conn, &collection.id).unwrap());
        assert!(get_collection(&conn, &collection.id).unwrap().is_none());

        let document_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(document_count, 2);
    }

    #[test]
    fn test_document_delete_cascades_membership() {
        let conn = setup_db();
        insert_document(&conn, "doc-cascade", "cascade.txt");
        let collection = test_collection(&conn, "Cascade");
        add_documents_to_collection(&conn, &collection.id, &["doc-cascade".to_string()]).unwrap();

        conn.execute("DELETE FROM documents WHERE id = ?1;", params!["doc-cascade"])
            .unwrap();

        assert!(get_collection_documents(&conn, &collection.id)
            .unwrap()
            .is_empty());
        assert_eq!(
            get_collection(&conn, &collection.id)
                .unwrap()
                .unwrap()
                .document_count,
            0
        );
    }

    #[test]
    fn test_duplicate_membership_is_ignored() {
        let conn = setup_db();
        insert_document(&conn, "doc-duplicate", "duplicate.txt");
        let collection = test_collection(&conn, "Duplicates");

        add_documents_to_collection(&conn, &collection.id, &["doc-duplicate".to_string()])
            .unwrap();
        let updated =
            add_documents_to_collection(&conn, &collection.id, &["doc-duplicate".to_string()])
                .unwrap();

        assert_eq!(updated.document_count, 1);
        assert_eq!(
            get_collection_documents(&conn, &collection.id)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_missing_document_rolls_back_whole_batch() {
        let conn = setup_db();
        insert_document(&conn, "doc-valid", "valid.txt");
        let collection = test_collection(&conn, "Rollback");

        let error = add_documents_to_collection(
            &conn,
            &collection.id,
            &["doc-valid".to_string(), "doc-missing".to_string()],
        )
        .unwrap_err();

        assert!(error.contains("Document not found"));
        assert_eq!(
            get_collection(&conn, &collection.id)
                .unwrap()
                .unwrap()
                .document_count,
            0
        );
        assert!(get_collection_documents(&conn, &collection.id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_empty_collection_name_is_rejected() {
        let conn = setup_db();
        assert!(create_collection(&conn, "   ", None, None, None).is_err());
        assert!(create_collection(&conn, "", None, None, None).is_err());
    }

    #[test]
    fn test_collections_persist_across_connection_restart() {
        let db_path = std::env::temp_dir().join(format!(
            "echo-collections-test-{}.db",
            Uuid::new_v4()
        ));

        let collection_id = {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn).unwrap();
            insert_document(&conn, "doc-persist", "persist.txt");
            let collection = test_collection(&conn, "Persistent");
            add_documents_to_collection(
                &conn,
                &collection.id,
                &["doc-persist".to_string()],
            )
            .unwrap();
            collection.id
        };

        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn).unwrap();
            let persisted = get_collection(&conn, &collection_id).unwrap().unwrap();
            assert_eq!(persisted.name, "Persistent");
            assert_eq!(persisted.document_count, 1);
        }

        let _ = std::fs::remove_file(&db_path);
    }
}
