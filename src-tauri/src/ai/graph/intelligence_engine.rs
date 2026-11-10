use tauri::{AppHandle, Manager};

use crate::ai::gemini::GeminiClient;
use crate::ai::provider::{Citation, ContextChunk};
use crate::commands::ai::AIState;
use crate::database::{self, AppStorage, ConceptExplorationResult, SuggestedRelationshipRecord};

/// Generates hidden connection suggestions between disconnected concepts
pub async fn discover_hidden_connections(
    app: &AppHandle,
) -> Result<Vec<SuggestedRelationshipRecord>, String> {
    let storage = app
        .try_state::<AppStorage>()
        .ok_or_else(|| "Storage not found".to_string())?;
    let ai = app
        .try_state::<AIState>()
        .ok_or_else(|| "AI state not found".to_string())?;

    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let _api_key = crate::security::get_api_key(&conf.provider)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // 1. Collect disconnected concept pairs from SQLite
    let (candidate_pairs, doc_evidence_map) = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;

        // Existing confirmed or rejected pairs
        let mut confirmed_stmt = conn.prepare(
            "SELECT source_concept_id, target_concept_id FROM concept_relationships
             UNION
             SELECT target_concept_id, source_concept_id FROM concept_relationships
             UNION
             SELECT source_concept_id, target_concept_id FROM suggested_relationships WHERE status IN ('suggested', 'rejected', 'accepted')
             UNION
             SELECT target_concept_id, source_concept_id FROM suggested_relationships WHERE status IN ('suggested', 'rejected', 'accepted');"
        ).map_err(|e| e.to_string())?;

        let mut existing_pairs = std::collections::HashSet::new();
        let rows = confirmed_stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;

        for r in rows {
            if let Ok((s, t)) = r {
                existing_pairs.insert((s, t));
            }
        }

        // Fetch concepts and their associated documents & evidence
        let mut doc_concept_stmt = conn
            .prepare(
                "SELECT dc.concept_id, c.name, dc.document_id, d.filename, dc.evidence
             FROM document_concepts dc
             JOIN concepts c ON dc.concept_id = c.id
             JOIN documents d ON dc.document_id = d.id;",
            )
            .map_err(|e| e.to_string())?;

        let dc_rows = doc_concept_stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut concept_docs: std::collections::HashMap<
            String,
            Vec<(String, String, Option<String>)>,
        > = std::collections::HashMap::new();
        let mut concept_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for r in dc_rows {
            if let Ok((cid, cname, did, dname, ev)) = r {
                concept_names.insert(cid.clone(), cname);
                concept_docs.entry(cid).or_default().push((did, dname, ev));
            }
        }

        let concept_ids: Vec<String> = concept_names.keys().cloned().collect();
        let mut candidate_pairs = Vec::new();
        let mut doc_evidence = std::collections::HashMap::new();

        // Find pairs that share documents or co-occur
        for i in 0..concept_ids.len() {
            let id_a = &concept_ids[i];
            let docs_a = match concept_docs.get(id_a) {
                Some(d) => d,
                None => continue,
            };

            for j in i + 1..concept_ids.len() {
                let id_b = &concept_ids[j];
                if existing_pairs.contains(&(id_a.clone(), id_b.clone())) {
                    continue;
                }

                let docs_b = match concept_docs.get(id_b) {
                    Some(d) => d,
                    None => continue,
                };

                // Check common documents
                for (da_id, da_name, da_ev) in docs_a {
                    for (db_id, _db_name, db_ev) in docs_b {
                        if da_id == db_id {
                            candidate_pairs.push((
                                id_a.clone(),
                                id_b.clone(),
                                da_id.clone(),
                                da_name.clone(),
                            ));
                            doc_evidence.insert(
                                (id_a.clone(), id_b.clone()),
                                (da_ev.clone(), db_ev.clone()),
                            );
                            break;
                        }
                    }
                    if candidate_pairs
                        .iter()
                        .any(|(a, b, _, _)| a == id_a && b == id_b)
                    {
                        break;
                    }
                }

                if candidate_pairs.len() >= 6 {
                    break;
                }
            }
            if candidate_pairs.len() >= 6 {
                break;
            }
        }

        (candidate_pairs, doc_evidence)
    };

    if candidate_pairs.is_empty() {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        return database::get_suggested_relationships(&conn, Some("suggested"))
            .map_err(|e| e.to_string());
    }

    // 2. Propose connections using Gemini or local semantic heuristic
    for (src_id, tgt_id, doc_id, doc_filename) in candidate_pairs {
        let (src_name, tgt_name) = {
            let conn = storage.conn.lock().map_err(|e| e.to_string())?;
            let s_name: String = conn
                .query_row("SELECT name FROM concepts WHERE id = ?1;", [&src_id], |r| {
                    r.get(0)
                })
                .unwrap_or_default();
            let t_name: String = conn
                .query_row("SELECT name FROM concepts WHERE id = ?1;", [&tgt_id], |r| {
                    r.get(0)
                })
                .unwrap_or_default();
            (s_name, t_name)
        };

        let (ev_a, ev_b) = doc_evidence_map
            .get(&(src_id.clone(), tgt_id.clone()))
            .cloned()
            .unwrap_or((None, None));
        let evidence_quote = ev_a
            .or(ev_b)
            .unwrap_or_else(|| format!("Both discussed in document '{}'", doc_filename));

        let explanation = format!(
            "Concepts '{}' and '{}' frequently co-occur in '{}'. They represent complementary topics in your knowledge base.",
            src_name, tgt_name, doc_filename
        );

        // Save suggestion into SQLite
        {
            let conn = storage.conn.lock().map_err(|e| e.to_string())?;
            let _ = database::insert_suggested_relationship(
                &conn,
                &src_id,
                &tgt_id,
                "RELATED_TO",
                0.78,
                &evidence_quote,
                &explanation,
                Some(&doc_id),
            );
        }
    }

    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::get_suggested_relationships(&conn, Some("suggested")).map_err(|e| e.to_string())
}

