//! Catalog Products API handlers
//!
//! Implements the catalog/product endpoints per docs/11-architecture.md §3:
//! - GET  /api/v1/catalog/products     - list products with pagination
//! - POST /api/v1/catalog/products     - create product bundle (R15: 3-tier pricing)
//! - GET  /api/v1/catalog/products/{id} - get product detail
//! - PATCH /api/v1/catalog/products/{id} - update product
//! - DELETE /api/v1/catalog/products/{id} - delete product

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
    db::entities::{LicenseTier, Product, ProductItem},
    error::AppError,
    middleware::auth::{RequireAdmin},
    state::AppState,
};

/// Query parameters for product listing
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ListProductsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub license_tier: Option<String>,
}

/// Product item response DTO
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProductItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Uuid,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    /// Resolved from design_variants — raw preview image key for the UI
    pub s3_key: Option<String>,
    /// Design the variant belongs to (assembly bundle key: products/<design_id>/bundle.zip)
    pub design_id: Option<Uuid>,
    /// Origin run + niche for traceability in the product view
    pub run_id: Option<Uuid>,
    pub niche_id: Option<Uuid>,
}

/// Product response DTO
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub title: String,
    pub license_tier: String,
    pub price: Decimal,
    pub s3_prefix: String,
    pub created_at: DateTime<Utc>,
    /// Draft per-platform listing metadata (Etsy/LemonSqueezy) for publish preview
    pub listing_metadata: Option<serde_json::Value>,
    /// s3 key of the first item's variant image — cover thumbnail for grid views
    pub cover_s3_key: Option<String>,
}

/// Product creation request DTO (R15: 3-tier pricing ladder)
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 100))]
    pub sku: String,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub license_tier: String, // "personal" | "small_commercial" | "extended_commercial"
    pub price: Decimal,
    pub s3_prefix: String,
    pub items: Vec<CreateProductItemRequest>,
    #[serde(default)]
    pub listing_metadata: Option<serde_json::Value>,
}

/// Product item creation request
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct CreateProductItemRequest {
    pub variant_id: Uuid,
    pub position: i32,
}

/// Product update request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub license_tier: Option<String>,
    pub price: Option<Decimal>,
    pub s3_prefix: Option<String>,
    pub items: Option<Vec<CreateProductItemRequest>>,
    pub listing_metadata: Option<serde_json::Value>,
}

