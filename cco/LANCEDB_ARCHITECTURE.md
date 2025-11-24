# LanceDB Knowledge Manager Architecture for CCO Daemon

## Issue #32: Embed LanceDB in CCO daemon using Rust SDK

### Executive Summary

This document defines the complete architecture for integrating LanceDB vector database into the CCO daemon, replicating the functionality of the Node.js knowledge-manager.js implementation in Rust. The system will provide persistent knowledge storage and retrieval across agent compactions using vector embeddings for semantic search.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    CCO Daemon Process                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │              HTTP API Layer (Axum)                │  │
│  │                                                   │  │
│  │  POST /api/knowledge/store                        │  │
│  │  POST /api/knowledge/store-batch                  │  │
│  │  POST /api/knowledge/search                       │  │
│  │  GET  /api/knowledge/project/{project_id}         │  │
│  │  POST /api/knowledge/pre-compaction               │  │
│  │  POST /api/knowledge/post-compaction              │  │
│  │  GET  /api/knowledge/stats                        │  │
│  │  POST /api/knowledge/cleanup                      │  │
│  └──────────────────────┬───────────────────────────┘  │
│                          │                              │
│  ┌──────────────────────▼───────────────────────────┐  │
│  │         Knowledge Store (Rust Module)             │  │
│  │                                                   │  │
│  │  • KnowledgeStore struct                          │  │
│  │  • Async operations via tokio                     │  │
│  │  • SHA256 embedding generation                    │  │
│  │  • Per-repository isolation                       │  │
│  └──────────────────────┬───────────────────────────┘  │
│                          │                              │
│  ┌──────────────────────▼───────────────────────────┐  │
│  │          LanceDB Vector Database                  │  │
│  │                                                   │  │
│  │  • 384-dimensional vectors                        │  │
│  │  • Semantic similarity search                     │  │
│  │  • Storage: ~/.cco/knowledge/{repo_name}/         │  │
│  └──────────────────────────────────────────────────┘  │
│                                                          │
└─────────────────────────────────────────────────────────┘

                             ↑
                             │ HTTP Requests
                             │
┌─────────────────────────────────────────────────────────┐
│                    Agent Processes                       │
│                                                          │
│  • Chief Architect (Opus)                               │
│  • Coding Agents (Sonnet/Haiku)                         │
│  • Support Agents (QA, Security, Docs)                  │
│                                                          │
│  Access via HTTP API (no filesystem access)             │
└─────────────────────────────────────────────────────────┘
```

## Module Structure

### Directory Layout
```
cco/src/daemon/knowledge/
├── mod.rs          # Module exports and public API
├── models.rs       # Data structures and types
├── store.rs        # KnowledgeStore implementation
├── embedding.rs    # SHA256 embedding generation
└── api.rs          # HTTP API endpoints and handlers
```

### Module Responsibilities

#### 1. `mod.rs` - Module Exports
```rust
pub mod models;
pub mod store;
pub mod embedding;
pub mod api;

pub use models::{KnowledgeItem, SearchOptions, KnowledgeStats};
pub use store::KnowledgeStore;
pub use embedding::generate_embedding;
pub use api::knowledge_routes;
```

#### 2. `models.rs` - Data Structures
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub vector: Vec<f32>,           // 384-dimensional
    pub text: String,
    pub r#type: KnowledgeType,
    pub project_id: String,
    pub session_id: String,
    pub agent: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeType {
    Decision,
    Architecture,
    Implementation,
    Configuration,
    Credential,
    Issue,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub limit: Option<usize>,
    pub threshold: Option<f32>,
    pub project_id: Option<String>,
    pub r#type: Option<KnowledgeType>,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub repository: String,
    pub total_records: usize,
    pub by_type: HashMap<String, usize>,
    pub by_agent: HashMap<String, usize>,
    pub by_project: HashMap<String, usize>,
    pub oldest_record: Option<DateTime<Utc>>,
    pub newest_record: Option<DateTime<Utc>>,
}
```

