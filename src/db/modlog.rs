use sqlx::{Row, SqlitePool};

/// Moderation log structure
#[derive(Debug, Clone)]
pub struct ModLog {
    pub id: i64,
    pub action: String,
    pub target: String,
    pub operator: String,
    pub reason: Option<String>,
    pub created_at: String,
}

/// Write moderation log
pub async fn log_action(
    pool: &SqlitePool,
    action: &str,
    target: &str,
    operator: &str,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO moderation_logs (action, target, operator, reason) VALUES (?, ?, ?, ?)"
    )
    .bind(action)
    .bind(target)
    .bind(operator)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query moderation logs
pub async fn get_logs(
    pool: &SqlitePool,
    target: Option<&str>,
    limit: usize,
) -> Result<Vec<ModLog>, sqlx::Error> {
    let rows = if let Some(target) = target {
        sqlx::query(
            "SELECT id, action, target, operator, reason, created_at FROM moderation_logs WHERE target = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(target)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT id, action, target, operator, reason, created_at FROM moderation_logs ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(rows.iter().map(|row| {
        ModLog {
            id: row.get(0),
            action: row.get(1),
            target: row.get(2),
            operator: row.get(3),
            reason: row.get(4),
            created_at: row.get(5),
        }
    }).collect())
}
