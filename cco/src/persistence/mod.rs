//! Persistence layer for metrics storage and retrieval
//! Handles SQLite database operations for metrics, aggregations, and session tracking

pub mod models;
pub mod schema;
pub mod claude_history_models;
pub mod claude_history_schema;
pub mod claude_history_persistence;

use models::{ApiMetricRecord, DailySummary, HourlyAggregation, MonitoringSession};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Custom error type for persistence operations
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;

/// Main persistence layer for metrics storage
/// Manages SQLite database connections and all metrics operations
pub struct PersistenceLayer {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl PersistenceLayer {
    /// Create a new persistence layer instance
    /// Initializes database connection pool and creates schema if needed
    pub async fn new(db_path: impl AsRef<Path>) -> PersistenceResult<Self> {
        let db_path = db_path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create SQLite connection string with WAL mode
        // SQLx requires sqlite: prefix with create flag
        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

        // Create connection pool with optimized settings
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .connect(&database_url)
            .await?;

        // Enable WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;

        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;

        // Create schema
        sqlx::raw_sql(schema::SCHEMA)
            .execute(&pool)
            .await?;

        // Create Claude history schema
        sqlx::raw_sql(claude_history_schema::CLAUDE_HISTORY_SCHEMA)
            .execute(&pool)
            .await?;

        Ok(Self {
            pool,
            db_path: db_path.to_path_buf(),
        })
    }

