use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::database::{self, AppStorage, DocumentRecord};
use crate::security;

pub const DEFAULT_CLIPPER_PORT: u16 = 23119;
pub const FALLBACK_CLIPPER_PORT: u16 = 23120;
pub const MAX_REQUEST_BODY_BYTES: usize = 15 * 1024 * 1024; // 15 MB hard streaming limit
pub const RATE_LIMIT_MAX_REQUESTS: usize = 60;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipperHealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub authenticated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipPayload {
    pub url: String,
    pub title: String,
    pub content_markdown: String,
    pub extracted_text: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub favicon_url: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub duplicate_strategy: Option<String>, // "skip", "update", "save_copy"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSuccessResponse {
    pub status: String, // "created", "updated", "skipped", "conflict"
    pub document_id: String,
    pub filename: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_detected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedExtensionInfo {
    pub extension_id: String,
    #[serde(default)]
    pub browser: Option<String>,
    pub paired_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipperPairingInfo {
    pub port: u16,
    pub token_configured: bool,
    pub is_running: bool,
    pub authorized_extension_id: Option<String>,
    pub authorized_extensions: Vec<AuthorizedExtensionInfo>,
}

struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
        }
    }

    fn check_rate_limit(&self, client_key: &str) -> bool {
        let mut map = match self.requests.lock() {
            Ok(m) => m,
            Err(_) => return true,
        };

        let now = Instant::now();
        let timestamps = map.entry(client_key.to_string()).or_default();
        timestamps.retain(|&t| now.duration_since(t).as_secs() < RATE_LIMIT_WINDOW_SECS);

        if timestamps.len() >= RATE_LIMIT_MAX_REQUESTS {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

pub struct ClipperServerController {
    pub is_running: Arc<AtomicBool>,
    pub should_stop: Arc<AtomicBool>,
    pub active_port: Arc<Mutex<u16>>,
}

impl ClipperServerController {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            should_stop: Arc::new(AtomicBool::new(false)),
            active_port: Arc::new(Mutex::new(DEFAULT_CLIPPER_PORT)),
        }
    }
}

/// Normalizes a URL conservatively:
/// 1. Lowercases scheme and host
/// 2. Strips port if standard (80, 443)
/// 3. Removes trailing slash on path (except root /)
/// 4. Preserves fragment if it represents routing (contains '/' or '=' or '!')
/// 5. Removes ONLY established marketing/analytics trackers (utm_*, fbclid, gclid, etc.)
/// 6. Strictly preserves query parameter ordering, repeated parameters, and content identifiers
pub fn normalize_canonical_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Split anchor fragment
    let (url_without_frag, frag_part) = match trimmed.find('#') {
        Some(idx) => (&trimmed[..idx], Some(&trimmed[idx + 1..])),
        None => (trimmed, None),
    };

    // Determine whether fragment is a routing fragment (SPA route)
    let should_preserve_frag = match frag_part {
        Some(frag) => {
            let f = frag.trim();
            // Preserve if looks like a route or query (/path, !/path, tab=, etc.)
            f.starts_with('/') || f.starts_with("!/") || f.contains('/') || f.contains('=')
        }
        None => false,
    };

    let (base_part, query_part) = match url_without_frag.find('?') {
        Some(idx) => (&url_without_frag[..idx], Some(&url_without_frag[idx + 1..])),
        None => (url_without_frag, None),
    };

    // Normalize base URL
    let normalized_base = if let Some(idx) = base_part.find("://") {
        let scheme = &base_part[..idx].to_lowercase();
        let rest = &base_part[idx + 3..];
        let (host, path) = match rest.find('/') {
            Some(p_idx) => (&rest[..p_idx], &rest[p_idx..]),
            None => (rest, ""),
        };
        let host_lower = host.to_lowercase();
        let host_clean = if host_lower.ends_with(":80") && scheme == "http" {
            &host_lower[..host_lower.len() - 3]
        } else if host_lower.ends_with(":443") && scheme == "https" {
            &host_lower[..host_lower.len() - 4]
        } else {
            &host_lower
        };
        let path_clean = if path.len() > 1 && path.ends_with('/') && !should_preserve_frag {
            &path[..path.len() - 1]
        } else {
            path
        };
        format!("{}://{}{}", scheme, host_clean, path_clean)
    } else {
        base_part.to_string()
    };

    // Filter ONLY established tracking parameters, preserving order and repeated parameters
    let normalized_query = match query_part {
        Some(q) if !q.is_empty() => {
            let tracking_keys: [&str; 12] = [
                "utm_source",
                "utm_medium",
                "utm_campaign",
                "utm_term",
                "utm_content",
                "fbclid",
                "gclid",
                "msclkid",
                "mc_eid",
                "igshid",
                "_hsenc",
                "_openstat",
            ];

            let mut preserved_pairs: Vec<&str> = Vec::new();
            for pair in q.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let k = match pair.find('=') {
                    Some(eq_idx) => &pair[..eq_idx],
                    None => pair,
                };
                let k_lower = k.trim().to_lowercase();
                if !tracking_keys.contains(&k_lower.as_str()) {
                    preserved_pairs.push(pair);
                }
            }

            if preserved_pairs.is_empty() {
                None
            } else {
                Some(preserved_pairs.join("&"))
            }
        }
        _ => None,
    };

    let mut result = normalized_base;
    if let Some(ref q) = normalized_query {
        result.push('?');
        result.push_str(q);
    }
    if should_preserve_frag {
        if let Some(f) = frag_part {
            result.push('#');
            result.push_str(f);
        }
    }
    result
}

