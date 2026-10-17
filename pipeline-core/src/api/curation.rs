//! Curation API handlers
//!
//! Implements the curation endpoints per docs/11-architecture.md §3 and §6:
//! - GET  /api/v1/curation/queue              - paginated previews of decided=pending
//! - POST /api/v1/curation/{variant_id}/approve - approve → curated S3 copy job
//! - POST /api/v1/curation/{variant_id}/reject  - reject with reason

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, RequireAdmin},
    state::AppState,
};

/// Query parameters for curation queue listing
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct CurationQueueQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub decided: Option<String>, // pending, approved, rejected
}

/// Curation queue item response
#[derive(Debug, Serialize, JsonSchema)]
pub struct CurationQueueItem {
    pub variant_id: Uuid,
    pub design_id: Uuid,
    pub run_id: Uuid,
    pub niche_id: Uuid,
    pub s3_key: String,
    pub sha256: String,
    pub kind: String,
    pub base_prompt: String,
    pub presigned_url: String,
    pub created_at: DateTime<Utc>,
}

/// Curation queue list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct CurationQueueResponse {
    pub items: Vec<CurationQueueItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Curation approve request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ApproveCurationRequest {
    /// Optional reviewer ID (defaults to authenticated admin)
    pub reviewer_id: Option<Uuid>,
}

/// Curation reject request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct RejectCurationRequest {
    /// Required reject reason from taxonomy
    #[validate(length(min = 1, max = 100))]
    pub reject_reason: String,
    /// Optional reviewer ID (defaults to authenticated admin)
    pub reviewer_id: Option<Uuid>,
}

/// Curation decision response
#[derive(Debug, Serialize, JsonSchema)]
pub struct CurationDecisionResponse {
    pub variant_id: Uuid,
    pub decision: String,
    pub reviewer_id: Uuid,
    pub decided_at: DateTime<Utc>,
    pub job_enqueued: Option<Uuid>,
}

/// GET /api/v1/curation/queue - Get curation queue
///
/// Returns paginated previews of variants with decided=pending.
/// Each item includes a presigned S3 URL for preview.
/// Requires JWT role=admin.
pub async fn get_curation_queue(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<CurationQueueQuery>,
) -> Result<Json<CurationQueueResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    let decided = query.decided.unwrap_or_else(|| "pending".to_string());

    // Get variants with curation status
    let items = sqlx::query_as!(
        CurationQueueRow,
        r#"
        SELECT 
            dv.id as variant_id,
            dv.design_id,
            d.run_id,
            r.niche_id,
            dv.s3_key,
            dv.sha256,
            dv.kind,
            d.base_prompt,
            dv.created_at,
            c.decision as "decision: Option<String>"
        FROM design_variants dv
        JOIN designs d ON dv.design_id = d.id
        JOIN production_runs r ON d.run_id = r.id
        LEFT JOIN curations c ON c.variant_id = dv.id
        WHERE c.decision IS NULL OR c.decision = $1
        ORDER BY dv.created_at ASC
        LIMIT $2 OFFSET $3
        "#,
        decided,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM design_variants dv
        LEFT JOIN curations c ON c.variant_id = dv.id
        WHERE c.decision IS NULL OR c.decision = $1
        "#,
        decided
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    // Generate presigned URLs for each item
    let mut queue_items = Vec::new();
    for row in items {
        let presigned_url = state
            .s3_client
            .presign_get(&row.s3_key, std::time::Duration::from_secs(3600)) // 1 hour expiry
            .await
            .map_err(|e| AppError::Internal(format!("Failed to generate presigned URL: {}", e)))?;

        queue_items.push(CurationQueueItem {
            variant_id: row.variant_id,
            design_id: row.design_id,
            run_id: row.run_id,
            niche_id: row.niche_id,
            s3_key: row.s3_key,
            sha256: row.sha256,
            kind: row.kind,
            base_prompt: row.base_prompt,
            presigned_url,
            created_at: row.created_at,
        });
    }

    Ok(Json(CurationQueueResponse {
        items: queue_items,
        total,
        limit,
        offset,
    }))
}

