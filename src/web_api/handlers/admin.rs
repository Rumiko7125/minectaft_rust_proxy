use crate::db;
use crate::proxy::Proxy;
use crate::web_api::auth::Claims;
use crate::web_api::handlers::AppState;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::Json;
use bcrypt::{hash, verify};
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

/// GET /api/v1/admin/accounts - Admin list
pub async fn list_admins(State(state): State<Arc<AppState>>) -> ApiResult {
    match db::admin::get_all_admins(&state.proxy.db_pool).await {
        Ok(admins) => {
            let list: Vec<Value> = admins
                .into_iter()
                .map(|a| {
                    json!({
                        "id": a.id,
                        "username": a.username,
                        "totp_bound": a.totp_bound,
                        "preferred_locale": a.preferred_locale,
                        "created_at": a.created_at,
                        "last_login_at": a.last_login_at
                    })
                })
                .collect();
            ok_response(json!({"admins": list}))
        }
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// POST /api/v1/admin/accounts - Create admin
pub async fn create_admin(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Username and password required"));
    }

    let password_hash = match hash(password, 12) {
        Ok(h) => h,
        Err(_) => return Err(error_response("HASH_ERROR", "Failed to hash password")),
    };

    match db::admin::create_admin(&state.proxy.db_pool, username, &password_hash).await {
        Ok(_) => ok_response(json!({"created": username})),
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// DELETE /api/v1/admin/accounts/:id - Delete admin
pub async fn delete_admin(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> ApiResult {
    match db::admin::delete_admin(&state.proxy.db_pool, id).await {
        Ok(_) => ok_response(json!({"deleted": id})),
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// POST /api/v1/admin/accounts/:id/reset-2fa - Reset admin 2FA
pub async fn reset_admin_2fa(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> ApiResult {
    // Get username first
    let username = match db::admin::get_admin_by_id(&state.proxy.db_pool, id).await {
        Ok(Some(admin)) => admin.username,
        Ok(None) => return Err(error_response("NOT_FOUND", "Admin not found")),
        Err(e) => return Err(error_response("DB_ERROR", &e.to_string())),
    };

    match db::admin::clear_totp(&state.proxy.db_pool, &username).await {
        Ok(_) => ok_response(json!({"reset": username})),
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// POST /api/v1/admin/me/password - Change current admin password
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> ApiResult {
    let username = claims.sub.as_str();
    let old_password = body.get("old_password").and_then(|v| v.as_str()).unwrap_or("");
    let new_password = body.get("new_password").and_then(|v| v.as_str()).unwrap_or("");

    if old_password.is_empty() || new_password.is_empty() {
        return Err(error_response("INVALID_REQUEST", "All fields required"));
    }

    // Verify old password
    let admin = match db::admin::get_admin_by_username(&state.proxy.db_pool, username).await {
        Ok(Some(a)) => a,
        Ok(None) => return Err(error_response("NOT_FOUND", "Admin not found")),
        Err(e) => return Err(error_response("DB_ERROR", &e.to_string())),
    };

    if !verify(old_password, &admin.password_hash).unwrap_or(false) {
        return Err(error_response("INVALID_PASSWORD", "Old password incorrect"));
    }

    let new_hash = match hash(new_password, 12) {
        Ok(h) => h,
        Err(_) => return Err(error_response("HASH_ERROR", "Failed to hash password")),
    };

    match db::admin::update_password(&state.proxy.db_pool, username, &new_hash).await {
        Ok(_) => ok_response(json!({"changed": username})),
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}

/// PATCH /api/v1/admin/me/locale - Update locale preference
pub async fn update_locale(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> ApiResult {
    let username = claims.sub.as_str();
    let locale = body.get("locale").and_then(|v| v.as_str()).unwrap_or("zh-CN");

    match db::admin::update_locale(&state.proxy.db_pool, username, locale).await {
        Ok(_) => ok_response(json!({"locale": locale})),
        Err(e) => Err(error_response("DB_ERROR", &e.to_string())),
    }
}
