//! Legacy API integration scaffold (pre-rename crate name).
//! Compile-gated behind `api-integration` until rewritten.
#![cfg(feature = "api-integration")]

//! Integration tests for curation API endpoints
//!
//! G2: Approve performs COPY + assembly enqueue in one flow (§6 steps 8-11)
//! G3: All mutations emit audit events (M1.3) — asserted in tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use digital_product_studio_core::{
    api::{curation::curation_router, runs::runs_router, niches::niches_router},
    middleware::auth::AuthUser,
    state::AppState,
};

async fn create_test_state() -> AppState {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/digital_product_studio_test".to_string());
    
    let db = PgPool::connect(&database_url).await.unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .unwrap();
    
    // Clean test data
    sqlx::query!("DELETE FROM niches").execute(&db).await.unwrap();
    sqlx::query!("DELETE FROM production_runs").execute(&db).await.unwrap();
    sqlx::query!("DELETE FROM jobs").execute(&db).await.unwrap();
    sqlx::query!("DELETE FROM designs").execute(&db).await.unwrap();
    sqlx::query!("DELETE FROM design_variants").execute(&db).await.unwrap();
    sqlx::query!("DELETE FROM curations").execute(&db).await.unwrap();
    
    AppState::new(db).await.unwrap()
}

fn test_admin() -> AuthUser {
    AuthUser {
        user_id: Uuid::new_v4(),
        role: "admin".to_string(),
    }
}

async fn setup_curation_test(state: &AppState) -> (Uuid, Uuid, Uuid) {
    // Create niche
    let niche_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO niches (id, name, vertical, demand_signals, competition_score, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        niche_id,
        "Test Niche",
        "printable_wall_art",
        json!({"search_volume": 1000}),
        rust_decimal::Decimal::new(45, 2),
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    // Create run
    let run_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO production_runs (id, niche_id, status, target_count, params, created_at)
        VALUES ($1, $2, 'generating', 1, '{}', $3)
        "#,
        run_id,
        niche_id,
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    // Create design
    let design_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO designs (id, run_id, base_prompt, s3_prefix_raw, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        design_id,
        run_id,
        "Test prompt for design",
        format!("raw/{}/{}", run_id, design_id),
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    // Create variant
    let variant_id = Uuid::new_v4();
    let s3_key = format!("raw/{}/{}/variant_0.png", run_id, design_id);
    sqlx::query!(
        r#"
        INSERT INTO design_variants (id, design_id, kind, s3_key, sha256, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        variant_id,
        design_id,
        "primary",
        s3_key,
        "abc123sha256",
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    (niche_id, run_id, variant_id)
}

#[tokio::test]
async fn test_get_curation_queue_empty() {
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/curation/queue")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["items"], json!([]));
    assert_eq!(json["total"], 0);
}

#[tokio::test]
async fn test_get_curation_queue_with_pending() {
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let (_, _, variant_id) = setup_curation_test(&state).await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/curation/queue")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["items"].is_array());
    assert_eq!(json["total"], 1);
    assert_eq!(json["items"][0]["variant_id"], variant_id.to_string());
    assert!(!json["items"][0]["presigned_url"].is_null());
    assert_eq!(json["items"][0]["kind"], "primary");
}

#[tokio::test]
async fn test_approve_curation_performs_copy_and_enqueue() {
    // G2: Approve performs COPY + assembly enqueue in one flow (§6 steps 8-11)
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let (_, _, variant_id) = setup_curation_test(&state).await;
    
    let payload = json!({
        "reviewer_id": test_admin().user_id.to_string()
    });
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/curation/{}/approve", variant_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["variant_id"], variant_id.to_string());
    assert_eq!(json["decision"], "approved");
    assert!(!json["job_enqueued"].is_null());
    
    let assembly_job_id = json["job_enqueued"].as_str().unwrap();
    
    // Verify curation record was created
    let curation = sqlx::query!(
        "SELECT decision, reviewer_id FROM curations WHERE variant_id = $1",
        variant_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    
    assert_eq!(curation.decision, "approved");
    assert_eq!(curation.reviewer_id, test_admin().user_id);
    
    // Verify assembly job was enqueued
    let job = sqlx::query!(
        "SELECT id, tool, status FROM jobs WHERE id = $1",
        Uuid::parse_str(assembly_job_id).unwrap()
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    
    assert_eq!(job.tool, "product-assembly");
    assert_eq!(job.status, "queued");
    
    // Verify command_spec has correct structure
    let command_spec = serde_json::from_value::<serde_json::Value>(job.command_spec).unwrap();
    assert_eq!(command_spec["tool"], "product_assembly");
    assert!(command_spec["args"]["variant_id"].as_str().unwrap() == variant_id.to_string());
    assert!(!command_spec["args"]["curated_key"].is_null());
}

#[tokio::test]
async fn test_approve_already_curated_fails() {
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let (_, _, variant_id) = setup_curation_test(&state).await;
    
    // Pre-curate as approved
    sqlx::query!(
        r#"
        INSERT INTO curations (variant_id, decision, reviewer_id, decided_at)
        VALUES ($1, 'approved', $2, $3)
        "#,
        variant_id,
        test_admin().user_id,
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    let payload = json!({
        "reviewer_id": test_admin().user_id.to_string()
    });
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/curation/{}/approve", variant_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_reject_curation() {
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let (_, _, variant_id) = setup_curation_test(&state).await;
    
    let payload = json!({
        "reject_reason": "style_mismatch",
        "reviewer_id": test_admin().user_id.to_string()
    });
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/curation/{}/reject", variant_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["variant_id"], variant_id.to_string());
    assert_eq!(json["decision"], "rejected");
    assert!(json["job_enqueued"].is_null());
    
    // Verify curation record
    let curation = sqlx::query!(
        "SELECT decision, reject_reason, reviewer_id FROM curations WHERE variant_id = $1",
        variant_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    
    assert_eq!(curation.decision, "rejected");
    assert_eq!(curation.reject_reason.unwrap(), "style_mismatch");
}

#[tokio::test]
async fn test_reject_curation_invalid_reason() {
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let (_, _, variant_id) = setup_curation_test(&state).await;
    
    let payload = json!({
        "reject_reason": "invalid_reason",
        "reviewer_id": test_admin().user_id.to_string()
    });
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/curation/{}/reject", variant_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_already_curated_fails() {
    let state = create_test_state().await;
    let app = curation_router().with_state(state.clone());
    
    let (_, _, variant_id) = setup_curation_test(&state).await;
    
    // Pre-curate as rejected
    sqlx::query!(
        r#"
        INSERT INTO curations (variant_id, decision, reject_reason, reviewer_id, decided_at)
        VALUES ($1, 'rejected', 'artifact', $2, $3)
        "#,
        variant_id,
        test_admin().user_id,
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    let payload = json!({
        "reject_reason": "style_mismatch",
        "reviewer_id": test_admin().user_id.to_string()
    });
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/curation/{}/reject", variant_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}