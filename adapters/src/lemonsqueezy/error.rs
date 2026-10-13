//! Lemon Squeezy Error Types
//!
//! Error types specific to the Lemon Squeezy adapter implementation.
//!
//! # Official Documentation
//! - API Errors: <https://docs.lemonsqueezy.com/api>

use thiserror::Error;

/// Result type for Lemon Squeezy operations
pub type Result<T> = std::result::Result<T, LemonSqueezyError>;

/// Errors that can occur when interacting with the Lemon Squeezy API
#[derive(Debug, Error)]
pub enum LemonSqueezyError {
    /// HTTP client creation failed
    #[error("Failed to create HTTP client: {0}")]
    HttpClientCreation(reqwest::Error),

    /// HTTP request failed
    #[error("Request failed: {0}")]
    RequestFailed(reqwest::Error),

    /// Failed to read response text
    #[error("Failed to read response text: {0}")]
    ResponseText(reqwest::Error),

    /// JSON parsing failed
    #[error("JSON parse error: {0}")]
    JsonParse(serde_json::Error),

    /// API returned an error response
    #[error("API error (status {status}): {body}")]
    ApiError {
        status: u16,
        body: String,
    },

    /// Expected field missing from API response
    #[error("Missing field in API response: {0}")]
    MissingField(String),

    /// Invalid credentials
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    /// Webhook verification failed
    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<reqwest::Error> for LemonSqueezyError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_status() {
            // We can't easily get the status code here without the response
            LemonSqueezyError::RequestFailed(err)
        } else {
            LemonSqueezyError::RequestFailed(err)
        }
    }
}

impl From<serde_json::Error> for LemonSqueezyError {
    fn from(err: serde_json::Error) -> Self {
        LemonSqueezyError::JsonParse(err)
    }
}

impl From<LemonSqueezyError> for crate::platform::AdapterError {
    fn from(err: LemonSqueezyError) -> Self {
        crate::platform::AdapterError::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LemonSqueezyError::ApiError {
            status: 401,
            body: "Unauthorized".to_string(),
        };
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("Unauthorized"));
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        let ls_err: LemonSqueezyError = json_err.unwrap_err().into();
        assert!(matches!(ls_err, LemonSqueezyError::JsonParse(_)));
    }
}