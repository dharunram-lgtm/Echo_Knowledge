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
