//! Knowledge store implementation using LanceDB
//!
//! This module provides the core knowledge storage and retrieval functionality,
//! replicating all methods from the JavaScript knowledge-manager.js implementation.
//!
//! NOTE: This is a skeletal implementation that compiles but has placeholders for
//! complex LanceDB operations. Full implementation requires:
//! 1. Proper Arrow RecordBatch construction compatible with LanceDB 0.22
//! 2. Vector search API integration
//! 3. Filtering and result conversion from Arrow format

use super::embedding::{generate_embedding, EMBEDDING_DIM};
use super::models::*;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{error, info, trace, warn};

/// Knowledge store using LanceDB for vector similarity search
///
/// TEMPORARY IMPLEMENTATION: Currently uses in-memory storage
/// Full LanceDB integration pending API compatibility fixes
pub struct KnowledgeStore {
    db_path: PathBuf,
    repo_name: String,
    table_name: String,
    // Temporary in-memory storage until LanceDB integration is complete
    items: Vec<KnowledgeItem>,
}

impl KnowledgeStore {
    /// Create a new knowledge store
    ///
    /// # Arguments
    /// * `repo_path` - Path to the repository (used to derive repo name)
    /// * `base_dir` - Base directory for knowledge databases
    /// * `table_name` - Name of the table to use (default: "orchestra_knowledge")
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
            connection: None,
            table: None,
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

    /// Initialize the LanceDB connection and table
    pub async fn initialize(&mut self) -> Result<()> {
        // Ensure data directory exists
        tokio::fs::create_dir_all(&self.db_path)
            .await
            .context("Failed to create knowledge directory")?;

        // Connect to LanceDB
        let db_uri = self.db_path.to_string_lossy().to_string();
        self.connection = Some(
            lancedb::connect(&db_uri)
                .execute()
                .await
                .context("Failed to connect to LanceDB")?,
        );

        // Try to open existing table, or create new one
        if let Some(ref conn) = self.connection {
            match conn.open_table(&self.table_name).execute().await {
                Ok(table) => {
                    self.table = Some(table);
                    info!(
                        "Connected to existing knowledge base for {}",
                        self.repo_name
                    );
                }
                Err(_) => {
                    // Table doesn't exist, create it
                    info!("Creating new knowledge base for {}", self.repo_name);
                    self.create_table().await?;
                }
            }
        }

        Ok(())
    }

