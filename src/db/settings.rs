use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

/// Get all settings
pub async fn get_all_settings(pool: &SqlitePool) -> Result<HashMap<String, String>, sqlx::Error> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(|row| {
        let key: String = row.get(0);
        let value: String = row.get(1);
        (key, value)
    }).collect())
}

/// Get single setting
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_all(pool)
        .await?;
    Ok(rows.first().map(|row| row.get::<String, _>(0)))
}

/// Set single setting
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let start = std::time::Instant::now();
    sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, ?)")
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(pool)
        .await?;
    tracing::info!("[DB] set_setting {} took {:?}", key, start.elapsed());
    Ok(())
}

/// Get last update time
pub async fn get_last_updated(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT MAX(updated_at) FROM settings")
        .fetch_all(pool)
        .await?;
    if let Some(row) = rows.first() {
        let val: Option<String> = row.get(0);
        Ok(val)
    } else {
        Ok(None)
    }
}
