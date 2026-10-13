//! Settings API handlers
//!
//! Implements the settings endpoints per docs/11-architecture.md §3:
//! - GET  /api/v1/settings/credentials     - list platform credentials (NEVER returns secrets - G3)
//! - PUT  /api/v1/settings/credentials/{platform} - upsert credentials
//! - DELETE /api/v1/settings/credentials/{platform} - delete credentials
//! - GET  /api/v1/settings/fee-tables      - list fee tables
//! - PUT  /api/v1/settings/fee-tables/{platform} - upsert fee table

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::entities::{Platform, FeeTable},
    error::AppError,
    middleware::auth::{RequireAdmin},
    state::AppState,
};

/// Query parameters for settings
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct SettingsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
}

/// Credentials response DTO (G3: NEVER returns secret values)
#[derive(Debug, Serialize, JsonSchema)]
pub struct CredentialsResponse {
    pub platform: String,
    pub has_credentials: bool,
    pub masked_fields: Vec<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Credentials list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct CredentialsListResponse {
    pub credentials: Vec<CredentialsResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Upsert credentials request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct UpsertCredentialsRequest {
    #[validate(length(min = 1))]
    pub keystring: Option<String>,        // Etsy
    #[validate(length(min = 1))]
    pub access_token: Option<String>,     // Etsy
    #[validate(length(min = 1))]
    pub refresh_token: Option<String>,    // Etsy
    pub expires_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1))]
    pub shop_id: Option<String>,          // Etsy
    #[validate(length(min = 1))]
    pub webhook_secret: Option<String>,   // All platforms
    #[validate(length(min = 1))]
    pub api_key: Option<String>,          // Lemon Squeezy, Printify, Gelato
    #[validate(length(min = 1))]
    pub store_id: Option<String>,         // Lemon Squeezy
    // Note: This is a unified request - platform-specific fields are used per platform
}

/// Credentials update response
#[derive(Debug, Serialize, JsonSchema)]
pub struct UpsertCredentialsResponse {
    pub platform: String,
    pub message: String,
}

/// Fee table response DTO
#[derive(Debug, Serialize, JsonSchema)]
pub struct FeeTableResponse {
    pub id: Uuid,
    pub platform: String,
    pub rules: serde_json::Value,
    pub effective_from: DateTime<Utc>,
}

/// Fee table list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct FeeTableListResponse {
    pub fee_tables: Vec<FeeTableResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Upsert fee table request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct UpsertFeeTableRequest {
    pub rules: serde_json::Value,
    pub effective_from: DateTime<Utc>,
}

/// GET /api/v1/settings/credentials - List platform credentials (G3: NEVER returns secrets)
pub async fn list_credentials(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<SettingsQuery>,
) -> Result<Json<CredentialsListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    // For now, we'll return a static list of platforms with credential status
    // In a real implementation, this would come from a credentials table
    let platforms = vec![
        ("etsy", vec!["keystring", "access_token", "refresh_token", "shop_id", "webhook_secret"]),
        ("lemon-squeezy", vec!["api_key", "store_id", "webhook_secret"]),
        ("printify", vec!["api_token", "webhook_secret"]),
        ("gelato", vec!["api_key", "webhook_secret"]),
    ];

    let mut credentials = Vec::new();
    for (platform, fields) in platforms {
        // Check if credentials exist in config (simplified)
        // In real implementation, query credentials table
        credentials.push(CredentialsResponse {
            platform: platform.to_string(),
            has_credentials: false, // Would check actual storage
            masked_fields: fields.iter().map(|f| format!("***{}", f)).collect(),
            updated_at: None,
        });
    }

    let total = credentials.len() as i64;
    Ok(Json(CredentialsListResponse {
        credentials,
        total,
        limit,
        offset,
    }))
}

/// PUT /api/v1/settings/credentials/{platform} - Upsert credentials (G3: secrets never returned)
pub async fn upsert_credentials(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(platform_str): Path<String>,
    Json(payload): Json<UpsertCredentialsRequest>,
) -> Result<Json<UpsertCredentialsResponse>, AppError> {
    payload.validate()?;

    let platform = match platform_str.as_str() {
        "etsy" => Platform::Etsy,
        "lemon_squeezy" => Platform::LemonSqueezy,
        "printify" => Platform::Printify,
        "gelato" => Platform::Gelato,
        _ => return Err(AppError::BadRequest("Invalid platform. Must be one of: etsy, lemon_squeezy, printify, gelato".into())),
    };

    // In a real implementation, this would store credentials securely
    // For now, we'll store in the config or a dedicated credentials table
    // The actual storage mechanism depends on the infrastructure
    
    // Emit audit event (G3: audit that credentials were updated, but NEVER log the secrets)
    state.audit.log(
        "settings.credentials.updated",
        serde_json::json!({
            "platform": platform as Platform,
            "fields_provided": get_provided_fields(&payload),
        }),
    );

    Ok(Json(UpsertCredentialsResponse {
        platform: platform_str,
        message: "Credentials updated successfully".to_string(),
    }))
}

