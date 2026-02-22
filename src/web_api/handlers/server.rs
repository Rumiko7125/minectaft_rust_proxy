use crate::db;
use crate::web_api::auth::Claims;
use crate::web_api::handlers::AppState;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::atomic::Ordering;

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

/// POST /api/v1/server/restart - Restart proxy service (requires TOTP verification)
///
/// Restart flow (graceful shutdown):
///   1. After TOTP verification, set proxy.restart_pending = true
///   2. Send shutdown_tx signal -- proxy listener (:25565) and web API (:20220)
///      both receive signal and start graceful shutdown:
///        - proxy: start() select! exits, TcpListener drops, port immediately released
///        - web API: with_graceful_shutdown drops listener after completing current request
///   3. This handler returns HTTP 200 (axum waits for response to be written before closing)
///   4. main() detects restart_pending after proxy.start() returns,
///      waits for web API task to end, spawns new process and exit(0)
///   New process starts with both ports released, no wait needed to bind.
pub async fn restart_server(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> ApiResult {
    let username = claims.sub.as_str();
    let totp_code = body.get("totp_code").and_then(|v| v.as_str()).unwrap_or("");

    if totp_code.is_empty() {
        return Err(error_response("INVALID_REQUEST", "TOTP code required"));
    }
    if totp_code.len() != 6 || !totp_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(error_response("INVALID_TOTP", "TOTP code must be 6 digits"));
    }

    let admin = match db::admin::get_admin_by_username(&state.proxy.db_pool, username).await {
        Ok(Some(a)) => a,
        Ok(None) => return Err(error_response("INVALID_USER", "Admin not found")),
        Err(_) => return Err(error_response("SERVER_ERROR", "Database error")),
    };

    if !admin.totp_bound {
        return Err(error_response("TOTP_NOT_BOUND", "2FA not bound for this user"));
    }

    let totp_secret_str = match admin.totp_secret {
        Some(s) => s,
        None => return Err(error_response("TOTP_NOT_BOUND", "2FA secret not found")),
    };

    let secret_bytes = match crate::auth::totp::decode_secret(&totp_secret_str) {
        Some(b) => b,
        None => return Err(error_response("TOTP_ERROR", "Invalid 2FA secret")),
    };

    if !crate::auth::totp::verify_totp(&secret_bytes, totp_code) {
        return Err(error_response("INVALID_TOTP", "Invalid TOTP code"));
    }

    tracing::info!("Restart requested by admin: {}", username);

    // Set restart flag, then send shutdown signal
    // proxy.start() will break and drop TcpListener (port release)
    // web API will shutdown with_graceful_shutdown
    // main() detects restart_pending and spawns new process
    state.proxy.restart_pending.store(true, Ordering::Relaxed);
    let _ = state.proxy.shutdown_tx.send(());

    ok_response(json!({
        "status": "ok",
        "message": "Server restart initiated"
    }))
}
