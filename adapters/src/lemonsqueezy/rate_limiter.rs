//! Lemon Squeezy Rate Limiter
//!
//! Token bucket rate limiter for Lemon Squeezy API.
//!
//! # Official Documentation
//! - Rate Limits: <https://docs.lemonsqueezy.com/api> (not explicitly documented; 150 req/min conservative assumption)

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration, Instant};
use tracing::debug;

/// Trait for rate limiting behavior
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Wait until a request can be made
    async fn wait(&self);

    /// Try to acquire a permit without waiting
    fn try_wait(&self) -> bool;
}

/// Token bucket rate limiter for Lemon Squeezy
///
/// Conservative limit: 150 requests per minute (UNVERIFIED - public docs don't publish)
/// This translates to 2.5 requests per second sustained with burst allowance.
pub struct LemonSqueezyRateLimiter {
    permits: Arc<Semaphore>,
    refill_interval: Duration,
    max_permits: usize,
    last_refill: std::sync::Mutex<Instant>,
}

impl LemonSqueezyRateLimiter {
    /// Create a new rate limiter with the given requests per minute
    ///
    /// # Arguments
    /// * `requests_per_minute` - Maximum requests allowed per minute
    pub fn new(requests_per_minute: usize) -> Self {
        // Allow burst of up to 2x the sustained rate
        let max_permits = requests_per_minute * 2;
        let permits = Arc::new(Semaphore::new(max_permits));
        let refill_interval = Duration::from_secs(60) / requests_per_minute as u32;

        Self {
            permits,
            refill_interval,
            max_permits,
            last_refill: std::sync::Mutex::new(Instant::now()),
        }
    }

    /// Create a new rate limiter with custom burst allowance
    pub fn with_burst(requests_per_minute: usize, burst_multiplier: usize) -> Self {
        let max_permits = requests_per_minute * burst_multiplier;
        let permits = Arc::new(Semaphore::new(max_permits));
        let refill_interval = Duration::from_secs(60) / requests_per_minute as u32;

        Self {
            permits,
            refill_interval,
            max_permits,
            last_refill: std::sync::Mutex::new(Instant::now()),
        }
    }

    /// Refill permits based on elapsed time
    fn refill_if_needed(&self) {
        let mut last_refill = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= self.refill_interval {
            // Calculate how many permits to add
            let permits_to_add = (elapsed.as_millis() / self.refill_interval.as_millis()) as usize;
            if permits_to_add > 0 {
                let current = self.permits.available_permits();
                let new_permits = (current + permits_to_add).min(self.max_permits);
                let to_add = new_permits - current;
                if to_add > 0 {
                    self.permits.add_permits(to_add);
                    debug!(
                        "Rate limiter refilled {} permits (total: {})",
                        to_add, new_permits
                    );
                }
                *last_refill = now;
            }
        }
    }
}

#[async_trait]
impl RateLimiter for LemonSqueezyRateLimiter {
    async fn wait(&self) {
        self.refill_if_needed();

        let permit = self.permits.clone().acquire_owned().await;
        // Permit is dropped here, releasing it back to the semaphore
        drop(permit);
    }

    fn try_wait(&self) -> bool {
        self.refill_if_needed();
        match self.permits.try_acquire() {
            Ok(permit) => {
                permit.forget(); // consume the permit; do NOT release on drop
                true
            }
            Err(_) => false,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests_up_to_limit() {
        let limiter = LemonSqueezyRateLimiter::new(60); // 60 req/min = 1/sec

        // Should be able to acquire permits up to the burst limit
        for _ in 0..120 {
            assert!(limiter.try_wait());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_when_exhausted() {
        // 1 req/min -> refill interval 60s (no refill can race the test), burst 2x = 2 permits
        let limiter = LemonSqueezyRateLimiter::new(1);

        // Exhaust all permits
        assert!(limiter.try_wait());
        assert!(limiter.try_wait());

        // Next should fail
        assert!(!limiter.try_wait());
    }

    #[tokio::test]
    async fn test_rate_limiter_wait_respects_limit() {
        let limiter = LemonSqueezyRateLimiter::new(120); // 120 req/min = 2/sec

        // Should be able to make several quick requests
        let start = Instant::now();
        for _ in 0..10 {
            limiter.wait().await;
        }
        let elapsed = start.elapsed();

        // 10 requests at 2/sec should take at least ~5 seconds without burst
        // But with burst of 240, they should be nearly instant
        assert!(elapsed < Duration::from_secs(2));
    }
}