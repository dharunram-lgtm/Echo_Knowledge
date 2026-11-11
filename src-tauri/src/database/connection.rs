use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct AppStorage {
    pub base_dir: PathBuf,
    pub db_path: PathBuf,
    pub documents_dir: PathBuf,
    pub conn: Mutex<Connection>,
}

impl AppStorage {
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        // Resolve application data directory (e.g., %APPDATA%/com.echo.desktop or %APPDATA%/ECHO)
        let app_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

        let base_dir = app_dir.join("ECHO");
        let db_dir = base_dir.join("database");
        let documents_dir = base_dir.join("documents").join("imported");

        // Automatically create application directories
        fs::create_dir_all(&db_dir)
            .map_err(|e| format!("Failed to create database directory {:?}: {}", db_dir, e))?;
        fs::create_dir_all(&documents_dir)
            .map_err(|e| format!("Failed to create documents directory {:?}: {}", documents_dir, e))?;

        let db_path = db_dir.join("echo.db");

        // Open SQLite connection
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open SQLite database at {:?}: {}", db_path, e))?;

        // Configure performance and safety pragmas
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(|e| format!("Failed to configure SQLite pragmas: {}", e))?;

        // Run migrations
        super::migrations::run_migrations(&conn)
            .map_err(|e| format!("Database migration failed: {}", e))?;

        Ok(Self {
            base_dir,
            db_path,
            documents_dir,
            conn: Mutex::new(conn),
        })
    }

    pub fn is_path_in_managed_storage(&self, path: &Path) -> bool {
        path.starts_with(&self.documents_dir)
    }
}

