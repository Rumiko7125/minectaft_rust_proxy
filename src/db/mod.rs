use sqlx::{Row, SqlitePool};
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;

/// Initialize database connection pool
pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let db_path_obj = Path::new(db_path);
    if let Some(parent) = db_path_obj.parent() {
        let parent_str = parent.to_string_lossy();
        if !parent_str.is_empty() && !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::warn!("Failed to create database directory: {}", e);
            }
        }
    }

    let db_path_abs = if db_path_obj.is_absolute() {
        db_path.to_string()
    } else {
        std::fs::canonicalize(db_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| db_path.to_string())
    };

    tracing::info!("Connecting to database: {}", db_path_abs);

    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect(&format!("sqlite:{}?mode=rwc", db_path_abs))
        .await?;

    // Run migrations (set_ignore_missing allows existing database to ignore missing old migration files)
    sqlx::migrate!("./migrations")
        .set_ignore_missing(true)
        .run(&pool)
        .await?;

    Ok(pool)
}

/// Initialize default config (if settings table is empty)
pub async fn init_default_settings(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("SELECT COUNT(*) FROM settings")
        .fetch_all(pool)
        .await?;
    let count: i64 = rows.first().map(|row| row.get(0)).unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    let defaults = [
        ("local_address", "0.0.0.0"),
        ("local_port", "25565"),
        ("whitelist_enabled", "false"),
        ("web_api_enable", "true"),
        ("web_api_address", "127.0.0.1"),
        ("web_api_port", "20220"),
        ("enable_2fa", "false"),
        ("two_factor_session_hours", "12"),
        ("two_factor_issuer", "MinecraftProxy"),
        ("language", "en"),
        ("log_dir", "./logs"),
        ("show_log_level", "3"),
        ("save_log_level", "3"),
        ("refresh_interval_secs", "30"),
        ("allow_input", "true"),
    ];

    for (key, value) in defaults {
        let _ = sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await;
    }

    tracing::info!("Initialized default settings in database");
    Ok(())
}

pub mod access;
pub mod admin;
pub mod backend_servers;
pub mod domain_routes;
pub mod modlog;
pub mod sessions;
pub mod settings;
pub mod two_factor;
