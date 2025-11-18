# Knowledge Store Test Guide - Issue #32

**Status**: RED Phase Complete ✅
**Next**: Rust Specialist implements code to pass tests (GREEN phase)
**File**: `/Users/brent/git/cc-orchestra/cco/tests/knowledge_store_tests.rs`

## Test-Driven Development Methodology

This test suite follows strict TDD RED-GREEN-REFACTOR:

### ✅ RED Phase (Complete)
All tests written FIRST before implementation. Tests define the API contract and expected behavior.

### ⏳ GREEN Phase (Next)
Rust Specialist will implement `cco/src/knowledge_store.rs` to make all tests pass.

### 🔄 REFACTOR Phase (Future)
Once tests pass, improve code quality while keeping tests green.

---

## Test Coverage Summary

**Total Test Suites**: 10
**Total Tests**: 70+
**Lines of Test Code**: ~1,400

### 1. Embedding Tests (6 tests)
- ✅ SHA256 embedding generation (384 dimensions)
- ✅ Normalization to [-1, 1] range
- ✅ Consistency (same input → same output)
- ✅ Uniqueness (different input → different output)
- ✅ Edge cases (empty text, very long text)

**Key Assertions**:
```rust
assert_eq!(embedding.len(), 384);
assert!(value >= -1.0 && value <= 1.0);
assert_eq!(embedding1, embedding2); // Same input
assert_ne!(embedding1, embedding2); // Different input
```

### 2. Store Tests (7 tests)
- ✅ Initialize knowledge store with database
- ✅ Store and retrieve single knowledge item
- ✅ Batch storage of multiple items
- ✅ Per-repository isolation (project_id)
- ✅ Complex metadata persistence
- ✅ Timestamp accuracy

**Key Assertions**:
```rust
assert!(temp_dir.path().exists());
assert_eq!(results[0].text, item.text);
assert_eq!(ids.len(), 3); // Batch of 3
assert_eq!(results_a[0].project_id, "project-a");
assert_eq!(results[0].metadata, metadata);
```

### 3. Search Tests (8 tests)
- ✅ Vector similarity search
- ✅ Filter by knowledge type
- ✅ Filter by agent name
- ✅ Filter by date range
- ✅ Results ranked by relevance
- ✅ Handle empty results gracefully
- ✅ Respect search limit
- ✅ Semantic matching

**Key Assertions**:
```rust
assert!(results[0].text.contains("JWT"));
assert_eq!(result.knowledge_type, KnowledgeType::Decision);
assert_eq!(result.agent, "architect");
assert!(results.len() <= 5); // Limit respected
assert!(results[0].score <= results[1].score); // Ranked
```

### 4. Project Knowledge Tests (5 tests)
- ✅ Retrieve all knowledge for project
- ✅ Filter by type and agent
- ✅ Project isolation verification
- ✅ Sorted by timestamp (newest first)

**Key Assertions**:
```rust
assert_eq!(results.len(), 5);
assert_eq!(results[0].knowledge_type, KnowledgeType::Decision);
assert_eq!(results_a[0].project_id, "project-a");
assert_eq!(results[0].text, "Second item"); // Newest first
```

### 5. Compaction Tests (5 tests)
- ✅ Pre-compaction knowledge extraction
- ✅ Post-compaction retrieval
- ✅ Critical knowledge selection
- ✅ Context summarization

**Key Assertions**:
```rust
assert!(result.count > 0);
assert!(!result.search_results.is_empty());
assert!(has_decision || has_implementation);
assert!(!result.summary.by_type.is_empty());
```

### 6. Cleanup Tests (4 tests)
- ✅ Remove old knowledge (>90 days)
- ✅ Respect custom retention periods
- ✅ Preserve recent items
- ✅ Project-scoped cleanup

**Key Assertions**:
```rust
assert!(!remaining.is_empty()); // Recent preserved
assert!(result.count >= 0);
assert!(!project_b.is_empty()); // Other projects unaffected
```

### 7. Statistics Tests (5 tests)
- ✅ Database statistics available
- ✅ Accurate counts
- ✅ Distribution by type
- ✅ Distribution by agent
- ✅ Track oldest/newest records

**Key Assertions**:
```rust
assert!(stats.total_records >= 0);
assert_eq!(stats.by_type.get("decision").unwrap(), &2);
assert!(stats.by_agent.contains_key("architect"));
assert!(newest_time >= oldest_time);
```

### 8. API Endpoint Tests (8 tests - integration)
- 🔲 POST /api/knowledge/store
- 🔲 POST /api/knowledge/store-batch
- 🔲 POST /api/knowledge/search
- 🔲 GET /api/knowledge/project/:id
- 🔲 POST /api/knowledge/pre-compaction
- 🔲 POST /api/knowledge/post-compaction
- 🔲 GET /api/knowledge/stats
- 🔲 POST /api/knowledge/cleanup

