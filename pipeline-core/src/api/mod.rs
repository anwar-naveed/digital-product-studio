//! API module for pipeline-core
//!
//! Exposes all REST API route handlers.

pub mod niches;
pub mod runs;
pub mod curation;
pub mod catalog;
pub mod listings;
pub mod analytics;
pub mod settings;

pub use niches::niches_router;
pub use runs::runs_router;
pub use curation::curation_router;
pub use catalog::catalog_router;
pub use listings::listings_router;
pub use analytics::analytics_router;
pub use settings::settings_router;

/// Combined API router for all v1 endpoints
pub fn api_router() -> axum::Router<crate::state::AppState> {
    axum::Router::new()
        .nest("/niches", niches_router())
        .nest("/runs", runs_router())
        .nest("/curation", curation_router())
        .nest("/catalog", catalog_router())
        .nest("/listings", listings_router())
        .nest("/analytics", analytics_router())
        .nest("/settings", settings_router())
}