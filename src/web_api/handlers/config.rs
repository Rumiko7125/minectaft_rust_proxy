use crate::auth::TwoFactorManager;
use crate::config::Config;
use crate::db;
use crate::proxy::Proxy;
use crate::web_api::handlers::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

type ApiResult = std::result::Result<Json<Value>, (StatusCode, Json<Value>)>;

fn error_response(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"code": code, "message": message})),
    )
}

fn ok_response(data: Value) -> ApiResult {
    Ok(Json(data))
}

/// GET /api/v1/config - Get all config
pub async fn get_config(State(state): State<Arc<AppState>>) -> ApiResult {
    let config = state.proxy.config.read().await;
    let ac = state.proxy.access_control.read().await;

    ok_response(json!({
        "local_address": config.local_address,
        "local_port": config.local_port,
        "whitelist_enabled": ac.whitelist_enabled,
        "enable_2fa": config.enable_2fa,
        "two_factor_session_hours": config.two_factor_session_hours,
        "two_factor_issuer": config.two_factor_issuer,
        "web_api_enable": config.web_api_enable,
        "web_api_address": config.web_api_address,
        "web_api_port": config.web_api_port,
        "language": config.language,
        "log_dir": config.log_dir,
        "show_log_level": config.show_log_level,
        "save_log_level": config.save_log_level,
        "allow_input": config.allow_input,
        "trusted_domain": config.trusted_domain,
        "refresh_interval_secs": 30
    }))
}

/// PATCH /api/v1/config - Batch update config
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> ApiResult {
    let keys = [
        "local_address",
        "local_port",
        "whitelist_enabled",
        "enable_2fa",
        "two_factor_session_hours",
        "two_factor_issuer",
        "web_api_enable",
        "web_api_address",
        "web_api_port",
        "language",
        "log_dir",
        "show_log_level",
        "save_log_level",
        "allow_input",
        "trusted_domain",
        "refresh_interval_secs",
    ];

    let mut updates: Vec<(String, String)> = Vec::new();

    for key in keys {
        if let Some(value) = body.get(key) {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            updates.push((key.to_string(), value_str));
        }
    }

    // Update database
    for (key, value_str) in &updates {
        if let Err(e) = db::settings::set_setting(&state.proxy.db_pool, key, value_str).await {
            tracing::warn!("Failed to set config {}: {}", key, e);
        }
    }

    // Handle whitelist_enabled separate update for access control
    if let Some(Value::Bool(enabled)) = body.get("whitelist_enabled") {
        state.proxy.access_control.write().await.whitelist_enabled = *enabled;
    } else if let Some(Value::String(s)) = body.get("whitelist_enabled") {
        if let Ok(enabled) = s.parse::<bool>() {
            state.proxy.access_control.write().await.whitelist_enabled = enabled;
        }
    }

    // Reload config to memory
    let new_config = match Config::load(&state.proxy.db_pool).await {
        Ok(c) => c,
        Err(e) => return Err(error_response("RELOAD_FAILED", &e.to_string())),
    };

    // Sync 2FA manager based on new config (enable/disable)
    {
        let mut tf_guard = state.proxy.two_factor.write().await;
        if new_config.enable_2fa && tf_guard.is_none() {
            match TwoFactorManager::new(
                &state.proxy.db_pool,
                new_config.two_factor_session_hours,
                &new_config.two_factor_issuer,
            ).await {
                Ok(tf) => {
                    tracing::info!("2FA enabled via config update");
                    *tf_guard = Some(tf);
                }
                Err(e) => tracing::error!("Failed to initialize 2FA: {}", e),
            }
        } else if !new_config.enable_2fa && tf_guard.is_some() {
            tracing::info!("2FA disabled via config update");
            *tf_guard = None;
        }
    }

    // Dynamically update log level (no restart needed)
    crate::CONSOLE_LOG_LEVEL.store(new_config.show_log_level, std::sync::atomic::Ordering::Relaxed);
    crate::FILE_LOG_LEVEL.store(new_config.save_log_level, std::sync::atomic::Ordering::Relaxed);
    // Refresh tracing callsite cache, ensure log level changes take effect immediately
    tracing::callsite::rebuild_interest_cache();

    *state.proxy.config.write().await = new_config;

    tracing::info!("Config updated successfully");
    ok_response(json!({"status": "ok", "message": "Config updated"}))
}

/// POST /api/v1/config/reload - Hot reload immediately
pub async fn reload_config(State(state): State<Arc<AppState>>) -> ApiResult {
    let new_config = match Config::load(&state.proxy.db_pool).await {
        Ok(c) => c,
        Err(e) => return Err(error_response("RELOAD_FAILED", &e.to_string())),
    };

    *state.proxy.config.write().await = new_config;

    ok_response(json!({"status": "ok", "message": "Config reloaded"}))
}