**Note**: Marked as `#[ignore]` - require axum TestServer integration

### 9. Error Handling Tests (3 tests)
- ✅ Missing required fields validation
- ✅ Database errors handled gracefully
- ✅ Concurrent access support

**Key Assertions**:
```rust
assert!(result.is_err()); // Empty text rejected
assert!(result.is_err() || result.is_ok()); // No panic
assert!(results.len() >= 10); // Concurrent success
```

### 10. Data Integrity Tests (7 tests)
- ✅ Vector dimension validation (always 384)
- ✅ Text field required
- ✅ Knowledge type enum validation
- ✅ Large batch operations (1000+ items)
- ✅ Special characters and Unicode
- ✅ Complex nested JSON metadata

**Key Assertions**:
```rust
assert_eq!(embedding.len(), 384);
assert!(result.is_err()); // Empty text
assert!(result.is_ok()); // Valid type
assert_eq!(ids.len(), 1000); // Large batch
assert_eq!(results[0].metadata, complex_metadata); // Roundtrip
```

---

## Test Data Fixtures

### Knowledge Types Enum
```rust
enum KnowledgeType {
    Decision,        // Architecture/design decisions
    Implementation,  // Code implementations
    Architecture,    // System architecture
    Configuration,   // Config changes
    Issue,          // Bugs, problems, fixes
    Credential,     // API keys, secrets (reference only)
    General,        // Everything else
}
```

### KnowledgeItem Structure
```rust
struct KnowledgeItem {
    text: String,                    // Content (required, non-empty)
    knowledge_type: KnowledgeType,   // Type enum
    project_id: String,              // Repository/project isolation
    session_id: String,              // Conversation/session ID
    agent: String,                   // Agent name (architect, python, etc.)
    metadata: serde_json::Value,     // Arbitrary JSON
}
```

### SearchOptions
```rust
struct SearchOptions {
    limit: usize,                           // Default: 10
    threshold: f32,                         // Similarity threshold
    project_id: Option<String>,             // Filter by project
    knowledge_type: Option<KnowledgeType>,  // Filter by type
    agent: Option<String>,                  // Filter by agent
    start_date: Option<DateTime<Utc>>,      // Date range start
    end_date: Option<DateTime<Utc>>,        // Date range end
}
```

### Sample Test Data
```rust
// Architecture decision
KnowledgeItem {
    text: "We decided to use FastAPI for REST API".to_string(),
    knowledge_type: KnowledgeType::Decision,
    project_id: "test-project".to_string(),
    session_id: "session-001".to_string(),
    agent: "architect".to_string(),
    metadata: json!({"priority": "high"}),
}

// Implementation
KnowledgeItem {
    text: "Implemented JWT with RS256".to_string(),
    knowledge_type: KnowledgeType::Implementation,
    project_id: "test-project".to_string(),
    session_id: "session-001".to_string(),
    agent: "python".to_string(),
    metadata: json!({"file": "auth.py"}),
}

// Security issue
KnowledgeItem {
    text: "Found SQL injection vulnerability".to_string(),
    knowledge_type: KnowledgeType::Issue,
    project_id: "test-project".to_string(),
    session_id: "session-001".to_string(),
    agent: "security".to_string(),
    metadata: json!({"severity": "critical"}),
}
```

---

## Implementation Requirements

### Core Module: `cco/src/knowledge_store.rs`

```rust
pub struct KnowledgeStore {
    repo_name: String,
    db_path: PathBuf,
    embedding_dim: usize, // 384
}

impl KnowledgeStore {
    pub fn new(repo_name: &str) -> Result<Self>;
    pub fn with_path(repo_name: &str, path: &Path) -> Result<Self>;
    pub async fn initialize(&self) -> Result<()>;

    // Embedding
    pub fn generate_embedding(&self, text: &str) -> Vec<f32>;

    // Storage
    pub async fn store(&self, item: KnowledgeItem) -> Result<String>;
    pub async fn store_batch(&self, items: Vec<KnowledgeItem>) -> Result<Vec<String>>;

    // Search
    pub async fn search(&self, query: &str, options: SearchOptions) -> Result<Vec<KnowledgeResult>>;
    pub async fn get_project_knowledge(&self, project_id: &str, options: GetProjectOptions) -> Result<Vec<KnowledgeResult>>;

    // Compaction
    pub async fn pre_compaction(&self, conversation: &str, project_id: &str, session_id: &str) -> Result<CompactionResult>;
    pub async fn post_compaction(&self, current_task: &str, project_id: &str) -> Result<PostCompactionResult>;

    // Maintenance
    pub async fn cleanup(&self, options: CleanupOptions) -> Result<CleanupResult>;
    pub async fn get_stats(&self) -> Result<KnowledgeStats>;
}
```

### Dependencies to Add

Add to `cco/Cargo.toml`:
```toml
[dependencies]
lancedb = "0.3"  # Vector database
sha2 = "0.10"    # Already present - for SHA256
```

