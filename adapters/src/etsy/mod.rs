//! Etsy platform adapter implementation.
//!
//! Implements PlatformAdapter trait for Etsy Open API v3.
//! 
//! ## Official Documentation References
//! - Authentication: https://developer.etsy.com/documentation/essentials/authentication
//! - Listings/Upload: https://developers.etsy.com/documentation/tutorials/listings
//! - Webhooks: https://developer.etsy.com/documentation/reference (webhooks section)
//! - API Reference: https://developer.etsy.com/documentation/reference

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use ring::hmac;
use subtle::ConstantTimeEq;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};

use crate::platform::{
    PlatformAdapter, Platform, ListingState, OrderStatus, Order, OrderItem, ListingStats,
    PublishReceipt, ListingPatch, UploadFile, PlatformFileId, VerifiedWebhook, VerificationError,
    RateLimiter, RateLimiterConfig, RetryPolicy, AdapterError, Product, HeaderMap as PlatformHeaderMap,
    build_shared_client,
};

/// Etsy-specific credentials structure
/// Per docs/11-architecture.md §4 and 6.3-per-adapter-designs.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtsyCredentials {
    /// Application keystring from Etsy Developer Dashboard
    pub keystring: String,
    /// OAuth2 access token (1 hour expiry)
    pub access_token: String,
    /// OAuth2 refresh token (90 day expiry)
    pub refresh_token: String,
    /// Token expiry timestamp
    pub expires_at: DateTime<Utc>,
    /// Etsy shop ID
    pub shop_id: String,
    /// Webhook signature secret
    pub webhook_secret: String,
}

impl EtsyCredentials {
    /// Check if token needs proactive refresh (≥5 min before expiry per M5.6)
    pub fn needs_refresh(&self) -> bool {
        let now = Utc::now();
        let five_min = Duration::minutes(5);
        self.expires_at - now < five_min
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

/// Etsy OAuth2 token response
#[derive(Debug, Deserialize)]
struct EtsyTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64, // seconds
    token_type: String,
    scope: String,
}

/// Etsy file upload response
#[derive(Debug, Deserialize)]
struct EtsyFileUploadResponse {
    file_id: String,
    #[serde(rename = "file")]
    file_info: Option<EtsyFileInfo>,
}

#[derive(Debug, Deserialize)]
struct EtsyFileInfo {
    id: String,
    name: String,
    size: u64,
    mime_type: String,
}

/// Etsy listing creation response
#[derive(Debug, Deserialize)]
struct EtsyListingResponse {
    listing_id: i64,
    state: String,
    #[serde(flatten)]
    extra: Value,
}

/// Etsy listing update response
#[derive(Debug, Deserialize)]
struct EtsyListingUpdateResponse {
    listing_id: i64,
    state: String,
}

/// Etsy order/receipt response
#[derive(Debug, Deserialize)]
struct EtsyReceiptsResponse {
    results: Vec<EtsyReceipt>,
    count: u32,
}

#[derive(Debug, Deserialize)]
struct EtsyReceipt {
    receipt_id: String,
    buyer_email: String,
    created_timestamp: i64,
    paid_timestamp: Option<i64>,
    total_price: EtsyMoney,
    transactions: Vec<EtsyTransaction>,
}

#[derive(Debug, Deserialize)]
struct EtsyMoney {
    amount: i64,
    divisor: i64,
    currency_code: String,
}

#[derive(Debug, Deserialize)]
struct EtsyTransaction {
    transaction_id: i64,
    listing_id: i64,
    title: String,
    quantity: u32,
    price: EtsyMoney,
}

/// Etsy listing stats response
#[derive(Debug, Deserialize)]
struct EtsyListingStatsResponse {
    views: u64,
    num_favorers: u64,
    // sales not directly available, compute from receipts
}

/// Etsy webhook payload structure
#[derive(Debug, Deserialize)]
struct EtsyWebhookPayload {
    event: String,
    id: String,
    data: Value,
    timestamp: i64,
}

/// Etsy adapter implementation
pub struct EtsyAdapter {
    credentials: Arc<RwLock<EtsyCredentials>>,
    client: Client,
    rate_limiter: RateLimiter,
    retry_policy: RetryPolicy,
    base_url: String,
}

impl EtsyAdapter {
    /// Create a new Etsy adapter
    pub fn new(credentials: EtsyCredentials) -> Self {
        let rate_limiter = RateLimiter::new(RateLimiterConfig {
            rate_per_sec: 5.0,  // 5 req/s sustained (well under Etsy 1000/hr per 6.3-per-adapter-designs.md)
            burst: 10,          // 10 burst
        });
        
        Self {
            credentials: Arc::new(RwLock::new(credentials)),
            client: build_shared_client(),
            rate_limiter,
            retry_policy: RetryPolicy::default(),
            base_url: "https://api.etsy.com/v3".to_string(),
        }
    }

