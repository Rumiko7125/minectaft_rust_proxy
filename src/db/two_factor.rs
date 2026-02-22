use sqlx::{Row, SqlitePool};

/// Save 2FA secret
pub async fn save_secret(pool: &SqlitePool, username: &str, secret: &str) -> Result<(), sqlx::Error> {
    // Use UTC time
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("INSERT OR REPLACE INTO two_factor_secrets (username, secret, created_at) VALUES (?, ?, ?)")
        .bind(username)
        .bind(secret)
        .bind(&now)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get 2FA secret
pub async fn get_secret(pool: &SqlitePool, username: &str) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT secret FROM two_factor_secrets WHERE username = ?")
        .bind(username)
        .fetch_all(pool)
        .await?;
    Ok(rows.first().map(|row| row.get::<String, _>(0)))
}

/// Check if has 2FA
pub async fn has_2fa(pool: &SqlitePool, username: &str) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query("SELECT 1 FROM two_factor_secrets WHERE username = ?")
        .bind(username)
        .fetch_all(pool)
        .await?;
    Ok(!rows.is_empty())
}

/// Delete 2FA secret
pub async fn delete_secret(pool: &SqlitePool, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM two_factor_secrets WHERE username = ?")
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// 2FA user record
#[derive(Debug)]
pub struct TwoFactorUser {
    pub username: String,
    pub created_at: String,
}

/// Get all 2FA users
pub async fn get_all_2fa_users(pool: &SqlitePool) -> Result<Vec<TwoFactorUser>, sqlx::Error> {
    let rows = sqlx::query("SELECT username, created_at FROM two_factor_secrets ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(|row| TwoFactorUser {
        username: row.get(0),
        created_at: row.get(1),
    }).collect())
}
