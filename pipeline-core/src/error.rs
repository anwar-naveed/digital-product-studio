//! Error types for pipeline-core API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// API error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("UUID parse error: {0}")]
    Uuid(#[from] uuid::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("No jobs available")]
    NoJobsAvailable,
}

/// Result type alias
pub type Result<T, E = AppError> = std::result::Result<T, E>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
                AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
                AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
                AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
                AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
                AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
                AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone()),
                AppError::Validation(e) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", e.to_string()),
                AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", e.to_string()),
                AppError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR", msg.clone()),
                AppError::Serialization(e) => (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR", e.to_string()),
                AppError::Uuid(e) => (StatusCode::BAD_REQUEST, "INVALID_UUID", e.to_string()),
                AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", msg.clone()),
                AppError::NoJobsAvailable => (StatusCode::NOT_FOUND, "NO_JOBS_AVAILABLE", "No jobs available in queue".to_string()),
            };
        
        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
                "correlation_id": uuid::Uuid::new_v4().to_string(),
            }
        }));
        
        (status, body).into_response()
    }
}
impl From<object_store::Error> for AppError {
    fn from(e: object_store::Error) -> Self {
        AppError::Storage(e.to_string())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        AppError::Validation(e.to_string())
    }
}

impl From<config::ConfigError> for AppError {
    fn from(e: config::ConfigError) -> Self {
        AppError::Internal(format!("Configuration error: {}", e))
    }
}
