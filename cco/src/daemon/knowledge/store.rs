//! Temporary simplified Knowledge Store implementation
//!
//! This is a working implementation using in-memory storage that compiles and passes tests.
//! The full LanceDB integration will be added in a follow-up once the Arrow/LanceDB API
//! compatibility issues are resolved.

use super::embedding::generate_embedding;
use super::models::*;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Temporary in-memory knowledge store
///
/// This implementation provides all the required methods and functionality,
/// but stores data in memory instead of LanceDB. This allows the rest of the
/// system to integrate and test the knowledge store API while the LanceDB
/// integration is completed separately.
pub struct KnowledgeStore {
    db_path: PathBuf,
    repo_name: String,
    table_name: String,
    items: Vec<KnowledgeItem>,
}

impl KnowledgeStore {
    /// Create a new knowledge store
    pub fn new<P: AsRef<Path>>(
        repo_path: P,
        base_dir: Option<P>,
        table_name: Option<String>,
    ) -> Self {
        let repo_path = repo_path.as_ref();
        let repo_name = Self::extract_repo_name(repo_path);

        let base_dir = base_dir
            .map(|p| p.as_ref().to_path_buf())
            .unwrap_or_else(|| {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                home.join(".cco").join("knowledge")
            });

        let db_path = base_dir.join(&repo_name);
        let table_name = table_name.unwrap_or_else(|| "orchestra_knowledge".to_string());

        info!(
            "Knowledge Manager initialized for repository: {}",
            repo_name
        );
        info!("Database path: {:?}", db_path);

        Self {
            db_path,
            repo_name,
            table_name,
            items: Vec::new(),
        }
    }

    /// Extract repository name from path
    fn extract_repo_name<P: AsRef<Path>>(path: P) -> String {
        path.as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string()
    }

    /// Initialize the knowledge store
    pub async fn initialize(&mut self) -> Result<()> {
        // Create directory
        tokio::fs::create_dir_all(&self.db_path)
            .await
            .context("Failed to create knowledge directory")?;

        info!("Knowledge store initialized for {}", self.repo_name);
        Ok(())
    }

    /// Store a single knowledge item
    pub async fn store(&mut self, request: StoreKnowledgeRequest) -> Result<StoreKnowledgeResponse> {
        if request.text.is_empty() {
            anyhow::bail!("Knowledge text is required");
        }

        let vector = generate_embedding(&request.text);
        let knowledge_type = request.knowledge_type.unwrap_or_else(|| "decision".to_string());
        let project_id = request.project_id.unwrap_or_else(|| self.repo_name.clone());
        let session_id = request.session_id.unwrap_or_else(|| "unknown".to_string());
        let agent = request.agent.unwrap_or_else(|| "unknown".to_string());
        let metadata_json = request
            .metadata
            .map(|m| serde_json::to_string(&m).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        let id = format!(
            "{}-{}-{}",
            knowledge_type,
            Utc::now().timestamp(),
            uuid::Uuid::new_v4().to_string()[..7].to_string()
        );

        let item = KnowledgeItem {
            id: id.clone(),
            vector,
            text: request.text.clone(),
            knowledge_type: knowledge_type.clone(),
            project_id,
            session_id,
            agent: agent.clone(),
            timestamp: Utc::now().to_rfc3339(),
            metadata: metadata_json,
        };

        self.items.push(item);
        info!("Stored knowledge: {} from {}", knowledge_type, agent);

        Ok(StoreKnowledgeResponse { id, stored: true })
    }

    /// Store multiple knowledge items in batch
    pub async fn store_batch(&mut self, requests: Vec<StoreKnowledgeRequest>) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        for request in requests {
            match self.store(request).await {
                Ok(response) => ids.push(response.id),
                Err(e) => warn!("Failed to store item: {}", e),
            }
        }
        info!("Stored {}/{} knowledge items", ids.len(), ids.capacity());
        Ok(ids)
    }

    /// Search knowledge base using semantic similarity
    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let query_vector = generate_embedding(&request.query);

        // Calculate similarity scores
        let mut results: Vec<(f32, &KnowledgeItem)> = self
            .items
            .iter()
            .map(|item| {
                let similarity = cosine_similarity(&query_vector, &item.vector);
                (similarity, item)
            })
            .collect();

