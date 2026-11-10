use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;

use super::normalization::{is_blacklisted_tag, normalize_tag_name};
use crate::ai::gemini::{DEFAULT_GENERATION_MODEL, GeminiClient};
use crate::database::{self, DocumentRecord, DocumentTagRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub category: String, // 'technology' | 'concept' | 'topic'
    pub confidence: f32,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiTagResponse {
    pub tags: Vec<GeminiTagItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiTagItem {
    pub name: Option<String>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub evidence: Option<String>,
}

/// Prepares representative text sample (up to 4000 chars) for AI analysis.
/// Uses 5-anchor multi-sampling (0%, 25%, 50%, 75%, 100%) with character-safe boundaries
/// to guarantee coverage of large documents (such as 50-page PDFs) without UTF-8 slicing panics.
pub fn prepare_representative_sample(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();
    if total_len <= 4000 {
        return text.trim().to_string();
    }

    let slice_chars = |start: usize, count: usize| -> String {
        chars.iter().skip(start).take(count).collect::<String>().trim().to_string()
    };

    let p0 = 0;
    let p25 = total_len / 4;
    let p50 = total_len / 2;
    let p75 = (total_len * 3) / 4;
    let p100 = total_len.saturating_sub(800);

    let head = slice_chars(p0, 1200);
    let quarter = slice_chars(p25, 600);
    let half = slice_chars(p50, 600);
    let three_quarter = slice_chars(p75, 600);
    let tail = slice_chars(p100, 800);

    format!(
        "{}\n\n[... Section 1 ...]\n\n{}\n\n[... Section 2 ...]\n\n{}\n\n[... Section 3 ...]\n\n{}\n\n[... Section 4 ...]\n\n{}",
        head, quarter, half, three_quarter, tail
    )
}

/// Calls Gemini to extract structured tags and concepts from document text
pub async fn extract_tags_with_gemini(
    _client: &GeminiClient,
    api_key: &str,
    model: &str,
    filename: &str,
    text: &str,
) -> Result<Vec<ExtractedTag>, String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err("Gemini API key is not configured".to_string());
    }

    let trimmed_text = text.trim();
    if trimmed_text.is_empty() {
        return Ok(Vec::new());
    }

    let raw_model = if model.trim().is_empty() {
        DEFAULT_GENERATION_MODEL
    } else {
        model.trim()
    };
    let clean_model = GeminiClient::clean_model_id(raw_model);

    let sample_text = prepare_representative_sample(trimmed_text);

    let system_instruction = "You are ECHO's Intelligent Knowledge Tagging & Concept Extraction engine.\n\
        Your mission is to analyze the document content and extract a concise set (between 3 and 7) of high-value tags and concepts.\n\
        Rules:\n\
        1. Tags must represent specific technologies, algorithms, data structures, methods, or core domain concepts actually present in the text.\n\
        2. Do NOT guess tags based solely on the filename.\n\
        3. Do NOT output generic, useless tags such as 'document', 'notes', 'information', 'file', 'content', 'text', 'summary', 'guide', 'test'.\n\
        4. Valid categories: 'technology' (languages, frameworks, tools, libraries), 'concept' (algorithms, data structures, architectural patterns, theories), 'topic' (domain subject matters).\n\
        5. For each tag, provide confidence (0.5 to 1.0) and a brief supporting evidence phrase directly from the text.\n\
        6. Return ONLY a valid JSON object matching this schema:\n\
        {\n\
          \"tags\": [\n\
            {\n\
              \"name\": \"Java\",\n\
              \"category\": \"technology\",\n\
              \"confidence\": 0.96,\n\
              \"evidence\": \"Java programming examples and class definitions\"\n\
            },\n\
            {\n\
              \"name\": \"Linked Lists\",\n\
              \"category\": \"concept\",\n\
              \"confidence\": 0.95,\n\
              \"evidence\": \"Singly and doubly linked list node operations\"\n\
            }\n\
          ]\n\
        }";

    let user_prompt = format!(
        "Analyze the following document and generate specific tags and concepts:\n\
        Document Filename: {}\n\n\
        Document Content:\n{}\n",
        filename, sample_text
    );

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

    let mut resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
        .post(&url)
        .header("x-goog-api-key", trimmed_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Tagging request failed: {}", e))?;

    // Handle transient 500/503 spikes with one automatic retry after brief backoff
    if resp.status().as_u16() == 503 || resp.status().as_u16() == 500 {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        if let Ok(retry_resp) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
            .post(&url)
            .header("x-goog-api-key", trimmed_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            resp = retry_resp;
        }
    }

    let status = resp.status();
    if !status.is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        return Err(GeminiClient::parse_error_response(
            status.as_u16(),
            &err_body,
        ));
    }

    let json_resp: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Gemini tagging response JSON: {}", e))?;

    let text_part = json_resp
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "Gemini returned empty candidate text for tagging".to_string())?;

    // Strip optional markdown json fence if present
    let cleaned_json = text_part
        .trim()
        .strip_prefix("```json")
        .unwrap_or(text_part.trim())
        .strip_prefix("```")
        .unwrap_or(text_part.trim())
        .strip_suffix("```")
        .unwrap_or(text_part.trim())
        .trim();

    let parsed_res: Result<GeminiTagResponse, _> = serde_json::from_str(cleaned_json);
    let tag_items = match parsed_res {
        Ok(res) => res.tags,
        Err(_) => {
            // Attempt to parse as array if the model returned [{...}] instead of {"tags": [...]}
            if let Ok(arr) = serde_json::from_str::<Vec<GeminiTagItem>>(cleaned_json) {
                arr
            } else {
                return Err(format!(
                    "Malformed JSON schema from model: {}",
                    cleaned_json
                ));
            }
        }
    };

    let mut validated_tags = Vec::new();
    let mut seen_canonical: HashSet<String> = HashSet::new();

    for item in tag_items {
        let raw_name = match item.name {
            Some(n) if !n.trim().is_empty() => n,
            _ => continue,
        };

        if is_blacklisted_tag(&raw_name) {
            continue;
        }

        let (display_name, canonical_id) = normalize_tag_name(&raw_name);

        if is_blacklisted_tag(&display_name) || is_blacklisted_tag(&canonical_id) {
            continue;
        }

        if seen_canonical.contains(&canonical_id) {
            continue;
        }

        let cat_str = item
            .category
            .unwrap_or_else(|| "concept".to_string())
            .to_lowercase();
        let category = match cat_str.as_str() {
            "technology" => "technology".to_string(),
            "topic" => "topic".to_string(),
            _ => "concept".to_string(),
        };

        let conf = item
            .confidence
            .map(|c| c.clamp(0.0, 1.0) as f32)
            .unwrap_or(0.90);

        if conf < 0.50 {
            continue;
        }

        seen_canonical.insert(canonical_id);

        validated_tags.push(ExtractedTag {
            name: display_name,
            category,
            confidence: conf,
            evidence: item.evidence.filter(|e| !e.trim().is_empty()),
        });

        // Limit to max 7 tags per document
        if validated_tags.len() >= 7 {
            break;
        }
    }

    Ok(validated_tags)
}

