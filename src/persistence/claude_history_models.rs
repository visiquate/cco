//! Data models for Claude conversation history metrics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Daily aggregated metrics for a specific model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyModelMetrics {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Normalized model name (e.g., "claude-opus-4")
    pub model: String,
    /// Input tokens for this day
    pub input_tokens: i64,
    /// Output tokens for this day
    pub output_tokens: i64,
    /// Cache creation tokens for this day
    pub cache_creation_tokens: i64,
    /// Cache read tokens for this day
    pub cache_read_tokens: i64,
    /// Total cost in USD for this day
    pub cost: f64,
    /// Number of conversations on this day
    pub conversation_count: i64,
    /// Number of messages on this day
    pub message_count: i64,
}

impl DailyModelMetrics {
    /// Create a new daily model metrics record
    pub fn new(date: String, model: String) -> Self {
        Self {
            date,
            model,
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            cost: 0.0,
            conversation_count: 0,
            message_count: 0,
        }
    }

    /// Get total tokens for this day
    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }

    /// Add another metrics record to this one (for aggregation)
    pub fn merge(&mut self, other: &Self) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_creation_tokens += other.cache_creation_tokens;
        self.cache_read_tokens += other.cache_read_tokens;
        self.cost += other.cost;
        self.conversation_count += other.conversation_count;
        self.message_count += other.message_count;
    }
}

/// Migration status tracking
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MigrationStatus {
    /// Always 1 (single row table)
    pub id: i64,
    /// Whether migration has completed
    pub migrated: bool,
    /// When migration started (ISO 8601 string)
    pub migration_started_at: Option<String>,
    /// When migration completed (ISO 8601 string)
    pub migration_completed_at: Option<String>,
    /// Number of JSONL files processed
    pub files_processed: i64,
    /// Number of conversations processed
    pub conversations_processed: i64,
    /// Number of messages processed
    pub messages_processed: i64,
    /// Error message if migration failed
    pub error_message: Option<String>,
}

/// Daily metrics aggregated across all models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTotalMetrics {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Total cost for all models on this day
    pub cost: f64,
    /// Total tokens for all models on this day
    pub tokens: i64,
    /// Breakdown by model
    pub models: HashMap<String, ModelDayBreakdown>,
}

/// Per-model breakdown for a single day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDayBreakdown {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost: f64,
    pub message_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_model_metrics_creation() {
        let metrics = DailyModelMetrics::new("2025-11-17".to_string(), "claude-opus-4".to_string());

        assert_eq!(metrics.date, "2025-11-17");
        assert_eq!(metrics.model, "claude-opus-4");
        assert_eq!(metrics.total_tokens(), 0);
    }

    #[test]
    fn test_daily_model_metrics_total_tokens() {
        let mut metrics =
            DailyModelMetrics::new("2025-11-17".to_string(), "claude-sonnet-4-5".to_string());

        metrics.input_tokens = 1000;
        metrics.output_tokens = 500;
        metrics.cache_creation_tokens = 200;
        metrics.cache_read_tokens = 100;

        assert_eq!(metrics.total_tokens(), 1800);
    }

    #[test]
    fn test_daily_model_metrics_merge() {
        let mut metrics1 =
            DailyModelMetrics::new("2025-11-17".to_string(), "claude-haiku-4-5".to_string());
        metrics1.input_tokens = 100;
        metrics1.output_tokens = 50;
        metrics1.cost = 0.005;
        metrics1.message_count = 5;

        let mut metrics2 =
            DailyModelMetrics::new("2025-11-17".to_string(), "claude-haiku-4-5".to_string());
        metrics2.input_tokens = 200;
        metrics2.output_tokens = 100;
        metrics2.cost = 0.010;
        metrics2.message_count = 10;

        metrics1.merge(&metrics2);

        assert_eq!(metrics1.input_tokens, 300);
        assert_eq!(metrics1.output_tokens, 150);
        assert!((metrics1.cost - 0.015).abs() < 0.0001);
        assert_eq!(metrics1.message_count, 15);
    }
}
