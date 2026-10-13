//! Middleware for pipeline-core API

pub mod auth;

pub use auth::{AuthUser, Claims, RequireAdmin, RequireOperator};
