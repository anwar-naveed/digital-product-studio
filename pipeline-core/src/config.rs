//! Configuration for pipeline-core

use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration loaded from environment and config files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub s3: S3Config,
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub audit: AuditConfig,
    pub tracing: TracingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_ssl: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub body_limit_mb: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: u64,
    pub issuer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditConfig {
    pub otlp_endpoint: String,
    pub service_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TracingConfig {
    pub level: String,
    pub otlp_endpoint: String,
}

impl AppConfig {
    /// Load configuration from environment and config files
    pub fn from_env() -> crate::error::Result<Self> {
        let mut builder = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::with_prefix("APP").separator("__"));
        
        // Required environment variables
        if let Ok(url) = std::env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", url)?;
        }
        if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
            builder = builder.set_override("s3.endpoint", endpoint)?;
        }
        if let Ok(bucket) = std::env::var("S3_BUCKET") {
            builder = builder.set_override("s3.bucket", bucket)?;
        }
        if let Ok(access_key) = std::env::var("S3_ACCESS_KEY") {
            builder = builder.set_override("s3.access_key", access_key)?;
        }
        if let Ok(secret_key) = std::env::var("S3_SECRET_KEY") {
            builder = builder.set_override("s3.secret_key", secret_key)?;
        }
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            builder = builder.set_override("jwt.secret", jwt_secret)?;
        }
        if let Ok(audit_endpoint) = std::env::var("OTLP_ENDPOINT") {
            builder = builder.set_override("audit.otlp_endpoint", audit_endpoint.clone())?;
            builder = builder.set_override("tracing.otlp_endpoint", audit_endpoint)?;
        }
        
        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }
}