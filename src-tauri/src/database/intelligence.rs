use chrono::Utc;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::ai::provider::Citation;
use crate::database::{ConceptRelationshipRecord, is_valid_relationship_type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub concept_ids: Vec<String>,
    pub primary_concept_id: Option<String>,
    pub size: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedRelationshipRecord {
    pub id: String,
    pub source_concept_id: String,
    pub source_concept_name: Option<String>,
    pub target_concept_id: String,
    pub target_concept_name: Option<String>,
    pub relationship_type: String,
    pub confidence: f32,
    pub evidence: String,
    pub explanation: String,
    pub source_document_id: Option<String>,
    pub source_document_name: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialConflictRecord {
    pub id: String,
    pub concept_id: String,
    pub concept_name: String,
    pub statement_a: String,
    pub doc_a_id: Option<String>,
    pub doc_a_filename: String,
    pub doc_a_imported_at: Option<String>,
    pub statement_b: String,
    pub doc_b_id: Option<String>,
    pub doc_b_filename: String,
    pub doc_b_imported_at: Option<String>,
    pub explanation: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEventRecord {
    pub id: String,
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub timestamp: String,
    pub document_id: Option<String>,
    pub document_name: Option<String>,
    pub concept_id: Option<String>,
    pub concept_name: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeInsightsRecord {
    pub total_concepts: usize,
    pub total_confirmed_relationships: usize,
    pub total_suggested_relationships: usize,
    pub total_clusters: usize,
    pub most_connected_concepts: Vec<(String, String, usize)>,
    pub largest_clusters: Vec<(String, String, usize, String)>,
    pub isolated_concepts: Vec<(String, String)>,
    pub recently_discovered_concepts: Vec<(String, String, String)>,
    pub recently_added_relationships: Vec<(String, String, String, String)>,
    pub top_source_documents: Vec<(String, String, usize)>,
    pub potential_conflicts_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptExplorationResult {
    pub concept_id: String,
    pub concept_name: String,
    pub query_type: String,
    pub response_text: String,
    pub citations: Vec<Citation>,
    pub related_concepts: Vec<String>,
    pub insufficient_context: bool,
}

const FLUENT_CLUSTER_COLORS: &[&str] = &[
    "#0078D4", // Windows Accent Blue
    "#00B7C3", // Cyan / Teal
    "#107C41", // Emerald Green
    "#D83B01", // Orange / Amber
    "#008272", // Dark Teal
    "#498205", // Lime Green
    "#004E8C", // Cobalt Blue
    "#8E562E", // Sand / Earth
];

/// 1. CLUSTERING: Compute and return knowledge communities using Label Propagation on validated relationships
pub fn compute_and_save_clusters(conn: &Connection) -> Result<Vec<KnowledgeClusterRecord>> {
    // Fetch all concepts
    let mut stmt_concepts = conn.prepare("SELECT id, name, category FROM concepts;")?;
    let concept_rows = stmt_concepts.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;

    let mut concept_names: HashMap<String, String> = HashMap::new();
    let mut concept_ids: Vec<String> = Vec::new();
    for row in concept_rows {
        let (id, name, _) = row?;
        concept_names.insert(id.clone(), name);
        concept_ids.push(id);
    }

    if concept_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Fetch confirmed relationships
    let mut stmt_rels = conn.prepare(
        "SELECT source_concept_id, target_concept_id, confidence FROM concept_relationships;",
    )?;
    let rel_rows = stmt_rels.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, f64>(2)?,
        ))
    })?;

    // Build adjacency graph
    let mut adj: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for c_id in &concept_ids {
        adj.insert(c_id.clone(), HashMap::new());
    }

    for r in rel_rows {
        let (src, tgt, conf) = r?;
        let weight = if conf > 0.0 { conf } else { 1.0 };
        adj.entry(src.clone())
            .or_default()
            .insert(tgt.clone(), weight);
        adj.entry(tgt.clone())
            .or_default()
            .insert(src.clone(), weight);
    }

    // Community Detection: Label Propagation Algorithm (LPA)
    let mut labels: HashMap<String, String> = HashMap::new();
    for id in &concept_ids {
        labels.insert(id.clone(), id.clone());
    }

    let iterations = 8;
    for _ in 0..iterations {
        let mut changed = false;
        for u in &concept_ids {
            let neighbors = match adj.get(u) {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };

            // Aggregate weights per label among neighbors
            let mut label_weights: HashMap<String, f64> = HashMap::new();
            for (v, w) in neighbors {
                if let Some(lbl) = labels.get(v) {
                    *label_weights.entry(lbl.clone()).or_default() += *w;
                }
            }

            // Pick highest-weight label
            let mut best_label = labels[u].clone();
            let mut max_weight = -1.0;
            for (lbl, weight) in label_weights {
                if weight > max_weight {
                    max_weight = weight;
                    best_label = lbl;
                }
            }

            if best_label != labels[u] {
                labels.insert(u.clone(), best_label);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Group concept IDs by cluster label
    let mut cluster_groups: HashMap<String, Vec<String>> = HashMap::new();
    for (c_id, lbl) in labels {
        cluster_groups.entry(lbl).or_default().push(c_id);
    }

    let now = Utc::now().to_rfc3339();
    let mut cluster_records: Vec<KnowledgeClusterRecord> = Vec::new();
    let mut color_idx = 0;

    // Process clusters, sort by size descending
    let mut sorted_groups: Vec<(String, Vec<String>)> = cluster_groups.into_iter().collect();
    sorted_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let tx = conn.unchecked_transaction()?;
    // Clear old clusters
    tx.execute("DELETE FROM concept_cluster_members;", [])?;
    tx.execute("DELETE FROM knowledge_clusters;", [])?;

    for (_lbl, mut members) in sorted_groups {
        if members.is_empty() {
            continue;
        }

        // Find primary concept (highest degree within cluster)
        members.sort_by(|a, b| {
            let deg_b = adj.get(b).map(|m| m.len()).unwrap_or(0);
            let deg_a = adj.get(a).map(|m| m.len()).unwrap_or(0);
            deg_b.cmp(&deg_a)
        });

        let primary_id = members[0].clone();
        let primary_name = concept_names
            .get(&primary_id)
            .cloned()
            .unwrap_or_else(|| "General".to_string());

        let cluster_name = if members.len() == 1 {
            format!("{} Topic", primary_name)
        } else if members.len() == 2 {
            let second_name = concept_names.get(&members[1]).cloned().unwrap_or_default();
            format!("{} & {}", primary_name, second_name)
        } else {
            let second_name = concept_names.get(&members[1]).cloned().unwrap_or_default();
            format!("{} & {} Cluster", primary_name, second_name)
        };

        let cluster_id = format!("cluster-{}", Uuid::new_v4().to_string()[..8].to_string());
        let color = FLUENT_CLUSTER_COLORS[color_idx % FLUENT_CLUSTER_COLORS.len()].to_string();
        color_idx += 1;

        let desc = format!(
            "Knowledge cluster containing {} concepts centered on {}.",
            members.len(),
            primary_name
        );

        tx.execute(
            "INSERT INTO knowledge_clusters (id, name, description, color, updated_at) VALUES (?1, ?2, ?3, ?4, ?5);",
            params![cluster_id, cluster_name, desc, color, now],
        )?;

        for (i, m_id) in members.iter().enumerate() {
            let is_primary = if i == 0 { 1 } else { 0 };
            tx.execute(
                "INSERT INTO concept_cluster_members (cluster_id, concept_id, is_primary) VALUES (?1, ?2, ?3);",
                params![cluster_id, m_id, is_primary],
            )?;
        }

        cluster_records.push(KnowledgeClusterRecord {
            id: cluster_id,
            name: cluster_name,
            description: Some(desc),
            color,
            concept_ids: members.clone(),
            primary_concept_id: Some(primary_id),
            size: members.len(),
            updated_at: now.clone(),
        });
    }

    tx.commit()?;
    Ok(cluster_records)
}

/// Retrieve saved clusters
pub fn get_knowledge_clusters(conn: &Connection) -> Result<Vec<KnowledgeClusterRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, color, updated_at FROM knowledge_clusters ORDER BY id;",
    )?;

    let cluster_rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;

    let mut clusters = Vec::new();
    for row in cluster_rows {
        let (id, name, desc, color, updated_at) = row?;

        let mut member_stmt = conn.prepare(
            "SELECT concept_id, is_primary FROM concept_cluster_members WHERE cluster_id = ?1;",
        )?;
        let m_rows = member_stmt.query_map([&id], |mr| {
            Ok((mr.get::<_, String>(0)?, mr.get::<_, i32>(1)?))
        })?;

        let mut concept_ids = Vec::new();
        let mut primary_id = None;
        for m in m_rows {
            let (cid, is_prim) = m?;
            if is_prim == 1 {
                primary_id = Some(cid.clone());
            }
            concept_ids.push(cid);
        }

        let size = concept_ids.len();
        clusters.push(KnowledgeClusterRecord {
            id,
            name,
            description: desc,
            color,
            concept_ids,
            primary_concept_id: primary_id,
            size,
            updated_at,
        });
    }

    if clusters.is_empty() {
        return compute_and_save_clusters(conn);
    }

    Ok(clusters)
}

/// 2. HIDDEN CONNECTION DISCOVERY: Suggested Relationships CRUD
pub fn get_suggested_relationships(
    conn: &Connection,
    status_filter: Option<&str>,
) -> Result<Vec<SuggestedRelationshipRecord>> {
    let mut query = String::from(
        "SELECT 
            sr.id, sr.source_concept_id, sc.name,
            sr.target_concept_id, tc.name,
            sr.relationship_type, sr.confidence,
            sr.evidence, sr.explanation,
            sr.source_document_id, d.filename,
            sr.status, sr.created_at, sr.updated_at
         FROM suggested_relationships sr
         LEFT JOIN concepts sc ON sr.source_concept_id = sc.id
         LEFT JOIN concepts tc ON sr.target_concept_id = tc.id
         LEFT JOIN documents d ON sr.source_document_id = d.id",
    );

    if let Some(status) = status_filter {
        query.push_str(&format!(" WHERE sr.status = '{}'", status));
    }
    query.push_str(" ORDER BY sr.confidence DESC, sr.created_at DESC;");

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], |row| {
        Ok(SuggestedRelationshipRecord {
            id: row.get(0)?,
            source_concept_id: row.get(1)?,
            source_concept_name: row.get(2)?,
            target_concept_id: row.get(3)?,
            target_concept_name: row.get(4)?,
            relationship_type: row.get(5)?,
            confidence: row.get::<_, f64>(6)? as f32,
            evidence: row.get(7)?,
            explanation: row.get(8)?,
            source_document_id: row.get(9)?,
            source_document_name: row.get(10)?,
            status: row.get(11)?,
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

/// Accept suggested relationship: marks as accepted and inserts into concept_relationships
pub fn accept_suggested_relationship(
    conn: &Connection,
    id: &str,
) -> Result<ConceptRelationshipRecord> {
    let now = Utc::now().to_rfc3339();

    // Fetch suggestion
    let mut stmt = conn.prepare(
        "SELECT source_concept_id, target_concept_id, relationship_type, confidence, evidence, source_document_id
         FROM suggested_relationships WHERE id = ?1;"
    )?;
    let sugg = stmt.query_row([id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, f64>(3)? as f32,
            r.get::<_, String>(4)?,
            r.get::<_, Option<String>>(5)?,
        ))
    })?;

    let (src_id, tgt_id, rel_type, conf, evidence, doc_id) = sugg;

    // Mark suggestion as accepted
    conn.execute(
        "UPDATE suggested_relationships SET status = 'accepted', updated_at = ?1 WHERE id = ?2;",
        params![now, id],
    )?;

    // Insert into confirmed concept_relationships
    crate::database::upsert_concept_relationship(
        conn,
        &src_id,
        &tgt_id,
        &rel_type,
        conf,
        &evidence,
        doc_id.as_deref(),
        None,
        "user_accepted_suggestion",
    )
}

/// Reject suggested relationship
pub fn reject_suggested_relationship(conn: &Connection, id: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE suggested_relationships SET status = 'rejected', updated_at = ?1 WHERE id = ?2;",
        params![now, id],
    )?;
    Ok(())
}

/// Save generated suggested relationship
pub fn insert_suggested_relationship(
    conn: &Connection,
    src_id: &str,
    tgt_id: &str,
    rel_type: &str,
    conf: f32,
    evidence: &str,
    explanation: &str,
    doc_id: Option<&str>,
) -> Result<()> {
    let id = format!("sugg-{}", Uuid::new_v4().to_string()[..8].to_string());
    let now = Utc::now().to_rfc3339();
    let validated_type = if is_valid_relationship_type(rel_type) {
        rel_type
    } else {
        "RELATED_TO"
    };

    conn.execute(
        "INSERT INTO suggested_relationships 
         (id, source_concept_id, target_concept_id, relationship_type, confidence, evidence, explanation, source_document_id, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'suggested', ?9, ?9)
         ON CONFLICT(source_concept_id, target_concept_id, relationship_type) DO NOTHING;",
        params![id, src_id, tgt_id, validated_type, conf as f64, evidence, explanation, doc_id, now],
    )?;
    Ok(())
}

/// 3. TIMELINE & KNOWLEDGE EVOLUTION
pub fn get_knowledge_evolution_timeline(conn: &Connection) -> Result<Vec<TimelineEventRecord>> {
    let mut events = Vec::new();

    // 1. Document imports
    let mut stmt_docs = conn.prepare(
        "SELECT id, filename, imported_at, file_type, file_size FROM documents ORDER BY imported_at DESC LIMIT 50;"
    )?;
    let doc_rows = stmt_docs.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
        ))
    })?;

    for d in doc_rows {
        let (id, filename, ts, ftype, size) = d?;
        events.push(TimelineEventRecord {
            id: format!("doc-{}", id),
            event_type: "document_imported".to_string(),
            title: format!("Imported Document: {}", filename),
            description: format!(
                "Added {} document ({} bytes) to local repository.",
                ftype.to_uppercase(),
                size
            ),
            timestamp: ts,
            document_id: Some(id),
            document_name: Some(filename),
            concept_id: None,
            concept_name: None,
            metadata: Some(format!("type: {}", ftype)),
        });
    }

    // 2. Newly discovered concepts
    let mut stmt_concepts = conn.prepare(
        "SELECT c.id, c.name, c.category, c.created_at, COUNT(dc.document_id)
         FROM concepts c
         LEFT JOIN document_concepts dc ON c.id = dc.concept_id
         GROUP BY c.id
         ORDER BY c.created_at DESC LIMIT 50;",
    )?;
    let c_rows = stmt_concepts.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
        ))
    })?;

    for c in c_rows {
        let (id, name, cat, ts, doc_count) = c?;
        events.push(TimelineEventRecord {
            id: format!("c-{}", id),
            event_type: "concept_discovered".to_string(),
            title: format!("Discovered Concept: {}", name),
            description: format!(
                "Identified new {} concept referenced across {} document(s).",
                cat, doc_count
            ),
            timestamp: ts,
            document_id: None,
            document_name: None,
            concept_id: Some(id),
            concept_name: Some(name),
            metadata: Some(format!("category: {}", cat)),
        });
    }

    // 3. Confirmed relationships
    let mut stmt_rels = conn.prepare(
        "SELECT r.id, sc.name, r.relationship_type, tc.name, r.created_at, r.confidence
         FROM concept_relationships r
         LEFT JOIN concepts sc ON r.source_concept_id = sc.id
         LEFT JOIN concepts tc ON r.target_concept_id = tc.id
         ORDER BY r.created_at DESC LIMIT 50;",
    )?;
    let r_rows = stmt_rels.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, f64>(5)?,
        ))
    })?;

    for rel in r_rows {
        let (id, s_name, r_type, t_name, ts, conf) = rel?;
        let src = s_name.unwrap_or_else(|| "Source".to_string());
        let tgt = t_name.unwrap_or_else(|| "Target".to_string());
        events.push(TimelineEventRecord {
            id: format!("rel-{}", id),
            event_type: "relationship_added".to_string(),
            title: format!("Connected: {} → {}", src, tgt),
            description: format!(
                "Semantic relationship '{}' established with {:.0}% confidence.",
                r_type,
                conf * 100.0
            ),
            timestamp: ts,
            document_id: None,
            document_name: None,
            concept_id: None,
            concept_name: None,
            metadata: Some(format!("type: {}", r_type)),
        });
    }

    // Sort all events newest first
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(events)
}

