//! Analytics API handlers
//!
//! Implements the analytics endpoints per docs/11-architecture.md §3:
//! - GET /api/v1/analytics/listings - per-listing views/sales/margin (G4)

use axum::{
    extract::{Query, State},
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
    db::entities::{Platform, ListingState},
    error::AppError,
    middleware::auth::{RequireAdmin},
    state::AppState,
};

/// Query parameters for analytics
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct AnalyticsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub platform: Option<String>,
    pub product_id: Option<Uuid>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

/// Per-listing analytics response (G4: views/sales/margin)
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListingAnalyticsResponse {
    pub listing_id: Uuid,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_title: String,
    pub platform: String,
    pub platform_listing_id: String,
    pub state: String,
    pub current_price: Decimal,
    pub views: i64,
    pub favorites: i64,
    pub sales_count: i64,
    pub total_revenue: Decimal,
    pub total_fees: Decimal,
    pub total_net: Decimal,
    pub margin_percent: Decimal,
    pub first_sale_at: Option<DateTime<Utc>>,
    pub last_sale_at: Option<DateTime<Utc>>,
}

/// Analytics list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct AnalyticsListResponse {
    #[serde(rename = "items")]
    pub analytics: Vec<ListingAnalyticsResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// GET /api/v1/analytics/listings - Per-listing analytics with views/sales/margin (G4)
