use crate::db;
use crate::proxy::Proxy;
use crate::proxy::router::DomainRoute;
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

/// GET /api/v1/routes/domain - Get domain routes
pub async fn get_domain_routes(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let routes = state.proxy.domain_routes.read().await;
    let total = routes.len() as i64;

    let list: Vec<Value> = routes
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|route| {
            json!({
                "id": route.id,
                "pattern": route.pattern_str,
                "target_addr": route.target_addr,
                "target_port": route.target_port,
                "priority": route.priority,
                "backend_id": route.backend_id
            })
        })
        .collect();
    ok_response(json!({"routes": list, "total": total, "limit": limit, "offset": offset}))
}

/// POST /api/v1/routes/domain - Add domain route
pub async fn add_domain_route(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let pattern = body.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
    let backend_id = body.get("backend_id").and_then(|v| v.as_i64());
    let target_addr = body.get("target_addr").and_then(|v| v.as_str()).unwrap_or("");
    let target_port = body.get("target_port").and_then(|v| v.as_u64()).unwrap_or(25565) as u16;
    let priority = body.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    if pattern.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Pattern is required"));
    }
    if backend_id.is_none() && target_addr.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Either backend_id or target_addr is required"));
    }

    // Validate regex
    if DomainRoute::new(pattern, target_addr, target_port, backend_id).is_err() {
        return Err(error_response("INVALID_PATTERN", "Invalid regex pattern"));
    }

    // Save to database
    match db::domain_routes::add_domain_route(&state.proxy.db_pool, pattern, target_addr, target_port, priority, backend_id).await {
        Ok(id) => {
            // Reload in-memory data
            if let Ok(routes) = db::domain_routes::get_all_domain_routes(&state.proxy.db_pool).await {
                let routes: Vec<DomainRoute> = routes.iter()
                    .filter_map(|r| DomainRoute::from_db(r).ok())
                    .collect();
                *state.proxy.domain_routes.write().await = routes;
            }
            ok_response(json!({"id": id, "pattern": pattern, "target_addr": target_addr, "target_port": target_port, "priority": priority, "backend_id": backend_id}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// PATCH /api/v1/routes/domain/:id - Update domain route
pub async fn update_domain_route(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(body): Json<Value>,
) -> ApiResult {
    // Get current route first
    let existing_routes = db::domain_routes::get_all_domain_routes(&state.proxy.db_pool).await
        .map_err(|e| error_response("DB_ERROR", &e.to_string()))?;
    let existing = existing_routes.iter().find(|r| r.id == id)
        .ok_or_else(|| error_response("NOT_FOUND", "Route not found"))?;

    let pattern = body.get("pattern").and_then(|v| v.as_str()).unwrap_or(&existing.pattern);
    let backend_id = if body.get("backend_id").is_some() {
        body.get("backend_id").and_then(|v| v.as_i64())
    } else {
        existing.backend_id
    };
    let target_addr = body.get("target_addr").and_then(|v| v.as_str()).unwrap_or(&existing.target_addr);
    let target_port = body.get("target_port").and_then(|v| v.as_u64()).map(|v| v as u16).unwrap_or(existing.target_port);
    let priority = body.get("priority").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(existing.priority);

    if DomainRoute::new(pattern, target_addr, target_port, backend_id).is_err() {
        return Err(error_response("INVALID_PATTERN", "Invalid regex pattern"));
    }

    if let Err(e) = db::domain_routes::update_domain_route(&state.proxy.db_pool, id, pattern, target_addr, target_port, priority, backend_id).await {
        return Err(error_response("DB_ERROR", &e.to_string()));
    }

    // Reload in-memory data
    if let Ok(routes) = db::domain_routes::get_all_domain_routes(&state.proxy.db_pool).await {
        let routes: Vec<DomainRoute> = routes.iter()
            .filter_map(|r| DomainRoute::from_db(r).ok())
            .collect();
        *state.proxy.domain_routes.write().await = routes;
    }

    ok_response(json!({"updated": id, "pattern": pattern, "backend_id": backend_id}))
}

/// DELETE /api/v1/routes/domain/:id - Delete domain route
pub async fn delete_domain_route(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> ApiResult {
    // Delete from database
    if let Err(e) = db::domain_routes::delete_domain_route(&state.proxy.db_pool, id).await {
        return Err(error_response("DB_ERROR", &e.to_string()));
    }

    // Reload in-memory data
    if let Ok(routes) = db::domain_routes::get_all_domain_routes(&state.proxy.db_pool).await {
        let routes: Vec<DomainRoute> = routes.iter()
            .filter_map(|r| DomainRoute::from_db(r).ok())
            .collect();
        *state.proxy.domain_routes.write().await = routes;
    }

    ok_response(json!({"removed": id}))
}
