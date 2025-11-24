//! Data models for persistence layer
//! These structures map to SQLite tables for durable metrics storage

use serde::{Deserialize, Serialize};

/// Represents a single API call event stored in database
/// Maps to the `api_metrics` table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiMetricRecord {
    /// Auto-incremented primary key
    pub id: Option<i64>,
    /// Unix timestamp when the call occurred
    pub timestamp: i64,
    /// Full model name (e.g., "claude-opus-4")
    pub model_name: String,
    /// Input tokens used
    pub input_tokens: i64,
    /// Output tokens generated
    pub output_tokens: i64,
    /// Tokens used for cache writes
    pub cache_write_tokens: i64,
    /// Tokens used for cache reads
    pub cache_read_tokens: i64,
    /// Total cost in USD for this call
    pub total_cost: f64,
    /// Unique request identifier (if available)
    pub request_id: Option<String>,
}

impl ApiMetricRecord {
    /// Create a new API metric record
    pub fn new(
        timestamp: i64,
        model_name: String,
        input_tokens: i64,
        output_tokens: i64,
        cache_write_tokens: i64,
        cache_read_tokens: i64,
        total_cost: f64,
        request_id: Option<String>,
    ) -> Self {
        Self {
            id: None,
            timestamp,
            model_name,
            input_tokens,
            output_tokens,
            cache_write_tokens,
            cache_read_tokens,
            total_cost,
            request_id,
        }
    }

    /// Get total tokens for this record
    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens + self.cache_write_tokens + self.cache_read_tokens
    }
}

/// Hourly aggregated metrics by model tier
/// Maps to the `hourly_aggregations` table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HourlyAggregation {
    /// Unix timestamp of hour start (primary key)
    pub hour_start: i64,
    /// Model tier (opus/sonnet/haiku)
    pub model_tier: String,
    /// Number of API calls in this hour
    pub total_calls: i64,
    /// Aggregated input tokens
    pub total_input_tokens: i64,
    /// Aggregated output tokens
    pub total_output_tokens: i64,
    /// Total cost for this hour
    pub total_cost: f64,
    /// Number of cache hits in this hour
    pub cache_hit_count: i64,
}

/// Daily aggregated metrics summary
/// Maps to the `daily_summaries` table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailySummary {
    /// Unix timestamp of day start (primary key)
    pub day_start: i64,
    /// Total API calls in this day
    pub total_calls: i64,
    /// Total cost for this day
    pub total_cost: f64,
    /// Cost breakdown as JSON: {opus: X, sonnet: Y, haiku: Z}
    pub cost_breakdown: String,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
}

/// Monitoring session tracking
/// Maps to the `monitoring_sessions` table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MonitoringSession {
    /// Auto-incremented primary key
    pub id: Option<i64>,
    /// Unix timestamp when session started
    pub session_start: i64,
    /// Unix timestamp when session ended (None if still running)
    pub session_end: Option<i64>,
    /// Number of metrics recorded in this session
    pub metrics_recorded: i64,
}

impl MonitoringSession {
    /// Create a new monitoring session
    pub fn new(session_start: i64) -> Self {
        Self {
            id: None,
            session_start,
            session_end: None,
            metrics_recorded: 0,
        }
    }

    /// Calculate session duration in seconds
    pub fn duration_seconds(&self) -> Option<i64> {
        self.session_end.map(|end| end - self.session_start)
    }

    /// Check if session is still active
    pub fn is_active(&self) -> bool {
        self.session_end.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_metric_record_creation() {
        let record = ApiMetricRecord::new(
            1000,
            "claude-opus-4".to_string(),
            1000,
            500,
            0,
            0,
            0.05,
            Some("req-123".to_string()),
        );

        assert_eq!(record.timestamp, 1000);
        assert_eq!(record.model_name, "claude-opus-4");
        assert_eq!(record.total_tokens(), 1500);
    }

    #[test]
    fn test_api_metric_record_total_tokens() {
        let record = ApiMetricRecord::new(
            1000,
            "claude-sonnet".to_string(),
            100,
            50,
            20,
            10,
            0.01,
            None,
        );

        assert_eq!(record.total_tokens(), 180);
    }

    #[test]
    fn test_monitoring_session_creation() {
        let session = MonitoringSession::new(1000);

        assert_eq!(session.session_start, 1000);
        assert!(session.session_end.is_none());
        assert!(session.is_active());
        assert_eq!(session.metrics_recorded, 0);
    }

    #[test]
    fn test_monitoring_session_duration() {
        let mut session = MonitoringSession::new(1000);
        session.session_end = Some(2000);

        assert_eq!(session.duration_seconds(), Some(1000));
        assert!(!session.is_active());
    }
}
