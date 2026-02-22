use crate::web_api::handlers::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

type ApiResult = std::result::Result<Json<Value>, (StatusCode, Json<Value>)>;

/// POST /api/v1/cli - Execute CLI command (only proxy built-in commands)
pub async fn execute_cli(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> ApiResult {
    let command = match body.get("command").and_then(|v| v.as_str()) {
        Some(c) if !c.trim().is_empty() => c.trim().to_string(),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": "INVALID_REQUEST", "message": "Command required"})),
            ));
        }
    };

    let output = crate::cli::execute_command(&state.proxy, &command).await;

    Ok(Json(json!({
        "output": output,
        "command": command,
    })))
}
