//! Audit logging with SQLite backend
//!
//! Stores full request/response bodies for debugging and compliance.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use tracing::info;

use super::config::AuditConfig;
use super::{CompletionRequest, CompletionResponse, RequestMetrics};

/// Audit logger with SQLite backend
pub struct AuditLogger {
    pool: SqlitePool,
    config: AuditConfig,
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new(config: &AuditConfig) -> Result<Self> {
        let db_path = config.get_db_path();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create audit log directory")?;
        }

        // Connect to SQLite
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await
            .context("Failed to connect to audit database")?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        info!("Audit logger initialized at {:?}", db_path);

        Ok(Self {
            pool,
            config: config.clone(),
        })
    }

    /// Run database migrations
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                agent_type TEXT,
                project_id TEXT,
                input_tokens INTEGER,
                output_tokens INTEGER,
                cache_write_tokens INTEGER,
                cache_read_tokens INTEGER,
                cost_usd REAL,
                latency_ms INTEGER,
                status TEXT NOT NULL,
                request_body TEXT,
                response_body TEXT,
                error_message TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create audit_log table")?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON audit_log(timestamp)")
            .execute(pool)
            .await
            .context("Failed to create timestamp index")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_agent_type ON audit_log(agent_type)")
            .execute(pool)
            .await
            .context("Failed to create agent_type index")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_provider ON audit_log(provider)")
            .execute(pool)
            .await
            .context("Failed to create provider index")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_id ON audit_log(project_id)")
            .execute(pool)
            .await
            .context("Failed to create project_id index")?;

        Ok(())
    }

    /// Log a successful request
    pub async fn log_success(
        &self,
        request_id: &str,
        request: &CompletionRequest,
        response: &CompletionResponse,
        metrics: &RequestMetrics,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let request_body = if self.config.log_request_bodies {
            serde_json::to_string(request).ok()
        } else {
            None
        };

        let response_body = if self.config.log_response_bodies {
            serde_json::to_string(response).ok()
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, timestamp, provider, model, agent_type, project_id,
                input_tokens, output_tokens, cache_write_tokens, cache_read_tokens,
                cost_usd, latency_ms, status, request_body, response_body, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(request_id)
        .bind(metrics.timestamp.to_rfc3339())
        .bind(&metrics.provider)
        .bind(&metrics.model)
        .bind(&metrics.agent_type)
        .bind(&metrics.project_id)
        .bind(metrics.input_tokens as i64)
        .bind(metrics.output_tokens as i64)
        .bind(metrics.cache_write_tokens as i64)
        .bind(metrics.cache_read_tokens as i64)
        .bind(metrics.cost_usd)
        .bind(metrics.latency_ms as i64)
        .bind("success")
        .bind(request_body)
        .bind(response_body)
        .bind::<Option<String>>(None)
        .execute(&self.pool)
        .await
        .context("Failed to insert audit log entry")?;

        Ok(())
    }

    /// Log a failed request
    pub async fn log_error(
        &self,
        request_id: &str,
        request: &CompletionRequest,
        provider: &str,
        error: &str,
        latency_ms: u64,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let request_body = if self.config.log_request_bodies {
            serde_json::to_string(request).ok()
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, timestamp, provider, model, agent_type, project_id,
                input_tokens, output_tokens, cache_write_tokens, cache_read_tokens,
                cost_usd, latency_ms, status, request_body, response_body, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(request_id)
        .bind(Utc::now().to_rfc3339())
        .bind(provider)
        .bind(&request.model)
        .bind(&request.agent_type)
        .bind(&request.project_id)
        .bind(0i64)
        .bind(0i64)
        .bind(0i64)
        .bind(0i64)
        .bind(0.0f64)
        .bind(latency_ms as i64)
        .bind("error")
        .bind(request_body)
        .bind::<Option<String>>(None)
        .bind(Some(error))
        .execute(&self.pool)
        .await
        .context("Failed to insert error audit log entry")?;

        Ok(())
    }

    /// Get recent audit entries
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, timestamp, provider, model, agent_type, project_id,
                   input_tokens, output_tokens, cache_write_tokens, cache_read_tokens,
                   cost_usd, latency_ms, status, request_body, response_body, error_message
            FROM audit_log
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch recent audit entries")?;

        let entries = rows
            .iter()
            .map(|row| AuditEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                provider: row.get("provider"),
                model: row.get("model"),
                agent_type: row.get("agent_type"),
                project_id: row.get("project_id"),
                input_tokens: row.get::<i64, _>("input_tokens") as u32,
                output_tokens: row.get::<i64, _>("output_tokens") as u32,
                cache_write_tokens: row.get::<i64, _>("cache_write_tokens") as u32,
                cache_read_tokens: row.get::<i64, _>("cache_read_tokens") as u32,
                cost_usd: row.get("cost_usd"),
                latency_ms: row.get::<i64, _>("latency_ms") as u64,
                status: row.get("status"),
                request_body: row.get("request_body"),
                response_body: row.get("response_body"),
                error_message: row.get("error_message"),
            })
            .collect();

        Ok(entries)
    }

    /// Get audit entry by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<AuditEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, timestamp, provider, model, agent_type, project_id,
                   input_tokens, output_tokens, cache_write_tokens, cache_read_tokens,
                   cost_usd, latency_ms, status, request_body, response_body, error_message
            FROM audit_log
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch audit entry")?;

        Ok(row.map(|row| AuditEntry {
            id: row.get("id"),
            timestamp: row.get("timestamp"),
            provider: row.get("provider"),
            model: row.get("model"),
            agent_type: row.get("agent_type"),
            project_id: row.get("project_id"),
            input_tokens: row.get::<i64, _>("input_tokens") as u32,
            output_tokens: row.get::<i64, _>("output_tokens") as u32,
            cache_write_tokens: row.get::<i64, _>("cache_write_tokens") as u32,
            cache_read_tokens: row.get::<i64, _>("cache_read_tokens") as u32,
            cost_usd: row.get("cost_usd"),
            latency_ms: row.get::<i64, _>("latency_ms") as u64,
            status: row.get("status"),
            request_body: row.get("request_body"),
            response_body: row.get("response_body"),
            error_message: row.get("error_message"),
        }))
    }

    /// Search audit entries
    pub async fn search(&self, query: AuditQuery) -> Result<Vec<AuditEntry>> {
        let mut sql = String::from(
            r#"
            SELECT id, timestamp, provider, model, agent_type, project_id,
                   input_tokens, output_tokens, cache_write_tokens, cache_read_tokens,
                   cost_usd, latency_ms, status, request_body, response_body, error_message
            FROM audit_log
            WHERE 1=1
            "#,
        );

        // Build dynamic WHERE clauses
        if query.provider.is_some() {
            sql.push_str(" AND provider = ?");
        }
        if query.agent_type.is_some() {
            sql.push_str(" AND agent_type = ?");
        }
        if query.project_id.is_some() {
            sql.push_str(" AND project_id = ?");
        }
        if query.status.is_some() {
            sql.push_str(" AND status = ?");
        }
        if query.from_timestamp.is_some() {
            sql.push_str(" AND timestamp >= ?");
        }
        if query.to_timestamp.is_some() {
            sql.push_str(" AND timestamp <= ?");
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");

        let mut query_builder = sqlx::query(&sql);

        // Bind parameters in order
        if let Some(ref provider) = query.provider {
            query_builder = query_builder.bind(provider);
        }
        if let Some(ref agent_type) = query.agent_type {
            query_builder = query_builder.bind(agent_type);
        }
        if let Some(ref project_id) = query.project_id {
            query_builder = query_builder.bind(project_id);
        }
        if let Some(ref status) = query.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(from) = query.from_timestamp {
            query_builder = query_builder.bind(from.to_rfc3339());
        }
        if let Some(to) = query.to_timestamp {
            query_builder = query_builder.bind(to.to_rfc3339());
        }
        query_builder = query_builder.bind(query.limit.unwrap_or(100) as i64);

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to search audit entries")?;

        let entries = rows
            .iter()
            .map(|row| AuditEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                provider: row.get("provider"),
                model: row.get("model"),
                agent_type: row.get("agent_type"),
                project_id: row.get("project_id"),
                input_tokens: row.get::<i64, _>("input_tokens") as u32,
                output_tokens: row.get::<i64, _>("output_tokens") as u32,
                cache_write_tokens: row.get::<i64, _>("cache_write_tokens") as u32,
                cache_read_tokens: row.get::<i64, _>("cache_read_tokens") as u32,
                cost_usd: row.get("cost_usd"),
                latency_ms: row.get::<i64, _>("latency_ms") as u64,
                status: row.get("status"),
                request_body: row.get("request_body"),
                response_body: row.get("response_body"),
                error_message: row.get("error_message"),
            })
            .collect();

        Ok(entries)
    }

    /// Cleanup old entries based on retention policy
    pub async fn cleanup(&self) -> Result<u64> {
        let cutoff = Utc::now() - Duration::days(self.config.retention_days as i64);

        let result = sqlx::query("DELETE FROM audit_log WHERE timestamp < ?")
            .bind(cutoff.to_rfc3339())
            .execute(&self.pool)
            .await
            .context("Failed to cleanup old audit entries")?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            info!("Cleaned up {} old audit entries", deleted);
        }

        Ok(deleted)
    }

    /// Get total count of entries
    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM audit_log")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count audit entries")?;

        Ok(row.get("count"))
    }
}

/// Audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub provider: String,
    pub model: String,
    pub agent_type: Option<String>,
    pub project_id: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_write_tokens: u32,
    pub cache_read_tokens: u32,
    pub cost_usd: f64,
    pub latency_ms: u64,
    pub status: String,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
}

/// Query parameters for audit search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    pub provider: Option<String>,
    pub agent_type: Option<String>,
    pub project_id: Option<String>,
    pub status: Option<String>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn test_logger() -> AuditLogger {
        let temp_dir = tempdir().unwrap();
        let config = AuditConfig {
            enabled: true,
            log_request_bodies: true,
            log_response_bodies: true,
            retention_days: 30,
            db_path: Some(temp_dir.path().join("test_audit.db")),
        };
        AuditLogger::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let logger = test_logger().await;
        assert_eq!(logger.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_audit_log_success() {
        let logger = test_logger().await;

        let request = CompletionRequest {
            model: "claude-sonnet-4-5".to_string(),
            messages: vec![],
            max_tokens: 1000,
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: false,
            agent_type: Some("test-agent".to_string()),
            project_id: Some("test-project".to_string()),
        };

        let response = CompletionResponse {
            id: "resp-123".to_string(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            model: "claude-sonnet-4-5".to_string(),
            content: vec![],
            stop_reason: Some("end_turn".to_string()),
            stop_sequence: None,
            usage: super::super::Usage::default(),
            provider: "anthropic".to_string(),
            latency_ms: 2500,
            cost_usd: 0.01,
        };

        let metrics = RequestMetrics {
            timestamp: Utc::now(),
            request_id: "req-123".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5".to_string(),
            agent_type: Some("test-agent".to_string()),
            project_id: Some("test-project".to_string()),
            input_tokens: 1000,
            output_tokens: 500,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
            cost_usd: 0.01,
            latency_ms: 2500,
        };

        logger
            .log_success("req-123", &request, &response, &metrics)
            .await
            .unwrap();

        let entries = logger.get_recent(10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "req-123");
        assert_eq!(entries[0].status, "success");
    }

    #[tokio::test]
    async fn test_audit_search() {
        let logger = test_logger().await;

        // Add some test entries
        for i in 0..5 {
            let request = CompletionRequest {
                model: "claude-sonnet-4-5".to_string(),
                messages: vec![],
                max_tokens: 1000,
                system: None,
                temperature: None,
                top_p: None,
                top_k: None,
                stop_sequences: None,
                stream: false,
                agent_type: Some(format!("agent-{}", i % 2)),
                project_id: None,
            };

            let response = CompletionResponse {
                id: format!("resp-{}", i),
                response_type: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-sonnet-4-5".to_string(),
                content: vec![],
                stop_reason: Some("end_turn".to_string()),
                stop_sequence: None,
                usage: super::super::Usage::default(),
                provider: "anthropic".to_string(),
                latency_ms: 2500,
                cost_usd: 0.01,
            };

            let metrics = RequestMetrics {
                timestamp: Utc::now(),
                request_id: format!("req-{}", i),
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-5".to_string(),
                agent_type: Some(format!("agent-{}", i % 2)),
                project_id: None,
                input_tokens: 1000,
                output_tokens: 500,
                cache_write_tokens: 0,
                cache_read_tokens: 0,
                cost_usd: 0.01,
                latency_ms: 2500,
            };

            logger
                .log_success(&format!("req-{}", i), &request, &response, &metrics)
                .await
                .unwrap();
        }

        // Search by agent type
        let query = AuditQuery {
            agent_type: Some("agent-0".to_string()),
            ..Default::default()
        };

        let results = logger.search(query).await.unwrap();
        assert_eq!(results.len(), 3); // 0, 2, 4
    }
}
