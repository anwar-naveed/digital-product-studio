//! Niche CRUD API handlers
//!
//! Implements the niche management endpoints per docs/11-architecture.md §3:
//! - GET  /api/v1/niches          - list niches w/ demand scores (R10)
//! - POST /api/v1/niches          - create niche (R10)
//! - GET  /api/v1/niches/{id}     - detail
//! - PATCH /api/v1/niches/{id}    - edit
//! - DELETE /api/v1/niches/{id}   - delete (cascade guard if runs exist)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
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
    error::AppError,
    middleware::auth::{AuthUser, RequireAdmin},
    state::AppState,
};

/// Query parameters for niche listing
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ListNichesQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub vertical: Option<String>,
}

/// Niche response DTO
#[derive(Debug, Serialize, JsonSchema)]
pub struct NicheResponse {
    pub id: Uuid,
    pub name: String,
    pub vertical: String,
    pub demand_signals: serde_json::Value,
    pub competition_score: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Niche creation request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct CreateNicheRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub vertical: String,
    pub demand_signals: serde_json::Value,
    pub competition_score: Decimal,
}

/// Niche update request DTO
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct UpdateNicheRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub vertical: Option<String>,
    pub demand_signals: Option<serde_json::Value>,
    pub competition_score: Option<Decimal>,
}

/// Niche list response
#[derive(Debug, Serialize, JsonSchema)]
pub struct NicheListResponse {
    pub niches: Vec<NicheResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// GET /api/v1/niches - List niches with demand scores
///
/// Returns paginated list of niches with their demand signals and competition scores.
/// Requires JWT role=admin.
pub async fn list_niches(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Query(query): Query<ListNichesQuery>,
) -> Result<Json<NicheListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let niches = if let Some(vertical) = query.vertical {
        sqlx::query_as!(
            NicheRow,
            r#"
            SELECT id, name, vertical, demand_signals, competition_score, created_at
            FROM niches
            WHERE vertical = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            vertical,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as!(
            NicheRow,
            r#"
            SELECT id, name, vertical, demand_signals, competition_score, created_at
            FROM niches
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await?
    };

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM niches")
        .fetch_one(&state.db)
        .await?
        .unwrap_or(0);

    Ok(Json(NicheListResponse {
        niches: niches.into_iter().map(Into::into).collect(),
        total,
        limit,
        offset,
    }))
}

/// POST /api/v1/niches - Create a new niche
///
/// Creates a new niche with the given parameters.
/// Requires JWT role=admin. Emits audit event (M1.3).
pub async fn create_niche(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Json(payload): Json<CreateNicheRequest>,
) -> Result<Json<NicheResponse>, AppError> {
    payload.validate()?;

    let niche = sqlx::query_as!(
        NicheRow,
        r#"
        INSERT INTO niches (name, vertical, demand_signals, competition_score)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, vertical, demand_signals, competition_score, created_at
        "#,
        payload.name,
        payload.vertical,
        payload.demand_signals,
        payload.competition_score
    )
    .fetch_one(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "niche.created",
        serde_json::json!({
            "niche_id": niche.id,
            "name": niche.name,
            "vertical": niche.vertical,
        }),
    );

    Ok(Json(niche.into()))
}

/// GET /api/v1/niches/{id} - Get niche detail
///
/// Returns full niche details by ID.
/// Requires JWT role=admin.
pub async fn get_niche(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<Json<NicheResponse>, AppError> {
    let niche = sqlx::query_as!(
        NicheRow,
        r#"
        SELECT id, name, vertical, demand_signals, competition_score, created_at
        FROM niches
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Niche not found".into()))?;

    Ok(Json(niche.into()))
}

/// PATCH /api/v1/niches/{id} - Update a niche
///
/// Partially updates a niche. Only provided fields are updated.
/// Requires JWT role=admin. Emits audit event (M1.3).
pub async fn update_niche(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNicheRequest>,
) -> Result<Json<NicheResponse>, AppError> {
    payload.validate()?;

    let existing = sqlx::query_as!(
        NicheRow,
        "SELECT id, name, vertical, demand_signals, competition_score, created_at FROM niches WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Niche not found".into()))?;

    let orig_name = existing.name.clone();
    let orig_vertical = existing.vertical.clone();
    let orig_demand = existing.demand_signals.clone();
    let orig_comp = existing.competition_score;

    let name = payload.name.unwrap_or(existing.name);
    let vertical = payload.vertical.unwrap_or(existing.vertical);
    let demand_signals = payload.demand_signals.unwrap_or(existing.demand_signals);
    let competition_score = payload.competition_score.unwrap_or(existing.competition_score);

    let niche = sqlx::query_as!(
        NicheRow,
        r#"
        UPDATE niches
        SET name = $1, vertical = $2, demand_signals = $3, competition_score = $4
        WHERE id = $5
        RETURNING id, name, vertical, demand_signals, competition_score, created_at
        "#,
        name,
        vertical,
        demand_signals,
        competition_score,
        id
    )
    .fetch_one(&state.db)
    .await?;

    // Emit audit event
    state.audit.log(
        "niche.updated",
        serde_json::json!({
            "niche_id": niche.id,
            "changes": {
                "name": name != orig_name,
                "vertical": vertical != orig_vertical,
                "demand_signals": demand_signals != orig_demand,
                "competition_score": competition_score != orig_comp,
            }
        }),
    );

    Ok(Json(niche.into()))
}

/// DELETE /api/v1/niches/{id} - Delete a niche
///
/// Deletes a niche if no production runs reference it (cascade guard).
/// Requires JWT role=admin. Emits audit event (M1.3).
pub async fn delete_niche(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let run_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM production_runs WHERE niche_id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    if run_count > 0 {
        return Err(AppError::Conflict(
            "Cannot delete niche: referenced by production runs".into(),
        ));
    }

    let deleted = sqlx::query!("DELETE FROM niches WHERE id = $1", id)
        .execute(&state.db)
        .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound("Niche not found".into()));
    }

    // Emit audit event
    state.audit.log(
        "niche.deleted",
        serde_json::json!({ "niche_id": id }),
    );

    Ok(StatusCode::NO_CONTENT)
}

// Internal row type for SQLx
#[derive(sqlx::FromRow)]
struct NicheRow {
    id: Uuid,
    name: String,
    vertical: String,
    demand_signals: serde_json::Value,
    competition_score: Decimal,
    created_at: DateTime<Utc>,
}

impl From<NicheRow> for NicheResponse {
    fn from(row: NicheRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            vertical: row.vertical,
            demand_signals: row.demand_signals,
            competition_score: row.competition_score,
            created_at: row.created_at,
        }
    }
}

/// Router for niche endpoints
pub fn niches_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_niches).post(create_niche))
        .route(
            "/{id}",
            axum::routing::get(get_niche)
                .patch(update_niche)
                .delete(delete_niche),
        )
}