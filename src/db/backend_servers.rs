use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};

/// Backend server structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendServer {
    pub id: i64,
    pub name: String,
    pub remote_address: String,
    pub remote_port: i32,
    pub max_player: i32,
    pub motd_json: Option<String>,
    pub limbo_message: Option<String>,
    pub log_dir: Option<String>,
    pub show_log_level: i32,
    pub save_log_level: i32,
    pub is_default: bool,
    pub enabled: bool,
    pub maintenance: bool,
    pub maintenance_message: Option<String>,
    pub ping_passthrough: bool,
    pub motd_passthrough: bool,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for creating backend server
#[derive(Debug, Deserialize)]
pub struct CreateBackendRequest {
    pub name: String,
    pub remote_address: String,
    pub remote_port: Option<i32>,
    pub max_player: Option<i32>,
    pub motd_json: Option<String>,
    pub limbo_message: Option<String>,
    pub log_dir: Option<String>,
    pub show_log_level: Option<i32>,
    pub save_log_level: Option<i32>,
    pub enabled: Option<bool>,
    pub maintenance: Option<bool>,
    pub maintenance_message: Option<String>,
    pub ping_passthrough: Option<bool>,
    pub motd_passthrough: Option<bool>,
    pub language: Option<String>,
}

/// Request body for updating backend server
#[derive(Debug, Deserialize)]
pub struct UpdateBackendRequest {
    pub name: Option<String>,
    pub remote_address: Option<String>,
    pub remote_port: Option<i32>,
    pub max_player: Option<i32>,
    pub motd_json: Option<String>,
    pub limbo_message: Option<String>,
    pub log_dir: Option<String>,
    pub show_log_level: Option<i32>,
    pub save_log_level: Option<i32>,
    pub enabled: Option<bool>,
    pub maintenance: Option<bool>,
    pub maintenance_message: Option<String>,
    pub ping_passthrough: Option<bool>,
    pub motd_passthrough: Option<bool>,
    pub language: Option<String>,
}

/// Parse BackendServer from row (using column name index)
fn row_to_backend(row: &sqlx::sqlite::SqliteRow) -> BackendServer {
    BackendServer {
        id: row.get("id"),
        name: row.get("name"),
        remote_address: row.get("remote_address"),
        remote_port: row.get("remote_port"),
        max_player: row.get("max_player"),
        motd_json: row.get("motd_json"),
        limbo_message: row.get("limbo_message"),
        log_dir: row.get("log_dir"),
        show_log_level: row.get("show_log_level"),
        save_log_level: row.get("save_log_level"),
        is_default: row.get::<i32, _>("is_default") != 0,
        enabled: row.get::<i32, _>("enabled") != 0,
        maintenance: row.get::<i32, _>("maintenance") != 0,
        maintenance_message: row.get("maintenance_message"),
        ping_passthrough: row.get::<i32, _>("ping_passthrough") != 0,
        motd_passthrough: row.get::<i32, _>("motd_passthrough") != 0,
        language: row.try_get("language").unwrap_or_else(|_| "en".to_string()),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

const SELECT_COLS: &str = r#"
    id, name, remote_address, remote_port, max_player,
    motd_json, limbo_message, log_dir,
    show_log_level, save_log_level,
    is_default, enabled, maintenance, maintenance_message,
    ping_passthrough, motd_passthrough, language,
    created_at, updated_at
"#;

/// Create backend server
pub async fn create_backend_server(
    pool: &SqlitePool,
    req: &CreateBackendRequest,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO backend_servers (
            name, remote_address, remote_port, max_player,
            motd_json, limbo_message, log_dir,
            show_log_level, save_log_level, enabled,
            maintenance, maintenance_message, ping_passthrough, motd_passthrough, language
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&req.name)
    .bind(&req.remote_address)
    .bind(req.remote_port.unwrap_or(25565))
    .bind(req.max_player.unwrap_or(-1))
    .bind(&req.motd_json)
    .bind(&req.limbo_message)
    .bind(&req.log_dir)
    .bind(req.show_log_level.unwrap_or(0))
    .bind(req.save_log_level.unwrap_or(0))
    .bind(req.enabled.unwrap_or(true) as i32)
    .bind(req.maintenance.unwrap_or(false) as i32)
    .bind(&req.maintenance_message)
    .bind(req.ping_passthrough.unwrap_or(false) as i32)
    .bind(req.motd_passthrough.unwrap_or(false) as i32)
    .bind(req.language.clone().unwrap_or_else(|| "en".to_string()))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get all backend servers
pub async fn get_all_backend_servers(pool: &SqlitePool) -> Result<Vec<BackendServer>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM backend_servers ORDER BY is_default DESC, id ASC",
        SELECT_COLS
    );
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    Ok(rows.iter().map(row_to_backend).collect())
}

/// Get single backend server
pub async fn get_backend_server(pool: &SqlitePool, id: i64) -> Result<Option<BackendServer>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM backend_servers WHERE id = ?",
        SELECT_COLS
    );
    let row = sqlx::query(&sql).bind(id).fetch_optional(pool).await?;
    Ok(row.as_ref().map(row_to_backend))
}

