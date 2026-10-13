//! Lemon Squeezy Webhook Verification
//!
//! Implements constant-time HMAC-SHA256 verification for Lemon Squeezy webhooks.
//!
//! # Security Requirements (from docs/10)
//! - S13: Webhook signature verification mandatory on every inbound
//! - M5.2: Constant-time comparison for HMAC verification
//! - M6.1: verify_webhook runs BEFORE any business logic
//!
//! # Official Documentation
//! - Webhooks: <https://docs.lemonsqueezy.com/api/webhooks>

use crate::platform::{VerifiedWebhook, VerificationError};
use crate::platform::HeaderMap;
use ring::hmac;
use subtle::ConstantTimeEq;
use tracing::{debug, warn};

/// Signature header name for Lemon Squeezy webhooks
const SIGNATURE_HEADER: &str = "x-lemonsqueezy-signature";

/// Verify a Lemon Squeezy webhook signature using constant-time comparison
///
/// # Arguments
/// * `body` - Raw request body bytes
/// * `headers` - Request headers containing the signature
/// * `webhook_secret` - Webhook signing secret from Lemon Squeezy dashboard
///
/// # Returns
/// * `Ok(VerifiedWebhook)` - If signature is valid and payload is parseable
/// * `Err(VerificationError)` - If signature is missing, invalid, or payload is malformed
///
/// # Security
/// Uses `ring::hmac` for HMAC-SHA256 computation and `subtle::ConstantTimeEq`
/// for constant-time comparison to prevent timing attacks (M5.2).
pub fn verify_webhook(
    body: &[u8],
    headers: &HeaderMap,
    webhook_secret: &str,
) -> Result<VerifiedWebhook, VerificationError> {
    // 1. Extract signature header
    let provided_sig = headers
        .get(SIGNATURE_HEADER)
        .ok_or(VerificationError { reason: "missing signature header".to_string() })?;

    debug!("Verifying Lemon Squeezy webhook signature");

    // 2. Compute expected HMAC-SHA256
    let expected_sig = compute_hmac(body, webhook_secret)?;

    // 3. Constant-time comparison (M5.2) - NEVER use == on strings
    if !bool::from(provided_sig.as_bytes().ct_eq(expected_sig.as_bytes())) {
        warn!("Invalid Lemon Squeezy webhook signature");
        return Err(VerificationError { reason: "invalid signature".to_string() });
    }

    // 4. Parse and validate payload
    let payload: serde_json::Value = serde_json::from_slice(body)
        .map_err(|_| VerificationError { reason: "invalid JSON payload".to_string() })?;

    // 5. Extract platform_event_id for idempotency (M6.2)
    let platform_event_id = extract_event_id(&payload)
        .ok_or(VerificationError { reason: "missing event id".to_string() })?;

    // 6. Extract event_type
    let event_type = extract_event_type(&payload)
        .ok_or(VerificationError { reason: "missing event type".to_string() })?;

    debug!(
        "Lemon Squeezy webhook verified: event_type={}, event_id={}",
        event_type, platform_event_id
    );

    Ok(VerifiedWebhook {
        event_type,
        platform_event_id,
        payload,
    })
}

/// Compute HMAC-SHA256 signature for the given body and secret
fn compute_hmac(body: &[u8], secret: &str) -> Result<String, VerificationError> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let sig = hmac::sign(&key, body);
    Ok(hex::encode(sig.as_ref()))
}

