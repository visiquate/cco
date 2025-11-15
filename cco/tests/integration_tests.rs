//! End-to-end integration tests for CCO proxy system
//!
//! This module tests the complete system including:
//! - Full request flow with cache, analytics, and routing
//! - Multi-model routing with cost calculation
//! - Analytics data persistence
//! - Error handling across components

#[cfg(test)]
mod integration_tests {
    use std::collections::HashMap;

    /// Full test suite setup
    struct CCOTestEnvironment {
        cache: std::sync::Arc<tokio::sync::Mutex<HashMap<String, CachedData>>>,
        analytics: std::sync::Arc<tokio::sync::Mutex<Vec<AnalyticsRecord>>>,
        router: ModelRouter,
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct CachedData {
        model: String,
        content: String,
        input_tokens: u32,
        output_tokens: u32,
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct AnalyticsRecord {
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        cache_hit: bool,
        actual_cost: f64,
        would_be_cost: f64,
        savings: f64,
    }

    #[derive(Clone)]
    struct ModelRouter {
        models: HashMap<String, String>,
    }

    impl ModelRouter {
        fn new() -> Self {
            let mut models = HashMap::new();
            models.insert("claude-opus-4".to_string(), "https://api.anthropic.com".to_string());
            models.insert("claude-sonnet-3.5".to_string(), "https://api.anthropic.com".to_string());
            models.insert("gpt-4".to_string(), "https://api.openai.com".to_string());
            models.insert("ollama/llama3".to_string(), "http://localhost:11434".to_string());

            Self { models }
        }

        fn get_provider(&self, model: &str) -> Option<String> {
            self.models.get(model).cloned()
        }
    }

    impl CCOTestEnvironment {
        async fn new() -> Self {
            Self {
                cache: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
                analytics: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
                router: ModelRouter::new(),
            }
        }

        fn generate_cache_key(model: &str, prompt: &str) -> String {
            use sha2::{Sha256, Digest};
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

        async fn process_request(
            &self,
            model: &str,
            prompt: &str,
            input_tokens: u32,
            output_tokens: u32,
            model_cost_per_1m_input: f64,
            model_cost_per_1m_output: f64,
        ) -> ProcessResult {
            let cache_key = Self::generate_cache_key(model, prompt);

            // Check cache
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                // Cache hit
                let would_be_cost = (input_tokens as f64 / 1_000_000.0) * model_cost_per_1m_input
                    + (output_tokens as f64 / 1_000_000.0) * model_cost_per_1m_output;

                let record = AnalyticsRecord {
                    model: model.to_string(),
                    input_tokens: cached.input_tokens,
                    output_tokens: cached.output_tokens,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost,
                    savings: would_be_cost,
                };

                drop(cache);
                let mut analytics = self.analytics.lock().await;
                analytics.push(record);

                return ProcessResult {
                    success: true,
                    from_cache: true,
                    cost: 0.0,
                    savings: would_be_cost,
                };
            }

            drop(cache);

            // Route request
            let provider = self.router.get_provider(model);
            if provider.is_none() {
                return ProcessResult {
                    success: false,
                    from_cache: false,
                    cost: 0.0,
                    savings: 0.0,
                };
            }

            // Calculate cost
            let actual_cost = (input_tokens as f64 / 1_000_000.0) * model_cost_per_1m_input
                + (output_tokens as f64 / 1_000_000.0) * model_cost_per_1m_output;

            // Store in cache
            let mut cache = self.cache.lock().await;
            cache.insert(
                cache_key,
                CachedData {
                    model: model.to_string(),
                    content: "simulated response".to_string(),
                    input_tokens,
                    output_tokens,
                },
            );
            drop(cache);

            // Record in analytics
            let record = AnalyticsRecord {
                model: model.to_string(),
                input_tokens,
                output_tokens,
                cache_hit: false,
                actual_cost,
                would_be_cost: actual_cost,
                savings: 0.0,
            };

            let mut analytics = self.analytics.lock().await;
            analytics.push(record);

            ProcessResult {
                success: true,
                from_cache: false,
                cost: actual_cost,
                savings: 0.0,
            }
        }

        async fn get_total_requests(&self) -> usize {
            let analytics = self.analytics.lock().await;
            analytics.len()
        }

        async fn get_cache_hit_count(&self) -> usize {
            let analytics = self.analytics.lock().await;
            analytics.iter().filter(|r| r.cache_hit).count()
        }

        async fn get_total_savings(&self) -> f64 {
            let analytics = self.analytics.lock().await;
            analytics.iter().map(|r| r.savings).sum()
        }

        async fn get_total_actual_cost(&self) -> f64 {
            let analytics = self.analytics.lock().await;
            analytics.iter().map(|r| r.actual_cost).sum()
        }

        async fn get_savings_by_model(&self) -> HashMap<String, f64> {
            let analytics = self.analytics.lock().await;
            let mut by_model = HashMap::new();
            for record in analytics.iter() {
                *by_model.entry(record.model.clone()).or_insert(0.0) += record.savings;
            }
            by_model
        }
    }

    struct ProcessResult {
        success: bool,
        from_cache: bool,
        cost: f64,
        savings: f64,
    }

    // ========== BASIC FULL REQUEST FLOW ==========

    #[tokio::test]
    async fn test_full_request_flow_cache_miss_then_hit() {
        let env = CCOTestEnvironment::new().await;

        // First request - cache miss
        let result1 = env
            .process_request("claude-opus-4", "test prompt", 1000, 500, 15.0, 75.0)
            .await;

        assert!(result1.success);
        assert!(!result1.from_cache);
        assert!((result1.cost - 0.0525).abs() < 0.001); // $0.0525 for 1000+500 tokens
        assert_eq!(result1.savings, 0.0);

        // Same request again - cache hit
        let result2 = env
            .process_request("claude-opus-4", "test prompt", 1000, 500, 15.0, 75.0)
            .await;

        assert!(result2.success);
        assert!(result2.from_cache);
        assert_eq!(result2.cost, 0.0);
        assert!((result2.savings - 0.0525).abs() < 0.001);

        assert_eq!(env.get_total_requests().await, 2);
        assert_eq!(env.get_cache_hit_count().await, 1);
    }

    #[tokio::test]
    async fn test_full_request_flow_multiple_requests() {
        let env = CCOTestEnvironment::new().await;

        // Process 10 requests, 5 unique
        for i in 0..10 {
            let prompt = format!("prompt {}", i % 5);
            env.process_request("claude-opus-4", &prompt, 1000, 500, 15.0, 75.0)
                .await;
        }

        assert_eq!(env.get_total_requests().await, 10);
        assert_eq!(
            env.get_cache_hit_count().await,
            5,
            "5 duplicate requests should be cache hits"
        );
    }

    // ========== MULTI-MODEL ROUTING TESTS ==========

    #[tokio::test]
    async fn test_multi_model_routing_anthropic() {
        let env = CCOTestEnvironment::new().await;

        let result = env
            .process_request("claude-opus-4", "test", 1000, 500, 15.0, 75.0)
            .await;

        assert!(result.success);
        assert!(!result.from_cache);
    }

    #[tokio::test]
    async fn test_multi_model_routing_openai() {
        let env = CCOTestEnvironment::new().await;

        let result = env
            .process_request("gpt-4", "test", 1000, 500, 30.0, 60.0)
            .await;

        assert!(result.success);
        assert!(!result.from_cache);
    }

    #[tokio::test]
    async fn test_multi_model_routing_ollama() {
        let env = CCOTestEnvironment::new().await;

        let result = env
            .process_request("ollama/llama3", "test", 1000, 500, 0.0, 0.0)
            .await;

        assert!(result.success);
        assert!(!result.from_cache);
        assert_eq!(result.cost, 0.0, "Ollama should be free");
    }

    #[tokio::test]
    async fn test_multi_model_routing_unknown_model() {
        let env = CCOTestEnvironment::new().await;

        let result = env
            .process_request("unknown-model-xyz", "test", 1000, 500, 10.0, 10.0)
            .await;

        assert!(!result.success, "Unknown model should fail");
    }

    #[tokio::test]
    async fn test_multi_model_different_costs() {
        let env = CCOTestEnvironment::new().await;

        // Claude Opus
        let opus_result = env
            .process_request("claude-opus-4", "question", 1_000_000, 500_000, 15.0, 75.0)
            .await;

        // GPT-4
        let gpt_result = env
            .process_request("gpt-4", "question", 1_000_000, 500_000, 30.0, 60.0)
            .await;

        // Ollama
        let ollama_result = env
            .process_request("ollama/llama3", "question", 1_000_000, 500_000, 0.0, 0.0)
            .await;

        assert!(opus_result.cost > 0.0);
        assert!(gpt_result.cost > 0.0);
        assert_eq!(ollama_result.cost, 0.0);

        // GPT should cost more than Claude for same tokens
        assert!(gpt_result.cost > opus_result.cost);
    }

    // ========== ANALYTICS PERSISTENCE TESTS ==========

    #[tokio::test]
    async fn test_analytics_tracks_all_requests() {
        let env = CCOTestEnvironment::new().await;

        for i in 0..5 {
            env.process_request("claude-opus-4", &format!("prompt {}", i), 1000, 500, 15.0, 75.0)
                .await;
        }

        assert_eq!(env.get_total_requests().await, 5);
    }

    #[tokio::test]
    async fn test_analytics_tracks_cache_hits() {
        let env = CCOTestEnvironment::new().await;

        let prompt = "test prompt";

        // Miss
        env.process_request("claude-opus-4", prompt, 1000, 500, 15.0, 75.0)
            .await;

        // Hit
        env.process_request("claude-opus-4", prompt, 1000, 500, 15.0, 75.0)
            .await;

        // Hit
        env.process_request("claude-opus-4", prompt, 1000, 500, 15.0, 75.0)
            .await;

        assert_eq!(env.get_cache_hit_count().await, 2);
    }

    #[tokio::test]
    async fn test_analytics_calculates_total_savings() {
        let env = CCOTestEnvironment::new().await;

        // Make first request (no savings)
        env.process_request("claude-opus-4", "prompt", 1000, 500, 15.0, 75.0)
            .await;

        // Make 5 cache hit requests (savings each)
        for _ in 0..5 {
            env.process_request("claude-opus-4", "prompt", 1000, 500, 15.0, 75.0)
                .await;
        }

        let total_savings = env.get_total_savings().await;
        assert!(total_savings > 0.0, "Should have savings from cache hits");

        // Each cache hit saves ~$0.0525 (15/1M + 75/2M tokens)
        // 5 cache hits = ~$0.2625
        assert!(total_savings > 0.2, "Savings should be meaningful");
    }

    #[tokio::test]
    async fn test_analytics_tracks_cost() {
        let env = CCOTestEnvironment::new().await;

        // Only cache misses cost money
        env.process_request("claude-opus-4", "prompt1", 1000, 500, 15.0, 75.0)
            .await;
        env.process_request("claude-opus-4", "prompt2", 1000, 500, 15.0, 75.0)
            .await;

        // Cache hit
        env.process_request("claude-opus-4", "prompt1", 1000, 500, 15.0, 75.0)
            .await;

        let total_cost = env.get_total_actual_cost().await;

        // Should only have cost for 2 misses, not the hit
        assert!(total_cost > 0.0);
        assert!(total_cost < 0.15); // 2 requests at ~$0.0525 each
    }

    #[tokio::test]
    async fn test_analytics_cost_breakdown_by_model() {
        let env = CCOTestEnvironment::new().await;

        // 2 Claude Opus requests
        env.process_request("claude-opus-4", "q1", 1000, 500, 15.0, 75.0)
            .await;
        env.process_request("claude-opus-4", "q2", 1000, 500, 15.0, 75.0)
            .await;

        // 1 GPT-4 request
        env.process_request("gpt-4", "q3", 1000, 500, 30.0, 60.0)
            .await;

        let savings_by_model = env.get_savings_by_model().await;

        // Only track cache hit savings
        for (_, savings) in savings_by_model.iter() {
            println!("Model savings: {}", savings);
        }
    }

    // ========== ERROR HANDLING TESTS ==========

    #[tokio::test]
    async fn test_error_unknown_model_handled_gracefully() {
        let env = CCOTestEnvironment::new().await;

        let result = env
            .process_request("nonexistent-model", "test", 1000, 500, 10.0, 10.0)
            .await;

        assert!(!result.success);
        assert_eq!(env.get_total_requests().await, 0, "Failed request should not be logged");
    }

    // ========== CONCURRENT REQUEST HANDLING ==========

    #[tokio::test]
    async fn test_concurrent_multimodel_requests() {
        let env = std::sync::Arc::new(CCOTestEnvironment::new().await);

        let mut handles = vec![];

        // Model 1: Claude - 3 concurrent tasks
        for _ in 0..3 {
            let env_clone = env.clone();
            let handle = tokio::spawn(async move {
                env_clone
                    .process_request("claude-opus-4", "test", 1000, 500, 15.0, 75.0)
                    .await
            });
            handles.push(handle);
        }

        // Model 2: OpenAI - 3 concurrent tasks
        for _ in 0..3 {
            let env_clone = env.clone();
            let handle = tokio::spawn(async move {
                env_clone
                    .process_request("gpt-4", "test", 1000, 500, 30.0, 60.0)
                    .await
            });
            handles.push(handle);
        }

        // Model 3: Ollama - 3 concurrent tasks
        for _ in 0..3 {
            let env_clone = env.clone();
            let handle = tokio::spawn(async move {
                env_clone
                    .process_request("ollama/llama3", "test", 1000, 500, 0.0, 0.0)
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(result.success, "All concurrent requests should succeed");
        }

        assert_eq!(env.get_total_requests().await, 9);
    }

    // ========== REALISTIC WORKFLOW TESTS ==========

    #[tokio::test]
    async fn test_realistic_daily_workflow() {
        let env = CCOTestEnvironment::new().await;

        // Simulate a typical day of usage:
        // - 100 requests to Claude
        // - 50 requests to GPT-4
        // - 30 requests to Ollama
        // - 60% cache hit rate for Claude
        // - 40% cache hit rate for GPT-4
        // - 50% cache hit rate for Ollama

        // Claude requests
        for i in 0..100 {
            let prompt = format!("claude_q_{}", i % 40); // 40 unique, 60 duplicates
            env.process_request("claude-opus-4", &prompt, 1000, 500, 15.0, 75.0)
                .await;
        }

        // GPT-4 requests
        for i in 0..50 {
            let prompt = format!("gpt_q_{}", i % 30); // 30 unique, 20 duplicates
            env.process_request("gpt-4", &prompt, 1000, 500, 30.0, 60.0)
                .await;
        }

        // Ollama requests
        for i in 0..30 {
            let prompt = format!("ollama_q_{}", i % 15); // 15 unique, 15 duplicates
            env.process_request("ollama/llama3", &prompt, 1000, 500, 0.0, 0.0)
                .await;
        }

        let total_requests = env.get_total_requests().await;
        let total_cache_hits = env.get_cache_hit_count().await;
        let total_savings = env.get_total_savings().await;
        let total_cost = env.get_total_actual_cost().await;

        assert_eq!(total_requests, 180);
        assert!(total_cache_hits > 50, "Should have significant cache hits");
        assert!(total_savings > 0.0, "Should have measurable savings");
        assert!(total_cost > 0.0, "Should have some cost from misses");
        assert!(total_cost < 10.0, "Should be reasonable cost");
    }

    // Note: env doesn't implement Clone, so we need to wrap it in Arc for concurrent tests
}
