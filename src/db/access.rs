use sqlx::{Row, SqlitePool};

/// Add to whitelist
pub async fn add_whitelist(pool: &SqlitePool, username: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("INSERT OR REPLACE INTO whitelist (username, added_at) VALUES (?, ?)")
        .bind(username)
        .bind(&now)
        .execute(pool)
        .await?;
    Ok(())
}

/// Remove from whitelist
pub async fn remove_whitelist(pool: &SqlitePool, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM whitelist WHERE username = ?")
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// Whitelist entry
#[derive(Debug)]
pub struct WhitelistEntry {
    pub username: String,
    pub added_at: String,
}

/// Get all whitelist users
pub async fn get_whitelist(pool: &SqlitePool) -> Result<Vec<WhitelistEntry>, sqlx::Error> {
    let rows = sqlx::query("SELECT username, added_at FROM whitelist ORDER BY added_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(|row| WhitelistEntry {
        username: row.get(0),
        added_at: row.get(1),
    }).collect())
}

/// Check if in whitelist
pub async fn is_whitelisted(pool: &SqlitePool, username: &str) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query("SELECT 1 FROM whitelist WHERE username = ?")
        .bind(username)
        .fetch_all(pool)
        .await?;
    Ok(!rows.is_empty())
}

/// Add to blacklist
pub async fn add_blacklist(pool: &SqlitePool, username: &str, reason: Option<&str>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("INSERT OR REPLACE INTO blacklist (username, reason, added_at) VALUES (?, ?, ?)")
        .bind(username)
        .bind(reason)
        .bind(&now)
        .execute(pool)
        .await?;
    Ok(())
}

/// Remove from blacklist
pub async fn remove_blacklist(pool: &SqlitePool, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM blacklist WHERE username = ?")
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// Blacklist entry
#[derive(Debug)]
pub struct BlacklistEntry {
    pub username: String,
    pub reason: Option<String>,
    pub added_at: String,
}

/// Get all blacklist users
pub async fn get_blacklist(pool: &SqlitePool) -> Result<Vec<BlacklistEntry>, sqlx::Error> {
    let rows = sqlx::query("SELECT username, reason, added_at FROM blacklist ORDER BY added_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(|row| BlacklistEntry {
        username: row.get(0),
        reason: row.get(1),
        added_at: row.get(2),
    }).collect())
}

/// Check if in blacklist
pub async fn is_blacklisted(pool: &SqlitePool, username: &str) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query("SELECT 1 FROM blacklist WHERE username = ?")
        .bind(username)
        .fetch_all(pool)
        .await?;
    Ok(!rows.is_empty())
}

/// Get blacklist reason
pub async fn get_blacklist_reason(pool: &SqlitePool, username: &str) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT reason FROM blacklist WHERE username = ?")
        .bind(username)
        .fetch_all(pool)
        .await?;
    if let Some(row) = rows.first() {
        Ok(row.get(0))
    } else {
        Ok(None)
    }
}
