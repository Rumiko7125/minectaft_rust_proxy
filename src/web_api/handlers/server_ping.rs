use crate::web_api::handlers::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

type ApiResult = std::result::Result<Json<Value>, (StatusCode, Json<Value>)>;

#[derive(Debug, Deserialize)]
pub struct PingQuery {
    pub addr: String,
    pub port: Option<u16>,
}

/// GET /api/v1/ping?addr=mc.hypixel.net&port=25565
pub async fn ping_server_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<PingQuery>,
) -> ApiResult {
    let port = query.port.unwrap_or(25565);

    match crate::protocol::ping::ping_server(&query.addr, port).await {
        Ok(result) => Ok(Json(json!({
            "description": result.description,
            "online": result.online,
            "max": result.max,
            "version_name": result.version_name,
            "favicon": result.favicon,
            "latency_ms": result.latency_ms,
        }))),
        Err(e) => Ok(Json(json!({
            "error": "unreachable",
            "detail": e.to_string(),
        }))),
    }
}