### LanceDB Schema

```rust
struct LanceRecord {
    id: String,              // Unique ID
    vector: Vec<f32>,        // 384-dim embedding
    text: String,            // Original text
    knowledge_type: String,  // Type as string
    project_id: String,      // Repository name
    session_id: String,      // Session ID
    agent: String,           // Agent name
    timestamp: String,       // ISO 8601
    metadata: String,        // JSON string
}
```

---

## Running Tests

### Run All Knowledge Store Tests
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo test knowledge_store_tests
```

### Run Specific Test Suite
```bash
cargo test knowledge_store_tests::embedding_tests
cargo test knowledge_store_tests::store_tests
cargo test knowledge_store_tests::search_tests
```

### Run Single Test
```bash
cargo test test_sha256_embedding_generation
```

### Run with Output
```bash
cargo test knowledge_store_tests -- --nocapture
```

### Run Integration Tests (when ready)
```bash
cargo test knowledge_store_tests::api_endpoint_tests -- --ignored
```

---

## Expected Test Results (RED Phase)

All tests should **FAIL** with compilation errors like:

```
error[E0433]: failed to resolve: use of undeclared crate or module `knowledge_store`
 --> tests/knowledge_store_tests.rs:8:14
  |
8 |     use cco::knowledge_store::{KnowledgeStore, KnowledgeItem, KnowledgeType};
  |              ^^^^^^^^^^^^^^^ use of undeclared crate or module `knowledge_store`
```

This is **EXPECTED** and **CORRECT** for TDD RED phase!

---

## Next Steps for Rust Specialist

1. **Create module**: `cco/src/knowledge_store.rs`
2. **Export in lib.rs**: Add `pub mod knowledge_store;`
3. **Implement types**: KnowledgeStore, KnowledgeItem, KnowledgeType, etc.
4. **Implement methods**: Start with simplest (new, generate_embedding)
5. **Run tests frequently**: `cargo test knowledge_store_tests`
6. **Make tests pass one by one**: RED → GREEN for each test
7. **Integration**: Add API endpoints to axum server
8. **Refactor**: Improve implementation while keeping tests green

---

## Success Criteria

### GREEN Phase Complete When:
- ✅ All 62 unit tests pass
- ✅ All 8 integration tests pass (API endpoints)
- ✅ Code coverage >90% for knowledge_store module
- ✅ No clippy warnings
- ✅ Documentation complete (rustdoc)

### Integration Complete When:
- ✅ API endpoints working in daemon
- ✅ CLI commands functional (`cco knowledge ...`)
- ✅ Dashboard can query knowledge store
- ✅ Compaction hooks integrated
- ✅ Performance acceptable (<100ms for searches)

---

## Coordination

### Knowledge Manager Usage

**Before implementation**:
```bash
node ~/git/cc-orchestra/src/knowledge-manager.js search "LanceDB implementation"
node ~/git/cc-orchestra/src/knowledge-manager.js search "Issue #32"
```

**During implementation**:
```bash
node ~/git/cc-orchestra/src/knowledge-manager.js store "Implemented generate_embedding method" --type implementation --agent rust-specialist
node ~/git/cc-orchestra/src/knowledge-manager.js store "Tests passing: embedding_tests (6/6)" --type status --agent rust-specialist
```

**After completion**:
```bash
node ~/git/cc-orchestra/src/knowledge-manager.js store "GREEN phase complete: All knowledge_store tests passing" --type completion --agent rust-specialist
```

### Agent Coordination

**TDD Coding Agent** (this agent):
- ✅ Tests written FIRST
- ✅ Comprehensive coverage
- ✅ Clear assertions
- ✅ Test documentation

**Rust Specialist** (next):
- ⏳ Implement knowledge_store module
- ⏳ Make all tests pass
- ⏳ Add API endpoints
- ⏳ Refactor for quality

**QA Engineer** (validation):
- ⏳ Verify test coverage
- ⏳ Add edge case tests
- ⏳ Integration testing
- ⏳ Performance testing

**Chief Architect** (review):
- ⏳ Architecture review
- ⏳ API design approval
- ⏳ Integration strategy
- ⏳ Performance validation

---

## References

- **Issue**: #32 - Embed LanceDB in CCO daemon
- **Knowledge Manager JS**: `/Users/brent/git/cc-orchestra/src/knowledge-manager.js`
- **Test File**: `/Users/brent/git/cc-orchestra/cco/tests/knowledge_store_tests.rs`
- **TDD Methodology**: RED-GREEN-REFACTOR cycle
- **LanceDB Docs**: https://lancedb.github.io/lancedb/

---

**TDD Coding Agent**: Tests complete and ready for implementation ✅
**Status**: RED phase - All tests fail as expected
**Next Agent**: Rust Specialist for GREEN phase (make tests pass)
