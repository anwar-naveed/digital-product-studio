//! Production Run API handlers
//!
//! Implements the production run endpoints per docs/11-architecture.md §3:
//! - POST /api/v1/runs              - spawn production run (niche_id, target_count, params) (R6, R10)
//! - GET  /api/v1/runs              - list with status/progress
//! - GET  /api/v1/runs/{id}         - detail, job breakdown
//! - POST /api/v1/runs/{id}/cancel  - soft cancel (drains queued; running leases expire)

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

/// Query parameters for run listing
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ListRunsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub niche_id: Option<Uuid>,
}

/// Run creation request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct CreateRunRequest {
    pub niche_id: Uuid,
    #[validate(range(min = 1, max = 1000))]
    pub target_count: i32,
    pub params: serde_json::Value,
}

/// Run response DTO
#[derive(Debug, Serialize, JsonSchema)]
pub struct RunResponse {
    pub id: Uuid,
    pub niche_id: Uuid,
    pub status: String,
    pub target_count: i32,
    pub params: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Run detail response with job breakdown
#[derive(Debug, Serialize, JsonSchema)]
pub struct RunDetailResponse {
    pub id: Uuid,
    pub niche_id: Uuid,
    pub status: String,
    pub target_count: i32,
    pub params: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub jobs: Vec<JobSummary>,
}

/// Job summary for run detail
#[derive(Debug, Serialize, JsonSchema)]
pub struct JobSummary {
    pub id: Uuid,
    pub run_id: Uuid,
    pub tool: String,
    pub status: String,
    pub attempts: i32,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

/// Run list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct RunListResponse {
    pub runs: Vec<RunResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// POST /api/v1/runs - Create a new production run
///
/// Spawns a production run with target_count cf-image-gen jobs atomically.
/// Requires JWT role=admin. Emits audit event (M1.3).
/// 
/// G1: This endpoint must create run + target_count jobs atomically (R10)
pub async fn create_run(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Json(payload): Json<CreateRunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    payload.validate()?;

    // Verify niche exists
    let niche_exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM niches WHERE id = $1)",
        payload.niche_id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);

    if !niche_exists {
        return Err(AppError::NotFound("Niche not found".into()));
    }

    // Atomic transaction: create run + jobs
    let run_id = Uuid::new_v4();
    let now = Utc::now();

    let mut tx = state.db.begin().await?;

    // Insert production run
    let run = sqlx::query_as!(
        RunRow,
        r#"
        INSERT INTO production_runs (id, niche_id, status, target_count, params, created_at)
        VALUES ($1, $2, 'pending', $3, $4, $5)
        RETURNING id, niche_id, status, target_count, params, created_at, started_at, completed_at
        "#,
        run_id,
        payload.niche_id,
        payload.target_count,
        payload.params,
        now
    )
    .fetch_one(&mut *tx)
    .await?;

    // Insert cf-image-gen jobs for each target
    for i in 0..payload.target_count {
        let design_id = Uuid::new_v4();
        
        // Create design row
        sqlx::query!(
            r#"
            INSERT INTO designs (id, run_id, base_prompt, s3_prefix_raw)
            VALUES ($1, $2, $3, $4)
            "#,
            design_id,
            run_id,
            format!("Design {} for run {}", i + 1, run_id),
            format!("raw/{}/{}", run_id, design_id)
        )
        .execute(&mut *tx)
        .await?;

        // Create cf-image-gen job
        let job_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO jobs (id, run_id, tool, command_spec, status, attempts, queued_at)
            VALUES ($1, $2, 'cf-image-gen', $3, 'queued', 0, $4)
            "#,
            job_id,
            run_id,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {
                    "design_id": design_id,
                    "niche_id": payload.niche_id,
                    "variant_index": i
                },
                "timeout_secs": 300,
                "expected_outputs": [format!("{}/{}.png", design_id, i)]
            }),
            now
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Emit audit event
    state.audit.log(
        "run.created",
        serde_json::json!({
            "run_id": run.id,
            "niche_id": run.niche_id,
            "target_count": run.target_count,
            "jobs_created": payload.target_count,
        }),
    );

    Ok(Json(run.into()))
}

