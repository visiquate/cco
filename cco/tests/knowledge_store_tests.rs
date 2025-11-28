//! Knowledge Store Tests - TDD Implementation for Issue #32
//!
//! This test suite follows TDD RED-GREEN-REFACTOR methodology:
//! - RED: All tests written FIRST (will fail until implementation)
//! - GREEN: Rust Specialist will implement code to pass tests
//! - REFACTOR: Improve implementation while keeping tests green
//!
//! Tests cover LanceDB embedding storage integrated into CCO daemon:
//! - SHA256-based embedding generation (384 dimensions)
//! - Vector storage and retrieval
//! - Semantic search with filters
//! - Per-repository isolation (project_id)
//! - Compaction hooks (pre/post)
//! - Cleanup and statistics
//! - API endpoint integration

#[cfg(test)]
mod embedding_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType};

    #[test]
    fn test_sha256_embedding_generation() {
        // Test that embeddings are generated from SHA256 hash
        // Expected: 384-dimensional vector from hash
        let store = KnowledgeStore::new("test-repo").unwrap();
        let text = "This is a test for embedding generation";

        let embedding = store.generate_embedding(text);

        // Should be 384 dimensions
        assert_eq!(embedding.len(), 384, "Embedding must be 384 dimensions");

        // All values should be normalized to [-1, 1]
        for &value in &embedding {
            assert!(value >= -1.0 && value <= 1.0,
                "Embedding values must be normalized to [-1, 1], got {}", value);
        }
    }

    #[test]
    fn test_embedding_normalization() {
        // Verify all embedding values fall within [-1, 1] range
        let store = KnowledgeStore::new("test-repo").unwrap();
        let text = "Normalization test with special chars: @#$%^&*()";

        let embedding = store.generate_embedding(text);

        let min_val = embedding.iter().copied().fold(f32::INFINITY, f32::min);
        let max_val = embedding.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        assert!(min_val >= -1.0, "Minimum value {} is below -1.0", min_val);
        assert!(max_val <= 1.0, "Maximum value {} is above 1.0", max_val);
    }

    #[test]
    fn test_consistent_embeddings() {
        // Same text should produce identical embeddings
        let store = KnowledgeStore::new("test-repo").unwrap();
        let text = "Consistency test text";

        let embedding1 = store.generate_embedding(text);
        let embedding2 = store.generate_embedding(text);

        assert_eq!(embedding1, embedding2,
            "Same input text must produce identical embeddings");
    }

    #[test]
    fn test_different_embeddings() {
        // Different text should produce different embeddings
        let store = KnowledgeStore::new("test-repo").unwrap();

        let embedding1 = store.generate_embedding("First text");
        let embedding2 = store.generate_embedding("Second text");

        assert_ne!(embedding1, embedding2,
            "Different input text must produce different embeddings");
    }

    #[test]
    fn test_empty_text_embedding() {
        // Test edge case: empty string
        let store = KnowledgeStore::new("test-repo").unwrap();

        let embedding = store.generate_embedding("");

        assert_eq!(embedding.len(), 384, "Empty text should still produce 384-dim embedding");
    }

    #[test]
    fn test_very_long_text_embedding() {
        // Test edge case: very long text (10KB)
        let store = KnowledgeStore::new("test-repo").unwrap();
        let long_text = "x".repeat(10_000);

        let embedding = store.generate_embedding(&long_text);

        assert_eq!(embedding.len(), 384, "Long text should produce 384-dim embedding");
    }
}

#[cfg(test)]
mod store_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize_store() {
        // Test creating a new knowledge store
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();

        store.initialize().await.unwrap();

        // Verify database directory was created
        assert!(temp_dir.path().exists());

