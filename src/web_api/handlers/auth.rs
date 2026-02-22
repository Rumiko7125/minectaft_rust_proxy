use crate::db;
use crate::proxy::Proxy;
use crate::web_api::auth::{AuthError, Claims, LoginResponse, TotpSetupResponse};
use crate::web_api::handlers::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use bcrypt::{hash, verify};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const JWT_SECRET_LEN: usize = 32;
const JWT_EXPIRE_HOURS: i64 = 8;
const TOTP_ISSUER: &str = "MinecraftProxy";

type ApiResult = std::result::Result<Json<Value>, (StatusCode, Json<Value>)>;

fn error_response(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"code": code, "message": message})),
    )
}

fn ok_response(data: Value) -> ApiResult {
    Ok(Json(data))
}

/// Generate JWT token
fn generate_token(username: &str, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let claims = Claims {
        sub: username.to_string(),
        exp: now + JWT_EXPIRE_HOURS * 3600,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

/// Verify JWT token
fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

/// Generate random secret
fn generate_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes)
}

/// Generate TOTP QR code Data URL
/// Simplified: return otpauth URL, frontend can use JS library to generate QR code
fn generate_totp_qr(username: &str, secret: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        TOTP_ISSUER, username, secret, TOTP_ISSUER
    )
}

/// Get or create JWT secret
fn get_jwt_secret(proxy: &Proxy) -> Vec<u8> {
    // Get jwt_secret from settings table
    // If not exists, generate one
    // Currently return fixed value, need to read from database later
    b"minecraft_rust_proxy_jwt_secret_key_2026".to_vec()
}

/// POST /api/v1/auth/login - Username/password login
pub async fn login(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Err(error_response("INVALID_CREDENTIALS", "Username and password required"));
    }

    // Get admin account from database
    let admin = match db::admin::get_admin_by_username(&state.proxy.db_pool, username).await {
        Ok(Some(a)) => a,
        Ok(None) => return Err(error_response("INVALID_CREDENTIALS", "Invalid username or password")),
        Err(_) => return Err(error_response("SERVER_ERROR", "Database error")),
    };

    // Verify password
    if !verify(password, &admin.password_hash).unwrap_or(false) {
        return Err(error_response("INVALID_CREDENTIALS", "Invalid username or password"));
    }

    // Update last login time
    let _ = db::admin::update_last_login(&state.proxy.db_pool, username).await;

    // Check TOTP binding status
    if !admin.totp_bound {
        // Need to bind TOTP, return setup_token
        let setup_token = generate_secret();
        return ok_response(json!({
            "status": "need_bind_2fa",
            "setup_token": setup_token
        }));
    }

    // Need to verify TOTP
    let session_token = generate_secret();
    ok_response(json!({
        "status": "need_totp",
        "session_token": session_token
    }))
}

/// GET /api/v1/auth/totp/setup?token=<setup_token> - Get TOTP binding QR
pub async fn totp_setup(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    // Check if admin already exists
    if let Ok(false) = db::admin::has_admins(&state.proxy.db_pool).await {
        return Err(error_response("NO_ADMIN", "No admin exists. Please run /api/v1/auth/setup first"));
    }

    let setup_token = body.get("setup_token").and_then(|v| v.as_str()).unwrap_or("");

    if setup_token.is_empty() {
        return Err(error_response("INVALID_TOKEN", "Setup token required"));
    }

    // Generate secret
    let secret = generate_secret();

    // Generate QR code
    let qr_data_url = generate_totp_qr("admin", &secret);

    ok_response(json!({
        "qr_data_url": qr_data_url,
        "secret": secret
    }))
}

