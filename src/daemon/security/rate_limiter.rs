//! Rate limiting module
//!
//! Implements token bucket rate limiting with:
//! - 100 requests per minute per token
//! - 1000 requests per hour per token
//! - 429 Too Many Requests responses with Retry-After header

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::debug;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            requests_per_hour: 1000,
        }
    }
}

/// Rate limit error
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {limit} requests per {window}. Retry after {retry_after_seconds} seconds")]
    Exceeded {
        limit: usize,
        window: String,
        retry_after_seconds: i64,
    },
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens_minute: usize,
    tokens_hour: usize,
    last_refill_minute: DateTime<Utc>,
    last_refill_hour: DateTime<Utc>,
    max_tokens_minute: usize,
    max_tokens_hour: usize,
}

impl TokenBucket {
    fn new(config: &RateLimitConfig) -> Self {
        let now = Utc::now();
        Self {
            tokens_minute: config.requests_per_minute,
            tokens_hour: config.requests_per_hour,
            last_refill_minute: now,
            last_refill_hour: now,
            max_tokens_minute: config.requests_per_minute,
            max_tokens_hour: config.requests_per_hour,
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Utc::now();

        // Refill minute bucket (every 60 seconds)
        let minutes_elapsed = (now - self.last_refill_minute).num_seconds() / 60;
        if minutes_elapsed > 0 {
            self.tokens_minute = self.max_tokens_minute;
            self.last_refill_minute = now;
            debug!("Refilled minute bucket: {} tokens", self.tokens_minute);
        }

        // Refill hour bucket (every 3600 seconds)
        let hours_elapsed = (now - self.last_refill_hour).num_seconds() / 3600;
        if hours_elapsed > 0 {
            self.tokens_hour = self.max_tokens_hour;
            self.last_refill_hour = now;
            debug!("Refilled hour bucket: {} tokens", self.tokens_hour);
        }
    }

    /// Try to consume a token
    fn try_consume(&mut self) -> Result<(), RateLimitError> {
        self.refill();

        // Check minute limit
        if self.tokens_minute == 0 {
            let retry_after = 60 - (Utc::now() - self.last_refill_minute).num_seconds();
            return Err(RateLimitError::Exceeded {
                limit: self.max_tokens_minute,
                window: "minute".to_string(),
                retry_after_seconds: retry_after.max(1),
            });
        }

        // Check hour limit
        if self.tokens_hour == 0 {
            let retry_after = 3600 - (Utc::now() - self.last_refill_hour).num_seconds();
            return Err(RateLimitError::Exceeded {
                limit: self.max_tokens_hour,
                window: "hour".to_string(),
                retry_after_seconds: retry_after.max(1),
            });
        }

        // Consume tokens
        self.tokens_minute -= 1;
        self.tokens_hour -= 1;

        debug!(
            "Consumed token. Remaining: minute={}, hour={}",
            self.tokens_minute, self.tokens_hour
        );

        Ok(())
    }

    /// Get remaining tokens
    fn remaining(&mut self) -> (usize, usize) {
        self.refill();
        (self.tokens_minute, self.tokens_hour)
    }
}

/// Rate limiter
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Check if request is allowed for a token
    pub async fn check_rate_limit(&self, token_hash: &str) -> Result<(), RateLimitError> {
        let mut buckets = self.buckets.write().await;

        // Get or create bucket for this token
        let bucket = buckets
            .entry(token_hash.to_string())
            .or_insert_with(|| TokenBucket::new(&self.config));

        bucket.try_consume()
    }

    /// Get remaining requests for a token
    pub async fn get_remaining(&self, token_hash: &str) -> Option<(usize, usize)> {
        let mut buckets = self.buckets.write().await;

        buckets.get_mut(token_hash).map(|bucket| bucket.remaining())
    }

    /// Cleanup expired buckets (call periodically)
    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.write().await;
        let initial_count = buckets.len();