/// Checks whether an Origin header matches a valid Chromium extension format:
/// `chrome-extension://[a-p]{32}`
pub fn parse_and_validate_extension_id(origin: &str) -> Option<String> {
    let trimmed = origin.trim();
    if !trimmed.starts_with("chrome-extension://") {
        return None;
    }

    let ext_id = &trimmed["chrome-extension://".len()..];
    let ext_id_clean = ext_id.trim_end_matches('/');

    // Chromium extension IDs are exactly 32 lowercase alpha characters in [a-p]
    if ext_id_clean.len() != 32 {
        return None;
    }

    if ext_id_clean.chars().all(|c| ('a'..='p').contains(&c)) {
        Some(ext_id_clean.to_string())
    } else {
        None
    }
}

pub fn is_valid_extension_origin(origin: &str) -> bool {
    parse_and_validate_extension_id(origin).is_some()
}

/// Checks Host header to prevent DNS rebinding attacks on standard or fallback ports
pub fn is_valid_host(host: &str) -> bool {
    is_valid_host_for_port(host, DEFAULT_CLIPPER_PORT)
        || is_valid_host_for_port(host, FALLBACK_CLIPPER_PORT)
}

pub fn is_valid_host_for_port(host: &str, port: u16) -> bool {
    let trimmed = host.trim().to_lowercase();
    trimmed == format!("127.0.0.1:{}", port)
        || trimmed == format!("localhost:{}", port)
        || trimmed == "127.0.0.1"
        || trimmed == "localhost"
}

/// Retrieves or creates the pairing token in Windows Credential Manager
pub fn get_or_create_pairing_token(conn: &rusqlite::Connection) -> Result<String, String> {
    // Check Windows Credential Manager first
    if let Ok(Some(token)) = security::get_clipper_token() {
        if !token.trim().is_empty() {
            return Ok(token.trim().to_string());
        }
    }

    // Fallback: check SQLite app_settings
    if let Ok(Some(token)) = database::get_app_setting(conn, "web_clipper_auth_token") {
        if !token.trim().is_empty() {
            let _ = security::store_clipper_token(token.trim());
            return Ok(token.trim().to_string());
        }
    }

    // Generate new 256-bit token (64 hex characters)
    let seed = format!(
        "{}-{}-{}",
        Uuid::new_v4(),
        Utc::now().to_rfc3339(),
        std::process::id()
    );
    let token = hex::encode(Sha256::digest(seed.as_bytes()));

    // Store in Windows Credential Manager and backup in app_settings
    let _ = security::store_clipper_token(&token);
    let _ = database::set_app_setting(conn, "web_clipper_auth_token", &token);

    Ok(token)
}

pub fn get_authorized_extensions(conn: &rusqlite::Connection) -> Vec<AuthorizedExtensionInfo> {
    if let Ok(Some(json_str)) = database::get_app_setting(conn, "web_clipper_authorized_extensions")
    {
        if let Ok(list) = serde_json::from_str::<Vec<AuthorizedExtensionInfo>>(&json_str) {
            return list;
        }
    }
    // Backward compatibility: check single legacy key
    if let Ok(Some(single_id)) =
        database::get_app_setting(conn, "web_clipper_authorized_extension_id")
    {
        let clean = single_id.trim();
        if !clean.is_empty() {
            return vec![AuthorizedExtensionInfo {
                extension_id: clean.to_string(),
                browser: Some("Browser".to_string()),
                paired_at: Utc::now().to_rfc3339(),
            }];
        }
    }
    Vec::new()
}

pub fn is_extension_authorized(conn: &rusqlite::Connection, ext_id: &str) -> bool {
    let clean = ext_id.trim().to_lowercase();
    get_authorized_extensions(conn)
        .iter()
        .any(|e| e.extension_id == clean)
}

/// Revokes the pairing token, removing it from Windows Credential Manager and SQLite
pub fn revoke_pairing_token(conn: &rusqlite::Connection) -> Result<(), String> {
    let _ = security::delete_clipper_token();
    database::set_app_setting(conn, "web_clipper_auth_token", "").map_err(|e| e.to_string())?;
    let _ = database::set_app_setting(conn, "web_clipper_authorized_extension_id", "");
    let _ = database::set_app_setting(conn, "web_clipper_authorized_extensions", "[]");
    Ok(())
}

