use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub phase: String,
    pub platform: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "ECHO".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        phase: "Phase 1: Project Foundation & Architecture".to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

