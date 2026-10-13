//! Platform Adapters for digital product publishing
//!
//! This crate provides implementations of the PlatformAdapter trait for:
//! - Etsy (OAuth2 + HMAC webhooks)
//! - Lemon Squeezy (Bearer API key + signed webhooks)
//! - Printify (Bearer token + HMAC webhooks, 600 req/min cap)
//! - Gelato (API key + HMAC webhooks, jittered backoff)

pub mod platform;
pub mod etsy;
pub mod printify;
pub mod gelato;
pub mod lemonsqueezy;

/// Re-export common types
pub use platform::{
    AdapterError, ListingPatch, ListingState, Order, OrderItem, OrderStatus, Platform,
    PlatformAdapter, PlatformFileId, Product, PublishReceipt, RateLimiter, RateLimiterConfig,
    RetryPolicy, UploadFile, VerifiedWebhook, VerificationError, build_shared_client,
};

/// Re-export Etsy adapter
pub use etsy::EtsyAdapter;
pub use etsy::EtsyCredentials;

/// Re-export Printify adapter
pub use printify::PrintifyAdapter;
pub use printify::PrintifyCredentials;

/// Re-export Gelato adapter
pub use gelato::GelatoAdapter;
pub use gelato::GelatoCredentials;

/// Re-export Lemon Squeezy adapter
pub use lemonsqueezy::LemonSqueezyAdapter;
pub use lemonsqueezy::LemonSqueezyCredentials;