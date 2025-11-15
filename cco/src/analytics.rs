//! Analytics module for tracking API calls and cache performance

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A single API call record
#[derive(Clone, Debug)]
pub struct ApiCallRecord {
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_hit: bool,
    pub actual_cost: f64,
    pub would_be_cost: f64,
    pub savings: f64,
}

/// Per-model metrics
#[derive(Clone, Debug)]
pub struct ModelMetrics {
    pub model: String,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub actual_cost: f64,
    pub would_be_cost: f64,
    pub total_savings: f64,
}

/// Analytics engine for tracking cache performance
pub struct AnalyticsEngine {
    records: Arc<Mutex<Vec<ApiCallRecord>>>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record an API call
    pub async fn record_api_call(&self, record: ApiCallRecord) {
        let mut records = self.records.lock().await;
        records.push(record);
    }

    /// Get total number of requests
    pub async fn get_total_requests(&self) -> u64 {
        let records = self.records.lock().await;
        records.len() as u64
    }

    /// Get number of cache hits
    pub async fn get_cache_hits(&self) -> u64 {
        let records = self.records.lock().await;
        records.iter().filter(|r| r.cache_hit).count() as u64
    }

    /// Get number of cache misses
    pub async fn get_cache_misses(&self) -> u64 {
        let records = self.records.lock().await;
        records.iter().filter(|r| !r.cache_hit).count() as u64
    }

    /// Get cache hit rate as percentage
    pub async fn get_hit_rate(&self) -> f64 {
        let records = self.records.lock().await;
        let total = records.len();
        if total == 0 {
            0.0
        } else {
            let hits = records.iter().filter(|r| r.cache_hit).count();
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Get total savings in dollars
    pub async fn get_total_savings(&self) -> f64 {
        let records = self.records.lock().await;
        records.iter().map(|r| r.savings).sum()
    }

    /// Get total actual cost
    pub async fn get_total_actual_cost(&self) -> f64 {
        let records = self.records.lock().await;
        records.iter().map(|r| r.actual_cost).sum()
    }

    /// Get total would-be cost
    pub async fn get_total_would_be_cost(&self) -> f64 {
        let records = self.records.lock().await;
        records.iter().map(|r| r.would_be_cost).sum()
    }

    /// Get savings breakdown by model
    pub async fn get_savings_by_model(&self) -> HashMap<String, f64> {
        let records = self.records.lock().await;
        let mut by_model = HashMap::new();
        for record in records.iter() {
            *by_model.entry(record.model.clone()).or_insert(0.0) += record.savings;
        }
        by_model
    }

    /// Get comprehensive metrics by model
    pub async fn get_metrics_by_model(&self) -> HashMap<String, ModelMetrics> {
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

    /// Clear all records
    pub async fn clear(&self) {
        let mut records = self.records.lock().await;
        records.clear();
    }
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!((savings - 52.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_cache_hit_rate_calculation() {
        let analytics = AnalyticsEngine::new();

        // 7 cache hits
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

        // 3 cache misses
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
        assert!((hit_rate - 70.0).abs() < 0.1);
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
        assert!((savings - 525.0).abs() < 0.1);
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

        let savings_by_model = analytics.get_savings_by_model().await;
        assert_eq!(savings_by_model.len(), 2);
        assert!((savings_by_model.get("claude-opus-4").unwrap() - 262.5).abs() < 0.1);
        assert!((savings_by_model.get("claude-sonnet-3.5").unwrap() - 31.5).abs() < 0.1);
    }

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
