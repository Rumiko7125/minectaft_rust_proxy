use crate::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use super::AppState;

type ApiResult = std::result::Result<Json<Value>, (StatusCode, Json<Value>)>;

fn ok_response(data: serde_json::Value) -> ApiResult {
    Ok(Json(data))
}

fn err_response(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (status, Json(serde_json::json!({"message": msg})))
}

/// GET /api/v1/backend - Get all backend servers
pub async fn get_backends(State(state): State<Arc<AppState>>) -> ApiResult {
    let backends = db::backend_servers::get_all_backend_servers(&state.proxy.db_pool)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    ok_response(serde_json::json!({ "list": backends, "total": backends.len() }))
}

/// GET /api/v1/backend/:id - Get single backend server
pub async fn get_backend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult {
    let backend = db::backend_servers::get_backend_server(&state.proxy.db_pool, id)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "Backend not found"))?;
    ok_response(serde_json::to_value(backend).unwrap())
}

/// POST /api/v1/backend - Create backend server
pub async fn create_backend(
    State(state): State<Arc<AppState>>,
    Json(req): Json<db::backend_servers::CreateBackendRequest>,
) -> ApiResult {
    let id = db::backend_servers::create_backend_server(&state.proxy.db_pool, &req)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;

    ok_response(serde_json::json!({
        "id": id,
        "message": "Backend created successfully"
    }))
}

/// PATCH /api/v1/backend/:id - Update backend server
pub async fn update_backend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<db::backend_servers::UpdateBackendRequest>,
) -> ApiResult {
    db::backend_servers::update_backend_server(&state.proxy.db_pool, id, &req)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;

    ok_response(serde_json::json!({"message": "Backend updated successfully"}))
}

/// DELETE /api/v1/backend/:id - Delete backend server
pub async fn delete_backend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult {
    let routes = db::backend_servers::get_backend_routes(&state.proxy.db_pool, id)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if !routes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "message": "This backend is used by domain routes, please remove the association first",
                "routes": routes
            })),
        ));
    }

    db::backend_servers::delete_backend_server(&state.proxy.db_pool, id)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;

    ok_response(serde_json::json!({"message": "Backend deleted successfully"}))
}

/// POST /api/v1/backend/:id/enable - Enable backend server
pub async fn enable_backend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult {
    db::backend_servers::enable_backend_server(&state.proxy.db_pool, id)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;
    ok_response(serde_json::json!({"message": "Backend enabled"}))
}

/// POST /api/v1/backend/:id/disable - Disable backend server
pub async fn disable_backend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult {
    db::backend_servers::disable_backend_server(&state.proxy.db_pool, id)
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("cannot_disable_default") {
                err_response(StatusCode::BAD_REQUEST, "Cannot disable the default backend. Please set another backend as default first.")
            } else {
                err_response(StatusCode::INTERNAL_SERVER_ERROR, &err_str)
            }
        })?;

    state.proxy.reload_backends().await;
    ok_response(serde_json::json!({"message": "Backend disabled"}))
}

/// POST /api/v1/backend/:id/unset-default - Unset default backend
pub async fn unset_default_backend_handler(
    State(state): State<Arc<AppState>>,
) -> ApiResult {
    db::backend_servers::unset_default_backend(&state.proxy.db_pool)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;
    ok_response(serde_json::json!({"message": "Default backend unset"}))
}

/// POST /api/v1/backend/:id/set-default - Set default backend
pub async fn set_default_backend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult {
    let backend = db::backend_servers::get_backend_server(&state.proxy.db_pool, id)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "Backend not found"))?;

    db::backend_servers::set_default_backend(&state.proxy.db_pool, id)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;

    ok_response(serde_json::json!({
        "message": format!("Default backend set to: {}", backend.name)
    }))
}

/// GET /api/v1/backend/default - Get default backend
pub async fn get_default_backend(State(state): State<Arc<AppState>>) -> ApiResult {
    let backend = db::backend_servers::get_default_backend(&state.proxy.db_pool)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    ok_response(serde_json::json!({ "backend": backend }))
}

#[derive(Deserialize)]
pub struct MaintenanceRequest {
    pub maintenance: bool,
    pub maintenance_message: Option<String>,
}

/// POST /api/v1/backend/:id/maintenance - Toggle maintenance mode
pub async fn toggle_maintenance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<MaintenanceRequest>,
) -> ApiResult {
    db::backend_servers::toggle_maintenance(
        &state.proxy.db_pool,
        id,
        req.maintenance,
        req.maintenance_message,
    )
    .await
    .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    state.proxy.reload_backends().await;

    let msg = if req.maintenance {
        "Backend maintenance mode enabled"
    } else {
        "Backend maintenance mode disabled"
    };
    ok_response(serde_json::json!({"message": msg}))
}