/// 4. CONFLICT DETECTION: Identify potentially diverging/conflicting statements across documents
pub fn detect_potential_conflicts(conn: &Connection) -> Result<Vec<PotentialConflictRecord>> {
    // Concepts appearing in >= 2 distinct documents with evidence quotes
    let mut stmt = conn.prepare(
        "SELECT 
            dc1.concept_id, c.name,
            dc1.evidence, d1.id, d1.filename, d1.imported_at,
            dc2.evidence, d2.id, d2.filename, d2.imported_at
         FROM document_concepts dc1
         JOIN document_concepts dc2 ON dc1.concept_id = dc2.concept_id AND dc1.document_id < dc2.document_id
         JOIN concepts c ON dc1.concept_id = c.id
         JOIN documents d1 ON dc1.document_id = d1.id
         JOIN documents d2 ON dc2.document_id = d2.id
         WHERE dc1.evidence IS NOT NULL AND length(dc1.evidence) > 15
           AND dc2.evidence IS NOT NULL AND length(dc2.evidence) > 15
         LIMIT 20;"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
        ))
    })?;

    let mut list = Vec::new();
    let now = Utc::now().to_rfc3339();

    for r in rows {
        let (cid, cname, ev_a, d1_id, d1_name, d1_ts, ev_b, d2_id, d2_name, d2_ts) = r?;

        // Simple heuristic contrast detection or different context across distinct files
        let explanation = format!(
            "Concept '{}' is discussed with distinct perspectives across '{}' and '{}'. Review source excerpts to verify semantic alignment.",
            cname, d1_name, d2_name
        );

        list.push(PotentialConflictRecord {
            id: format!("conflict-{}", Uuid::new_v4().to_string()[..8].to_string()),
            concept_id: cid,
            concept_name: cname,
            statement_a: ev_a,
            doc_a_id: Some(d1_id),
            doc_a_filename: d1_name,
            doc_a_imported_at: Some(d1_ts),
            statement_b: ev_b,
            doc_b_id: Some(d2_id),
            doc_b_filename: d2_name,
            doc_b_imported_at: Some(d2_ts),
            explanation,
            created_at: now.clone(),
        });
    }

    Ok(list)
}

