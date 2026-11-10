use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::ai::gemini::{DEFAULT_GENERATION_MODEL, GeminiClient};
use crate::ai::tagging::service::prepare_representative_sample;
use crate::database::{
    self, ConceptRelationshipRecord, DocumentConceptRecord, is_valid_relationship_type,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRelationshipItem {
    pub source: Option<String>,
    pub target: Option<String>,
    pub relationship_type: Option<String>,
    pub confidence: Option<f64>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiRelationshipResponse {
    pub relationships: Vec<RawRelationshipItem>,
}

/// Calls Gemini to extract relationships without holding any DB connection
pub async fn extract_relationships_with_gemini(
    _client: &GeminiClient,
    api_key: &str,
    model: &str,
    doc_filename: &str,
    doc_text: &str,
    doc_concepts: &[DocumentConceptRecord],
) -> Result<Vec<RawRelationshipItem>, String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err("Gemini API key is not configured".to_string());
    }

    if doc_concepts.len() < 2 {
        return Ok(Vec::new());
    }

    let mut concepts_prompt_list = String::new();
    for c in doc_concepts {
        concepts_prompt_list.push_str(&format!(
            "- ID: \"{}\" | Name: \"{}\" | Category: \"{}\"\n",
            c.concept_id, c.name, c.category
        ));
    }

    let sample_text = prepare_representative_sample(doc_text);

    let system_instruction = "You are a Knowledge Graph Relationship Discovery Engine. \
        Your task is to analyze document text and identify factual, evidence-backed semantic connections between the provided concepts.\n\
        RULES:\n\
        1. Only discover relationships that are directly supported by the document text.\n\
        2. STRICTLY connect only concepts provided in the concepts list.\n\
        3. Allowed relationship_type values:\n\
           - RELATED_TO: General semantic association\n\
           - IS_A: Subtype or category relationship (A is a type of B)\n\
           - PART_OF: Component or structural hierarchy (A is part of B)\n\
           - USES: Dependency or utilization (A uses or leverages B)\n\
           - IMPLEMENTS: Realization of a specification, interface, or standard\n\
           - REQUIRES: Hard prerequisite or requirement (A requires B)\n\
           - COMPARED_WITH: Contrast or comparison between alternatives\n\
           - INCLUDES: Encompasses or contains (A includes B)\n\
        4. Confidence must be a number between 0.60 and 1.0 based on clarity in the text.\n\
        5. Evidence must be an exact quote or sentence from the document text showing why this relationship holds.\n\
        6. Do not create self-referential relationships (source and target must be distinct).\n\
        7. Respond with valid JSON matching:\n\
        {\n\
          \"relationships\": [\n\
            {\n\
              \"source\": \"<concept id or name>\",\n\
              \"target\": \"<concept id or name>\",\n\
              \"relationship_type\": \"USES\",\n\
              \"confidence\": 0.95,\n\
              \"evidence\": \"exact quote from text\"\n\
            }\n\
          ]\n\
        }";

    let user_prompt = format!(
        "Document Filename: {}\n\nAvailable Concepts in Document:\n{}\n\nDocument Text:\n{}\n",
        doc_filename, concepts_prompt_list, sample_text
    );

    let raw_model = if model.trim().is_empty() {
        DEFAULT_GENERATION_MODEL
    } else {
        model.trim()
    };
    let clean_model = GeminiClient::clean_model_id(raw_model);

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        clean_model
    );

    let request_body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [{ "text": user_prompt }]
            }
        ],
        "systemInstruction": {
            "parts": [{ "text": system_instruction }]
        },
        "generationConfig": {
            "temperature": 0.1,
            "maxOutputTokens": 4096,
            "responseMimeType": "application/json"
        }
    });

    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
        .post(&url)
        .header("x-goog-api-key", trimmed_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Relationship discovery request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        return Err(GeminiClient::parse_error_response(
            status.as_u16(),
            &err_body,
        ));
    }

    let json_val: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Gemini JSON: {}", e))?;

    let text_content = json_val
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("{}");

    let parsed: GeminiRelationshipResponse = match serde_json::from_str(text_content) {
        Ok(data) => data,
        Err(_) => {
            let clean = text_content
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            serde_json::from_str(clean).unwrap_or(GeminiRelationshipResponse {
                relationships: vec![],
            })
        }
    };

    Ok(parsed.relationships)
}

/// Saves raw relationships to SQLite after resolving concept names/ids and validating constraints
pub fn save_discovered_relationships(
    conn: &Connection,
    doc_id: &str,
    doc_filename: &str,
    doc_concepts: &[DocumentConceptRecord],
    raw_relationships: &[RawRelationshipItem],
) -> Result<Vec<ConceptRelationshipRecord>, String> {
    let mut concept_id_map: HashMap<String, String> = HashMap::new();
    let mut name_to_id_map: HashMap<String, String> = HashMap::new();

    for c in doc_concepts {
        concept_id_map.insert(c.concept_id.clone(), c.name.clone());
        name_to_id_map.insert(c.name.to_lowercase(), c.concept_id.clone());
        name_to_id_map.insert(c.canonical_id.to_lowercase(), c.concept_id.clone());
    }

    let mut saved_records = Vec::new();

    for item in raw_relationships {
        let raw_src = match &item.source {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => continue,
        };
        let raw_tgt = match &item.target {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => continue,
        };

        let src_id = if concept_id_map.contains_key(&raw_src) {
            raw_src.clone()
        } else if let Some(id) = name_to_id_map.get(&raw_src.to_lowercase()) {
            id.clone()
        } else {
            continue;
        };

        let tgt_id = if concept_id_map.contains_key(&raw_tgt) {
            raw_tgt.clone()
        } else if let Some(id) = name_to_id_map.get(&raw_tgt.to_lowercase()) {
            id.clone()
        } else {
            continue;
        };

        if src_id == tgt_id {
            continue;
        }

        let rel_type = match &item.relationship_type {
            Some(rt) => {
                let upper = rt.trim().to_uppercase();
                if is_valid_relationship_type(&upper) {
                    upper
                } else {
                    "RELATED_TO".to_string()
                }
            }
            None => "RELATED_TO".to_string(),
        };

        let conf = item.confidence.unwrap_or(0.85) as f32;
        if conf < 0.60 {
            continue;
        }

        let evidence = item
            .evidence
            .clone()
            .unwrap_or_else(|| format!("Extracted from {}", doc_filename));

        match database::upsert_concept_relationship(
            conn,
            &src_id,
            &tgt_id,
            &rel_type,
            conf,
            &evidence,
            Some(doc_id),
            None,
            "ai_extracted",
        ) {
            Ok(rec) => saved_records.push(rec),
            Err(e) => eprintln!("Failed to save relationship: {}", e),
        }
    }

    Ok(saved_records)
}
