//! Security Module
//!
//! Enforces isolation, secret storage boundary (Windows Credential Manager via keyring),
//! and sandboxed IPC boundaries for ECHO.
//! API keys and provider tokens are handled strictly within the backend.
//! Keys are NEVER stored in SQLite, NEVER exposed to JavaScript, and NEVER logged.

use keyring::Entry;

pub const SERVICE_NAME: &str = "echo.knowledge.ai";

/// Stores an API key for the given provider in Windows Credential Manager
pub fn store_api_key(provider: &str, key: &str) -> Result<(), String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let entry = Entry::new(SERVICE_NAME, provider)
        .map_err(|e| format!("Failed to initialize credential entry: {}", e))?;
    entry
        .set_password(trimmed)
        .map_err(|e| format!("Failed to store secret in Windows Credential Manager: {}", e))?;
    Ok(())
}

/// Retrieves the raw API key for internal Rust backend use only.
/// Never send the return value of this function to the frontend.
pub fn get_api_key(provider: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, provider)
        .map_err(|e| format!("Failed to initialize credential entry: {}", e))?;
    match entry.get_password() {
        Ok(pass) => {
            let trimmed = pass.trim().to_string();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed))
            }
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve secret from credential store: {}", e)),
    }
}

/// Removes a saved API key from Windows Credential Manager
pub fn delete_api_key(provider: &str) -> Result<bool, String> {
    let entry = Entry::new(SERVICE_NAME, provider)
        .map_err(|e| format!("Failed to initialize credential entry: {}", e))?;
    match entry.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("Failed to remove secret from credential store: {}", e)),
    }
}

/// Checks whether an API key is configured for the given provider without exposing it
pub fn has_api_key(provider: &str) -> Result<bool, String> {
    match get_api_key(provider) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    }
}

pub const WEB_CLIPPER_SERVICE_TARGET: &str = "web_clipper_token";

/// Stores the Web Clipper pairing token in Windows Credential Manager
pub fn store_clipper_token(token: &str) -> Result<(), String> {
    store_api_key(WEB_CLIPPER_SERVICE_TARGET, token)
}

/// Retrieves the Web Clipper pairing token from Windows Credential Manager
pub fn get_clipper_token() -> Result<Option<String>, String> {
    get_api_key(WEB_CLIPPER_SERVICE_TARGET)
}

/// Deletes the Web Clipper pairing token from Windows Credential Manager
pub fn delete_clipper_token() -> Result<bool, String> {
    delete_api_key(WEB_CLIPPER_SERVICE_TARGET)
}

/// Timing-safe comparison of two strings to prevent timing attacks
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    if a_bytes.len() != b_bytes.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    result == 0
}
