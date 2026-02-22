use serde::{Deserialize, Serialize};

pub mod middleware;

pub use middleware::auth_middleware;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // username
    pub exp: i64,
    pub iat: i64,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub status: String,           // "need_bind_2fa" | "need_totp" | "ok"
    pub setup_token: Option<String>,
    pub session_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
}

/// TOTP setup information
#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
    pub qr_data_url: String,
    pub secret: String,
}

/// Token verification request
#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub session_token: Option<String>,
    pub setup_token: Option<String>,
    pub totp_code: String,
}

/// Authentication error response
#[derive(Debug, Serialize)]
pub struct AuthError {
    pub code: String,
    pub message: String,
}

impl AuthError {
    pub fn invalid_credentials() -> Self {
        Self {
            code: "INVALID_CREDENTIALS".to_string(),
            message: "Invalid username or password".to_string(),
        }
    }

    pub fn invalid_totp() -> Self {
        Self {
            code: "INVALID_TOTP".to_string(),
            message: "Invalid TOTP code".to_string(),
        }
    }

    pub fn account_locked(minutes: i32) -> Self {
        Self {
            code: "ACCOUNT_LOCKED".to_string(),
            message: format!("Account locked for {} minutes", minutes),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
        }
    }
}
