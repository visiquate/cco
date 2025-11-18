# LanceDB Module Responsibility Matrix

## File-by-File Implementation Guide

### 1. `cco/src/daemon/knowledge/mod.rs` (~50 lines)
**Purpose:** Module organization and public API exports

```rust
// Public exports for external use
pub mod models;
pub mod store;
pub mod embedding;
pub mod api;

// Re-export commonly used types
pub use models::{
    KnowledgeItem,
    KnowledgeType,
    SearchOptions,
    KnowledgeStats,
    StoreRequest,
    SearchRequest,
    CompactionContext,
};

pub use store::KnowledgeStore;
pub use embedding::generate_embedding;
pub use api::knowledge_routes;

// Module-level constants
pub const EMBEDDING_DIM: usize = 384;
pub const DEFAULT_TABLE_NAME: &str = "orchestra_knowledge";
pub const DEFAULT_SEARCH_LIMIT: usize = 10;
pub const DEFAULT_SEARCH_THRESHOLD: f32 = 0.5;
```

**Dependencies:** None (just re-exports)

**Interfaces With:**
- All other modules (provides public API)
- `cco/src/server.rs` (imported here)

---

### 2. `cco/src/daemon/knowledge/models.rs` (~250 lines)
**Purpose:** Data structures and type definitions

**Key Structures:**
```rust
// Core data model matching Node.js schema
pub struct KnowledgeItem {
    pub id: String,
    pub vector: Vec<f32>,      // 384-dimensional
    pub text: String,
    pub r#type: KnowledgeType,
    pub project_id: String,
    pub session_id: String,
    pub agent: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: JsonValue,
}

// Request/response types for API
pub struct StoreRequest { /* fields */ }
pub struct StoreResponse { /* fields */ }
pub struct SearchRequest { /* fields */ }
pub struct SearchResponse { /* fields */ }
pub struct CompactionRequest { /* fields */ }
pub struct CompactionResponse { /* fields */ }

// Enums
pub enum KnowledgeType {
    Decision,
    Architecture,
    Implementation,
    Configuration,
    Credential,
    Issue,
    General,
}
```

**Dependencies:**
- `serde` (serialization)
- `serde_json` (JSON handling)
- `chrono` (timestamps)

**Interfaces With:**
- `store.rs` (uses all models)
- `api.rs` (uses for request/response)

---

### 3. `cco/src/daemon/knowledge/embedding.rs` (~100 lines)
**Purpose:** SHA256-based embedding generation

**Core Function:**
```rust
use sha2::{Sha256, Digest};

pub fn generate_embedding(text: &str) -> Vec<f32> {
    // Must match Node.js implementation exactly
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();

    let mut embedding = Vec::with_capacity(384);
    for i in 0..384 {
        let byte_val = hash[i % hash.len()] as f32;
        embedding.push((byte_val / 128.0) - 1.0);
    }
    embedding
}
```

**Critical Requirements:**
- MUST produce identical output to Node.js for same input
- 384 dimensions exactly
- Normalize to [-1.0, 1.0] range
- Use modulo to cycle through 32-byte hash

**Dependencies:**
- `sha2` (hashing)

**Interfaces With:**
- `store.rs` (called when storing items)

---

### 4. `cco/src/daemon/knowledge/store.rs` (~800 lines)
**Purpose:** Core LanceDB operations and business logic

**Main Structure:**
```rust
pub struct KnowledgeStore {
    db_path: PathBuf,
    table_name: String,
    repo_name: String,
    embedding_dim: usize,
    connection: Arc<RwLock<Option<Connection>>>,
    table: Arc<RwLock<Option<Table>>>,
}
```

**Key Methods:**
```rust
impl KnowledgeStore {
    // Lifecycle
    pub async fn new(repo_path: Option<PathBuf>) -> Result<Self>
    pub async fn initialize(&mut self) -> Result<()>

    // Core operations
    pub async fn store(&self, request: StoreRequest) -> Result<String>
    pub async fn store_batch(&self, items: Vec<StoreRequest>) -> Result<Vec<String>>
    pub async fn search(&self, query: &str, options: SearchOptions) -> Result<Vec<KnowledgeItem>>

    // Project operations
    pub async fn get_project_knowledge(&self, project_id: &str, options: GetOptions) -> Result<Vec<KnowledgeItem>>

    // Compaction hooks
    pub async fn pre_compaction(&self, conversation: &str, context: CompactionContext) -> Result<CompactionResult>
    pub async fn post_compaction(&self, current_task: &str, context: CompactionContext) -> Result<PostCompactionResult>

    // Maintenance
    pub async fn cleanup(&self, options: CleanupOptions) -> Result<CleanupResult>
    pub async fn get_stats(&self) -> Result<KnowledgeStats>

    // Internal helpers
    fn extract_critical_knowledge(&self, conversation: &str, context: &CompactionContext) -> Vec<StoreRequest>
    fn generate_context_summary(&self, search_results: &[KnowledgeItem], recent: &[KnowledgeItem]) -> ContextSummary
    fn get_repo_name(repo_path: &Path) -> String
}
```