/// Update backend server
pub async fn update_backend_server(
    pool: &SqlitePool,
    id: i64,
    req: &UpdateBackendRequest,
) -> Result<(), sqlx::Error> {
    let existing = get_backend_server(pool, id).await?;
    let Some(existing) = existing else {
        return Err(sqlx::Error::RowNotFound);
    };

    let name = req.name.clone().unwrap_or(existing.name);
    let remote_address = req.remote_address.clone().unwrap_or(existing.remote_address);
    let remote_port = req.remote_port.unwrap_or(existing.remote_port);
    let max_player = req.max_player.unwrap_or(existing.max_player);
    let motd_json = req.motd_json.clone().or(existing.motd_json);
    let limbo_message = req.limbo_message.clone().or(existing.limbo_message);
    let log_dir = req.log_dir.clone().or(existing.log_dir);
    let show_log_level = req.show_log_level.unwrap_or(existing.show_log_level);
    let save_log_level = req.save_log_level.unwrap_or(existing.save_log_level);
    let enabled = req.enabled.map(|e| e as i32).unwrap_or(existing.enabled as i32);
    let maintenance = req.maintenance.map(|m| m as i32).unwrap_or(existing.maintenance as i32);
    let maintenance_message = req.maintenance_message.clone().or(existing.maintenance_message);
    let ping_passthrough = req.ping_passthrough.map(|v| v as i32).unwrap_or(existing.ping_passthrough as i32);
    let motd_passthrough = req.motd_passthrough.map(|v| v as i32).unwrap_or(existing.motd_passthrough as i32);
    let language = req.language.clone().unwrap_or(existing.language);

    sqlx::query(
        r#"
        UPDATE backend_servers SET
            name = ?,
            remote_address = ?,
            remote_port = ?,
            max_player = ?,
            motd_json = ?,
            limbo_message = ?,
            log_dir = ?,
            show_log_level = ?,
            save_log_level = ?,
            enabled = ?,
            maintenance = ?,
            maintenance_message = ?,
            ping_passthrough = ?,
            motd_passthrough = ?,
            language = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(remote_address)
    .bind(remote_port)
    .bind(max_player)
    .bind(motd_json)
    .bind(limbo_message)
    .bind(log_dir)
    .bind(show_log_level)
    .bind(save_log_level)
    .bind(enabled)
    .bind(maintenance)
    .bind(maintenance_message)
    .bind(ping_passthrough)
    .bind(motd_passthrough)
    .bind(language)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete backend server
pub async fn delete_backend_server(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    let routes = sqlx::query("SELECT COUNT(*) FROM domain_routes WHERE backend_id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = routes {
        let count: i32 = row.get(0);
        if count > 0 {
            return Err(sqlx::Error::RowNotFound);
        }
    }

    sqlx::query("DELETE FROM backend_servers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Enable backend server
pub async fn enable_backend_server(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE backend_servers SET enabled = 1, updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Disable backend server
pub async fn disable_backend_server(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT is_default FROM backend_servers WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    let Some(row) = row else {
        return Err(sqlx::Error::RowNotFound);
    };

    let is_default: i32 = row.get(0);
    if is_default == 1 {
        return Err(sqlx::Error::Protocol("cannot_disable_default".into()));
    }

    sqlx::query("UPDATE backend_servers SET enabled = 0, updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Unset default backend (clear all is_default flags)
pub async fn unset_default_backend(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE backend_servers SET is_default = 0 WHERE is_default = 1")
        .execute(pool)
        .await?;
    Ok(())
}

/// Toggle maintenance mode
pub async fn toggle_maintenance(pool: &SqlitePool, id: i64, maintenance: bool, message: Option<String>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE backend_servers SET maintenance = ?, maintenance_message = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(maintenance as i32)
    .bind(message)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Set default backend
pub async fn set_default_backend(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE backend_servers SET is_default = 0")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE backend_servers SET is_default = 1, updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get default backend
pub async fn get_default_backend(pool: &SqlitePool) -> Result<Option<BackendServer>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM backend_servers WHERE is_default = 1 AND enabled = 1 LIMIT 1",
        SELECT_COLS
    );
    let row = sqlx::query(&sql).fetch_optional(pool).await?;
    Ok(row.as_ref().map(row_to_backend))
}

/// Get enabled backend servers
pub async fn get_enabled_backends(pool: &SqlitePool) -> Result<Vec<BackendServer>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM backend_servers WHERE enabled = 1 ORDER BY is_default DESC, id ASC",
        SELECT_COLS
    );
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    Ok(rows.iter().map(row_to_backend).collect())
}

/// Get backend server by ID (internal use)
pub async fn get_backend_by_id(pool: &SqlitePool, id: i64) -> Result<Option<BackendServer>, sqlx::Error> {
    get_backend_server(pool, id).await
}

/// Check if domain routes are associated with backend, return used route patterns
pub async fn get_backend_routes(pool: &SqlitePool, backend_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT pattern FROM domain_routes WHERE backend_id = ?")
        .bind(backend_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

/// Check if domain routes are associated with backend
pub async fn is_backend_in_use(pool: &SqlitePool, backend_id: i64) -> Result<bool, sqlx::Error> {
    let routes = get_backend_routes(pool, backend_id).await?;
    Ok(!routes.is_empty())
}

/// Initialize default backend server (not auto-created, admin must configure manually)
pub async fn init_default_backend(_pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Do not auto-create default backend, admin must manually configure backend server via web panel
    tracing::info!("Backend server initialization skipped - admin must configure via web panel");
    Ok(())
}