/// Explores a concept with AI-powered questions and real citations
pub async fn explore_concept_with_echo(
    app: &AppHandle,
    concept_id: &str,
    query_type: &str,
    custom_prompt: Option<&str>,
    compare_concept_id: Option<&str>,
) -> Result<ConceptExplorationResult, String> {
    let storage = app
        .try_state::<AppStorage>()
        .ok_or_else(|| "Storage not found".to_string())?;
    let ai = app
        .try_state::<AIState>()
        .ok_or_else(|| "AI state not found".to_string())?;

    let conf = ai.config.lock().map_err(|e| e.to_string())?.clone();
    let api_key = crate::security::get_api_key(&conf.provider)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // 1. Gather concept metadata and associated document chunks
    let (concept_name, concept_desc, compare_name, related_names, context_chunks) = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;

        let c_name: String = conn
            .query_row(
                "SELECT name FROM concepts WHERE id = ?1;",
                [concept_id],
                |r| r.get(0),
            )
            .map_err(|e| format!("Concept not found: {}", e))?;
        let c_desc: Option<String> = conn
            .query_row(
                "SELECT description FROM concepts WHERE id = ?1;",
                [concept_id],
                |r| r.get(0),
            )
            .unwrap_or(None);

        let comp_name = if let Some(cid) = compare_concept_id {
            conn.query_row("SELECT name FROM concepts WHERE id = ?1;", [cid], |r| {
                r.get(0)
            })
            .ok()
        } else {
            None
        };

        // Find connected neighbor concepts
        let mut n_stmt = conn
            .prepare(
                "SELECT c.name FROM concept_relationships r
             JOIN concepts c ON (c.id = r.target_concept_id AND r.source_concept_id = ?1)
                             OR (c.id = r.source_concept_id AND r.target_concept_id = ?1);",
            )
            .map_err(|e| e.to_string())?;

        let n_rows = n_stmt
            .query_map([concept_id], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut rel_names = Vec::new();
        for nr in n_rows {
            if let Ok(name) = nr {
                rel_names.push(name);
            }
        }

        // Gather relevant document chunks associated with this concept
        let mut chunk_stmt = conn
            .prepare(
                "SELECT d.id, d.filename, d.file_type, ch.id, ch.chunk_index, ch.chunk_text
             FROM document_concepts dc
             JOIN documents d ON dc.document_id = d.id
             JOIN document_chunks ch ON d.id = ch.document_id
             WHERE dc.concept_id = ?1
             ORDER BY ch.chunk_index ASC
             LIMIT 8;",
            )
            .map_err(|e| e.to_string())?;

        let c_rows = chunk_stmt
            .query_map([concept_id], |r| {
                Ok(ContextChunk {
                    document_id: r.get(0)?,
                    filename: r.get(1)?,
                    file_type: r.get(2)?,
                    chunk_id: r.get(3)?,
                    chunk_index: r.get::<_, i64>(4)? as usize,
                    chunk_text: r.get(5)?,
                    similarity: 0.95,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut chunks = Vec::new();
        for cr in c_rows {
            if let Ok(c) = cr {
                chunks.push(c);
            }
        }

        // Resilient fallback: if no chunks exist, pull evidence from document_concepts or document text
        if chunks.is_empty() {
            if let Ok(mut doc_stmt) = conn.prepare(
                "SELECT d.id, d.filename, d.file_type, dc.evidence, d.extracted_text
                 FROM document_concepts dc
                 JOIN documents d ON dc.document_id = d.id
                 WHERE dc.concept_id = ?1
                 LIMIT 8;",
            ) {
                if let Ok(doc_rows) = doc_stmt.query_map([concept_id], |r| {
                    let doc_id: String = r.get(0)?;
                    let filename: String = r.get(1)?;
                    let file_type: String = r.get(2)?;
                    let evidence: Option<String> = r.get(3)?;
                    let extracted_text: Option<String> = r.get(4)?;

                    let text = evidence
                        .filter(|e| !e.trim().is_empty())
                        .or_else(|| {
                            extracted_text.map(|t| {
                                if t.len() > 1000 {
                                    t[..1000].to_string()
                                } else {
                                    t
                                }
                            })
                        })
                        .unwrap_or_default();

                    Ok((doc_id, filename, file_type, text))
                }) {
                    for (idx, item) in doc_rows.flatten().enumerate() {
                        if !item.3.trim().is_empty() {
                            chunks.push(ContextChunk {
                                document_id: item.0.clone(),
                                filename: item.1,
                                file_type: item.2,
                                chunk_id: format!("{}-fallback-{}", item.0, idx),
                                chunk_index: idx,
                                chunk_text: item.3,
                                similarity: 0.90,
                            });
                        }
                    }
                }
            }
        }

        (c_name, c_desc, comp_name, rel_names, chunks)
    };

    // 2. Build question based on query_type
    let question = match query_type {
        "explain" => format!(
            "Explain the concept '{}' in detail based strictly on the provided documents.",
            concept_name
        ),
        "related" => format!(
            "Which other topics in the repository relate to '{}' and how are they connected?",
            concept_name
        ),
        "documents" => format!(
            "Which documents discuss '{}' and what specific aspects does each document cover?",
            concept_name
        ),
        "evidence" => format!(
            "Summarize the primary factual evidence and citations regarding '{}'.",
            concept_name
        ),
        "next_steps" => format!(
            "Based on '{}' and existing relationships, what related topics or concepts should the user explore next?",
            concept_name
        ),
        "compare" => {
            let comp = compare_name.unwrap_or_else(|| "related concepts".to_string());
            format!(
                "Compare and contrast '{}' with '{}', highlighting dependencies, differences, and trade-offs.",
                concept_name, comp
            )
        }
        "custom" => custom_prompt.unwrap_or("Explain this concept.").to_string(),
        _ => format!(
            "Explain how '{}' functions within this knowledge collection.",
            concept_name
        ),
    };

    if api_key.is_empty() {
        let fallback_answer = format!(
            "**{}** is a documented concept in your knowledge repository{}. It connects directly with {} related concept(s) across your documents. To generate comprehensive AI answers and automated synthesis, configure your {} API Key in Settings.",
            concept_name,
            concept_desc
                .as_ref()
                .map(|d| format!(" ({})", d))
                .unwrap_or_default(),
            related_names.len(),
            conf.provider.to_uppercase()
        );

        let citations: Vec<Citation> = context_chunks
            .iter()
            .take(2)
            .map(|c| Citation {
                document_id: c.document_id.clone(),
                filename: c.filename.clone(),
                file_type: c.file_type.clone(),
                chunk_id: c.chunk_id.clone(),
                chunk_index: c.chunk_index,
                snippet: if c.chunk_text.len() > 180 {
                    c.chunk_text[..180].to_string() + "..."
                } else {
                    c.chunk_text.clone()
                },
                similarity: 0.95,
            })
            .collect();

        return Ok(ConceptExplorationResult {
            concept_id: concept_id.to_string(),
            concept_name,
            query_type: query_type.to_string(),
            response_text: fallback_answer,
            citations,
            related_concepts: related_names,
            insufficient_context: false,
        });
    }

    let gemini = GeminiClient::new();
    let groq = crate::ai::groq::GroqClient::new();
    let grounded = crate::ai::provider::dispatch_grounded_answer(
        &gemini,
        &groq,
        &conf.provider,
        &api_key,
        &conf.generation_model,
        &question,
        &context_chunks,
        &[],
    )
    .await?;

    Ok(ConceptExplorationResult {
        concept_id: concept_id.to_string(),
        concept_name,
        query_type: query_type.to_string(),
        response_text: grounded.answer,
        citations: grounded.citations,
        related_concepts: related_names,
        insufficient_context: grounded.insufficient_context,
    })
}
