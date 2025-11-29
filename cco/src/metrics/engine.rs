//! Metrics aggregation engine for real-time API call tracking

use super::ApiCallEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Maximum number of events to keep in memory
const DEFAULT_BUFFER_SIZE: usize = 1000;

/// Summary of aggregated metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total cost in USD
    pub total_cost_usd: f64,

    /// Total number of API calls
    pub call_count: u64,

    /// Token breakdown by type
    pub tokens_by_type: TokensByType,

    /// Breakdown by model tier
    pub by_model_tier: HashMap<String, TierMetrics>,

    /// Total tokens across all calls
    pub total_tokens: u64,
}

/// Token counts by type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokensByType {
    pub input: u64,
    pub output: u64,
    pub cache_write: u64,
    pub cache_read: u64,
}

/// Metrics for a specific model tier
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TierMetrics {
    pub call_count: u64,
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_read_tokens: u64,
}

/// Metrics aggregation engine
pub struct MetricsEngine {
    /// Ring buffer of recent events
    events: Arc<RwLock<Vec<ApiCallEvent>>>,

    /// Maximum buffer size
    buffer_size: usize,
}

impl MetricsEngine {
    /// Create a new metrics engine with default buffer size
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_BUFFER_SIZE)
    }

    /// Create a new metrics engine with specified buffer size
    pub fn with_capacity(buffer_size: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::with_capacity(buffer_size))),
            buffer_size,
        }
    }

    /// Record a new API call event
    pub async fn record_event(&self, event: ApiCallEvent) {
        let mut events = self.events.write().await;

        // Ring buffer: remove oldest if at capacity
        if events.len() >= self.buffer_size {
            events.remove(0);
            debug!(
                "Event buffer at capacity ({}), removed oldest event",
                self.buffer_size
            );
        }

        debug!(
            "Recording event: model={}, cost=${:.4}, tokens={}",
            event.model_name,
            event.cost_usd,
            event.tokens.total_tokens()
        );

        events.push(event);
    }

    /// Get summary of all recorded metrics
    pub async fn get_summary(&self) -> MetricsSummary {
        let events = self.events.read().await;

        let mut total_cost_usd = 0.0;
        let call_count = events.len() as u64;
        let mut tokens_by_type = TokensByType::default();
        let mut by_model_tier: HashMap<String, TierMetrics> = HashMap::new();
        let mut total_tokens = 0u64;

        for event in events.iter() {
            // Accumulate total cost
            total_cost_usd += event.cost_usd;

            // Accumulate tokens by type
            tokens_by_type.input += event.tokens.input_tokens;
            tokens_by_type.output += event.tokens.output_tokens;
            tokens_by_type.cache_write += event.tokens.cache_write_tokens;
            tokens_by_type.cache_read += event.tokens.cache_read_tokens;

            // Accumulate total tokens
            total_tokens += event.tokens.total_tokens();

            // Accumulate by model tier
            let tier_name = format!("{:?}", event.model_tier);
            let tier_metrics = by_model_tier.entry(tier_name).or_default();

            tier_metrics.call_count += 1;
            tier_metrics.total_cost_usd += event.cost_usd;
            tier_metrics.total_tokens += event.tokens.total_tokens();
            tier_metrics.input_tokens += event.tokens.input_tokens;
            tier_metrics.output_tokens += event.tokens.output_tokens;
            tier_metrics.cache_write_tokens += event.tokens.cache_write_tokens;
            tier_metrics.cache_read_tokens += event.tokens.cache_read_tokens;
        }

        MetricsSummary {
            total_cost_usd,
            call_count,
            tokens_by_type,
            by_model_tier,
            total_tokens,
        }
    }

    /// Get recent API calls (last N events)
    pub async fn get_recent_calls(&self, limit: usize) -> Vec<ApiCallEvent> {
        let events = self.events.read().await;

        let start = if events.len() > limit {
            events.len() - limit
        } else {
            0
        };

        events[start..].to_vec()
    }

    /// Get all events
    pub async fn get_all_events(&self) -> Vec<ApiCallEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Clear all events
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        events.clear();
        debug!("Cleared all metrics events");
    }

    /// Get current buffer size
    pub async fn get_buffer_size(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    /// Get buffer capacity
    pub fn get_buffer_capacity(&self) -> usize {
        self.buffer_size
    }
}

