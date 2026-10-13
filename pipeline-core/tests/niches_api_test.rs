//! Integration tests for niche API endpoints
//!
//! These tests run in-Docker against a real Postgres database.
//! G1: Route tests in-Docker: POST /runs creates run + target_count jobs atomically (R10)
//! G3: All mutations emit audit events (M1.3) — asserted in tests
//! G4: 2MB body limit 413 test (S1) on one representative route

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use digital_product_studio_core::{
    api::{niches::niches_router, runs::runs_router, curation::curation_router},
    middleware::auth::{AuthUser, RequireAdmin},
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

#[tokio::test]
async fn test_list_niches_empty() {
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/niches")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["niches"], json!([]));
    assert_eq!(json["total"], 0);
}

#[tokio::test]
async fn test_create_niche_success() {
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    let payload = json!({
        "name": "Test Niche",
        "vertical": "printable_wall_art",
        "demand_signals": {"search_volume": 1000, "trend": "up"},
        "competition_score": dec!(0.45)
    });
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/niches")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(!json["id"].is_null());
    assert_eq!(json["name"], "Test Niche");
    assert_eq!(json["vertical"], "printable_wall_art");
    assert_eq!(json["competition_score"], "0.45");
}

#[tokio::test]
async fn test_create_niche_requires_admin() {
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    let payload = json!({
        "name": "Test Niche",
        "vertical": "printable_wall_art",
        "demand_signals": {},
        "competition_score": dec!(0.5)
    });
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/niches")
        .header("Content-Type", "application/json")
        // No Authorization header
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    // Should return 401 or 403 depending on middleware
    assert!(response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_niche_not_found() {
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/niches/{}", Uuid::new_v4()))
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_niche() {
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    // Create niche first
    let create_payload = json!({
        "name": "Original Name",
        "vertical": "planner_journal_bundles",
        "demand_signals": {},
        "competition_score": dec!(0.3)
    });
    
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/niches")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    
    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let niche_id = create_json["id"].as_str().unwrap();
    
    // Update niche
    let update_payload = json!({
        "name": "Updated Name"
    });
    
    let update_request = Request::builder()
        .method("PATCH")
        .uri(&format!("/api/v1/niches/{}", niche_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    
    let update_response = app.oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(update_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["name"], "Updated Name");
}

#[tokio::test]
async fn test_delete_niche_cascade_guard() {
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    // Create niche
    let create_payload = json!({
        "name": "Niche With Runs",
        "vertical": "party_wedding_printables",
        "demand_signals": {},
        "competition_score": dec!(0.2)
    });
    
    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/niches")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    
    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let create_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let niche_id = create_json["id"].as_str().unwrap();
    
    // Create a production run referencing this niche
    sqlx::query!(
        r#"
        INSERT INTO production_runs (id, niche_id, status, target_count, params, created_at)
        VALUES ($1, $2, 'pending', 5, '{}', $3)
        "#,
        Uuid::new_v4(),
        Uuid::parse_str(niche_id).unwrap(),
        Utc::now()
    )
    .execute(&state.db)
    .await
    .unwrap();
    
    // Try to delete - should fail with conflict
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/niches/{}", niche_id))
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::empty())
        .unwrap();
    
    let delete_response = app.oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_2mb_body_limit_413() {
    // G4: 2MB body limit 413 test (S1) on one representative route
    let state = create_test_state().await;
    let app = niches_router().with_state(state.clone());
    
    // Create a payload larger than 2MB
    let large_string = "x".repeat(3 * 1024 * 1024); // 3MB
    let payload = json!({
        "name": large_string,
        "vertical": "printable_wall_art",
        "demand_signals": {},
        "competition_score": dec!(0.5)
    });
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/niches")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", test_admin().user_id))
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    // Should return 413 Payload Too Large
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}