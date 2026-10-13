//! Gelato Platform Adapter
//!
//! Implements the PlatformAdapter trait for Gelato (POD network with global production).
//!
//! References:
//! - docs/11-architecture.md §4 (adapter trait)
//! - docs/09-requirements.md §2.4 (Gelato API)
//! - docs/11-architecture.md §8 (reqwest + rustls ≥0.23.45)
//!
//! Auth: API key (X-API-KEY); some flows OAuth2
//! Rate limit: Conservative 4 req/s + jittered backoff (UNVERIFIED public cap)
//! Webhook: X-Gelato-Signature HMAC-SHA256

use async_trait::async_trait;
use base64;
use chrono::{DateTime, Utc};
use hex;
use rand::Rng;
use ring::hmac;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use subtle::ConstantTimeEq;
use tokio::sync::Mutex;
use uuid;

use crate::platform::{
    AdapterError, ListingPatch, ListingState, ListingStats, Order, OrderItem, OrderStatus, Platform,
    PlatformAdapter, PlatformFileId, Product, PublishReceipt, RateLimiter, RateLimiterConfig,
    RetryPolicy, UploadFile, VerifiedWebhook, VerificationError, build_shared_client,
};

/// Gelato credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GelatoCredentials {
    pub api_key: String,
    pub webhook_secret: String,
    // Optional OAuth2 fields if flow used
    pub oauth2_access_token: Option<String>,
    pub oauth2_refresh_token: Option<String>,
    pub oauth2_expires_at: Option<DateTime<Utc>>,
}

/// Gelato product creation request
#[derive(Debug, Clone, Serialize)]
struct GelatoProductRequest {
    product_type: String,
    name: String,
    description: String,
    files: Vec<GelatoFileRequest>,
    variants: Vec<GelatoVariantRequest>,
    tags: Vec<String>,
}

/// Gelato file request
#[derive(Debug, Clone, Serialize)]
struct GelatoFileRequest {
    r#type: String, // "design", "mockup", etc.
    url: String,
}

/// Gelato variant request
#[derive(Debug, Clone, Serialize)]
struct GelatoVariantRequest {
    id: String,
    options: HashMap<String, String>,
    price: i32, // Price in cents
}

/// Gelato product response
#[derive(Serialize, Debug, Clone, Deserialize)]
struct GelatoProductResponse {
    id: String,
    name: String,
    status: String,
}

/// Gelato order response
#[derive(Debug, Clone, Deserialize)]
struct GelatoOrderResponse {
    id: String,
    status: String,
    total: GelatoPrice,
    items: Vec<GelatoOrderItem>,
    created_at: String,
}

/// Gelato price
#[derive(Debug, Clone, Deserialize)]
struct GelatoPrice {
    amount: i32,
    currency: String,
}

/// Gelato order item
#[derive(Debug, Clone, Deserialize)]
struct GelatoOrderItem {
    product_id: String,
    variant_id: String,
    quantity: i32,
    price: GelatoPrice,
    name: String,
}

/// Gelato adapter implementation
pub struct GelatoAdapter {
    client: Client,
    credentials: GelatoCredentials,
    rate_limiter: RateLimiter,
    retry_policy: RetryPolicy,
    base_url: String,
}

impl GelatoAdapter {
    /// Create a new GelatoAdapter
    ///
    /// # Arguments
    /// * `credentials` - Gelato API credentials
    ///
    /// # References
    /// - docs/09-requirements.md §2.4: "Exceeding daily limit → no response; exceeding max rate → 429"
    /// - docs/11-architecture.md §4: Conservative 4 req/s + jittered backoff
    pub fn new(credentials: GelatoCredentials) -> Self {
        // Rate limiter: Conservative 4 req/s with burst 10
        // Per docs/09 §2.4: "implement retry with exponential backoff and jitter"
        let rate_limiter = RateLimiter::new(RateLimiterConfig {
            rate_per_sec: 4.0,
            burst: 10,
        });

        let retry_policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(60),
            jitter: true, // Jittered backoff as required by Gelato
        };