        // Remove buckets that haven't been used in over an hour
        let now = Utc::now();
        buckets.retain(|_, bucket| (now - bucket.last_refill_hour).num_hours() < 2);

        let removed = initial_count - buckets.len();
        if removed > 0 {
            debug!("Cleaned up {} inactive rate limit buckets", removed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_creation() {
        let config = RateLimitConfig::default();
        let bucket = TokenBucket::new(&config);

        assert_eq!(bucket.tokens_minute, 100);
        assert_eq!(bucket.tokens_hour, 1000);
    }

    #[test]
    fn test_token_bucket_consume() {
        let config = RateLimitConfig::default();
        let mut bucket = TokenBucket::new(&config);

        assert!(bucket.try_consume().is_ok());
        assert_eq!(bucket.tokens_minute, 99);
        assert_eq!(bucket.tokens_hour, 999);
    }

    #[test]
    fn test_token_bucket_minute_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 1000,
        };
        let mut bucket = TokenBucket::new(&config);

        assert!(bucket.try_consume().is_ok());
        assert!(bucket.try_consume().is_ok());

        let result = bucket.try_consume();
        assert!(result.is_err());

        if let Err(RateLimitError::Exceeded { limit, window, .. }) = result {
            assert_eq!(limit, 2);
            assert_eq!(window, "minute");
        } else {
            panic!("Expected RateLimitError::Exceeded");
        }
    }

    #[test]
    fn test_token_bucket_hour_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 1000,
            requests_per_hour: 2,
        };
        let mut bucket = TokenBucket::new(&config);

        assert!(bucket.try_consume().is_ok());
        assert!(bucket.try_consume().is_ok());

        let result = bucket.try_consume();
        assert!(result.is_err());

        if let Err(RateLimitError::Exceeded { limit, window, .. }) = result {
            assert_eq!(limit, 2);
            assert_eq!(window, "hour");
        } else {
            panic!("Expected RateLimitError::Exceeded");
        }
    }

    #[test]
    fn test_token_bucket_remaining() {
        let config = RateLimitConfig::default();
        let mut bucket = TokenBucket::new(&config);

        bucket.try_consume().unwrap();
        let (minute, hour) = bucket.remaining();

        assert_eq!(minute, 99);
        assert_eq!(hour, 999);
    }

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::default();
        assert_eq!(limiter.config.requests_per_minute, 100);
    }

    #[tokio::test]
    async fn test_rate_limiter_check() {
        let limiter = RateLimiter::default();
        let token = "test-token-hash";

        assert!(limiter.check_rate_limit(token).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_limit_enforcement() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            requests_per_hour: 100,
        };
        let limiter = RateLimiter::new(config);
        let token = "test-token";

        // Should allow 5 requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(token).await.is_ok());
        }

        // 6th request should fail
        let result = limiter.check_rate_limit(token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_get_remaining() {
        let limiter = RateLimiter::default();
        let token = "test-token";

        limiter.check_rate_limit(token).await.unwrap();

        let remaining = limiter.get_remaining(token).await;
        assert!(remaining.is_some());

        let (minute, hour) = remaining.unwrap();
        assert_eq!(minute, 99);
        assert_eq!(hour, 999);
    }

    #[tokio::test]
    async fn test_rate_limiter_multiple_tokens() {
        let limiter = RateLimiter::default();

        assert!(limiter.check_rate_limit("token1").await.is_ok());
        assert!(limiter.check_rate_limit("token2").await.is_ok());

        let remaining1 = limiter.get_remaining("token1").await;
        let remaining2 = limiter.get_remaining("token2").await;

        assert_eq!(remaining1, Some((99, 999)));
        assert_eq!(remaining2, Some((99, 999)));
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup() {
        let limiter = RateLimiter::default();
        limiter.check_rate_limit("token1").await.unwrap();

        limiter.cleanup().await;

        // Token should still exist (not old enough)
        let remaining = limiter.get_remaining("token1").await;
        assert!(remaining.is_some());
    }
}