    /// Record a single API call event in the database
    pub async fn record_event(&self, record: ApiMetricRecord) -> PersistenceResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO api_metrics (timestamp, model_name, input_tokens, output_tokens,
                                     cache_write_tokens, cache_read_tokens, total_cost, request_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(record.timestamp)
        .bind(&record.model_name)
        .bind(record.input_tokens)
        .bind(record.output_tokens)
        .bind(record.cache_write_tokens)
        .bind(record.cache_read_tokens)
        .bind(record.total_cost)
        .bind(&record.request_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get all metrics for a specific time range
    pub async fn get_metrics(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> PersistenceResult<Vec<ApiMetricRecord>> {
        let records = sqlx::query_as::<_, ApiMetricRecord>(
            r#"
            SELECT id, timestamp, model_name, input_tokens, output_tokens,
                   cache_write_tokens, cache_read_tokens, total_cost, request_id
            FROM api_metrics
            WHERE timestamp BETWEEN ? AND ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(start_timestamp)
        .bind(end_timestamp)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get hourly aggregations for a specific model tier and time range
    pub async fn get_hourly_stats(
        &self,
        model_tier: &str,
        start_hour: i64,
        end_hour: i64,
    ) -> PersistenceResult<Vec<HourlyAggregation>> {
        let records = sqlx::query_as::<_, HourlyAggregation>(
            r#"
            SELECT hour_start, model_tier, total_calls, total_input_tokens,
                   total_output_tokens, total_cost, cache_hit_count
            FROM hourly_aggregations
            WHERE model_tier = ? AND hour_start BETWEEN ? AND ?
            ORDER BY hour_start DESC
            "#,
        )
        .bind(model_tier)
        .bind(start_hour)
        .bind(end_hour)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get daily summaries for a specific time range
    pub async fn get_daily_stats(
        &self,
        start_day: i64,
        end_day: i64,
    ) -> PersistenceResult<Vec<DailySummary>> {
        let records = sqlx::query_as::<_, DailySummary>(
            r#"
            SELECT day_start, total_calls, total_cost, cost_breakdown, cache_hit_rate
            FROM daily_summaries
            WHERE day_start BETWEEN ? AND ?
            ORDER BY day_start DESC
            "#,
        )
        .bind(start_day)
        .bind(end_day)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get current cost summary for the specified time period
    pub async fn get_cost_summary(&self, start_timestamp: i64) -> PersistenceResult<(f64, i64)> {
        let result = sqlx::query(
            r#"
            SELECT COALESCE(SUM(total_cost), 0.0) as total_cost,
                   COUNT(*) as total_calls
            FROM api_metrics
            WHERE timestamp >= ?
            "#,
        )
        .bind(start_timestamp)
        .fetch_one(&self.pool)
        .await?;

        let total_cost: f64 = result.get("total_cost");
        let total_calls: i64 = result.get("total_calls");

        Ok((total_cost, total_calls))
    }

    /// Start a new monitoring session
    pub async fn start_session(&self, session_start: i64) -> PersistenceResult<i64> {
        let session = MonitoringSession::new(session_start);

        let result = sqlx::query(
            r#"
            INSERT INTO monitoring_sessions (session_start, session_end, metrics_recorded)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(session.session_start)
        .bind(session.session_end)
        .bind(session.metrics_recorded)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// End the current monitoring session
    pub async fn end_session(&self, session_id: i64, session_end: i64) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            UPDATE monitoring_sessions
            SET session_end = ?
            WHERE id = ?
            "#,
        )
        .bind(session_end)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the current active session (if any)
    pub async fn get_active_session(&self) -> PersistenceResult<Option<MonitoringSession>> {
        let session = sqlx::query_as::<_, MonitoringSession>(
            r#"
            SELECT id, session_start, session_end, metrics_recorded
            FROM monitoring_sessions
            WHERE session_end IS NULL
            ORDER BY session_start DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    /// Get hourly aggregation for a specific hour and tier
    pub async fn get_hourly_aggregation(
        &self,
        hour_start: i64,
        model_tier: &str,
    ) -> PersistenceResult<Option<HourlyAggregation>> {
        let agg = sqlx::query_as::<_, HourlyAggregation>(
            r#"
            SELECT hour_start, model_tier, total_calls, total_input_tokens,
                   total_output_tokens, total_cost, cache_hit_count
            FROM hourly_aggregations
            WHERE hour_start = ? AND model_tier = ?
            "#,
        )
        .bind(hour_start)
        .bind(model_tier)
        .fetch_optional(&self.pool)
        .await?;

        Ok(agg)
    }

    /// Insert or update an hourly aggregation
    pub async fn insert_hourly_aggregation(
        &self,
        agg: HourlyAggregation,
    ) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            INSERT INTO hourly_aggregations (hour_start, model_tier, total_calls,
                                            total_input_tokens, total_output_tokens,
                                            total_cost, cache_hit_count)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(hour_start, model_tier)
            DO UPDATE SET total_calls = excluded.total_calls,
                         total_input_tokens = excluded.total_input_tokens,
                         total_output_tokens = excluded.total_output_tokens,
                         total_cost = excluded.total_cost,
                         cache_hit_count = excluded.cache_hit_count
            "#,
        )
        .bind(agg.hour_start)
        .bind(&agg.model_tier)
        .bind(agg.total_calls)
        .bind(agg.total_input_tokens)
        .bind(agg.total_output_tokens)
        .bind(agg.total_cost)
        .bind(agg.cache_hit_count)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert or update a daily summary
    pub async fn insert_daily_summary(&self, summary: DailySummary) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            INSERT INTO daily_summaries (day_start, total_calls, total_cost,
                                        cost_breakdown, cache_hit_rate)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(day_start)
            DO UPDATE SET total_calls = excluded.total_calls,
                         total_cost = excluded.total_cost,
                         cost_breakdown = excluded.cost_breakdown,
                         cache_hit_rate = excluded.cache_hit_rate
            "#,
        )
        .bind(summary.day_start)
        .bind(summary.total_calls)
        .bind(summary.total_cost)
        .bind(&summary.cost_breakdown)
        .bind(summary.cache_hit_rate)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the database file path
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Get Claude history persistence layer
    pub fn claude_history(&self) -> claude_history_persistence::ClaudeHistoryPersistence {
        claude_history_persistence::ClaudeHistoryPersistence::new(self.pool.clone())
    }

    /// Get metrics recorded count for a specific session
    pub async fn get_session_metrics_count(&self, session_id: i64) -> PersistenceResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT metrics_recorded
            FROM monitoring_sessions
            WHERE id = ?
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(row) => Ok(row.get("metrics_recorded")),
            None => Err(PersistenceError::NotFound(format!(
                "Session with id {} not found",
                session_id
            ))),
        }
    }

    /// Update metrics recorded count for a session
    pub async fn update_session_metrics_count(
        &self,
        session_id: i64,
        count: i64,
    ) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            UPDATE monitoring_sessions
            SET metrics_recorded = ?
            WHERE id = ?
            "#,
        )
        .bind(count)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clear all metrics older than the specified timestamp
    pub async fn cleanup_old_metrics(&self, cutoff_timestamp: i64) -> PersistenceResult<u64> {
        let result = sqlx::query("DELETE FROM api_metrics WHERE timestamp < ?")
            .bind(cutoff_timestamp)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get database statistics for monitoring
    pub async fn get_stats(&self) -> PersistenceResult<DatabaseStats> {
        let metrics_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_metrics")
            .fetch_one(&self.pool)
            .await?;

        let hourly_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM hourly_aggregations")
            .fetch_one(&self.pool)
            .await?;

        let daily_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM daily_summaries")
            .fetch_one(&self.pool)
            .await?;

        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM monitoring_sessions")
            .fetch_one(&self.pool)
            .await?;

        Ok(DatabaseStats {
            api_metrics_count: metrics_count.0,
            hourly_aggregations_count: hourly_count.0,
            daily_summaries_count: daily_count.0,
            monitoring_sessions_count: session_count.0,
        })
    }
}

/// Database statistics summary
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub api_metrics_count: i64,
    pub hourly_aggregations_count: i64,
    pub daily_summaries_count: i64,
    pub monitoring_sessions_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistence_layer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let result = PersistenceLayer::new(&db_path).await;
        assert!(result.is_ok());

        let persistence = result.unwrap();
        assert_eq!(persistence.db_path(), db_path);
    }

    #[tokio::test]
    async fn test_record_event() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

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

        let result = persistence.record_event(record).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_get_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

        // Insert test record
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

        persistence.record_event(record).await.unwrap();

        // Retrieve metrics
        let metrics = persistence.get_metrics(900, 1100).await.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].model_name, "claude-sonnet");
        assert_eq!(metrics[0].input_tokens, 100);
    }

    #[tokio::test]
    async fn test_session_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

        // Start session
        let session_id = persistence.start_session(1000).await.unwrap();
        assert!(session_id > 0);

        // Check active session
        let active = persistence.get_active_session().await.unwrap();
        assert!(active.is_some());

        let session = active.unwrap();
        assert_eq!(session.session_start, 1000);
        assert!(session.session_end.is_none());
        assert!(session.is_active());

        // End session
        persistence.end_session(session_id, 2000).await.unwrap();

        // Verify session ended
        let active = persistence.get_active_session().await.unwrap();
        assert!(active.is_none());
    }