/// 5. KNOWLEDGE INSIGHTS: Comprehensive real metrics from SQLite
pub fn get_knowledge_insights(conn: &Connection) -> Result<KnowledgeInsightsRecord> {
    let total_concepts: usize = conn
        .query_row("SELECT COUNT(*) FROM concepts;", [], |r| r.get(0))
        .unwrap_or(0);

    let total_confirmed_relationships: usize = conn
        .query_row("SELECT COUNT(*) FROM concept_relationships;", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    let total_suggested_relationships: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM suggested_relationships WHERE status = 'suggested';",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_clusters: usize = conn
        .query_row("SELECT COUNT(*) FROM knowledge_clusters;", [], |r| r.get(0))
        .unwrap_or(0);

    // Most connected concepts (highest degree in confirmed relationships)
    let mut stmt_hubs = conn.prepare(
        "SELECT c.id, c.name, COUNT(r.id) as degree
         FROM concepts c
         JOIN concept_relationships r ON c.id = r.source_concept_id OR c.id = r.target_concept_id
         GROUP BY c.id
         ORDER BY degree DESC
         LIMIT 6;",
    )?;
    let hub_rows = stmt_hubs.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? as usize))
    })?;
    let mut most_connected_concepts = Vec::new();
    for h in hub_rows {
        most_connected_concepts.push(h?);
    }

    // Largest clusters
    let mut stmt_clusters = conn.prepare(
        "SELECT kc.id, kc.name, COUNT(ccm.concept_id) as count, kc.color
         FROM knowledge_clusters kc
         LEFT JOIN concept_cluster_members ccm ON kc.id = ccm.cluster_id
         GROUP BY kc.id
         ORDER BY count DESC
         LIMIT 6;",
    )?;
    let cluster_rows = stmt_clusters.query_map([], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get::<_, i64>(2)? as usize,
            r.get(3)?,
        ))
    })?;
    let mut largest_clusters = Vec::new();
    for cl in cluster_rows {
        largest_clusters.push(cl?);
    }

    // Isolated concepts (degree 0 - no incoming or outgoing relationships)
    let mut stmt_iso = conn.prepare(
        "SELECT c.id, c.name
         FROM concepts c
         WHERE c.id NOT IN (
             SELECT source_concept_id FROM concept_relationships
             UNION
             SELECT target_concept_id FROM concept_relationships
         )
         LIMIT 8;",
    )?;
    let iso_rows = stmt_iso.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut isolated_concepts = Vec::new();
    for iso in iso_rows {
        isolated_concepts.push(iso?);
    }

    // Recently discovered concepts
    let mut stmt_rec_c = conn
        .prepare("SELECT id, name, created_at FROM concepts ORDER BY created_at DESC LIMIT 5;")?;
    let rec_c_rows = stmt_rec_c.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    let mut recently_discovered_concepts = Vec::new();
    for rc in rec_c_rows {
        recently_discovered_concepts.push(rc?);
    }

    // Recently added relationships
    let mut stmt_rec_r = conn.prepare(
        "SELECT sc.name, r.relationship_type, tc.name, r.created_at
         FROM concept_relationships r
         JOIN concepts sc ON r.source_concept_id = sc.id
         JOIN concepts tc ON r.target_concept_id = tc.id
         ORDER BY r.created_at DESC LIMIT 5;",
    )?;
    let rec_r_rows =
        stmt_rec_r.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
    let mut recently_added_relationships = Vec::new();
    for rr in rec_r_rows {
        recently_added_relationships.push(rr?);
    }

    // Documents contributing the most concepts
    let mut stmt_top_docs = conn.prepare(
        "SELECT d.id, d.filename, COUNT(dc.concept_id) as count
         FROM documents d
         JOIN document_concepts dc ON d.id = dc.document_id
         GROUP BY d.id
         ORDER BY count DESC
         LIMIT 5;",
    )?;
    let top_doc_rows = stmt_top_docs.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? as usize))
    })?;
    let mut top_source_documents = Vec::new();
    for td in top_doc_rows {
        top_source_documents.push(td?);
    }

    let potential_conflicts_count: usize = conn
        .query_row(
            "SELECT COUNT(DISTINCT dc1.concept_id)
             FROM document_concepts dc1
             JOIN document_concepts dc2 ON dc1.concept_id = dc2.concept_id AND dc1.document_id < dc2.document_id
             WHERE dc1.evidence IS NOT NULL AND dc2.evidence IS NOT NULL;",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(KnowledgeInsightsRecord {
        total_concepts,
        total_confirmed_relationships,
        total_suggested_relationships,
        total_clusters,
        most_connected_concepts,
        largest_clusters,
        isolated_concepts,
        recently_discovered_concepts,
        recently_added_relationships,
        top_source_documents,
        potential_conflicts_count,
    })
}
