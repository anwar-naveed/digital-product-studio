//! Listings API handlers
//!
//! Implements the listings endpoints per docs/11-architecture.md §3:
//! - GET  /api/v1/listings                - list platform listings with filters
//! - POST /api/v1/listings/{id}/publish   - enqueue publish job (G1: adapter routing)
//! - POST /api/v1/listings/{id}/delist    - enqueue delist job
//! - POST /api/v1/listings/{id}/reprice   - compute 3-tier ladder + fee preview (G2, R9/R15)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
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
    db::entities::{LicenseTier, Platform, ListingState, PlatformListing, Product},
    error::AppError,
    middleware::auth::{RequireAdmin},
    state::AppState,
};

/// Query parameters for listings listing
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ListListingsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub platform: Option<String>,
    pub state: Option<String>,
    pub product_id: Option<Uuid>,
}

/// Platform listing response DTO
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListingResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub platform: String,
    pub platform_listing_id: String,
    pub state: String,
    pub price: Decimal,
    pub fee_table_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Listing list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListingListResponse {
    #[serde(rename = "items")]
    pub listings: Vec<ListingResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Publish request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct PublishRequest {
    pub platform: String, // "etsy" | "lemon-squeezy" | "printify" | "gelato"
    pub license_tier: Option<String>, // optional override
}

/// Delist request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct DelistRequest {
    pub platform: String,
}

/// Reprice request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct RepriceRequest {
    pub platform: String,
    pub base_price: Decimal,
}

/// Reprice response with 3-tier ladder and fee preview (G2)
#[derive(Debug, Serialize, JsonSchema)]
pub struct RepriceResponse {
    pub platform: String,
    pub base_price: Decimal,
    pub personal_price: Decimal,
    pub small_commercial_price: Decimal,
    pub extended_commercial_price: Decimal,
    pub fee_preview: FeePreview,
}

/// Fee preview for repricing (R9/R15)
#[derive(Debug, Serialize, JsonSchema)]
pub struct FeePreview {
    pub platform_fee_percent: Decimal,
    pub transaction_fee: Decimal,
    pub personal_net: Decimal,
    pub small_commercial_net: Decimal,
    pub extended_commercial_net: Decimal,
}

