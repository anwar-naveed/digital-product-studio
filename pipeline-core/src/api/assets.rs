//! Authenticated asset serving.
//!
//! The admin UI renders generated variants with plain <img> tags, which
//! cannot send Authorization headers. This route accepts the SAME JWT via
//! `?token=` and streams object bytes from S3/MinIO. Keys are validated to
//! stay within the designs/ namespace.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    middleware::auth::Claims,
    state::AppState,
};

#[derive(Deserialize)]
pub struct AssetQuery {
    token: Option<String>,
}

fn content_type_for(key: &str) -> &'static str {
    match key.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": {
                "code": "UNAUTHORIZED",
                "message": "Missing or invalid token",
                "correlation_id": Uuid::new_v4().to_string(),
            }
        })),
    )
        .into_response()
}

/// GET /api/v1/assets/{*key}?token=<jwt>
///
/// Streams a pipeline asset (generated variant, bundle, ...) from object
/// storage to the authenticated operator.
pub async fn get_asset(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<AssetQuery>,
) -> Result<Response> {
    // Validate the JWT (query param carries it for <img> src usage)
    let token = query.token.ok_or_else(|| AppError::Unauthorized("missing token".into()))?;

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "development-secret-change-in-production".to_string());
    let claims = jsonwebtoken::decode::<Claims>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("invalid token".into()))?;

    if claims.claims.role != "admin" {
        return Err(AppError::Forbidden("admin role required".into()));
    }

    // Jail the key: no traversal, and only the pipeline's own key shapes —
    // "products/<uuid>/bundle.zip" (assembled product bundles),
    // "curated/<...>" (approved variants), "<design-uuid>/<file>" (raw
    // variant outputs) or "designs/<...>".
    let key = key.trim_start_matches('/');
    let segs: Vec<&str> = key.split('/').collect();
    let valid_shape = key.starts_with("designs/")
        || key.starts_with("curated/")
        || (segs.len() == 3
            && segs[0] == "products"
            && segs[1].len() == 36
            && segs[2] == "bundle.zip")
        || (segs.len() == 2
            && segs[0].len() == 36
            && !key.contains(".."));
    if !valid_shape {
        return Err(AppError::Forbidden("key outside pipeline namespace".into()));
    }

    let result = state
        .s3_client
        .get(key)
        .await
        .map_err(|e| AppError::NotFound(format!("asset not found: {}", e)))?;

    let bytes = result
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("asset read failed: {}", e)))?;

    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, content_type_for(key).parse().unwrap());
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        "private, max-age=3600".parse().unwrap(),
    );
    Ok(resp)
}