    /// Get current credentials (read lock)
    async fn get_credentials(&self) -> tokio::sync::RwLockReadGuard<'_, EtsyCredentials> {
        self.credentials.read().await
    }

    /// Get mutable credentials (write lock)
    async fn get_credentials_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, EtsyCredentials> {
        self.credentials.write().await
    }

    /// Ensure valid access token (proactive refresh per M5.6)
    async fn ensure_valid_token(&self) -> Result<String, AdapterError> {
        let mut creds = self.get_credentials_mut().await;
        
        if creds.needs_refresh() || creds.is_expired() {
            info!("Proactively refreshing Etsy OAuth2 token (M5.6: ≥5 min before expiry)");
            self.refresh_token(&mut creds).await?;
        }
        
        Ok(creds.access_token.clone())
    }

    /// Refresh OAuth2 access token using refresh token
    /// 
    /// POST https://api.etsy.com/v3/public/oauth/token
    /// Per: https://developer.etsy.com/documentation/essentials/authentication
    #[instrument(skip(self, creds))]
    async fn refresh_token(&self, creds: &mut EtsyCredentials) -> Result<(), AdapterError> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &creds.refresh_token),
            ("client_id", &creds.keystring),
        ];

        let response = self.client
            .post("https://api.etsy.com/v3/public/oauth/token")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Etsy token refresh failed: {} - {}", status, body);
            return Err(AdapterError::Auth(format!("Token refresh failed: {} - {}", status, body)));
        }

        let token_response: EtsyTokenResponse = response.json().await?;
        
        creds.access_token = token_response.access_token;
        creds.refresh_token = token_response.refresh_token;
        creds.expires_at = Utc::now() + Duration::seconds(token_response.expires_in);
        
        info!("Etsy OAuth2 token refreshed successfully, expires at: {}", creds.expires_at);
        Ok(())
    }

    /// Execute request with rate limiting and retry logic
    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T, AdapterError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, AdapterError>>,
    {
        let mut attempt = 0;
        
        loop {
            // Acquire rate limiter token
            self.rate_limiter.acquire().await;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if we should retry
                    let should_retry = match &e {
                        AdapterError::RateLimited(_) => true,
                        AdapterError::Http(req_err) => {
                            req_err.status().map(|s| self.retry_policy.should_retry(attempt, s.as_u16())).unwrap_or(false)
                        }
                        AdapterError::PlatformApi(_) => {
                            // Check if it's a 5xx error
                            false // We'd need to parse the error for status code
                        }
                        _ => false,
                    };
                    
                    if should_retry && attempt < self.retry_policy.max_attempts {
                        let delay = self.retry_policy.delay_for_attempt(attempt);
                        warn!("Etsy API request failed (attempt {}): {}. Retrying in {:?}", attempt + 1, e, delay);
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }
                    
                    return Err(e);
                }
            }
        }
    }

    /// Build authorization header
    fn auth_header(&self, token: &str) -> HeaderValue {
        HeaderValue::from_str(&format!("Bearer {}", token)).expect("Valid header value")
    }

    /// Convert Etsy state to ListingState
    pub fn etsy_state_to_listing_state(state: &str) -> ListingState {
        match state.to_lowercase().as_str() {
            "active" | "published" => ListingState::Live,
            "draft" => ListingState::Draft,
            "inactive" | "expired" => ListingState::Delisted,
            _ => ListingState::Failed,
        }
    }

    /// Convert Etsy money to Decimal
    fn etsy_money_to_decimal(money: &EtsyMoney) -> rust_decimal::Decimal {
        rust_decimal::Decimal::new(money.amount, money.divisor as u32)
    }
}