#### 3. `store.rs` - KnowledgeStore Implementation
```rust
use lancedb::{Connection, Table};
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct KnowledgeStore {
    db_path: PathBuf,
    table_name: String,
    repo_name: String,
    embedding_dim: usize,
    connection: Arc<RwLock<Option<Connection>>>,
    table: Arc<RwLock<Option<Table>>>,
}

impl KnowledgeStore {
    pub async fn new(repo_path: Option<PathBuf>) -> Result<Self>
    pub async fn initialize(&mut self) -> Result<()>
    pub async fn store(&self, item: KnowledgeRequest) -> Result<String>
    pub async fn store_batch(&self, items: Vec<KnowledgeRequest>) -> Result<Vec<String>>
    pub async fn search(&self, query: &str, options: SearchOptions) -> Result<Vec<KnowledgeItem>>
    pub async fn get_project_knowledge(&self, project_id: &str, options: GetOptions) -> Result<Vec<KnowledgeItem>>
    pub async fn pre_compaction(&self, conversation: &str, context: CompactionContext) -> Result<CompactionResult>
    pub async fn post_compaction(&self, current_task: &str, context: CompactionContext) -> Result<PostCompactionResult>
    pub async fn cleanup(&self, options: CleanupOptions) -> Result<CleanupResult>
    pub async fn get_stats(&self) -> Result<KnowledgeStats>

    // Internal methods
    fn extract_critical_knowledge(&self, conversation: &str, context: &CompactionContext) -> Vec<KnowledgeRequest>
    fn generate_context_summary(&self, search_results: &[KnowledgeItem], recent: &[KnowledgeItem]) -> ContextSummary
}
```

#### 4. `embedding.rs` - SHA256 Embedding Generation
```rust
use sha2::{Sha256, Digest};

const EMBEDDING_DIM: usize = 384;

/// Generate a 384-dimensional embedding from text using SHA256
/// Matches the Node.js implementation for compatibility
pub fn generate_embedding(text: &str) -> Vec<f32> {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();

    let mut embedding = Vec::with_capacity(EMBEDDING_DIM);

    for i in 0..EMBEDDING_DIM {
        // Normalize to [-1.0, 1.0] range
        let byte_val = hash[i % hash.len()] as f32;
        embedding.push((byte_val / 128.0) - 1.0);
    }

    embedding
}
```

#### 5. `api.rs` - HTTP API Endpoints
```rust
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

pub fn knowledge_routes() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/api/knowledge/store", post(store_handler))
        .route("/api/knowledge/store-batch", post(store_batch_handler))
        .route("/api/knowledge/search", post(search_handler))
        .route("/api/knowledge/project/:project_id", get(get_project_handler))
        .route("/api/knowledge/pre-compaction", post(pre_compaction_handler))
        .route("/api/knowledge/post-compaction", post(post_compaction_handler))
        .route("/api/knowledge/stats", get(stats_handler))
        .route("/api/knowledge/cleanup", post(cleanup_handler))
}

async fn store_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, ApiError>

async fn search_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError>

// ... other handlers
```

## API Contract

### 1. Store Knowledge
**POST** `/api/knowledge/store`
```json
Request:
{
  "text": "We decided to use FastAPI for the REST API",
  "type": "decision",
  "project_id": "cc-orchestra",
  "session_id": "session-123",
  "agent": "architect",
  "metadata": {}
}

Response:
{
  "id": "decision-1234567890-abc123",
  "success": true
}
```

### 2. Batch Store
**POST** `/api/knowledge/store-batch`
```json
Request:
{
  "items": [
    {
      "text": "Item 1 text",
      "type": "architecture",
      "agent": "architect"
    },
    {
      "text": "Item 2 text",
      "type": "implementation",
      "agent": "python-specialist"
    }
  ]
}

Response:
{
  "ids": ["id1", "id2"],
  "stored": 2,
  "failed": 0
}
```

### 3. Search Knowledge
**POST** `/api/knowledge/search`
```json
Request:
{
  "query": "authentication JWT FastAPI",
  "limit": 10,
  "threshold": 0.5,
  "project_id": "cc-orchestra",
  "type": "decision",
  "agent": null
}

Response:
{
  "results": [
    {
      "id": "decision-123",
      "text": "We decided to use FastAPI...",
      "type": "decision",
      "project_id": "cc-orchestra",
      "session_id": "session-123",
      "agent": "architect",
      "timestamp": "2025-01-01T00:00:00Z",
      "metadata": {},
      "score": 0.89
    }
  ]
}
```