        Self {
            client: build_shared_client(),
            credentials,
            rate_limiter,
            retry_policy,
            base_url: "https://api.gelato.com/v1".to_string(),
        }
    }

    /// Get the authorization header value
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.credentials.api_key)
    }

    /// Get X-API-KEY header value
    fn api_key_header(&self) -> String {
        self.credentials.api_key.clone()
    }

    /// Execute a request with rate limiting and jittered retry logic
    async fn execute_with_retry<F, Fut, T>(&self, mut f: F) -> Result<T, AdapterError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, AdapterError>>,
    {
        let mut attempt = 0;
        loop {
            // Acquire rate limiter token
            self.rate_limiter.acquire().await;

            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if !self.retry_policy.should_retry(attempt, e.status_code().unwrap_or(0)) {
                        return Err(e);
                    }
                    let delay = self.retry_policy.delay_for_attempt(attempt);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Build product creation request from our Product type
    fn build_product_request(&self, product: &Product, files: &[UploadFile]) -> GelatoProductRequest {
        // Map files to Gelato format
        let gelato_files: Vec<GelatoFileRequest> = files
            .iter()
            .map(|f| GelatoFileRequest {
                r#type: "design".to_string(),
                url: format!("data:{};base64,{}", f.content_type, base64::encode(&f.bytes)),
            })
            .collect();

        // Create variants - for Gelato we need size/material variants
        let variants = vec![GelatoVariantRequest {
            id: "default".to_string(),
            options: HashMap::from([
                ("size".to_string(), "A4".to_string()),
                ("material".to_string(), "paper".to_string()),
            ]),
            price: (product.price * rust_decimal::Decimal::new(100, 0))
                .to_string()
                .parse::<i32>()
                .unwrap_or(1000),
        }];

        GelatoProductRequest {
            product_type: "poster".to_string(), // Default to poster
            name: product.title.clone(),
            description: product.description.clone(),
            files: gelato_files,
            variants,
            tags: product.tags.clone(),
        }
    }
}

#[async_trait]
impl PlatformAdapter for GelatoAdapter {
    /// Publish a new listing to Gelato
    ///
    /// References:
    /// - docs/09-requirements.md §2.4: POST /v1/products with design file
    /// - docs/11-architecture.md §4: publish_listing signature
    async fn publish_listing(
        &self,
        product: &Product,
        files: &[UploadFile],
    ) -> Result<PublishReceipt, AdapterError> {
        let request = self.build_product_request(product, files);

        let response = self
            .execute_with_retry(|| async {
                let resp = self
                    .client
                    .post(&format!("{}/products", self.base_url))
                    .header("X-API-KEY", self.api_key_header())
                    .header("Authorization", self.auth_header())
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .send()
                    .await
                    .map_err(AdapterError::Http)?;

                let status = resp.status().as_u16();
                if status == 429 {
                    return Err(AdapterError::RateLimited("Gelato rate limit exceeded".to_string()));
                }
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(AdapterError::PlatformApi(format!(
                        "Gelato API error {}: {}",
                        status, text
                    )));
                }

                let product_resp: GelatoProductResponse = resp
                    .json()
                    .await
                    .map_err(AdapterError::Http)?;

                Ok(product_resp)
            })
            .await?;

        Ok(PublishReceipt {
            platform_raw: serde_json::to_value(&response).map_err(AdapterError::Serialization)?,
            platform_listing_id: response.id,
            platform: "gelato".to_string(),
            state: ListingState::Live,
        })
    }

    /// Update an existing listing on Gelato
    ///
    /// References:
    /// - docs/09-requirements.md §2.4: Product update endpoints
    async fn update_listing(
        &self,
        listing_id: &str,
        patch: ListingPatch,
    ) -> Result<(), AdapterError> {
        let mut update_body = serde_json::Map::new();

        if let Some(title) = patch.title {
            update_body.insert("name".to_string(), Value::String(title));
        }
        if let Some(description) = patch.description {
            update_body.insert("description".to_string(), Value::String(description));
        }
        if let Some(price) = patch.price {
            update_body.insert(
                "price".to_string(),
                Value::Number(
                    (price * rust_decimal::Decimal::new(100, 0))
                        .to_string()
                        .parse()
                        .unwrap(),
                ),
            );
        }
        if let Some(tags) = patch.tags {
            update_body.insert("tags".to_string(), Value::Array(
                tags.into_iter().map(Value::String).collect()
            ));
        }
        if let Some(state) = patch.state {
            let status_str = match state {
                ListingState::Draft => "draft",
                ListingState::Publishing => "draft",
                ListingState::Live => "active",
                ListingState::Updating => "active",
                ListingState::Failed => "draft",
                ListingState::Delisting => "draft",
                ListingState::Delisted => "archived",
                ListingState::Suspended => "archived",
            };
            update_body.insert("status".to_string(), Value::String(status_str.to_string()));
        }

        self.execute_with_retry(|| async {
            let resp = self
                .client
                .patch(&format!("{}/products/{}", self.base_url, listing_id))
                .header("X-API-KEY", self.api_key_header())
                .header("Authorization", self.auth_header())
                .header("Content-Type", "application/json")
                .json(&update_body)
                .send()
                .await
                .map_err(AdapterError::Http)?;

            let status = resp.status().as_u16();
            if status == 429 {
                return Err(AdapterError::RateLimited("Gelato rate limit exceeded".to_string()));
            }
            if !resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(AdapterError::PlatformApi(format!(
                    "Gelato update error {}: {}",
                    status, text
                )));
            }

            Ok(())
        })
        .await
    }

    /// Upload a digital file to Gelato
    ///
    /// References:
    /// - docs/09-requirements.md §2.4: Files uploaded as part of product creation
    async fn upload_digital_file(
        &self,
        _listing_id: &str,
        file: UploadFile,
    ) -> Result<PlatformFileId, AdapterError> {
        // Gelato files are uploaded as part of product creation
        // Return a reference that can be used in product creation
        Ok(PlatformFileId {
            id: format!("gelato-file-{}", uuid::Uuid::new_v4()),
            url: Some(format!(
                "data:{};base64,{}",
                file.content_type,
                base64::encode(&file.bytes)
            )),
        })
    }

    /// Fetch orders from Gelato
    ///
    /// References:
    /// - docs/09-requirements.md §2.4: order.created, order.shipped, order.delivered webhooks
    async fn fetch_orders(&self, since: DateTime<Utc>) -> Result<Vec<Order>, AdapterError> {
        let since_str = since.to_rfc3339();

        let response = self
            .execute_with_retry(|| async {
                let resp = self
                    .client
                    .get(&format!("{}/orders?created_after={}", self.base_url, since_str))
                    .header("X-API-KEY", self.api_key_header())
                    .header("Authorization", self.auth_header())
                    .send()
                    .await
                    .map_err(AdapterError::Http)?;

                let status = resp.status().as_u16();
                if status == 429 {
                    return Err(AdapterError::RateLimited("Gelato rate limit exceeded".to_string()));
                }
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(AdapterError::PlatformApi(format!(
                        "Gelato orders fetch error {}: {}",
                        status, text
                    )));
                }

                let orders: Vec<GelatoOrderResponse> = resp
                    .json()
                    .await
                    .map_err(AdapterError::Http)?;

                Ok(orders)
            })
            .await?;

        let mut orders = Vec::new();
        for order in response {
            let mut items = Vec::new();
            for item in order.items {
                items.push(OrderItem {
                    listing_id: item.product_id,
                    quantity: item.quantity as u32,
                    unit_price: rust_decimal::Decimal::new(item.price.amount as i64, 2),
                    title: item.name,
                });
            }

            let occurred_at = DateTime::parse_from_rfc3339(&order.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            orders.push(Order {
                id: order.id,
                platform: "gelato".to_string(),
                status: match order.status.as_str() {
                    "created" | "pending" => OrderStatus::Pending,
                    "paid" | "confirmed" => OrderStatus::Paid,
                    "printed" | "in_production" => OrderStatus::Paid,
                    "shipped" | "in_transit" => OrderStatus::Fulfilled,
                    "delivered" => OrderStatus::Fulfilled,
                    "cancelled" => OrderStatus::Cancelled,
                    "refunded" => OrderStatus::Refunded,
                    _ => OrderStatus::Pending,
                },
                total: rust_decimal::Decimal::new(order.total.amount as i64, 2),
                items,
                buyer_email: None,
                occurred_at,
            });
        }

        Ok(orders)
    }

    /// Fetch statistics for a listing
    async fn fetch_stats(&self, listing_id: &str) -> Result<ListingStats, AdapterError> {
        let response = self
            .execute_with_retry(|| async {
                let resp = self
                    .client
                    .get(&format!("{}/products/{}", self.base_url, listing_id))
                    .header("X-API-KEY", self.api_key_header())
                    .header("Authorization", self.auth_header())
                    .send()
                    .await
                    .map_err(AdapterError::Http)?;

                let status = resp.status().as_u16();
                if status == 429 {
                    return Err(AdapterError::RateLimited("Gelato rate limit exceeded".to_string()));
                }
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(AdapterError::PlatformApi(format!(
                        "Gelato stats fetch error {}: {}",
                        status, text
                    )));
                }

                let product: GelatoProductResponse = resp
                    .json()
                    .await
                    .map_err(AdapterError::Http)?;

                Ok(product)
            })
            .await?;

        // Gelato doesn't provide detailed stats in product response
        // Return minimal stats
        Ok(ListingStats {
            views: 0,
            favorites: 0,
            sales: 0,
            revenue: rust_decimal::Decimal::ZERO,
        })
    }

    /// Verify webhook signature using constant-time comparison
    ///
    /// References:
    /// - docs/11-architecture.md §4: HMAC signature
    /// - docs/09-requirements.md §2.4: X-Gelato-Signature header
    /// - docs/10-security-research.md M5.2: Constant-time comparison
    /// - docs/11-architecture.md §3: verify_webhook runs BEFORE any business logic (M6.1)
    fn verify_webhook(
        &self,
        body: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<VerifiedWebhook, VerificationError> {
        // 1. Extract signature header
        let sig_header = "x-gelato-signature";
        let provided_sig = headers
            .get(sig_header)
            .ok_or_else(|| VerificationError {
                reason: "Missing X-Gelato-Signature header".to_string(),
            })?;

        // 2. Compute expected HMAC-SHA256
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.credentials.webhook_secret.as_bytes());
        let expected_sig = hmac::sign(&key, body);
        let expected_sig_hex = hex::encode(expected_sig.as_ref());

        // 3. Constant-time comparison (M5.2) - NEVER use ==
        if !bool::from(provided_sig.as_bytes().ct_eq(expected_sig_hex.as_bytes())) {
            return Err(VerificationError {
                reason: "Invalid signature".to_string(),
            });
        }

        // 4. Parse and validate payload
        let payload: Value = serde_json::from_slice(body).map_err(|_| VerificationError {
            reason: "Invalid JSON".to_string(),
        })?;

        // 5. Extract platform_event_id for idempotency
        let platform_event_id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| VerificationError {
                reason: "Missing event ID".to_string(),
            })?
            .to_string();

        // 6. Extract event_type
        let event_type = payload
            .get("type")
            .or_else(|| payload.get("event_type"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| VerificationError {
                reason: "Missing event type".to_string(),
            })?
            .to_string();

        Ok(VerifiedWebhook {
            event_type,
            platform_event_id,
            payload,
        })
    }

    /// Returns the rate limiter for this platform
    fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_gelato_adapter_creation() {
        let credentials = GelatoCredentials {
            api_key: "test_api_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            oauth2_access_token: None,
            oauth2_refresh_token: None,
            oauth2_expires_at: None,
        };
        let adapter = GelatoAdapter::new(credentials);

        // Verify rate limiter config: 4 req/s, burst 10 (conservative)
        let (max_tokens, refill_rate) = adapter.rate_limiter().test_config();
        assert_eq!(max_tokens, 10.0);
        assert_eq!(refill_rate, 4.0);
    }

    #[test]
    fn test_gelato_webhook_verification() {
        let credentials = GelatoCredentials {
            api_key: "test_api_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            oauth2_access_token: None,
            oauth2_refresh_token: None,
            oauth2_expires_at: None,
        };
        let adapter = GelatoAdapter::new(credentials);

        let body = br#"{"id": "evt_456", "type": "order.created"}"#;
        let mut headers = HashMap::new();

        // Compute valid signature
        let key = hmac::Key::new(hmac::HMAC_SHA256, b"test_secret");
        let sig = hmac::sign(&key, body);
        let sig_hex = hex::encode(sig.as_ref());
        headers.insert("x-gelato-signature".to_string(), sig_hex);

        let result = adapter.verify_webhook(body, &headers);
        assert!(result.is_ok());

        let webhook = result.unwrap();
        assert_eq!(webhook.event_type, "order.created");
        assert_eq!(webhook.platform_event_id, "evt_456");
    }

    #[test]
    fn test_gelato_webhook_invalid_signature() {
        let credentials = GelatoCredentials {
            api_key: "test_api_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            oauth2_access_token: None,
            oauth2_refresh_token: None,
            oauth2_expires_at: None,
        };
        let adapter = GelatoAdapter::new(credentials);

        let body = br#"{"id": "evt_456", "type": "order.created"}"#;
        let mut headers = HashMap::new();
        headers.insert("x-gelato-signature".to_string(), "invalid_signature".to_string());

        let result = adapter.verify_webhook(body, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_gelato_webhook_missing_header() {
        let credentials = GelatoCredentials {
            api_key: "test_api_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            oauth2_access_token: None,
            oauth2_refresh_token: None,
            oauth2_expires_at: None,
        };
        let adapter = GelatoAdapter::new(credentials);

        let body = br#"{"id": "evt_456", "type": "order.created"}"#;
        let headers = HashMap::new();

        let result = adapter.verify_webhook(body, &headers);
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("Missing"));
    }

    // G2: Gelato jittered backoff unit test
    #[test]
    fn test_gelato_jittered_backoff_g2() {
        let credentials = GelatoCredentials {
            api_key: "test_api_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            oauth2_access_token: None,
            oauth2_refresh_token: None,
            oauth2_expires_at: None,
        };
        let adapter = GelatoAdapter::new(credentials);

        // Verify jitter is enabled in retry policy
        assert!(adapter.retry_policy.jitter);

        // Test that delay_for_attempt produces varying results due to jitter
        let mut delays = Vec::new();
        for _ in 0..10 {
            let delay = adapter.retry_policy.delay_for_attempt(1); // 2nd attempt
            delays.push(delay.as_millis());
        }

        // With jitter, delays should vary
        // Note: This is a probabilistic test; in rare cases all could be same
        let min_delay = *delays.iter().min().unwrap();
        let max_delay = *delays.iter().max().unwrap();
        
        // Base delay for attempt 1: 200ms * 2 = 400ms, with jitter ±25% (±100ms)
        // So range should be roughly 300-500ms
        assert!(min_delay >= 300);
        assert!(max_delay <= 500);
        
        // At least some variation expected (allowing for small probability of all same)
        assert!(max_delay > min_delay || delays.len() < 5);
    }

    // G1: Rate limiter test for Gelato (conservative 4 req/s)
    #[tokio::test]
    async fn test_gelato_rate_limit_g1() {
        let credentials = GelatoCredentials {
            api_key: "test_api_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            oauth2_access_token: None,
            oauth2_refresh_token: None,
            oauth2_expires_at: None,
        };
        let adapter = GelatoAdapter::new(credentials);

        // The rate limiter allows burst of 10, then 4 req/s sustained
        // This test verifies the rate limiter config is correct

        // Acquire burst tokens
        for _ in 0..10 {
            adapter.rate_limiter().acquire().await;
        }

        // Next acquire should wait (refill at 4 tokens/sec)
        let start = std::time::Instant::now();
        adapter.rate_limiter().acquire().await;
        let elapsed = start.elapsed();

        // Should have waited approximately 1/4 seconds = 250ms
        assert!(elapsed >= Duration::from_millis(200));
        assert!(elapsed < Duration::from_millis(600));
    }
}