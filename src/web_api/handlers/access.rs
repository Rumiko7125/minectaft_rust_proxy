use crate::db;
use crate::proxy::Proxy;
use crate::web_api::handlers::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use sqlx::Row;
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

/// GET /api/v1/whitelist - Get whitelist
pub async fn get_whitelist(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Query database first (outside lock)
    let total = sqlx::query("SELECT COUNT(*) FROM whitelist")
        .fetch_one(&state.proxy.db_pool)
        .await
        .map(|row| row.get::<i64, _>(0))
        .unwrap_or(0);

    // Paginated query
    let list = sqlx::query(
        "SELECT username, added_at FROM whitelist ORDER BY added_at DESC LIMIT ? OFFSET ?"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.proxy.db_pool)
    .await
    .unwrap_or_default();

    let list_json: Vec<Value> = list.iter().map(|row| {
        json!({
            "username": row.get::<String, _>(0),
            "added_at": row.get::<String, _>(1)
        })
    }).collect();

    // Get whitelist enabled status only (no async methods)
    let ac = state.proxy.access_control.read().await;
    let whitelist_enabled = ac.whitelist_enabled;

    ok_response(json!({
        "enabled": whitelist_enabled,
        "list": list_json,
        "total": total,
        "limit": limit,
        "offset": offset
    }))
}

/// POST /api/v1/whitelist - Add whitelist
pub async fn add_whitelist(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    if username.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Username required"));
    }

    let mut ac = state.proxy.access_control.write().await;
    ac.add_whitelist(username).await;
    ok_response(json!({"added": username}))
}

/// DELETE /api/v1/whitelist/:username - Remove whitelist
pub async fn remove_whitelist(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(username): axum::extract::Path<String>,
) -> ApiResult {
    let mut ac = state.proxy.access_control.write().await;
    ac.remove_whitelist(&username).await;
    ok_response(json!({"removed": username}))
}

/// PATCH /api/v1/whitelist - Toggle whitelist status (also persist to database)
pub async fn toggle_whitelist(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

    // Persist to database
    if let Err(e) = crate::db::settings::set_setting(
        &state.proxy.db_pool,
        "whitelist_enabled",
        &enabled.to_string(),
    ).await {
        tracing::warn!("Failed to persist whitelist_enabled to DB: {}", e);
    }

    // Update memory state
    state.proxy.access_control.write().await.whitelist_enabled = enabled;

    tracing::info!("Whitelist {}", if enabled { "enabled" } else { "disabled" });
    ok_response(json!({"enabled": enabled}))
}

/// GET /api/v1/blacklist - Get blacklist
pub async fn get_blacklist(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Get total count
    let total: i64 = sqlx::query("SELECT COUNT(*) FROM blacklist")
        .fetch_one(&state.proxy.db_pool)
        .await
        .map(|row| row.get(0))
        .unwrap_or(0);

    // Paginated query
    let list = sqlx::query(
        "SELECT username, reason, added_at FROM blacklist ORDER BY added_at DESC LIMIT ? OFFSET ?"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.proxy.db_pool)
    .await
    .unwrap_or_default();

    let list_json: Vec<Value> = list.iter().map(|row| {
        json!({
            "username": row.get::<String, _>(0),
            "reason": row.get::<Option<String>, _>(1),
            "added_at": row.get::<String, _>(2)
        })
    }).collect();

    ok_response(json!({"list": list_json, "total": total, "limit": limit, "offset": offset}))
}

/// POST /api/v1/blacklist - Add blacklist
pub async fn add_blacklist(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let reason = body.get("reason").and_then(|v| v.as_str());

    if username.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Username required"));
    }

    let mut ac = state.proxy.access_control.write().await;

    // If in whitelist, remove first (blacklist has higher priority than whitelist)
    // Use case-insensitive comparison
    let in_whitelist = ac.whitelist.iter().any(|w| w.eq_ignore_ascii_case(username));
    if in_whitelist {
        // Find actual key in whitelist and remove
        if let Some(existing) = ac.whitelist.iter().find(|w| w.eq_ignore_ascii_case(username)).cloned() {
            ac.remove_whitelist(&existing).await;
        }
    }

    ac.add_blacklist(username, reason).await;

    // Record audit log
    db::modlog::log_action(&state.proxy.db_pool, "ban", username, "webapi", reason).await.ok();

    // Kick if player is online
    state.proxy.kick_player(username);

    ok_response(json!({"added": username}))
}

/// DELETE /api/v1/blacklist/:username - Remove blacklist
pub async fn remove_blacklist(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(username): axum::extract::Path<String>,
) -> ApiResult {
    let mut ac = state.proxy.access_control.write().await;
    ac.remove_blacklist(&username).await;

    // Record audit log
    db::modlog::log_action(&state.proxy.db_pool, "pardon", &username, "webapi", None).await.ok();

    ok_response(json!({"removed": username}))
}

/// POST /api/v1/kick - Kick player
pub async fn kick_player(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let reason = body.get("reason").and_then(|v| v.as_str());

    if username.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Username required"));
    }

    // Get user info (including session_id and kick_reason) to update session when kicking
    let (upload, download, session_id) = if let Some(user_info) = state.proxy.users.get(username) {
        // Save kick reason to UserInfo
        if let Some(r) = reason {
            *user_info.kick_reason.lock().unwrap() = Some(r.to_string());
        }
        (
            user_info.upload_bytes.load(std::sync::atomic::Ordering::Relaxed),
            user_info.download_bytes.load(std::sync::atomic::Ordering::Relaxed),
            user_info.session_id,
        )
    } else {
        (0, 0, None)
    };

    // Kick player
    let kicked = state.proxy.kick_player(username);

    // If session_id exists, update session record immediately
    if kicked {
        if let Some(sid) = session_id {
            db::sessions::update_session(&state.proxy.db_pool, sid, upload, download, reason).await.ok();
        }
    }

    // Record audit log
    db::modlog::log_action(&state.proxy.db_pool, "kick", username, "webapi", reason).await.ok();

    ok_response(json!({"kicked": kicked, "username": username}))
}