### 4. Get Project Knowledge
**GET** `/api/knowledge/project/{project_id}?type=decision&limit=100`

### 5. Pre-Compaction Hook
**POST** `/api/knowledge/pre-compaction`
```json
Request:
{
  "conversation": "Full conversation text...",
  "context": {
    "project_id": "cc-orchestra",
    "session_id": "session-123"
  }
}

Response:
{
  "success": true,
  "count": 5,
  "ids": ["id1", "id2", "id3", "id4", "id5"]
}
```

### 6. Post-Compaction Hook
**POST** `/api/knowledge/post-compaction`
```json
Request:
{
  "current_task": "Implementing authentication",
  "context": {
    "project_id": "cc-orchestra",
    "limit": 10
  }
}

Response:
{
  "search_results": [...],
  "recent_knowledge": [...],
  "summary": {
    "total_items": 15,
    "by_type": {"decision": 5, "architecture": 3},
    "by_agent": {"architect": 8, "python-specialist": 7},
    "top_decisions": [...],
    "recent_activity": [...]
  }
}
```

### 7. Get Statistics
**GET** `/api/knowledge/stats`

### 8. Cleanup Old Knowledge
**POST** `/api/knowledge/cleanup`
```json
Request:
{
  "older_than_days": 90,
  "project_id": "cc-orchestra"
}

Response:
{
  "count": 42
}
```

## Database Schema

### LanceDB Table Schema
```rust
struct LanceDBRecord {
    id: String,                    // Unique identifier
    vector: Vec<f32>,              // 384-dimensional embedding
    text: String,                  // Knowledge content
    type: String,                  // Knowledge type enum
    project_id: String,            // Repository identifier
    session_id: String,            // Agent session
    agent: String,                 // Agent name
    timestamp: String,             // ISO 8601 timestamp
    metadata: String,              // JSON string
}
```

### Indexes
- Primary index on `id`
- Vector index on `vector` for similarity search
- Secondary indexes on `project_id`, `type`, `agent` for filtering

## Storage Strategy

### File Organization
```
~/.cco/knowledge/
├── cc-orchestra/
│   ├── orchestra_knowledge.lance/
│   │   ├── _versions/
│   │   ├── data/
│   │   └── metadata/
│   └── .lock
├── another-repo/
│   └── orchestra_knowledge.lance/
└── default/
    └── orchestra_knowledge.lance/
```

### Database Lifecycle
1. **Initialization**: Create directory structure and table on first use
2. **Connection Pool**: Single connection per repository, shared across requests
3. **Transactions**: Use read-write locks for concurrent access
4. **Cleanup**: Periodic cleanup of old records (configurable)

## Integration Points

### 1. Server Integration (`cco/src/server.rs`)
```rust
// In run_server function
let knowledge_store = Arc::new(KnowledgeStore::new(None).await?);
knowledge_store.initialize().await?;

// Add to ServerState
pub struct ServerState {
    // ... existing fields
    pub knowledge_store: Arc<KnowledgeStore>,
}

// Mount routes
let app = Router::new()
    // ... existing routes
    .merge(knowledge_routes())
    .with_state(state);
```

### 2. Daemon Lifecycle Integration
```rust
// In daemon startup
let knowledge_store = KnowledgeStore::new(Some(repo_path)).await?;
knowledge_store.initialize().await?;

// In daemon shutdown
// LanceDB handles cleanup automatically
```

### 3. Agent Access Pattern
```javascript
// Agents access via HTTP API
const storeResult = await fetch('http://localhost:9898/api/knowledge/store', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    text: 'Architecture decision...',
    type: 'decision',
    agent: 'architect'
  })
});

const searchResults = await fetch('http://localhost:9898/api/knowledge/search', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    query: 'authentication',
    limit: 10
  })
});
```