/// POST /api/v1/curation/{variant_id}/approve - Approve a variant
///
/// Approves a variant, copies from raw/ to curated/ in S3,
/// and enqueues a product_assembly job.
/// Requires JWT role=admin. Emits audit event (M1.3).
///
/// G2: This performs COPY + assembly enqueue in one flow (§6 steps 8-11)
pub async fn approve_curation(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(variant_id): Path<Uuid>,
    Json(payload): Json<ApproveCurationRequest>,
) -> Result<Json<CurationDecisionResponse>, AppError> {
    let reviewer_id = payload.reviewer_id.unwrap_or(auth.0.user_id);
    let now = Utc::now();

    // Get variant details
    let variant = sqlx::query_as!(
        VariantRow,
        r#"
        SELECT dv.id, dv.design_id, dv.s3_key, dv.sha256, dv.kind,
               d.run_id, r.niche_id, d.base_prompt, d.s3_prefix_raw
        FROM design_variants dv
        JOIN designs d ON dv.design_id = d.id
        JOIN production_runs r ON d.run_id = r.id
        WHERE dv.id = $1
        "#,
        variant_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Variant not found".into()))?;

    // Check if already curated
    let existing: Option<String> = sqlx::query_scalar!(
        "SELECT decision FROM curations WHERE variant_id = $1",
        variant_id
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(decision) = existing {
        return Err(AppError::Conflict(format!(
            "Variant already curated with decision: {}",
            decision
        )));
    }

    // Start transaction for atomic curation + S3 copy + job enqueue
    let mut tx = state.db.begin().await?;

    // 1. Insert curation record with decision=approved
    sqlx::query!(
        r#"
        INSERT INTO curations (variant_id, decision, reviewer_id, decided_at)
        VALUES ($1, 'approved', $2, $3)
        "#,
        variant_id,
        reviewer_id,
        now
    )
    .execute(&mut *tx)
    .await?;

    // 2. S3 COPY: raw/... → curated/...
    let curated_key = format!(
        "curated/{}/{}/{}",
        variant.niche_id,
        variant.design_id,
        variant.s3_key.split('/').last().unwrap()
    );

    state
        .s3_client
        .copy(&variant.s3_key, &curated_key)
        .await
        .map_err(|e| AppError::Internal(format!("S3 copy failed: {}", e)))?;

    // 3. Enqueue product_assembly job
    let assembly_job_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, run_id, tool, command_spec, status, attempts, queued_at)
        VALUES ($1, $2, 'product-assembly', $3, 'queued', 0, $4)
        "#,
        assembly_job_id,
        variant.run_id,
        serde_json::json!({
            "tool": "product-assembly",
            "subcommand": "run",
            "args": {
                "design_id": variant.design_id,
                "variant_id": variant_id,
                "curated_key": curated_key,
                "base_prompt": variant.base_prompt,
                "niche_id": variant.niche_id
            },
            "timeout_secs": 300,
            "expected_outputs": [format!("products/{}/bundle.zip", variant.design_id)]
        }),
        now
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Emit audit event
    state.audit.log(
        "curation.approved",
        serde_json::json!({
            "variant_id": variant_id,
            "design_id": variant.design_id,
            "reviewer_id": reviewer_id,
            "curated_key": curated_key,
            "assembly_job_id": assembly_job_id,
        }),
    );

    Ok(Json(CurationDecisionResponse {
        variant_id,
        decision: "approved".to_string(),
        reviewer_id,
        decided_at: now,
        job_enqueued: Some(assembly_job_id),
    }))
}

/// POST /api/v1/curation/{variant_id}/reject - Reject a variant
///
/// Rejects a variant with a reason from the taxonomy.
/// Requires JWT role=admin. Emits audit event (M1.3).
pub async fn reject_curation(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(variant_id): Path<Uuid>,
    Json(payload): Json<RejectCurationRequest>,
) -> Result<Json<CurationDecisionResponse>, AppError> {
    payload.validate()?;

    // Validate reject reason against taxonomy
    let valid_reasons = [
        "style_mismatch",
        "artifact",
        "license",
        "off_niche",
        "duplicate",
    ];
    if !valid_reasons.contains(&payload.reject_reason.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid reject reason. Must be one of: {}",
            valid_reasons.join(", ")
        )));
    }

    let reviewer_id = payload.reviewer_id.unwrap_or(auth.0.user_id);
    let now = Utc::now();

    // Check variant exists
    let exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM design_variants WHERE id = $1)",
        variant_id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound("Variant not found".into()));
    }

    // Check if already curated
    let existing: Option<String> = sqlx::query_scalar!(
        "SELECT decision FROM curations WHERE variant_id = $1",
        variant_id
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(decision) = existing {
        return Err(AppError::Conflict(format!(
            "Variant already curated with decision: {}",
            decision
        )));
    }

    // Insert curation record with decision=rejected
    sqlx::query!(
        r#"
        INSERT INTO curations (variant_id, decision, reject_reason, reviewer_id, decided_at)
        VALUES ($1, 'rejected', $2, $3, $4)
        "#,
        variant_id,
        payload.reject_reason,
        reviewer_id,
        now
    )
    .execute(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "curation.rejected",
        serde_json::json!({
            "variant_id": variant_id,
            "reject_reason": payload.reject_reason,
            "reviewer_id": reviewer_id,
        }),
    );

    Ok(Json(CurationDecisionResponse {
        variant_id,
        decision: "rejected".to_string(),
        reviewer_id,
        decided_at: now,
        job_enqueued: None,
    }))
}

// Internal row types for SQLx
#[derive(sqlx::FromRow)]
struct CurationQueueRow {
    variant_id: Uuid,
    design_id: Uuid,
    run_id: Uuid,
    niche_id: Uuid,
    s3_key: String,
    sha256: String,
    kind: String,
    base_prompt: String,
    created_at: DateTime<Utc>,
    decision: Option<String>,
}

#[derive(sqlx::FromRow)]
struct VariantRow {
    id: Uuid,
    design_id: Uuid,
    s3_key: String,
    sha256: String,
    kind: String,
    run_id: Uuid,
    niche_id: Uuid,
    base_prompt: String,
    s3_prefix_raw: String,
}

/// Router for curation endpoints
pub fn curation_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/queue", axum::routing::get(get_curation_queue))
        .route(
            "/{variant_id}/approve",
            axum::routing::post(approve_curation),
        )
        .route(
            "/{variant_id}/reject",
            axum::routing::post(reject_curation),
        )
}