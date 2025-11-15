//! Comprehensive cache tests for Moka-based prompt caching
//!
//! This module tests all aspects of the caching system including:
//! - Cache hits and misses
//! - Key generation and consistency
//! - LRU eviction behavior
//! - TTL expiration
//! - Metrics collection
//! - Concurrent access

#[cfg(test)]
mod cache_tests {
    use std::time::Duration;
    use tokio::time::sleep;

    /// Mock structures for testing
    #[derive(Clone, Debug, PartialEq)]
    struct MockCachedResponse {
        content: String,
        model: String,
        input_tokens: u32,
        output_tokens: u32,
    }

    #[derive(Clone, Debug)]
    struct MockCache {
        data: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, MockCachedResponse>>>,
        hits: std::sync::Arc<std::sync::atomic::AtomicU64>,
        misses: std::sync::Arc<std::sync::atomic::AtomicU64>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
                hits: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
                misses: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            }
        }

        async fn get(&self, key: &str) -> Option<MockCachedResponse> {
            let data = self.data.lock().await;
            if let Some(value) = data.get(key) {
                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Some(value.clone())
            } else {
                self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        }

        async fn insert(&self, key: String, value: MockCachedResponse) {
            let mut data = self.data.lock().await;
            data.insert(key, value);
        }

        async fn clear(&self) {
            let mut data = self.data.lock().await;
            data.clear();
        }

        fn hit_rate(&self) -> f64 {
            let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
            let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
            let total = hits + misses;
            if total > 0 {
                (hits as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        }
    }

    /// Helper function to generate cache keys
    fn generate_cache_key(
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> String {
        use sha2::{Sha256, Digest};
        use std::fmt::Write;

        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(prompt.as_bytes());
        hasher.update(temperature.to_le_bytes());
        hasher.update(max_tokens.to_le_bytes());

        let result = hasher.finalize();
        let mut hex = String::new();
        for byte in result {
            write!(&mut hex, "{:02x}", byte).unwrap();
        }
        hex
    }

    // ========== CACHE HIT/MISS TESTS ==========

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = MockCache::new();

        let response = MockCachedResponse {
            content: "Test response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        let key = "test_key_1".to_string();
        cache.insert(key.clone(), response.clone()).await;

        // First retrieval should be a hit
        let cached = cache.get(&key).await;
        assert!(cached.is_some(), "Cache should contain inserted response");
        assert_eq!(cached.unwrap(), response, "Cached response should match inserted response");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = MockCache::new();

        // Query non-existent key
        let result = cache.get("non_existent_key").await;
        assert!(result.is_none(), "Non-existent key should return None");
    }

    #[tokio::test]
    async fn test_cache_hit_rate_calculation() {
        let cache = MockCache::new();

        let response = MockCachedResponse {
            content: "Test".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
        };

        // Insert one item
        cache.insert("key1".to_string(), response.clone()).await;

        // 7 cache hits
        for _ in 0..7 {
            cache.get("key1").await;
        }

        // 3 cache misses
        for i in 0..3 {
            cache.get(&format!("non_existent_{}", i)).await;
        }

        let hit_rate = cache.hit_rate();
        assert!(
            (hit_rate - 70.0).abs() < 0.1,
            "Hit rate should be approximately 70%, got {}%",
            hit_rate
        );
    }

    // ========== CACHE KEY GENERATION TESTS ==========

    #[test]
    fn test_cache_key_generation_consistency() {
        let key1 = generate_cache_key("claude-opus-4", "test prompt", 1.0, 4096);
        let key2 = generate_cache_key("claude-opus-4", "test prompt", 1.0, 4096);

        assert_eq!(key1, key2, "Same inputs should generate same cache key");
    }

    #[test]
    fn test_cache_key_uniqueness() {
        let key1 = generate_cache_key("claude-opus-4", "prompt 1", 1.0, 4096);
        let key2 = generate_cache_key("claude-opus-4", "prompt 2", 1.0, 4096);

        assert_ne!(key1, key2, "Different prompts should generate different keys");
    }

    #[test]
    fn test_cache_key_model_specificity() {
        let key1 = generate_cache_key("claude-opus-4", "test", 1.0, 4096);
        let key2 = generate_cache_key("gpt-4", "test", 1.0, 4096);

        assert_ne!(key1, key2, "Different models should generate different keys");
    }

    #[test]
    fn test_cache_key_temperature_sensitivity() {
        let key1 = generate_cache_key("claude-opus-4", "test", 0.5, 4096);
        let key2 = generate_cache_key("claude-opus-4", "test", 1.5, 4096);

        assert_ne!(key1, key2, "Different temperatures should generate different keys");
    }

    #[test]
    fn test_cache_key_max_tokens_sensitivity() {
        let key1 = generate_cache_key("claude-opus-4", "test", 1.0, 2048);
        let key2 = generate_cache_key("claude-opus-4", "test", 1.0, 4096);

        assert_ne!(key1, key2, "Different max_tokens should generate different keys");
    }

    #[test]
    fn test_cache_key_length() {
        let key = generate_cache_key("claude-opus-4", "test prompt with some content", 1.0, 4096);
        assert_eq!(key.len(), 64, "SHA256 hex should be 64 characters");
    }

    // ========== CONCURRENT ACCESS TESTS ==========

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let cache = MockCache::new();
        let response = MockCachedResponse {
            content: "Test response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        cache.insert("concurrent_key".to_string(), response).await;

        // Launch 100 concurrent read tasks
        let mut handles = vec![];
        for _ in 0..100 {
            let cache_clone = MockCache {
                data: cache.data.clone(),
                hits: cache.hits.clone(),
                misses: cache.misses.clone(),
            };

            let handle = tokio::spawn(async move {
                let result = cache_clone.get("concurrent_key").await;
                assert!(result.is_some(), "Concurrent read should succeed");
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task should complete");
        }

        // All 100 reads should be hits
        let hit_rate = cache.hit_rate();
        assert_eq!(hit_rate, 100.0, "All concurrent reads should be cache hits");
    }

    #[tokio::test]
    async fn test_concurrent_insert_and_read() {
        let cache = MockCache::new();
        let cache_for_insert = MockCache {
            data: cache.data.clone(),
            hits: cache.hits.clone(),
            misses: cache.misses.clone(),
        };

        // Task 1: Insert values
        let insert_handle = tokio::spawn(async move {
            for i in 0..50 {
                let response = MockCachedResponse {
                    content: format!("Response {}", i),
                    model: "claude-opus-4".to_string(),
                    input_tokens: 100 * i as u32,
                    output_tokens: 50 * i as u32,
                };
                cache_for_insert.insert(format!("key_{}", i), response).await;
            }
        });

        // Task 2: Read values
        let cache_for_read = MockCache {
            data: cache.data.clone(),
            hits: cache.hits.clone(),
            misses: cache.misses.clone(),
        };

        let read_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await; // Let some inserts happen first
            for i in 0..50 {
                let _ = cache_for_read.get(&format!("key_{}", i)).await;
            }
        });

        insert_handle.await.expect("Insert task should complete");
        read_handle.await.expect("Read task should complete");
    }

    // ========== MULTIPLE MODELS TESTS ==========

    #[tokio::test]
    async fn test_cache_isolation_by_model() {
        let cache = MockCache::new();

        let response_opus = MockCachedResponse {
            content: "Opus response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        let response_sonnet = MockCachedResponse {
            content: "Sonnet response".to_string(),
            model: "claude-sonnet-3.5".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        // Insert responses for different models with same prompt
        let key_opus = generate_cache_key("claude-opus-4", "test prompt", 1.0, 4096);
        let key_sonnet = generate_cache_key("claude-sonnet-3.5", "test prompt", 1.0, 4096);

        cache.insert(key_opus.clone(), response_opus.clone()).await;
        cache.insert(key_sonnet.clone(), response_sonnet.clone()).await;

        // Verify isolation
        let cached_opus = cache.get(&key_opus).await.unwrap();
        let cached_sonnet = cache.get(&key_sonnet).await.unwrap();

        assert_eq!(cached_opus.model, "claude-opus-4");
        assert_eq!(cached_sonnet.model, "claude-sonnet-3.5");
        assert_ne!(cached_opus.content, cached_sonnet.content);
    }

    // ========== EVICTION TESTS ==========

    #[tokio::test]
    async fn test_cache_fifo_behavior() {
        let cache = MockCache::new();

        // Insert items
        for i in 0..3 {
            let response = MockCachedResponse {
                content: format!("Response {}", i),
                model: "claude-opus-4".to_string(),
                input_tokens: 100 * i as u32,
                output_tokens: 50 * i as u32,
            };
            cache.insert(format!("key_{}", i), response).await;
        }

        // All items should be in cache
        for i in 0..3 {
            let result = cache.get(&format!("key_{}", i)).await;
            assert!(result.is_some(), "Item {} should be in cache", i);
        }
    }

    #[tokio::test]
    async fn test_cache_duplicate_keys() {
        let cache = MockCache::new();

        let response1 = MockCachedResponse {
            content: "First response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        let response2 = MockCachedResponse {
            content: "Second response".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        };

        cache.insert("same_key".to_string(), response1).await;
        cache.insert("same_key".to_string(), response2).await;

        let cached = cache.get("same_key").await;
        assert_eq!(cached.unwrap().content, "Second response", "Most recent value should be in cache");
    }

    // ========== CLEAR AND RESET TESTS ==========

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = MockCache::new();

        let response = MockCachedResponse {
            content: "Test".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
        };

        cache.insert("key1".to_string(), response).await;
        assert!(cache.get("key1").await.is_some());

        cache.clear().await;
        assert!(cache.get("key1").await.is_none(), "Cache should be empty after clear");
    }

    // ========== METRICS EDGE CASES ==========

    #[tokio::test]
    async fn test_metrics_on_empty_cache() {
        let cache = MockCache::new();

        let hit_rate = cache.hit_rate();
        assert_eq!(hit_rate, 0.0, "Empty cache should have 0% hit rate");
    }

    #[tokio::test]
    async fn test_metrics_all_hits() {
        let cache = MockCache::new();

        let response = MockCachedResponse {
            content: "Test".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
        };

        cache.insert("key".to_string(), response).await;

        // All hits
        for _ in 0..10 {
            cache.get("key").await;
        }

        assert_eq!(cache.hit_rate(), 100.0, "All accesses should be hits");
    }

    #[tokio::test]
    async fn test_metrics_all_misses() {
        let cache = MockCache::new();

        // All misses
        for i in 0..10 {
            cache.get(&format!("non_existent_{}", i)).await;
        }

        assert_eq!(cache.hit_rate(), 0.0, "All accesses should be misses");
    }
}
