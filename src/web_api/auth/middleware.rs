use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::json;

use super::Claims;

/// Authentication middleware - verify JWT token
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    // Get Authorization header
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..]; // Remove "Bearer " prefix

            // Validate token
            match validate_token(token) {
                Ok(claims) => {
                    // Store claims in request extension for handler
                    req.extensions_mut().insert(claims);
                    next.run(req).await
                }
                Err(_) => auth_error()
            }
        }
        _ => {
            // No token or invalid format
            auth_error()
        }
    }
}

/// JWT validation secret (must match token generation)
const JWT_SECRET: &[u8] = b"minecraft_rust_proxy_jwt_secret_key_2026";

/// Validate JWT token
fn validate_token(token: &str) -> Result<Claims, ()> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear(); // No specific claims required

    match decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET), &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err(()),
    }
}

/// Authentication error response
fn auth_error() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&json!({
                "code": "UNAUTHORIZED",
                "message": "Invalid or missing token"
            })).unwrap()
        ))
        .unwrap()
}
