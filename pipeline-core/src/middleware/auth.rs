//! Authentication and authorization middleware

use axum::{
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    state::AppState,
};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id as UUID string
    pub role: String, // admin, operator, etc.
    pub exp: usize,   // expiration timestamp
    pub iat: usize,   // issued at timestamp
}

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

/// Extractor for requiring admin role
pub struct RequireAdmin(pub AuthUser);

impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Missing Authorization header",
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Invalid Authorization header format, expected Bearer token",
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        // Get JWT config from state (we need AppState)
        // For now, we'll use a simpler approach - validate with environment secret
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development-secret-change-in-production".to_string());

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": format!("Invalid token: {}", e),
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Invalid user ID in token",
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        // Check role is admin
        if token_data.claims.role != "admin" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": {
                        "code": "FORBIDDEN",
                        "message": "Admin role required",
                        "correlation_id": Uuid::new_v4().to_string(),
                    }
                })),
            )
                .into_response());
        }

        Ok(RequireAdmin(AuthUser {
            user_id,
            role: token_data.claims.role,
        }))
    }
}

/// Extractor for optional auth (allows unauthenticated requests)
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> std::result::Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        if let Some(auth_header) = auth_header {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                let jwt_secret = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "development-secret-change-in-production".to_string());

                let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
                let validation = Validation::new(Algorithm::HS256);

                if let Ok(token_data) = decode::<Claims>(token, &decoding_key, &validation) {
                    if let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) {
                        return Ok(OptionalAuth(Some(AuthUser {
                            user_id,
                            role: token_data.claims.role,
                        })));
                    }
                }
            }
        }

        Ok(OptionalAuth(None))
    }
}

/// Extractor for requiring operator role (or admin)
pub struct RequireOperator(pub AuthUser);

impl<S> FromRequestParts<S> for RequireOperator
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> std::result::Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Missing Authorization header",
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Invalid Authorization header format, expected Bearer token",
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development-secret-change-in-production".to_string());

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": format!("Invalid token: {}", e),
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": "Invalid user ID in token",
                            "correlation_id": Uuid::new_v4().to_string(),
                        }
                    })),
                )
                    .into_response()
            })?;

        // operator or admin may pass
        if token_data.claims.role != "operator" && token_data.claims.role != "admin" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": {
                        "code": "FORBIDDEN",
                        "message": "Operator role required",
                        "correlation_id": Uuid::new_v4().to_string(),
                    }
                })),
            )
                .into_response());
        }

        Ok(RequireOperator(AuthUser {
            user_id,
            role: token_data.claims.role,
        }))
    }
}
