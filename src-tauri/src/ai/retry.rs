use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    Transient,
    RateLimit,
    QuotaExhausted,
    Permanent,
}

impl ErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Transient => "transient",
            ErrorCategory::RateLimit => "rate_limit",
            ErrorCategory::QuotaExhausted => "quota_exhausted",
            ErrorCategory::Permanent => "permanent",
        }
    }
}

/// Classifies an error into transient, rate-limited, quota exhausted, or permanent.
/// Returns (category, optional Retry-After seconds).
pub fn classify_error(status_code: Option<u16>, err_msg: &str) -> (ErrorCategory, Option<u64>) {
    let lower_msg = err_msg.to_lowercase();

    // Check status code first if present
    if let Some(code) = status_code {
        match code {
            400 => {
                // If model not found or invalid argument, permanent
                return (ErrorCategory::Permanent, None);
            }
            401 | 403 => {
                // Authentication / permission error
                return (ErrorCategory::Permanent, None);
            }
            404 => {
                // Model not found or resource missing
                return (ErrorCategory::Permanent, None);
            }
            429 => {
                // Parse retry-after from message if available
                let retry_sec = extract_retry_seconds(&lower_msg);
                if lower_msg.contains("quota") || lower_msg.contains("exhausted") {
                    return (ErrorCategory::QuotaExhausted, retry_sec);
                }
                return (ErrorCategory::RateLimit, retry_sec);
            }
            500 | 502 | 503 | 504 => {
                return (ErrorCategory::Transient, None);
            }
            _ => {}
        }
    }

    // Inspect error message text
    if lower_msg.contains("timed out")
        || lower_msg.contains("timeout")
        || lower_msg.contains("connection reset")
        || lower_msg.contains("connection refused")
        || lower_msg.contains("database is locked")
        || lower_msg.contains("database locked")
        || lower_msg.contains("busy")
        || lower_msg.contains("503")
        || lower_msg.contains("502")
        || lower_msg.contains("504")
        || lower_msg.contains("500 internal server error")
        || lower_msg.contains("temporarily unavailable")
    {
        return (ErrorCategory::Transient, None);
    }

    if lower_msg.contains("429") || lower_msg.contains("rate limit") || lower_msg.contains("too many requests") {
        let retry_sec = extract_retry_seconds(&lower_msg);
        if lower_msg.contains("quota") || lower_msg.contains("exhausted") {
            return (ErrorCategory::QuotaExhausted, retry_sec);
        }
        return (ErrorCategory::RateLimit, retry_sec);
    }

    if lower_msg.contains("quota exceeded") || lower_msg.contains("resource_exhausted") {
        return (ErrorCategory::QuotaExhausted, extract_retry_seconds(&lower_msg));
    }

    if lower_msg.contains("api key")
        || lower_msg.contains("invalid_argument")
        || lower_msg.contains("not found (404)")
        || lower_msg.contains("unauthorized")
        || lower_msg.contains("forbidden")
    {
        return (ErrorCategory::Permanent, None);
    }

    // Default to Transient for unrecognized network errors to allow safety retries
    (ErrorCategory::Transient, None)
}

fn extract_retry_seconds(msg: &str) -> Option<u64> {
    // Look for patterns like "retry-after: 15" or "retry in 12s" or "after 30 seconds"
    if let Some(idx) = msg.find("retry after") {
        let snippet = &msg[idx..];
        for word in snippet.split_whitespace() {
            if let Ok(num) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u64>() {
                if num > 0 && num <= 3600 {
                    return Some(num);
                }
            }
        }
    }
    None
}

/// Calculates the next retry timestamp and delay seconds using bounded exponential backoff with jitter.
/// Base backoff sequence: 5s, 15s, 30s, 60s, 120s, 300s, max 1800s (30m).
pub fn calculate_next_retry(
    attempt_count: usize,
    category: &ErrorCategory,
    retry_after: Option<u64>,
) -> (String, u64) {
    if let Some(sec) = retry_after {
        let jitter = (Utc::now().timestamp_subsec_millis() % 3) as u64;
        let total_sec = (sec + jitter).min(1800);
        let next_time = Utc::now() + ChronoDuration::seconds(total_sec as i64);
        return (next_time.to_rfc3339(), total_sec);
    }

    let base_seconds = match category {
        ErrorCategory::Transient => match attempt_count {
            0 | 1 => 5,
            2 => 15,
            3 => 30,
            4 => 60,
            5 => 120,
            6 => 300,
            _ => 600,
        },
        ErrorCategory::RateLimit => match attempt_count {
            0 | 1 => 15,
            2 => 30,
            3 => 60,
            4 => 120,
            _ => 300,
        },
        ErrorCategory::QuotaExhausted => match attempt_count {
            0 | 1 => 300, // 5m
            2 => 900,     // 15m
            _ => 1800,    // 30m
        },
        ErrorCategory::Permanent => {
            // Permanent errors shouldn't be scheduled, but if queried return max
            1800
        }
    };

    // Jitter: add between 0 and 4 seconds based on millisecond timestamp
    let jitter = (Utc::now().timestamp_subsec_millis() % 5) as u64;
    let final_seconds = (base_seconds + jitter).min(1800);
    let next_retry = Utc::now() + ChronoDuration::seconds(final_seconds as i64);

    (next_retry.to_rfc3339(), final_seconds)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryOverview {
    pub is_globally_paused: bool,
    pub pending_indexing_count: usize,
    pub retry_wait_indexing_count: usize,
    pub succeeded_indexing_count: usize,
    pub failed_indexing_count: usize,
    pub pending_tagging_count: usize,
    pub retry_wait_tagging_count: usize,
    pub succeeded_tagging_count: usize,
    pub failed_tagging_count: usize,
}

