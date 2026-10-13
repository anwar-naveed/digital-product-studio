//! Application state for pipeline-core
//!
//! Holds shared state across all request handlers including database pool,
//! S3 client, audit logger, and configuration.

use std::sync::Arc;
use sqlx::PgPool;
use object_store::ObjectStore;
use tracing::info;

use crate::{
    error::Result,
    s3::S3Client,
    audit::AuditLogger,
    config::AppConfig,
};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub s3: Arc<dyn ObjectStore>,
    pub s3_client: S3Client,
    pub audit: Arc<AuditLogger>,
    pub config: AppConfig,
}

impl AppState {
    /// Create a new application state
    pub async fn new(db: PgPool) -> Result<Self> {
        let config = AppConfig::from_env()?;
        
        // Initialize S3 client
        let s3_client = S3Client::new(
            &config.s3.endpoint,
            &config.s3.bucket,
            &config.s3.access_key,
            &config.s3.secret_key,
            &config.s3.region,
        )
        .await?;
        let s3: Arc<dyn ObjectStore> = s3_client.store_handle();
        
        // Initialize audit logger
        let audit = Arc::new(AuditLogger::new(&config.audit).await?);
        
        info!("Application state initialized");
        
        Ok(Self {
            db,
            s3,
            s3_client,
            audit,
            config,
        })
    }
}