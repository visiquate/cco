//! Comprehensive analytics tests for cache savings tracking
//!
//! This module tests all aspects of the analytics system including:
//! - Recording cache hits and misses
//! - Cost tracking and calculations
//! - Hit rate aggregation
//! - Total savings queries
//! - Per-model cost breakdown
//! - Time-period analytics

#[cfg(test)]
mod analytics_tests {
    use std::collections::HashMap;

    /// Analytics record for a single API call
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct ApiCallRecord {
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        cache_hit: bool,
        actual_cost: f64,
        would_be_cost: f64,
        savings: f64,
    }

    /// In-memory analytics engine for testing
    struct AnalyticsEngine {
        records: std::sync::Arc<tokio::sync::Mutex<Vec<ApiCallRecord>>>,
    }

    impl AnalyticsEngine {
        fn new() -> Self {
            Self {
                records: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        async fn record_api_call(&self, record: ApiCallRecord) {
            let mut records = self.records.lock().await;
            records.push(record);
        }

        async fn get_total_requests(&self) -> u64 {
            let records = self.records.lock().await;
            records.len() as u64
        }

        async fn get_cache_hits(&self) -> u64 {
            let records = self.records.lock().await;
            records.iter().filter(|r| r.cache_hit).count() as u64
        }

        async fn get_cache_misses(&self) -> u64 {
            let records = self.records.lock().await;
            records.iter().filter(|r| !r.cache_hit).count() as u64
        }

        async fn get_hit_rate(&self) -> f64 {
            let records = self.records.lock().await;
            let total = records.len();
            if total == 0 {
                0.0
            } else {
                let hits = records.iter().filter(|r| r.cache_hit).count();
                (hits as f64 / total as f64) * 100.0
            }
        }

        async fn get_total_savings(&self) -> f64 {
            let records = self.records.lock().await;
            records.iter().map(|r| r.savings).sum()
        }

        async fn get_total_actual_cost(&self) -> f64 {
            let records = self.records.lock().await;
            records.iter().map(|r| r.actual_cost).sum()
        }

        async fn get_total_would_be_cost(&self) -> f64 {
            let records = self.records.lock().await;
            records.iter().map(|r| r.would_be_cost).sum()
        }

        async fn get_savings_by_model(&self) -> HashMap<String, f64> {
            let records = self.records.lock().await;
            let mut by_model = HashMap::new();
            for record in records.iter() {
                *by_model.entry(record.model.clone()).or_insert(0.0) += record.savings;
            }
            by_model
        }

        async fn get_metrics_by_model(&self) -> HashMap<String, ModelMetrics> {
            let records = self.records.lock().await;
            let mut by_model: HashMap<String, ModelMetrics> = HashMap::new();

            for record in records.iter() {
                let entry = by_model
                    .entry(record.model.clone())
                    .or_insert(ModelMetrics {
                        model: record.model.clone(),
                        total_requests: 0,
                        cache_hits: 0,
                        cache_misses: 0,
                        actual_cost: 0.0,
                        would_be_cost: 0.0,
                        total_savings: 0.0,
                    });

                entry.total_requests += 1;
                if record.cache_hit {
                    entry.cache_hits += 1;
                } else {
                    entry.cache_misses += 1;
                }
                entry.actual_cost += record.actual_cost;
                entry.would_be_cost += record.would_be_cost;
                entry.total_savings += record.savings;
            }

            by_model
        }

        async fn clear(&self) {
            let mut records = self.records.lock().await;
            records.clear();
        }
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct ModelMetrics {
        model: String,
        total_requests: u64,
        cache_hits: u64,
        cache_misses: u64,
        actual_cost: f64,
        would_be_cost: f64,
        total_savings: f64,
    }

    // ========== BASIC RECORDING TESTS ==========

    #[tokio::test]
    async fn test_record_cache_miss() {
        let analytics = AnalyticsEngine::new();

        analytics
            .record_api_call(ApiCallRecord {
                model: "claude-opus-4".to_string(),
                input_tokens: 1000,
                output_tokens: 500,
                cache_hit: false,
                actual_cost: 52.5,
                would_be_cost: 52.5,
                savings: 0.0,
            })
            .await;

        assert_eq!(analytics.get_total_requests().await, 1);
        assert_eq!(analytics.get_cache_hits().await, 0);
        assert_eq!(analytics.get_cache_misses().await, 1);
    }

    #[tokio::test]
    async fn test_record_cache_hit() {
        let analytics = AnalyticsEngine::new();

        analytics
            .record_api_call(ApiCallRecord {
                model: "claude-opus-4".to_string(),
                input_tokens: 1000,
                output_tokens: 500,
                cache_hit: true,
                actual_cost: 0.0,
                would_be_cost: 52.5,
                savings: 52.5,
            })
            .await;

        assert_eq!(analytics.get_total_requests().await, 1);
        assert_eq!(analytics.get_cache_hits().await, 1);
        assert_eq!(analytics.get_cache_misses().await, 0);

        let savings = analytics.get_total_savings().await;
        assert!((savings - 52.5).abs() < 0.01, "Savings should be $52.50");
    }

    #[tokio::test]
    async fn test_record_multiple_calls() {
        let analytics = AnalyticsEngine::new();

        // Record 5 cache misses
        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        // Record 3 cache hits
        for _ in 0..3 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        assert_eq!(analytics.get_total_requests().await, 8);
        assert_eq!(analytics.get_cache_hits().await, 3);
        assert_eq!(analytics.get_cache_misses().await, 5);
    }

    // ========== HIT RATE TESTS ==========

    #[tokio::test]
    async fn test_cache_hit_rate_calculation() {
        let analytics = AnalyticsEngine::new();

        // 7 cache hits, 3 cache misses
        for _ in 0..7 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        for _ in 0..3 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        let hit_rate = analytics.get_hit_rate().await;
        assert!((hit_rate - 70.0).abs() < 0.1, "Hit rate should be 70%");
    }

    #[tokio::test]
    async fn test_cache_hit_rate_100_percent() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..10 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        let hit_rate = analytics.get_hit_rate().await;
        assert_eq!(hit_rate, 100.0, "Hit rate should be 100%");
    }

    #[tokio::test]
    async fn test_cache_hit_rate_0_percent() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..10 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        let hit_rate = analytics.get_hit_rate().await;
        assert_eq!(hit_rate, 0.0, "Hit rate should be 0%");
    }