#[async_trait]
impl PlatformAdapter for EtsyAdapter {
    /// Publish a new listing to Etsy
    ///
    /// Etsy API: POST /v3/application/shops/{shop_id}/listings
    /// Documentation: https://developers.etsy.com/documentation/tutorials/listings
    /// 
    /// Flow per docs/09 §2.1:
    /// 1. Upload file(s) via POST /v3/application/shops/{shop_id}/files → returns file_id
    /// 2. Create listing via POST /v3/application/listings with listing_files: [{file_id}]
    #[instrument(skip(self, product, files))]
    async fn publish_listing(&self, product: &Product, files: &[UploadFile])
        -> Result<PublishReceipt, AdapterError>
    {
        let access_token = self.ensure_valid_token().await?;
        let creds = self.get_credentials().await;
        let shop_id = &creds.shop_id;

        // Step 1: Upload digital files
        // POST /v3/application/shops/{shop_id}/files
        // Doc: https://developers.etsy.com/documentation/tutorials/listings
        let mut file_ids = Vec::new();
        for file in files {
            let file_id = self.upload_digital_file_internal(shop_id, file, &access_token).await?;
            file_ids.push(file_id);
        }

        // Step 2: Create listing with file associations
        // POST /v3/application/listings
        // Doc: https://developer.etsy.com/documentation/tutorials/listings
        let listing_payload = json!({
            "title": product.title,
            "description": product.description,
            "price": product.price.to_string(),
            "quantity": 999, // Digital listings typically use high quantity
            "listing_files": file_ids.iter().map(|id| json!({"file_id": id})).collect::<Vec<_>>(),
            "tags": product.tags,
            "is_digital": true,
            "digital_file_ids": file_ids,
        });

        let response = self.execute_with_retry(|| async {
            self.client
                .post(&format!("{}/application/shops/{}/listings", self.base_url, shop_id))
                .header(AUTHORIZATION, self.auth_header(&access_token))
                .header(CONTENT_TYPE, "application/json")
                .json(&listing_payload)
                .send()
                .await
                .map_err(AdapterError::from)
        }).await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(AdapterError::RateLimited(body));
            }
            return Err(AdapterError::PlatformApi(format!("Failed to create listing: {} - {}", status, body)));
        }

        let listing_resp: EtsyListingResponse = response.json().await?;
        let platform_listing_id = listing_resp.listing_id.to_string();

        info!("Etsy listing created: {}", platform_listing_id);

        Ok(PublishReceipt {
            platform_listing_id,
            platform: "etsy".to_string(),
            state: Self::etsy_state_to_listing_state(&listing_resp.state),
            platform_raw: listing_resp.extra,
        })
    }

    /// Update an existing listing on Etsy
    ///
    /// Etsy API: PUT /v3/application/listings/{listing_id}
    /// Documentation: https://developers.etsy.com/documentation/tutorials/listings
    #[instrument(skip(self, patch))]
    async fn update_listing(&self, listing_id: &str, patch: ListingPatch)
        -> Result<(), AdapterError>
    {
        let access_token = self.ensure_valid_token().await?;
        let creds = self.get_credentials().await;
        let shop_id = &creds.shop_id;

        let mut payload = json!({});
        
        if let Some(title) = patch.title {
            payload["title"] = json!(title);
        }
        if let Some(description) = patch.description {
            payload["description"] = json!(description);
        }
        if let Some(price) = patch.price {
            payload["price"] = json!(price.to_string());
        }
        if let Some(tags) = patch.tags {
            payload["tags"] = json!(tags);
        }
        if let Some(state) = patch.state {
            payload["state"] = json!(match state {
                ListingState::Live => "active",
                ListingState::Draft => "draft",
                ListingState::Delisted => "inactive",
                _ => "active",
            });
        }

        self.execute_with_retry(|| async {
            self.client
                .put(&format!("{}/application/shops/{}/listings/{}", self.base_url, shop_id, listing_id))
                .header(AUTHORIZATION, self.auth_header(&access_token))
                .header(CONTENT_TYPE, "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(AdapterError::from)
        }).await?;

        info!("Etsy listing updated: {}", listing_id);
        Ok(())
    }

    /// Upload a digital file to Etsy
    ///
    /// Etsy API: POST /v3/application/shops/{shop_id}/files
    /// Documentation: https://developers.etsy.com/documentation/tutorials/listings
    /// 
    /// Returns file_id that can be used in listing creation
    #[instrument(skip(self, file))]
    async fn upload_digital_file(&self, listing_id: &str, file: UploadFile)
        -> Result<PlatformFileId, AdapterError>
    {
        let access_token = self.ensure_valid_token().await?;
        let creds = self.get_credentials().await;
        let shop_id = &creds.shop_id;
        
        let file_id = self.upload_digital_file_internal(shop_id, &file, &access_token).await?;
        
        Ok(PlatformFileId {
            id: file_id.clone(),
            url: Some(format!("https://api.etsy.com/v3/application/shops/{}/files/{}", shop_id, file_id)),
        })
    }


    /// Fetch orders from Etsy since a given timestamp
    ///
    /// Etsy API: GET /v3/application/shops/{shop_id}/receipts
    /// Documentation: https://developer.etsy.com/documentation/reference
    #[instrument(skip(self))]
    async fn fetch_orders(&self, since: DateTime<Utc>) -> Result<Vec<Order>, AdapterError> {
        let access_token = self.ensure_valid_token().await?;
        let creds = self.get_credentials().await;
        let shop_id = &creds.shop_id;

        let since_timestamp = since.timestamp();

        let response = self.execute_with_retry(|| async {
            self.client
                .get(&format!("{}/application/shops/{}/receipts", self.base_url, shop_id))
                .header(AUTHORIZATION, self.auth_header(&access_token))
                .query(&[
                    ("was_paid", "true"),
                    ("min_created_timestamp", &since_timestamp.to_string()),
                    ("limit", "100"),
                ])
                .send()
                .await
                .map_err(AdapterError::from)
        }).await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(AdapterError::RateLimited(body));
            }
            return Err(AdapterError::PlatformApi(format!("Failed to fetch orders: {} - {}", status, body)));
        }

        let receipts_resp: EtsyReceiptsResponse = response.json().await?;
        
        let mut orders = Vec::new();
        for receipt in receipts_resp.results {
            let occurred_at = DateTime::from_timestamp(receipt.paid_timestamp.unwrap_or(receipt.created_timestamp), 0)
                .unwrap_or_else(|| Utc::now());

            let mut items = Vec::new();
            for txn in receipt.transactions {
                items.push(OrderItem {
                    listing_id: txn.listing_id.to_string(),
                    quantity: txn.quantity,
                    unit_price: Self::etsy_money_to_decimal(&txn.price),
                    title: txn.title,
                });
            }

            orders.push(Order {
                id: receipt.receipt_id,
                platform: "etsy".to_string(),
                status: if receipt.paid_timestamp.is_some() { OrderStatus::Paid } else { OrderStatus::Pending },
                total: Self::etsy_money_to_decimal(&receipt.total_price),
                items,
                buyer_email: Some(receipt.buyer_email),
                occurred_at,
            });
        }

        info!("Fetched {} Etsy orders since {}", orders.len(), since);
        Ok(orders)
    }

    /// Fetch statistics for a listing
    ///
    /// Etsy API: GET /v3/application/listings/{listing_id}/stats
    /// Documentation: https://developer.etsy.com/documentation/reference
    #[instrument(skip(self))]
    async fn fetch_stats(&self, listing_id: &str) -> Result<ListingStats, AdapterError> {
        let access_token = self.ensure_valid_token().await?;

        let response = self.execute_with_retry(|| async {
            self.client
                .get(&format!("{}/application/listings/{}/stats", self.base_url, listing_id))
                .header(AUTHORIZATION, self.auth_header(&access_token))
                .send()
                .await
                .map_err(AdapterError::from)
        }).await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(AdapterError::RateLimited(body));
            }
            return Err(AdapterError::PlatformApi(format!("Failed to fetch stats: {} - {}", status, body)));
        }

        let stats_resp: EtsyListingStatsResponse = response.json().await?;

        // Note: Sales data would typically come from receipts; this is a simplified version
        Ok(ListingStats {
            views: stats_resp.views,
            favorites: stats_resp.num_favorers,
            sales: 0, // Would need separate receipts query
            revenue: rust_decimal::Decimal::ZERO,
        })
    }

    /// Verify Etsy webhook signature using constant-time comparison
    ///
    /// Etsy uses HMAC-SHA256 with the webhook secret
    /// Signature header: X-Etsy-Signature
    /// 
    /// Per M5.2: Constant-time comparison using subtle::ConstantTimeEq
    /// Per M6.1: Runs BEFORE any business logic or DB persistence
    /// Per docs/11-architecture.md §4 and 6.3-webhook-ingestion-spec.md
    fn verify_webhook(&self, body: &[u8], headers: &PlatformHeaderMap) 
        -> Result<VerifiedWebhook, VerificationError>
    {
        // Extract signature header (X-Etsy-Signature)
        let sig_header = "x-etsy-signature";
        let provided_sig = headers
            .get(sig_header)
            .ok_or_else(|| VerificationError { reason: "Missing X-Etsy-Signature header".to_string() })?;

        // Compute expected HMAC-SHA256
        let creds = self.credentials.blocking_read();
        let key = hmac::Key::new(hmac::HMAC_SHA256, creds.webhook_secret.as_bytes());
        let expected_sig = hmac::sign(&key, body);
        let expected_hex = hex::encode(expected_sig.as_ref());

        // Constant-time comparison (M5.2) - NEVER use ==
        if !bool::from(provided_sig.as_bytes().ct_eq(expected_hex.as_bytes())) {
            warn!("Etsy webhook signature verification failed");
            return Err(VerificationError { reason: "Invalid signature".to_string() });
        }

        // Parse and validate payload
        let payload: Value = serde_json::from_slice(body)
            .map_err(|_| VerificationError { reason: "Invalid JSON payload".to_string() })?;

        // Extract platform_event_id and event_type from Etsy webhook format
        let webhook_payload: EtsyWebhookPayload = serde_json::from_value(payload.clone())
            .map_err(|_| VerificationError { reason: "Invalid webhook payload structure".to_string() })?;

        Ok(VerifiedWebhook {
            event_type: webhook_payload.event,
            platform_event_id: webhook_payload.id,
            payload,
        })
    }

    /// Return the rate limiter for Etsy
    fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
}

