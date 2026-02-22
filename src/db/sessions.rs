use sqlx::{Row, SqlitePool};

/// Create new session, return session_id
pub async fn create_session(
    pool: &SqlitePool,
    username: &str,
    uuid: &str,
    backend_addr: &str,
    backend_port: u16,
    protocol_version: i32,
) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let result = sqlx::query(
        "INSERT INTO player_sessions (username, uuid, backend_addr, backend_port, protocol_version, login_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(username)
    .bind(uuid)
    .bind(backend_addr)
    .bind(backend_port)
    .bind(protocol_version)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Update session end information
pub async fn update_session(
    pool: &SqlitePool,
    session_id: i64,
    upload_bytes: u64,
    download_bytes: u64,
    kick_reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query(
        "UPDATE player_sessions SET logout_at = ?, upload_bytes = ?, download_bytes = ?, kick_reason = ? WHERE id = ?"
    )
    .bind(&now)
    .bind(upload_bytes as i64)
    .bind(download_bytes as i64)
    .bind(kick_reason)
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get session statistics
pub async fn get_session_stats(pool: &SqlitePool, username: &str) -> Result<Option<(i64, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT SUM(upload_bytes), SUM(download_bytes) FROM player_sessions WHERE username = ?"
    )
    .bind(username)
    .fetch_all(pool)
    .await?;

    if let Some(row) = rows.first() {
        let upload: Option<i64> = row.get(0);
        let download: Option<i64> = row.get(1);
        Ok(Some((upload.unwrap_or(0), download.unwrap_or(0))))
    } else {
        Ok(None)
    }
}