/// Product list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProductListResponse {
    #[serde(rename = "items")]
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// GET /api/v1/catalog/products - List products with pagination
pub async fn list_products(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<ProductListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let products = if let Some(license_tier) = query.license_tier {
        sqlx::query_as!(
            ProductRow,
            r#"
            SELECT p.id, p.sku, p.title, p.license_tier as "license_tier: LicenseTier", p.price, p.s3_prefix, p.created_at,
                   p.listing_metadata,
                   (SELECT dv.s3_key FROM product_items pi JOIN design_variants dv ON dv.id = pi.variant_id
                     WHERE pi.product_id = p.id ORDER BY pi.position LIMIT 1) as "cover_s3_key"
            FROM products p
            WHERE p.license_tier = $1
            ORDER BY p.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            license_tier,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as!(
            ProductRow,
            r#"
            SELECT p.id, p.sku, p.title, p.license_tier as "license_tier: LicenseTier", p.price, p.s3_prefix, p.created_at,
                   p.listing_metadata,
                   (SELECT dv.s3_key FROM product_items pi JOIN design_variants dv ON dv.id = pi.variant_id
                     WHERE pi.product_id = p.id ORDER BY pi.position LIMIT 1) as "cover_s3_key"
            FROM products p
            ORDER BY p.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    };

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    Ok(Json(ProductListResponse {
        products: products.into_iter().map(Into::into).collect(),
        total,
        limit,
        offset,
    }))
}

/// POST /api/v1/catalog/products - Create a product bundle (R15: 3-tier pricing)
pub async fn create_product(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    payload.validate()?;

    // Validate license_tier
    let license_tier = match payload.license_tier.as_str() {
        "personal" => LicenseTier::Personal,
        "small_commercial" => LicenseTier::SmallCommercial,
        "extended_commercial" => LicenseTier::ExtendedCommercial,
        _ => return Err(AppError::BadRequest("Invalid license_tier. Must be one of: personal, small_commercial, extended_commercial".into())),
    };

    // Insert product
    let product = sqlx::query_as!(
        ProductRow,
        r#"
        INSERT INTO products (sku, title, license_tier, price, s3_prefix, listing_metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, sku, title, license_tier as "license_tier: LicenseTier", price, s3_prefix, created_at,
                  listing_metadata,
                  (SELECT dv.s3_key FROM product_items pi JOIN design_variants dv ON dv.id = pi.variant_id
                    WHERE pi.product_id = products.id ORDER BY pi.position LIMIT 1) as "cover_s3_key"
        "#,
        payload.sku,
        payload.title,
        license_tier as LicenseTier,
        payload.price,
        payload.s3_prefix,
        payload.listing_metadata
    )
    .fetch_one(&state.db)
    .await?;

    // Insert product items
    let items_count = payload.items.len();
    for item in payload.items {
        sqlx::query!(
            r#"
            INSERT INTO product_items (product_id, variant_id, position)
            VALUES ($1, $2, $3)
            "#,
            product.id,
            item.variant_id,
            item.position
        )
        .execute(&state.db)
        .await?;
    }

    // Emit audit event
    state.audit.log(
        "catalog.product.created",
        serde_json::json!({
            "product_id": product.id,
            "sku": product.sku,
            "title": product.title,
            "license_tier": product.license_tier,
            "price": product.price,
            "items_count": items_count,
        }),
    );

    Ok(Json(product.into()))
}

/// GET /api/v1/catalog/products/{id} - Get product detail with items
pub async fn get_product(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductDetailResponse>, AppError> {
    let product = sqlx::query_as!(
        ProductRow,
        r#"
        SELECT p.id, p.sku, p.title, p.license_tier as "license_tier: LicenseTier", p.price, p.s3_prefix, p.created_at,
               p.listing_metadata,
               (SELECT dv.s3_key FROM product_items pi JOIN design_variants dv ON dv.id = pi.variant_id
                 WHERE pi.product_id = p.id ORDER BY pi.position LIMIT 1) as "cover_s3_key"
        FROM products p
        WHERE p.id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Product not found".into()))?;

    // Fetch product items joined to variant + design for previews & origin
    let items = sqlx::query_as!(
        ProductItemRow,
        r#"
        SELECT pi.id, pi.product_id, pi.variant_id, pi.position, pi.created_at,
               dv.s3_key,
               dv.design_id,
               d.run_id,
               pr.niche_id
        FROM product_items pi
        LEFT JOIN design_variants dv ON dv.id = pi.variant_id
        LEFT JOIN designs d ON d.id = dv.design_id
        LEFT JOIN production_runs pr ON pr.id = d.run_id
        WHERE pi.product_id = $1
        ORDER BY pi.position
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ProductDetailResponse {
        product: product.into(),
        items: items.into_iter().map(Into::into).collect(),
    }))
}

/// PATCH /api/v1/catalog/products/{id} - Update a product
pub async fn update_product(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    payload.validate()?;

    let existing = sqlx::query_as!(
        ProductRow,
        r#"SELECT p.id, p.sku, p.title, p.license_tier as "license_tier: LicenseTier", p.price, p.s3_prefix, p.created_at,
                  p.listing_metadata,
                  (SELECT dv.s3_key FROM product_items pi JOIN design_variants dv ON dv.id = pi.variant_id
                    WHERE pi.product_id = p.id ORDER BY pi.position LIMIT 1) as "cover_s3_key"
           FROM products p WHERE p.id = $1"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Product not found".into()))?;

    let orig_title = existing.title.clone();
    let orig_license_tier = existing.license_tier;
    let orig_price = existing.price;
    let orig_s3_prefix = existing.s3_prefix.clone();
    let orig_items_is_some = payload.items.is_some();

    let title = payload.title.unwrap_or(existing.title);
    let license_tier = if let Some(lt) = payload.license_tier {
        match lt.as_str() {
            "personal" => LicenseTier::Personal,
            "small_commercial" => LicenseTier::SmallCommercial,
            "extended_commercial" => LicenseTier::ExtendedCommercial,
            _ => return Err(AppError::BadRequest("Invalid license_tier".into())),
        }
    } else {
        existing.license_tier
    };
    let price = payload.price.unwrap_or(existing.price);
    let s3_prefix = payload.s3_prefix.unwrap_or(existing.s3_prefix);
    let listing_metadata = payload.listing_metadata.or(existing.listing_metadata);

    let product = sqlx::query_as!(
        ProductRow,
        r#"
        UPDATE products
        SET title = $1, license_tier = $2, price = $3, s3_prefix = $4, listing_metadata = $5
        WHERE id = $6
        RETURNING id, sku, title, license_tier as "license_tier: LicenseTier", price, s3_prefix, created_at,
                  listing_metadata,
                  (SELECT dv.s3_key FROM product_items pi JOIN design_variants dv ON dv.id = pi.variant_id
                    WHERE pi.product_id = products.id ORDER BY pi.position LIMIT 1) as "cover_s3_key"
        "#,
        title,
        license_tier as LicenseTier,
        price,
        s3_prefix,
        listing_metadata,
        id
    )
    .fetch_one(&state.db)
    .await?;

    // Update items if provided
    if let Some(items) = payload.items {
        // Delete existing items
        sqlx::query!("DELETE FROM product_items WHERE product_id = $1", id)
            .execute(&state.db)
            .await?;
        
        // Insert new items
        for item in items {
            sqlx::query!(
                r#"
                INSERT INTO product_items (product_id, variant_id, position)
                VALUES ($1, $2, $3)
                "#,
                id,
                item.variant_id,
                item.position
            )
            .execute(&state.db)
            .await?;
        }
    }

    // Emit audit event
    state.audit.log(
        "catalog.product.updated",
        serde_json::json!({
            "product_id": id,
            "changes": {
                "title": title != orig_title,
                "license_tier": license_tier != orig_license_tier,
                "price": price != orig_price,
                "s3_prefix": s3_prefix != orig_s3_prefix,
                "items_updated": orig_items_is_some,
            }
        }),
    );

    Ok(Json(product.into()))
}

/// DELETE /api/v1/catalog/products/{id} - Delete a product
pub async fn delete_product(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check if any platform listings reference this product
    let listing_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM platform_listings WHERE product_id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    if listing_count > 0 {
        return Err(AppError::Conflict(
            "Cannot delete product: referenced by platform listings".into(),
        ));
    }

    // Delete product items first (cascade)
    sqlx::query!("DELETE FROM product_items WHERE product_id = $1", id)
        .execute(&state.db)
        .await?;

    let deleted = sqlx::query!("DELETE FROM products WHERE id = $1", id)
        .execute(&state.db)
        .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound("Product not found".into()));
    }

    // Emit audit event
    state.audit.log(
        "catalog.product.deleted",
        serde_json::json!({ "product_id": id }),
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Product detail response with items
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProductDetailResponse {
    pub product: ProductResponse,
    pub items: Vec<ProductItemResponse>,
}

// Internal row types for SQLx
#[derive(sqlx::FromRow)]
struct ProductRow {
    id: Uuid,
    sku: String,
    title: String,
    license_tier: LicenseTier,
    price: Decimal,
    s3_prefix: String,
    created_at: DateTime<Utc>,
    listing_metadata: Option<serde_json::Value>,
    cover_s3_key: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ProductItemRow {
    id: Uuid,
    product_id: Uuid,
    variant_id: Uuid,
    position: i32,
    created_at: DateTime<Utc>,
    s3_key: Option<String>,
    design_id: Option<Uuid>,
    run_id: Option<Uuid>,
    niche_id: Option<Uuid>,
}

impl From<ProductRow> for ProductResponse {
    fn from(row: ProductRow) -> Self {
        Self {
            id: row.id,
            sku: row.sku,
            title: row.title,
            license_tier: row.license_tier.to_string(),
            price: row.price,
            s3_prefix: row.s3_prefix,
            created_at: row.created_at,
            listing_metadata: row.listing_metadata,
            cover_s3_key: row.cover_s3_key,
        }
    }
}

impl From<ProductItemRow> for ProductItemResponse {
    fn from(row: ProductItemRow) -> Self {
        Self {
            id: row.id,
            product_id: row.product_id,
            variant_id: row.variant_id,
            position: row.position,
            created_at: row.created_at,
            s3_key: row.s3_key,
            design_id: row.design_id,
            run_id: row.run_id,
            niche_id: row.niche_id,
        }
    }
}

/// Router for catalog endpoints
pub fn catalog_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/products",
            axum::routing::get(list_products).post(create_product),
        )
        .route(
            "/products/{id}",
            axum::routing::get(get_product)
                .patch(update_product)
                .delete(delete_product),
        )
}