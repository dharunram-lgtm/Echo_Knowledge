use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: usize,
    pub chunk_text: String,
    pub char_count: usize,
    pub word_count: usize,
    pub created_at: String,
}

pub fn insert_chunks(conn: &Connection, chunks: &[DocumentChunk]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO document_chunks (
            id, document_id, chunk_index, chunk_text, char_count, word_count, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
    )?;

    for chunk in chunks {
        stmt.execute(params![
            chunk.id,
            chunk.document_id,
            chunk.chunk_index as i64,
            chunk.chunk_text,
            chunk.char_count as i64,
            chunk.word_count as i64,
            chunk.created_at,
        ])?;
    }

    Ok(())
}

pub fn get_chunks_for_document(conn: &Connection, document_id: &str) -> Result<Vec<DocumentChunk>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, chunk_index, chunk_text, char_count, word_count, created_at
         FROM document_chunks
         WHERE document_id = ?1
         ORDER BY chunk_index ASC;",
    )?;

    let rows = stmt.query_map(params![document_id], |row| {
        let chunk_index: i64 = row.get(2)?;
        let char_count: i64 = row.get(4)?;
        let word_count: i64 = row.get(5)?;
        Ok(DocumentChunk {
            id: row.get(0)?,
            document_id: row.get(1)?,
            chunk_index: chunk_index as usize,
            chunk_text: row.get(3)?,
            char_count: char_count as usize,
            word_count: word_count as usize,
            created_at: row.get(6)?,
        })
    })?;

    let mut chunks = Vec::new();
    for row in rows {
        chunks.push(row?);
    }
    Ok(chunks)
}

pub fn delete_chunks_for_document(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM document_chunks WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

pub fn get_chunk_by_id(conn: &Connection, chunk_id: &str) -> Result<Option<DocumentChunk>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, chunk_index, chunk_text, char_count, word_count, created_at
         FROM document_chunks
         WHERE id = ?1;",
    )?;

    let mut rows = stmt.query(params![chunk_id])?;
    if let Some(row) = rows.next()? {
        let chunk_index: i64 = row.get(2)?;
        let char_count: i64 = row.get(4)?;
        let word_count: i64 = row.get(5)?;
        Ok(Some(DocumentChunk {
            id: row.get(0)?,
            document_id: row.get(1)?,
            chunk_index: chunk_index as usize,
            chunk_text: row.get(3)?,
            char_count: char_count as usize,
            word_count: word_count as usize,
            created_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_total_chunks(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM document_chunks;", [], |row| {
        row.get(0)
    })
}