/// Reads request body with strict byte-level streaming enforcement.
/// Returns Err if missing, malformed, or exceeding 15 MB.
pub fn read_body_bounded<R: Read>(
    reader: &mut R,
    headers: &HashMap<String, String>,
) -> Result<Vec<u8>, (u16, &'static str, &'static str)> {
    if let Some(te) = headers.get("transfer-encoding") {
        if te.to_lowercase().contains("chunked") {
            return Err((
                411,
                "Length Required",
                "Chunked transfer encoding is not supported. Use Content-Length.",
            ));
        }
    }

    let cl_str = match headers.get("content-length") {
        Some(val) => val,
        None => return Err((411, "Length Required", "Content-Length header required")),
    };

    let content_length: usize = match cl_str.trim().parse() {
        Ok(len) => len,
        Err(_) => return Err((400, "Bad Request", "Malformed Content-Length header")),
    };

    if content_length > MAX_REQUEST_BODY_BYTES {
        return Err((413, "Payload Too Large", "Payload exceeds 15 MB limit"));
    }

    let mut body_bytes = vec![0u8; content_length];
    if let Err(_) = reader.read_exact(&mut body_bytes) {
        return Err((
            400,
            "Bad Request",
            "Unexpected EOF or failure reading request body",
        ));
    }

    Ok(body_bytes)
}

/// Starts the loopback HTTP server on 127.0.0.1:23119 in an independent background thread
pub fn start_clipper_server(app: AppHandle) {
    let storage = match app.try_state::<AppStorage>() {
        Some(s) => s,
        None => return,
    };

    // Check if web clipper server is enabled in settings (default: true)
    let is_enabled = {
        let conn = match storage.conn.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        database::get_app_setting(&conn, "web_clipper_server_enabled")
            .unwrap_or(None)
            .map(|v| v != "false")
            .unwrap_or(true)
    };

    if !is_enabled {
        println!("[ECHO Clipper] Server disabled by user setting.");
        return;
    }

    let controller = app.state::<ClipperServerController>();
    if controller.is_running.swap(true, Ordering::SeqCst) {
        return; // Already running
    }
    controller.should_stop.store(false, Ordering::SeqCst);

    let (listener, bound_port) = match TcpListener::bind(format!(
        "127.0.0.1:{}",
        DEFAULT_CLIPPER_PORT
    )) {
        Ok(l) => (l, DEFAULT_CLIPPER_PORT),
        Err(e) => {
            eprintln!(
                "[ECHO Clipper] Port {} occupied or unavailable ({}), attempting fallback port {}...",
                DEFAULT_CLIPPER_PORT, e, FALLBACK_CLIPPER_PORT
            );
            match TcpListener::bind(format!("127.0.0.1:{}", FALLBACK_CLIPPER_PORT)) {
                Ok(l) => (l, FALLBACK_CLIPPER_PORT),
                Err(e2) => {
                    eprintln!(
                        "[ECHO Clipper] Failed to bind fallback port {}: {}",
                        FALLBACK_CLIPPER_PORT, e2
                    );
                    controller.is_running.store(false, Ordering::SeqCst);
                    return;
                }
            }
        }
    };

    if let Ok(mut port_guard) = controller.active_port.lock() {
        *port_guard = bound_port;
    }

    let _ = listener.set_nonblocking(true);
    println!(
        "[ECHO Clipper] Loopback server listening on 127.0.0.1:{}",
        bound_port
    );

    let rate_limiter = Arc::new(RateLimiter::new());
    let should_stop = controller.should_stop.clone();
    let is_running_flag = controller.is_running.clone();

    thread::spawn(move || {
        while !should_stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let app_handle = app.clone();
                    let limiter = rate_limiter.clone();
                    thread::spawn(move || {
                        handle_client_connection(stream, app_handle, limiter, bound_port);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("[ECHO Clipper] Accept error: {}", e);
                    thread::sleep(Duration::from_millis(200));
                }
            }
        }
        is_running_flag.store(false, Ordering::SeqCst);
        println!("[ECHO Clipper] Server cleanly stopped.");
    });
}