## Error Handling Strategy

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Table operation error: {0}")]
    TableError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("LanceDB error: {0}")]
    LanceDbError(#[from] lancedb::Error),
}
```

### Graceful Degradation
1. **Connection Failures**: Return empty results, log warnings
2. **Search Failures**: Fall back to recent knowledge
3. **Storage Failures**: Queue for retry, continue operation
4. **Compaction Failures**: Log but don't block conversation

## Performance Considerations

### Optimization Strategies
1. **Connection Pooling**: Single connection per repository
2. **Batch Operations**: Support batch storage for efficiency
3. **Async I/O**: All operations are async via tokio
4. **Caching**: Consider in-memory cache for frequent searches
5. **Index Optimization**: Create appropriate indexes for common queries

### Expected Performance
- **Storage**: < 10ms per item
- **Search**: < 50ms for 10 results
- **Batch Storage**: < 100ms for 100 items
- **Stats Query**: < 100ms

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_generation() {
        let embedding = generate_embedding("test text");
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().all(|&v| v >= -1.0 && v <= 1.0));
    }

    #[tokio::test]
    async fn test_store_and_search() {
        let store = KnowledgeStore::new(None).await.unwrap();
        store.initialize().await.unwrap();

        let id = store.store(/* ... */).await.unwrap();
        assert!(!id.is_empty());

        let results = store.search("test", SearchOptions::default()).await.unwrap();
        assert!(!results.is_empty());
    }
}
```

### Integration Tests
```rust
// tests/knowledge_integration_tests.rs
#[tokio::test]
async fn test_api_endpoints() {
    // Test all HTTP endpoints
}

#[tokio::test]
async fn test_compaction_hooks() {
    // Test pre/post compaction workflow
}
```

## Migration from Node.js

### Compatibility Requirements
1. **Embedding Algorithm**: Must match SHA256-based 384-dim vectors
2. **ID Format**: `{type}-{timestamp}-{random}`
3. **Timestamp Format**: ISO 8601 strings
4. **Metadata Format**: JSON strings
5. **API Response Format**: Match existing JSON structure

### Migration Path
1. Implement Rust modules with feature parity
2. Test against existing Node.js test cases
3. Verify embedding compatibility
4. Deploy alongside Node.js for validation
5. Switch agents to use HTTP API
6. Deprecate Node.js implementation

## Security Considerations

1. **Input Validation**: Validate all API inputs
2. **Rate Limiting**: Use existing ConnectionTracker
3. **Authentication**: Consider API keys for agent access
4. **Data Sanitization**: Sanitize text before storage
5. **Access Control**: Per-project isolation via project_id

## Dependencies

### Cargo.toml Addition
```toml
[dependencies]
# ... existing dependencies
lancedb = "0.22.3"
# Note: tokio, serde, serde_json, chrono, sha2 already present
```

## Implementation Phases

### Phase 1: Core Implementation (Week 1)
- [ ] Create module structure
- [ ] Implement models.rs
- [ ] Implement embedding.rs
- [ ] Basic KnowledgeStore with store/search

### Phase 2: API Layer (Week 1)
- [ ] Implement HTTP endpoints
- [ ] Integrate with server.rs
- [ ] Add error handling

### Phase 3: Advanced Features (Week 2)
- [ ] Implement compaction hooks
- [ ] Add cleanup functionality
- [ ] Implement statistics

### Phase 4: Testing & Validation (Week 2)
- [ ] Unit tests
- [ ] Integration tests
- [ ] Performance testing
- [ ] Node.js compatibility validation

## Success Criteria

1. **Functional Parity**: All 15 methods from Node.js implemented
2. **Performance**: Meet or exceed Node.js performance
3. **Compatibility**: Embeddings match Node.js implementation
4. **Reliability**: 99.9% uptime with graceful degradation
5. **Scalability**: Support 1000+ agents concurrent access

## Monitoring & Observability

1. **Metrics to Track**:
   - Storage operations/second
   - Search latency P50/P95/P99
   - Database size growth
   - Error rates by operation

2. **Logging**:
   - Info: Successful operations
   - Warn: Degraded operations
   - Error: Failed operations with context

3. **Health Checks**:
   - Add to /health endpoint
   - Check database connectivity
   - Report storage statistics

## Future Enhancements

1. **Real Embeddings**: Replace SHA256 with actual embedding model
2. **Clustering**: Support distributed deployment
3. **Backup/Restore**: Automated backup strategy
4. **Query Optimization**: Advanced search capabilities
5. **Compression**: Reduce storage footprint

## Conclusion

This architecture provides a robust, scalable implementation of the Knowledge Manager in Rust, maintaining full compatibility with the existing Node.js implementation while leveraging Rust's performance and safety benefits. The HTTP API ensures agents can access knowledge without filesystem dependencies, and the per-repository isolation maintains data separation across projects.