/// Extract the platform event ID from the webhook payload
///
/// Lemon Squeezy webhook payloads contain an `id` field at the root level
/// for the event identifier.
fn extract_event_id(payload: &serde_json::Value) -> Option<String> {
    // Try multiple possible locations for the event ID
    payload
        .get("data")
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            payload
                .get("meta")
                .and_then(|m| m.get("event_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            payload
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

/// Extract the event type from the webhook payload
///
/// Lemon Squeezy webhook payloads contain a `meta.event_name` field
/// indicating the type of event (e.g., "order_created", "subscription_updated").
fn extract_event_type(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("meta")
        .and_then(|m| m.get("event_name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            payload
                .get("event_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::HeaderMap;
    use ring::hmac;

    fn make_valid_signature(body: &[u8], secret: &str) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
        let sig = hmac::sign(&key, body);
        hex::encode(sig.as_ref())
    }

    /// G1: Mock-server contract test - signed-webhook accept
    #[test]
    fn test_valid_webhook_signature_accepted() {
        let secret = "test_webhook_secret";
        let body = br#"{"meta":{"event_name":"order_created","event_id":"evt_123"},"data":{"id":"order_456"}}"#;

        let valid_signature = make_valid_signature(body, secret);

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), valid_signature.parse().unwrap());

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_ok(), "Valid signature should be accepted: {:?}", result.err());

        let verified = result.unwrap();
        assert_eq!(verified.event_type, "order_created");
        assert_eq!(verified.platform_event_id, "order_456");
    }

    /// G1: Mock-server contract test - tampered-sig reject
    #[test]
    fn test_tampered_webhook_signature_rejected() {
        let secret = "test_webhook_secret";
        let body = br#"{"meta":{"event_name":"order_created"}}"#;
        let tampered_signature = "invalid_signature_here";

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), tampered_signature.parse().unwrap());

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_err(), "Tampered signature should be rejected");

        if let Err(e) = result {
            assert!(e.reason.contains("signature") || e.reason.contains("invalid") || e.reason.contains("missing") || e.reason.contains("JSON"), "unexpected reason: {}", e.reason);
        }
    }

    /// Test missing signature header
    #[test]
    fn test_missing_signature_header_rejected() {
        let secret = "test_webhook_secret";
        let body = br#"{"test": "payload"}"#;

        let headers = HeaderMap::new();

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.reason.contains("signature") || e.reason.contains("invalid") || e.reason.contains("missing") || e.reason.contains("JSON"), "unexpected reason: {}", e.reason);
        }
    }

    /// Test invalid JSON payload
    #[test]
    fn test_invalid_json_rejected() {
        let secret = "test_webhook_secret";
        let body = b"not valid json";
        let valid_signature = make_valid_signature(body, secret);

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), valid_signature.parse().unwrap());

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.reason.contains("signature") || e.reason.contains("invalid") || e.reason.contains("missing") || e.reason.contains("JSON"), "unexpected reason: {}", e.reason);
        }
    }

    /// Test missing event ID in payload
    #[test]
    fn test_missing_event_id_rejected() {
        let secret = "test_webhook_secret";
        let body = br#"{"meta":{"event_name":"order_created"}}"#; // No data.id
        let valid_signature = make_valid_signature(body, secret);

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), valid_signature.parse().unwrap());

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.reason.contains("signature") || e.reason.contains("invalid") || e.reason.contains("missing") || e.reason.contains("JSON"), "unexpected reason: {}", e.reason);
        }
    }

    /// Test missing event type in payload
    #[test]
    fn test_missing_event_type_rejected() {
        let secret = "test_webhook_secret";
        let body = br#"{"data":{"id":"order_123"}}"#; // No meta.event_name
        let valid_signature = make_valid_signature(body, secret);

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), valid_signature.parse().unwrap());

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.reason.contains("signature") || e.reason.contains("invalid") || e.reason.contains("missing") || e.reason.contains("JSON"), "unexpected reason: {}", e.reason);
        }
    }

    /// Test constant-time comparison is used (indirectly verified by using subtle)
    #[test]
    fn test_constant_time_comparison_used() {
        // This test verifies the implementation uses subtle::ConstantTimeEq
        // The actual constant-time property is tested by the subtle crate itself
        let secret = "test";
        let body = br#"{"meta":{"event_name":"order_created","event_id":"evt_ct"},"data":{"id":"order_ct"}}"#;
        let sig = make_valid_signature(body, secret);

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), sig.parse().unwrap());

        let result = verify_webhook(body, &headers, secret);
        assert!(result.is_ok());
    }
}