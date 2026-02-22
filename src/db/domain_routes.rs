use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Domain route rule
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoute {
    pub id: i64,
    pub pattern: String,
    pub target_addr: String,
    pub target_port: u16,
    pub priority: i32,
    pub backend_id: Option<i64>,  // Associated backend server ID
}

/// Add domain route (with backend_id)
pub async fn add_domain_route(
    pool: &SqlitePool,
    pattern: &str,
    target_addr: &str,
    target_port: u16,
    priority: i32,
    backend_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO domain_routes (pattern, target_addr, target_port, priority, backend_id) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(pattern)
    .bind(target_addr)
    .bind(target_port)
    .bind(priority)
    .bind(backend_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Update domain route
pub async fn update_domain_route(
    pool: &SqlitePool,
    id: i64,
    pattern: &str,
    target_addr: &str,
    target_port: u16,
    priority: i32,
    backend_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE domain_routes SET pattern = ?, target_addr = ?, target_port = ?, priority = ?, backend_id = ? WHERE id = ?"
    )
    .bind(pattern)
    .bind(target_addr)
    .bind(target_port as i32)
    .bind(priority)
    .bind(backend_id)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete domain route by ID
pub async fn delete_domain_route(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM domain_routes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Clear all domain routes
pub async fn clear_all_domain_routes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM domain_routes")
        .execute(pool)
        .await?;
    Ok(())
}

/// Get all domain routes (sorted by priority)
pub async fn get_all_domain_routes(pool: &SqlitePool) -> Result<Vec<DomainRoute>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, pattern, target_addr, target_port, priority, backend_id FROM domain_routes ORDER BY priority DESC, id ASC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| {
        DomainRoute {
            id: row.get(0),
            pattern: row.get(1),
            target_addr: row.get(2),
            target_port: row.get::<i32, _>(3) as u16,
            priority: row.get(4),
            backend_id: row.get(5),
        }
    }).collect())
}

/// Get last update time
pub async fn get_last_updated(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT MAX(updated_at) FROM domain_routes")
        .fetch_all(pool)
        .await?;
    if let Some(row) = rows.first() {
        let val: Option<String> = row.get(0);
        Ok(val)
    } else {
        Ok(None)
    }
}

/// Get backend ID associated with domain route
pub async fn get_backend_id(pool: &SqlitePool, id: i64) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query("SELECT backend_id FROM domain_routes WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| r.get(0)))
}
