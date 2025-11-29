//! Schema extension for Claude conversation history metrics
//! Stores daily aggregated metrics extracted from JSONL conversation files

/// Schema for Claude history metrics table
/// This extends the existing persistence schema with historical data from JSONL files
pub const CLAUDE_HISTORY_SCHEMA: &str = r#"
-- Table: claude_history_metrics
-- Stores daily aggregated metrics from Claude conversation history JSONL files
-- Data is grouped by date and model, allowing time-series analysis and trending
CREATE TABLE IF NOT EXISTS claude_history_metrics (
    date TEXT NOT NULL,                    -- Date in YYYY-MM-DD format
    model TEXT NOT NULL,                   -- Normalized model name (e.g., "claude-opus-4")
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cost REAL NOT NULL DEFAULT 0.0,        -- Total cost in USD for this day+model
    conversation_count INTEGER NOT NULL DEFAULT 0,  -- Number of conversations on this day
    message_count INTEGER NOT NULL DEFAULT 0,       -- Number of messages on this day
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (date, model)
) WITHOUT ROWID;

-- Index: claude_history_metrics by date (for time-range queries)
CREATE INDEX IF NOT EXISTS idx_claude_history_date
    ON claude_history_metrics(date DESC);

-- Index: claude_history_metrics by model (for model-specific queries)
CREATE INDEX IF NOT EXISTS idx_claude_history_model
    ON claude_history_metrics(model, date DESC);

-- Table: claude_history_migration_status
-- Tracks migration state to ensure JSONL files are only processed once
CREATE TABLE IF NOT EXISTS claude_history_migration_status (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Single row table
    migrated BOOLEAN NOT NULL DEFAULT 0,
    migration_started_at DATETIME,
    migration_completed_at DATETIME,
    files_processed INTEGER DEFAULT 0,
    conversations_processed INTEGER DEFAULT 0,
    messages_processed INTEGER DEFAULT 0,
    error_message TEXT
);

-- Initialize migration status row
INSERT OR IGNORE INTO claude_history_migration_status (id, migrated) VALUES (1, 0);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_contains_claude_history_metrics_table() {
        assert!(CLAUDE_HISTORY_SCHEMA.contains("CREATE TABLE IF NOT EXISTS claude_history_metrics"));
    }

    #[test]
    fn test_schema_contains_migration_status_table() {
        assert!(CLAUDE_HISTORY_SCHEMA
            .contains("CREATE TABLE IF NOT EXISTS claude_history_migration_status"));
    }

    #[test]
    fn test_schema_has_date_model_composite_key() {
        assert!(CLAUDE_HISTORY_SCHEMA.contains("PRIMARY KEY (date, model)"));
    }

    #[test]
    fn test_schema_has_date_index() {
        assert!(CLAUDE_HISTORY_SCHEMA.contains("idx_claude_history_date"));
    }

    #[test]
    fn test_schema_has_model_index() {
        assert!(CLAUDE_HISTORY_SCHEMA.contains("idx_claude_history_model"));
    }

    #[test]
    fn test_migration_status_single_row_constraint() {
        assert!(CLAUDE_HISTORY_SCHEMA.contains("CHECK (id = 1)"));
    }
}
