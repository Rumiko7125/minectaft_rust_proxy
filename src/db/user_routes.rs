use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::net::SocketAddr;

/// User route rule
pub struct UserRoute {
    pub username: String,
    pub target_addr: String,
    pub target_port: u16,
}

/// Add or update user route
pub async fn upsert_user_route(
    pool: &SqlitePool,
    username: &str,
    target_addr: &str,
    target_port: u16,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query(
        "INSERT OR REPLACE INTO user_routes (username, target_addr, target_port, updated_at) VALUES (?, ?, ?, ?)"
    )
    .bind(username)
    .bind(target_addr)
    .bind(target_port)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete user route
pub async fn delete_user_route(pool: &SqlitePool, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_routes WHERE username = ?")
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// Clear all user routes
pub async fn clear_all_user_routes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_routes")
        .execute(pool)
        .await?;
    Ok(())
}

/// Get all user routes
pub async fn get_all_user_routes(pool: &SqlitePool) -> Result<HashMap<String, SocketAddr>, sqlx::Error> {
    let rows = sqlx::query("SELECT username, target_addr, target_port FROM user_routes")
        .fetch_all(pool)
        .await?;

    let mut map = HashMap::new();
    for row in rows {
        let username: String = row.get(0);
        let addr: String = row.get(1);
        let port: i32 = row.get(2);
        let addr = format!("{}:{}", addr, port);
        if let Ok(socket_addr) = addr.parse() {
            map.insert(username, socket_addr);
        }
    }
    Ok(map)
}

/// Get last update time
pub async fn get_last_updated(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT MAX(updated_at) FROM user_routes")
        .fetch_all(pool)
        .await?;
    if let Some(row) = rows.first() {
        let val: Option<String> = row.get(0);
        Ok(val)
    } else {
        Ok(None)
    }
}
