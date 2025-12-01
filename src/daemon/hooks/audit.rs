//! Audit logging and decision history tracking
//!
//! Provides persistent storage for permission decisions with:
//! - SQLite database for decision history
//! - Statistics and reporting
//! - Automatic cleanup of old records
//! - Async, non-blocking operations

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::{debug, error, info, warn};

use super::error::{HookError, HookResult};
use super::permissions::PermissionDecision;
use super::types::CrudClassification;
use crate::daemon::security::CredentialDetector;

/// Decision record stored in the audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique decision ID
    pub id: i64,

    /// The command that was classified
    pub command: String,

    /// CRUD classification
    pub classification: CrudClassification,

    /// Timestamp of the decision
    pub timestamp: DateTime<Utc>,

    /// User's decision (APPROVED, DENIED, etc.)
    pub user_decision: PermissionDecision,

    /// Reasoning for the classification
    pub reasoning: Option<String>,

    /// Confidence score (0.0 - 1.0)
    pub confidence_score: Option<f32>,

    /// Response time in milliseconds
    pub response_time_ms: Option<i32>,
}

/// Statistics about decision history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionStats {
    /// Number of READ operations
    pub read_count: i64,

    /// Number of CREATE operations
    pub create_count: i64,

    /// Number of UPDATE operations
    pub update_count: i64,

    /// Number of DELETE operations
    pub delete_count: i64,

    /// Total number of requests
    pub total_requests: i64,

    /// Number of approved decisions
    pub approved_count: i64,

    /// Number of denied decisions
    pub denied_count: i64,

    /// Number of pending decisions
    pub pending_count: i64,

    /// Number of skipped decisions
    pub skipped_count: i64,
}

impl Default for DecisionStats {
    fn default() -> Self {
        Self {
            read_count: 0,
            create_count: 0,
            update_count: 0,
            delete_count: 0,
            total_requests: 0,
            approved_count: 0,
            denied_count: 0,
            pending_count: 0,
            skipped_count: 0,
        }
    }
}

/// Trait for decision database operations
///
/// Abstraction to allow different storage backends (SQLite, in-memory, etc.)
#[async_trait::async_trait]
pub trait DecisionDatabase: Send + Sync {
    /// Store a decision in the database
    async fn store_decision(&self, decision: Decision) -> HookResult<()>;

    /// Get recent decisions with pagination
    async fn get_recent_decisions(&self, limit: usize, offset: usize) -> HookResult<Vec<Decision>>;

    /// Get decision statistics
    async fn get_stats(&self) -> HookResult<DecisionStats>;

    /// Clean up old decisions (older than specified days)
    async fn cleanup_old_decisions(&self, days: u32) -> HookResult<u64>;

    /// Close the database connection
    async fn close(&self) -> HookResult<()>;
}

/// SQLite implementation of the decision database
pub struct SqliteAuditDatabase {
    pool: SqlitePool,
    db_path: PathBuf,
    credential_detector: CredentialDetector,
}

