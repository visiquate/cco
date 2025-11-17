//! Integration tests for model override feature
//!
//! This module tests the complete end-to-end flow of model overrides including:
//! - Full request processing with overrides
//! - Cache key generation with overridden models
//! - Analytics tracking of overrides
//! - Cost calculation with overridden models

#[cfg(test)]
mod model_override_integration_tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Model override configuration
    #[derive(Clone)]
    struct OverrideConfig {
        rules: HashMap<String, String>,
    }

    impl OverrideConfig {
        fn new() -> Self {
            Self {
                rules: HashMap::new(),
            }
        }

        fn add_rule(&mut self, from: &str, to: &str) {
            self.rules.insert(from.to_string(), to.to_string());
        }

        fn apply_override(&self, model: &str) -> String {
            self.rules
                .get(model)
                .cloned()
                .unwrap_or_else(|| model.to_string())
        }
    }

    /// Chat request
    #[derive(Clone, Debug)]
    struct ChatRequest {
        model: String,
        messages: Vec<Message>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    }

    #[derive(Clone, Debug)]
    struct Message {
        role: String,
        content: String,
    }

    /// Chat response
    #[derive(Clone, Debug)]
    struct ChatResponse {
        model: String,
        content: String,
        input_tokens: u32,
        output_tokens: u32,
        from_cache: bool,
    }

    /// Cached response
    #[derive(Clone)]
    struct CachedResponse {
        model: String,
        content: String,
        input_tokens: u32,
        output_tokens: u32,
    }

    /// Analytics record
    #[derive(Clone, Debug)]
    struct AnalyticsRecord {
        model: String, // The overridden model
        original_model: Option<String>, // The requested model before override
        input_tokens: u32,
        output_tokens: u32,
        cache_hit: bool,
        actual_cost: f64,
        would_be_cost: f64,
        savings: f64,
    }

    /// Override record
    #[derive(Clone, Debug)]
    struct OverrideRecord {
        from_model: String,
        to_model: String,
    }

    /// Model pricing
    struct ModelPricing {
        input_cost: f64,  // per 1M tokens
        output_cost: f64, // per 1M tokens
    }

    /// Full proxy environment with override support
    struct ProxyWithOverrides {
        cache: Arc<Mutex<HashMap<String, CachedResponse>>>,
        analytics: Arc<Mutex<Vec<AnalyticsRecord>>>,
        override_records: Arc<Mutex<Vec<OverrideRecord>>>,
        override_config: OverrideConfig,
        pricing: HashMap<String, ModelPricing>,
    }

    impl ProxyWithOverrides {
        fn new(override_config: OverrideConfig) -> Self {
            let mut pricing = HashMap::new();

            // Add pricing for common models
            pricing.insert(
                "claude-sonnet-4.5".to_string(),
                ModelPricing {
                    input_cost: 3.0,
                    output_cost: 15.0,
                },
            );
            pricing.insert(
                "claude-haiku-4.5".to_string(),
                ModelPricing {
                    input_cost: 0.8,
                    output_cost: 4.0,
                },
            );
            pricing.insert(
                "claude-opus-4".to_string(),
                ModelPricing {
                    input_cost: 15.0,
                    output_cost: 75.0,
                },
            );

            Self {
                cache: Arc::new(Mutex::new(HashMap::new())),
                analytics: Arc::new(Mutex::new(Vec::new())),
                override_records: Arc::new(Mutex::new(Vec::new())),
                override_config,
                pricing,
            }
        }

        fn generate_cache_key(model: &str, prompt: &str) -> String {
            use sha2::{Digest, Sha256};
            use std::fmt::Write;

            let mut hasher = Sha256::new();
            hasher.update(model.as_bytes());
            hasher.update(prompt.as_bytes());

            let result = hasher.finalize();
            let mut hex = String::new();
            for byte in result {
                write!(&mut hex, "{:02x}", byte).unwrap();
            }
            hex
        }

        fn calculate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
            if let Some(pricing) = self.pricing.get(model) {
                (input_tokens as f64 / 1_000_000.0) * pricing.input_cost
                    + (output_tokens as f64 / 1_000_000.0) * pricing.output_cost
            } else {
                0.0
            }
        }

        async fn handle_request(&self, mut request: ChatRequest) -> ChatResponse {
            let original_model = request.model.clone();
            let prompt = request
                .messages
                .last()
                .map(|m| m.content.clone())
                .unwrap_or_default();

            // Apply override
            let overridden_model = self.override_config.apply_override(&request.model);
            let was_overridden = overridden_model != request.model;

            if was_overridden {
                // Record override
                let mut override_records = self.override_records.lock().await;
                override_records.push(OverrideRecord {
                    from_model: request.model.clone(),
                    to_model: overridden_model.clone(),
                });
            }

            request.model = overridden_model.clone();

            // Generate cache key using overridden model
            let cache_key = Self::generate_cache_key(&request.model, &prompt);

            // Check cache
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                drop(cache);

                // Calculate savings
                let would_be_cost =
                    self.calculate_cost(&request.model, cached.input_tokens, cached.output_tokens);

                // Record analytics
                let mut analytics = self.analytics.lock().await;
                analytics.push(AnalyticsRecord {
                    model: request.model.clone(),
                    original_model: if was_overridden {
                        Some(original_model)
                    } else {
                        None
                    },
                    input_tokens: cached.input_tokens,
                    output_tokens: cached.output_tokens,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost,
                    savings: would_be_cost,
                });

                return ChatResponse {
                    model: cached.model.clone(),
                    content: cached.content.clone(),
                    input_tokens: cached.input_tokens,
                    output_tokens: cached.output_tokens,
                    from_cache: true,
                };
            }
            drop(cache);

            // Simulate API call
            let input_tokens = 1000;
            let output_tokens = 500;
            let content = "Simulated response".to_string();

            let actual_cost = self.calculate_cost(&request.model, input_tokens, output_tokens);

            // Store in cache
            let mut cache = self.cache.lock().await;
            cache.insert(
                cache_key,
                CachedResponse {
                    model: request.model.clone(),
                    content: content.clone(),
                    input_tokens,
                    output_tokens,
                },
            );
            drop(cache);

            // Record analytics
            let mut analytics = self.analytics.lock().await;
            analytics.push(AnalyticsRecord {
                model: request.model.clone(),
                original_model: if was_overridden {
                    Some(original_model)
                } else {
                    None
                },
                input_tokens,
                output_tokens,
                cache_hit: false,
                actual_cost,
                would_be_cost: actual_cost,
                savings: 0.0,
            });

            ChatResponse {
                model: request.model,
                content,
                input_tokens,
                output_tokens,
                from_cache: false,
            }
        }

        async fn get_override_count(&self) -> usize {
            let records = self.override_records.lock().await;
            records.len()
        }

        async fn get_analytics_count(&self) -> usize {
            let analytics = self.analytics.lock().await;
            analytics.len()
        }

        async fn get_total_cost(&self) -> f64 {
            let analytics = self.analytics.lock().await;
            analytics.iter().map(|r| r.actual_cost).sum()
        }

        async fn get_total_savings(&self) -> f64 {
            let analytics = self.analytics.lock().await;
            analytics.iter().map(|r| r.savings).sum()
        }
    }

    // ========== FULL REQUEST FLOW WITH OVERRIDE ==========

    #[tokio::test]
    async fn test_full_chat_completion_with_override() {
        // Setup: Start server with override rules
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        // Send: ChatRequest with model="claude-sonnet-4.5"
        let request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let response = proxy.handle_request(request).await;

        // Then:
        // - Response should indicate model="claude-haiku-4.5" was used
        assert_eq!(
            response.model, "claude-haiku-4.5",
            "Response should use overridden model"
        );

        // - Analytics should record the override
        assert_eq!(proxy.get_override_count().await, 1);

        // - Cost should be calculated using haiku pricing
        let analytics = proxy.analytics.lock().await;
        let record = &analytics[0];
        assert_eq!(record.model, "claude-haiku-4.5");
        assert!(record.original_model.is_some());
        assert_eq!(record.original_model.as_ref().unwrap(), "claude-sonnet-4.5");

        // Haiku pricing: 1000 tokens * $0.8/1M + 500 tokens * $4/1M = $0.0028
        assert!(
            (record.actual_cost - 0.0028).abs() < 0.0001,
            "Cost should be for haiku, got {}",
            record.actual_cost
        );
    }

    #[tokio::test]
    async fn test_cache_key_uses_overridden_model() {
        // Given: Two requests - one for sonnet, one for haiku
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        let request1 = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Same prompt".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let request2 = ChatRequest {
            model: "claude-haiku-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Same prompt".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // When: sonnet request gets overridden to haiku
        let response1 = proxy.handle_request(request1).await;
        assert!(!response1.from_cache, "First request should be cache miss");

        // Then: second haiku request should hit same cache entry
        let response2 = proxy.handle_request(request2).await;
        assert!(
            response2.from_cache,
            "Second request should be cache hit (same model after override)"
        );
    }

    #[tokio::test]
    async fn test_analytics_records_overridden_model() {
        // Given: Chat request with sonnet
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        let request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // When: processed through proxy with override to haiku
        proxy.handle_request(request).await;

        // Then: analytics records should show:
        let analytics = proxy.analytics.lock().await;
        assert_eq!(analytics.len(), 1);

        let record = &analytics[0];
        // - api_call_record.model = "claude-haiku-4.5" (the actual model used)
        assert_eq!(record.model, "claude-haiku-4.5");

        // - override_record shows: sonnet → haiku transformation
        assert!(record.original_model.is_some());
        assert_eq!(record.original_model.as_ref().unwrap(), "claude-sonnet-4.5");
    }

    #[tokio::test]
    async fn test_cost_calculated_for_overridden_model() {
        // Given: Request for sonnet (expensive) overridden to haiku
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        let request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // When: cost calculation done
        proxy.handle_request(request).await;

        // Then: cost should be haiku pricing, not sonnet
        let analytics = proxy.analytics.lock().await;
        let record = &analytics[0];

        // Haiku: 1000 * $0.8/1M + 500 * $4/1M = $0.0028
        // Sonnet: 1000 * $3/1M + 500 * $15/1M = $0.0105
        assert!(
            (record.actual_cost - 0.0028).abs() < 0.0001,
            "Cost should be haiku pricing (~$0.0028), got ${}",
            record.actual_cost
        );
    }

    #[tokio::test]
    async fn test_multiple_requests_with_different_overrides() {
        // Given: 3 requests - sonnet, opus, sonnet
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        // No rule for opus, it passes through
        let proxy = ProxyWithOverrides::new(config);

        // Request 1: sonnet (will be overridden)
        let request1 = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Question 1".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // Request 2: opus (no override)
        let request2 = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Question 2".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // Request 3: sonnet again (will be overridden)
        let request3 = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Question 3".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // When: all processed through proxy
        proxy.handle_request(request1).await;
        proxy.handle_request(request2).await;
        proxy.handle_request(request3).await;

        // Then: analytics should show correct overrides
        assert_eq!(proxy.get_override_count().await, 2, "Two sonnet requests should be overridden");

        let analytics = proxy.analytics.lock().await;
        assert_eq!(analytics.len(), 3);

        // First: sonnet → haiku (overridden)
        assert_eq!(analytics[0].model, "claude-haiku-4.5");
        assert_eq!(analytics[0].original_model.as_ref().unwrap(), "claude-sonnet-4.5");

        // Second: opus → opus (unchanged)
        assert_eq!(analytics[1].model, "claude-opus-4");
        assert!(analytics[1].original_model.is_none());

        // Third: sonnet → haiku (overridden)
        assert_eq!(analytics[2].model, "claude-haiku-4.5");
        assert_eq!(analytics[2].original_model.as_ref().unwrap(), "claude-sonnet-4.5");
    }

    // ========== CACHE + OVERRIDE INTERACTION TESTS ==========

    #[tokio::test]
    async fn test_override_then_cache_hit() {
        // Given: proxy with override rule
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        let request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Same question".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // First request: override + cache miss
        let response1 = proxy.handle_request(request.clone()).await;
        assert!(!response1.from_cache);
        assert_eq!(response1.model, "claude-haiku-4.5");

        // Second identical request: override + cache hit
        let response2 = proxy.handle_request(request).await;
        assert!(response2.from_cache, "Second request should be cache hit");
        assert_eq!(response2.model, "claude-haiku-4.5");

        // Both should have recorded override
        assert_eq!(proxy.get_override_count().await, 2);

        // Second should have savings
        let analytics = proxy.analytics.lock().await;
        assert!(analytics[1].cache_hit);
        assert!(analytics[1].savings > 0.0);
    }

    #[tokio::test]
    async fn test_different_models_same_prompt_different_cache() {
        // Given: two different models with same prompt
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        let request1 = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Same question".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let request2 = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Same question".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // When: both processed
        let response1 = proxy.handle_request(request1).await;
        let response2 = proxy.handle_request(request2).await;

        // Then: both should be cache misses (different models)
        assert!(!response1.from_cache);
        assert!(!response2.from_cache);

        // One override, one not
        assert_eq!(proxy.get_override_count().await, 1);
    }

    // ========== CONCURRENT OVERRIDE TESTS ==========

    #[tokio::test]
    async fn test_concurrent_requests_with_overrides() {
        // Given: proxy with override rules
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = Arc::new(ProxyWithOverrides::new(config));

        // When: 50 concurrent requests
        let mut handles = vec![];
        for i in 0..50 {
            let proxy_clone = proxy.clone();
            let handle = tokio::spawn(async move {
                let request = ChatRequest {
                    model: "claude-sonnet-4.5".to_string(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: format!("Question {}", i % 10), // 10 unique questions
                    }],
                    temperature: Some(1.0),
                    max_tokens: Some(4096),
                };
                proxy_clone.handle_request(request).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let response = handle.await.expect("Task should complete");
            assert_eq!(
                response.model, "claude-haiku-4.5",
                "All responses should use overridden model"
            );
        }

        // Then: all 50 should be overridden
        assert_eq!(proxy.get_override_count().await, 50);

        // 50 requests with 10 unique prompts = ~10 cache misses, ~40 cache hits
        let analytics = proxy.analytics.lock().await;
        let cache_hits = analytics.iter().filter(|r| r.cache_hit).count();
        assert!(
            cache_hits >= 30,
            "Should have significant cache hits from duplicates"
        );
    }

    // ========== COST SAVINGS TESTS ==========

    #[tokio::test]
    async fn test_cost_savings_from_override() {
        // Given: sonnet being overridden to cheaper haiku
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        let proxy = ProxyWithOverrides::new(config);

        // Calculate what sonnet would cost
        let sonnet_cost = proxy.calculate_cost("claude-sonnet-4.5", 1000, 500);
        let haiku_cost = proxy.calculate_cost("claude-haiku-4.5", 1000, 500);

        let request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // When: request processed with override
        proxy.handle_request(request).await;

        // Then: actual cost should be haiku, not sonnet
        let total_cost = proxy.get_total_cost().await;
        assert!(
            (total_cost - haiku_cost).abs() < 0.0001,
            "Should pay haiku cost (${}), not sonnet cost (${})",
            haiku_cost,
            sonnet_cost
        );
    }
}
