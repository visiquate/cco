-- CCO Metrics Backend - Performance Indexes
-- Version: 2025.11.2
-- Purpose: Add indexes for common query patterns

-- Covering index for tier-based cost analysis queries
CREATE INDEX IF NOT EXISTS idx_tier_timestamp_cost ON api_calls(tier, timestamp, cost_usd)
WHERE cost_usd > 0;

-- Covering index for model-based queries
CREATE INDEX IF NOT EXISTS idx_model_timestamp_tokens ON api_calls(model_used, timestamp, input_tokens, output_tokens);

-- Composite index for session queries
CREATE INDEX IF NOT EXISTS idx_session_timestamp ON api_calls(session_id, timestamp)
WHERE session_id IS NOT NULL;

-- Index for recent calls queries (dashboard)
CREATE INDEX IF NOT EXISTS idx_recent_calls ON api_calls(timestamp DESC, id DESC);

-- Partial index for error tracking
CREATE INDEX IF NOT EXISTS idx_errors ON api_calls(timestamp, error_code)
WHERE error_code IS NOT NULL;

-- Index for cache hit analysis
CREATE INDEX IF NOT EXISTS idx_cache_hits ON api_calls(cache_hit, timestamp)
WHERE cache_hit = 1;

-- Aggregated metrics indexes for dashboard queries
CREATE INDEX IF NOT EXISTS idx_agg_tier_window ON metrics_aggregated(tier, window_start DESC, window_size_seconds);

CREATE INDEX IF NOT EXISTS idx_agg_cost ON metrics_aggregated(window_start DESC, total_cost_usd DESC);

-- Session performance index
CREATE INDEX IF NOT EXISTS idx_session_cost ON sessions(total_cost_usd DESC, started_at DESC);

-- Add ANALYZE to update statistics for query optimizer
ANALYZE;

-- Insert index metadata
INSERT OR REPLACE INTO config (key, value, description) VALUES (
    'indexes_version',
    '3',
    'Performance indexes version'
);
