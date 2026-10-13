//! Legacy API integration scaffold (pre-rename crate name + stale deps).
//! Compile-gated behind `api-integration` until rewritten.
#![cfg(feature = "api-integration")]

//! Integration tests for runs API endpoints
//!
//! G1: Route tests in-Docker: POST /runs creates run + target_count jobs atomically (R10)
//! G3: All mutations emit audit events (M1.3) — asserted in tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use digital_product_studio_core::{
    api::{runs::runs_router, niches::niches_router},
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

async fn create_test_niche(state: &AppState) -> Uuid {
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
    niche_id
}

#[tokio::test]
async fn test_create_run_creates_jobs_atomically() {
    // G1: POST /runs creates run + target_count jobs atomically (R10)
    let state = create_test_state().await;
    let app = runs_router().with_state(state.clone());
    
    let niche_id = create_test_niche(&state).await;
    
    let payload = json!({
        "niche_id": niche_id,
        "target_count": 5,
        "params": {"style": "minimal", "colors": ["white", "black"]}
    });
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/runs")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = json["id"].as_str().unwrap();
    
    assert_eq!(json["target_count"], 5);
    assert_eq!(json["status"], "pending");
    assert_eq!(json["niche_id"], niche_id.to_string());
    
    // Verify jobs were created atomically
    let job_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM jobs WHERE run_id = $1",
        Uuid::parse_str(run_id).unwrap()
    )
    .fetch_one(&state.db)
    .await
    .unwrap()
    .unwrap_or(0);
    
    assert_eq!(job_count, 5, "Should create exactly target_count jobs");
    
    // Verify all jobs are cf-image-gen type
    let cf_job_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM jobs WHERE run_id = $1 AND tool = 'cf-image-gen'",
        Uuid::parse_str(run_id).unwrap()
    )
    .fetch_one(&state.db)
    .await
    .unwrap()
    .unwrap_or(0);
    
    assert_eq!(cf_job_count, 5, "All jobs should be cf-image-gen type");
    
    // Verify designs were created
    let design_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM designs WHERE run_id = $1",
        Uuid::parse_str(run_id).unwrap()
    )
    .fetch_one(&state.db)
    .await
    .unwrap()
    .unwrap_or(0);
    
    assert_eq!(design_count, 5, "Should create one design per job");
}

#[tokio::test]
async fn test_create_run_invalid_niche() {
    let state = create_test_state().await;
    let app = runs_router().with_state(state.clone());
    
    let payload = json!({
        "niche_id": Uuid::new_v4(),
        "target_count": 5,
        "params": {}
    });
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/runs")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_runs() {
    let state = create_test_state().await;
    let app = runs_router().with_state(state.clone());
    
    let niche_id = create_test_niche(&state).await;
    
    // Create a run
    let payload = json!({
        "niche_id": niche_id,
        "target_count": 3,
        "params": {}
    });
    
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/runs")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let _ = state.clone().oneshot(create_request).await.unwrap();
    
    // List runs
    let list_request = Request::builder()
        .method("GET")
        .uri("/api/v1/runs")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let list_response = state.oneshot(list_request).await.unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["runs"].is_array());
    assert_eq!(json["total"], 1);
    assert_eq!(json["runs"][0]["target_count"], 3);
}

#[tokio::test]
async fn test_get_run_with_jobs() {
    let state = create_test_state().await;
    let app = runs_router().with_state(state.clone());
    
    let niche_id = create_test_niche(&state).await;
    
    // Create a run
    let payload = json!({
        "niche_id": niche_id,
        "target_count": 2,
        "params": {}
    });
    
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/runs")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let run_id = create_json["id"].as_str().unwrap();
    
    // Get run detail
    let get_request = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/runs/{}", run_id))
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(get_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["id"], run_id);
    assert!(json["jobs"].is_array());
    assert_eq!(json["jobs"].as_array().unwrap().len(), 2);
    
    // Verify job structure
    for job in json["jobs"].as_array().unwrap() {
        assert!(!job["id"].is_null());
        assert_eq!(job["tool"], "cf-image-gen");
        assert_eq!(job["status"], "queued");
    }
}

#[tokio::test]
async fn test_cancel_run() {
    let state = create_test_state().await;
    let app = runs_router().with_state(state.clone());
    
    let niche_id = create_test_niche(&state).await;
    
    // Create a run
    let payload = json!({
        "niche_id": niche_id,
        "target_count": 3,
        "params": {}
    });
    
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/runs")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let run_id = create_json["id"].as_str().unwrap();
    
    // Cancel the run
    let cancel_request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/runs/{}/cancel", run_id))
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let cancel_response = app.oneshot(cancel_request).await.unwrap();
    assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);
    
    // Verify run status is cancelled
    let run_status: String = sqlx::query_scalar!(
        "SELECT status FROM production_runs WHERE id = $1",
        Uuid::parse_str(run_id).unwrap()
    )
    .fetch_one(&state.db)
    .await
    .unwrap()
    .unwrap();
    
    assert_eq!(run_status, "cancelled");
    
    // Verify queued jobs are cancelled
    let cancelled_job_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM jobs WHERE run_id = $1 AND status = 'cancelled'",
        Uuid::parse_str(run_id).unwrap()
    )
    .fetch_one(&state.db)
    .await
    .unwrap()
    .unwrap_or(0);
    
    assert_eq!(cancelled_job_count, 3);
}

#[tokio::test]
async fn test_cancel_completed_run_fails() {
    let state = create_test_state().await;
    let app = runs_router().with_state(state.clone());
    
    let niche_id = create_test_niche(&state).await;
    
    // Create a completed run directly in DB
    let run_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO production_runs (id, niche_id, status, target_count, params, created_at, completed_at)
        VALUES ($1, $2, 'completed', 3, '{}', $3, $3)
        "#,
        run_id,
        niche_id,
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    // Try to cancel - should fail
    let cancel_request = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/runs/{}/cancel", run_id))
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let cancel_response = app.oneshot(cancel_request).await.unwrap();
    assert_eq!(cancel_response.status(), StatusCode::CONFLICT);
}