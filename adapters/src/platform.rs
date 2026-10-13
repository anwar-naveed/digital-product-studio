//! Platform adapter trait and common types for digital product publishing.
//!
//! This module defines the PlatformAdapter trait per docs/11-architecture.md §4
//! and supporting types used across all platform adapters (Etsy, Lemon Squeezy, Printify, Gelato).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// HTTP header map type alias for webhook verification
pub type HeaderMap = HashMap<String, String>;

/// Platform identifier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Etsy,
    LemonSqueezy,
    Printify,
    Gelato,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Etsy => write!(f, "etsy"),
            Platform::LemonSqueezy => write!(f, "lemon-squeezy"),
            Platform::Printify => write!(f, "printify"),
            Platform::Gelato => write!(f, "gelato"),
        }
    }
}

/// Listing state machine states (per docs/11-architecture.md §2)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListingState {
    Draft,
    Publishing,
    Live,
    Updating,
    Failed,
    Delisting,
    Delisted,
    Suspended,
}

/// Order status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Paid,
    Fulfilled,
    Cancelled,
    Refunded,
}

/// Error returned when webhook verification fails
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationError {
    pub reason: String,
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Webhook verification failed: {}", self.reason)
    }
}

impl std::error::Error for VerificationError {}

/// Result of a successful webhook verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedWebhook {
    pub event_type: String,
    pub platform_event_id: String,
    pub payload: Value,
}

/// Receipt returned after publishing a listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub platform_listing_id: String,
    pub platform: String,
    pub state: ListingState,
    pub platform_raw: Value,
}

/// Patch structure for updating an existing listing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListingPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub tags: Option<Vec<String>>,
    pub state: Option<ListingState>,
}

/// File to be uploaded to a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFile {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// Platform-specific file identifier returned after upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFileId {
    pub id: String,
    pub url: Option<String>,
}

/// Order information fetched from platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub platform: String,
    pub status: OrderStatus,
    pub total: Decimal,
    pub items: Vec<OrderItem>,
    pub buyer_email: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

/// Individual item within an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub listing_id: String,
    pub quantity: u32,
    pub unit_price: Decimal,
    pub title: String,
}

/// Statistics for a listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingStats {
    pub views: u64,
    pub favorites: u64,
    pub sales: u64,
    pub revenue: Decimal,
}

/// Token bucket rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub rate_per_sec: f64,
    pub burst: usize,
}

/// Token bucket rate limiter (tower Layer)
#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: std::sync::Arc<tokio::sync::Mutex<TokenBucket>>,
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    /// Test-only accessor for (max_tokens, refill_rate) — used by adapter config tests.
    #[cfg(test)]
    pub(crate) fn test_config(&self) -> (f64, f64) {
        let b = self.inner.blocking_lock();
        (b.max_tokens, b.refill_rate)
    }


    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            inner: std::sync::Arc::new(tokio::sync::Mutex::new(TokenBucket {
                tokens: config.burst as f64,
                max_tokens: config.burst as f64,
                refill_rate: config.rate_per_sec,
                last_refill: std::time::Instant::now(),
            })),
        }
    }

    pub async fn acquire(&self) {
        let mut bucket = self.inner.lock().await;
        loop {
            bucket.refill();
            if bucket.tokens >= 1.0 {
                bucket.tokens -= 1.0;
                return;
            }
            // Wait until next token available
            let wait_secs = (1.0 - bucket.tokens) / bucket.refill_rate;
            drop(bucket);
            tokio::time::sleep(std::time::Duration::from_secs_f64(wait_secs.max(0.001))).await;
            bucket = self.inner.lock().await;
        }
    }
}

impl TokenBucket {
    fn refill(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

/// Retry policy for HTTP requests
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(30),
            jitter: true,
        }
    }
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> std::time::Duration {
        let exp_delay = self.base_delay * 2_u32.pow(attempt);
        let delay = exp_delay.min(self.max_delay);
        if self.jitter {
            let jitter_range = delay.as_millis() as u64 / 4;
            let jitter = rand::random::<u64>() % jitter_range;
            delay + std::time::Duration::from_millis(jitter)
        } else {
            delay
        }
    }

    pub fn should_retry(&self, attempt: u32, status: u16) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        matches!(status, 429 | 500..=599)
    }
}

/// Product structure (simplified for adapter interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub tags: Vec<String>,
    pub files: Vec<UploadFile>,
}

/// PlatformAdapter trait - must be implemented by each platform adapter
///
/// Per docs/11-architecture.md §4:
/// - 6 async methods for platform operations
/// - 1 sync method for webhook verification
/// - 1 method returning rate limiter
#[async_trait]
pub trait PlatformAdapter: Send + Sync + 'static {
    /// Publish a new listing to the platform
    /// 
    /// Etsy: POST /v3/application/listings with listing_files (docs/09 §2.1)
    async fn publish_listing(&self, product: &Product, files: &[UploadFile])
        -> Result<PublishReceipt, AdapterError>;

    /// Update an existing listing on the platform
    ///
    /// Etsy: PUT /v3/application/listings/{listing_id} (docs/09 §2.1)
    async fn update_listing(&self, listing_id: &str, patch: ListingPatch)
        -> Result<(), AdapterError>;

    /// Upload a digital file to the platform
    ///
    /// Etsy: POST /v3/application/shops/{shop_id}/files → returns file_id (docs/09 §2.1)
    async fn upload_digital_file(&self, listing_id: &str, file: UploadFile)
        -> Result<PlatformFileId, AdapterError>;

    /// Fetch orders from the platform since a given timestamp
    ///
    /// Etsy: GET /v3/application/shops/{shop_id}/receipts (docs/09 §2.1)
    async fn fetch_orders(&self, since: DateTime<Utc>)
        -> Result<Vec<Order>, AdapterError>;

    /// Fetch statistics for a listing
    ///
    /// Etsy: GET /v3/application/listings/{listing_id}/stats (docs/09 §2.1)
    async fn fetch_stats(&self, listing_id: &str)
        -> Result<ListingStats, AdapterError>;

    /// Verify webhook signature using constant-time comparison
    ///
    /// Runs BEFORE any business logic or DB persistence (M6.1)
    /// Uses HMAC-SHA256 with platform-specific header (M5.2)
    fn verify_webhook(&self, body: &[u8], headers: &HeaderMap)
        -> Result<VerifiedWebhook, VerificationError>;

    /// Returns the rate limiter for this platform
    fn rate_limiter(&self) -> &RateLimiter;
}

/// Adapter error type
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Platform API error: {0}")]
    PlatformApi(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Webhook verification failed: {0}")]
    WebhookVerification(#[from] VerificationError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl AdapterError {
    /// Best-effort HTTP-ish status code for retry-policy decisions.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            AdapterError::Auth(_) => Some(401),
            AdapterError::RateLimited(_) => Some(429),
            AdapterError::InvalidRequest(_) => Some(400),
            AdapterError::WebhookVerification(_) => Some(400),
            AdapterError::Http(e) => e.status().map(|s| s.as_u16()),
            _ => None,
        }
    }
}

/// Shared HTTP client builder for all adapters
/// Uses reqwest with rustls ≥0.23.45 (per docs/11-architecture.md §8 S4/S5)
pub fn build_shared_client() -> reqwest::Client {
    reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client")
}