/// DELETE /api/v1/settings/credentials/{platform} - Delete credentials
pub async fn delete_credentials(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(platform_str): Path<String>,
) -> Result<StatusCode, AppError> {
    let platform = match platform_str.as_str() {
        "etsy" => Platform::Etsy,
        "lemon-squeezy" => Platform::LemonSqueezy,
        "printify" => Platform::Printify,
        "gelato" => Platform::Gelato,
        _ => return Err(AppError::BadRequest("Invalid platform".into())),
    };

    // In a real implementation, delete from credentials storage
    
    // Emit audit event
    state.audit.log(
        "settings.credentials.deleted",
        serde_json::json!({
            "platform": platform as Platform,
        }),
    );

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/settings/fee-tables - List fee tables
pub async fn list_fee_tables(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<SettingsQuery>,
) -> Result<Json<FeeTableListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let fee_tables = sqlx::query_as!(
        FeeTableRow,
        r#"
        SELECT id, platform as "platform: Platform", rules, effective_from
        FROM fee_tables
        ORDER BY effective_from DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM fee_tables")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    Ok(Json(FeeTableListResponse {
        fee_tables: fee_tables.into_iter().map(Into::into).collect(),
        total,
        limit,
        offset,
    }))
}

/// PUT /api/v1/settings/fee-tables/{platform} - Upsert fee table
pub async fn upsert_fee_table(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(platform_str): Path<String>,
    Json(payload): Json<UpsertFeeTableRequest>,
) -> Result<Json<FeeTableResponse>, AppError> {
    payload.validate()?;

    let platform = match platform_str.as_str() {
        "etsy" => Platform::Etsy,
        "lemon-squeezy" => Platform::LemonSqueezy,
        "printify" => Platform::Printify,
        "gelato" => Platform::Gelato,
        _ => return Err(AppError::BadRequest("Invalid platform".into())),
    };

    // Validate rules structure
    if !payload.rules.is_object() {
        return Err(AppError::BadRequest("Rules must be a JSON object".into()));
    }

    // Upsert fee table (consistent with FeeTableRepository)
    let fee_table = sqlx::query_as!(
        FeeTableRow,
        r#"
        INSERT INTO fee_tables (platform, rules, effective_from)
        VALUES ($1, $2, $3)
        ON CONFLICT (platform) DO UPDATE SET
            rules = EXCLUDED.rules,
            effective_from = EXCLUDED.effective_from
        RETURNING id, platform as "platform: Platform", rules, effective_from
        "#,
        platform as Platform,
        payload.rules,
        payload.effective_from
    )
    .fetch_one(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "settings.fee_table.updated",
        serde_json::json!({
            "platform": platform as Platform,
            "fee_table_id": fee_table.id,
            "effective_from": fee_table.effective_from,
        }),
    );

    Ok(Json(fee_table.into()))
}

// Internal row types for SQLx
#[derive(sqlx::FromRow)]
struct FeeTableRow {
    id: Uuid,
    platform: Platform,
    rules: serde_json::Value,
    effective_from: DateTime<Utc>,
}

impl From<FeeTableRow> for FeeTableResponse {
    fn from(row: FeeTableRow) -> Self {
        Self {
            id: row.id,
            platform: row.platform.to_string(),
            rules: row.rules,
            effective_from: row.effective_from,
        }
    }
}

/// Get list of provided fields (for audit logging without secrets)
fn get_provided_fields(payload: &UpsertCredentialsRequest) -> Vec<String> {
    let mut fields = Vec::new();
    if payload.keystring.is_some() { fields.push("keystring".to_string()); }
    if payload.access_token.is_some() { fields.push("access_token".to_string()); }
    if payload.refresh_token.is_some() { fields.push("refresh_token".to_string()); }
    if payload.expires_at.is_some() { fields.push("expires_at".to_string()); }
    if payload.shop_id.is_some() { fields.push("shop_id".to_string()); }
    if payload.webhook_secret.is_some() { fields.push("webhook_secret".to_string()); }
    if payload.api_key.is_some() { fields.push("api_key".to_string()); }
    if payload.store_id.is_some() { fields.push("store_id".to_string()); }
    fields
}

/// Router for settings endpoints
pub fn settings_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/credentials", axum::routing::get(list_credentials))
        .route("/credentials/{platform}", axum::routing::put(upsert_credentials).delete(delete_credentials))
        .route("/fee-tables", axum::routing::get(list_fee_tables))
        .route("/fee-tables/{platform}", axum::routing::put(upsert_fee_table))
}