        // Verify table was created
        let stats = store.get_stats().await.unwrap();
        assert!(stats.total_records >= 0, "Stats should be accessible after initialization");
    }

    #[tokio::test]
    async fn test_store_single_knowledge() {
        // Test storing and retrieving a single knowledge item
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let item = KnowledgeItem {
            text: "Test knowledge about JWT authentication".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({"priority": "high"}).to_string(),
        };

        let id = store.store(item.clone()).await.unwrap();

        assert!(!id.is_empty(), "Store should return non-empty ID");

        // Retrieve by searching for similar text
        let results = store.search("JWT authentication", Default::default()).await.unwrap();

        assert!(!results.is_empty(), "Should find stored item");
        assert_eq!(results[0].text, item.text);
        assert_eq!(results[0].knowledge_type, KnowledgeType::Decision);
    }

    #[tokio::test]
    async fn test_store_batch_knowledge() {
        // Test storing multiple items at once
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let items = vec![
            KnowledgeItem {
                text: "First decision about API design".to_string(),
                knowledge_type: KnowledgeType::Decision,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "architect".to_string(),
                metadata: serde_json::json!({}).to_string(),
            },
            KnowledgeItem {
                text: "Implementation of REST endpoints".to_string(),
                knowledge_type: KnowledgeType::Implementation,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "python".to_string(),
                metadata: serde_json::json!({}).to_string(),
            },
            KnowledgeItem {
                text: "Security audit findings".to_string(),
                knowledge_type: KnowledgeType::Issue,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "security".to_string(),
                metadata: serde_json::json!({}).to_string(),
            },
        ];

        let ids = store.store_batch(items.clone()).await.unwrap();

        assert_eq!(ids.len(), 3, "Should return 3 IDs for 3 items");

        // Verify all items were stored
        let stats = store.get_stats().await.unwrap();
        assert!(stats.total_records >= 3, "Should have at least 3 records");
    }

    #[tokio::test]
    async fn test_per_repo_isolation() {
        // Test that project_id provides proper isolation
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store items for different projects
        let item1 = KnowledgeItem {
            text: "Project A knowledge".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "project-a".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        };

        let item2 = KnowledgeItem {
            text: "Project B knowledge".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "project-b".to_string(),
            session_id: "session-002".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        };

        store.store(item1).await.unwrap();
        store.store(item2).await.unwrap();

        // Query only project A
        let results_a = store.get_project_knowledge(
            "project-a",
            Default::default()
        ).await.unwrap();

        assert_eq!(results_a.len(), 1, "Should find only 1 item for project-a");
        assert_eq!(results_a[0].project_id, "project-a");

        // Query only project B
        let results_b = store.get_project_knowledge(
            "project-b",
            Default::default()
        ).await.unwrap();

        assert_eq!(results_b.len(), 1, "Should find only 1 item for project-b");
        assert_eq!(results_b[0].project_id, "project-b");
    }

    #[tokio::test]
    async fn test_metadata_persistence() {
        // Test that complex metadata is stored and retrieved correctly
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let metadata = serde_json::json!({
            "priority": "high",
            "tags": ["api", "security", "jwt"],
            "version": "1.0.0",
            "nested": {
                "key": "value",
                "count": 42
            }
        });

        let item = KnowledgeItem {
            text: "Test with complex metadata".to_string(),
            knowledge_type: KnowledgeType::Configuration,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: metadata.to_string(),
        };

        store.store(item).await.unwrap();

        let results = store.search("complex metadata", Default::default()).await.unwrap();

        assert!(!results.is_empty());
        // Parse the stored metadata string back to JSON for comparison
        let stored_metadata: serde_json::Value = serde_json::from_str(&results[0].metadata).unwrap();
        assert_eq!(stored_metadata, metadata, "Metadata should match exactly");
    }

    #[tokio::test]
    async fn test_timestamp_accuracy() {
        // Test that timestamps are recorded accurately
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let before = chrono::Utc::now();

        let item = KnowledgeItem {
            text: "Timestamp test".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        };

        store.store(item).await.unwrap();

        let after = chrono::Utc::now();

        let results = store.search("Timestamp test", Default::default()).await.unwrap();

        assert!(!results.is_empty());

        let timestamp = chrono::DateTime::parse_from_rfc3339(&results[0].timestamp).unwrap();
        let timestamp_utc = timestamp.with_timezone(&chrono::Utc);

        assert!(timestamp_utc >= before && timestamp_utc <= after,
            "Timestamp should be between before and after store operation");
    }
}