impl EtsyAdapter {
    /// Internal file upload helper
    async fn upload_digital_file_internal(&self, shop_id: &str, file: &UploadFile, access_token: &str)
        -> Result<String, AdapterError>
    {
        // Validate file size (≤250 MB per docs/09 §2.1)
        const MAX_FILE_SIZE: usize = 250 * 1024 * 1024;
        if file.bytes.len() > MAX_FILE_SIZE {
            return Err(AdapterError::InvalidRequest(format!(
                "File size {} bytes exceeds 250 MB limit", file.bytes.len()
            )));
        }

        // Validate file type
        let allowed_types = [
            "image/jpeg", "image/png", "image/gif",
            "application/pdf",
            "video/mp4", "video/quicktime",
            "audio/mpeg", "audio/wav",
            "application/msword",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/vnd.ms-excel",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.ms-powerpoint",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "application/vnd.oasis.opendocument.text",
            "application/vnd.apple.keynote",
            "application/vnd.apple.pages",
        ];
        
        if !allowed_types.contains(&file.content_type.as_str()) {
            return Err(AdapterError::InvalidRequest(format!(
                "Unsupported file type: {}", file.content_type
            )));
        }

        let response = self.execute_with_retry(|| async {
            let form = reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file.bytes.clone())
                    .file_name(file.filename.clone())
                    .mime_str(&file.content_type).unwrap());

            self.client
                .post(&format!("{}/application/shops/{}/files", self.base_url, shop_id))
                .header(AUTHORIZATION, self.auth_header(access_token))
                .multipart(form)
                .send()
                .await
                .map_err(AdapterError::from)
        }).await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(AdapterError::RateLimited(body));
            }
            if status.as_u16() == 413 {
                return Err(AdapterError::InvalidRequest("File too large (413)".to_string()));
            }
            return Err(AdapterError::PlatformApi(format!("File upload failed: {} - {}", status, body)));
        }

        let upload_resp: EtsyFileUploadResponse = response.json().await?;
        
        info!("Etsy file uploaded: {}", upload_resp.file_id);
        Ok(upload_resp.file_id)
    }
}


