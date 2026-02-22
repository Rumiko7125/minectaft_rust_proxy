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

fn ok_response(data: Value) -> ApiResult {
    Ok(Json(data))
}

fn error_response(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"code": code, "message": message})),
    )
}

/// Convert Minecraft protocol version to human-readable version
fn protocol_to_version(p: i32) -> &'static str {
    match p {
        // 1.21.x
        774.. => "1.21.11+",
        773 => "1.21.9-1.21.10",
        772 => "1.21.7-1.21.8",
        771 => "1.21.6",
        770 => "1.21.5",
        769 => "1.21.4",
        768 => "1.21.2-1.21.3",
        767 => "1.21-1.21.1",

        // 1.20.x
        766 => "1.20.5-1.20.6",
        765 => "1.20.3-1.20.4",
        764 => "1.20.2",
        763 => "1.20-1.20.1",

        // 1.19.x
        762 => "1.19.4",
        761 => "1.19.3",
        760 => "1.19.1-1.19.2",
        759 => "1.19",

        // 1.18.x
        758 => "1.18.2",
        757 => "1.18-1.18.1",

        // 1.17.x
        756 => "1.17.1",
        755 => "1.17",

        // 1.16.x
        754 => "1.16.5",
        753 => "1.16.3-1.16.4",
        751 => "1.16.2",
        736 => "1.16.1",
        735 => "1.16",

        // 1.15.x
        578 => "1.15.2",
        575 => "1.15.1",
        573 => "1.15",

        // 1.14.x
        498 => "1.14.4",
        490 => "1.14.3",
        485 => "1.14.2",
        480 => "1.14.1",
        477 => "1.14",

        // 1.13.x
        404 => "1.13.2",
        401 => "1.13.1",
        393 => "1.13",

        // 1.12.x
        340 => "1.12.2",
        338 => "1.12.1",
        335 => "1.12",

        // 1.11.x
        316 => "1.11.2",
        315 => "1.11-1.11.1",

        // 1.10.x
        210 => "1.10.x",

        // 1.9.x
        110 => "1.9.4",
        109 => "1.9.2-1.9.3",
        107 => "1.9-1.9.1",

        // 1.8.x
        47 => "1.8-1.8.9",

        // 1.7.x
        5 => "1.7.10",
        4 => "1.7.2-1.7.9",

        _ => "unknown",
    }
}

/// GET /api/v1/players - Online player list
pub async fn get_players(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let all_users = state.proxy.get_online_users();
    let total = all_users.len() as i64;

    let players: Vec<Value> = all_users
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|(name, uuid, backend_name, login_time, up, down, protocol)| {
            json!({
                "username": name,
                "uuid": uuid,
                "backend_name": backend_name,
                "login_time": login_time,
                "upload_bytes": up,
                "download_bytes": down,
                "version": protocol_to_version(protocol)
            })
        })
        .collect();
    ok_response(json!({"players": players, "total": total, "limit": limit, "offset": offset}))
}