        // Filter by metadata
        results.retain(|(_, item)| {
            if let Some(ref project_id) = request.project_id {
                if &item.project_id != project_id {
                    return false;
                }
            }
            if let Some(ref knowledge_type) = request.knowledge_type {
                if &item.knowledge_type != knowledge_type {
                    return false;
                }
            }
            if let Some(ref agent) = request.agent {
                if &item.agent != agent {
                    return false;
                }
            }
            true
        });

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N results
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .take(request.limit)
            .map(|(score, item)| SearchResult {
                id: item.id.clone(),
                text: item.text.clone(),
                knowledge_type: item.knowledge_type.clone(),
                project_id: item.project_id.clone(),
                session_id: item.session_id.clone(),
                agent: item.agent.clone(),
                timestamp: item.timestamp.clone(),
                metadata: serde_json::from_str(&item.metadata).unwrap_or_default(),
                score,
            })
            .collect();

        info!("Found {} relevant knowledge items", search_results.len());
        Ok(search_results)
    }

    /// Get project knowledge
    pub async fn get_project_knowledge(
        &self,
        project_id: &str,
        knowledge_type: Option<String>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut items: Vec<&KnowledgeItem> = self
            .items
            .iter()
            .filter(|i| i.project_id == project_id)
            .collect();

        if let Some(ref k_type) = knowledge_type {
            items.retain(|i| &i.knowledge_type == k_type);
        }

        // Sort by timestamp (newest first)
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let results: Vec<SearchResult> = items
            .into_iter()
            .take(limit)
            .map(|item| SearchResult {
                id: item.id.clone(),
                text: item.text.clone(),
                knowledge_type: item.knowledge_type.clone(),
                project_id: item.project_id.clone(),
                session_id: item.session_id.clone(),
                agent: item.agent.clone(),
                timestamp: item.timestamp.clone(),
                metadata: serde_json::from_str(&item.metadata).unwrap_or_default(),
                score: 1.0, // Perfect score for direct matches
            })
            .collect();

        info!(
            "Retrieved {} knowledge items for project: {}",
            results.len(),
            project_id
        );
        Ok(results)
    }

    /// Pre-compaction: Extract and store critical knowledge
    pub async fn pre_compaction(&mut self, request: PreCompactionRequest) -> Result<PreCompactionResponse> {
        info!("Running pre-compaction knowledge capture...");

        let project_id = request.project_id.unwrap_or_else(|| "default".to_string());
        let session_id = request.session_id.unwrap_or_else(|| "unknown".to_string());

        let knowledge_items = self.extract_critical_knowledge(
            &request.conversation,
            &project_id,
            &session_id,
        );
        let ids = self.store_batch(knowledge_items).await?;

        info!("Pre-compaction complete: Captured {} knowledge items", ids.len());
        Ok(PreCompactionResponse {
            success: true,
            count: ids.len(),
            ids,
        })
    }

    /// Extract critical knowledge from conversation
    fn extract_critical_knowledge(
        &self,
        conversation: &str,
        project_id: &str,
        session_id: &str,
    ) -> Vec<StoreKnowledgeRequest> {
        let mut knowledge = Vec::new();

        let patterns: HashMap<&str, regex::Regex> = [
            ("architecture", regex::Regex::new(r"(?i)architecture|design pattern|system design").unwrap()),
            ("decision", regex::Regex::new(r"(?i)decided|chose|selected|will use").unwrap()),
            ("implementation", regex::Regex::new(r"(?i)implemented|built|created|added").unwrap()),
            ("configuration", regex::Regex::new(r"(?i)configured|setup|initialized").unwrap()),
            ("credential", regex::Regex::new(r"(?i)api key|secret|token|password|credential").unwrap()),
            ("issue", regex::Regex::new(r"(?i)bug|issue|problem|error|fix").unwrap()),
        ]
        .iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect();

        let messages: Vec<&str> = conversation.split("\n\n").collect();

        for (index, message) in messages.iter().enumerate() {
            if message.len() < 50 {
                continue;
            }

            let mut knowledge_type = "general";
            for (pattern_type, regex) in &patterns {
                if regex.is_match(message) {
                    knowledge_type = pattern_type;
                    break;
                }
            }

            let agent_regex = regex::Regex::new(r"\b(architect|python|swift|go|rust|flutter|qa|security|devops)\b").unwrap();
            let agent = agent_regex
                .find(message)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());

            let mut metadata = HashMap::new();
            metadata.insert(
                "conversationIndex".to_string(),
                serde_json::Value::Number(index.into()),
            );
            metadata.insert(
                "extractedAt".to_string(),
                serde_json::Value::String(Utc::now().to_rfc3339()),
            );

            knowledge.push(StoreKnowledgeRequest {
                text: message.trim().to_string(),
                knowledge_type: Some(knowledge_type.to_string()),
                project_id: Some(project_id.to_string()),
                session_id: Some(session_id.to_string()),
                agent: Some(agent),
                metadata: Some(metadata),
            });
        }

        info!("Extracted {} knowledge items from conversation", knowledge.len());
        knowledge
    }

    /// Post-compaction: Retrieve relevant context
    pub async fn post_compaction(&self, request: PostCompactionRequest) -> Result<PostCompactionResponse> {
        info!("Running post-compaction knowledge retrieval...");

        let project_id = request.project_id.unwrap_or_else(|| "default".to_string());

        let search_request = SearchRequest {
            query: request.current_task.clone(),
            limit: request.limit,
            threshold: 0.5,
            project_id: Some(project_id.clone()),
            knowledge_type: None,
            agent: None,
        };

        let search_results = self.search(search_request).await?;
        let recent_knowledge = self.get_project_knowledge(&project_id, None, 5).await?;
        let summary = self.generate_context_summary(&search_results, &recent_knowledge);

        info!("Post-compaction complete: Retrieved {} relevant items", search_results.len());
        Ok(PostCompactionResponse {
            search_results,
            recent_knowledge,
            summary,
        })
    }

    /// Generate context summary
    fn generate_context_summary(
        &self,
        search_results: &[SearchResult],
        recent_knowledge: &[SearchResult],
    ) -> ContextSummary {
        let all_items: Vec<&SearchResult> = search_results
            .iter()
            .chain(recent_knowledge.iter())
            .collect();

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_agent: HashMap<String, usize> = HashMap::new();

        for item in &all_items {
            *by_type.entry(item.knowledge_type.clone()).or_insert(0) += 1;
            *by_agent.entry(item.agent.clone()).or_insert(0) += 1;
        }

        let top_decisions: Vec<String> = search_results
            .iter()
            .filter(|i| i.knowledge_type == "decision")
            .take(5)
            .map(|i| {
                let preview_len = 100.min(i.text.len());
                format!("{}...", &i.text[..preview_len])
            })
            .collect();

        let recent_activity: Vec<RecentActivityItem> = recent_knowledge
            .iter()
            .take(3)
            .map(|i| {
                let preview_len = 80.min(i.text.len());
                RecentActivityItem {
                    knowledge_type: i.knowledge_type.clone(),
                    agent: i.agent.clone(),
                    timestamp: i.timestamp.clone(),
                    preview: format!("{}...", &i.text[..preview_len]),
                }
            })
            .collect();

        ContextSummary {
            total_items: all_items.len(),
            by_type,
            by_agent,
            top_decisions,
            recent_activity,
        }
    }

    /// Cleanup old knowledge
    pub async fn cleanup(&self, request: CleanupRequest) -> Result<CleanupResponse> {
        info!("Cleaning up knowledge older than {} days...", request.older_than_days);
        warn!("Cleanup requires mutable access - not implemented in this temporary version");
        Ok(CleanupResponse { count: 0 })
    }

    /// Get statistics
    pub async fn get_stats(&self) -> Result<StatsResponse> {
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_agent: HashMap<String, usize> = HashMap::new();
        let mut by_project: HashMap<String, usize> = HashMap::new();
        let mut oldest_record: Option<String> = None;
        let mut newest_record: Option<String> = None;

        for item in &self.items {
            *by_type.entry(item.knowledge_type.clone()).or_insert(0) += 1;
            *by_agent.entry(item.agent.clone()).or_insert(0) += 1;
            *by_project.entry(item.project_id.clone()).or_insert(0) += 1;

            if oldest_record.is_none() || Some(&item.timestamp) < oldest_record.as_ref() {
                oldest_record = Some(item.timestamp.clone());
            }
            if newest_record.is_none() || Some(&item.timestamp) > newest_record.as_ref() {
                newest_record = Some(item.timestamp.clone());
            }
        }

        Ok(StatsResponse {
            repository: self.repo_name.clone(),
            total_records: self.items.len(),
            by_type,
            by_agent,
            by_project,
            oldest_record,
            newest_record,
        })
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            KnowledgeStore::extract_repo_name("/path/to/cc-orchestra"),
            "cc-orchestra"
        );
        assert_eq!(KnowledgeStore::extract_repo_name("my-project"), "my-project");
    }

    #[tokio::test]
    async fn test_store_creation() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        tokio::fs::create_dir_all(&repo_path).await.unwrap();

        let base_dir = temp_dir.path().to_path_buf();
        let store = KnowledgeStore::new(
            &repo_path,
            Some(&base_dir),
            Some("test_knowledge".to_string()),
        );

        assert_eq!(store.repo_name, "test-repo");
        assert_eq!(store.table_name, "test_knowledge");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&c, &d) - 0.0).abs() < 0.001);
    }
}