/// GET /api/v1/listings - List platform listings with filters
pub async fn list_listings(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<ListListingsQuery>,
) -> Result<Json<ListingListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let mut sql = String::from(
        "SELECT id, product_id, platform, platform_listing_id, state, price, fee_table_id, created_at, updated_at
         FROM platform_listings
         WHERE 1=1"
    );
    let mut params: Vec<String> = Vec::new();
    let mut param_idx = 1;

    if let Some(platform) = &query.platform {
        sql.push_str(&format!(" AND platform = ${}", param_idx));
        params.push(platform.clone());
        param_idx += 1;
    }
    if let Some(state_filter) = &query.state {
        sql.push_str(&format!(" AND state = ${}", param_idx));
        params.push(state_filter.clone());
        param_idx += 1;
    }
    if let Some(product_id) = query.product_id {
        sql.push_str(&format!(" AND product_id = ${}", param_idx));
        params.push(product_id.to_string());
        param_idx += 1;
    }

    sql.push_str(" ORDER BY created_at DESC");
    sql.push_str(&format!(" LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));
    params.push(limit.to_string());
    params.push(offset.to_string());

    // Build query dynamically - using query_as with manual binding
    // For simplicity, we'll use separate queries per filter combination
    let listings = if query.platform.is_some() && query.state.is_some() {
        sqlx::query_as!(
            ListingRow,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
            FROM platform_listings
            WHERE platform = $1 AND state = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            query.platform.as_deref().unwrap(),
            query.state.as_deref().unwrap(),
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else if query.platform.is_some() {
        sqlx::query_as!(
            ListingRow,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
            FROM platform_listings
            WHERE platform = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            query.platform.as_deref().unwrap(),
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else if query.state.is_some() {
        sqlx::query_as!(
            ListingRow,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
            FROM platform_listings
            WHERE state = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            query.state.as_deref().unwrap(),
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else if let Some(product_id) = query.product_id {
        sqlx::query_as!(
            ListingRow,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
            FROM platform_listings
            WHERE product_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            product_id,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as!(
            ListingRow,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
            FROM platform_listings
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    };

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM platform_listings")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    Ok(Json(ListingListResponse {
        listings: listings.into_iter().map(Into::into).collect(),
        total,
        limit,
        offset,
    }))
}

/// POST /api/v1/listings/{id}/publish - Enqueue publish job with adapter routing (G1)
pub async fn publish_listing(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<PublishRequest>,
) -> Result<Json<PublishResponse>, AppError> {
    payload.validate()?;

    // Get the product
    let product = sqlx::query_as!(
        ProductRow,
        "SELECT id, sku, title, license_tier as \"license_tier: LicenseTier\", price, s3_prefix, created_at FROM products WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Product not found".into()))?;

    // Get platform enum
    let platform = match payload.platform.as_str() {
        "etsy" => Platform::Etsy,
        "lemon_squeezy" => Platform::LemonSqueezy,
        "printify" => Platform::Printify,
        "gelato" => Platform::Gelato,
        _ => return Err(AppError::BadRequest("Invalid platform. Must be one of: etsy, lemon_squeezy, printify, gelato".into())),
    };

    // Determine license tier (use product's or override)
    let license_tier = if let Some(lt) = payload.license_tier {
        match lt.as_str() {
            "personal" => LicenseTier::Personal,
            "small_commercial" => LicenseTier::SmallCommercial,
            "extended_commercial" => LicenseTier::ExtendedCommercial,
            _ => return Err(AppError::BadRequest("Invalid license_tier".into())),
        }
    } else {
        product.license_tier
    };

    // Create platform listing record in Draft state
    let listing = sqlx::query_as!(
        ListingRow,
        r#"
        INSERT INTO platform_listings (product_id, platform, platform_listing_id, state, price, fee_table_id)
        VALUES ($1, $2, '', $3, $4, NULL)
        RETURNING id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
        "#,
        id,
        serde_json::to_value(platform).unwrap().as_str().unwrap().to_string(),
        serde_json::to_value(ListingState::Publishing).unwrap().as_str().unwrap().to_string(),
        product.price
    )
    .fetch_one(&state.db)
    .await?;

    // Enqueue publish job (G1: adapter routing)
    // The job will be picked up by the publish-listing worker which routes to the correct adapter
    let job = sqlx::query!(
        r#"
        INSERT INTO jobs (run_id, tool, command_spec, status, next_attempt_at)
        SELECT $1, $2, $3, $4, NOW()
        RETURNING id
        "#,
        // Use a dummy run_id - in practice this might be linked to a specific run
        uuid::Uuid::nil(),
        serde_json::to_value(crate::db::entities::JobTool::PublishListing).unwrap().as_str().unwrap().to_string(),
        serde_json::json!({
            "listing_id": listing.id,
            "product_id": id,
            "platform": platform,
            "license_tier": license_tier,
        }),
        serde_json::to_value(crate::db::entities::JobStatus::Queued).unwrap().as_str().unwrap().to_string()
    )
    .fetch_one(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "listing.publish.enqueued",
        serde_json::json!({
            "listing_id": listing.id,
            "product_id": id,
            "platform": platform,
            "job_id": job.id,
        }),
    );

    Ok(Json(PublishResponse {
        listing_id: listing.id,
        job_id: job.id,
        state: ListingState::Publishing.to_string(),
        message: "Publish job enqueued".to_string(),
    }))
}

/// POST /api/v1/listings/{id}/delist - Enqueue delist job
pub async fn delist_listing(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<DelistRequest>,
) -> Result<Json<DelistResponse>, AppError> {
    payload.validate()?;

    let platform = match payload.platform.as_str() {
        "etsy" => Platform::Etsy,
        "lemon_squeezy" => Platform::LemonSqueezy,
        "printify" => Platform::Printify,
        "gelato" => Platform::Gelato,
        _ => return Err(AppError::BadRequest("Invalid platform".into())),
    };

    // Find the listing
    let listing = sqlx::query_as!(
        ListingRow,
        r#"
        SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
        FROM platform_listings
        WHERE product_id = $1 AND platform = $2
        "#,
        id,
        serde_json::to_value(platform).unwrap().as_str().unwrap().to_string()
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Listing not found for this product/platform".into()))?;

    // Update state to Delisting
    let updated = sqlx::query_as!(
        ListingRow,
        r#"
        UPDATE platform_listings
        SET state = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, fee_table_id, created_at, updated_at
        "#,
        ListingState::Delisting as ListingState,
        listing.id
    )
    .fetch_one(&state.db)
    .await?;

    // Enqueue delist job
    let job = sqlx::query!(
        r#"
        INSERT INTO jobs (run_id, tool, command_spec, status, next_attempt_at)
        SELECT $1, $2, $3, $4, NOW()
        RETURNING id
        "#,
        uuid::Uuid::nil(),
        serde_json::to_value(crate::db::entities::JobTool::PublishListing).unwrap().as_str().unwrap().to_string(),
        serde_json::json!({
            "listing_id": updated.id,
            "product_id": id,
            "platform": platform,
            "action": "delist",
        }),
        serde_json::to_value(crate::db::entities::JobStatus::Queued).unwrap().as_str().unwrap().to_string()
    )
    .fetch_one(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "listing.delist.enqueued",
        serde_json::json!({
            "listing_id": updated.id,
            "product_id": id,
            "platform": platform,
            "job_id": job.id,
        }),
    );

    Ok(Json(DelistResponse {
        listing_id: updated.id,
        job_id: job.id,
        state: ListingState::Delisting.to_string(),
        message: "Delist job enqueued".to_string(),
    }))
}

/// POST /api/v1/listings/{id}/reprice - Compute 3-tier ladder + fee preview (G2, R9/R15)
pub async fn reprice_listing(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<RepriceRequest>,
) -> Result<Json<RepriceResponse>, AppError> {
    payload.validate()?;

    let platform = match payload.platform.as_str() {
        "etsy" => Platform::Etsy,
        "lemon_squeezy" => Platform::LemonSqueezy,
        "printify" => Platform::Printify,
        "gelato" => Platform::Gelato,
        _ => return Err(AppError::BadRequest("Invalid platform".into())),
    };

    // Get the fee table for this platform
    let fee_table = sqlx::query_as!(
        FeeTableRow,
        r#"
        SELECT id, platform as "platform: Platform", rules, effective_from
        FROM fee_tables
        WHERE platform = $1
        ORDER BY effective_from DESC
        LIMIT 1
        "#,
        serde_json::to_value(platform).unwrap().as_str().unwrap().to_string()
    )
    .fetch_optional(&state.db)
    .await?;

    // Default fee rules per platform (R9)
    let (platform_fee_percent, transaction_fee) = if let Some(ft) = fee_table {
        // Parse rules JSON - simplified for now
        let rules = ft.rules;
        let platform_fee = rules.get("platform_fee_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(get_default_platform_fee(&platform)) as f64;
        let transaction_fee_val = rules.get("transaction_fee")
            .and_then(|v| v.as_f64())
            .unwrap_or(get_default_transaction_fee(&platform)) as f64;
        (Decimal::try_from(platform_fee).unwrap_or(Decimal::ZERO), 
         Decimal::try_from(transaction_fee_val).unwrap_or(Decimal::ZERO))
    } else {
        (Decimal::try_from(get_default_platform_fee(&platform)).unwrap(), 
         Decimal::try_from(get_default_transaction_fee(&platform)).unwrap())
    };

    // Compute 3-tier pricing ladder (R15)
    let base_price = payload.base_price;
    
    // Tier multipliers per R15
    let personal_multiplier = Decimal::new(100, 0); // 1.0x
    let small_commercial_multiplier = Decimal::new(150, 2); // 1.5x
    let extended_commercial_multiplier = Decimal::new(250, 2); // 2.5x

    let personal_price = base_price * personal_multiplier / Decimal::new(100, 0);
    let small_commercial_price = base_price * small_commercial_multiplier / Decimal::new(100, 0);
    let extended_commercial_price = base_price * extended_commercial_multiplier / Decimal::new(100, 0);

    // Compute fee preview for each tier
    let personal_fee = (personal_price * platform_fee_percent / Decimal::new(100, 0)).round_dp(2) + transaction_fee;
    let small_commercial_fee = (small_commercial_price * platform_fee_percent / Decimal::new(100, 0)).round_dp(2) + transaction_fee;
    let extended_commercial_fee = (extended_commercial_price * platform_fee_percent / Decimal::new(100, 0)).round_dp(2) + transaction_fee;

    let personal_net = personal_price - personal_fee;
    let small_commercial_net = small_commercial_price - small_commercial_fee;
    let extended_commercial_net = extended_commercial_price - extended_commercial_fee;

    // Emit audit event
    state.audit.log(
        "listing.repriced",
        serde_json::json!({
            "product_id": id,
            "platform": platform,
            "base_price": base_price,
            "personal_price": personal_price,
            "small_commercial_price": small_commercial_price,
            "extended_commercial_price": extended_commercial_price,
        }),
    );

    Ok(Json(RepriceResponse {
        platform: payload.platform,
        base_price,
        personal_price,
        small_commercial_price,
        extended_commercial_price,
        fee_preview: FeePreview {
            platform_fee_percent,
            transaction_fee,
            personal_net,
            small_commercial_net,
            extended_commercial_net,
        },
    }))
}

/// Response for publish
#[derive(Debug, Serialize, JsonSchema)]
pub struct PublishResponse {
    pub listing_id: Uuid,
    pub job_id: Uuid,
    pub state: String,
    pub message: String,
}

/// Response for delist
#[derive(Debug, Serialize, JsonSchema)]
pub struct DelistResponse {
    pub listing_id: Uuid,
    pub job_id: Uuid,
    pub state: String,
    pub message: String,
}

// Internal row types for SQLx
#[derive(sqlx::FromRow)]
struct ListingRow {
    id: Uuid,
    product_id: Uuid,
    platform: Platform,
    platform_listing_id: String,
    state: ListingState,
    price: Decimal,
    fee_table_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ProductRow {
    id: Uuid,
    sku: String,
    title: String,
    license_tier: LicenseTier,
    price: Decimal,
    s3_prefix: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct FeeTableRow {
    id: Uuid,
    platform: Platform,
    rules: serde_json::Value,
    effective_from: DateTime<Utc>,
}

impl From<ListingRow> for ListingResponse {
    fn from(row: ListingRow) -> Self {
        Self {
            id: row.id,
            product_id: row.product_id,
            platform: row.platform.to_string(),
            platform_listing_id: row.platform_listing_id,
            state: row.state.to_string(),
            price: row.price,
            fee_table_id: row.fee_table_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Get default platform fee percentage
fn get_default_platform_fee(platform: &Platform) -> f64 {
    match platform {
        Platform::Etsy => 6.5,        // Etsy: 6.5% transaction fee
        Platform::LemonSqueezy => 5.0, // Lemon Squeezy: 5% + $0.50
        Platform::Printify => 0.0,    // Printify: cost-based, no platform fee
        Platform::Gelato => 0.0,      // Gelato: cost-based, no platform fee
    }
}

/// Get default transaction fee
fn get_default_transaction_fee(platform: &Platform) -> f64 {
    match platform {
        Platform::Etsy => 0.25,       // Etsy: $0.25 listing fee (per listing, not per sale)
        Platform::LemonSqueezy => 0.50, // Lemon Squeezy: $0.50 per transaction
        Platform::Printify => 0.0,
        Platform::Gelato => 0.0,
    }
}

/// Router for listings endpoints
pub fn listings_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_listings))
        .route("/{id}/publish", axum::routing::post(publish_listing))
        .route("/{id}/delist", axum::routing::post(delist_listing))
        .route("/{id}/reprice", axum::routing::post(reprice_listing))
}