impl Default for MetricsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{ModelTier, TokenBreakdown};

    fn create_test_event(
        model_tier: ModelTier,
        input_tokens: u64,
        output_tokens: u64,
    ) -> ApiCallEvent {
        let model_name = match model_tier {
            ModelTier::Opus => "claude-opus-4",
            ModelTier::Sonnet => "claude-sonnet-3.5",
            ModelTier::Haiku => "claude-3-haiku-20240307",
        };

        let tokens = TokenBreakdown {
            input_tokens,
            output_tokens,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
        };

        ApiCallEvent::new(model_name.to_string(), tokens, None, None).unwrap()
    }

    #[tokio::test]
    async fn test_record_event() {
        let engine = MetricsEngine::new();
        let event = create_test_event(ModelTier::Opus, 1000, 500);

        engine.record_event(event).await;

        assert_eq!(engine.get_buffer_size().await, 1);
    }

    #[tokio::test]
    async fn test_get_summary() {
        let engine = MetricsEngine::new();

        // Record 3 Opus calls
        for _ in 0..3 {
            let event = create_test_event(ModelTier::Opus, 1000, 500);
            engine.record_event(event).await;
        }

        // Record 2 Sonnet calls
        for _ in 0..2 {
            let event = create_test_event(ModelTier::Sonnet, 2000, 1000);
            engine.record_event(event).await;
        }

        let summary = engine.get_summary().await;

        assert_eq!(summary.call_count, 5);
        assert_eq!(summary.by_model_tier.len(), 2);
        assert_eq!(summary.by_model_tier.get("Opus").unwrap().call_count, 3);
        assert_eq!(summary.by_model_tier.get("Sonnet").unwrap().call_count, 2);

        // Total tokens: 3 * (1000 + 500) + 2 * (2000 + 1000) = 4500 + 6000 = 10500
        assert_eq!(summary.total_tokens, 10500);
    }

    #[tokio::test]
    async fn test_ring_buffer_overflow() {
        let engine = MetricsEngine::with_capacity(5);

        // Add 10 events (should only keep last 5)
        for i in 0..10 {
            let event = create_test_event(ModelTier::Haiku, i * 100, i * 50);
            engine.record_event(event).await;
        }

        assert_eq!(engine.get_buffer_size().await, 5);

        // Verify we kept the last 5 events (indices 5-9)
        let events = engine.get_all_events().await;
        assert_eq!(events[0].tokens.input_tokens, 500); // event index 5
        assert_eq!(events[4].tokens.input_tokens, 900); // event index 9
    }

    #[tokio::test]
    async fn test_get_recent_calls() {
        let engine = MetricsEngine::new();

        // Add 10 events
        for i in 0..10 {
            let event = create_test_event(ModelTier::Sonnet, i * 100, i * 50);
            engine.record_event(event).await;
        }

        // Get last 3
        let recent = engine.get_recent_calls(3).await;
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].tokens.input_tokens, 700); // index 7
        assert_eq!(recent[2].tokens.input_tokens, 900); // index 9
    }

    #[tokio::test]
    async fn test_clear() {
        let engine = MetricsEngine::new();

        // Add some events
        for _ in 0..5 {
            let event = create_test_event(ModelTier::Opus, 1000, 500);
            engine.record_event(event).await;
        }

        assert_eq!(engine.get_buffer_size().await, 5);

        // Clear
        engine.clear().await;

        assert_eq!(engine.get_buffer_size().await, 0);

        let summary = engine.get_summary().await;
        assert_eq!(summary.call_count, 0);
        assert_eq!(summary.total_cost_usd, 0.0);
    }

    #[tokio::test]
    async fn test_cost_calculation_in_summary() {
        let engine = MetricsEngine::new();

        // Add 1 Opus call: 1M input + 500K output
        // Cost: $15 + $37.5 = $52.5
        let event = create_test_event(ModelTier::Opus, 1_000_000, 500_000);
        engine.record_event(event).await;

        let summary = engine.get_summary().await;

        assert!((summary.total_cost_usd - 52.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_tokens_by_type() {
        let engine = MetricsEngine::new();

        let tokens = TokenBreakdown {
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 200,
            cache_read_tokens: 300,
        };

        let event = ApiCallEvent::new("claude-opus-4".to_string(), tokens, None, None).unwrap();

        engine.record_event(event).await;

        let summary = engine.get_summary().await;

        assert_eq!(summary.tokens_by_type.input, 1000);
        assert_eq!(summary.tokens_by_type.output, 500);
        assert_eq!(summary.tokens_by_type.cache_write, 200);
        assert_eq!(summary.tokens_by_type.cache_read, 300);
    }
}