/// GET /api/v1/players/history - Historical sessions
pub async fn get_players_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let search = params.get("search").map(|s| s.as_str()).unwrap_or("");
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Build query conditions
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
        "SELECT id, username, uuid, backend_addr, backend_port, login_at, logout_at, upload_bytes, download_bytes, protocol_version FROM player_sessions {} ORDER BY login_at DESC LIMIT ? OFFSET ?",
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
                let protocol: i32 = row.get(9);
                json!({
                    "id": row.get::<i64, _>(0),
                    "username": row.get::<String, _>(1),
                    "uuid": row.get::<String, _>(2),
                    "backend_addr": row.get::<String, _>(3),
                    "backend_port": row.get::<i32, _>(4),
                    "login_at": row.get::<String, _>(5),
                    "logout_at": row.get::<Option<String>, _>(6),
                    "upload_bytes": row.get::<i64, _>(7),
                    "download_bytes": row.get::<i64, _>(8),
                    "version": protocol_to_version(protocol)
                })
            }).collect();
            ok_response(json!({"sessions": sessions, "total": total, "limit": limit, "offset": offset}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// GET /api/v1/players/stats - Statistics for charts
pub async fn get_players_stats(State(state): State<Arc<AppState>>) -> ApiResult {
    // Get today's ended sessions traffic (using UTC date('now') to match DB storage)
    let today_stats: Vec<(i64, i64)> = sqlx::query(
        "SELECT COALESCE(SUM(upload_bytes), 0), COALESCE(SUM(download_bytes), 0) FROM player_sessions WHERE date(login_at) = date('now') AND logout_at IS NOT NULL"
    )
    .fetch_all(&state.proxy.db_pool)
    .await
    .ok()
    .map(|rows| {
        rows.iter().map(|row| {
            (row.get::<i64, _>(0), row.get::<i64, _>(1))
        }).collect()
    })
    .unwrap_or_default();

    let (mut today_upload, mut today_download) = today_stats.first().copied().unwrap_or((0, 0));

    // Add current online players' real-time traffic
    let online_users = state.proxy.get_online_users();
    let mut backend_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for (_, _, backend_name, _, up, down, _) in &online_users {
        today_upload += *up as i64;
        today_download += *down as i64;
        *backend_counts.entry(backend_name.clone()).or_insert(0) += 1;
    }

    // Get online count stats for past 24 hours (one entry per hour)
    let hourly_stats = sqlx::query(
        "SELECT CAST(strftime('%H', login_at) AS INTEGER) as hour, COUNT(DISTINCT username) FROM player_sessions WHERE login_at >= datetime('now', '-24 hours') GROUP BY hour"
    )
    .fetch_all(&state.proxy.db_pool)
    .await
    .ok()
    .map(|rows| {
        rows.iter().map(|row| {
            json!({
                "hour": row.get::<i64, _>(0),
                "count": row.get::<i64, _>(1)
            })
        }).collect::<Vec<Value>>()
    })
    .unwrap_or_default();

    // Get traffic stats for past 24 hours (one entry per hour)
    let hourly_traffic = sqlx::query(
        "SELECT CAST(strftime('%H', login_at) AS INTEGER) as hour, COALESCE(SUM(upload_bytes), 0), COALESCE(SUM(download_bytes), 0) FROM player_sessions WHERE login_at >= datetime('now', '-24 hours') GROUP BY hour"
    )
    .fetch_all(&state.proxy.db_pool)
    .await
    .ok()
    .map(|rows| {
        rows.iter().map(|row| {
            json!({
                "hour": row.get::<i64, _>(0),
                "upload": row.get::<i64, _>(1),
                "download": row.get::<i64, _>(2)
            })
        }).collect::<Vec<Value>>()
    })
    .unwrap_or_default();

    ok_response(json!({
        "online_count": state.proxy.users.len(),
        "max_players": -1,
        "today_upload": today_upload,
        "today_download": today_download,
        "hourly_stats": hourly_stats,
        "hourly_traffic": hourly_traffic,
        "backend_counts": backend_counts
    }))
}

/// GET /api/v1/dashboard/recent - Get recent activity records for Dashboard
pub async fn get_dashboard_recent(State(state): State<Arc<AppState>>) -> ApiResult {
    // Get recent 10 moderation_logs
    let recent_mod_logs = sqlx::query(
        "SELECT id, action, target, operator, reason, created_at FROM moderation_logs ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.proxy.db_pool)
    .await
    .ok()
    .map(|rows| {
        rows.iter().map(|row| {
            json!({
                "id": row.get::<i64, _>(0),
                "action": row.get::<String, _>(1),
                "target": row.get::<String, _>(2),
                "operator": row.get::<Option<String>, _>(3),
                "reason": row.get::<Option<String>, _>(4),
                "timestamp": row.get::<String, _>(5)
            })
        }).collect::<Vec<Value>>()
    })
    .unwrap_or_default();

    // Get recent 10 ended session records
    let recent_sessions = sqlx::query(
        "SELECT id, username, uuid, backend_addr, login_at, logout_at, upload_bytes, download_bytes, protocol_version FROM player_sessions WHERE logout_at IS NOT NULL ORDER BY login_at DESC LIMIT 10"
    )
    .fetch_all(&state.proxy.db_pool)
    .await
    .ok()
    .map(|rows| {
        rows.iter().map(|row| {
            let protocol: i32 = row.get(8);
            json!({
                "id": row.get::<i64, _>(0),
                "username": row.get::<String, _>(1),
                "uuid": row.get::<String, _>(2),
                "backend_addr": row.get::<String, _>(3),
                "login_at": row.get::<String, _>(4),
                "logout_at": row.get::<Option<String>, _>(5),
                "upload_bytes": row.get::<i64, _>(6),
                "download_bytes": row.get::<i64, _>(7),
                "version": protocol_to_version(protocol)
            })
        }).collect::<Vec<Value>>()
    })
    .unwrap_or_default();

    // Get current online players (real-time traffic)
    let online_users = state.proxy.get_online_users();
    let online_sessions: Vec<Value> = online_users
        .iter()
        .map(|(name, uuid, backend_name, login_time, up, down, protocol)| {
            // Convert Unix timestamp to readable time
            let login_at = chrono::DateTime::from_timestamp(*login_time as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            json!({
                "id": 0,
                "username": name,
                "uuid": uuid,
                "backend_addr": backend_name,
                "login_at": login_at,
                "logout_at": None::<String>,
                "upload_bytes": up,
                "download_bytes": down,
                "version": protocol_to_version(*protocol),
                "online": true
            })
        })
        .collect();

    ok_response(json!({
        "recent_mod_logs": recent_mod_logs,
        "recent_sessions": recent_sessions,
        "online_sessions": online_sessions,
        "server_start_time": state.proxy.start_time
    }))
}
