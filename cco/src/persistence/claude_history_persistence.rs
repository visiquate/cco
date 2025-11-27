//! Persistence layer for Claude conversation history metrics
//! Manages SQLite storage of daily aggregated metrics from JSONL files

use super::claude_history_models::{DailyModelMetrics, DailyTotalMetrics, MigrationStatus, ModelDayBreakdown};
use super::claude_history_schema::CLAUDE_HISTORY_SCHEMA;
use super::PersistenceError;
use crate::claude_history::{load_claude_project_metrics_by_date, normalize_model_name, get_model_pricing, calculate_cost};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Claude history metrics persistence layer
pub struct ClaudeHistoryPersistence {
    pool: SqlitePool,
}

impl ClaudeHistoryPersistence {
    /// Create a new persistence layer for Claude history metrics
    /// Shares the same database pool as the main persistence layer
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize schema (create tables if they don't exist)
    pub async fn initialize_schema(&self) -> Result<(), PersistenceError> {
        sqlx::raw_sql(CLAUDE_HISTORY_SCHEMA)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Check if migration has already been completed
    pub async fn is_migrated(&self) -> Result<bool, PersistenceError> {
        let result = sqlx::query("SELECT migrated FROM claude_history_migration_status WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;

        let migrated: bool = result.get("migrated");
        Ok(migrated)
    }

    /// Mark migration as started
    async fn mark_migration_started(&self) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            UPDATE claude_history_migration_status
            SET migration_started_at = CURRENT_TIMESTAMP
            WHERE id = 1
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mark migration as completed
    async fn mark_migration_completed(
        &self,
        files_processed: i64,
        conversations_processed: i64,
        messages_processed: i64,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            UPDATE claude_history_migration_status
            SET migrated = 1,
                migration_completed_at = CURRENT_TIMESTAMP,
                files_processed = ?,
                conversations_processed = ?,
                messages_processed = ?
            WHERE id = 1
            "#,
        )
        .bind(files_processed)
        .bind(conversations_processed)
        .bind(messages_processed)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mark migration as failed
    async fn mark_migration_failed(&self, error_message: &str) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            UPDATE claude_history_migration_status
            SET error_message = ?
            WHERE id = 1
            "#,
        )
        .bind(error_message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Insert or update daily model metrics
    pub async fn upsert_daily_metrics(&self, metrics: &DailyModelMetrics) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            INSERT INTO claude_history_metrics (
                date, model, input_tokens, output_tokens,
                cache_creation_tokens, cache_read_tokens, cost,
                conversation_count, message_count, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(date, model) DO UPDATE SET
                input_tokens = input_tokens + excluded.input_tokens,
                output_tokens = output_tokens + excluded.output_tokens,
                cache_creation_tokens = cache_creation_tokens + excluded.cache_creation_tokens,
                cache_read_tokens = cache_read_tokens + excluded.cache_read_tokens,
                cost = cost + excluded.cost,
                conversation_count = conversation_count + excluded.conversation_count,
                message_count = message_count + excluded.message_count,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&metrics.date)
        .bind(&metrics.model)
        .bind(metrics.input_tokens)
        .bind(metrics.output_tokens)
        .bind(metrics.cache_creation_tokens)
        .bind(metrics.cache_read_tokens)
        .bind(metrics.cost)
        .bind(metrics.conversation_count)
        .bind(metrics.message_count)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get metrics for a date range
    pub async fn get_metrics_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DailyModelMetrics>, PersistenceError> {
        let metrics = sqlx::query_as::<_, DailyModelMetrics>(
            r#"
            SELECT date, model, input_tokens, output_tokens,
                   cache_creation_tokens, cache_read_tokens, cost,
                   conversation_count, message_count
            FROM claude_history_metrics
            WHERE date BETWEEN ? AND ?
            ORDER BY date ASC, model
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    /// Get total metrics by date (aggregated across all models)
    pub async fn get_daily_totals(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DailyTotalMetrics>, PersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT date,
                   SUM(cost) as total_cost,
                   SUM(input_tokens + output_tokens + cache_creation_tokens + cache_read_tokens) as total_tokens
            FROM claude_history_metrics
            WHERE date BETWEEN ? AND ?
            GROUP BY date
            ORDER BY date ASC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let date: String = row.get("date");
            let total_cost: f64 = row.get("total_cost");
            let total_tokens: i64 = row.get("total_tokens");

            // Get model breakdown for this date
            let model_metrics = self.get_metrics_for_date(&date).await?;
            let mut models = HashMap::new();

            for metrics in model_metrics {
                models.insert(
                    metrics.model.clone(),
                    ModelDayBreakdown {
                        input_tokens: metrics.input_tokens,
                        output_tokens: metrics.output_tokens,
                        cache_creation_tokens: metrics.cache_creation_tokens,
                        cache_read_tokens: metrics.cache_read_tokens,
                        cost: metrics.cost,
                        message_count: metrics.message_count,
                    },
                );
            }

            results.push(DailyTotalMetrics {
                date,
                cost: total_cost,
                tokens: total_tokens,
                models,
            });
        }

        Ok(results)
    }

    /// Get metrics for a specific date
    pub async fn get_metrics_for_date(
        &self,
        date: &str,
    ) -> Result<Vec<DailyModelMetrics>, PersistenceError> {
        let metrics = sqlx::query_as::<_, DailyModelMetrics>(
            r#"
            SELECT date, model, input_tokens, output_tokens,
                   cache_creation_tokens, cache_read_tokens, cost,
                   conversation_count, message_count
            FROM claude_history_metrics
            WHERE date = ?
            ORDER BY model
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    /// Migrate JSONL files to database (one-time operation)
    pub async fn migrate_from_jsonl(&self, project_path: &str) -> Result<(), PersistenceError> {
        // Check if already migrated
        if self.is_migrated().await? {
            info!("Claude history already migrated, skipping");
            return Ok(());
        }

        info!("Starting Claude history migration from JSONL files");
        self.mark_migration_started().await?;

        // Load metrics grouped by date
        let metrics_by_date = match load_claude_project_metrics_by_date(project_path).await {
            Ok(metrics) => metrics,
            Err(e) => {
                let error_msg = format!("Failed to load JSONL files: {}", e);
                warn!("{}", error_msg);
                self.mark_migration_failed(&error_msg).await?;
                return Err(PersistenceError::Configuration(error_msg));
            }
        };

        let mut files_processed = 0;
        let mut messages_processed = 0;
        let conversations_processed = metrics_by_date.len() as i64;

        // Process each day's metrics
        for (date, messages) in metrics_by_date {
            debug!("Processing {} messages for date {}", messages.len(), date);

            // Aggregate by model for this date
            let mut daily_metrics: HashMap<String, DailyModelMetrics> = HashMap::new();

            for (model, usage, _timestamp) in messages {
                let normalized_model = normalize_model_name(&model);
                let (input_price, output_price, cache_write_price, cache_read_price) =
                    get_model_pricing(&normalized_model);

                let input_tokens = usage.input_tokens.unwrap_or(0) as i64;
                let output_tokens = usage.output_tokens.unwrap_or(0) as i64;
                let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0) as i64;
                let cache_read = usage.cache_read_input_tokens.unwrap_or(0) as i64;

                // Calculate costs
                let input_cost = calculate_cost(input_tokens as u64, input_price);
                let output_cost = calculate_cost(output_tokens as u64, output_price);
                let cache_write_cost = calculate_cost(cache_creation as u64, cache_write_price);
                let cache_read_cost = calculate_cost(cache_read as u64, cache_read_price);
                let total_cost = input_cost + output_cost + cache_write_cost + cache_read_cost;

                // Update or create daily metrics for this model
                let metrics = daily_metrics
                    .entry(normalized_model.clone())
                    .or_insert_with(|| DailyModelMetrics::new(date.clone(), normalized_model.clone()));

                metrics.input_tokens += input_tokens;
                metrics.output_tokens += output_tokens;
                metrics.cache_creation_tokens += cache_creation;
                metrics.cache_read_tokens += cache_read;
                metrics.cost += total_cost;
                metrics.message_count += 1;

                messages_processed += 1;
            }

            // Insert all daily metrics for this date
            for metrics in daily_metrics.values() {
                self.upsert_daily_metrics(metrics).await?;
            }

            files_processed += 1;
        }

        // Mark migration as complete
        self.mark_migration_completed(files_processed, conversations_processed, messages_processed).await?;
        info!(
            "Migration complete: {} files, {} conversations, {} messages",
            files_processed, conversations_processed, messages_processed
        );

        Ok(())
    }

    /// Get migration status
    pub async fn get_migration_status(&self) -> Result<MigrationStatus, PersistenceError> {
        let status = sqlx::query_as::<_, MigrationStatus>(
            r#"
            SELECT id, migrated, migration_started_at, migration_completed_at,
                   files_processed, conversations_processed, messages_processed, error_message
            FROM claude_history_migration_status
            WHERE id = 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(status)
    }

    /// Store aggregated metrics from ClaudeMetrics (used by background parser)
    pub async fn store_aggregated_metrics(&self, metrics: &crate::claude_history::ClaudeMetrics) -> Result<(), PersistenceError> {
        // Get current date for metrics storage
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        debug!("Storing aggregated metrics for {}: {} models", today, metrics.model_breakdown.len());

        // Store each model's breakdown as daily metrics
        for (model_name, breakdown) in &metrics.model_breakdown {
            let daily_metrics = DailyModelMetrics {
                date: today.clone(),
                model: model_name.clone(),
                input_tokens: breakdown.input_tokens as i64,
                output_tokens: breakdown.output_tokens as i64,
                cache_creation_tokens: breakdown.cache_creation_tokens as i64,
                cache_read_tokens: breakdown.cache_read_tokens as i64,
                cost: breakdown.total_cost,
                conversation_count: 1, // Aggregated data doesn't track individual conversations
                message_count: breakdown.message_count as i64,
            };

            self.upsert_daily_metrics(&daily_metrics).await?;
        }

        debug!("Successfully stored metrics for {} models", metrics.model_breakdown.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_persistence() -> (ClaudeHistoryPersistence, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await.unwrap();

        let persistence = ClaudeHistoryPersistence::new(pool);
        persistence.initialize_schema().await.unwrap();

        (persistence, temp_dir)
    }

    #[tokio::test]
    async fn test_schema_initialization() {
        let (persistence, _temp_dir) = create_test_persistence().await;

        // Schema should be initialized
        let is_migrated = persistence.is_migrated().await.unwrap();
        assert!(!is_migrated);
    }

    #[tokio::test]
    async fn test_upsert_daily_metrics() {
        let (persistence, _temp_dir) = create_test_persistence().await;

        let metrics = DailyModelMetrics {
            date: "2025-11-17".to_string(),
            model: "claude-opus-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_tokens: 200,
            cache_read_tokens: 100,
            cost: 0.05,
            conversation_count: 1,
            message_count: 5,
        };

        persistence.upsert_daily_metrics(&metrics).await.unwrap();

        // Retrieve and verify
        let retrieved = persistence.get_metrics_for_date("2025-11-17").await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].model, "claude-opus-4");
        assert_eq!(retrieved[0].input_tokens, 1000);
        assert_eq!(retrieved[0].cost, 0.05);
    }

    #[tokio::test]
    async fn test_get_metrics_range() {
        let (persistence, _temp_dir) = create_test_persistence().await;

        // Insert metrics for multiple days
        for i in 15..20 {
            let metrics = DailyModelMetrics {
                date: format!("2025-11-{:02}", i),
                model: "claude-sonnet-4-5".to_string(),
                input_tokens: 100 * i,
                output_tokens: 50 * i,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                cost: 0.01 * i as f64,
                conversation_count: 1,
                message_count: i,
            };
            persistence.upsert_daily_metrics(&metrics).await.unwrap();
        }

        let metrics = persistence.get_metrics_range("2025-11-16", "2025-11-18").await.unwrap();
        assert_eq!(metrics.len(), 3);
    }

    #[tokio::test]
    async fn test_get_daily_totals() {
        let (persistence, _temp_dir) = create_test_persistence().await;

        // Insert metrics for multiple models on same day
        let models = vec!["claude-opus-4", "claude-sonnet-4-5", "claude-haiku-4-5"];
        for model in models {
            let metrics = DailyModelMetrics {
                date: "2025-11-17".to_string(),
                model: model.to_string(),
                input_tokens: 1000,
                output_tokens: 500,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                cost: 0.01,
                conversation_count: 1,
                message_count: 5,
            };
            persistence.upsert_daily_metrics(&metrics).await.unwrap();
        }

        let totals = persistence.get_daily_totals("2025-11-17", "2025-11-17").await.unwrap();
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].date, "2025-11-17");
        assert_eq!(totals[0].models.len(), 3);
        assert!((totals[0].cost - 0.03).abs() < 0.001); // 3 models × $0.01 each
    }
}
