use chrono::Utc;
use uuid::Uuid;
use crate::database::DocumentChunk;

/// Default target chunk length in characters (~150-200 words)
pub const DEFAULT_CHUNK_SIZE: usize = 800;
/// Default overlap in characters (~30-40 words)
pub const DEFAULT_CHUNK_OVERLAP: usize = 150;

/// Splits extracted document text into overlapping chunks.
/// Respects paragraph boundaries (`\n\n`), sentence boundaries (`. `, `? `, `! `),
/// and word breaks so sentences aren't split unnecessarily.
pub fn chunk_text(
    document_id: &str,
    text: &str,
    target_size: usize,
    overlap: usize,
) -> Vec<DocumentChunk> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let chunk_size = if target_size == 0 { DEFAULT_CHUNK_SIZE } else { target_size };
    let chunk_overlap = if overlap >= chunk_size { chunk_size / 4 } else { overlap };

    // If text is smaller than target chunk size, return single chunk
    if trimmed.len() <= chunk_size {
        return vec![DocumentChunk {
            id: Uuid::new_v4().to_string(),
            document_id: document_id.to_string(),
            chunk_index: 0,
            chunk_text: trimmed.to_string(),
            char_count: trimmed.chars().count(),
            word_count: trimmed.split_whitespace().count(),
            created_at: Utc::now().to_rfc3339(),
        }];
    }

    let mut chunks = Vec::new();
    let mut start_idx = 0;
    let chars: Vec<char> = trimmed.chars().collect();
    let total_len = chars.len();
    let mut chunk_index = 0;

    while start_idx < total_len {
        let mut end_idx = (start_idx + chunk_size).min(total_len);

        if end_idx < total_len {
            // Try to find natural break point within the last 25% of the target window
            let search_window_start = (start_idx + chunk_size * 3 / 4).max(start_idx);
            let window: String = chars[search_window_start..end_idx].iter().collect();

            if let Some(pos) = window.rfind("\n\n") {
                end_idx = search_window_start + pos + 2;
            } else if let Some(pos) = window.rfind(". ") {
                end_idx = search_window_start + pos + 2;
            } else if let Some(pos) = window.rfind("? ") {
                end_idx = search_window_start + pos + 2;
            } else if let Some(pos) = window.rfind("! ") {
                end_idx = search_window_start + pos + 2;
            } else if let Some(pos) = window.rfind('\n') {
                end_idx = search_window_start + pos + 1;
            } else if let Some(pos) = window.rfind(' ') {
                end_idx = search_window_start + pos + 1;
            }
        }

        let chunk_slice: String = chars[start_idx..end_idx].iter().collect();
        let chunk_trimmed = chunk_slice.trim().to_string();

        if !chunk_trimmed.is_empty() {
            chunks.push(DocumentChunk {
                id: Uuid::new_v4().to_string(),
                document_id: document_id.to_string(),
                chunk_index,
                chunk_text: chunk_trimmed.clone(),
                char_count: chunk_trimmed.chars().count(),
                word_count: chunk_trimmed.split_whitespace().count(),
                created_at: Utc::now().to_rfc3339(),
            });
            chunk_index += 1;
        }

        if end_idx >= total_len {
            break;
        }

        // Advance start_idx with overlap
        let next_start = if end_idx > chunk_overlap {
            end_idx - chunk_overlap
        } else {
            end_idx
        };

        if next_start <= start_idx {
            start_idx = end_idx;
        } else {
            start_idx = next_start;
        }
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text_chunks() {
        let chunks = chunk_text("doc-1", "", 800, 150);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_small_text_single_chunk() {
        let text = "ECHO Personal Knowledge Operating System.";
        let chunks = chunk_text("doc-1", text, 800, 150);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_text, text);
        assert_eq!(chunks[0].chunk_index, 0);
    }

    #[test]
    fn test_paragraph_respecting_chunks() {
        let p1 = "First paragraph discussing local-first knowledge systems.";
        let p2 = "Second paragraph discussing SQLite database storage.";
        let text = format!("{}\n\n{}", p1, p2);
        let chunks = chunk_text("doc-1", &text, 65, 10);
        assert!(chunks.len() >= 2);
    }
}

