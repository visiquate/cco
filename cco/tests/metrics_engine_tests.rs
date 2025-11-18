//! Metrics Engine Tests for Phase 1a
//!
//! This module tests the metrics aggregation and cost calculation engine
//! following TDD principles - tests written BEFORE implementation.
//!
//! ## Test Coverage Areas:
//! - Token aggregation (input, output, cache types)
//! - Cost calculations per model tier
//! - Event recording and retrieval
//! - Buffer management (overflow handling)
//! - Concurrent access (Arc<Mutex<>>)
//! - Summary generation

#[cfg(test)]
mod metrics_engine_tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // ===== Mock Types for Testing (to be replaced with actual implementation) =====

    #[derive(Clone, Debug, PartialEq)]
    pub enum TokenType {
        Input,
        Output,
        CachedInput,
        CachedOutput,
    }

    #[derive(Clone, Debug)]
    pub struct TokenMetrics {
        pub input_tokens: u64,
        pub output_tokens: u64,
        pub cached_input_tokens: u64,
        pub cached_output_tokens: u64,
    }

    #[derive(Clone, Debug)]
    pub struct CostMetrics {
        pub total_cost: f64,
        pub input_cost: f64,
        pub output_cost: f64,
        pub cached_input_cost: f64,
        pub cached_output_cost: f64,
    }

    #[derive(Clone, Debug)]
    pub struct MetricsEvent {
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub model: String,
        pub input_tokens: u64,
        pub output_tokens: u64,
        pub cached_input_tokens: u64,
        pub cached_output_tokens: u64,
        pub cost: f64,
    }

    #[derive(Clone, Debug)]
    pub struct MetricsSummary {
        pub total_requests: u64,
        pub total_tokens: u64,
        pub total_cost: f64,
        pub by_model: HashMap<String, ModelSummary>,
    }

    #[derive(Clone, Debug)]
    pub struct ModelSummary {
        pub requests: u64,
        pub tokens: TokenMetrics,
        pub cost: CostMetrics,
    }

    /// Mock MetricsEngine - to be replaced with actual implementation
    pub struct MetricsEngine {
        events: Arc<Mutex<Vec<MetricsEvent>>>,
        max_buffer_size: usize,
    }

    impl MetricsEngine {
        pub fn new(max_buffer_size: usize) -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::with_capacity(max_buffer_size))),
                max_buffer_size,
            }
        }

        pub async fn record_event(
            &self,
            model: String,
            input_tokens: u64,
            output_tokens: u64,
            cached_input_tokens: u64,
            cached_output_tokens: u64,
        ) -> Result<(), String> {
            let mut events = self.events.lock().await;

            // Buffer overflow check
            if events.len() >= self.max_buffer_size {
                return Err("Buffer overflow: max capacity reached".to_string());
            }

            let cost = self.calculate_cost(
                &model,
                input_tokens,
                output_tokens,
                cached_input_tokens,
                cached_output_tokens,
            );

            events.push(MetricsEvent {
                timestamp: chrono::Utc::now(),
                model,
                input_tokens,
                output_tokens,
                cached_input_tokens,
                cached_output_tokens,
                cost,
            });

            Ok(())
        }

        pub async fn get_events(&self, limit: usize) -> Vec<MetricsEvent> {
            let events = self.events.lock().await;
            events.iter().rev().take(limit).cloned().collect()
        }

        pub async fn get_summary(&self) -> MetricsSummary {
            let events = self.events.lock().await;

            let mut by_model: HashMap<String, ModelSummary> = HashMap::new();
            let mut total_requests = 0u64;
            let mut total_tokens = 0u64;
            let mut total_cost = 0f64;

            for event in events.iter() {
                total_requests += 1;
                let event_tokens = event.input_tokens + event.output_tokens
                    + event.cached_input_tokens + event.cached_output_tokens;
                total_tokens += event_tokens;
                total_cost += event.cost;

                let summary = by_model
                    .entry(event.model.clone())
                    .or_insert_with(|| ModelSummary {
                        requests: 0,
                        tokens: TokenMetrics {
                            input_tokens: 0,
                            output_tokens: 0,
                            cached_input_tokens: 0,
                            cached_output_tokens: 0,
                        },
                        cost: CostMetrics {
                            total_cost: 0.0,
                            input_cost: 0.0,
                            output_cost: 0.0,
                            cached_input_cost: 0.0,
                            cached_output_cost: 0.0,
                        },
                    });

                summary.requests += 1;
                summary.tokens.input_tokens += event.input_tokens;
                summary.tokens.output_tokens += event.output_tokens;
                summary.tokens.cached_input_tokens += event.cached_input_tokens;
                summary.tokens.cached_output_tokens += event.cached_output_tokens;
                summary.cost.total_cost += event.cost;
            }

            MetricsSummary {
                total_requests,
                total_tokens,
                total_cost,
                by_model,
            }
        }

        pub async fn clear(&self) {
            let mut events = self.events.lock().await;
            events.clear();
        }

        fn calculate_cost(
            &self,
            model: &str,
            input_tokens: u64,
            output_tokens: u64,
            cached_input_tokens: u64,
            cached_output_tokens: u64,
        ) -> f64 {
            // Pricing per 1M tokens (as of 2025)
            let (input_price, output_price, cached_input_price, cached_output_price) =
                match model {
                    "claude-opus-4" => (15.0, 75.0, 1.5, 7.5),
                    "claude-sonnet-4.5" => (3.0, 15.0, 0.3, 1.5),
                    "claude-haiku-4.5" => (0.8, 4.0, 0.08, 0.4),
                    _ => (0.0, 0.0, 0.0, 0.0),
                };

            let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;
            let cached_input_cost = (cached_input_tokens as f64 / 1_000_000.0) * cached_input_price;
            let cached_output_cost = (cached_output_tokens as f64 / 1_000_000.0) * cached_output_price;

            input_cost + output_cost + cached_input_cost + cached_output_cost
        }
    }

    // ===== TEST SUITE =====

    // Test 1: Basic Token Aggregation
    #[tokio::test]
    async fn test_token_aggregation() {
        let engine = MetricsEngine::new(1000);

        engine
            .record_event("claude-opus-4".to_string(), 1000, 500, 200, 100)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.total_tokens, 1800); // 1000+500+200+100
    }

    // Test 2: Multiple Token Types
    #[tokio::test]
    async fn test_multiple_token_types() {
        let engine = MetricsEngine::new(1000);

        engine
            .record_event("claude-opus-4".to_string(), 1000, 500, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-opus-4".to_string(), 0, 0, 800, 400)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        let opus_summary = summary.by_model.get("claude-opus-4").unwrap();

        assert_eq!(opus_summary.tokens.input_tokens, 1000);
        assert_eq!(opus_summary.tokens.output_tokens, 500);
        assert_eq!(opus_summary.tokens.cached_input_tokens, 800);
        assert_eq!(opus_summary.tokens.cached_output_tokens, 400);
    }

    // Test 3: Cost Calculation - Opus
    #[tokio::test]
    async fn test_cost_calculation_opus() {
        let engine = MetricsEngine::new(1000);

        // 1M input + 1M output = $15 + $75 = $90
        engine
            .record_event("claude-opus-4".to_string(), 1_000_000, 1_000_000, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert!((summary.total_cost - 90.0).abs() < 0.01);
    }

    // Test 4: Cost Calculation - Sonnet
    #[tokio::test]
    async fn test_cost_calculation_sonnet() {
        let engine = MetricsEngine::new(1000);

        // 1M input + 1M output = $3 + $15 = $18
        engine
            .record_event("claude-sonnet-4.5".to_string(), 1_000_000, 1_000_000, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert!((summary.total_cost - 18.0).abs() < 0.01);
    }

    // Test 5: Cost Calculation - Haiku
    #[tokio::test]
    async fn test_cost_calculation_haiku() {
        let engine = MetricsEngine::new(1000);

        // 1M input + 1M output = $0.8 + $4.0 = $4.8
        engine
            .record_event("claude-haiku-4.5".to_string(), 1_000_000, 1_000_000, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert!((summary.total_cost - 4.8).abs() < 0.01);
    }

    // Test 6: Cached Token Pricing (90% discount)
    #[tokio::test]
    async fn test_cached_token_pricing() {
        let engine = MetricsEngine::new(1000);

        // Regular: 1M input = $15, 1M output = $75 -> Total = $90
        // Cached: 1M input = $1.5, 1M output = $7.5 -> Total = $9
        engine
            .record_event("claude-opus-4".to_string(), 0, 0, 1_000_000, 1_000_000)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert!((summary.total_cost - 9.0).abs() < 0.01);
    }

    // Test 7: Event Recording and Retrieval
    #[tokio::test]
    async fn test_event_recording_and_retrieval() {
        let engine = MetricsEngine::new(1000);

        engine
            .record_event("claude-opus-4".to_string(), 1000, 500, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-sonnet-4.5".to_string(), 2000, 1000, 0, 0)
            .await
            .unwrap();

        let events = engine.get_events(10).await;
        assert_eq!(events.len(), 2);

        // Most recent first
        assert_eq!(events[0].model, "claude-sonnet-4.5");
        assert_eq!(events[1].model, "claude-opus-4");
    }

    // Test 8: Buffer Overflow Handling
    #[tokio::test]
    async fn test_buffer_overflow_handling() {
        let engine = MetricsEngine::new(5); // Small buffer

        // Fill buffer
        for i in 0..5 {
            engine
                .record_event(format!("model-{}", i), 1000, 500, 0, 0)
                .await
                .unwrap();
        }

        // Attempt to overflow
        let result = engine
            .record_event("overflow-model".to_string(), 1000, 500, 0, 0)
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Buffer overflow: max capacity reached"
        );
    }

    // Test 9: Concurrent Access (Race Condition Test)
    #[tokio::test]
    async fn test_concurrent_access() {
        let engine = Arc::new(MetricsEngine::new(1000));
        let mut handles = vec![];

        // Spawn 10 concurrent tasks recording events
        for i in 0..10 {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    engine_clone
                        .record_event(
                            format!("model-{}", i),
                            j * 100,
                            j * 50,
                            0,
                            0,
                        )
                        .await
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let summary = engine.get_summary().await;
        assert_eq!(summary.total_requests, 100); // 10 tasks * 10 events each
    }

    // Test 10: Per-Model Aggregation
    #[tokio::test]
    async fn test_per_model_aggregation() {
        let engine = MetricsEngine::new(1000);

        // Record for 3 different models
        engine
            .record_event("claude-opus-4".to_string(), 1000, 500, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-opus-4".to_string(), 2000, 1000, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-sonnet-4.5".to_string(), 1500, 750, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-haiku-4.5".to_string(), 800, 400, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert_eq!(summary.by_model.len(), 3);

        let opus = summary.by_model.get("claude-opus-4").unwrap();
        assert_eq!(opus.requests, 2);
        assert_eq!(opus.tokens.input_tokens, 3000); // 1000 + 2000

        let sonnet = summary.by_model.get("claude-sonnet-4.5").unwrap();
        assert_eq!(sonnet.requests, 1);

        let haiku = summary.by_model.get("claude-haiku-4.5").unwrap();
        assert_eq!(haiku.requests, 1);
    }

    // Test 11: Summary Generation Accuracy
    #[tokio::test]
    async fn test_summary_generation_accuracy() {
        let engine = MetricsEngine::new(1000);

        // Record various events
        engine
            .record_event("claude-opus-4".to_string(), 1_000_000, 500_000, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-sonnet-4.5".to_string(), 2_000_000, 1_000_000, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;

        // Opus: $15 * 1 + $75 * 0.5 = $52.5
        // Sonnet: $3 * 2 + $15 * 1 = $21
        // Total: $73.5
        assert!((summary.total_cost - 73.5).abs() < 0.1);
        assert_eq!(summary.total_requests, 2);
    }

    // Test 12: Clear Functionality
    #[tokio::test]
    async fn test_clear_metrics() {
        let engine = MetricsEngine::new(1000);

        engine
            .record_event("claude-opus-4".to_string(), 1000, 500, 0, 0)
            .await
            .unwrap();
        engine
            .record_event("claude-sonnet-4.5".to_string(), 2000, 1000, 0, 0)
            .await
            .unwrap();

        let summary_before = engine.get_summary().await;
        assert_eq!(summary_before.total_requests, 2);

        engine.clear().await;

        let summary_after = engine.get_summary().await;
        assert_eq!(summary_after.total_requests, 0);
        assert_eq!(summary_after.total_cost, 0.0);
    }

    // Test 13: Mixed Regular and Cached Tokens
    #[tokio::test]
    async fn test_mixed_regular_and_cached_tokens() {
        let engine = MetricsEngine::new(1000);

        // Half regular, half cached
        engine
            .record_event(
                "claude-opus-4".to_string(),
                500_000, // regular input
                250_000, // regular output
                500_000, // cached input
                250_000, // cached output
            )
            .await
            .unwrap();

        let summary = engine.get_summary().await;

        // Regular: 0.5M * $15 + 0.25M * $75 = $7.5 + $18.75 = $26.25
        // Cached: 0.5M * $1.5 + 0.25M * $7.5 = $0.75 + $1.875 = $2.625
        // Total: $28.875
        assert!((summary.total_cost - 28.875).abs() < 0.01);
    }

    // Test 14: Zero Token Handling
    #[tokio::test]
    async fn test_zero_token_handling() {
        let engine = MetricsEngine::new(1000);

        engine
            .record_event("claude-opus-4".to_string(), 0, 0, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;
        assert_eq!(summary.total_tokens, 0);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_requests, 1);
    }

    // Test 15: Large Token Volumes
    #[tokio::test]
    async fn test_large_token_volumes() {
        let engine = MetricsEngine::new(1000);

        // 100M tokens (extreme case)
        engine
            .record_event("claude-opus-4".to_string(), 50_000_000, 50_000_000, 0, 0)
            .await
            .unwrap();

        let summary = engine.get_summary().await;

        // 50M * $15 + 50M * $75 = $750 + $3750 = $4500
        assert!((summary.total_cost - 4500.0).abs() < 1.0);
    }

    // Test 16: Event Limit Retrieval
    #[tokio::test]
    async fn test_event_limit_retrieval() {
        let engine = MetricsEngine::new(1000);

        // Record 20 events
        for i in 0..20 {
            engine
                .record_event(format!("model-{}", i), 1000, 500, 0, 0)
                .await
                .unwrap();
        }

        // Get only last 5
        let events = engine.get_events(5).await;
        assert_eq!(events.len(), 5);

        // Should be models 19, 18, 17, 16, 15 (most recent first)
        assert_eq!(events[0].model, "model-19");
        assert_eq!(events[4].model, "model-15");
    }

    // Test 17: Timestamp Ordering
    #[tokio::test]
    async fn test_timestamp_ordering() {
        let engine = MetricsEngine::new(1000);

        for i in 0..5 {
            engine
                .record_event(format!("model-{}", i), 1000, 500, 0, 0)
                .await
                .unwrap();

            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let events = engine.get_events(5).await;

        // Events should be in reverse chronological order (newest first)
        for i in 0..events.len() - 1 {
            assert!(events[i].timestamp >= events[i + 1].timestamp);
        }
    }
}
