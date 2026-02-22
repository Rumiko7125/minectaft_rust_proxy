use sqlx::{Row, SqlitePool};
use std::net::SocketAddr;

/// Admin account
#[derive(Clone, Debug)]
pub struct AdminAccount {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub totp_secret: Option<String>,
    pub totp_bound: bool,
    pub preferred_locale: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// Create admin account
pub async fn create_admin(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO admin_accounts (username, password_hash) VALUES (?, ?)"
    )
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get admin by username
pub async fn get_admin_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<AdminAccount>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, username, password_hash, totp_secret, totp_bound, preferred_locale, created_at, last_login_at FROM admin_accounts WHERE username = ?"
    )
    .bind(username)
    .fetch_all(pool)
    .await?;

    if let Some(row) = rows.first() {
        Ok(Some(AdminAccount {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            totp_secret: row.get(3),
            totp_bound: row.get::<i32, _>(4) != 0,
            preferred_locale: row.get(5),
            created_at: row.get(6),
            last_login_at: row.get(7),
        }))
    } else {
        Ok(None)
    }
}

/// Get admin by ID
pub async fn get_admin_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<AdminAccount>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, username, password_hash, totp_secret, preferred_locale, totp_bound, created_at, last_login_at FROM admin_accounts WHERE id = ?"
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    if let Some(row) = rows.first() {
        Ok(Some(AdminAccount {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            totp_secret: row.get(3),
            totp_bound: row.get::<i32, _>(4) != 0,
            preferred_locale: row.get(5),
            created_at: row.get(6),
            last_login_at: row.get(7),
        }))
    } else {
        Ok(None)
    }
}

/// Update password
pub async fn update_password(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE admin_accounts SET password_hash = ? WHERE username = ?"
    )
    .bind(password_hash)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update TOTP secret
pub async fn update_totp(
    pool: &SqlitePool,
    username: &str,
    secret: &str,
    bound: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE admin_accounts SET totp_secret = ?, totp_bound = ? WHERE username = ?"
    )
    .bind(secret)
    .bind(if bound { 1 } else { 0 })
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

/// Clear TOTP (unbind)
pub async fn clear_totp(
    pool: &SqlitePool,
    username: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE admin_accounts SET totp_secret = NULL, totp_bound = 0 WHERE username = ?"
    )
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update last login time
pub async fn update_last_login(
    pool: &SqlitePool,
    username: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query(
        "UPDATE admin_accounts SET last_login_at = ? WHERE username = ?"
    )
    .bind(&now)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update locale preference
pub async fn update_locale(
    pool: &SqlitePool,
    username: &str,
    locale: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE admin_accounts SET preferred_locale = ? WHERE username = ?"
    )
    .bind(locale)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get all admins
pub async fn get_all_admins(
    pool: &SqlitePool,
) -> Result<Vec<AdminAccount>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, username, password_hash, totp_secret, totp_bound, preferred_locale, created_at, last_login_at FROM admin_accounts ORDER BY id ASC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| {
        AdminAccount {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            totp_secret: row.get(3),
            totp_bound: row.get::<i32, _>(4) != 0,
            preferred_locale: row.get(5),
            created_at: row.get(6),
            last_login_at: row.get(7),
        }
    }).collect())
}

/// Delete admin
pub async fn delete_admin(
    pool: &SqlitePool,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM admin_accounts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Check if admin exists
pub async fn has_admins(
    pool: &SqlitePool,
) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query("SELECT COUNT(*) FROM admin_accounts")
        .fetch_all(pool)
        .await?;
    let count: i64 = rows.first().map(|row| row.get(0)).unwrap_or(0);
    Ok(count > 0)
}