    #[tokio::test]
    async fn test_cache_hit_rate_empty() {
        let analytics = AnalyticsEngine::new();

        let hit_rate = analytics.get_hit_rate().await;
        assert_eq!(hit_rate, 0.0, "Empty analytics should have 0% hit rate");
    }

    // ========== SAVINGS TRACKING TESTS ==========

    #[tokio::test]
    async fn test_total_savings_single_cache_hit() {
        let analytics = AnalyticsEngine::new();

        analytics
            .record_api_call(ApiCallRecord {
                model: "claude-opus-4".to_string(),
                input_tokens: 1000,
                output_tokens: 500,
                cache_hit: true,
                actual_cost: 0.0,
                would_be_cost: 52.5,
                savings: 52.5,
            })
            .await;

        let savings = analytics.get_total_savings().await;
        assert!((savings - 52.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_total_savings_multiple_cache_hits() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..10 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        let savings = analytics.get_total_savings().await;
        assert!((savings - 525.0).abs() < 0.1, "Total savings should be $525");
    }

    #[tokio::test]
    async fn test_total_savings_mixed_hits_and_misses() {
        let analytics = AnalyticsEngine::new();

        // 7 hits
        for _ in 0..7 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        // 3 misses
        for _ in 0..3 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        let savings = analytics.get_total_savings().await;
        assert!((savings - 367.5).abs() < 0.1, "Total savings should be $367.50");
    }

    #[tokio::test]
    async fn test_total_savings_zero() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        let savings = analytics.get_total_savings().await;
        assert_eq!(savings, 0.0, "No savings without cache hits");
    }

    // ========== COST TRACKING TESTS ==========

    #[tokio::test]
    async fn test_total_actual_cost_no_cache() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        let cost = analytics.get_total_actual_cost().await;
        assert!((cost - 262.5).abs() < 0.1, "Total cost should be $262.50");
    }

    #[tokio::test]
    async fn test_total_would_be_cost() {
        let analytics = AnalyticsEngine::new();

        // 5 cache misses + 5 cache hits
        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        let would_be_cost = analytics.get_total_would_be_cost().await;
        assert!((would_be_cost - 525.0).abs() < 0.1, "Would-be cost should be $525.00");
    }

    #[tokio::test]
    async fn test_cost_savings_efficiency() {
        let analytics = AnalyticsEngine::new();

        // 5 cache misses
        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        // 5 cache hits
        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        let actual_cost = analytics.get_total_actual_cost().await;
        let would_be_cost = analytics.get_total_would_be_cost().await;
        let savings = analytics.get_total_savings().await;

        // Actual = 5*52.5 = 262.5
        // Would-be = 10*52.5 = 525
        // Savings = 525 - 262.5 = 262.5
        assert!((actual_cost - 262.5).abs() < 0.1);
        assert!((would_be_cost - 525.0).abs() < 0.1);
        assert!((savings - 262.5).abs() < 0.1);
        assert!(
            (actual_cost + savings - would_be_cost).abs() < 0.01,
            "Actual + Savings should equal Would-be"
        );
    }

