use chrono::Utc;
use rusqlite::{params, Connection, Result};

pub fn get_app_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1;")?;
    let mut rows = stmt.query([key])?;

    if let Some(row) = rows.next()? {
        let val: String = row.get(0)?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

pub fn set_app_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3;",
        params![key, value, now],
    )?;
    Ok(())
}

