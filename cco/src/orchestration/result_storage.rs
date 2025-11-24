//! Result Storage Component
//!
//! Persists agent results to JSON files in ~/.cco/orchestration/results/
//! Features project isolation, automatic cleanup, and thread-safe concurrent writes.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;

const RETENTION_DAYS: i64 = 30;

/// Result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResultMetadata {
    pub id: String,
    pub issue_id: String,
    pub agent_type: String,
    pub stored_at: DateTime<Utc>,
    pub size_bytes: usize,
}

/// Result storage manager
pub struct ResultStorage {
    base_path: PathBuf,
    metadata: RwLock<Vec<ResultMetadata>>,
}

impl ResultStorage {
    /// Create a new result storage
    pub fn new(base_path: String) -> Result<Self> {
        let path = PathBuf::from(base_path);

        // Create directories if they don't exist
        fs::create_dir_all(&path).context("Failed to create storage directory")?;

        let results_path = path.join("results");
        fs::create_dir_all(&results_path).context("Failed to create results directory")?;

        Ok(Self {
            base_path: path,
            metadata: RwLock::new(Vec::new()),
        })
    }

    /// Store a result
    pub async fn store_result(
        &self,
        issue_id: &str,
        agent_type: &str,
        result: &serde_json::Value,
    ) -> Result<String> {
        let result_id = uuid::Uuid::new_v4().to_string();

        // Create issue-specific directory
        let issue_dir = self.base_path.join("results").join(issue_id);
        fs::create_dir_all(&issue_dir).context("Failed to create issue directory")?;

        // Write result to file
        let result_path = issue_dir.join(format!("{}.json", agent_type));
        let result_json = serde_json::to_string_pretty(result)?;
        fs::write(&result_path, &result_json).context("Failed to write result file")?;

        // Update metadata
        let metadata = ResultMetadata {
            id: result_id.clone(),
            issue_id: issue_id.to_string(),
            agent_type: agent_type.to_string(),
            stored_at: Utc::now(),
            size_bytes: result_json.len(),
        };

        let mut meta_list = self.metadata.write().await;
        meta_list.push(metadata);

        // Cleanup old results
        self.cleanup_old_results().await?;

        Ok(result_id)
    }

    /// Query results by issue ID
    pub async fn query_by_issue(&self, issue_id: &str) -> Result<Vec<serde_json::Value>> {
        let issue_dir = self.base_path.join("results").join(issue_id);

        if !issue_dir.exists() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();

        for entry in fs::read_dir(issue_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let value: serde_json::Value = serde_json::from_str(&content)?;
                results.push(value);
            }
        }

        Ok(results)
    }

    /// Query results by agent type
    pub async fn query_by_agent(&self, agent_type: &str) -> Result<Vec<serde_json::Value>> {
        let results_dir = self.base_path.join("results");

        if !results_dir.exists() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();

        // Scan all issue directories
        for issue_entry in fs::read_dir(results_dir)? {
            let issue_entry = issue_entry?;
            if !issue_entry.file_type()?.is_dir() {
                continue;
            }

            let agent_file = issue_entry.path().join(format!("{}.json", agent_type));
            if agent_file.exists() {
                let content = fs::read_to_string(&agent_file)?;
                let value: serde_json::Value = serde_json::from_str(&content)?;
                results.push(value);
            }
        }

        Ok(results)
    }

    /// Query results by timestamp range
    #[allow(dead_code)]
    pub async fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ResultMetadata>> {
        let metadata = self.metadata.read().await;

        Ok(metadata
            .iter()
            .filter(|m| m.stored_at >= start && m.stored_at <= end)
            .cloned()
            .collect())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        let metadata = self.metadata.read().await;

        let total_results = metadata.len();
        let total_size: usize = metadata.iter().map(|m| m.size_bytes).sum();

        StorageStats {
            total_results,
            total_size_bytes: total_size,
            total_size_mb: (total_size as f64) / (1024.0 * 1024.0),
        }
    }

    /// Cleanup old results beyond retention period
    async fn cleanup_old_results(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(RETENTION_DAYS);
        let results_dir = self.base_path.join("results");

        if !results_dir.exists() {
            return Ok(());
        }

        let mut removed_count = 0;

        // Scan all issue directories
        for issue_entry in fs::read_dir(&results_dir)? {
            let issue_entry = issue_entry?;
            if !issue_entry.file_type()?.is_dir() {
                continue;
            }

            let issue_path = issue_entry.path();

            // Check all result files in this issue
            let mut all_files_old = true;
            for result_entry in fs::read_dir(&issue_path)? {
                let result_entry = result_entry?;
                let metadata = result_entry.metadata()?;

                if let Ok(modified) = metadata.modified() {
                    let modified_datetime: DateTime<Utc> = modified.into();
                    if modified_datetime >= cutoff {
                        all_files_old = false;
                        break;
                    }
                }
            }

            // If all files are old, remove the entire issue directory
            if all_files_old {
                fs::remove_dir_all(&issue_path)?;
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            tracing::info!(
                "Cleaned up {} old issue directories (retention: {} days)",
                removed_count,
                RETENTION_DAYS
            );
        }

        Ok(())
    }

    /// Clear all results (for testing)
    pub async fn clear_all(&self) -> Result<usize> {
        let results_dir = self.base_path.join("results");

        if !results_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;

        for entry in fs::read_dir(&results_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                fs::remove_dir_all(entry.path())?;
                count += 1;
            }
        }

        let mut metadata = self.metadata.write().await;
        metadata.clear();

        Ok(count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_results: usize,
    pub total_size_bytes: usize,
    pub total_size_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_store_and_query() {
        let temp_dir = tempdir().unwrap();
        let storage = ResultStorage::new(temp_dir.path().to_string_lossy().to_string()).unwrap();

        let result = serde_json::json!({
            "status": "success",
            "message": "Test completed"
        });

        let result_id = storage
            .store_result("issue-123", "test-agent", &result)
            .await
            .unwrap();

        assert!(!result_id.is_empty());

        let results = storage.query_by_issue("issue-123").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["status"], "success");
    }

    #[tokio::test]
    async fn test_query_by_agent() {
        let temp_dir = tempdir().unwrap();
        let storage = ResultStorage::new(temp_dir.path().to_string_lossy().to_string()).unwrap();

        let result1 = serde_json::json!({"issue": "issue-1"});
        let result2 = serde_json::json!({"issue": "issue-2"});

        storage
            .store_result("issue-1", "python-specialist", &result1)
            .await
            .unwrap();

        storage
            .store_result("issue-2", "python-specialist", &result2)
            .await
            .unwrap();

        let results = storage.query_by_agent("python-specialist").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let temp_dir = tempdir().unwrap();
        let storage = ResultStorage::new(temp_dir.path().to_string_lossy().to_string()).unwrap();

        let result = serde_json::json!({"test": "data"});
        storage
            .store_result("issue-1", "test-agent", &result)
            .await
            .unwrap();

        let stats = storage.get_stats().await;
        assert_eq!(stats.total_results, 1);
        assert!(stats.total_size_bytes > 0);
    }
}
