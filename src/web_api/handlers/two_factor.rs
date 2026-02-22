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

/// Generate TOTP QR code data URL
fn generate_totp_qr(username: &str, secret: &str, issuer: &str) -> String {
    let otpauth = format!("otpauth://totp/{}?secret={}&issuer={}", username, secret, issuer);
    // Return otpauth URL, frontend will generate QR code
    otpauth
}

/// GET /api/v1/2fa/players - Player 2FA list
pub async fn list_player_2fa(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult {
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Get total count
    let total: i64 = sqlx::query("SELECT COUNT(*) FROM two_factor_secrets")
        .fetch_one(&state.proxy.db_pool)
        .await
        .map(|row| row.get(0))
        .unwrap_or(0);

    let rows = sqlx::query(
        "SELECT username, created_at FROM two_factor_secrets ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.proxy.db_pool)
    .await
    .unwrap_or_default();

    let list: Vec<Value> = rows.into_iter().map(|row| {
        json!({
            "username": row.get::<String, _>(0),
            "bound": true,
            "created_at": row.get::<String, _>(1)
        })
    }).collect();

    ok_response(json!({"users": list, "total": total, "limit": limit, "offset": offset}))
}

/// DELETE /api/v1/2fa/players/:username - Unbind player 2FA
pub async fn unbind_player_2fa(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(username): axum::extract::Path<String>,
) -> ApiResult {
    // First delete 2FA secret from database
    match db::two_factor::delete_secret(&state.proxy.db_pool, &username).await {
        Ok(_) => {
            // Invalidate player's session, so they need to verify 2FA again next time
            let tf_guard = state.proxy.two_factor.read().await;
            if let Some(ref two_factor) = *tf_guard {
                two_factor.invalidate_session(&username).await;
            }
            drop(tf_guard);
            ok_response(json!({"unbound": username}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// GET /api/v1/2fa/players/:username/qr - Get player 2FA QR code
pub async fn get_player_2fa_qr(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(username): axum::extract::Path<String>,
) -> ApiResult {
    // Get player's secret
    match db::two_factor::get_secret(&state.proxy.db_pool, &username).await {
        Ok(Some(secret)) => {
            let config = state.proxy.config.read().await;
            let issuer = config.two_factor_issuer.clone();
            let qr_data_url = generate_totp_qr(&username, &secret, &issuer);
            ok_response(json!({
                "username": username,
                "secret": secret,
                "qr_data_url": qr_data_url
            }))
        }
        Ok(None) => Err(error_response("NOT_FOUND", "Player has no 2FA bound")),
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}