/// Background token refresh worker
/// 
/// Per M5.6: Proactive refresh ≥5 min before expiry
/// Runs as a background task checking token expiry
pub struct EtsyTokenRefresher {
    adapter: Arc<EtsyAdapter>,
    interval: tokio::time::Interval,
}

impl EtsyTokenRefresher {
    pub fn new(adapter: Arc<EtsyAdapter>) -> Self {
        // Check every minute
        Self {
            adapter,
            interval: tokio::time::interval(std::time::Duration::from_secs(60)),
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.interval.tick().await;
            
            let creds = self.adapter.get_credentials().await;
            if creds.needs_refresh() {
                info!("EtsyTokenRefresher: Token needs refresh, triggering proactive refresh");
                drop(creds); // Release read lock before write
                
                if let Err(e) = self.adapter.ensure_valid_token().await {
                    error!("EtsyTokenRefresher: Failed to refresh token: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_etsy_credentials_needs_refresh() {
        let mut creds = EtsyCredentials {
            keystring: "test".to_string(),
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: Utc::now() + Duration::minutes(10),
            shop_id: "123".to_string(),
            webhook_secret: "secret".to_string(),
        };
        
        // Should not need refresh at 10 min
        assert!(!creds.needs_refresh());
        
        // Should need refresh at 4 min
        creds.expires_at = Utc::now() + Duration::minutes(4);
        assert!(creds.needs_refresh());
    }

    #[test]
    fn test_etsy_state_conversion() {
        assert_eq!(EtsyAdapter::etsy_state_to_listing_state("active"), ListingState::Live);
        assert_eq!(EtsyAdapter::etsy_state_to_listing_state("draft"), ListingState::Draft);
        assert_eq!(EtsyAdapter::etsy_state_to_listing_state("inactive"), ListingState::Delisted);
    }
}