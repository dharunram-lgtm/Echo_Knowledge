use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

use super::provider::{Citation, ContextChunk, DiscoveredModel, GroundedAnswer};
use super::tagging::normalization::{is_blacklisted_tag, normalize_tag_name};
use super::tagging::service::{prepare_representative_sample, ExtractedTag};
use crate::database::{ConversationMessage, DocumentConceptRecord, is_valid_relationship_type};

pub const DEFAULT_GROQ_GENERATION_MODEL: &str = "openai/gpt-oss-20b";

pub const SUPPORTED_GROQ_MODELS: &[&str] = &[
    "openai/gpt-oss-20b",
    "llama-3.3-70b-versatile",
    "llama-3.1-8b-instant",
    "mixtral-8x7b-32768",
    "gemma2-9b-it",
    "deepseek-r1-distill-llama-70b",
    "qwen-2.5-32b",
];

#[derive(Clone)]
pub struct GroqClient {
    client: Client,
}

impl GroqClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(45))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Discovers available Groq models via OpenAI-compatible GET /models
    pub async fn list_models(&self, api_key: &str) -> Result<Vec<DiscoveredModel>, String> {
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("Groq API key is empty".to_string());
        }

        let url = "https://api.groq.com/openai/v1/models";
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", trimmed_key))
            .send()
            .await
            .map_err(|e| format!("Failed to reach Groq models endpoint: {}", e))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(Self::parse_error_response(status.as_u16(), &error_body, Some(&headers)));
        }

        let json_resp: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq models JSON response: {}", e))?;

        let models_arr = json_resp
            .get("data")
            .and_then(|m| m.as_array())
            .ok_or_else(|| "Malformed Groq response: missing 'data' list".to_string())?;

        let mut discovered = Vec::new();
        for m in models_arr {
            let id = m.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_string();
            if id.is_empty() {
                continue;
            }
            // Filter out audio or whisper models that are not chat models
            if id.contains("whisper") {
                continue;
            }

            let owned_by = m.get("owned_by").and_then(|o| o.as_str()).unwrap_or("groq");
            let display_name = id.clone();
            let description = format!("Groq high-speed LLM ({}, owned by {})", id, owned_by);

            discovered.push(DiscoveredModel {
                id: id.clone(),
                full_name: id.clone(),
                display_name,
                description,
                supported_methods: vec!["chat.completions".to_string()],
                supports_generation: true,
                supports_embedding: false, // Groq does not provide vector embeddings
            });
        }

        // Sort so default model appears prominently
        discovered.sort_by(|a, b| {
            if a.id == DEFAULT_GROQ_GENERATION_MODEL {
                std::cmp::Ordering::Less
            } else if b.id == DEFAULT_GROQ_GENERATION_MODEL {
                std::cmp::Ordering::Greater
            } else {
                a.id.cmp(&b.id)
            }
        });

        Ok(discovered)
    }

    /// Verifies the Groq API key by listing available models
    pub async fn test_connection(&self, api_key: &str) -> Result<bool, String> {
        let models = self.list_models(api_key).await?;
        if models.is_empty() {
            Err("No supported Groq models found for this API key".to_string())
        } else {
            Ok(true)
        }
    }

    /// Generates a document-grounded answer using OpenAI-compatible /chat/completions
    pub async fn generate_grounded_answer(
        &self,
        api_key: &str,
        model: &str,
        question: &str,
        context_chunks: &[ContextChunk],
        conversation_history: &[ConversationMessage],
    ) -> Result<GroundedAnswer, String> {
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("Groq API key is not configured. Please set your key in Settings.".to_string());
        }

        let raw_model = if model.trim().is_empty() {
            DEFAULT_GROQ_GENERATION_MODEL
        } else {
            model.trim()
        };

        // If no relevant context chunks were found, return an honest insufficient-context response immediately
        if context_chunks.is_empty() {
            return Ok(GroundedAnswer {
                answer: "Based on your locally stored documents, there is insufficient information to answer this question. No matching document passages were found in your local repository.".to_string(),
                citations: Vec::new(),
                insufficient_context: true,
            });
        }

        // Build grounded prompt
        let mut context_text = String::new();
        for (idx, chunk) in context_chunks.iter().enumerate() {
            context_text.push_str(&format!(
                "\n--- [Source #{}: \"{}\", Chunk #{}] ---\n{}\n",
                idx + 1,
                chunk.filename,
                chunk.chunk_index,
                chunk.chunk_text
            ));
        }

        let system_instruction = "You are ECHO, a secure, local-first personal knowledge assistant.\n\
            Grounding Rules:\n\
            1. Answer the question strictly using the provided Source excerpts below.\n\
            2. Do NOT hallucinate, assume, or invent facts not stated in the sources.\n\
            3. If the sources do not contain enough information to fully answer the question, clearly state: 'Based on your locally stored documents, there is insufficient information to answer this question.'\n\
            4. When referencing facts from a source, reference the source clearly (e.g. [Source #1: filename]).\n\
            5. Distinguish direct documented evidence from necessary inference.";

        let mut messages = Vec::new();
        messages.push(json!({
            "role": "system",
            "content": system_instruction
        }));

        // Include last 4 conversation messages if available
        let recent_history = if conversation_history.len() > 4 {
            &conversation_history[conversation_history.len() - 4..]
        } else {
            conversation_history
        };

        for msg in recent_history {
            let role = if msg.role == "user" { "user" } else { "assistant" };
            messages.push(json!({
                "role": role,
                "content": msg.content
            }));
        }

        let user_content = format!(
            "Retrieved Local Document Excerpts:\n{}\n\nUser Question:\n{}",
            context_text, question
        );

        messages.push(json!({
            "role": "user",
            "content": user_content
        }));

        let request_body = json!({
            "model": raw_model,
            "messages": messages,
            "temperature": 0.2,
            "max_tokens": 2048
        });

        let url = "https://api.groq.com/openai/v1/chat/completions";
        let resp = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", trimmed_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send generation request to Groq API: {}", e))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(Self::parse_error_response(status.as_u16(), &error_body, Some(&headers)));
        }

        let json_resp: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq chat completion response: {}", e))?;

        let answer_text = json_resp
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("No answer text generated by Groq.")
            .to_string();

        let insufficient = answer_text
            .to_lowercase()
            .contains("insufficient information")
            || answer_text.to_lowercase().contains("does not contain enough information");

        // Validate and attach citations from the verified retrieved context chunks
        let citations: Vec<Citation> = context_chunks
            .iter()
            .map(|c| {
                let snippet = if c.chunk_text.chars().count() > 220 {
                    let s: String = c.chunk_text.chars().take(220).collect();
                    format!("{}...", s.trim())
                } else {
                    c.chunk_text.clone()
                };

                Citation {
                    document_id: c.document_id.clone(),
                    filename: c.filename.clone(),
                    file_type: c.file_type.clone(),
                    chunk_id: c.chunk_id.clone(),
                    chunk_index: c.chunk_index,
                    snippet,
                    similarity: c.similarity,
                }
            })
            .collect();

        Ok(GroundedAnswer {
            answer: answer_text,
            citations,
            insufficient_context: insufficient,
        })
    }

    /// Extracts structured tags from document content using Groq
    pub async fn extract_tags(
        &self,
        api_key: &str,
        model: &str,
        filename: &str,
        text: &str,
    ) -> Result<Vec<ExtractedTag>, String> {
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("Groq API key is not configured".to_string());
        }

        let trimmed_text = text.trim();
        if trimmed_text.is_empty() {
            return Ok(Vec::new());
        }

        let raw_model = if model.trim().is_empty() {
            DEFAULT_GROQ_GENERATION_MODEL
        } else {
            model.trim()
        };

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
                }\n\
              ]\n\
            }";

        let user_prompt = format!(
            "Analyze the following document and generate specific tags and concepts:\n\
            Document Filename: {}\n\n\
            Document Content:\n{}\n",
            filename, sample_text
        );

        let request_body = json!({
            "model": raw_model,
            "messages": [
                { "role": "system", "content": system_instruction },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 4096
        });

        let url = "https://api.groq.com/openai/v1/chat/completions";
        let resp = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", trimmed_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Groq tagging request failed: {}", e))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(Self::parse_error_response(status.as_u16(), &error_body, Some(&headers)));
        }

        let json_resp: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq tagging JSON: {}", e))?;

        let raw_content = json_resp
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("{}");

        let tag_data: Value = serde_json::from_str(raw_content)
            .map_err(|e| format!("Failed to parse tags JSON payload from Groq: {}", e))?;

        let tag_items = tag_data
            .get("tags")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();

        let mut cleaned_tags = Vec::new();
        let mut seen_canonical = std::collections::HashSet::new();

        for item in tag_items {
            let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("").trim();
            if name.is_empty() || is_blacklisted_tag(name) {
                continue;
            }

            let (normalized, canonical) = normalize_tag_name(name);
            if normalized.is_empty() || seen_canonical.contains(&canonical) {
                continue;
            }
            seen_canonical.insert(canonical);

            let category = item
                .get("category")
                .and_then(|c| c.as_str())
                .unwrap_or("topic")
                .to_lowercase();
            let cat = match category.as_str() {
                "technology" => "technology".to_string(),
                "concept" => "concept".to_string(),
                _ => "topic".to_string(),
            };

            let confidence = item
                .get("confidence")
                .and_then(|c| c.as_f64())
                .map(|c| c as f32)
                .unwrap_or(0.85)
                .clamp(0.5, 1.0);

            let evidence = item
                .get("evidence")
                .and_then(|e| e.as_str())
                .map(|e| e.trim().to_string())
                .filter(|e| !e.is_empty());

            cleaned_tags.push(ExtractedTag {
                name: normalized,
                category: cat,
                confidence,
                evidence,
            });
        }

        Ok(cleaned_tags)
    }

    /// Discovers relationships between concepts using Groq
    pub async fn extract_relationships(
        &self,
        api_key: &str,
        model: &str,
        doc_filename: &str,
        doc_text: &str,
        doc_concepts: &[DocumentConceptRecord],
    ) -> Result<Vec<crate::ai::graph::discovery::RawRelationshipItem>, String> {
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("Groq API key is not configured".to_string());
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
            DEFAULT_GROQ_GENERATION_MODEL
        } else {
            model.trim()
        };

        let request_body = json!({
            "model": raw_model,
            "messages": [
                { "role": "system", "content": system_instruction },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 4096
        });

        let url = "https://api.groq.com/openai/v1/chat/completions";
        let resp = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", trimmed_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Groq relationship discovery request failed: {}", e))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(Self::parse_error_response(status.as_u16(), &error_body, Some(&headers)));
        }

        let json_resp: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq relationship response: {}", e))?;

        let raw_content = json_resp
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("{}");

        let rel_data: Value = serde_json::from_str(raw_content)
            .map_err(|e| format!("Failed to parse relationships JSON from Groq: {}", e))?;

        let raw_arr = rel_data
            .get("relationships")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        let mut items = Vec::new();
        for val in raw_arr {
            let src = val.get("source").and_then(|s| s.as_str()).map(|s| s.to_string());
            let tgt = val.get("target").and_then(|s| s.as_str()).map(|s| s.to_string());
            let r_type = val.get("relationship_type").and_then(|s| s.as_str()).map(|s| s.to_string());
            let conf = val.get("confidence").and_then(|c| c.as_f64());
            let ev = val.get("evidence").and_then(|s| s.as_str()).map(|s| s.to_string());

            if let (Some(s), Some(t), Some(rt)) = (&src, &tgt, &r_type) {
                if s != t && is_valid_relationship_type(rt) {
                    items.push(crate::ai::graph::discovery::RawRelationshipItem {
                        source: Some(s.clone()),
                        target: Some(t.clone()),
                        relationship_type: Some(rt.clone()),
                        confidence: conf,
                        evidence: ev,
                    });
                }
            }
        }

        Ok(items)
    }

    /// Parses error bodies safely without echoing API keys or secrets
    pub fn parse_error_response(
        status_code: u16,
        body: &str,
        headers: Option<&reqwest::header::HeaderMap>,
    ) -> String {
        let retry_info = if let Some(hdrs) = headers {
            if let Some(val) = hdrs.get("retry-after").and_then(|v| v.to_str().ok()) {
                format!(" (Retry-After: {}s)", val)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        match status_code {
            401 => "Invalid Groq API key (401). Please verify your API key in Settings or at https://console.groq.com/keys.".to_string(),
            403 => "Access forbidden (403). Check that your Groq account has permissions for this resource.".to_string(),
            404 => {
                let clean_msg = Self::clean_error_message(body);
                format!(
                    "Configured Groq model not found (404): {}. Please check your model name in Settings.",
                    clean_msg
                )
            }
            429 => format!(
                "Groq API rate limit or quota exceeded (429){}. Please wait before making further requests.",
                retry_info
            ),
            500 | 503 => {
                let clean_msg = Self::clean_error_message(body);
                format!(
                    "Groq service is temporarily unavailable (HTTP {}): {}. Please try again shortly.",
                    status_code, clean_msg
                )
            }
            _ => format!("Groq API error (HTTP {}): {}", status_code, Self::clean_error_message(body)),
        }
    }

    fn clean_error_message(body: &str) -> String {
        if let Ok(val) = serde_json::from_str::<Value>(body) {
            if let Some(msg) = val.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                return msg.to_string();
            }
        }
        if body.len() > 200 {
            let truncated: String = body.chars().take(200).collect();
            format!("{}...", truncated)
        } else {
            body.to_string()
        }
    }
}