impl SqliteAuditDatabase {
    /// Create a new SQLite audit database
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created or initialized
    pub async fn new(db_path: PathBuf) -> HookResult<Self> {
        info!("Initializing audit database at: {}", db_path.display());

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                HookError::execution_failed(
                    "audit_db_init",
                    format!("Failed to create database directory: {}", e),
                )
            })?;
        }

        // Create connection options with read-write mode
        let connection_str = format!("sqlite://{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&connection_str)
            .map_err(|e| {
                HookError::execution_failed(
                    "audit_db_init",
                    format!("Failed to create connection options: {}", e),
                )
            })?
            .create_if_missing(true)
            .read_only(false);

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| {
                HookError::execution_failed(
                    "audit_db_init",
                    format!("Failed to connect to database: {}", e),
                )
            })?;

        let db = Self {
            pool,
            db_path: db_path.clone(),
            credential_detector: CredentialDetector::new(),
        };

        // Initialize schema
        db.init_schema().await?;

        // Set secure file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if db_path.exists() {
                match std::fs::metadata(&db_path) {
                    Ok(metadata) => {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o600); // Owner read/write only
                        if let Err(e) = std::fs::set_permissions(&db_path, perms) {
                            warn!("Failed to set secure permissions on audit database: {}", e);
                        } else {
                            info!("✅ Secure file permissions (600) set on audit database");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get audit database metadata: {}", e);
                    }
                }
            }
        }

        info!("✅ Audit database initialized successfully");
        Ok(db)
    }

    /// Initialize the database schema
    async fn init_schema(&self) -> HookResult<()> {
        debug!("Initializing database schema");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                classification TEXT NOT NULL CHECK(classification IN ('Read', 'Create', 'Update', 'Delete')),
                timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                user_decision TEXT NOT NULL CHECK(user_decision IN ('Approved', 'Denied', 'Pending', 'Skipped')),
                reasoning TEXT,
                confidence_score REAL CHECK(confidence_score BETWEEN 0.0 AND 1.0),
                response_time_ms INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            HookError::execution_failed(
                "schema_init",
                format!("Failed to create decisions table: {}", e),
            )
        })?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON decisions(timestamp DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                HookError::execution_failed(
                    "schema_init",
                    format!("Failed to create timestamp index: {}", e),
                )
            })?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_classification ON decisions(classification)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                HookError::execution_failed(
                    "schema_init",
                    format!("Failed to create classification index: {}", e),
                )
            })?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_decision ON decisions(user_decision)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                HookError::execution_failed(
                    "schema_init",
                    format!("Failed to create user_decision index: {}", e),
                )
            })?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Get the database file path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Sanitize command by redacting detected credentials
    ///
    /// This method scans the command for credential patterns and redacts them
    /// using prefix***suffix format before storage in the audit database.
    fn sanitize_command(&self, command: &str) -> String {
        let matches = self.credential_detector.detect(command);

        if matches.is_empty() {
            return command.to_string();
        }

        // Build sanitized string by replacing matches
        let mut result = String::new();
        let mut last_pos = 0;

        // Process matches in forward order, skipping overlaps
        let mut sorted_matches = matches;
        sorted_matches.sort_by(|a, b| a.start_pos.cmp(&b.start_pos));

        for credential_match in sorted_matches {
            // Skip if this match overlaps with already processed text
            if credential_match.start_pos < last_pos {
                continue;
            }

            // Append text before the match
            result.push_str(&command[last_pos..credential_match.start_pos]);

            // Append redacted credential
            result.push_str(&credential_match.matched_text);

            // Update position
            last_pos = credential_match.end_pos;
        }

        // Append remaining text
        result.push_str(&command[last_pos..]);

        result
    }
}

#[async_trait::async_trait]
impl DecisionDatabase for SqliteAuditDatabase {
    async fn store_decision(&self, decision: Decision) -> HookResult<()> {
        debug!(
            "Storing decision: {} ({})",
            decision.command, decision.classification
        );

        // Sanitize command to remove credentials before storage
        let sanitized_command = self.sanitize_command(&decision.command);

        // Warn if credentials were detected and sanitized
        if sanitized_command != decision.command {
            warn!(
                "Credentials detected in command and sanitized before audit storage: {}",
                sanitized_command
            );
        }

        let classification_str = format!("{:?}", decision.classification);
        let user_decision_str = format!("{:?}", decision.user_decision);

        sqlx::query(
            r#"
            INSERT INTO decisions (command, classification, timestamp, user_decision, reasoning, confidence_score, response_time_ms)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&sanitized_command)
        .bind(classification_str)
        .bind(decision.timestamp.to_rfc3339())
        .bind(user_decision_str)
        .bind(&decision.reasoning)
        .bind(decision.confidence_score)
        .bind(decision.response_time_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to store decision: {}", e);
            HookError::execution_failed("store_decision", format!("Database error: {}", e))
        })?;

