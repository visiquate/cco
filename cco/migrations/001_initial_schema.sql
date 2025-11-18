-- CCO Metrics Backend - Initial Schema
-- Version: 2025.11.2
-- Purpose: Core tables for API call tracking and cost analysis

-- Enable WAL mode for better concurrency
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;

-- Core API call tracking table
-- Stores individual API calls with token counts, costs, and performance metrics
CREATE TABLE IF NOT EXISTS api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Timestamp and session tracking
    timestamp INTEGER NOT NULL,          -- Unix timestamp (milliseconds)
    session_id TEXT,                     -- Session identifier (optional)

    -- Agent information
    agent_type TEXT,                     -- e.g., "chief-architect", "python-specialist"
    agent_name TEXT,                     -- Agent instance name (optional)

    -- Model information
    model_requested TEXT NOT NULL,       -- Original model requested by client
    model_used TEXT NOT NULL,            -- Actual model used (after overrides)
    provider TEXT NOT NULL DEFAULT 'anthropic',  -- "anthropic", "openai", "ollama"
    tier TEXT NOT NULL,                  -- "opus", "sonnet", "haiku", "other"

    -- Token counts
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cache_write_tokens INTEGER DEFAULT 0,   -- Claude prompt cache writes
    cache_read_tokens INTEGER DEFAULT 0,    -- Claude prompt cache reads

    -- Cost tracking (in USD)
    cost_usd REAL NOT NULL DEFAULT 0.0,     -- Actual cost
    would_be_cost_usd REAL,                 -- Cost without cache (for savings calc)

    -- Performance metrics
    latency_ms INTEGER NOT NULL DEFAULT 0,  -- Total API call latency
    ttfb_ms INTEGER,                        -- Time to first byte (streaming)

    -- Source information
    source_file TEXT,                       -- File that triggered the call (if available)
    cache_hit BOOLEAN DEFAULT 0,            -- Proxy cache hit (not Claude cache)
    error_code TEXT,                        -- Error code if request failed

    -- Metadata
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_timestamp ON api_calls(timestamp);
CREATE INDEX IF NOT EXISTS idx_model_used ON api_calls(model_used);
CREATE INDEX IF NOT EXISTS idx_tier ON api_calls(tier);
CREATE INDEX IF NOT EXISTS idx_session_id ON api_calls(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_timestamp_tier ON api_calls(timestamp, tier);  -- Composite for tier queries

-- Aggregated metrics by time window
-- Pre-computed summaries for efficient dashboard queries
CREATE TABLE IF NOT EXISTS metrics_aggregated (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Window definition
    window_start INTEGER NOT NULL,          -- Unix timestamp (start of window)
    window_size_seconds INTEGER NOT NULL,   -- 60, 300, 600 (1m, 5m, 10m)
    model TEXT NOT NULL,
    tier TEXT NOT NULL,

    -- Aggregated counts
    total_calls INTEGER NOT NULL DEFAULT 0,
    total_input_tokens INTEGER NOT NULL DEFAULT 0,
    total_output_tokens INTEGER NOT NULL DEFAULT 0,
    total_cache_write_tokens INTEGER NOT NULL DEFAULT 0,
    total_cache_read_tokens INTEGER NOT NULL DEFAULT 0,

    -- Aggregated costs (USD)
    total_cost_usd REAL NOT NULL DEFAULT 0.0,
    total_would_be_cost_usd REAL NOT NULL DEFAULT 0.0,
    total_savings_usd REAL NOT NULL DEFAULT 0.0,

    -- Performance aggregations
    avg_latency_ms REAL NOT NULL DEFAULT 0.0,
    p50_latency_ms REAL,
    p95_latency_ms REAL,
    p99_latency_ms REAL,

    -- Rate metrics
    calls_per_minute REAL NOT NULL DEFAULT 0.0,
    tokens_per_minute REAL NOT NULL DEFAULT 0.0,
    cost_per_minute_usd REAL NOT NULL DEFAULT 0.0,

    -- Metadata
    aggregated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),

    -- Unique constraint to prevent duplicates
    UNIQUE(window_start, window_size_seconds, model, tier)
);

-- Indexes for aggregated queries
CREATE INDEX IF NOT EXISTS idx_window ON metrics_aggregated(window_start, window_size_seconds);
CREATE INDEX IF NOT EXISTS idx_model_agg ON metrics_aggregated(model);
CREATE INDEX IF NOT EXISTS idx_tier_agg ON metrics_aggregated(tier);

-- Model tier mapping and pricing
-- Reference table for model costs and tier classification
CREATE TABLE IF NOT EXISTS model_tiers (
    model TEXT PRIMARY KEY,
    tier TEXT NOT NULL,                     -- "opus", "sonnet", "haiku", "other"
    provider TEXT NOT NULL,                 -- "anthropic", "openai", "ollama"

    -- Pricing per 1 million tokens (USD)
    input_cost_per_1m REAL NOT NULL,
    output_cost_per_1m REAL NOT NULL,
    cache_write_cost_per_1m REAL DEFAULT 0.0,
    cache_read_cost_per_1m REAL DEFAULT 0.0,

    -- Metadata
    active BOOLEAN DEFAULT 1,               -- For deprecating old models
    updated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),

    CHECK (tier IN ('opus', 'sonnet', 'haiku', 'other')),
    CHECK (provider IN ('anthropic', 'openai', 'ollama', 'other'))
);

-- Index for tier lookups
CREATE INDEX IF NOT EXISTS idx_tier_lookup ON model_tiers(tier, active);

-- Session tracking (for multi-agent workflows)
-- Tracks cost and call counts per session
CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,

    -- Session lifecycle
    started_at INTEGER NOT NULL,
    ended_at INTEGER,

    -- Session metadata
    project_name TEXT,
    user_name TEXT,

    -- Aggregated metrics
    total_cost_usd REAL DEFAULT 0.0,
    total_calls INTEGER DEFAULT 0,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,

    -- Metadata
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

-- Index for recent sessions
CREATE INDEX IF NOT EXISTS idx_started_at ON sessions(started_at);

-- Configuration and pricing (runtime updates)
-- Key-value store for runtime configuration
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
);

-- Insert default configuration values
INSERT OR IGNORE INTO config (key, value, description) VALUES
    ('db_version', '1', 'Database schema version'),
    ('archival_enabled', 'true', 'Enable automatic data archival'),
    ('archival_retention_days', '7', 'Days to retain raw API calls'),
    ('batch_size', '100', 'Batch writer buffer size'),
    ('batch_flush_seconds', '5', 'Batch writer flush interval'),
    ('query_cache_ttl_seconds', '1', 'Query cache TTL');
