use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexingStatusRecord {
    pub document_id: String,
    pub status: String, // 'indexed', 'pending', 'indexing', 'failed', 'awaiting_key'
    pub error_message: Option<String>,
    pub content_hash: String,
    pub model: Option<String>,
    pub dimensions: Option<usize>,
    pub chunks_count: usize,
    pub embeddings_count: usize,
    pub last_attempt_at: Option<String>,
    pub indexed_at: Option<String>,
    // Phase 4.1 Retry Metadata
    pub attempt_count: usize,
    pub next_retry_at: Option<String>,
    pub retry_state: String, // 'pending', 'processing', 'retry_wait', 'succeeded', 'awaiting_key', 'paused', 'permanently_failed', 'cancelled'
    pub last_successful_attempt_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingCounts {
    pub total: i64,
    pub indexed: i64,
    pub pending: i64,
    pub indexing: i64,
    pub failed: i64,
    pub awaiting_key: i64,
    pub last_indexed_at: Option<String>,
}

pub fn upsert_indexing_status(
    conn: &Connection,
    record: &DocumentIndexingStatusRecord,
) -> Result<()> {
    conn.execute(
        "INSERT INTO document_indexing_status (
            document_id, status, error_message, content_hash, model,
            dimensions, chunks_count, embeddings_count, last_attempt_at, indexed_at,
            attempt_count, next_retry_at, retry_state, last_successful_attempt_at, last_error
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(document_id) DO UPDATE SET
            status = ?2,
            error_message = ?3,
            content_hash = ?4,
            model = ?5,
            dimensions = ?6,
            chunks_count = ?7,
            embeddings_count = ?8,
            last_attempt_at = ?9,
            indexed_at = ?10,
            attempt_count = ?11,
            next_retry_at = ?12,
            retry_state = ?13,
            last_successful_attempt_at = ?14,
            last_error = ?15;",
        params![
            record.document_id,
            record.status,
            record.error_message,
            record.content_hash,
            record.model,
            record.dimensions.map(|d| d as i64),
            record.chunks_count as i64,
            record.embeddings_count as i64,
            record.last_attempt_at,
            record.indexed_at,
            record.attempt_count as i64,
            record.next_retry_at,
            record.retry_state,
            record.last_successful_attempt_at,
            record.last_error,
        ],
    )?;
    Ok(())
}

pub fn update_status(
    conn: &Connection,
    document_id: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let retry_state = match status {
        "indexed" => "succeeded",
        "indexing" => "processing",
        "failed" => "permanently_failed",
        "awaiting_key" => "awaiting_key",
        _ => "pending",
    };

    let last_success = if status == "indexed" {
        Some(now.clone())
    } else {
        None
    };

    conn.execute(
        "UPDATE document_indexing_status 
         SET status = ?1, error_message = ?2, last_attempt_at = ?3,
             retry_state = ?4, last_error = ?2,
             last_successful_attempt_at = COALESCE(?5, last_successful_attempt_at)
         WHERE document_id = ?6;",
        params![status, error_message, now, retry_state, last_success, document_id],
    )?;
    Ok(())
}

pub fn update_indexing_retry_state(
    conn: &Connection,
    document_id: &str,
    retry_state: &str,
    status: &str,
    attempt_count: usize,
    next_retry_at: Option<&str>,
    error_message: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let last_success = if retry_state == "succeeded" || status == "indexed" {
        Some(now.clone())
    } else {
        None
    };

    conn.execute(
        "UPDATE document_indexing_status
         SET retry_state = ?1,
             status = ?2,
             attempt_count = ?3,
             next_retry_at = ?4,
             error_message = ?5,
             last_error = ?5,
             last_attempt_at = ?6,
             last_successful_attempt_at = COALESCE(?7, last_successful_attempt_at)
         WHERE document_id = ?8;",
        params![
            retry_state,
            status,
            attempt_count as i64,
            next_retry_at,
            error_message,
            now,
            last_success,
            document_id
        ],
    )?;
    Ok(())
}

pub fn get_indexing_status(
    conn: &Connection,
    doc_id: &str,
) -> Result<Option<DocumentIndexingStatusRecord>> {
    let mut stmt = conn.prepare(
        "SELECT document_id, status, error_message, content_hash, model,
                dimensions, chunks_count, embeddings_count, last_attempt_at, indexed_at,
                COALESCE(attempt_count, 0), next_retry_at, COALESCE(retry_state, 'pending'),
                last_successful_attempt_at, last_error
         FROM document_indexing_status WHERE document_id = ?1;",
    )?;

    let mut rows = stmt.query([doc_id])?;
    if let Some(row) = rows.next()? {
        let dim_i64: Option<i64> = row.get(5)?;
        let chunks_i64: i64 = row.get(6)?;
        let emb_i64: i64 = row.get(7)?;
        let att_i64: i64 = row.get(10).unwrap_or(0);

        Ok(Some(DocumentIndexingStatusRecord {
            document_id: row.get(0)?,
            status: row.get(1)?,
            error_message: row.get(2)?,
            content_hash: row.get(3)?,
            model: row.get(4)?,
            dimensions: dim_i64.map(|d| d as usize),
            chunks_count: chunks_i64 as usize,
            embeddings_count: emb_i64 as usize,
            last_attempt_at: row.get(8)?,
            indexed_at: row.get(9)?,
            attempt_count: att_i64 as usize,
            next_retry_at: row.get(11)?,
            retry_state: row.get(12)?,
            last_successful_attempt_at: row.get(13)?,
            last_error: row.get(14)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_all_indexing_statuses(
    conn: &Connection,
) -> Result<HashMap<String, DocumentIndexingStatusRecord>> {
    let mut stmt = conn.prepare(
        "SELECT document_id, status, error_message, content_hash, model,
                dimensions, chunks_count, embeddings_count, last_attempt_at, indexed_at,
                COALESCE(attempt_count, 0), next_retry_at, COALESCE(retry_state, 'pending'),
                last_successful_attempt_at, last_error
         FROM document_indexing_status;",
    )?;

    let rows = stmt.query_map([], |row| {
        let dim_i64: Option<i64> = row.get(5)?;
        let chunks_i64: i64 = row.get(6)?;
        let emb_i64: i64 = row.get(7)?;
        let att_i64: i64 = row.get(10).unwrap_or(0);

        Ok((
            row.get::<_, String>(0)?,
            DocumentIndexingStatusRecord {
                document_id: row.get(0)?,
                status: row.get(1)?,
                error_message: row.get(2)?,
                content_hash: row.get(3)?,
                model: row.get(4)?,
                dimensions: dim_i64.map(|d| d as usize),
                chunks_count: chunks_i64 as usize,
                embeddings_count: emb_i64 as usize,
                last_attempt_at: row.get(8)?,
                indexed_at: row.get(9)?,
                attempt_count: att_i64 as usize,
                next_retry_at: row.get(11)?,
                retry_state: row.get(12)?,
                last_successful_attempt_at: row.get(13)?,
                last_error: row.get(14)?,
            },
        ))
    })?;

    let mut map = HashMap::new();
    for r in rows {
        let (id, status) = r?;
        map.insert(id, status);
    }
    Ok(map)
}

pub fn get_due_indexing_retries(conn: &Connection, now_iso: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT document_id FROM document_indexing_status
         WHERE retry_state = 'retry_wait' AND (next_retry_at IS NULL OR next_retry_at <= ?1)
         ORDER BY next_retry_at ASC;",
    )?;

    let rows = stmt.query_map([now_iso], |row| row.get(0))?;
    let mut ids = Vec::new();
    for r in rows {
        ids.push(r?);
    }
    Ok(ids)
}

pub fn pause_all_indexing_retries(conn: &Connection) -> Result<usize> {
    conn.execute(
        "UPDATE document_indexing_status
         SET retry_state = 'paused'
         WHERE retry_state IN ('retry_wait', 'pending');",
        [],
    )
}

pub fn resume_all_indexing_retries(conn: &Connection) -> Result<usize> {
    conn.execute(
        "UPDATE document_indexing_status
         SET retry_state = 'retry_wait'
         WHERE retry_state = 'paused';",
        [],
    )
}

pub fn cancel_indexing_retry(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE document_indexing_status
         SET retry_state = 'cancelled', next_retry_at = NULL
         WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

pub fn reset_indexing_retry(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE document_indexing_status
         SET retry_state = 'pending', status = 'pending', attempt_count = 0, next_retry_at = NULL, last_error = NULL, error_message = NULL
         WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

pub fn get_indexing_counts(conn: &Connection) -> Result<IndexingCounts> {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents;", [], |r| r.get(0))
        .unwrap_or(0);

    let indexed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM document_indexing_status WHERE status = 'indexed';",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let pending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM document_indexing_status WHERE status = 'pending';",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let indexing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM document_indexing_status WHERE status = 'indexing';",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let failed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM document_indexing_status WHERE status = 'failed';",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let awaiting_key: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM document_indexing_status WHERE status = 'awaiting_key';",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // If documents exist that are not yet in document_indexing_status, treat them as pending
    let tracked: i64 = conn
        .query_row("SELECT COUNT(*) FROM document_indexing_status;", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let untracked = (total - tracked).max(0);

    let last_indexed_at: Option<String> = conn
        .query_row(
            "SELECT indexed_at FROM document_indexing_status WHERE status = 'indexed' AND indexed_at IS NOT NULL ORDER BY indexed_at DESC LIMIT 1;",
            [],
            |r| r.get(0),
        )
        .ok();

    Ok(IndexingCounts {
        total,
        indexed,
        pending: pending + untracked,
        indexing,
        failed,
        awaiting_key,
        last_indexed_at,
    })
}