        debug!("Decision stored successfully (credentials sanitized if present)");
        Ok(())
    }

    async fn get_recent_decisions(&self, limit: usize, offset: usize) -> HookResult<Vec<Decision>> {
        debug!(
            "Fetching recent decisions (limit: {}, offset: {})",
            limit, offset
        );

        let rows = sqlx::query(
            r#"
            SELECT id, command, classification, timestamp, user_decision, reasoning, confidence_score, response_time_ms
            FROM decisions
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch decisions: {}", e);
            HookError::execution_failed("get_recent_decisions", format!("Database error: {}", e))
        })?;

        let mut decisions = Vec::new();
        for row in rows {
            let classification_str: String = row.try_get("classification").map_err(|e| {
                HookError::execution_failed(
                    "parse_decision",
                    format!("Failed to get classification: {}", e),
                )
            })?;
            let user_decision_str: String = row.try_get("user_decision").map_err(|e| {
                HookError::execution_failed(
                    "parse_decision",
                    format!("Failed to get user_decision: {}", e),
                )
            })?;

            let classification = match classification_str.as_str() {
                "Read" => CrudClassification::Read,
                "Create" => CrudClassification::Create,
                "Update" => CrudClassification::Update,
                "Delete" => CrudClassification::Delete,
                _ => {
                    return Err(HookError::execution_failed(
                        "parse_decision",
                        format!("Invalid classification: {}", classification_str),
                    ))
                }
            };

            let user_decision = match user_decision_str.as_str() {
                "Approved" => PermissionDecision::Approved,
                "Denied" => PermissionDecision::Denied,
                "Pending" => PermissionDecision::Pending,
                "Skipped" => PermissionDecision::Skipped,
                _ => {
                    return Err(HookError::execution_failed(
                        "parse_decision",
                        format!("Invalid user_decision: {}", user_decision_str),
                    ))
                }
            };

            // Parse timestamp from RFC3339 string
            let timestamp_str: String = row.try_get("timestamp").map_err(|e| {
                HookError::execution_failed(
                    "parse_decision",
                    format!("Failed to get timestamp: {}", e),
                )
            })?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| {
                    HookError::execution_failed(
                        "parse_decision",
                        format!("Failed to parse timestamp: {}", e),
                    )
                })?
                .with_timezone(&Utc);

            decisions.push(Decision {
                id: row.try_get("id").map_err(|e| {
                    HookError::execution_failed(
                        "parse_decision",
                        format!("Failed to get id: {}", e),
                    )
                })?,
                command: row.try_get("command").map_err(|e| {
                    HookError::execution_failed(
                        "parse_decision",
                        format!("Failed to get command: {}", e),
                    )
                })?,
                classification,
                timestamp,
                user_decision,
                reasoning: row.try_get("reasoning").ok(),
                confidence_score: row.try_get("confidence_score").ok(),
                response_time_ms: row.try_get("response_time_ms").ok(),
            });
        }

        debug!("Fetched {} decisions", decisions.len());
        Ok(decisions)
    }

    async fn get_stats(&self) -> HookResult<DecisionStats> {
        debug!("Calculating decision statistics");

        // Count by classification
        let read_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE classification = 'Read'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let create_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE classification = 'Create'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let update_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE classification = 'Update'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let delete_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE classification = 'Delete'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        // Count by decision
        let approved_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE user_decision = 'Approved'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let denied_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE user_decision = 'Denied'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let pending_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE user_decision = 'Pending'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let skipped_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM decisions WHERE user_decision = 'Skipped'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let total_requests = read_count + create_count + update_count + delete_count;

        debug!("Statistics calculated: {} total requests", total_requests);

        Ok(DecisionStats {
            read_count,
            create_count,
            update_count,
            delete_count,
            total_requests,
            approved_count,
            denied_count,
            pending_count,
            skipped_count,
        })
    }

    async fn cleanup_old_decisions(&self, days: u32) -> HookResult<u64> {
        info!("Cleaning up decisions older than {} days", days);

        let cutoff = Utc::now() - ChronoDuration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let result = sqlx::query("DELETE FROM decisions WHERE timestamp < ?")
            .bind(cutoff_str)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!("Failed to cleanup old decisions: {}", e);
                HookError::execution_failed(
                    "cleanup_old_decisions",
                    format!("Database error: {}", e),
                )
            })?;

        let deleted = result.rows_affected();
        info!("Cleaned up {} old decisions", deleted);

        Ok(deleted)
    }

    async fn close(&self) -> HookResult<()> {
        info!("Closing audit database");
        self.pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> SqliteAuditDatabase {
        // Create temp directory and database file in it
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_decisions.db");

        // Create the database
        let db = SqliteAuditDatabase::new(db_path).await.unwrap();

        // Keep the temp directory alive by leaking it (for test duration)
        std::mem::forget(dir);

        db
    }

    fn create_test_decision() -> Decision {
        Decision {
            id: 0,
            command: "ls -la".to_string(),
            classification: CrudClassification::Read,
            timestamp: Utc::now(),
            user_decision: PermissionDecision::Approved,
            reasoning: Some("Safe READ operation".to_string()),
            confidence_score: Some(0.95),
            response_time_ms: Some(100),
        }
    }

    #[tokio::test]
    async fn test_database_creation() {
        let db = create_test_db().await;
        // Database file might not exist until first write
        // Just verify the database object was created successfully
        assert!(db.db_path().parent().is_some());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_decision() {
        let db = create_test_db().await;
        let decision = create_test_decision();

        // Store decision
        db.store_decision(decision.clone()).await.unwrap();

        // Retrieve decisions
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].command, "ls -la");
        assert_eq!(decisions[0].classification, CrudClassification::Read);
    }

    #[tokio::test]
    async fn test_decision_stats() {
        let db = create_test_db().await;

        // Store various decisions
        let mut decision = create_test_decision();
        db.store_decision(decision.clone()).await.unwrap();

        decision.classification = CrudClassification::Create;
        decision.command = "mkdir test".to_string();
        db.store_decision(decision.clone()).await.unwrap();

        decision.classification = CrudClassification::Delete;
        decision.command = "rm file.txt".to_string();
        db.store_decision(decision).await.unwrap();

        // Get stats
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.read_count, 1);
        assert_eq!(stats.create_count, 1);
        assert_eq!(stats.delete_count, 1);
        assert_eq!(stats.approved_count, 3);
    }

    #[tokio::test]
    async fn test_cleanup_old_decisions() {
        let db = create_test_db().await;

        // Store old decision
        let mut old_decision = create_test_decision();
        old_decision.timestamp = Utc::now() - ChronoDuration::days(10);
        db.store_decision(old_decision).await.unwrap();

        // Store recent decision
        let recent_decision = create_test_decision();
        db.store_decision(recent_decision).await.unwrap();

        // Cleanup decisions older than 7 days
        let deleted = db.cleanup_old_decisions(7).await.unwrap();
        assert_eq!(deleted, 1);

        // Verify only recent decision remains
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_pagination() {
        let db = create_test_db().await;

        // Store multiple decisions
        for i in 0..5 {
            let mut decision = create_test_decision();
            decision.command = format!("command_{}", i);
            db.store_decision(decision).await.unwrap();
        }

        // Test pagination
        let page1 = db.get_recent_decisions(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = db.get_recent_decisions(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = db.get_recent_decisions(2, 4).await.unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn test_close_database() {
        let db = create_test_db().await;
        assert!(db.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_credential_sanitization_api_key() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        decision.command =
            "curl -H 'api_key=sk_test_1234567890abcdef' https://api.example.com".to_string();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify sanitization
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Command should be sanitized (not contain the raw API key)
        assert!(!decisions[0].command.contains("sk_test_1234567890abcdef"));
        assert!(decisions[0].command.contains("***")); // Should contain redaction marker
    }

    #[tokio::test]
    async fn test_credential_sanitization_password() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        decision.command = "mysql -u root -p password=\"MySecurePassword123\" mydb".to_string();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify sanitization
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Command should be sanitized
        assert!(!decisions[0].command.contains("MySecurePassword123"));
        assert!(decisions[0].command.contains("***"));
    }

    #[tokio::test]
    async fn test_credential_sanitization_jwt_token() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        decision.command = "curl -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U' https://api.example.com".to_string();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify sanitization
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Command should be sanitized
        assert!(!decisions[0]
            .command
            .contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(decisions[0].command.contains("***"));
    }

    #[tokio::test]
    async fn test_credential_sanitization_github_token() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        decision.command =
            "git push https://ghp_1234567890abcdefghijklmnopqrstuv@github.com/user/repo"
                .to_string();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify sanitization
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Command should be sanitized
        assert!(!decisions[0]
            .command
            .contains("ghp_1234567890abcdefghijklmnopqrstuv"));
        assert!(decisions[0].command.contains("***"));
    }

    #[tokio::test]
    async fn test_credential_sanitization_database_url() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        decision.command = "postgres://user:MySecretPassword123@localhost:5432/mydb".to_string();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify sanitization
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Command should be sanitized
        assert!(!decisions[0].command.contains("MySecretPassword123"));
        assert!(decisions[0].command.contains("***"));
    }

    #[tokio::test]
    async fn test_no_sanitization_for_safe_commands() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        let safe_command = "ls -la /tmp/test_directory".to_string();
        decision.command = safe_command.clone();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify no sanitization occurred
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Command should be unchanged
        assert_eq!(decisions[0].command, safe_command);
        assert!(!decisions[0].command.contains("***"));
    }

    #[tokio::test]
    async fn test_multiple_credentials_sanitization() {
        let db = create_test_db().await;

        let mut decision = create_test_decision();
        decision.command = "curl -H 'api_key=\"abc123456789012345\"' -H 'password=\"secretpassword123\"' https://api.example.com".to_string();

        // Store decision
        db.store_decision(decision).await.unwrap();

        // Retrieve and verify both credentials sanitized
        let decisions = db.get_recent_decisions(10, 0).await.unwrap();
        assert_eq!(decisions.len(), 1);

        // Both credentials should be sanitized
        assert!(!decisions[0].command.contains("abc123456789012345"));
        assert!(!decisions[0].command.contains("secretpassword123"));
        assert!(decisions[0].command.contains("***"));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_database_file_permissions() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        // Create temp directory and database
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_permissions.db");

        let db = SqliteAuditDatabase::new(db_path.clone()).await.unwrap();

        // Store a decision to ensure file exists
        let decision = create_test_decision();
        db.store_decision(decision).await.unwrap();

        // Check file permissions
        let metadata = fs::metadata(&db_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Verify 0600 permissions (owner read/write only)
        assert_eq!(
            mode & 0o777,
            0o600,
            "Database file should have 0600 permissions"
        );
    }
}
