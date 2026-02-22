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

/// GET /api/v1/logs/moderation - Moderation logs
pub async fn get_moderation_logs(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let action = params.get("action").map(|s| s.as_str());
    let target = params.get("target").map(|s| s.as_str());
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Build query conditions
    let mut conditions: Vec<&str> = vec![];
    let mut bindings: Vec<String> = vec![];

    if !target.unwrap_or("").is_empty() {
        conditions.push("target LIKE ?");
        bindings.push(format!("%{}%", target.unwrap()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Query data
    let query_str = format!(
        "SELECT id, action, target, operator, reason, created_at FROM moderation_logs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut query = sqlx::query(&query_str);
    for binding in &bindings {
        query = query.bind(binding);
    }
    query = query.bind(limit as i64).bind(offset as i64);

    // Query total count
    let count_str = format!(
        "SELECT COUNT(*) FROM moderation_logs {}",
        where_clause
    );
    let mut count_query = sqlx::query(&count_str);
    for binding in &bindings {
        count_query = count_query.bind(binding);
    }

    let total: i64 = match count_query.fetch_one(&state.proxy.db_pool).await {
        Ok(row) => row.get(0),
        Err(_) => 0
    };

    let rows = match query.fetch_all(&state.proxy.db_pool).await {
        Ok(rows) => rows,
        Err(e) => return Err(error_response("DB_ERROR", &e.to_string())),
    };

    // Filter by action
    let filtered: Vec<Value> = rows
        .into_iter()
        .filter(|row| {
            let log_action: String = row.get(1);
            if action.is_some() && action != Some(&log_action) {
                return false;
            }
            true
        })
        .map(|row| {
            json!({
                "id": row.get::<i64, _>(0),
                "action": row.get::<String, _>(1),
                "target": row.get::<String, _>(2),
                "operator": row.get::<String, _>(3),
                "reason": row.get::<Option<String>, _>(4),
                "timestamp": row.get::<String, _>(5)
            })
        })
        .collect();

    ok_response(json!({"logs": filtered, "total": total, "limit": limit, "offset": offset}))
}

/// GET /api/v1/logs/sessions - Session logs
pub async fn get_session_logs(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let search = params.get("search").map(|s| s.as_str()).unwrap_or("");
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Build query conditions - supports fuzzy search by username and backend server (case-insensitive)
    let mut conditions: Vec<&str> = vec![];
    let mut bindings: Vec<String> = vec![];

    if !search.is_empty() {
        conditions.push("(username LIKE ? OR backend_addr LIKE ?)");
        let search_pattern = format!("%{}%", search);
        bindings.push(search_pattern.clone());
        bindings.push(search_pattern);
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Query data
    let query_str = format!(
        "SELECT id, username, uuid, backend_addr, backend_port, login_at, logout_at, upload_bytes, download_bytes FROM player_sessions {} ORDER BY login_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut query = sqlx::query(&query_str);
    for binding in &bindings {
        query = query.bind(binding);
    }
    query = query.bind(limit as i64).bind(offset as i64);

    // Query total count
    let count_str = format!(
        "SELECT COUNT(*) FROM player_sessions {}",
        where_clause
    );
    let mut count_query = sqlx::query(&count_str);
    for binding in &bindings {
        count_query = count_query.bind(binding);
    }

    let total: i64 = match count_query.fetch_one(&state.proxy.db_pool).await {
        Ok(row) => row.get(0),
        Err(_) => 0
    };

    match query.fetch_all(&state.proxy.db_pool).await {
        Ok(rows) => {
            let sessions: Vec<Value> = rows.iter().map(|row| {
                json!({
                    "id": row.get::<i64, _>(0),
                    "username": row.get::<String, _>(1),
                    "uuid": row.get::<String, _>(2),
                    "backend_addr": row.get::<String, _>(3),
                    "backend_port": row.get::<i32, _>(4),
                    "login_at": row.get::<String, _>(5),
                    "logout_at": row.get::<Option<String>, _>(6),
                    "upload_bytes": row.get::<i64, _>(7),
                    "download_bytes": row.get::<i64, _>(8)
                })
            }).collect();

            ok_response(json!({"sessions": sessions, "total": total, "limit": limit, "offset": offset}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// GET /api/v1/logs/moderation/export - Export moderation logs CSV
pub async fn export_moderation(
    State(state): State<Arc<AppState>>,
) -> ApiResult {
    match sqlx::query(
        "SELECT timestamp, action, target, operator, reason FROM moderation_logs ORDER BY timestamp DESC LIMIT 1000"
    )
    .fetch_all(&state.proxy.db_pool)
    .await
    {
        Ok(rows) => {
            let mut csv = String::from("timestamp,action,target,operator,reason\n");
            for row in rows {
                csv.push_str(&format!(
                    "{},{},{},{},{}\n",
                    row.get::<String, _>(0),
                    row.get::<String, _>(1),
                    row.get::<String, _>(2),
                    row.get::<Option<String>, _>(3).unwrap_or_default(),
                    row.get::<Option<String>, _>(4).unwrap_or_default()
                ));
            }
            ok_response(json!({"csv": csv}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// GET /api/v1/logs/sessions/export - Export sessions CSV
pub async fn export_sessions(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let username = params.get("username").map(|s| s.as_str());

    let query = if let Some(user) = username {
        sqlx::query(
            "SELECT username, uuid, backend_addr, backend_port, login_at, logout_at, upload_bytes, download_bytes FROM player_sessions WHERE username = ? ORDER BY login_at DESC"
        )
        .bind(user)
    } else {
        sqlx::query(
            "SELECT username, uuid, backend_addr, backend_port, login_at, logout_at, upload_bytes, download_bytes FROM player_sessions ORDER BY login_at DESC LIMIT 1000"
        )
    };

    match query.fetch_all(&state.proxy.db_pool).await {
        Ok(rows) => {
            let mut csv = String::from("username,uuid,backend_addr,backend_port,login_at,logout_at,upload_bytes,download_bytes\n");
            for row in rows {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{}\n",
                    row.get::<String, _>(0),
                    row.get::<String, _>(1),
                    row.get::<String, _>(2),
                    row.get::<i32, _>(3),
                    row.get::<String, _>(4),
                    row.get::<Option<String>, _>(5).unwrap_or_default(),
                    row.get::<i64, _>(6),
                    row.get::<i64, _>(7)
                ));
            }
            ok_response(json!({"csv": csv}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}
