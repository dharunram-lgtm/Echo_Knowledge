use chrono::Utc;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const VALID_RELATIONSHIP_TYPES: &[&str] = &[
    "RELATED_TO",
    "IS_A",
    "PART_OF",
    "USES",
    "IMPLEMENTS",
    "REQUIRES",
    "COMPARED_WITH",
    "INCLUDES",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelationshipRecord {
    pub id: String,
    pub source_concept_id: String,
    pub source_concept_name: Option<String>,
    pub target_concept_id: String,
    pub target_concept_name: Option<String>,
    pub relationship_type: String,
    pub confidence: f32,
    pub evidence: String,
    pub source_document_id: Option<String>,
    pub source_document_name: Option<String>,
    pub source_chunk_id: Option<String>,
    pub discovery_method: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNeighborRecord {
    pub relationship_id: String,
    pub relationship_type: String,
    pub direction: String, // "outgoing" or "incoming"
    pub neighbor_concept_id: String,
    pub neighbor_concept_name: String,
    pub neighbor_category: String,
    pub confidence: f32,
    pub evidence: String,
    pub source_document_id: Option<String>,
    pub source_document_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptDocumentRecord {
    pub document_id: String,
    pub document_filename: String,
    pub concept_id: String,
    pub concept_name: String,
    pub confidence: f32,
    pub evidence: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_concepts: usize,
    pub total_relationships: usize,
    pub connected_documents: usize,
    pub relationship_type_counts: Vec<(String, usize)>,
}

pub fn is_valid_relationship_type(rel_type: &str) -> bool {
    VALID_RELATIONSHIP_TYPES.contains(&rel_type)
}

/// Upserts a concept relationship. If the (source, target, type) exists, updates confidence, evidence, updated_at.
pub fn upsert_concept_relationship(
    conn: &Connection,
    source_concept_id: &str,
    target_concept_id: &str,
    relationship_type: &str,
    confidence: f32,
    evidence: &str,
    source_document_id: Option<&str>,
    source_chunk_id: Option<&str>,
    discovery_method: &str,
) -> Result<ConceptRelationshipRecord> {
    if !is_valid_relationship_type(relationship_type) {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let now = Utc::now().to_rfc3339();

    // Check existing
    let existing: Result<(String, String), _> = conn.query_row(
        "SELECT id, created_at FROM concept_relationships 
         WHERE source_concept_id = ?1 AND target_concept_id = ?2 AND relationship_type = ?3;",
        params![source_concept_id, target_concept_id, relationship_type],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (id, created_at) = match existing {
        Ok((ex_id, ex_created_at)) => {
            conn.execute(
                "UPDATE concept_relationships 
                 SET confidence = ?1, evidence = ?2, source_document_id = ?3, source_chunk_id = ?4, discovery_method = ?5, updated_at = ?6
                 WHERE id = ?7;",
                params![confidence, evidence, source_document_id, source_chunk_id, discovery_method, now, ex_id],
            )?;
            (ex_id, ex_created_at)
        }
        Err(_) => {
            let new_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO concept_relationships (
                    id, source_concept_id, target_concept_id, relationship_type, confidence,
                    evidence, source_document_id, source_chunk_id, discovery_method, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);",
                params![
                    new_id,
                    source_concept_id,
                    target_concept_id,
                    relationship_type,
                    confidence,
                    evidence,
                    source_document_id,
                    source_chunk_id,
                    discovery_method,
                    now,
                    now
                ],
            )?;
            (new_id, now.clone())
        }
    };

    // Retrieve joined concept names
    let (source_name, target_name, doc_name) = conn.query_row(
        "SELECT 
            (SELECT name FROM concepts WHERE id = ?1),
            (SELECT name FROM concepts WHERE id = ?2),
            (SELECT filename FROM documents WHERE id = ?3)",
        params![source_concept_id, target_concept_id, source_document_id],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    ).unwrap_or((None, None, None));

    Ok(ConceptRelationshipRecord {
        id,
        source_concept_id: source_concept_id.to_string(),
        source_concept_name: source_name,
        target_concept_id: target_concept_id.to_string(),
        target_concept_name: target_name,
        relationship_type: relationship_type.to_string(),
        confidence,
        evidence: evidence.to_string(),
        source_document_id: source_document_id.map(|s| s.to_string()),
        source_document_name: doc_name,
        source_chunk_id: source_chunk_id.map(|s| s.to_string()),
        discovery_method: discovery_method.to_string(),
        created_at,
        updated_at: now,
    })
}

/// Retrieve all relationships with optional limit
pub fn get_all_concept_relationships(
    conn: &Connection,
    limit: Option<usize>,
) -> Result<Vec<ConceptRelationshipRecord>> {
    let limit_val = limit.unwrap_or(200);
    let mut stmt = conn.prepare(
        "SELECT 
            cr.id, cr.source_concept_id, sc.name, 
            cr.target_concept_id, tc.name,
            cr.relationship_type, cr.confidence, cr.evidence,
            cr.source_document_id, d.filename, cr.source_chunk_id,
            cr.discovery_method, cr.created_at, cr.updated_at
         FROM concept_relationships cr
         LEFT JOIN concepts sc ON cr.source_concept_id = sc.id
         LEFT JOIN concepts tc ON cr.target_concept_id = tc.id
         LEFT JOIN documents d ON cr.source_document_id = d.id
         ORDER BY cr.confidence DESC, cr.created_at DESC
         LIMIT ?1;",
    )?;

    let rows = stmt.query_map(params![limit_val], |row| {
        Ok(ConceptRelationshipRecord {
            id: row.get(0)?,
            source_concept_id: row.get(1)?,
            source_concept_name: row.get(2)?,
            target_concept_id: row.get(3)?,
            target_concept_name: row.get(4)?,
            relationship_type: row.get(5)?,
            confidence: row.get::<_, f64>(6)? as f32,
            evidence: row.get(7)?,
            source_document_id: row.get(8)?,
            source_document_name: row.get(9)?,
            source_chunk_id: row.get(10)?,
            discovery_method: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Get all direct neighbors for a concept (both outgoing and incoming)
pub fn get_concept_neighbors(
    conn: &Connection,
    concept_id: &str,
) -> Result<Vec<ConceptNeighborRecord>> {
    let mut neighbors = Vec::new();

    // 1. Outgoing: concept -> neighbor
    let mut stmt_out = conn.prepare(
        "SELECT 
            cr.id, cr.relationship_type, 'outgoing',
            cr.target_concept_id, tc.name, tc.category,
            cr.confidence, cr.evidence,
            cr.source_document_id, d.filename
         FROM concept_relationships cr
         JOIN concepts tc ON cr.target_concept_id = tc.id
         LEFT JOIN documents d ON cr.source_document_id = d.id
         WHERE cr.source_concept_id = ?1
         ORDER BY cr.confidence DESC;",
    )?;

    let rows_out = stmt_out.query_map(params![concept_id], |row| {
        Ok(ConceptNeighborRecord {
            relationship_id: row.get(0)?,
            relationship_type: row.get(1)?,
            direction: row.get(2)?,
            neighbor_concept_id: row.get(3)?,
            neighbor_concept_name: row.get(4)?,
            neighbor_category: row.get(5)?,
            confidence: row.get::<_, f64>(6)? as f32,
            evidence: row.get(7)?,
            source_document_id: row.get(8)?,
            source_document_name: row.get(9)?,
        })
    })?;

    for r in rows_out {
        neighbors.push(r?);
    }

    // 2. Incoming: neighbor -> concept
    let mut stmt_in = conn.prepare(
        "SELECT 
            cr.id, cr.relationship_type, 'incoming',
            cr.source_concept_id, sc.name, sc.category,
            cr.confidence, cr.evidence,
            cr.source_document_id, d.filename
         FROM concept_relationships cr
         JOIN concepts sc ON cr.source_concept_id = sc.id
         LEFT JOIN documents d ON cr.source_document_id = d.id
         WHERE cr.target_concept_id = ?1
         ORDER BY cr.confidence DESC;",
    )?;

    let rows_in = stmt_in.query_map(params![concept_id], |row| {
        Ok(ConceptNeighborRecord {
            relationship_id: row.get(0)?,
            relationship_type: row.get(1)?,
            direction: row.get(2)?,
            neighbor_concept_id: row.get(3)?,
            neighbor_concept_name: row.get(4)?,
            neighbor_category: row.get(5)?,
            confidence: row.get::<_, f64>(6)? as f32,
            evidence: row.get(7)?,
            source_document_id: row.get(8)?,
            source_document_name: row.get(9)?,
        })
    })?;

    for r in rows_in {
        neighbors.push(r?);
    }

    Ok(neighbors)
}

/// Get all documents linked to a concept via document_concepts
pub fn get_concept_documents(
    conn: &Connection,
    concept_id: &str,
) -> Result<Vec<ConceptDocumentRecord>> {
    let mut stmt = conn.prepare(
        "SELECT 
            dc.document_id, d.filename,
            dc.concept_id, c.name,
            dc.confidence, dc.evidence, dc.created_at
         FROM document_concepts dc
         JOIN documents d ON dc.document_id = d.id
         JOIN concepts c ON dc.concept_id = c.id
         WHERE dc.concept_id = ?1
         ORDER BY dc.confidence DESC, d.filename ASC;",
    )?;

    let rows = stmt.query_map(params![concept_id], |row| {
        Ok(ConceptDocumentRecord {
            document_id: row.get(0)?,
            document_filename: row.get(1)?,
            concept_id: row.get(2)?,
            concept_name: row.get(3)?,
            confidence: row.get::<_, f64>(4)? as f32,
            evidence: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Get all relationships discovered from a specific source document
pub fn get_document_relationships(
    conn: &Connection,
    document_id: &str,
) -> Result<Vec<ConceptRelationshipRecord>> {
    let mut stmt = conn.prepare(
        "SELECT 
            cr.id, cr.source_concept_id, sc.name, 
            cr.target_concept_id, tc.name,
            cr.relationship_type, cr.confidence, cr.evidence,
            cr.source_document_id, d.filename, cr.source_chunk_id,
            cr.discovery_method, cr.created_at, cr.updated_at
         FROM concept_relationships cr
         LEFT JOIN concepts sc ON cr.source_concept_id = sc.id
         LEFT JOIN concepts tc ON cr.target_concept_id = tc.id
         LEFT JOIN documents d ON cr.source_document_id = d.id
         WHERE cr.source_document_id = ?1
         ORDER BY cr.confidence DESC;",
    )?;

    let rows = stmt.query_map(params![document_id], |row| {
        Ok(ConceptRelationshipRecord {
            id: row.get(0)?,
            source_concept_id: row.get(1)?,
            source_concept_name: row.get(2)?,
            target_concept_id: row.get(3)?,
            target_concept_name: row.get(4)?,
            relationship_type: row.get(5)?,
            confidence: row.get::<_, f64>(6)? as f32,
            evidence: row.get(7)?,
            source_document_id: row.get(8)?,
            source_document_name: row.get(9)?,
            source_chunk_id: row.get(10)?,
            discovery_method: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Delete a concept relationship
pub fn delete_concept_relationship(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM concept_relationships WHERE id = ?1;",
        params![id],
    )?;
    Ok(())
}

/// Graph high-level statistics
pub fn get_graph_stats(conn: &Connection) -> Result<GraphStats> {
    let total_concepts: usize = conn
        .query_row("SELECT COUNT(*) FROM concepts;", [], |r| r.get(0))
        .unwrap_or(0);

    let total_relationships: usize = conn
        .query_row("SELECT COUNT(*) FROM concept_relationships;", [], |r| r.get(0))
        .unwrap_or(0);

    let connected_documents: usize = conn
        .query_row(
            "SELECT COUNT(DISTINCT document_id) FROM document_concepts;",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT relationship_type, COUNT(*) 
         FROM concept_relationships 
         GROUP BY relationship_type 
         ORDER BY COUNT(*) DESC;",
    )?;

    let type_rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut relationship_type_counts = Vec::new();
    for t in type_rows {
        relationship_type_counts.push(t?);
    }

    Ok(GraphStats {
        total_concepts,
        total_relationships,
        connected_documents,
        relationship_type_counts,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNodePositionRecord {
    pub concept_id: String,
    pub x: f64,
    pub y: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePositionInput {
    pub concept_id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphViewportRecord {
    pub zoom: f64,
    pub x: f64,
    pub y: f64,
}

/// Save batch of concept node positions (upsert)
pub fn save_node_positions(conn: &Connection, positions: &[NodePositionInput]) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO concept_node_positions (concept_id, position_x, position_y, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(concept_id) DO UPDATE SET position_x = ?2, position_y = ?3, updated_at = ?4;",
        )?;

        for pos in positions {
            stmt.execute(params![pos.concept_id, pos.x, pos.y, now])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Retrieve all saved concept node positions
pub fn get_all_node_positions(conn: &Connection) -> Result<Vec<ConceptNodePositionRecord>> {
    let mut stmt = conn.prepare(
        "SELECT concept_id, position_x, position_y, updated_at FROM concept_node_positions;",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ConceptNodePositionRecord {
            concept_id: row.get(0)?,
            x: row.get(1)?,
            y: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Delete a specific concept node position
pub fn delete_node_position(conn: &Connection, concept_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM concept_node_positions WHERE concept_id = ?1;",
        params![concept_id],
    )?;
    Ok(())
}

/// Save graph canvas viewport
pub fn save_graph_viewport(conn: &Connection, viewport: &GraphViewportRecord) -> Result<()> {
    let json_val = serde_json::to_string(viewport)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    crate::database::set_app_setting(conn, "graph_viewport", &json_val)
}

/// Retrieve saved graph canvas viewport
pub fn get_graph_viewport(conn: &Connection) -> Result<Option<GraphViewportRecord>> {
    let val_opt = crate::database::get_app_setting(conn, "graph_viewport")?;
    match val_opt {
        Some(val) => {
            let parsed = serde_json::from_str::<GraphViewportRecord>(&val).ok();
            Ok(parsed)
        }
        None => Ok(None),
    }
}