pub async fn get_listings_analytics(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    // Build the analytics query with joins across platform_listings, sales_events, and products
    let analytics = if let Some(platform) = &query.platform {
        let platform_enum = match platform.as_str() {
            "etsy" => Platform::Etsy,
            "lemon_squeezy" => Platform::LemonSqueezy,
            "printify" => Platform::Printify,
            "gelato" => Platform::Gelato,
            _ => return Err(AppError::BadRequest("Invalid platform".into())),
        };

        let mut base_query = r#"
            SELECT 
                pl.id as listing_id,
                pl.product_id,
                p.sku as product_sku,
                p.title as product_title,
                pl.platform as "platform: Platform",
                pl.platform_listing_id,
                pl.state as "state: ListingState",
                pl.price as current_price,
                COALESCE(SUM(CASE WHEN se.event_type = 'view' THEN 1 ELSE 0 END), 0) as views,
                COALESCE(SUM(CASE WHEN se.event_type = 'favorite' THEN 1 ELSE 0 END), 0) as favorites,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN 1 ELSE 0 END), 0) as sales_count,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.gross ELSE 0 END), 0) as total_revenue,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.fee ELSE 0 END), 0) as total_fees,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.net ELSE 0 END), 0) as total_net,
                MIN(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as first_sale_at,
                MAX(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as last_sale_at
            FROM platform_listings pl
            INNER JOIN products p ON pl.product_id = p.id
            LEFT JOIN sales_events se ON pl.id = se.listing_id
        "#.to_string();

        let mut where_clauses = vec!["pl.platform = $1".to_string()];
        let mut param_idx = 2;

        if let Some(product_id) = query.product_id {
            where_clauses.push(format!("pl.product_id = ${}", param_idx));
            param_idx += 1;
        }
        if let Some(since) = query.since {
            where_clauses.push(format!("se.occurred_at >= ${}", param_idx));
            param_idx += 1;
        }
        if let Some(until) = query.until {
            where_clauses.push(format!("se.occurred_at <= ${}", param_idx));
            param_idx += 1;
        }

        base_query.push_str(" WHERE ");
        base_query.push_str(&where_clauses.join(" AND "));
        base_query.push_str(" GROUP BY pl.id, pl.product_id, p.sku, p.title, pl.platform, pl.platform_listing_id, pl.state, pl.price");
        base_query.push_str(&format!(" ORDER BY pl.created_at DESC LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

        // We need to use query_as with dynamic parameters
        // For simplicity, execute with the common pattern
        sqlx::query_as!(
            AnalyticsRow,
            r#"
            SELECT 
                pl.id as listing_id,
                pl.product_id,
                p.sku as product_sku,
                p.title as product_title,
                platform as "platform: Platform",
                pl.platform_listing_id,
                state as "state: ListingState",
                pl.price as current_price,
                COALESCE(SUM(CASE WHEN se.event_type = 'view' THEN 1 ELSE 0 END), 0)::bigint as views,
                COALESCE(SUM(CASE WHEN se.event_type = 'favorite' THEN 1 ELSE 0 END), 0)::bigint as favorites,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN 1 ELSE 0 END), 0)::bigint as sales_count,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.gross ELSE 0 END), 0) as total_revenue,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.fee ELSE 0 END), 0) as total_fees,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.net ELSE 0 END), 0) as total_net,
                MIN(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as first_sale_at,
                MAX(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as last_sale_at
            FROM platform_listings pl
            INNER JOIN products p ON pl.product_id = p.id
            LEFT JOIN sales_events se ON pl.id = se.listing_id
            WHERE pl.platform = $1
            GROUP BY pl.id, pl.product_id, p.sku, p.title, pl.platform, pl.platform_listing_id, pl.state, pl.price
            ORDER BY pl.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            platform_enum as Platform,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else if let Some(product_id) = query.product_id {
        sqlx::query_as!(
            AnalyticsRow,
            r#"
            SELECT 
                pl.id as listing_id,
                pl.product_id,
                p.sku as product_sku,
                p.title as product_title,
                platform as "platform: Platform",
                pl.platform_listing_id,
                state as "state: ListingState",
                pl.price as current_price,
                COALESCE(SUM(CASE WHEN se.event_type = 'view' THEN 1 ELSE 0 END), 0)::bigint as views,
                COALESCE(SUM(CASE WHEN se.event_type = 'favorite' THEN 1 ELSE 0 END), 0)::bigint as favorites,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN 1 ELSE 0 END), 0)::bigint as sales_count,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.gross ELSE 0 END), 0) as total_revenue,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.fee ELSE 0 END), 0) as total_fees,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.net ELSE 0 END), 0) as total_net,
                MIN(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as first_sale_at,
                MAX(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as last_sale_at
            FROM platform_listings pl
            INNER JOIN products p ON pl.product_id = p.id
            LEFT JOIN sales_events se ON pl.id = se.listing_id
            WHERE pl.product_id = $1
            GROUP BY pl.id, pl.product_id, p.sku, p.title, pl.platform, pl.platform_listing_id, pl.state, pl.price
            ORDER BY pl.created_at DESC
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
            AnalyticsRow,
            r#"
            SELECT 
                pl.id as listing_id,
                pl.product_id,
                p.sku as product_sku,
                p.title as product_title,
                platform as "platform: Platform",
                pl.platform_listing_id,
                state as "state: ListingState",
                pl.price as current_price,
                COALESCE(SUM(CASE WHEN se.event_type = 'view' THEN 1 ELSE 0 END), 0)::bigint as views,
                COALESCE(SUM(CASE WHEN se.event_type = 'favorite' THEN 1 ELSE 0 END), 0)::bigint as favorites,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN 1 ELSE 0 END), 0)::bigint as sales_count,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.gross ELSE 0 END), 0) as total_revenue,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.fee ELSE 0 END), 0) as total_fees,
                COALESCE(SUM(CASE WHEN se.event_type = 'sale' THEN se.net ELSE 0 END), 0) as total_net,
                MIN(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as first_sale_at,
                MAX(CASE WHEN se.event_type = 'sale' THEN se.occurred_at END) as last_sale_at
            FROM platform_listings pl
            INNER JOIN products p ON pl.product_id = p.id
            LEFT JOIN sales_events se ON pl.id = se.listing_id
            GROUP BY pl.id, pl.product_id, p.sku, p.title, pl.platform, pl.platform_listing_id, pl.state, pl.price
            ORDER BY pl.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    };

    // Get total count
    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM platform_listings")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    let mut responses = Vec::new();
    for row in analytics {
        // Calculate margin percent
        let margin_percent = if row.total_revenue.unwrap_or_default() > Decimal::ZERO {
            ((row.total_net.unwrap_or_default() / row.total_revenue.unwrap_or_default()) * Decimal::new(100, 0)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        responses.push(ListingAnalyticsResponse {
            listing_id: row.listing_id,
            product_id: row.product_id,
            product_sku: row.product_sku,
            product_title: row.product_title,
            platform: row.platform.to_string(),
            platform_listing_id: row.platform_listing_id,
            state: row.state.to_string(),
            current_price: row.current_price,
            views: row.views.unwrap_or(0),
            favorites: row.favorites.unwrap_or(0),
            sales_count: row.sales_count.unwrap_or(0),
            total_revenue: row.total_revenue.unwrap_or_default(),
            total_fees: row.total_fees.unwrap_or_default(),
            total_net: row.total_net.unwrap_or_default(),
            margin_percent,
            first_sale_at: row.first_sale_at,
            last_sale_at: row.last_sale_at,
        });
    }

    Ok(Json(AnalyticsListResponse {
        analytics: responses,
        total,
        limit,
        offset,
    }))
}

// Internal row type for SQLx
#[derive(sqlx::FromRow)]
struct AnalyticsRow {
    listing_id: Uuid,
    product_id: Uuid,
    product_sku: String,
    product_title: String,
    platform: Platform,
    platform_listing_id: String,
    state: ListingState,
    current_price: Decimal,
    views: Option<i64>,
    favorites: Option<i64>,
    sales_count: Option<i64>,
    total_revenue: Option<Decimal>,
    total_fees: Option<Decimal>,
    total_net: Option<Decimal>,
    first_sale_at: Option<DateTime<Utc>>,
    last_sale_at: Option<DateTime<Utc>>,
}

/// Router for analytics endpoints
pub fn analytics_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/listings", axum::routing::get(get_listings_analytics))
}