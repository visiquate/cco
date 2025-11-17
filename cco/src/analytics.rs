//! Analytics module for tracking API calls and cache performance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A single API call record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiCallRecord {
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_hit: bool,
    pub actual_cost: f64,
    pub would_be_cost: f64,
    pub savings: f64,
}

/// Activity event for tracking user actions and API activity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub timestamp: String,  // ISO 8601
    pub event_type: String, // "api_call", "error", "cache_hit", "cache_miss", "model_override"
    pub agent_name: Option<String>,
    pub model: Option<String>,
    pub tokens: Option<u64>,
    pub latency_ms: Option<u64>,
    pub status: Option<String>, // "success", "error", "pending"
    pub cost: Option<f64>,      // calculated cost for this event
}

/// Model override record for tracking transparent rewrites
#[derive(Clone, Debug, Serialize)]
pub struct OverrideRecord {
    pub original_model: String,
    pub override_to: String,
    pub timestamp: DateTime<Utc>,
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
    model_overrides: Arc<Mutex<Vec<OverrideRecord>>>,
    activity_events: Arc<Mutex<std::collections::VecDeque<ActivityEvent>>>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
            model_overrides: Arc::new(Mutex::new(Vec::new())),
            activity_events: Arc::new(Mutex::new(std::collections::VecDeque::with_capacity(100))),
        }
    }

    /// Record an API call
    pub async fn record_api_call(&self, record: ApiCallRecord) {
        let mut records = self.records.lock().await;
        records.push(record);
    }

    /// Record an activity event
    pub async fn record_event(&self, event: ActivityEvent) {
        let mut events = self.activity_events.lock().await;

        // Keep only the last 100 events using a ring buffer approach
        if events.len() >= 100 {
            events.pop_front();
        }

        events.push_back(event);
    }

    /// Get recent activity events (last N)
    pub async fn get_recent_activity(&self, limit: usize) -> Vec<ActivityEvent> {
        let events = self.activity_events.lock().await;
        let start = if events.len() > limit {
            events.len() - limit
        } else {
            0
        };
        events.iter().skip(start).cloned().collect()
    }

    /// Record a model override event
    pub async fn record_model_override(&self, original_model: &str, override_model: &str) {
        let record = OverrideRecord {
            original_model: original_model.to_string(),
            override_to: override_model.to_string(),
            timestamp: Utc::now(),
        };

        let mut overrides = self.model_overrides.lock().await;
        overrides.push(record);

        tracing::debug!("Recorded override: {} → {}", original_model, override_model);

        // Also record as activity event
        self.record_event(ActivityEvent {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "model_override".to_string(),
            agent_name: None,
            model: Some(original_model.to_string()),
            tokens: None,
            latency_ms: None,
            status: Some("success".to_string()),
            cost: None,
        }).await;
    }

    /// Get override statistics
    pub async fn get_override_statistics(&self) -> Vec<OverrideRecord> {
        let overrides = self.model_overrides.lock().await;
        overrides.clone()
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

    /// Load metrics from disk (~/.claude/metrics.json)
    pub async fn load_from_disk() -> anyhow::Result<Vec<ApiCallRecord>> {
        let path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
            .join(".claude")
            .join("metrics.json");

        if !path.exists() {
            return Ok(Vec::new()); // Return empty if file doesn't exist
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let records: Vec<ApiCallRecord> = serde_json::from_str(&content)?;
        Ok(records)
    }

    /// Save metrics to disk (~/.claude/metrics.json)
    pub async fn save_to_disk(&self) -> anyhow::Result<()> {
        let path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
            .join(".claude")
            .join("metrics.json");

        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;

        let records = self.records.lock().await;
        let json = serde_json::to_string_pretty(&*records)?;
        tokio::fs::write(&path, json).await?;
        Ok(())
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

    #[tokio::test]
    async fn test_record_event_adds_correctly() {
        let analytics = AnalyticsEngine::new();
        let event = ActivityEvent {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "api_call".to_string(),
            agent_name: Some("test-agent".to_string()),
            model: Some("claude-opus-4".to_string()),
            tokens: Some(1000),
            latency_ms: Some(150),
            status: Some("success".to_string()),
            cost: Some(0.05),
        };

        analytics.record_event(event).await;

        let recent = analytics.get_recent_activity(10).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].event_type, "api_call");
        assert_eq!(recent[0].agent_name, Some("test-agent".to_string()));
    }

    #[tokio::test]
    async fn test_activity_buffer_maintains_max_100_events() {
        let analytics = AnalyticsEngine::new();

        // Add 150 events
        for i in 0..150 {
            let event = ActivityEvent {
                timestamp: Utc::now().to_rfc3339(),
                event_type: format!("event_{}", i),
                agent_name: None,
                model: None,
                tokens: None,
                latency_ms: None,
                status: Some("success".to_string()),
                cost: None,
            };
            analytics.record_event(event).await;
        }

        // Should only have 100 events (oldest 50 discarded)
        let all_events = analytics.get_recent_activity(200).await;
        assert_eq!(all_events.len(), 100);

        // The first event should be event_50 (50-149 kept, 0-49 discarded)
        assert_eq!(all_events[0].event_type, "event_50");
        assert_eq!(all_events[99].event_type, "event_149");
    }

    #[tokio::test]
    async fn test_get_recent_activity_respects_limit() {
        let analytics = AnalyticsEngine::new();

        for i in 0..20 {
            let event = ActivityEvent {
                timestamp: Utc::now().to_rfc3339(),
                event_type: format!("event_{}", i),
                agent_name: None,
                model: None,
                tokens: None,
                latency_ms: None,
                status: Some("success".to_string()),
                cost: None,
            };
            analytics.record_event(event).await;
        }

        let recent = analytics.get_recent_activity(5).await;
        assert_eq!(recent.len(), 5);
        assert_eq!(recent[0].event_type, "event_15");
        assert_eq!(recent[4].event_type, "event_19");
    }
}