    #[tokio::test]
    async fn test_get_cost_summary() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

        // Insert test records
        for i in 0..5 {
            let record = ApiMetricRecord::new(
                1000 + (i * 100),
                "claude-haiku".to_string(),
                50,
                25,
                0,
                0,
                0.001 * (i as f64 + 1.0),
                None,
            );
            persistence.record_event(record).await.unwrap();
        }

        let (total_cost, total_calls) = persistence.get_cost_summary(900).await.unwrap();

        assert_eq!(total_calls, 5);
        assert!(total_cost > 0.01 && total_cost < 0.02); // Sum of 0.001, 0.002, 0.003, 0.004, 0.005
    }

    #[tokio::test]
    async fn test_hourly_aggregation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

        let agg = HourlyAggregation {
            hour_start: 3600,
            model_tier: "sonnet".to_string(),
            total_calls: 100,
            total_input_tokens: 50000,
            total_output_tokens: 25000,
            total_cost: 5.0,
            cache_hit_count: 10,
        };

        let result = persistence.insert_hourly_aggregation(agg.clone()).await;
        if let Err(e) = &result {
            eprintln!("Error inserting hourly aggregation: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to insert hourly aggregation");

        let retrieved = persistence
            .get_hourly_aggregation(3600, "sonnet")
            .await
            .unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.total_calls, 100);
        assert_eq!(retrieved.total_cost, 5.0);
    }

    #[tokio::test]
    async fn test_database_stats() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

        // Insert some data
        for i in 0..3 {
            let record = ApiMetricRecord::new(
                1000 + (i * 100),
                "claude-opus-4".to_string(),
                1000,
                500,
                0,
                0,
                0.05,
                None,
            );
            persistence.record_event(record).await.unwrap();
        }

        let stats = persistence.get_stats().await.unwrap();
        assert_eq!(stats.api_metrics_count, 3);
        assert_eq!(stats.hourly_aggregations_count, 0);
        assert_eq!(stats.daily_summaries_count, 0);
    }

    #[tokio::test]
    async fn test_cleanup_old_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let persistence = PersistenceLayer::new(&db_path).await.unwrap();

        // Insert records with different timestamps
        for ts in [500, 1000, 1500, 2000] {
            let record =
                ApiMetricRecord::new(ts, "claude-haiku".to_string(), 50, 25, 0, 0, 0.001, None);
            persistence.record_event(record).await.unwrap();
        }

        // Cleanup metrics older than 1200
        let deleted = persistence.cleanup_old_metrics(1200).await.unwrap();
        assert_eq!(deleted, 2); // Should delete records with ts 500 and 1000

        // Verify remaining records
        let metrics = persistence.get_metrics(0, 3000).await.unwrap();
        assert_eq!(metrics.len(), 2);
    }
}