/// POST /api/v1/auth/totp/confirm - Confirm TOTP binding (save secret after code is correct)
pub async fn totp_confirm(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    // Check if admin already exists
    if let Ok(false) = db::admin::has_admins(&state.proxy.db_pool).await {
        return Err(error_response("NO_ADMIN", "No admin exists. Please run /api/v1/auth/setup first"));
    }

    // Receive secret from frontend (returned by totp/setup) and user input verification code
    // Frontend sends setup_token, also compatible with secret field name
    let secret_b32 = body.get("secret")
        .or_else(|| body.get("setup_token"))
        .and_then(|v| v.as_str()).unwrap_or("");
    let totp_code = body.get("totp_code").and_then(|v| v.as_str()).unwrap_or("");
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("admin");

    if secret_b32.is_empty() || totp_code.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Secret and TOTP code required"));
    }

    // Decode secret and verify TOTP code
    let secret_bytes = match crate::auth::totp::decode_secret(secret_b32) {
        Some(b) => b,
        None => return Err(error_response("INVALID_SECRET", "Invalid secret format")),
    };
    if !crate::auth::totp::verify_totp(&secret_bytes, totp_code) {
        return Err(error_response("INVALID_TOTP", "Invalid TOTP code"));
    }

    // Save correct secret to database
    if let Err(e) = db::admin::update_totp(&state.proxy.db_pool, username, secret_b32, true).await {
        tracing::error!("Failed to save TOTP: {}", e);
        return Err(error_response("SERVER_ERROR", "Failed to save TOTP"));
    }

    // Binding successful, generate access_token
    let jwt_secret = get_jwt_secret(&state.proxy);
    let access_token = generate_token(username, &jwt_secret).unwrap_or_default();

    ok_response(json!({
        "status": "ok",
        "access_token": access_token,
        "expires_in": JWT_EXPIRE_HOURS * 3600
    }))
}

/// POST /api/v1/auth/totp/verify - Verify TOTP to get Token (login step 2)
pub async fn totp_verify(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let totp_code = body.get("totp_code").and_then(|v| v.as_str()).unwrap_or("");
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("admin");

    if totp_code.is_empty() {
        return Err(error_response("INVALID_REQUEST", "TOTP code required"));
    }

    // Get admin account from database
    let admin = match db::admin::get_admin_by_username(&state.proxy.db_pool, username).await {
        Ok(Some(a)) => a,
        Ok(None) => return Err(error_response("INVALID_USER", "Admin not found")),
        Err(_) => return Err(error_response("SERVER_ERROR", "Database error")),
    };

    if !admin.totp_bound {
        return Err(error_response("TOTP_NOT_BOUND", "2FA not bound for this admin"));
    }

    let secret_b32 = match admin.totp_secret {
        Some(s) => s,
        None => return Err(error_response("TOTP_NOT_BOUND", "2FA secret not found")),
    };

    let secret_bytes = match crate::auth::totp::decode_secret(&secret_b32) {
        Some(b) => b,
        None => return Err(error_response("TOTP_ERROR", "Invalid 2FA secret")),
    };

    if !crate::auth::totp::verify_totp(&secret_bytes, totp_code) {
        return Err(error_response("INVALID_TOTP", "Invalid TOTP code"));
    }

    // Verification successful, generate access_token
    let jwt_secret = get_jwt_secret(&state.proxy);
    let access_token = generate_token(username, &jwt_secret).unwrap_or_default();

    ok_response(json!({
        "access_token": access_token,
        "expires_in": JWT_EXPIRE_HOURS * 3600
    }))
}

/// POST /api/v1/auth/logout - Logout
pub async fn logout(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    ok_response(json!({"status": "ok"}))
}

/// Create initial admin account (first-time setup)
pub async fn setup_admin(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> ApiResult {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Err(error_response("INVALID_REQUEST", "Username and password required"));
    }

    // Check if admin already exists
    if let Ok(true) = db::admin::has_admins(&state.proxy.db_pool).await {
        return Err(error_response("ADMIN_EXISTS", "Admin already exists"));
    }

    // Hash password
    let password_hash = hash(password, 12).unwrap_or_default();

    // Create admin
    match db::admin::create_admin(&state.proxy.db_pool, username, &password_hash).await {
        Ok(_) => {
            // Return setup_token for subsequent 2FA binding
            let setup_token = generate_secret();
            ok_response(json!({
                "status": "ok",
                "message": "Admin created",
                "setup_token": setup_token
            }))
        },
        Err(e) => Err(error_response("SERVER_ERROR", &format!("Failed to create admin: {}", e))),
    }
}

/// Get authentication status (no auth required)
pub async fn get_auth_status(State(state): State<Arc<AppState>>) -> ApiResult {
    let has_admins = db::admin::has_admins(&state.proxy.db_pool).await.unwrap_or(false);
    let config = state.proxy.config.read().await;
    ok_response(json!({
        "needs_setup": !has_admins,
        "2fa_enabled": config.enable_2fa
    }))
}
