//! Lemon Squeezy Credentials
//!
//! Credentials structure for Lemon Squeezy API authentication.
//!
//! # Official Documentation
//! - API Key: <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started>
//! - Webhook Secret: <https://docs.lemonsqueezy.com/api/webhooks>

use serde::{Deserialize, Serialize};

/// Credentials for Lemon Squeezy API access
///
/// # Fields
/// - `api_key`: Bearer token from Lemon Squeezy dashboard (Settings → API)
/// - `store_id`: Store identifier for scoping API requests
/// - `webhook_secret`: Signing secret for webhook signature verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LemonSqueezyCredentials {
    /// Bearer API key from Lemon Squeezy dashboard
    pub api_key: String,

    /// Store ID for scoping requests
    pub store_id: String,

    /// Webhook signing secret for signature verification
    pub webhook_secret: String,
}

impl LemonSqueezyCredentials {
    /// Create new credentials
    pub fn new(api_key: String, store_id: String, webhook_secret: String) -> Self {
        Self {
            api_key,
            store_id,
            webhook_secret,
        }
    }

    /// Validate that all required fields are present
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.api_key.is_empty() {
            return Err("api_key is required");
        }
        if self.store_id.is_empty() {
            return Err("store_id is required");
        }
        if self.webhook_secret.is_empty() {
            return Err("webhook_secret is required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_validation() {
        let creds = LemonSqueezyCredentials::new(
            "test_key".to_string(),
            "test_store".to_string(),
            "test_secret".to_string(),
        );
        assert!(creds.validate().is_ok());

        let creds_empty = LemonSqueezyCredentials::new(
            "".to_string(),
            "test_store".to_string(),
            "test_secret".to_string(),
        );
        assert!(creds_empty.validate().is_err());
    }
}