fn handle_client_connection(
    mut stream: TcpStream,
    app: AppHandle,
    rate_limiter: Arc<RateLimiter>,
    active_port: u16,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();

    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let line_parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if line_parts.len() < 2 {
        let _ = write_response(
            &mut stream,
            400,
            "Bad Request",
            "text/plain",
            b"Invalid HTTP request line",
        );
        return;
    }

    let method = line_parts[0];
    let path = line_parts[1];

    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line).is_err() {
            break;
        }
        let trimmed = header_line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(idx) = trimmed.find(':') {
            let key = trimmed[..idx].trim().to_lowercase();
            let val = trimmed[idx + 1..].trim().to_string();
            headers.insert(key, val);
        }
    }

    // 1. Host header validation (DNS Rebinding protection)
    let host_header = headers.get("host").cloned().unwrap_or_default();
    if !is_valid_host_for_port(&host_header, active_port) {
        let _ = write_response(
            &mut stream,
            400,
            "Bad Request",
            "text/plain",
            b"Invalid or untrusted Host header",
        );
        return;
    }

    // 2. Rate limiting check
    let client_ip = stream
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    if !rate_limiter.check_rate_limit(&client_ip) {
        let _ = write_response_with_retry(
            &mut stream,
            429,
            "Too Many Requests",
            "text/plain",
            b"Rate limit exceeded. Please wait 30 seconds.",
            30,
        );
        return;
    }

    // 3. Origin header validation
    // REJECT: missing Origin, "null" Origin, and all HTTP/HTTPS web origins
    let origin_header = match headers.get("origin") {
        Some(o) => o.trim(),
        None => {
            let _ = write_response(
                &mut stream,
                403,
                "Forbidden",
                "text/plain",
                b"Missing Origin header: Only authorized extensions permitted",
            );
            return;
        }
    };

    if origin_header.is_empty()
        || origin_header == "null"
        || origin_header.starts_with("http://")
        || origin_header.starts_with("https://")
    {
        let _ = write_response(
            &mut stream,
            403,
            "Forbidden",
            "text/plain",
            b"Web origins and null origins are strictly forbidden",
        );
        return;
    }

    let ext_id = match parse_and_validate_extension_id(origin_header) {
        Some(id) => id,
        None => {
            let _ = write_response(
                &mut stream,
                403,
                "Forbidden",
                "text/plain",
                b"Invalid Chromium extension Origin format",
            );
            return;
        }
    };

    // CORS preflight OPTIONS
    if method == "OPTIONS" {
        let cors_headers = format!(
            "HTTP/1.1 204 No Content\r\n\
             Access-Control-Allow-Origin: {}\r\n\
             Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
             Access-Control-Allow-Headers: Authorization, Content-Type, X-Idempotency-Key, X-ECHO-Client-Version\r\n\
             Access-Control-Max-Age: 86400\r\n\
             Content-Length: 0\r\n\r\n",
            origin_header
        );
        let _ = stream.write_all(cors_headers.as_bytes());
        return;
    }

    // 4. Token & Multi-Extension Pairing Authorization
    let (expected_token, is_authorized) = {
        let storage = match app.try_state::<AppStorage>() {
            Some(s) => s,
            None => {
                let _ = write_cors_response(
                    &mut stream,
                    500,
                    "Internal Server Error",
                    "application/json",
                    b"{\"error\":\"App storage unavailable\"}",
                    origin_header,
                );
                return;
            }
        };
        let conn = match storage.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                let _ = write_cors_response(
                    &mut stream,
                    500,
                    "Internal Server Error",
                    "application/json",
                    b"{\"error\":\"Database lock error\"}",
                    origin_header,
                );
                return;
            }
        };
        let tok = match get_or_create_pairing_token(&conn) {
            Ok(t) => t,
            Err(e) => {
                let _ = write_cors_response(
                    &mut stream,
                    500,
                    "Internal Server Error",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", e).as_bytes(),
                    origin_header,
                );
                return;
            }
        };
        let auth = is_extension_authorized(&conn, &ext_id);
        (tok, auth)
    };

    let auth_header = headers.get("authorization").cloned().unwrap_or_default();
    let token_provided = if auth_header.starts_with("Bearer ") {
        auth_header["Bearer ".len()..].trim()
    } else {
        ""
    };

    let is_authenticated = !token_provided.is_empty()
        && security::constant_time_compare(token_provided, &expected_token);
    if !is_authenticated {
        let _ = write_cors_response(&mut stream, 401, "Unauthorized", "application/json", b"{\"status\":\"unauthorized\",\"authenticated\":false,\"error\":\"Invalid or missing pairing token\"}", origin_header);
        return;
    }

    // Extension pairing verification:
    // Requires explicit pairing authorization; rejects unpaired or revoked extension IDs.
    if !is_authorized {
        let _ = write_cors_response(&mut stream, 403, "Forbidden", "application/json", b"{\"error\":\"Extension not authorized. Please pair your extension in ECHO Settings.\"}", origin_header);
        return;
    }

    // Route: GET /api/v1/health
    if method == "GET" && (path == "/api/v1/health" || path == "/health") {
        let health = ClipperHealthResponse {
            status: "online".to_string(),
            service: "echo-web-clipper".to_string(),
            version: "1.2.1".to_string(),
            authenticated: true,
        };
        let body = serde_json::to_vec(&health).unwrap_or_default();
        let _ = write_cors_response(
            &mut stream,
            200,
            "OK",
            "application/json",
            &body,
            origin_header,
        );
        return;
    }

    // Route: POST /api/v1/clip
    if method == "POST" && (path == "/api/v1/clip" || path == "/clip") {
        let body_bytes = match read_body_bounded(&mut reader, &headers) {
            Ok(bytes) => bytes,
            Err((code, status, msg)) => {
                let err_body = format!("{{\"error\":\"{}\"}}", msg);
                let _ = write_cors_response(
                    &mut stream,
                    code,
                    status,
                    "application/json",
                    err_body.as_bytes(),
                    origin_header,
                );
                return;
            }
        };

        let payload: ClipPayload = match serde_json::from_slice(&body_bytes) {
            Ok(p) => p,
            Err(e) => {
                let _ = write_cors_response(
                    &mut stream,
                    400,
                    "Bad Request",
                    "application/json",
                    format!("{{\"error\":\"Invalid JSON payload: {}\"}}", e).as_bytes(),
                    origin_header,
                );
                return;
            }
        };

        let idempotency_key = headers.get("x-idempotency-key").cloned();
        let payload_hash = format!("{:x}", Sha256::digest(&body_bytes));

        let result = process_web_clip(&app, payload, idempotency_key, &payload_hash);
        match result {
            Ok((status_code, response)) => {
                let resp_body = serde_json::to_vec(&response).unwrap_or_default();
                let status_text = match status_code {
                    200 => "OK",
                    201 => "Created",
                    409 => "Conflict",
                    _ => "OK",
                };
                let _ = write_cors_response(
                    &mut stream,
                    status_code,
                    status_text,
                    "application/json",
                    &resp_body,
                    origin_header,
                );
            }
            Err(e) => {
                let _ = write_cors_response(
                    &mut stream,
                    500,
                    "Internal Server Error",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", e).as_bytes(),
                    origin_header,
                );
            }
        }
        return;
    }

    // 404 for unknown endpoints
    let _ = write_cors_response(
        &mut stream,
        404,
        "Not Found",
        "application/json",
        b"{\"error\":\"Endpoint not found\"}",
        origin_header,
    );
}

