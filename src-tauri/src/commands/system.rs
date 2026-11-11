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
        name: "ECHO Knowledge".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        phase: "V1.0.0 Release".to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

