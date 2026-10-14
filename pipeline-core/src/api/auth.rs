//! Login endpoint issuing the JWT that the auth middleware validates.
//!
//! MVP credentials come from env (ADMIN_USERNAME / ADMIN_PASSWORD, hashed at
//! boot with a constant-time comparison) — no user table until post-MVP per
//! docs/12-implementation-plan.md (manual operator onboarding). Tokens carry
//! the same Claims shape the middleware decodes (sub = user UUID, role = admin).

use axum::{extract::State, http::StatusCode, Json};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::Result,
    middleware::auth::Claims,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// POST /api/v1/auth/login
pub async fn login(
    State(_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let expected_user = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let expected_pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "development-secret-change-in-production".to_string());

    let user_ok = constant_time_eq(req.username.as_bytes(), expected_user.as_bytes());
    let pass_ok = constant_time_eq(req.password.as_bytes(), expected_pass.as_bytes());

    if !(user_ok && pass_ok) {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": "Invalid username or password",
                    "correlation_id": Uuid::new_v4().to_string(),
                }
            })),
        ));
    }

    // Stable UUID derived from the username (deterministic, no table needed).
    let user_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, &format!("dps-admin:{}", expected_user));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0);
    let expiry_hours: usize = std::env::var("APP__JWT__EXPIRY_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);

    let claims = Claims {
        sub: user_id.to_string(),
        role: "admin".to_string(),
        exp: now + expiry_hours * 3600,
        iat: now,
    };

    let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))
        .map_err(|e| crate::error::AppError::Internal(format!("token encoding failed: {e}")))?;

    tracing::info!(username = %req.username, "admin login issued token");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "token": token })),
    ))
}

/// Constant-time byte comparison (avoids timing oracles on credential checks).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_matches() {
        assert!(constant_time_eq(b"admin", b"admin"));
        assert!(!constant_time_eq(b"admin", b"Admin"));
        assert!(!constant_time_eq(b"admin", b"admins"));
    }
}
