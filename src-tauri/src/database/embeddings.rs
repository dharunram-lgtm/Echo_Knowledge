use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEmbedding {
    pub id: String,
    pub chunk_id: String,
    pub document_id: String,
    pub model: String,
    pub dimensions: usize,
    pub embedding: Vec<f32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbeddingWithChunk {
    pub embedding_id: String,
    pub chunk_id: String,
    pub document_id: String,
    pub filename: String,
    pub file_type: String,
    pub chunk_index: usize,
    pub chunk_text: String,
    pub model: String,
    pub dimensions: usize,
    pub embedding: Vec<f32>,
}

pub fn vector_to_blob(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &val in vec {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

pub fn blob_to_vector(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            f32::from_le_bytes(arr)
        })
        .collect()
}

pub fn insert_embedding(conn: &Connection, emb: &DocumentEmbedding) -> Result<()> {
    let blob = vector_to_blob(&emb.embedding);
    conn.execute(
        "INSERT INTO document_embeddings (
            id, chunk_id, document_id, model, dimensions, embedding_blob, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
        params![
            emb.id,
            emb.chunk_id,
            emb.document_id,
            emb.model,
            emb.dimensions as i64,
            blob,
            emb.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_all_embeddings(conn: &Connection) -> Result<Vec<StoredEmbeddingWithChunk>> {
    let mut stmt = conn.prepare(
        "SELECT 
            e.id, e.chunk_id, e.document_id, d.filename, d.file_type, 
            c.chunk_index, c.chunk_text, e.model, e.dimensions, e.embedding_blob
         FROM document_embeddings e
         JOIN document_chunks c ON e.chunk_id = c.id
         JOIN documents d ON e.document_id = d.id
         ORDER BY d.filename ASC, c.chunk_index ASC;",
    )?;

    let rows = stmt.query_map([], |row| {
        let chunk_index: i64 = row.get(5)?;
        let dimensions: i64 = row.get(8)?;
        let blob: Vec<u8> = row.get(9)?;
        let embedding = blob_to_vector(&blob);

        Ok(StoredEmbeddingWithChunk {
            embedding_id: row.get(0)?,
            chunk_id: row.get(1)?,
            document_id: row.get(2)?,
            filename: row.get(3)?,
            file_type: row.get(4)?,
            chunk_index: chunk_index as usize,
            chunk_text: row.get(6)?,
            model: row.get(7)?,
            dimensions: dimensions as usize,
            embedding,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_embeddings_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM document_embeddings;", [], |row| row.get(0))
}

pub fn get_indexed_documents_count(conn: &Connection) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(DISTINCT document_id) FROM document_embeddings;",
        [],
        |row| row.get(0),
    )
}

pub fn is_document_indexed(conn: &Connection, document_id: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM document_embeddings WHERE document_id = ?1;",
        params![document_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn delete_embeddings_for_document(conn: &Connection, document_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM document_embeddings WHERE document_id = ?1;",
        params![document_id],
    )?;
    Ok(())
}