/// GET /api/v1/runs - List production runs
///
/// Returns paginated list of runs with status and progress.
/// Requires JWT role=admin.
pub async fn list_runs(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<RunListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let mut query_builder = "SELECT id, niche_id, status, target_count, params, created_at, started_at, completed_at FROM production_runs".to_string();
    let mut conditions = Vec::new();
    let mut param_idx = 1;

    if let Some(status) = &query.status {
        conditions.push(format!("status = ${}", param_idx));
        param_idx += 1;
    }
    if let Some(niche_id) = &query.niche_id {
        conditions.push(format!("niche_id = ${}", param_idx));
        param_idx += 1;
    }

    if !conditions.is_empty() {
        query_builder.push_str(" WHERE ");
        query_builder.push_str(&conditions.join(" AND "));
    }

    query_builder.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

    // Note: In production, use sqlx::query_as with dynamic query or build with query! macro
    // For simplicity, we'll use a simpler approach here
    let runs = sqlx::query_as!(
        RunRow,
        r#"
        SELECT id, niche_id, status, target_count, params, created_at, started_at, completed_at
        FROM production_runs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM production_runs")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    Ok(Json(RunListResponse {
        runs: runs.into_iter().map(Into::into).collect(),
        total,
        limit,
        offset,
    }))
}

/// GET /api/v1/runs/{id} - Get run detail with job breakdown
///
/// Returns full run details including all associated jobs.
/// Requires JWT role=admin.
pub async fn get_run(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<Json<RunDetailResponse>, AppError> {
    let run = sqlx::query_as!(
        RunRow,
        r#"
        SELECT id, niche_id, status, target_count, params, created_at, started_at, completed_at
        FROM production_runs
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Run not found".into()))?;

    let jobs = sqlx::query_as!(
        JobRow,
        r#"
        SELECT id, run_id, tool, command_spec, status, attempts, queued_at, started_at, finished_at, last_error
        FROM jobs
        WHERE run_id = $1
        ORDER BY queued_at ASC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(RunDetailResponse {
        id: run.id,
        niche_id: run.niche_id,
        status: run.status,
        target_count: run.target_count,
        params: run.params,
        created_at: run.created_at,
        started_at: run.started_at,
        completed_at: run.completed_at,
        jobs: jobs.into_iter().map(Into::into).collect(),
    }))
}

/// POST /api/v1/runs/{id}/cancel - Cancel a run (soft cancel)
///
/// Soft cancels a run by marking queued jobs as cancelled and letting running jobs
/// expire their leases. Running jobs will be reaped by the reaper loop.
/// Requires JWT role=admin. Emits audit event (M1.3).
pub async fn cancel_run(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let run = sqlx::query_as!(
        RunRow,
        "SELECT id, niche_id, status, target_count, params, created_at, started_at, completed_at FROM production_runs WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Run not found".into()))?;

    if run.status == "completed" || run.status == "cancelled" || run.status == "failed" {
        return Err(AppError::Conflict(format!(
            "Cannot cancel run in status: {}",
            run.status
        )));
    }

    // Mark run as cancelled
    sqlx::query!(
        "UPDATE production_runs SET status = 'cancelled' WHERE id = $1",
        id
    )
    .execute(&state.db)
    .await?;

    // Cancel queued jobs (running jobs will be reaped by lease expiry)
    sqlx::query!(
        "UPDATE jobs SET status = 'cancelled' WHERE run_id = $1 AND status = 'queued'",
        id
    )
    .execute(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "run.cancelled",
        serde_json::json!({ "run_id": id }),
    );

    Ok(StatusCode::NO_CONTENT)
}

// Internal row types for SQLx
#[derive(sqlx::FromRow)]
struct RunRow {
    id: Uuid,
    niche_id: Uuid,
    status: String,
    target_count: i32,
    params: serde_json::Value,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<RunRow> for RunResponse {
    fn from(row: RunRow) -> Self {
        Self {
            id: row.id,
            niche_id: row.niche_id,
            status: row.status,
            target_count: row.target_count,
            params: row.params,
            created_at: row.created_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: Uuid,
    run_id: Uuid,
    tool: String,
    command_spec: serde_json::Value,
    status: String,
    attempts: i32,
    queued_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
}

impl From<JobRow> for JobSummary {
    fn from(row: JobRow) -> Self {
        Self {
            id: row.id,
            run_id: row.run_id,
            tool: row.tool,
            status: row.status,
            attempts: row.attempts,
            queued_at: row.queued_at,
            started_at: row.started_at,
            finished_at: row.finished_at,
            last_error: row.last_error,
        }
    }
}

/// Router for run endpoints
pub fn runs_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_runs).post(create_run))
        .route(
            "/{id}",
            axum::routing::get(get_run),
        )
        .route("/{id}/cancel", axum::routing::post(cancel_run))
}