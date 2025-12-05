//! Audit logging for credential operations

use crate::credentials::keyring::{CredentialError, CredentialResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub credential_key: String,
    pub action: AuditAction,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub agent_name: Option<String>,
    pub ip_address: Option<String>,
}

/// Actions that can be audited
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Store,
    Retrieve,
    Update,
    Delete,
    List,
    Rotate,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store => write!(f, "store"),
            Self::Retrieve => write!(f, "retrieve"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::List => write!(f, "list"),
            Self::Rotate => write!(f, "rotate"),
        }
    }
}

/// Audit logger
pub struct AuditLogger {
    pool: sqlx::SqlitePool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> CredentialResult<Self> {
        let db_path = Self::get_audit_db_path()?;
        Self::new_with_path(db_path)
    }

    /// Create with custom database path
    pub fn new_with_path(db_path: PathBuf) -> CredentialResult<Self> {
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = tokio::runtime::Runtime::new()
            .map_err(|e| CredentialError::Other(format!("Failed to create runtime: {}", e)))?
            .block_on(async {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .connect(&db_url)
                    .await
                    .map_err(|e| CredentialError::Database(e))?;

                // Create table
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS credential_audit_log (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        credential_key TEXT NOT NULL,
                        action TEXT NOT NULL,
                        timestamp TEXT NOT NULL,
                        success BOOLEAN NOT NULL,
                        error_message TEXT,
                        agent_name TEXT,
                        ip_address TEXT
                    )
                    "#,
                )
                .execute(&pool)
                .await
                .map_err(|e| CredentialError::Database(e))?;

                // Create index
                sqlx::query(
                    r#"
                    CREATE INDEX IF NOT EXISTS idx_credential_key
                    ON credential_audit_log(credential_key)
                    "#,
                )
                .execute(&pool)
                .await
                .map_err(|e| CredentialError::Database(e))?;

                Ok::<_, CredentialError>(pool)
            })?;

        info!("Audit logger initialized");
        Ok(Self { pool })
    }

    /// Log a credential access
    pub async fn log_access(
        &self,
        key: &str,
        action: AuditAction,
        success: bool,
        error: Option<&str>,
    ) -> CredentialResult<()> {
        sqlx::query(
            r#"
            INSERT INTO credential_audit_log
            (credential_key, action, timestamp, success, error_message)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(action.to_string())
        .bind(Utc::now().to_rfc3339())
        .bind(success)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(CredentialError::Database)?;

        Ok(())
    }

    /// Get audit log for a credential
    pub async fn get_log(&self, key: &str, limit: i64) -> CredentialResult<Vec<AuditEntry>> {
        let rows = sqlx::query_as::<_, (i64, String, String, String, bool, Option<String>)>(
            r#"
            SELECT id, credential_key, action, timestamp, success, error_message
            FROM credential_audit_log
            WHERE credential_key = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(CredentialError::Database)?;

        let entries = rows
            .into_iter()
            .map(
                |(id, credential_key, action, timestamp, success, error_message)| AuditEntry {
                    id,
                    credential_key,
                    action: serde_json::from_str(&format!("\"{}\"", action))
                        .unwrap_or(AuditAction::Retrieve),
                    timestamp: DateTime::parse_from_rfc3339(&timestamp)
                        .unwrap_or_else(|_| chrono::Local::now().into())
                        .with_timezone(&Utc),
                    success,
                    error_message,
                    agent_name: None,
                    ip_address: None,
                },
            )
            .collect();

        Ok(entries)
    }

    /// Cleanup old audit entries
    pub async fn cleanup_old_entries(&self, days: i64) -> CredentialResult<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query(
            r#"
            DELETE FROM credential_audit_log
            WHERE timestamp < ?
            "#,
        )
        .bind(cutoff.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(CredentialError::Database)?;

        Ok(result.rows_affected())
    }

    fn get_audit_db_path() -> CredentialResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CredentialError::Other("Could not determine home directory".to_string())
        })?;
        let path = home.join(".cco").join("credential_audit.db");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CredentialError::Io(e))?;
        }

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_action_display() {
        assert_eq!(AuditAction::Store.to_string(), "store");
        assert_eq!(AuditAction::Retrieve.to_string(), "retrieve");
        assert_eq!(AuditAction::Delete.to_string(), "delete");
    }
}
