use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

use super::provider::{Citation, ContextChunk, DiscoveredModel, GroundedAnswer};
use crate::database::ConversationMessage;

pub const DEFAULT_GENERATION_MODEL: &str = "gemini-3.5-flash";
pub const DEFAULT_EMBEDDING_MODEL: &str = "gemini-embedding-001";

pub const SUPPORTED_GENERATION_MODELS: &[&str] = &[
    "gemini-3.5-flash",
    "gemini-3.8-flash",
    "gemini-3.1-pro-preview",
    "gemini-flash-latest",
    "gemini-2.5-flash-lite",
];

pub const SUPPORTED_EMBEDDING_MODELS: &[&str] = &[
    "gemini-embedding-001",
    "gemini-embedding-2",
    "gemini-embedding-2-preview",
];

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
}

impl GeminiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(45))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Strips leading "models/" if present to ensure clean IDs
    pub fn clean_model_id(model: &str) -> &str {
        model.trim().strip_prefix("models/").unwrap_or(model.trim())
    }

    /// Dynamically discovers models available to the provided API key via the official Gemini API
    /// Parses supportedGenerationMethods to distinguish generateContent (generation) and embedContent (embedding)
    pub async fn list_models(&self, api_key: &str) -> Result<Vec<DiscoveredModel>, String> {
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("Gemini API key is empty".to_string());
        }

        let url = "https://generativelanguage.googleapis.com/v1beta/models";
        let resp = self
            .client
            .get(url)
            .header("x-goog-api-key", trimmed_key)
            .send()
            .await
            .map_err(|e| format!("Failed to reach Gemini models endpoint: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(Self::parse_error_response(status.as_u16(), &error_body));
        }

        let json_resp: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Gemini models JSON response: {}", e))?;

        let models_arr = json_resp
            .get("models")
            .and_then(|m| m.as_array())
            .ok_or_else(|| "Malformed Gemini response: missing 'models' list".to_string())?;

        let mut discovered = Vec::new();
        for m in models_arr {
            let full_name = m.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string();
            let id = Self::clean_model_id(&full_name).to_string();
            let display_name = m.get("displayName").and_then(|d| d.as_str()).unwrap_or(&id).to_string();
            let description = m.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();

            let methods: Vec<String> = m
                .get("supportedGenerationMethods")
                .and_then(|arr| arr.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let supports_generation = methods.iter().any(|m| m == "generateContent");
            let supports_embedding = methods.iter().any(|m| m == "embedContent");

            // Only expose models usable for text generation or text embedding
            if supports_generation || supports_embedding {
                discovered.push(DiscoveredModel {
                    id,
                    full_name,
                    display_name,
                    description,
                    supported_methods: methods,
                    supports_generation,
                    supports_embedding,
                });
            }
        }

        Ok(discovered)
    }

    /// Verifies the API key by querying the model list
    pub async fn test_connection(&self, api_key: &str) -> Result<bool, String> {
        let models = self.list_models(api_key).await?;
        if models.is_empty() {
            Err("No supported Gemini models found for this API key".to_string())
        } else {
            Ok(true)
        }
    }

    /// Generates vector embeddings for a list of text strings using the configured embedding model
    pub async fn generate_embeddings(
        &self,
        api_key: &str,
        model: &str,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, String> {
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("Gemini API key not configured".to_string());
        }

        let raw_model = if model.trim().is_empty() {
            DEFAULT_EMBEDDING_MODEL
        } else {
            model.trim()
        };
        let clean_model = Self::clean_model_id(raw_model);

        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut embeddings = Vec::with_capacity(texts.len());

        // Process in bounded batches of 16 for high throughput and rate-limit safety
        for batch in texts.chunks(16) {
            let batch_url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:batchEmbedContents",
                clean_model
            );

            let requests: Vec<Value> = batch
                .iter()
                .map(|t| {
                    json!({
                        "model": format!("models/{}", clean_model),
                        "content": {
                            "parts": [{ "text": t }]
                        }
                    })
                })
                .collect();

            let batch_body = json!({ "requests": requests });

            let mut attempts = 0;
            let max_attempts = 4;
            let mut batch_success = false;

            while attempts < max_attempts && !batch_success {
                attempts += 1;
                let resp_res = self
                    .client
                    .post(&batch_url)
                    .header("x-goog-api-key", trimmed_key)
                    .header("Content-Type", "application/json")
                    .json(&batch_body)
                    .send()
                    .await;

                match resp_res {
                    Ok(resp) => {
                        let status = resp.status();
                        if status.is_success() {
                            let json_resp: Value = resp.json().await.map_err(|e| {
                                format!("Failed to parse Gemini batch embedding JSON: {}", e)
                            })?;

                            if let Some(arr) = json_resp.get("embeddings").and_then(|e| e.as_array()) {
                                for emb_obj in arr {
                                    let vec: Vec<f32> = emb_obj
                                        .get("values")
                                        .and_then(|v| v.as_array())
                                        .map(|v| v.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
                                        .unwrap_or_default();
                                    embeddings.push(vec);
                                }
                                batch_success = true;
                                break;
                            }
                        } else if status.as_u16() == 429 || status.as_u16() == 503 || status.as_u16() == 500 {
                            // Transient rate-limit or service spike: wait and retry
                            let backoff_ms = 1000 * (1 << attempts);
                            std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                            continue;
                        } else {
                            // Non-transient error in batch; fall back to sequential for this batch
                            break;
                        }
                    }
                    Err(_) if attempts < max_attempts => {
                        let backoff_ms = 1000 * (1 << attempts);
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        continue;
                    }
                    Err(e) => {
                        return Err(format!("Failed to send batch embedding request: {}", e));
                    }
                }
            }

            // Fallback to sequential if batching did not succeed
            if !batch_success {
                for text in batch {
                    let single_url = format!(
                        "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent",
                        clean_model
                    );
                    let single_body = json!({
                        "model": format!("models/{}", clean_model),
                        "content": { "parts": [{ "text": text }] }
                    });

                    let mut single_attempts = 0;
                    let mut single_done = false;

                    while single_attempts < 3 && !single_done {
                        single_attempts += 1;
                        let resp = self
                            .client
                            .post(&single_url)
                            .header("x-goog-api-key", trimmed_key)
                            .header("Content-Type", "application/json")
                            .json(&single_body)
                            .send()
                            .await
                            .map_err(|e| format!("Failed to send embedding request: {}", e))?;

                        let status = resp.status();
                        if status.is_success() {
                            let json_resp: Value = resp.json().await.map_err(|e| {
                                format!("Failed to parse Gemini embedding JSON: {}", e)
                            })?;
                            let vec: Vec<f32> = json_resp
                                .get("embedding")
                                .and_then(|e| e.get("values"))
                                .and_then(|v| v.as_array())
                                .map(|v| v.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
                                .unwrap_or_default();
                            if vec.is_empty() {
                                return Err("Gemini returned an empty embedding vector".to_string());
                            }
                            embeddings.push(vec);
                            single_done = true;
                        } else if (status.as_u16() == 429 || status.as_u16() == 503) && single_attempts < 3 {
                            std::thread::sleep(std::time::Duration::from_millis(1500 * single_attempts));
                        } else {
                            let error_body = resp.text().await.unwrap_or_default();
                            return Err(Self::parse_error_response(status.as_u16(), &error_body));
                        }
                    }
                }
            }
        }

        Ok(embeddings)
    }

    /// Generates a document-grounded response using the retrieved local context chunks
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
            return Err("Gemini API key is not configured. Please set your key in Settings.".to_string());
        }

        let raw_model = if model.trim().is_empty() {
            DEFAULT_GENERATION_MODEL
        } else {
            model.trim()
        };
        let clean_model = Self::clean_model_id(raw_model);

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

        // Format recent conversation context
        let mut messages_payload = Vec::new();

        // Include last 4 messages for conversational continuity if available
        let recent_history = if conversation_history.len() > 4 {
            &conversation_history[conversation_history.len() - 4..]
        } else {
            conversation_history
        };

        for msg in recent_history {
            let role = if msg.role == "user" { "user" } else { "model" };
            messages_payload.push(json!({
                "role": role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let prompt_content = format!(
            "{}\n\nRetrieved Local Document Excerpts:\n{}\n\nUser Question:\n{}",
            system_instruction, context_text, question
        );

        messages_payload.push(json!({
            "role": "user",
            "parts": [{ "text": prompt_content }]
        }));

        let request_body = json!({
            "contents": messages_payload,
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 2048,
                "topP": 0.8
            }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            clean_model
        );

        let resp = self
            .client
            .post(&url)
            .header("x-goog-api-key", trimmed_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send generation request to Gemini API: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            return Err(Self::parse_error_response(status.as_u16(), &error_body));
        }

        let json_resp: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Gemini generation response: {}", e))?;

        let answer_text = json_resp
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("No answer text generated by Gemini.")
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

    /// Parses error bodies safely without ever echoing the API key or secret tokens
    pub fn parse_error_response(status_code: u16, body: &str) -> String {
        match status_code {
            400 => {
                if body.contains("API_KEY_INVALID") || body.contains("API key not valid") {
                    "Invalid Google Gemini API key. Please verify your key at Google AI Studio.".to_string()
                } else {
                    format!("Gemini API bad request (400): {}", Self::clean_error_message(body))
                }
            }
            403 => "Permission denied or unauthorized API key (403). Check that the Gemini API is enabled for your Google Cloud / AI Studio project.".to_string(),
            404 => {
                let clean_msg = Self::clean_error_message(body);
                format!("Configured Gemini model not found (404): {}. Please refresh and select an active model in Settings.", clean_msg)
            }
            429 => "Gemini API rate limit or quota exceeded (429). Please wait a moment or check your Google AI Studio plan.".to_string(),
            500 | 503 => {
                let clean_msg = Self::clean_error_message(body);
                format!("Google Gemini service is temporarily unavailable (503/500): {}. Please try again shortly.", clean_msg)
            }
            _ => format!("Gemini API error (HTTP {}): {}", status_code, Self::clean_error_message(body)),
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
