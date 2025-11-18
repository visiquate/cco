//! SQLite schema definitions for metrics persistence
//! Contains CREATE TABLE statements and index definitions

/// Complete SQLite schema for metrics persistence
/// Includes tables for raw metrics, hourly aggregations, daily summaries, and session tracking
pub const SCHEMA: &str = r#"
-- Enable WAL mode for better concurrent access
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

-- Table: api_metrics
-- Stores individual API call events with full token and cost breakdown
CREATE TABLE IF NOT EXISTS api_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    model_name TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cache_write_tokens INTEGER NOT NULL,
    cache_read_tokens INTEGER NOT NULL,
    total_cost REAL NOT NULL,
    request_id TEXT UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index: api_metrics by timestamp (for time-range queries)
CREATE INDEX IF NOT EXISTS idx_api_metrics_timestamp
    ON api_metrics(timestamp DESC);

-- Index: api_metrics by model_name (for model-specific queries)
CREATE INDEX IF NOT EXISTS idx_api_metrics_model_name
    ON api_metrics(model_name, timestamp DESC);

-- Index: api_metrics by request_id (for deduplication and lookup)
CREATE INDEX IF NOT EXISTS idx_api_metrics_request_id
    ON api_metrics(request_id) WHERE request_id IS NOT NULL;

-- Table: hourly_aggregations
-- Aggregated metrics for each hour, grouped by model tier
-- Computed from api_metrics during regular aggregation runs
CREATE TABLE IF NOT EXISTS hourly_aggregations (
    hour_start INTEGER NOT NULL,
    model_tier TEXT NOT NULL,
    total_calls INTEGER NOT NULL DEFAULT 0,
    total_input_tokens INTEGER NOT NULL DEFAULT 0,
    total_output_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,
    cache_hit_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(hour_start, model_tier)
);

-- Index: hourly_aggregations by model_tier (for tier-specific queries)
CREATE INDEX IF NOT EXISTS idx_hourly_aggregations_model_tier
    ON hourly_aggregations(model_tier, hour_start DESC);

-- Index: hourly_aggregations by hour_start (for time-range queries)
CREATE INDEX IF NOT EXISTS idx_hourly_aggregations_hour_start
    ON hourly_aggregations(hour_start DESC);

-- Table: daily_summaries
-- Daily summary metrics with cost breakdown by tier
-- Computed from hourly_aggregations during daily aggregation runs
CREATE TABLE IF NOT EXISTS daily_summaries (
    day_start INTEGER PRIMARY KEY,
    total_calls INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,
    cost_breakdown TEXT NOT NULL DEFAULT '{"opus": 0, "sonnet": 0, "haiku": 0}',
    cache_hit_rate REAL NOT NULL DEFAULT 0.0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index: daily_summaries by day_start (for time-range queries)
CREATE INDEX IF NOT EXISTS idx_daily_summaries_day_start
    ON daily_summaries(day_start DESC);

-- Table: monitoring_sessions
-- Tracks daemon startup/shutdown and metrics collection periods
CREATE TABLE IF NOT EXISTS monitoring_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_start INTEGER NOT NULL,
    session_end INTEGER,
    metrics_recorded INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index: monitoring_sessions by session_start (for timeline queries)
CREATE INDEX IF NOT EXISTS idx_monitoring_sessions_session_start
    ON monitoring_sessions(session_start DESC);

-- Index: monitoring_sessions by active sessions (for current session lookup)
CREATE INDEX IF NOT EXISTS idx_monitoring_sessions_active
    ON monitoring_sessions(session_end) WHERE session_end IS NULL;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_contains_all_tables() {
        assert!(SCHEMA.contains("CREATE TABLE IF NOT EXISTS api_metrics"));
        assert!(SCHEMA.contains("CREATE TABLE IF NOT EXISTS hourly_aggregations"));
        assert!(SCHEMA.contains("CREATE TABLE IF NOT EXISTS daily_summaries"));
        assert!(SCHEMA.contains("CREATE TABLE IF NOT EXISTS monitoring_sessions"));
    }

    #[test]
    fn test_schema_contains_all_indexes() {
        // api_metrics indexes
        assert!(SCHEMA.contains("idx_api_metrics_timestamp"));
        assert!(SCHEMA.contains("idx_api_metrics_model_name"));
        assert!(SCHEMA.contains("idx_api_metrics_request_id"));

        // hourly_aggregations indexes
        assert!(SCHEMA.contains("idx_hourly_aggregations_model_tier"));
        assert!(SCHEMA.contains("idx_hourly_aggregations_hour_start"));

        // daily_summaries indexes
        assert!(SCHEMA.contains("idx_daily_summaries_day_start"));

        // monitoring_sessions indexes
        assert!(SCHEMA.contains("idx_monitoring_sessions_session_start"));
        assert!(SCHEMA.contains("idx_monitoring_sessions_active"));
    }

    #[test]
    fn test_schema_enables_wal_mode() {
        assert!(SCHEMA.contains("PRAGMA journal_mode = WAL"));
        assert!(SCHEMA.contains("PRAGMA synchronous = NORMAL"));
    }

    #[test]
    fn test_schema_has_unique_constraints() {
        // api_metrics request_id should be unique
        assert!(SCHEMA.contains("UNIQUE(hour_start, model_tier)"));
    }

    #[test]
    fn test_schema_defines_primary_keys() {
        assert!(SCHEMA.contains("INTEGER PRIMARY KEY AUTOINCREMENT"));
        assert!(SCHEMA.contains("INTEGER PRIMARY KEY"));
    }

    #[test]
    fn test_api_metrics_table_has_required_columns() {
        let start_idx = SCHEMA.find("CREATE TABLE IF NOT EXISTS api_metrics").unwrap();
        let end_idx = SCHEMA[start_idx..].find(");").unwrap() + start_idx + 2;
        let table_def = &SCHEMA[start_idx..end_idx];

        assert!(table_def.contains("timestamp"));
        assert!(table_def.contains("model_name"));
        assert!(table_def.contains("input_tokens"));
        assert!(table_def.contains("output_tokens"));
        assert!(table_def.contains("cache_write_tokens"));
        assert!(table_def.contains("cache_read_tokens"));
        assert!(table_def.contains("total_cost"));
        assert!(table_def.contains("request_id"));
    }

    #[test]
    fn test_hourly_aggregations_table_has_required_columns() {
        assert!(SCHEMA.contains("model_tier TEXT NOT NULL"));
        assert!(SCHEMA.contains("total_calls INTEGER"));
        assert!(SCHEMA.contains("total_input_tokens INTEGER"));
        assert!(SCHEMA.contains("total_output_tokens INTEGER"));
        assert!(SCHEMA.contains("cache_hit_count INTEGER"));
    }

    #[test]
    fn test_daily_summaries_table_has_required_columns() {
        assert!(SCHEMA.contains("cost_breakdown TEXT"));
        assert!(SCHEMA.contains("cache_hit_rate REAL"));
    }
}
