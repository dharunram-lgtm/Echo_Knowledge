use chrono::Utc;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRecord {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub category: String, // 'technology', 'concept', 'topic'
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTagRecord {
    pub document_id: String,
    pub tag_id: String,
    pub name: String,
    pub normalized_name: String,
    pub category: String,
    pub is_ai_generated: bool,
    pub confidence: f32,
    pub evidence: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRecord {
    pub id: String,
    pub name: String,
    pub canonical_id: String,
    pub category: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentConceptRecord {
    pub document_id: String,
    pub concept_id: String,
    pub name: String,
    pub canonical_id: String,
    pub category: String,
    pub confidence: f32,
    pub evidence: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTaggingStatusRecord {
    pub document_id: String,
    pub status: String, // 'untagged', 'pending', 'tagging', 'tagged', 'failed', 'awaiting_key'
    pub error_message: Option<String>,
    pub content_hash: String,
    pub tags_count: usize,
    pub last_attempt_at: Option<String>,
    pub tagged_at: Option<String>,
    // Phase 4.1 Retry Metadata
    pub attempt_count: usize,
    pub next_retry_at: Option<String>,
    pub retry_state: String, // 'pending', 'processing', 'retry_wait', 'succeeded', 'awaiting_key', 'paused', 'permanently_failed', 'cancelled'
    pub last_successful_attempt_at: Option<String>,
    pub last_error: Option<String>,
}

/// Upserts a tag record based on its normalized_name and returns the tag ID
pub fn upsert_tag(
    conn: &Connection,
    name: &str,
    normalized_name: &str,
    category: &str,
) -> Result<String> {
    let existing_id: Result<String, _> = conn.query_row(
        "SELECT id FROM tags WHERE normalized_name = ?1;",
        params![normalized_name],
        |row| row.get(0),
    );

    if let Ok(id) = existing_id {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO tags (id, name, normalized_name, category, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(name) DO UPDATE SET normalized_name = ?3, category = ?4;",
        params![id, name, normalized_name, category, now],
    )?;

    // Fetch confirmed ID in case of name collision
    let confirmed_id: String = conn.query_row(
        "SELECT id FROM tags WHERE normalized_name = ?1 OR name = ?2;",
        params![normalized_name, name],
        |row| row.get(0),
    )?;

    Ok(confirmed_id)
}

/// Links a tag to a document with confidence, evidence, and AI flag
pub fn add_tag_to_document(
    conn: &Connection,
    document_id: &str,
    tag_id: &str,
    is_ai_generated: bool,
    confidence: f32,
    evidence: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO document_tags (document_id, tag_id, is_ai_generated, confidence, evidence, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(document_id, tag_id) DO UPDATE SET
            is_ai_generated = ?3,
            confidence = ?4,
            evidence = ?5;",
        params![
            document_id,
            tag_id,
            if is_ai_generated { 1 } else { 0 },
            confidence,
            evidence,
            now
        ],
    )?;
    Ok(())
}

/// Removes a tag from a document
pub fn remove_tag_from_document(
    conn: &Connection,
    document_id: &str,
    tag_id: &str,
) -> Result<bool> {
    let count = conn.execute(
        "DELETE FROM document_tags WHERE document_id = ?1 AND tag_id = ?2;",
        params![document_id, tag_id],
    )?;

    // Update tags count in status table if present
    let current_tags_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM document_tags WHERE document_id = ?1;",
            params![document_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let _ = conn.execute(
        "UPDATE document_tagging_status SET tags_count = ?1 WHERE document_id = ?2;",
        params![current_tags_count as i64, document_id],
    );

    Ok(count > 0)
}

/// Retrieves all tags attached to a specific document
pub fn get_tags_for_document(
    conn: &Connection,
    document_id: &str,
) -> Result<Vec<DocumentTagRecord>> {
    let mut stmt = conn.prepare(
        "SELECT dt.document_id, dt.tag_id, t.name, t.normalized_name, t.category,
                dt.is_ai_generated, dt.confidence, dt.evidence, dt.created_at
         FROM document_tags dt
         JOIN tags t ON dt.tag_id = t.id
         WHERE dt.document_id = ?1
         ORDER BY dt.confidence DESC, t.name ASC;",
    )?;

    let rows = stmt.query_map(params![document_id], |row| {
        let is_ai_gen_int: i32 = row.get(5)?;
        let conf_f64: f64 = row.get(6)?;
        Ok(DocumentTagRecord {
            document_id: row.get(0)?,
            tag_id: row.get(1)?,
            name: row.get(2)?,
            normalized_name: row.get(3)?,
            category: row.get(4)?,
            is_ai_generated: is_ai_gen_int != 0,
            confidence: conf_f64 as f32,
            evidence: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Retrieves all distinct tags across the entire repository
pub fn get_all_tags(conn: &Connection) -> Result<Vec<TagRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, normalized_name, category, created_at
         FROM tags
         ORDER BY name ASC;",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(TagRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            normalized_name: row.get(2)?,
            category: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Retrieves document IDs associated with given tag name (or normalized name)
pub fn get_documents_by_tag(conn: &Connection, tag_name: &str) -> Result<Vec<String>> {
    let lower = tag_name.trim().to_lowercase();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT dt.document_id
         FROM document_tags dt
         JOIN tags t ON dt.tag_id = t.id
         WHERE LOWER(t.name) = ?1 OR LOWER(t.normalized_name) = ?1;",
    )?;

    let rows = stmt.query_map(params![lower], |row| row.get(0))?;
    let mut doc_ids = Vec::new();
    for id in rows {
        doc_ids.push(id?);
    }
    Ok(doc_ids)
}

/// Retrieves document IDs matching ALL of the given tag names (AND filter)
pub fn get_documents_matching_all_tags(
    conn: &Connection,
    tag_names: &[String],
) -> Result<Vec<String>> {
    if tag_names.is_empty() {
        return Ok(Vec::new());
    }

    let mut candidate_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for t in tag_names {
        let docs = get_documents_by_tag(conn, t)?;
        for d in docs {
            *candidate_counts.entry(d).or_insert(0) += 1;
        }
    }

    let needed = tag_names.len();
    let mut result = Vec::new();
    for (doc_id, count) in candidate_counts {
        if count >= needed {
            result.push(doc_id);
        }
    }
    Ok(result)
}

/// Upserts a concept record
pub fn upsert_concept(
    conn: &Connection,
    name: &str,
    canonical_id: &str,
    category: &str,
    description: Option<&str>,
) -> Result<String> {
    let existing_id: Result<String, _> = conn.query_row(
        "SELECT id FROM concepts WHERE canonical_id = ?1;",
        params![canonical_id],
        |row| row.get(0),
    );

    if let Ok(id) = existing_id {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO concepts (id, name, canonical_id, category, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(canonical_id) DO UPDATE SET
            name = ?2,
            category = ?4,
            description = ?5;",
        params![id, name, canonical_id, category, description, now],
    )?;

    let confirmed_id: String = conn.query_row(
        "SELECT id FROM concepts WHERE canonical_id = ?1;",
        params![canonical_id],
        |row| row.get(0),
    )?;

    Ok(confirmed_id)
}

/// Links a concept to a document
pub fn add_concept_to_document(
    conn: &Connection,
    document_id: &str,
    concept_id: &str,
    confidence: f32,
    evidence: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO document_concepts (document_id, concept_id, confidence, evidence, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(document_id, concept_id) DO UPDATE SET
            confidence = ?3,
            evidence = ?4;",
        params![document_id, concept_id, confidence, evidence, now],
    )?;
    Ok(())
}

/// Retrieves all concepts linked to a document
pub fn get_concepts_for_document(
    conn: &Connection,
    document_id: &str,
) -> Result<Vec<DocumentConceptRecord>> {
    let mut stmt = conn.prepare(
        "SELECT dc.document_id, dc.concept_id, c.name, c.canonical_id, c.category,
                dc.confidence, dc.evidence, dc.created_at
         FROM document_concepts dc
         JOIN concepts c ON dc.concept_id = c.id
         WHERE dc.document_id = ?1
         ORDER BY dc.confidence DESC, c.name ASC;",
    )?;

    let rows = stmt.query_map(params![document_id], |row| {
        let conf_f64: f64 = row.get(5)?;
        Ok(DocumentConceptRecord {
            document_id: row.get(0)?,
            concept_id: row.get(1)?,
            name: row.get(2)?,
            canonical_id: row.get(3)?,
            category: row.get(4)?,
            confidence: conf_f64 as f32,
            evidence: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Retrieves all concepts across the repository
pub fn get_all_concepts(conn: &Connection) -> Result<Vec<ConceptRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, canonical_id, category, description, created_at
         FROM concepts
         ORDER BY name ASC;",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ConceptRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            canonical_id: row.get(2)?,
            category: row.get(3)?,
            description: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Upserts document tagging status
pub fn upsert_tagging_status(
    conn: &Connection,
    record: &DocumentTaggingStatusRecord,
) -> Result<()> {
    conn.execute(
        "INSERT INTO document_tagging_status (
            document_id, status, error_message, content_hash, tags_count, last_attempt_at, tagged_at,
            attempt_count, next_retry_at, retry_state, last_successful_attempt_at, last_error
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(document_id) DO UPDATE SET
            status = ?2,
            error_message = ?3,
            content_hash = ?4,
            tags_count = ?5,
            last_attempt_at = ?6,
            tagged_at = ?7,
            attempt_count = ?8,
            next_retry_at = ?9,
            retry_state = ?10,
            last_successful_attempt_at = ?11,
            last_error = ?12;",
        params![
            record.document_id,
            record.status,
            record.error_message,
            record.content_hash,
            record.tags_count as i64,
            record.last_attempt_at,
            record.tagged_at,
            record.attempt_count as i64,
            record.next_retry_at,
            record.retry_state,
            record.last_successful_attempt_at,
            record.last_error,
        ],
    )?;
    Ok(())
}

/// Updates or inserts document tagging status (UPSERT)
pub fn update_tagging_status(
    conn: &Connection,
    document_id: &str,
    status: &str,
    error_message: Option<&str>,
    tags_count: usize,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let tagged_at = if status == "tagged" {
        Some(now.clone())
    } else {
        None
    };

    let retry_state = match status {
        "tagged" => "succeeded",
        "tagging" => "processing",
        "failed" => "permanently_failed",
        "awaiting_key" => "awaiting_key",
        _ => "pending",
    };

    conn.execute(
        "INSERT INTO document_tagging_status (
            document_id, status, error_message, content_hash, tags_count, last_attempt_at, tagged_at,
            retry_state, last_error, last_successful_attempt_at
         ) VALUES (
            ?1, ?2, ?3, COALESCE((SELECT content_hash FROM documents WHERE id = ?1), ''), ?4, ?5, ?6,
            ?7, ?3, ?6
         )
         ON CONFLICT(document_id) DO UPDATE SET
             status = ?2,
             error_message = ?3,
             tags_count = ?4,
             last_attempt_at = ?5,
             tagged_at = COALESCE(?6, tagged_at),
             retry_state = ?7,
             last_error = ?3,
             last_successful_attempt_at = COALESCE(?6, last_successful_attempt_at);",
        params![
            document_id,
            status,
            error_message,
            tags_count as i64,
            now,
            tagged_at,
            retry_state,
        ],
    )?;
    Ok(())
}

/// Updates document tagging retry state with exponential backoff scheduling
pub fn update_tagging_retry_state(
    conn: &Connection,
    document_id: &str,
    retry_state: &str,
    status: &str,
    attempt_count: usize,
    next_retry_at: Option<&str>,
    error_message: Option<&str>,
    tags_count: usize,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let tagged_at = if status == "tagged" || retry_state == "succeeded" {
        Some(now.clone())
    } else {
        None
    };

    conn.execute(
        "INSERT INTO document_tagging_status (
            document_id, status, error_message, content_hash, tags_count, last_attempt_at, tagged_at,
            attempt_count, next_retry_at, retry_state, last_successful_attempt_at, last_error
         ) VALUES (
            ?1, ?2, ?3, COALESCE((SELECT content_hash FROM documents WHERE id = ?1), ''), ?4, ?5, ?6,
            ?7, ?8, ?9, ?6, ?3
         )
         ON CONFLICT(document_id) DO UPDATE SET
             status = ?2,
             error_message = ?3,
             tags_count = ?4,
             last_attempt_at = ?5,
             tagged_at = COALESCE(?6, tagged_at),
             attempt_count = ?7,
             next_retry_at = ?8,
             retry_state = ?9,
             last_error = ?3,
             last_successful_attempt_at = COALESCE(?6, last_successful_attempt_at);",
        params![
            document_id,
            status,
            error_message,
            tags_count as i64,
            now,
            tagged_at,
            attempt_count as i64,
            next_retry_at,
            retry_state,
        ],
    )?;
    Ok(())
}

/// Retrieves all document tagging statuses across the repository
pub fn get_all_tagging_statuses(conn: &Connection) -> Result<Vec<DocumentTaggingStatusRecord>> {
    let mut stmt = conn.prepare(
        "SELECT document_id, status, error_message, content_hash, tags_count, last_attempt_at, tagged_at,
                COALESCE(attempt_count, 0), next_retry_at, COALESCE(retry_state, 'pending'),
                last_successful_attempt_at, last_error
         FROM document_tagging_status;",
    )?;

    let rows = stmt.query_map([], |row| {
        let count: i64 = row.get(4)?;
        let att_i64: i64 = row.get(7).unwrap_or(0);
        Ok(DocumentTaggingStatusRecord {
            document_id: row.get(0)?,
            status: row.get(1)?,
            error_message: row.get(2)?,
            content_hash: row.get(3)?,
            tags_count: count as usize,
            last_attempt_at: row.get(5)?,
            tagged_at: row.get(6)?,
            attempt_count: att_i64 as usize,
            next_retry_at: row.get(8)?,
            retry_state: row.get(9)?,
            last_successful_attempt_at: row.get(10)?,
            last_error: row.get(11)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Retrieves tagging status for a single document
pub fn get_tagging_status(conn: &Connection, document_id: &str) -> Result<Option<DocumentTaggingStatusRecord>> {
    let mut stmt = conn.prepare(
        "SELECT document_id, status, error_message, content_hash, tags_count, last_attempt_at, tagged_at,
                COALESCE(attempt_count, 0), next_retry_at, COALESCE(retry_state, 'pending'),
                last_successful_attempt_at, last_error
         FROM document_tagging_status
         WHERE document_id = ?1;",
    )?;

    let mut rows = stmt.query(params![document_id])?;
    if let Some(row) = rows.next()? {
        let count: i64 = row.get(4)?;
        let att_i64: i64 = row.get(7).unwrap_or(0);
        Ok(Some(DocumentTaggingStatusRecord {
            document_id: row.get(0)?,
            status: row.get(1)?,
            error_message: row.get(2)?,
            content_hash: row.get(3)?,
            tags_count: count as usize,
            last_attempt_at: row.get(5)?,
            tagged_at: row.get(6)?,
            attempt_count: att_i64 as usize,
            next_retry_at: row.get(8)?,
            retry_state: row.get(9)?,
            last_successful_attempt_at: row.get(10)?,
            last_error: row.get(11)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_due_tagging_retries(conn: &Connection, now_iso: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT document_id FROM document_tagging_status
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

pub fn pause_all_tagging_retries(conn: &Connection) -> Result<usize> {
    conn.execute(
        "UPDATE document_tagging_status
         SET retry_state = 'paused'
         WHERE retry_state IN ('retry_wait', 'pending');",
        [],
    )
}

pub fn resume_all_tagging_retries(conn: &Connection) -> Result<usize> {
    conn.execute(
        "UPDATE document_tagging_status
         SET retry_state = 'retry_wait'
         WHERE retry_state = 'paused';",
        [],
    )
}

pub fn cancel_tagging_retry(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE document_tagging_status
         SET retry_state = 'cancelled', next_retry_at = NULL
         WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

pub fn reset_tagging_retry(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE document_tagging_status
         SET retry_state = 'pending', status = 'pending', attempt_count = 0, next_retry_at = NULL, last_error = NULL, error_message = NULL
         WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

/// Returns count of AI-generated tags attached to a document
pub fn get_ai_tags_count_for_document(conn: &Connection, document_id: &str) -> Result<usize> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM document_tags WHERE document_id = ?1 AND is_ai_generated = 1;",
            params![document_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(count as usize)
}

/// Removes all tags and concepts for a document
pub fn delete_all_tags_for_document(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM document_tags WHERE document_id = ?1;",
        params![document_id],
    )?;
    conn.execute(
        "DELETE FROM document_concepts WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

/// Renames a tag across the entire repository
pub fn rename_tag(
    conn: &Connection,
    tag_id: &str,
    new_name: &str,
    new_normalized: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE tags SET name = ?1, normalized_name = ?2 WHERE id = ?3;",
        params![new_name, new_normalized, tag_id],
    )?;
    Ok(())
}