/// Writes extracted tags and concepts directly into SQLite within a transaction
pub fn save_extracted_tags(
    conn: &Connection,
    doc: &DocumentRecord,
    extracted: &[ExtractedTag],
) -> Result<Vec<DocumentTagRecord>, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Clean up existing AI-generated tags for this document so they are replaced cleanly,
    // while strictly preserving manual user tags (where is_ai_generated = 0)
    let _ = tx.execute(
        "DELETE FROM document_tags WHERE document_id = ?1 AND is_ai_generated = 1;",
        rusqlite::params![doc.id],
    );

    for tag in extracted {
        let (display_name, canonical_id) = normalize_tag_name(&tag.name);

        // 1. Tag table & document_tags link
        let tag_id = database::upsert_tag(&tx, &display_name, &canonical_id, &tag.category)
            .map_err(|e| format!("Failed to upsert tag: {}", e))?;

        database::add_tag_to_document(
            &tx,
            &doc.id,
            &tag_id,
            true, // is_ai_generated
            tag.confidence,
            tag.evidence.as_deref(),
        )
        .map_err(|e| format!("Failed to link tag to document: {}", e))?;

        // 2. Concept table & document_concepts link (for Knowledge Graph)
        let concept_id = database::upsert_concept(
            &tx,
            &display_name,
            &canonical_id,
            &tag.category,
            tag.evidence.as_deref(),
        )
        .map_err(|e| format!("Failed to upsert concept: {}", e))?;

        database::add_concept_to_document(
            &tx,
            &doc.id,
            &concept_id,
            tag.confidence,
            tag.evidence.as_deref(),
        )
        .map_err(|e| format!("Failed to link concept to document: {}", e))?;
    }

    database::update_tagging_status(&tx, &doc.id, "tagged", None, extracted.len())
        .map_err(|e| format!("Failed to update tagging status: {}", e))?;

    tx.commit().map_err(|e| e.to_string())?;

    database::get_tags_for_document(conn, &doc.id)
        .map_err(|e| format!("Failed to retrieve updated tags: {}", e))
}