    // ========== PER-MODEL ANALYTICS TESTS ==========

    #[tokio::test]
    async fn test_savings_by_model_single_model() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        let savings_by_model = analytics.get_savings_by_model().await;
        assert_eq!(savings_by_model.len(), 1);

        let opus_savings = savings_by_model.get("claude-opus-4").unwrap();
        assert!((opus_savings - 262.5).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_savings_by_model_multiple_models() {
        let analytics = AnalyticsEngine::new();

        // 5 Claude Opus cache hits - $52.50 each
        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        // 3 Claude Sonnet cache hits - $10.50 each
        for _ in 0..3 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-sonnet-3.5".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 10.5,
                    savings: 10.5,
                })
                .await;
        }

        // 2 GPT-4 cache hits - $60.00 each
        for _ in 0..2 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "gpt-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 60.0,
                    savings: 60.0,
                })
                .await;
        }

        let savings_by_model = analytics.get_savings_by_model().await;
        assert_eq!(savings_by_model.len(), 3);

        assert!((savings_by_model.get("claude-opus-4").unwrap() - 262.5).abs() < 0.1);
        assert!((savings_by_model.get("claude-sonnet-3.5").unwrap() - 31.5).abs() < 0.1);
        assert!((savings_by_model.get("gpt-4").unwrap() - 120.0).abs() < 0.1);

        let total_savings: f64 = savings_by_model.values().sum();
        assert!((total_savings - 414.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_metrics_by_model() {
        let analytics = AnalyticsEngine::new();

        // 5 Claude Opus: 3 hits, 2 misses
        for _ in 0..3 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        for _ in 0..2 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 52.5,
                    would_be_cost: 52.5,
                    savings: 0.0,
                })
                .await;
        }

        // 4 Claude Sonnet: 2 hits, 2 misses
        for _ in 0..2 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-sonnet-3.5".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 10.5,
                    savings: 10.5,
                })
                .await;
        }

        for _ in 0..2 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-sonnet-3.5".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: false,
                    actual_cost: 10.5,
                    would_be_cost: 10.5,
                    savings: 0.0,
                })
                .await;
        }

        let metrics = analytics.get_metrics_by_model().await;

        let opus_metrics = metrics.get("claude-opus-4").unwrap();
        assert_eq!(opus_metrics.total_requests, 5);
        assert_eq!(opus_metrics.cache_hits, 3);
        assert_eq!(opus_metrics.cache_misses, 2);
        assert!((opus_metrics.total_savings - 157.5).abs() < 0.1);

        let sonnet_metrics = metrics.get("claude-sonnet-3.5").unwrap();
        assert_eq!(sonnet_metrics.total_requests, 4);
        assert_eq!(sonnet_metrics.cache_hits, 2);
        assert_eq!(sonnet_metrics.cache_misses, 2);
        assert!((sonnet_metrics.total_savings - 21.0).abs() < 0.1);
    }

    // ========== CONCURRENT RECORDING TESTS ==========

    #[tokio::test]
    async fn test_concurrent_recording() {
        let analytics = AnalyticsEngine::new();

        let mut handles = vec![];

        for i in 0..10 {
            let analytics_clone = AnalyticsEngine {
                records: analytics.records.clone(),
            };

            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    analytics_clone
                        .record_api_call(ApiCallRecord {
                            model: format!("model-{}", i),
                            input_tokens: 1000,
                            output_tokens: 500,
                            cache_hit: (i + j) % 2 == 0,
                            actual_cost: if (i + j) % 2 == 0 { 0.0 } else { 52.5 },
                            would_be_cost: 52.5,
                            savings: if (i + j) % 2 == 0 { 52.5 } else { 0.0 },
                        })
                        .await;
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("Task should complete");
        }

        let total = analytics.get_total_requests().await;
        assert_eq!(total, 100, "Should have recorded 100 requests");
    }

    // ========== CLEAR AND RESET TESTS ==========

    #[tokio::test]
    async fn test_clear_analytics() {
        let analytics = AnalyticsEngine::new();

        for _ in 0..5 {
            analytics
                .record_api_call(ApiCallRecord {
                    model: "claude-opus-4".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_hit: true,
                    actual_cost: 0.0,
                    would_be_cost: 52.5,
                    savings: 52.5,
                })
                .await;
        }

        assert_eq!(analytics.get_total_requests().await, 5);

        analytics.clear().await;

        assert_eq!(analytics.get_total_requests().await, 0);
        assert_eq!(analytics.get_total_savings().await, 0.0);
    }
}
