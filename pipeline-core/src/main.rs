//! pipeline-core - Digital Product Studio API Server
//!
//! Main entry point for the pipeline-core REST API service.
//! Provides endpoints for niche management, production runs, and curation.

use axum::{
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
    cors::CorsLayer,
};
use tracing::{info, Level};

use pipeline_core::{
    api::api_router,
    state::AppState,
    config::AppConfig,
    error::AppError,
    audit::init_tracing,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let config = AppConfig::from_env()?;
    init_tracing(&config.tracing)?;
    
    info!("Starting pipeline-core server");
    
    // Create database pool
    let db = pipeline_core::db::create_pool(&config.database.url, config.database.max_connections).await?;
    
    // Run migrations
    pipeline_core::db::run_migrations(&db).await?;
    
    // Create application state
    let state = AppState::new(db).await?;
    
    // Build router with middleware
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(RequestBodyLimitLayer::new(config.server.body_limit_mb * 1024 * 1024))
        )
        .with_state(state);
    
    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("Server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}