/// Startup crash recovery & reconciliation:
/// 1. Cleans orphan `.tmp_*` files left by incomplete atomic stream writes.
/// 2. Reconciles `.md.bak` files from interrupted updates:
///    - Checks SQLite: does the document record have the updated content hash?
///    - If DB update committed: deletes the `.md.bak` file, keeping the updated file.
///    - If DB update did NOT commit: restores the `.md.bak` file to its original target name.
/// 3. Reconciles orphaned web note files created during crashes prior to SQLite insert:
///    - Identifies `{uuid}_*.md` files in documents_dir.
///    - Checks if `{uuid}` exists in SQLite `documents` table with `file_type = 'web'`.
///    - If absent in SQLite: removes the uncommitted orphaned file.
/// 4. NEVER modifies or deletes authentic user documents (`.pdf`, `.md`, `.txt`) or non-web notes.
pub fn reconcile_and_recover_clipper_files(conn: &rusqlite::Connection, documents_dir: &Path) {
    let entries = match fs::read_dir(documents_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut bak_files: Vec<PathBuf> = Vec::new();
    let mut tmp_files: Vec<PathBuf> = Vec::new();
    let mut web_note_files: Vec<(String, PathBuf)> = Vec::new(); // (doc_id, path)

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if file_name.starts_with(".tmp_") {
                tmp_files.push(path);
            } else if file_name.ends_with(".md.bak") {
                bak_files.push(path);
            } else if file_name.ends_with(".md") {
                // Check if file starts with a 36-character UUIDv4
                if file_name.len() > 37 && file_name.chars().nth(36) == Some('_') {
                    let possible_uuid = &file_name[..36];
                    if Uuid::parse_str(possible_uuid).is_ok() {
                        web_note_files.push((possible_uuid.to_string(), path));
                    }
                }
            }
        }
    }

    // 1. Clean incomplete .tmp_* files
    for tmp in tmp_files {
        let _ = fs::remove_file(tmp);
    }

    // 2. Reconcile .md.bak files
    for bak_path in bak_files {
        let orig_path = bak_path.with_extension(""); // removes .bak, leaving .md
        let file_name = match orig_path.file_name().and_then(|f| f.to_str()) {
            Some(n) => n,
            None => {
                let _ = fs::remove_file(bak_path);
                continue;
            }
        };

        // Extract doc_id from filename `{uuid}_...`
        if file_name.len() > 37 && file_name.chars().nth(36) == Some('_') {
            let doc_id = &file_name[..36];
            let doc_in_db: Option<(String, String)> = conn
                .query_row(
                    "SELECT content_hash, file_type FROM documents WHERE id = ?1;",
                    [doc_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok();

            match doc_in_db {
                Some((stored_hash, ref file_type)) if file_type == "web" => {
                    // Check hash of current on-disk target file
                    let current_matches = if orig_path.exists() {
                        if let Ok(bytes) = fs::read(&orig_path) {
                            let disk_hash = format!("{:x}", Sha256::digest(&bytes));
                            disk_hash == stored_hash
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if current_matches {
                        // DB update committed and file on disk matches DB -> safely delete backup
                        let _ = fs::remove_file(bak_path);
                    } else {
                        // DB update did NOT commit (or file was corrupted) -> restore backup!
                        let _ = fs::rename(&bak_path, &orig_path);
                    }
                }
                _ => {
                    // Document does not exist in DB -> restore backup to be safe, or if uncommitted, cleanup
                    if !orig_path.exists() {
                        let _ = fs::rename(&bak_path, &orig_path);
                    } else {
                        let _ = fs::remove_file(bak_path);
                    }
                }
            }
        } else {
            let _ = fs::remove_file(bak_path);
        }
    }

    // 3. Reconcile orphaned new web note files (written to disk but crash occurred before SQLite commit)
    for (doc_id, path) in web_note_files {
        let exists_in_db: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE id = ?1;",
                [&doc_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !exists_in_db {
            // Check if file is inside managed documents directory
            let _ = fs::remove_file(path);
        }
    }
}

pub fn write_file_atomic(dest_path: &Path, content: &[u8]) -> std::io::Result<()> {
    let parent = dest_path.parent().unwrap_or_else(|| Path::new("."));
    let temp_name = format!(
        ".tmp_{}_{}",
        Uuid::new_v4(),
        dest_path.file_name().unwrap_or_default().to_string_lossy()
    );
    let temp_path = parent.join(temp_name);

    if let Err(e) = fs::write(&temp_path, content) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    if let Err(e) = fs::rename(&temp_path, dest_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(())
}

/// Processes a verified Web Note clip payload with atomic database transactions and file safety
pub fn process_web_clip(
    app: &AppHandle,
    payload: ClipPayload,
    idempotency_key: Option<String>,
    payload_hash: &str,
) -> Result<(u16, ClipSuccessResponse), String> {
    let storage = app
        .try_state::<AppStorage>()
        .ok_or("Storage state not available")?;

    // 1. Strict Idempotency Verification
    if let Some(ref ikey) = idempotency_key {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        if let Ok(Some((existing_doc_id, stored_hash))) =
            database::check_idempotency_key(&conn, ikey)
        {
            // Reusing an idempotency key with a DIFFERENT payload must be rejected deterministically
            if stored_hash != payload_hash {
                return Err("Idempotency key reuse with different payload is rejected.".to_string());
            }

            if let Ok(Some(existing_doc)) = database::get_document_by_id(&conn, &existing_doc_id) {
                return Ok((
                    200,
                    ClipSuccessResponse {
                        status: "created".to_string(),
                        document_id: existing_doc.id,
                        filename: existing_doc.filename,
                        message: "Request already processed with this idempotency key".to_string(),
                        duplicate_detected: None,
                    },
                ));
            }
        }
    }

    let trimmed_title = payload.title.trim();
    let title = if trimmed_title.is_empty() {
        "Untitled Web Note"
    } else {
        trimmed_title
    };

    let raw_url = payload.url.trim().to_string();
    if raw_url.is_empty() {
        return Err("Source URL cannot be empty".to_string());
    }

    let canonical_url = normalize_canonical_url(&raw_url);

    // Prefer Markdown for stored document file, fall back to extracted plain text
    let document_text = if !payload.content_markdown.trim().is_empty() {
        payload.content_markdown.clone()
    } else if !payload.extracted_text.trim().is_empty() {
        payload.extracted_text.clone()
    } else {
        return Err("Web note must contain extractable text or markdown".to_string());
    };

    let text_bytes = document_text.as_bytes();
    let file_size = text_bytes.len() as i64;
    let content_hash = format!("{:x}", Sha256::digest(text_bytes));

    let sanitized_title = sanitize_web_title(title);
    let filename = format!("{}.md", sanitized_title);
    let duplicate_strategy = payload.duplicate_strategy.as_deref().unwrap_or("skip");

    // 2. Duplicate Detection
    let existing_doc_by_url = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::find_by_source_url(&conn, &canonical_url).map_err(|e| e.to_string())?
    };

    let existing_doc_by_hash = {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::find_by_content_hash(&conn, &content_hash).map_err(|e| e.to_string())?
    };

    let duplicate_doc =
        existing_doc_by_url.or_else(|| existing_doc_by_hash.filter(|d| d.file_type == "web"));

    if let Some(existing) = duplicate_doc {
        match duplicate_strategy {
            "skip" => {
                return Ok((
                    200,
                    ClipSuccessResponse {
                        status: "skipped".to_string(),
                        document_id: existing.id,
                        filename: existing.filename,
                        message: "Web note with identical URL or content already exists"
                            .to_string(),
                        duplicate_detected: Some(true),
                    },
                ));
            }
            "update" => {
                if existing.file_type != "web" {
                    return Err(format!(
                        "Refusing to overwrite existing '{}' file (id: {}) with a web note update.",
                        existing.file_type, existing.id
                    ));
                }

                let stored_path = PathBuf::from(&existing.stored_path);
                let backup_path = stored_path.with_extension("md.bak");

                // Create backup of existing file before updating
                if stored_path.exists() {
                    let _ = fs::copy(&stored_path, &backup_path);
                }

                // Atomic file update
                if let Err(e) = write_file_atomic(&stored_path, text_bytes) {
                    if backup_path.exists() {
                        let _ = fs::rename(&backup_path, &stored_path);
                    }
                    return Err(format!("Failed to update note file atomically: {}", e));
                }

                let metadata_str = payload.metadata.map(|m| m.to_string());

                // Atomic database update with idempotency within transaction
                let db_result = {
                    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
                    database::update_web_note_atomic(
                        &conn,
                        &existing.id,
                        &existing.filename,
                        &document_text,
                        file_size,
                        &content_hash,
                        payload.author.as_deref(),
                        payload.favicon_url.as_deref(),
                        metadata_str.as_deref(),
                        idempotency_key.as_deref(),
                        Some(payload_hash),
                    )
                };

                if let Err(e) = db_result {
                    // Restore backup file if database update fails
                    if backup_path.exists() {
                        let _ = fs::rename(&backup_path, &stored_path);
                    }
                    return Err(format!("Database atomic update failed: {}", e));
                }

                // Clean up backup upon success
                if backup_path.exists() {
                    let _ = fs::remove_file(&backup_path);
                }

                crate::ai::enqueue_document_for_indexing(app, &existing.id);
                crate::ai::enqueue_document_for_tagging(app, &existing.id);

                return Ok((
                    200,
                    ClipSuccessResponse {
                        status: "updated".to_string(),
                        document_id: existing.id,
                        filename: existing.filename,
                        message: "Existing Web Note updated successfully".to_string(),
                        duplicate_detected: Some(true),
                    },
                ));
            }
            "save_copy" => {}
            _ => {
                return Ok((
                    409,
                    ClipSuccessResponse {
                        status: "conflict".to_string(),
                        document_id: existing.id,
                        filename: existing.filename,
                        message: "Duplicate Web Note detected. Please specify duplicate_strategy: 'skip', 'update', or 'save_copy'".to_string(),
                        duplicate_detected: Some(true),
                    },
                ));
            }
        }
    }

    // 3. Insert New Web Note
    let doc_id = Uuid::new_v4().to_string();
    let final_filename = if duplicate_strategy == "save_copy" {
        format!(
            "{} (Copy {}).md",
            sanitized_title,
            Utc::now().format("%Y%m%d_%H%M%S")
        )
    } else {
        filename
    };

    let stored_filename = format!("{}_{}", doc_id, final_filename);
    let stored_path = storage.documents_dir.join(&stored_filename);

    // Atomic file write
    write_file_atomic(&stored_path, text_bytes)
        .map_err(|e| format!("Failed to write web note to storage: {}", e))?;

    let now = Utc::now().to_rfc3339();
    let metadata_str = payload.metadata.map(|m| m.to_string());

    let doc_record = DocumentRecord {
        id: doc_id.clone(),
        filename: final_filename.clone(),
        original_path: raw_url,
        stored_path: stored_path.to_string_lossy().to_string(),
        file_type: "web".to_string(),
        file_size,
        extracted_text: document_text,
        content_hash,
        imported_at: now.clone(),
        updated_at: now,
        source_url: Some(canonical_url),
        author: payload.author,
        favicon_url: payload.favicon_url,
        metadata_json: metadata_str,
    };

    // Atomic database insert with idempotency in ONE transaction
    {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        if let Err(e) = database::insert_web_note_atomic(
            &conn,
            &doc_record,
            idempotency_key.as_deref(),
            Some(payload_hash),
        ) {
            // Clean up written file on database failure
            let _ = fs::remove_file(&stored_path);
            return Err(format!("Database atomic transaction failed: {}", e));
        }
    }

    // 4. Asynchronously enqueue into indexing and AI tagging without holding locks
    crate::ai::enqueue_document_for_indexing(app, &doc_id);
    crate::ai::enqueue_document_for_tagging(app, &doc_id);

    Ok((
        201,
        ClipSuccessResponse {
            status: "created".to_string(),
            document_id: doc_id,
            filename: final_filename,
            message: "Web note clipped and enqueued for indexing successfully".to_string(),
            duplicate_detected: Some(false),
        },
    ))
}

fn sanitize_web_title(title: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let clean = sanitized.trim().replace("  ", " ");
    if clean.is_empty() {
        "Web_Note".to_string()
    } else {
        clean.chars().take(80).collect()
    }
}

fn write_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        status_code,
        status_text,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn write_cors_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    origin: &str,
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {} {}\r\n\
         Access-Control-Allow-Origin: {}\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Authorization, Content-Type, X-Idempotency-Key, X-ECHO-Client-Version\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        status_code,
        status_text,
        origin,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn write_response_with_retry(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    retry_after: u64,
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {} {}\r\n\
         Retry-After: {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        status_code,
        status_text,
        retry_after,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

// ================= Tauri IPC Commands =================

#[tauri::command]
pub fn get_clipper_pairing_info(
    storage: State<AppStorage>,
    controller: State<ClipperServerController>,
) -> Result<ClipperPairingInfo, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let token_res = get_or_create_pairing_token(&conn);
    let token_configured = token_res.is_ok() && !token_res.unwrap().is_empty();
    let authorized_exts = get_authorized_extensions(&conn);
    let legacy_auth_id = authorized_exts.first().map(|e| e.extension_id.clone());
    let port = controller
        .active_port
        .lock()
        .map(|p| *p)
        .unwrap_or(DEFAULT_CLIPPER_PORT);
    Ok(ClipperPairingInfo {
        port,
        token_configured,
        is_running: controller.is_running.load(Ordering::SeqCst),
        authorized_extension_id: legacy_auth_id,
        authorized_extensions: authorized_exts,
    })
}

#[tauri::command]
pub fn reveal_clipper_pairing_token(storage: State<AppStorage>) -> Result<String, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    get_or_create_pairing_token(&conn)
}

#[tauri::command]
pub fn regenerate_clipper_pairing_token(storage: State<AppStorage>) -> Result<String, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let seed = format!(
        "{}-{}-{}",
        Uuid::new_v4(),
        Utc::now().to_rfc3339(),
        std::process::id()
    );
    let token = hex::encode(Sha256::digest(seed.as_bytes()));

    let _ = security::store_clipper_token(&token);
    database::set_app_setting(&conn, "web_clipper_auth_token", &token)
        .map_err(|e| e.to_string())?;
    // Immediately invalidate ALL paired extension authorizations on token rotation
    let _ = database::set_app_setting(&conn, "web_clipper_authorized_extension_id", "");
    let _ = database::set_app_setting(&conn, "web_clipper_authorized_extensions", "[]");

    Ok(token)
}

#[tauri::command]
pub fn revoke_clipper_pairing_token(storage: State<AppStorage>) -> Result<bool, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    revoke_pairing_token(&conn)?;
    Ok(true)
}

#[tauri::command]
pub fn authorize_clipper_extension(
    storage: State<AppStorage>,
    extension_id: String,
    browser: Option<String>,
) -> Result<bool, String> {
    let clean_id = extension_id.trim().to_lowercase();
    if clean_id.len() != 32 || !clean_id.chars().all(|c| ('a'..='p').contains(&c)) {
        return Err(
            "Invalid extension ID format: Must be exactly 32 lowercase characters [a-p]"
                .to_string(),
        );
    }

    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    let mut exts = get_authorized_extensions(&conn);

    // If already authorized, update browser info and timestamp
    if let Some(existing) = exts.iter_mut().find(|e| e.extension_id == clean_id) {
        if browser.is_some() {
            existing.browser = browser;
        }
        existing.paired_at = Utc::now().to_rfc3339();
    } else {
        exts.push(AuthorizedExtensionInfo {
            extension_id: clean_id.clone(),
            browser: browser.or_else(|| Some("Chromium Browser".to_string())),
            paired_at: Utc::now().to_rfc3339(),
        });
    }

    let json_str = serde_json::to_string(&exts).map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "web_clipper_authorized_extensions", &json_str)
        .map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "web_clipper_authorized_extension_id", &clean_id)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn deauthorize_clipper_extension(
    storage: State<AppStorage>,
    extension_id: String,
) -> Result<bool, String> {
    let clean_id = extension_id.trim().to_lowercase();
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    // If extension_id is empty or "*", deauthorize all
    let updated_exts: Vec<AuthorizedExtensionInfo> = if clean_id.is_empty() || clean_id == "*" {
        Vec::new()
    } else {
        get_authorized_extensions(&conn)
            .into_iter()
            .filter(|e| e.extension_id != clean_id)
            .collect()
    };

    let json_str = serde_json::to_string(&updated_exts).map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "web_clipper_authorized_extensions", &json_str)
        .map_err(|e| e.to_string())?;

    let new_primary = updated_exts
        .first()
        .map(|e| e.extension_id.as_str())
        .unwrap_or("");
    database::set_app_setting(&conn, "web_clipper_authorized_extension_id", new_primary)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn test_clipper_server_connection(
    controller: State<ClipperServerController>,
) -> Result<serde_json::Value, String> {
    let port = controller
        .active_port
        .lock()
        .map(|p| *p)
        .unwrap_or(DEFAULT_CLIPPER_PORT);
    let is_running = controller.is_running.load(Ordering::SeqCst);

    if !is_running {
        return Ok(serde_json::json!({
            "reachable": false,
            "port": port,
            "is_running": false,
            "message": "Web Clipper server is currently stopped in Settings."
        }));
    }

    // Attempt loopback TCP connection directly on local interface
    match TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(1500),
    ) {
        Ok(_) => Ok(serde_json::json!({
            "reachable": true,
            "port": port,
            "is_running": true,
            "message": format!("Server listening and reachable on 127.0.0.1:{}", port)
        })),
        Err(e) => Ok(serde_json::json!({
            "reachable": false,
            "port": port,
            "is_running": true,
            "message": format!("Server failed socket connection on 127.0.0.1:{}: {}", port, e)
        })),
    }
}

#[tauri::command]
pub fn set_clipper_server_state(
    app: AppHandle,
    storage: State<AppStorage>,
    controller: State<ClipperServerController>,
    enabled: bool,
) -> Result<bool, String> {
    {
        let conn = storage.conn.lock().map_err(|e| e.to_string())?;
        database::set_app_setting(
            &conn,
            "web_clipper_server_enabled",
            if enabled { "true" } else { "false" },
        )
        .map_err(|e| e.to_string())?;
    }

    if enabled {
        start_clipper_server(app);
    } else {
        controller.should_stop.store(true, Ordering::SeqCst);
    }

    Ok(enabled)
}
