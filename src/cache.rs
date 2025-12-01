//! Cache module with Moka-based prompt caching and metrics

use moka::future::Cache;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Cached response structure
#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Cache metrics snapshot
#[derive(Clone, Debug)]
pub struct CacheMetricsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub total_savings: f64,
}

/// Internal metrics tracking
struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    total_savings_cents: AtomicU64,
}

impl CacheMetrics {
    fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            total_savings_cents: AtomicU64::new(0),
        }
    }

    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_savings(&self, savings_cents: u64) {
        self.total_savings_cents
            .fetch_add(savings_cents, Ordering::Relaxed);
    }

    fn snapshot(&self) -> CacheMetricsSnapshot {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        CacheMetricsSnapshot {
            hits,
            misses,
            hit_rate: if total > 0 {
                (hits as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            total_savings: self.total_savings_cents.load(Ordering::Relaxed) as f64 / 100.0,
        }
    }
}

/// Moka-based cache with Window-TinyLFU eviction policy
#[derive(Clone)]
pub struct MokaCache {
    cache: Arc<Cache<String, CachedResponse>>,
    metrics: Arc<CacheMetrics>,
}

impl MokaCache {
    /// Create a new Moka cache
    ///
    /// # Arguments
    /// * `max_capacity` - Maximum capacity in bytes (e.g., 1GB = 1_073_741_824)
    /// * `ttl_seconds` - Time to live for cached entries
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache: Arc::new(cache),
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    /// Generate a SHA256-based cache key
    ///
    /// The key incorporates model, prompt, temperature, and max_tokens
    /// to ensure identical requests map to the same key.
    pub fn generate_key(model: &str, prompt: &str, temperature: f32, max_tokens: u32) -> String {
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(prompt.as_bytes());
        hasher.update(temperature.to_le_bytes());
        hasher.update(max_tokens.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get a cached response, recording metrics
    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        let result = self.cache.get(key).await;

        if result.is_some() {
            self.metrics.record_hit();
        } else {
            self.metrics.record_miss();
        }

        result
    }

    /// Insert a response into the cache
    pub async fn insert(&self, key: String, response: CachedResponse) {
        self.cache.insert(key, response).await;
    }

    /// Get current cache metrics
    pub async fn get_metrics(&self) -> CacheMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Record savings (in cents)
    pub fn record_savings(&self, savings_cents: u64) {
        self.metrics.record_savings(savings_cents);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_generation_consistency() {
        let key1 = MokaCache::generate_key("claude-opus-4", "test prompt", 1.0, 4096);
        let key2 = MokaCache::generate_key("claude-opus-4", "test prompt", 1.0, 4096);
        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_cache_key_uniqueness() {
        let key1 = MokaCache::generate_key("claude-opus-4", "prompt 1", 1.0, 4096);
        let key2 = MokaCache::generate_key("claude-opus-4", "prompt 2", 1.0, 4096);
        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = MokaCache::new(1024 * 1024, 60);
        let response = CachedResponse {
            content: "Test response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        let key = "test_key_1".to_string();
        cache.insert(key.clone(), response.clone()).await;

        let cached = cache.get(&key).await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = MokaCache::new(1024 * 1024, 60);
        let result = cache.get("non_existent_key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = MokaCache::new(1024 * 1024, 60);
        let response = CachedResponse {
            content: "Test".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
        };

        cache.insert("key1".to_string(), response).await;

        // 7 hits
        for _ in 0..7 {
            cache.get("key1").await;
        }

        // 3 misses
        for i in 0..3 {
            cache.get(&format!("non_existent_{}", i)).await;
        }

        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 7);
        assert_eq!(metrics.misses, 3);
        assert!((metrics.hit_rate - 70.0).abs() < 0.1);
    }
}