**Complex Logic:**
- Repository path parsing (same as Node.js)
- Pattern matching for knowledge extraction
- Summary generation for compaction

**Dependencies:**
- `lancedb` (vector database)
- `tokio` (async runtime)
- `embedding.rs` (generate vectors)
- All models from `models.rs`

**Interfaces With:**
- `api.rs` (called by HTTP handlers)
- `embedding.rs` (generates vectors)

---

### 5. `cco/src/daemon/knowledge/api.rs` (~500 lines)
**Purpose:** HTTP API endpoints using Axum

**Route Definition:**
```rust
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
```

**Handler Pattern:**
```rust
async fn store_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, ApiError> {
    // Input validation
    // Call store.rs methods
    // Format response
    // Error handling
}
```

**Error Handling:**
- Convert internal errors to HTTP status codes
- Always return JSON with success flag
- Log errors with context

**Dependencies:**
- `axum` (web framework)
- `store.rs` (business logic)
- All models from `models.rs`

**Interfaces With:**
- `cco/src/server.rs` (routes mounted here)
- `store.rs` (delegates operations)

---

## Integration Points

### 1. `cco/src/server.rs` Modifications (~50 lines added)
```rust
// In ServerState struct
pub struct ServerState {
    // ... existing fields
    pub knowledge_store: Option<Arc<KnowledgeStore>>,
}

// In run_server function
let knowledge_store = match KnowledgeStore::new(None).await {
    Ok(mut store) => {
        store.initialize().await?;
        Some(Arc::new(store))
    }
    Err(e) => {
        warn!("Knowledge store initialization failed: {}", e);
        None
    }
};

// Add to state
let state = Arc::new(ServerState {
    // ... existing fields
    knowledge_store,
});

// Mount routes
let app = Router::new()
    // ... existing routes
    .merge(knowledge_routes())
    .with_state(state);
```

### 2. `cco/Cargo.toml` Modifications (1 line added)
```toml
[dependencies]
# ... existing dependencies
lancedb = "0.22.3"
```

---

## Testing Requirements

### Unit Tests per Module

**models.rs:**
- Serialization/deserialization
- Type conversions
- Default values

**embedding.rs:**
- Deterministic output
- Dimension count
- Range validation
- Match with Node.js output

**store.rs:**
- Database initialization
- CRUD operations
- Search algorithms
- Compaction logic

**api.rs:**
- Request validation
- Response formatting
- Error handling
- Status codes

### Integration Tests
```rust
// tests/knowledge_integration_tests.rs
#[tokio::test]
async fn test_full_workflow() {
    // Start server
    // Store items via API
    // Search via API
    // Verify results
    // Test compaction
}
```

---

## Implementation Order

1. **Phase 1: Foundation** (Day 1-2)
   - `models.rs` - Data structures
   - `embedding.rs` - Vector generation
   - `mod.rs` - Module setup

2. **Phase 2: Core Logic** (Day 3-4)
   - `store.rs` - Database operations
   - Unit tests for core functions

3. **Phase 3: API Layer** (Day 5)
   - `api.rs` - HTTP endpoints
   - Integration with `server.rs`

4. **Phase 4: Testing** (Day 6-7)
   - Comprehensive unit tests
   - Integration tests
   - Node.js compatibility validation

---

## Critical Success Factors

### 1. Embedding Compatibility
```rust
// Test vector to ensure compatibility
// Input: "test"
// Expected first 10 values: [-0.3984375, 0.6796875, ...]
```

### 2. ID Generation Format
```rust
format!("{}-{}-{}",
    knowledge_type.to_string().to_lowercase(),
    timestamp_millis,
    random_7_chars
)
```

### 3. Error Recovery
- Never panic in production
- Log errors with context
- Return degraded service rather than fail

### 4. Performance Metrics
- Measure and log operation latencies
- Track database size growth
- Monitor memory usage

---

## Common Pitfalls to Avoid

1. **Don't use real embedding models** - Must use SHA256 for compatibility
2. **Don't change vector dimensions** - Must be exactly 384
3. **Don't modify ID format** - Agents may depend on it
4. **Don't skip error handling** - Every Result must be handled
5. **Don't block the async runtime** - Use spawn_blocking for CPU work
6. **Don't forget per-repo isolation** - Each repo gets own database
7. **Don't change JSON field names** - Must match Node.js exactly

---

## Debugging Checklist

- [ ] Embeddings match Node.js byte-for-byte?
- [ ] Database path includes repository name?
- [ ] All errors logged with context?
- [ ] API responses include success flag?
- [ ] Timestamps in ISO 8601 format?
- [ ] Metadata stored as JSON string?
- [ ] Search results include score?
- [ ] Cleanup doesn't actually delete (LanceDB limitation)?

---

## Future Enhancements (Not in MVP)

1. Real embedding models (sentence-transformers)
2. Database connection pooling
3. Caching layer for frequent searches
4. WebSocket for real-time updates
5. Authentication middleware
6. Metrics and monitoring
7. Database backup/restore
8. Query optimization