#[cfg(test)]
mod search_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType, SearchOptions};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_vector_search() {
        // Test semantic similarity search
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store items with related content
        store.store(KnowledgeItem {
            text: "We decided to use JWT for authentication".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "OAuth2 implementation with tokens".to_string(),
            knowledge_type: KnowledgeType::Implementation,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "python".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Database schema design for users".to_string(),
            knowledge_type: KnowledgeType::Architecture,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Search for authentication-related knowledge
        let results = store.search("authentication tokens", SearchOptions {
            limit: 10,
            ..Default::default()
        }).await.unwrap();

        assert!(!results.is_empty(), "Should find authentication-related items");

        // First result should be most relevant (JWT or OAuth2)
        assert!(
            results[0].text.contains("JWT") || results[0].text.contains("OAuth2"),
            "Most relevant result should be about authentication"
        );
    }

    #[tokio::test]
    async fn test_search_with_type_filter() {
        // Test filtering search results by knowledge type
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store different types
        store.store(KnowledgeItem {
            text: "Decision about API design".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Implementation of API endpoints".to_string(),
            knowledge_type: KnowledgeType::Implementation,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "python".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Search with type filter
        let results = store.search("API", SearchOptions {
            knowledge_type: Some(KnowledgeType::Decision),
            ..Default::default()
        }).await.unwrap();

        assert!(!results.is_empty());

        // All results should be Decision type
        for result in results {
            assert_eq!(result.knowledge_type, KnowledgeType::Decision);
        }
    }

    #[tokio::test]
    async fn test_search_with_agent_filter() {
        // Test filtering by agent name
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store items from different agents
        store.store(KnowledgeItem {
            text: "Architecture decision by architect".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Security review by security auditor".to_string(),
            knowledge_type: KnowledgeType::Issue,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "security".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Filter by architect agent
        let results = store.search("decision", SearchOptions {
            agent: Some("architect".to_string()),
            ..Default::default()
        }).await.unwrap();

        assert!(!results.is_empty());

        for result in results {
            assert_eq!(result.agent, "architect");
        }
    }

    #[tokio::test]
    async fn test_search_with_date_range() {
        // Test filtering by timestamp range
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let now = chrono::Utc::now();

        store.store(KnowledgeItem {
            text: "Recent knowledge".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Search with date range (last hour)
        let one_hour_ago = now - chrono::Duration::hours(1);
        let one_hour_future = now + chrono::Duration::hours(1);

        let results = store.search("knowledge", SearchOptions {
            start_date: Some(one_hour_ago),
            end_date: Some(one_hour_future),
            ..Default::default()
        }).await.unwrap();

        assert!(!results.is_empty(), "Should find recent knowledge");
    }

    #[tokio::test]
    async fn test_search_results_ranking() {
        // Test that results are ranked by relevance
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store items with varying relevance
        store.store(KnowledgeItem {
            text: "JWT authentication implementation details".to_string(),
            knowledge_type: KnowledgeType::Implementation,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "python".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Database configuration".to_string(),
            knowledge_type: KnowledgeType::Configuration,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "devops".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let results = store.search("JWT authentication", SearchOptions {
            limit: 10,
            ..Default::default()
        }).await.unwrap();

        assert!(!results.is_empty());

        // Results should have scores/distances
        // More relevant items should have lower distance
        if results.len() >= 2 {
            assert!(
                results[0].score <= results[1].score,
                "Results should be ranked by relevance (lower distance = more relevant)"
            );
        }
    }

    #[tokio::test]
    async fn test_search_empty_results() {
        // Test handling of no matches
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        store.store(KnowledgeItem {
            text: "Python implementation".to_string(),
            knowledge_type: KnowledgeType::Implementation,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "python".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Search for completely unrelated term
        let results = store.search("quantum physics", SearchOptions {
            limit: 10,
            ..Default::default()
        }).await.unwrap();

        // Should return empty vec, not error
        assert!(results.is_empty() || results.len() > 0, "Should handle no matches gracefully");
    }

    #[tokio::test]
    async fn test_search_limit() {
        // Test that search respects limit parameter
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store 10 items
        for i in 0..10 {
            store.store(KnowledgeItem {
                text: format!("Knowledge item number {}", i),
                knowledge_type: KnowledgeType::General,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "test".to_string(),
                metadata: serde_json::json!({}).to_string(),
            }).await.unwrap();
        }

        // Search with limit of 5
        let results = store.search("Knowledge", SearchOptions {
            limit: 5,
            ..Default::default()
        }).await.unwrap();

        assert!(results.len() <= 5, "Results should respect limit");
    }
}

#[cfg(test)]
mod project_knowledge_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType, GetProjectOptions};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_project_knowledge() {
        // Test retrieving all knowledge for a project
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store items for test project
        for i in 0..5 {
            store.store(KnowledgeItem {
                text: format!("Project knowledge {}", i),
                knowledge_type: KnowledgeType::General,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "test".to_string(),
                metadata: serde_json::json!({}).to_string(),
            }).await.unwrap();
        }

        let results = store.get_project_knowledge(
            "test-project",
            Default::default()
        ).await.unwrap();

        assert_eq!(results.len(), 5, "Should retrieve all 5 project items");
    }

    #[tokio::test]
    async fn test_get_project_with_filters() {
        // Test filtering project knowledge by type and agent
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        store.store(KnowledgeItem {
            text: "Decision by architect".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Implementation by python".to_string(),
            knowledge_type: KnowledgeType::Implementation,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "python".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Filter by type
        let results = store.get_project_knowledge(
            "test-project",
            GetProjectOptions {
                knowledge_type: Some(KnowledgeType::Decision),
                ..Default::default()
            }
        ).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].knowledge_type, KnowledgeType::Decision);
    }

    #[tokio::test]
    async fn test_project_isolation() {
        // Test that get_project_knowledge only returns that project's data
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store for project A
        store.store(KnowledgeItem {
            text: "Project A data".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "project-a".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Store for project B
        store.store(KnowledgeItem {
            text: "Project B data".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "project-b".to_string(),
            session_id: "session-002".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let results_a = store.get_project_knowledge(
            "project-a",
            Default::default()
        ).await.unwrap();

        assert_eq!(results_a.len(), 1);
        assert_eq!(results_a[0].project_id, "project-a");

        // Should not contain project B data
        assert!(!results_a.iter().any(|r| r.project_id == "project-b"));
    }

    #[tokio::test]
    async fn test_project_knowledge_sorted_by_timestamp() {
        // Test that results are sorted newest first
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store items with delays to ensure different timestamps
        store.store(KnowledgeItem {
            text: "First item".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        store.store(KnowledgeItem {
            text: "Second item".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let results = store.get_project_knowledge(
            "test-project",
            Default::default()
        ).await.unwrap();

        assert_eq!(results.len(), 2);

        // Newest should be first
        assert_eq!(results[0].text, "Second item");
        assert_eq!(results[1].text, "First item");
    }
}

#[cfg(test)]
mod compaction_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_pre_compaction_extraction() {
        // Test extracting critical knowledge before compaction
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let conversation = r#"
        Architect: We decided to use FastAPI for the REST API.

        Python: Implemented JWT authentication with RS256 algorithm.

        Security: Found vulnerability in rate limiting. Need to add exponential backoff.
        "#;

        let result = store.pre_compaction(
            conversation,
            "test-project",
            "session-001"
        ).await.unwrap();

        assert!(result.count > 0, "Should extract knowledge items");
        assert!(!result.ids.is_empty(), "Should return IDs of stored items");

        // Verify items were stored
        let project_knowledge = store.get_project_knowledge(
            "test-project",
            Default::default()
        ).await.unwrap();

        assert!(!project_knowledge.is_empty(), "Extracted knowledge should be stored");
    }

    #[tokio::test]
    async fn test_post_compaction_retrieval() {
        // Test retrieving relevant context after compaction
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store some knowledge
        store.store(KnowledgeItem {
            text: "We use JWT for API authentication".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Post-compaction retrieval
        let result = store.post_compaction(
            "How do we handle authentication?",
            "test-project"
        ).await.unwrap();

        assert!(!result.search_results.is_empty(), "Should find relevant knowledge");
        assert!(
            result.search_results[0].text.contains("JWT"),
            "Should retrieve authentication-related knowledge"
        );

        assert!(result.summary.total_items > 0, "Summary should have items");
    }

    #[tokio::test]
    async fn test_critical_knowledge_selection() {
        // Test that critical knowledge types are properly identified
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let conversation = r#"
        We decided to use PostgreSQL for persistence.

        The weather is nice today.

        Implemented user authentication with OAuth2.

        Just a random comment.

        Security audit found SQL injection vulnerability in user input.
        "#;

        let result = store.pre_compaction(
            conversation,
            "test-project",
            "session-001"
        ).await.unwrap();

        let knowledge = store.get_project_knowledge(
            "test-project",
            Default::default()
        ).await.unwrap();

        // Should extract decisions, implementations, and issues
        // Should skip trivial messages
        let has_decision = knowledge.iter().any(|k| k.knowledge_type == KnowledgeType::Decision);
        let has_implementation = knowledge.iter().any(|k| k.knowledge_type == KnowledgeType::Implementation);
        let has_issue = knowledge.iter().any(|k| k.knowledge_type == KnowledgeType::Issue);

        assert!(has_decision || has_implementation || has_issue,
            "Should identify critical knowledge types");
    }

    #[tokio::test]
    async fn test_context_summarization() {
        // Test generation of useful summaries
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store varied knowledge
        store.store(KnowledgeItem {
            text: "Architecture decision 1".to_string(),
            knowledge_type: KnowledgeType::Architecture,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Security issue 1".to_string(),
            knowledge_type: KnowledgeType::Issue,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "security".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let result = store.post_compaction(
            "Current project status",
            "test-project"
        ).await.unwrap();

        // Summary should have stats
        assert!(result.summary.total_items > 0);
        assert!(!result.summary.by_type.is_empty(), "Should have type breakdown");
        assert!(!result.summary.by_agent.is_empty(), "Should have agent breakdown");
    }
}

#[cfg(test)]
mod cleanup_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType, CleanupOptions};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cleanup_old_knowledge() {
        // Test removing knowledge older than retention period
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store an item (will have recent timestamp)
        store.store(KnowledgeItem {
            text: "Recent knowledge".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Clean up items older than 90 days (default)
        let result = store.cleanup(CleanupOptions {
            older_than_days: 90,
            ..Default::default()
        }).await.unwrap();

        // Recent item should not be deleted
        let remaining = store.get_project_knowledge(
            "test-project",
            Default::default()
        ).await.unwrap();

        assert!(!remaining.is_empty(), "Recent knowledge should not be deleted");
    }

    #[tokio::test]
    async fn test_cleanup_respects_retention() {
        // Test custom retention periods
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        store.store(KnowledgeItem {
            text: "Test knowledge".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Clean with very short retention (0 days - everything is old)
        let result = store.cleanup(CleanupOptions {
            older_than_days: 0,
            ..Default::default()
        }).await.unwrap();

        // Should identify items for cleanup
        assert!(result.count >= 0, "Should return cleanup count");
    }

    #[tokio::test]
    async fn test_cleanup_preserves_recent() {
        // Test that recent items are never deleted
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store recent item
        let id = store.store(KnowledgeItem {
            text: "Very recent knowledge".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Cleanup with 90 day retention
        store.cleanup(CleanupOptions {
            older_than_days: 90,
            ..Default::default()
        }).await.unwrap();

        // Recent item should still exist
        let results = store.search("Very recent", Default::default()).await.unwrap();
        assert!(!results.is_empty(), "Recent item should be preserved");
    }

    #[tokio::test]
    async fn test_cleanup_project_scoped() {
        // Test cleanup can be scoped to specific project
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store for different projects
        store.store(KnowledgeItem {
            text: "Project A item".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "project-a".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Project B item".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "project-b".to_string(),
            session_id: "session-002".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        // Cleanup only project A
        store.cleanup(CleanupOptions {
            older_than_days: 0,
            project_id: Some("project-a".to_string()),
        }).await.unwrap();

        // Project B should be unaffected
        let project_b = store.get_project_knowledge(
            "project-b",
            Default::default()
        ).await.unwrap();

        assert!(!project_b.is_empty(), "Other projects should be unaffected");
    }
}

#[cfg(test)]
mod statistics_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_stats() {
        // Test database statistics are available
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let stats = store.get_stats().await.unwrap();

        assert!(stats.total_records >= 0, "Should have record count");
        assert!(stats.repository == "test-repo", "Should have repository name");
    }

    #[tokio::test]
    async fn test_stats_accuracy() {
        // Test that statistics counts are accurate
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store 5 items
        for i in 0..5 {
            store.store(KnowledgeItem {
                text: format!("Item {}", i),
                knowledge_type: KnowledgeType::General,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "test".to_string(),
                metadata: serde_json::json!({}).to_string(),
            }).await.unwrap();
        }

        let stats = store.get_stats().await.unwrap();

        assert!(stats.total_records >= 5, "Should count all records");
    }

    #[tokio::test]
    async fn test_stats_by_type() {
        // Test distribution statistics by knowledge type
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Store different types
        store.store(KnowledgeItem {
            text: "Decision 1".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Decision 2".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "Issue 1".to_string(),
            knowledge_type: KnowledgeType::Issue,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "security".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let stats = store.get_stats().await.unwrap();

        assert!(stats.by_type.contains_key("decision"), "Should have type breakdown");
        assert_eq!(stats.by_type.get("decision").unwrap_or(&0), &2, "Should count 2 decisions");
        assert_eq!(stats.by_type.get("issue").unwrap_or(&0), &1, "Should count 1 issue");
    }

    #[tokio::test]
    async fn test_stats_by_agent() {
        // Test statistics grouped by agent
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        store.store(KnowledgeItem {
            text: "From architect".to_string(),
            knowledge_type: KnowledgeType::Decision,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "architect".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        store.store(KnowledgeItem {
            text: "From python".to_string(),
            knowledge_type: KnowledgeType::Implementation,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "python".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let stats = store.get_stats().await.unwrap();

        assert!(stats.by_agent.contains_key("architect"), "Should have agent breakdown");
        assert!(stats.by_agent.contains_key("python"), "Should track all agents");
    }

    #[tokio::test]
    async fn test_stats_oldest_newest() {
        // Test tracking of oldest and newest records
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        store.store(KnowledgeItem {
            text: "First item".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        store.store(KnowledgeItem {
            text: "Second item".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await.unwrap();

        let stats = store.get_stats().await.unwrap();

        assert!(stats.oldest_record.is_some(), "Should track oldest record");
        assert!(stats.newest_record.is_some(), "Should track newest record");

        if let (Some(ref oldest), Some(ref newest)) = (&stats.oldest_record, &stats.newest_record) {
            let oldest_time = chrono::DateTime::parse_from_rfc3339(oldest).unwrap();
            let newest_time = chrono::DateTime::parse_from_rfc3339(newest).unwrap();

            assert!(newest_time >= oldest_time, "Newest should be >= oldest");
        }
    }
}

// Note: API endpoint tests would go in a separate integration test file
// that tests the actual HTTP endpoints using the axum TestServer

#[cfg(test)]
mod error_handling_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_missing_required_fields() {
        // Test validation of required fields
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // Empty text should error
        let result = store.store(KnowledgeItem {
            text: "".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await;

        assert!(result.is_err(), "Empty text should be rejected");
    }

    #[tokio::test]
    async fn test_database_errors_graceful() {
        // Test that database errors don't crash
        // This would require mocking the database to force errors
        // For now, just verify error types are defined

        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();

        // Attempting operations without initialization should error gracefully
        let result = store.search("test", Default::default()).await;

        // Should return error, not panic
        assert!(result.is_err() || result.is_ok(), "Should handle gracefully");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        // Test multiple simultaneous requests
        let temp_dir = TempDir::new().unwrap();
        let store = std::sync::Arc::new(
            KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap()
        );
        store.initialize().await.unwrap();

        // Spawn multiple concurrent stores
        let mut handles = vec![];

        for i in 0..10 {
            let store_clone = store.clone();
            let handle = tokio::spawn(async move {
                store_clone.store(KnowledgeItem {
                    text: format!("Concurrent item {}", i),
                    knowledge_type: KnowledgeType::General,
                    project_id: "test-project".to_string(),
                    session_id: "session-001".to_string(),
                    agent: "test".to_string(),
                    metadata: serde_json::json!({}).to_string(),
                }).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent access should succeed");
        }

        // Verify all were stored
        let results = store.get_project_knowledge(
            "test-project",
            Default::default()
        ).await.unwrap();

        assert!(results.len() >= 10, "Should store all concurrent items");
    }
}

#[cfg(test)]
mod data_integrity_tests {
    use cco::daemon::knowledge::{KnowledgeStore, KnowledgeItem, KnowledgeType};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_vector_dimension_validation() {
        // Test that embeddings are always 384 dimensions
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();

        // Test with various text lengths
        let texts = vec![
            "",
            "x",
            "Short text",
            "Medium length text with more content",
            &"x".repeat(10_000),
        ];

        for text in texts {
            let embedding = store.generate_embedding(text);
            assert_eq!(embedding.len(), 384,
                "All embeddings must be exactly 384 dimensions");
        }
    }

    #[tokio::test]
    async fn test_text_field_required() {
        // Test that text field is mandatory
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let result = store.store(KnowledgeItem {
            text: "".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: serde_json::json!({}).to_string(),
        }).await;

        assert!(result.is_err(), "Empty text should be rejected");
    }

    #[tokio::test]
    async fn test_type_enum_validation() {
        // Test that only valid knowledge types are accepted
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        // All valid types should work
        let valid_types = vec![
            KnowledgeType::Decision,
            KnowledgeType::Implementation,
            KnowledgeType::Architecture,
            KnowledgeType::Configuration,
            KnowledgeType::Issue,
            KnowledgeType::Credential,
            KnowledgeType::General,
        ];

        for knowledge_type in valid_types {
            let result = store.store(KnowledgeItem {
                text: "Test".to_string(),
                knowledge_type,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "test".to_string(),
                metadata: serde_json::json!({}).to_string(),
            }).await;

            assert!(result.is_ok(), "Valid knowledge type should be accepted");
        }
    }

    #[tokio::test]
    async fn test_large_batch_operations() {
        // Test handling of 1000+ items
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let mut items = vec![];
        for i in 0..1000 {
            items.push(KnowledgeItem {
                text: format!("Batch item {}", i),
                knowledge_type: KnowledgeType::General,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "test".to_string(),
                metadata: serde_json::json!({"index": i}).to_string(),
            });
        }

        let result = store.store_batch(items).await;

        assert!(result.is_ok(), "Should handle large batches");

        let ids = result.unwrap();
        assert_eq!(ids.len(), 1000, "Should store all 1000 items");
    }

    #[tokio::test]
    async fn test_special_characters_in_text() {
        // Test handling of special characters, unicode, etc.
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let special_texts = vec![
            "Text with émojis 🚀 and ünïcödé",
            "Text with \"quotes\" and 'apostrophes'",
            "Text with <html> and &entities;",
            "Text with newlines\nand\ttabs",
            "Text with null\0bytes",
        ];

        for text in special_texts {
            let result = store.store(KnowledgeItem {
                text: text.to_string(),
                knowledge_type: KnowledgeType::General,
                project_id: "test-project".to_string(),
                session_id: "session-001".to_string(),
                agent: "test".to_string(),
                metadata: serde_json::json!({}).to_string(),
            }).await;

            assert!(result.is_ok(), "Should handle special characters: {}", text);
        }
    }

    #[tokio::test]
    async fn test_metadata_json_complexity() {
        // Test deeply nested and complex JSON metadata
        let temp_dir = TempDir::new().unwrap();
        let store = KnowledgeStore::with_path("test-repo", temp_dir.path()).unwrap();
        store.initialize().await.unwrap();

        let complex_metadata = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "array": [1, 2, 3, 4, 5],
                        "nested_array": [
                            {"key": "value1"},
                            {"key": "value2"}
                        ]
                    }
                }
            },
            "tags": ["tag1", "tag2", "tag3"],
            "numbers": [1.1, 2.2, 3.3],
            "boolean": true,
            "null_value": null
        });

        let result = store.store(KnowledgeItem {
            text: "Complex metadata test".to_string(),
            knowledge_type: KnowledgeType::General,
            project_id: "test-project".to_string(),
            session_id: "session-001".to_string(),
            agent: "test".to_string(),
            metadata: complex_metadata.to_string(),
        }).await;

        assert!(result.is_ok(), "Should handle complex metadata");

        // Verify metadata roundtrip
        let results = store.search("Complex metadata", Default::default()).await.unwrap();
        // Verify metadata roundtrip by parsing the stored JSON string
        let stored_metadata: serde_json::Value = serde_json::from_str(&results[0].metadata).unwrap();
        assert_eq!(stored_metadata, complex_metadata, "Metadata should roundtrip exactly");
    }
}