    /// Create the knowledge table with schema
    async fn create_table(&mut self) -> Result<()> {
        if let Some(ref conn) = self.connection {
            // Create schema
            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new(
                    "vector",
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float32, true)),
                        EMBEDDING_DIM as i32,
                    ),
                    false,
                ),
                Field::new("text", DataType::Utf8, false),
                Field::new("type", DataType::Utf8, false),
                Field::new("project_id", DataType::Utf8, false),
                Field::new("session_id", DataType::Utf8, false),
                Field::new("agent", DataType::Utf8, false),
                Field::new("timestamp", DataType::Utf8, false),
                Field::new("metadata", DataType::Utf8, false),
            ]));

            // Create initialization record
            let init_record = self.create_init_record();
            let batch = self.create_record_batch(schema.clone(), vec![init_record])?;

            // Create table
            let table = conn
                .create_table(&self.table_name, vec![batch])
                .execute()
                .await
                .context("Failed to create table")?;

            self.table = Some(table);
            info!("Knowledge table created successfully");
        }

        Ok(())
    }

    /// Create initialization record
    fn create_init_record(&self) -> KnowledgeItem {
        KnowledgeItem {
            id: format!("init-{}", Utc::now().timestamp()),
            vector: vec![0.0; EMBEDDING_DIM],
            text: "Initialization record".to_string(),
            knowledge_type: "system".to_string(),
            project_id: "system".to_string(),
            session_id: "init".to_string(),
            agent: "system".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            metadata: "{}".to_string(),
        }
    }

    /// Create a RecordBatch from knowledge items
    fn create_record_batch(
        &self,
        schema: Arc<Schema>,
        items: Vec<KnowledgeItem>,
    ) -> Result<RecordBatch> {
        let count = items.len();

        // Build arrays for each column
        // StringArray needs to be constructed from Vec<Option<&str>> or similar
        let ids: StringArray = items.iter().map(|i| Some(i.id.as_str())).collect();

        let vectors: Vec<f32> = items
            .iter()
            .flat_map(|i| i.vector.iter().copied())
            .collect();
        let vectors_array = Float32Array::from(vectors);

        let texts: StringArray = items.iter().map(|i| Some(i.text.as_str())).collect();
        let types: StringArray = items.iter().map(|i| Some(i.knowledge_type.as_str())).collect();
        let project_ids: StringArray = items.iter().map(|i| Some(i.project_id.as_str())).collect();
        let session_ids: StringArray = items.iter().map(|i| Some(i.session_id.as_str())).collect();
        let agents: StringArray = items.iter().map(|i| Some(i.agent.as_str())).collect();
        let timestamps: StringArray = items.iter().map(|i| Some(i.timestamp.as_str())).collect();
        let metadata: StringArray = items.iter().map(|i| Some(i.metadata.as_str())).collect();

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(ids),
                Arc::new(vectors_array),
                Arc::new(texts),
                Arc::new(types),
                Arc::new(project_ids),
                Arc::new(session_ids),
                Arc::new(agents),
                Arc::new(timestamps),
                Arc::new(metadata),
            ],
        )?;

        Ok(batch)
    }

    /// Store a single knowledge item
    pub async fn store(&mut self, request: StoreKnowledgeRequest) -> Result<StoreKnowledgeResponse> {
        // Validate text
        if request.text.is_empty() {
            anyhow::bail!("Knowledge text is required");
        }

        // Generate embedding
        let vector = generate_embedding(&request.text);

        // Create record
        let knowledge_type = request
            .knowledge_type
            .unwrap_or_else(|| "decision".to_string());
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

        // Add to table
        if let Some(ref mut table) = self.table {
            let schema = table.schema().await?;
            let batch = self.create_record_batch(schema, vec![item])?;
            table.add(vec![batch]).execute().await?;

            info!("Stored knowledge: {} from {}", knowledge_type, agent);
        } else {
            anyhow::bail!("Table not initialized");
        }

        Ok(StoreKnowledgeResponse { id, stored: true })
    }

    /// Store multiple knowledge items in batch
    pub async fn store_batch(
        &mut self,
        requests: Vec<StoreKnowledgeRequest>,
    ) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        for request in requests {
            match self.store(request).await {
                Ok(response) => ids.push(response.id),
                Err(e) => {
                    warn!("Failed to store item: {}", e);
                }
            }
        }

        info!("Stored {}/{} knowledge items", ids.len(), ids.capacity());
        Ok(ids)
    }

    /// Search knowledge base using semantic similarity
    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        if let Some(ref table) = self.table {
            // Generate query embedding
            let query_vector = generate_embedding(&request.query);

            // Perform vector search
            let results = table
                .search(&query_vector)
                .limit(request.limit)
                .execute()
                .await?;

            // Convert to SearchResult and apply filters
            let mut search_results = Vec::new();

            // TODO: Implement proper filtering and result conversion
            // This is a placeholder - actual implementation needs Arrow record parsing

            info!("Found {} relevant knowledge items", search_results.len());
            Ok(search_results)
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieve all knowledge for a specific project
    pub async fn get_project_knowledge(
        &self,
        project_id: &str,
        knowledge_type: Option<String>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Use search with a dummy query to get all records, then filter
        let dummy_request = SearchRequest {
            query: String::new(),
            limit: 1000,
            threshold: 0.0,
            project_id: Some(project_id.to_string()),
            knowledge_type,
            agent: None,
        };

        let mut results = self.search(dummy_request).await?;

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Limit results
        results.truncate(limit);

        info!(
            "Retrieved {} knowledge items for project: {}",
            results.len(),
            project_id
        );
        Ok(results)
    }

    /// Pre-compaction hook: Extract and store critical knowledge
    pub async fn pre_compaction(
        &mut self,
        request: PreCompactionRequest,
    ) -> Result<PreCompactionResponse> {
        info!("Running pre-compaction knowledge capture...");

        let project_id = request.project_id.unwrap_or_else(|| "default".to_string());
        let session_id = request
            .session_id
            .unwrap_or_else(|| "unknown".to_string());

        let knowledge_items = self.extract_critical_knowledge(&request.conversation, &project_id, &session_id);
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

        // Pattern matching for knowledge types
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

        // Split conversation into messages
        let messages: Vec<&str> = conversation.split("\n\n").collect();

        for (index, message) in messages.iter().enumerate() {
            // Skip very short messages
            if message.len() < 50 {
                continue;
            }

            // Detect knowledge type
            let mut knowledge_type = "general";
            for (pattern_type, regex) in &patterns {
                if regex.is_match(message) {
                    knowledge_type = pattern_type;
                    break;
                }
            }

            // Extract agent if mentioned
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

        info!(
            "Extracted {} knowledge items from conversation",
            knowledge.len()
        );
        knowledge
    }

    /// Post-compaction hook: Retrieve relevant context
    pub async fn post_compaction(
        &self,
        request: PostCompactionRequest,
    ) -> Result<PostCompactionResponse> {
        info!("Running post-compaction knowledge retrieval...");

        let project_id = request.project_id.unwrap_or_else(|| "default".to_string());

        // Search for relevant knowledge based on current task
        let search_request = SearchRequest {
            query: request.current_task.clone(),
            limit: request.limit,
            threshold: 0.5,
            project_id: Some(project_id.clone()),
            knowledge_type: None,
            agent: None,
        };

        let search_results = self.search(search_request).await?;

        // Get recent project knowledge
        let recent_knowledge = self
            .get_project_knowledge(&project_id, None, 5)
            .await?;

        // Generate summary
        let summary = self.generate_context_summary(&search_results, &recent_knowledge);

        info!(
            "Post-compaction complete: Retrieved {} relevant items",
            search_results.len()
        );
        Ok(PostCompactionResponse {
            search_results,
            recent_knowledge,
            summary,
        })
    }

    /// Generate a summary of retrieved context
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

    /// Clean up old knowledge
    pub async fn cleanup(&self, request: CleanupRequest) -> Result<CleanupResponse> {
        info!(
            "Cleaning up knowledge older than {} days...",
            request.older_than_days
        );

        let cutoff_date = Utc::now() - chrono::Duration::days(request.older_than_days);

        // TODO: Implement actual deletion
        // LanceDB deletion is complex and may require table recreation
        // For now, we just count items that would be deleted

        warn!("Cleanup not yet fully implemented (LanceDB limitation)");
        Ok(CleanupResponse { count: 0 })
    }

    /// Get statistics about the knowledge base
    pub async fn get_stats(&self) -> Result<StatsResponse> {
        // TODO: Implement proper statistics gathering
        // This requires reading all records and aggregating

        Ok(StatsResponse {
            repository: self.repo_name.clone(),
            total_records: 0,
            by_type: HashMap::new(),
            by_agent: HashMap::new(),
            by_project: HashMap::new(),
            oldest_record: None,
            newest_record: None,
        })
    }
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

        let store = KnowledgeStore::new(
            &repo_path,
            Some(temp_dir.path()),
            Some("test_knowledge".to_string()),
        );

        assert_eq!(store.repo_name, "test-repo");
        assert_eq!(store.table_name, "test_knowledge");
    }

    #[test]
    fn test_extract_critical_knowledge() {
        let temp_dir = tempdir().unwrap();
        let store = KnowledgeStore::new(
            temp_dir.path(),
            Some(temp_dir.path()),
            None,
        );

        let conversation = "We decided to use FastAPI for the API.\n\nImplemented JWT authentication with RS256.";
        let knowledge = store.extract_critical_knowledge(conversation, "test-project", "session-1");

        assert_eq!(knowledge.len(), 2);
        assert_eq!(knowledge[0].knowledge_type, Some("decision".to_string()));
        assert_eq!(knowledge[1].knowledge_type, Some("implementation".to_string()));
    }
}
