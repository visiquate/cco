# Logic Migration Verification Matrix
## Node.js → Rust Knowledge Manager

**QA Engineer Report**
**Date:** November 18, 2025
**Source:** `/Users/brent/git/cc-orchestra/src/knowledge-manager.js`
**Target:** `cco/src/knowledge/` (NOT YET IMPLEMENTED)

---

## Overview

This document maps every method from the Node.js knowledge-manager.js to its expected Rust equivalent. Each method must be replicated with identical functionality to ensure backward compatibility.

**Total Methods to Replicate:** 15 core methods + CLI interface

---

## Method-by-Method Comparison Matrix

### 1. Constructor / Initialization

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| `constructor(options)` | `KnowledgeManager::new(config)` | ❌ Not implemented | Must support per-repo databases |
| Lines 16-32 | `manager.rs` | ❌ | **CRITICAL**: Repository name extraction from path |
| Repository isolation | Per-repo database path | ❌ | Format: `{baseDir}/{repoName}/` |
| Database path | `~/.cco/knowledge/{repo}/` | ❌ | **CHANGE**: Move from `data/knowledge/` to `~/.cco/knowledge/` |

**Key Implementation Notes:**
- Extract repo name from current directory path
- Create per-repository database isolation
- Default baseDir: `~/.cco/knowledge/` (NOT `data/knowledge/`)
- Default tableName: `orchestra_knowledge`
- Embedding dimension: 384 (configurable)

**Test Cases:**
```rust
#[test]
fn test_repository_name_extraction() {
    let km = KnowledgeManager::new(KnowledgeConfig {
        repo_path: "/Users/test/git/my-project",
        ..Default::default()
    });
    assert_eq!(km.repo_name, "my-project");
}

#[test]
fn test_per_repo_database_path() {
    let km1 = KnowledgeManager::new(KnowledgeConfig {
        repo_path: "/path/to/project-a",
        ..Default::default()
    });
    let km2 = KnowledgeManager::new(KnowledgeConfig {
        repo_path: "/path/to/project-b",
        ..Default::default()
    });
    assert_ne!(km1.db_path, km2.db_path);
}
```

---

### 2. getRepoName(repoPath)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 37-40 | `fn get_repo_name(path: &Path) -> String` | ❌ Not implemented | Extract last path component |
| Implementation | Split by path separator, return last | ❌ | Fallback: `"default"` if empty |

**Node.js Code:**
```javascript
getRepoName(repoPath) {
  const parts = repoPath.split(path.sep);
  return parts[parts.length - 1] || 'default';
}
```

**Expected Rust:**
```rust
fn get_repo_name(repo_path: &Path) -> String {
    repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "default".to_string())
}
```

**Test Cases:**
```rust
#[test]
fn test_repo_name_extraction() {
    assert_eq!(get_repo_name(Path::new("/users/me/projects/myapp")), "myapp");
    assert_eq!(get_repo_name(Path::new("/myapp")), "myapp");
    assert_eq!(get_repo_name(Path::new("/")), "default");
    assert_eq!(get_repo_name(Path::new("")), "default");
}
```

---

### 3. initialize()

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 45-68 | `async fn initialize() -> Result<()>` | ❌ Not implemented | Create dir, connect to DB, open/create table |
| Create directory | `fs::create_dir_all()` | ❌ | Recursive creation |
| Connect to LanceDB | `lancedb::connect()` | ❌ | Async operation |
| Open existing table | `db.open_table()` | ❌ | Try first |
| Create if missing | `create_table()` | ❌ | Fallback if open fails |

**Node.js Flow:**
```javascript
async initialize() {
  await fs.mkdir(this.dbPath, { recursive: true });
  this.db = await lancedb.connect(this.dbPath);
  try {
    this.table = await this.db.openTable(this.tableName);
  } catch (error) {
    await this.createTable();
  }
}
```

**Expected Rust:**
```rust
pub async fn initialize(&mut self) -> Result<(), KnowledgeError> {
    // Create directory
    tokio::fs::create_dir_all(&self.db_path).await?;

    // Connect to LanceDB
    self.db = Some(lancedb::connect(&self.db_path).await?);

    // Try to open existing table
    match self.db.as_ref().unwrap().open_table(&self.table_name).await {
        Ok(table) => {
            self.table = Some(table);
            println!("✅ Connected to existing knowledge base for {}", self.repo_name);
        }
        Err(_) => {
            println!("📝 Creating new knowledge base for {}", self.repo_name);
            self.create_table().await?;
        }
    }

    Ok(())
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_initialize_new_database() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut km = KnowledgeManager::new(KnowledgeConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    });

    assert!(km.initialize().await.is_ok());
    assert!(km.db.is_some());
    assert!(km.table.is_some());
}

#[tokio::test]
async fn test_initialize_existing_database() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut km1 = KnowledgeManager::new(KnowledgeConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    });
    km1.initialize().await.unwrap();
    drop(km1);

    // Reopen same database
    let mut km2 = KnowledgeManager::new(KnowledgeConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    });
    assert!(km2.initialize().await.is_ok());
}
```

---

### 4. createTable()

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 73-90 | `async fn create_table() -> Result<()>` | ❌ Not implemented | Create with init record |
| Schema definition | Arrow schema | ❌ | **CRITICAL**: Must match exactly |
| Init record | System record with zeros | ❌ | ID: `init-{timestamp}` |

**Node.js Schema:**
```javascript
{
  id: 'init-' + Date.now(),
  vector: Array(384).fill(0),
  text: 'Initialization record',
  type: 'system',
  project_id: 'system',
  session_id: 'init',
  agent: 'system',
  timestamp: new Date().toISOString(),
  metadata: JSON.stringify({})
}
```

**Expected Rust Arrow Schema:**
```rust
fn knowledge_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("vector", DataType::FixedSizeList(
            Arc::new(Field::new("item", DataType::Float32, true)),
            384
        ), false),
        Field::new("text", DataType::Utf8, false),
        Field::new("type", DataType::Utf8, false),
        Field::new("project_id", DataType::Utf8, false),
        Field::new("session_id", DataType::Utf8, false),
        Field::new("agent", DataType::Utf8, false),
        Field::new("timestamp", DataType::Utf8, false), // ISO8601 string
        Field::new("metadata", DataType::Utf8, false), // JSON string
    ]))
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_create_table_schema() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut km = KnowledgeManager::new(KnowledgeConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    });

    km.create_table().await.unwrap();

    // Verify init record exists
    let stats = km.get_stats().await.unwrap();
    assert_eq!(stats.total_records, 1);
    assert_eq!(stats.by_type.get("system"), Some(&1));
}
```

---

### 5. generateEmbedding(text) - CRITICAL

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 96-108 | `fn generate_embedding(&self, text: &str) -> Vec<f32>` | ❌ Not implemented | **SHA256 hash-based** |
| Hash algorithm | SHA256 | ❌ | **MUST MATCH** for compatibility |
| Normalization | `(hash[i % 32] / 128.0) - 1.0` | ❌ | Range: [-1.0, 1.0] |
| Dimension | 384 | ❌ | Repeats hash bytes |

**⚠️ CRITICAL COMPATIBILITY ISSUE:**

The Node.js implementation does NOT use real embeddings. It uses SHA256 hash as a pseudo-embedding:

```javascript
generateEmbedding(text) {
  const hash = crypto.createHash('sha256').update(text).digest(); // 32 bytes
  const embedding = [];

  for (let i = 0; i < 384; i++) {
    // Cycle through hash bytes, normalize to [-1, 1]
    embedding.push((hash[i % hash.length] / 128.0) - 1.0);
  }

  return embedding;
}
```

**Expected Rust (Backward Compatible):**
```rust
use sha2::{Sha256, Digest};

fn generate_embedding(&self, text: &str) -> Vec<f32> {
    // MUST use SHA256 for backward compatibility
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();

    let mut embedding = Vec::with_capacity(self.embedding_dim);
    for i in 0..self.embedding_dim {
        // Normalize: (hash_byte / 128.0) - 1.0
        // Range: [-1.0, 1.0]
        let byte = hash[i % hash.len()] as f32;
        embedding.push((byte / 128.0) - 1.0);
    }

    embedding
}
```

**Test Cases (EXACT MATCH REQUIRED):**
```rust
#[test]
fn test_embedding_exact_match() {
    // This test MUST pass for backward compatibility
    let km = KnowledgeManager::new(Default::default());
    let text = "We decided to use FastAPI";

    // Generate embedding in Rust
    let rust_embedding = km.generate_embedding(text);

    // Expected from Node.js (pre-computed)
    let nodejs_embedding = nodejs_generate_embedding(text); // Helper function

    // MUST match exactly
    assert_eq!(rust_embedding.len(), nodejs_embedding.len());
    for (i, (r, n)) in rust_embedding.iter().zip(nodejs_embedding.iter()).enumerate() {
        assert!(
            (r - n).abs() < 0.0001,
            "Embedding mismatch at index {}: Rust={}, Node.js={}",
            i, r, n
        );
    }
}

#[test]
fn test_embedding_properties() {
    let km = KnowledgeManager::new(Default::default());
    let embedding = km.generate_embedding("test text");

    assert_eq!(embedding.len(), 384);
    assert!(embedding.iter().all(|&v| v >= -1.0 && v <= 1.0));
}

#[test]
fn test_embedding_deterministic() {
    let km = KnowledgeManager::new(Default::default());
    let text = "same input";

    let emb1 = km.generate_embedding(text);
    let emb2 = km.generate_embedding(text);

    assert_eq!(emb1, emb2);
}
```

**DECISION NEEDED:**

The Chief Architect must decide:

1. **Option A: Backward Compatible (SHA256)**
   - Pros: Existing data works, no migration needed
   - Cons: Poor semantic search quality
   - Recommendation: Start here for MVP

2. **Option B: Real Embeddings (sentence-transformers)**
   - Pros: Much better search quality
   - Cons: Requires data migration, larger binary
   - Recommendation: Phase 2 enhancement

3. **Option C: Hybrid Approach**
   - Support both embedding types
   - Add `embedding_version` field to schema
   - Gradual migration
   - Recommendation: Best long-term solution

---

### 6. store(knowledge)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 113-153 | `async fn store(&self, req: StoreRequest) -> Result<String>` | ❌ Not implemented | Store single entry |
| Validation | Check text is non-empty string | ❌ | Return error if invalid |
| ID generation | `{type}-{timestamp}-{random}` | ❌ | **MUST MATCH** format |
| Embedding | Call `generate_embedding()` | ❌ | Use SHA256 method |
| Timestamp | ISO8601 string | ❌ | `Utc::now().to_rfc3339()` |
| Metadata | Serialize to JSON string | ❌ | serde_json |

**Node.js ID Format:**
```javascript
id: `${type}-${Date.now()}-${Math.random().toString(36).substring(7)}`
// Example: "decision-1700000000000-abc123x"
```

**Expected Rust:**
```rust
#[derive(Debug, Deserialize)]
pub struct StoreRequest {
    pub text: String,
    #[serde(default = "default_type")]
    pub type_: String,  // "decision", "implementation", etc.
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl KnowledgeManager {
    pub async fn store(&self, req: StoreRequest) -> Result<String, KnowledgeError> {
        // Validate
        if req.text.trim().is_empty() {
            return Err(KnowledgeError::InvalidQuery("Text is required".into()));
        }

        // Generate embedding
        let vector = self.generate_embedding(&req.text);

        // Generate ID
        let timestamp = Utc::now().timestamp_millis();
        let random = generate_random_id(); // 7-char alphanumeric
        let id = format!("{}-{}-{}", req.type_, timestamp, random);

        // Create record
        let record = KnowledgeEntry {
            id: id.clone(),
            vector,
            text: req.text,
            type_: req.type_,
            project_id: req.project_id.unwrap_or_else(|| self.repo_name.clone()),
            session_id: req.session_id,
            agent: req.agent,
            timestamp: Utc::now().to_rfc3339(),
            metadata: serde_json::to_string(&req.metadata)?,
        };

        // Insert into table
        self.table.add(vec![record]).await?;

        println!("✅ Stored knowledge: {} from {}", record.type_, record.agent);
        Ok(id)
    }
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_store_single_entry() {
    let mut km = setup_test_km().await;

    let id = km.store(StoreRequest {
        text: "Test knowledge".into(),
        type_: "test".into(),
        project_id: Some("test-project".into()),
        agent: "test-agent".into(),
        ..Default::default()
    }).await.unwrap();

    assert!(id.starts_with("test-"));
    assert!(id.contains("-"));
}

#[tokio::test]
async fn test_store_validation() {
    let mut km = setup_test_km().await;

    // Empty text should fail
    let result = km.store(StoreRequest {
        text: "".into(),
        ..Default::default()
    }).await;

    assert!(result.is_err());
}
```

---

### 7. storeBatch(knowledgeItems)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 158-172 | `async fn store_batch(&self, items: Vec<StoreRequest>) -> Result<Vec<String>>` | ❌ Not implemented | Store multiple entries |
| Error handling | Continue on individual failures | ❌ | Log warnings, return successful IDs |
| Return | Array of successful IDs | ❌ | Partial success allowed |

**Node.js Implementation:**
```javascript
async storeBatch(knowledgeItems) {
  const ids = [];
  for (const item of knowledgeItems) {
    try {
      const id = await this.store(item);
      ids.push(id);
    } catch (error) {
      console.error(`⚠️  Failed to store item: ${error.message}`);
    }
  }
  return ids;
}
```

**Expected Rust:**
```rust
pub async fn store_batch(
    &self,
    items: Vec<StoreRequest>
) -> Result<Vec<String>, KnowledgeError> {
    let mut ids = Vec::new();

    for item in items {
        match self.store(item).await {
            Ok(id) => ids.push(id),
            Err(e) => {
                eprintln!("⚠️  Failed to store item: {}", e);
            }
        }
    }

    println!("✅ Stored {}/{} knowledge items", ids.len(), items.len());
    Ok(ids)
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_store_batch_all_success() {
    let mut km = setup_test_km().await;

    let items = vec![
        StoreRequest { text: "Item 1".into(), ..Default::default() },
        StoreRequest { text: "Item 2".into(), ..Default::default() },
        StoreRequest { text: "Item 3".into(), ..Default::default() },
    ];

    let ids = km.store_batch(items).await.unwrap();
    assert_eq!(ids.len(), 3);
}

#[tokio::test]
async fn test_store_batch_partial_failure() {
    let mut km = setup_test_km().await;

    let items = vec![
        StoreRequest { text: "Valid".into(), ..Default::default() },
        StoreRequest { text: "".into(), ..Default::default() }, // Invalid
        StoreRequest { text: "Also valid".into(), ..Default::default() },
    ];

    let ids = km.store_batch(items).await.unwrap();
    assert_eq!(ids.len(), 2); // Only valid items
}
```

---

### 8. search(query, options)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 177-225 | `async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>` | ❌ Not implemented | Vector similarity search |
| Query embedding | Generate vector from query text | ❌ | Use SHA256 method |
| LanceDB search | `.search(vector).limit(N)` | ❌ | Vector similarity |
| Metadata filtering | Filter by project_id, type, agent | ❌ | **AFTER** vector search |
| Score field | `_distance` in LanceDB | ❌ | Include in results |
| Result format | Parse metadata JSON | ❌ | Deserialize metadata field |

**Node.js Implementation:**
```javascript
async search(query, options = {}) {
  const { limit = 10, threshold = 0.5, project_id, type, agent } = options;

  const queryVector = this.generateEmbedding(query);

  let results = await this.table
    .search(queryVector)
    .limit(limit)
    .execute();

  // Filter by metadata
  if (project_id || type || agent) {
    results = results.filter(result => {
      if (project_id && result.project_id !== project_id) return false;
      if (type && result.type !== type) return false;
      if (agent && result.agent !== agent) return false;
      return true;
    });
  }

  return results.map(result => ({
    ...result,
    metadata: JSON.parse(result.metadata || '{}'),
    score: result._distance
  }));
}
```

**Expected Rust:**
```rust
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub threshold: Option<f32>,
    pub project_id: Option<String>,
    pub type_: Option<String>,
    pub agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    pub type_: String,
    pub project_id: String,
    pub session_id: String,
    pub agent: String,
    pub timestamp: String,
    pub metadata: serde_json::Value,
    pub score: f32, // _distance from LanceDB
}

impl KnowledgeManager {
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, KnowledgeError> {
        // Generate query vector
        let query_vector = self.generate_embedding(&query.q);

        // Execute vector search
        let mut results = self.table
            .search(query_vector)
            .limit(query.limit)
            .execute()
            .await?;

        // Filter by metadata
        if query.project_id.is_some() || query.type_.is_some() || query.agent.is_some() {
            results.retain(|r| {
                if let Some(ref pid) = query.project_id {
                    if r.project_id != *pid { return false; }
                }
                if let Some(ref t) = query.type_ {
                    if r.type_ != *t { return false; }
                }
                if let Some(ref a) = query.agent {
                    if r.agent != *a { return false; }
                }
                true
            });
        }

        // Format results
        let formatted: Vec<SearchResult> = results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                text: r.text,
                type_: r.type_,
                project_id: r.project_id,
                session_id: r.session_id,
                agent: r.agent,
                timestamp: r.timestamp,
                metadata: serde_json::from_str(&r.metadata).unwrap_or_default(),
                score: r._distance,
            })
            .collect();

        println!("🔍 Found {} relevant knowledge items", formatted.len());
        Ok(formatted)
    }
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_search_basic() {
    let mut km = setup_test_km_with_data().await;

    let results = km.search(SearchQuery {
        q: "authentication".into(),
        limit: 5,
        ..Default::default()
    }).await.unwrap();

    assert!(results.len() > 0);
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_search_with_filters() {
    let mut km = setup_test_km_with_data().await;

    let results = km.search(SearchQuery {
        q: "API".into(),
        limit: 10,
        project_id: Some("test-project".into()),
        type_: Some("decision".into()),
        ..Default::default()
    }).await.unwrap();

    // All results should match filters
    assert!(results.iter().all(|r| r.project_id == "test-project"));
    assert!(results.iter().all(|r| r.type_ == "decision"));
}
```

---

### 9. getProjectKnowledge(project_id, options)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 230-274 | `async fn get_project_knowledge(&self, project_id: &str) -> Result<Vec<KnowledgeEntry>>` | ❌ Not implemented | Get all entries for project |
| Retrieval method | Search with dummy vector | ❌ | Workaround (LanceDB limitation) |
| Filtering | Filter by project_id | ❌ | **AFTER** retrieval |
| Type filter | Optional type filter | ❌ | Additional filtering |
| Sorting | Newest first (by timestamp) | ❌ | **CRITICAL** |
| Limit | Default 100, configurable | ❌ | Slice after sorting |

**Node.js Workaround:**
```javascript
async getProjectKnowledge(project_id, options = {}) {
  // LanceDB doesn't have toArray(), so use search with dummy vector
  const dummyVector = Array(384).fill(0);
  const allRecords = await this.table.search(dummyVector).limit(1000).execute();

  let filtered = allRecords.filter(r => r.project_id === project_id);
  if (type) filtered = filtered.filter(r => r.type === type);

  // Sort by timestamp (newest first)
  filtered.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

  return filtered.slice(0, limit);
}
```

**Expected Rust:**
```rust
pub async fn get_project_knowledge(
    &self,
    project_id: &str,
    type_: Option<&str>,
    limit: usize
) -> Result<Vec<KnowledgeEntry>, KnowledgeError> {
    // Workaround: LanceDB search with dummy vector
    let dummy_vector = vec![0.0_f32; self.embedding_dim];

    let all_records = self.table
        .search(dummy_vector)
        .limit(1000) // Get many records
        .execute()
        .await?;

    // Filter by project_id
    let mut filtered: Vec<_> = all_records
        .into_iter()
        .filter(|r| r.project_id == project_id)
        .collect();

    // Optional type filter
    if let Some(t) = type_ {
        filtered.retain(|r| r.type_ == t);
    }

    // Sort by timestamp (newest first)
    filtered.sort_by(|a, b| {
        b.timestamp.cmp(&a.timestamp)
    });

    // Limit results
    filtered.truncate(limit);

    // Format results
    let formatted: Vec<KnowledgeEntry> = filtered
        .into_iter()
        .map(|r| KnowledgeEntry {
            // ... convert from LanceDB record
        })
        .collect();

    println!("📚 Retrieved {} knowledge items for project: {}", formatted.len(), project_id);
    Ok(formatted)
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_get_project_knowledge() {
    let mut km = setup_test_km_with_data().await;

    // Store items for multiple projects
    km.store(StoreRequest {
        text: "Project A item".into(),
        project_id: Some("project-a".into()),
        ..Default::default()
    }).await.unwrap();

    km.store(StoreRequest {
        text: "Project B item".into(),
        project_id: Some("project-b".into()),
        ..Default::default()
    }).await.unwrap();

    let results = km.get_project_knowledge("project-a", None, 100).await.unwrap();

    assert!(results.iter().all(|r| r.project_id == "project-a"));
}

#[tokio::test]
async fn test_project_knowledge_sorted_by_timestamp() {
    let mut km = setup_test_km_with_data().await;

    // Store items with delays
    for i in 1..=5 {
        km.store(StoreRequest {
            text: format!("Item {}", i),
            project_id: Some("test".into()),
            ..Default::default()
        }).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let results = km.get_project_knowledge("test", None, 10).await.unwrap();

    // Should be in reverse chronological order
    assert_eq!(results[0].text, "Item 5");
    assert_eq!(results[4].text, "Item 1");
}
```

---

### 10. preCompaction(conversation, context)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 279-292 | `async fn pre_compaction(&self, conversation: &str, context: HashMap) -> Result<PreCompactionResult>` | ❌ Not implemented | Extract and store knowledge |
| Extract knowledge | Call `extract_critical_knowledge()` | ❌ | Pattern matching |
| Store batch | Call `store_batch()` | ❌ | Store extracted items |
| Return | Success status + count + IDs | ❌ | Result struct |

**Node.js Implementation:**
```javascript
async preCompaction(conversation, context = {}) {
  console.log('🔄 Running pre-compaction knowledge capture...');

  const knowledge = this.extractCriticalKnowledge(conversation, context);
  const ids = await this.storeBatch(knowledge);

  return { success: true, count: ids.length, ids };
}
```

**Expected Rust:**
```rust
#[derive(Debug, Serialize)]
pub struct PreCompactionResult {
    pub success: bool,
    pub count: usize,
    pub ids: Vec<String>,
}

impl KnowledgeManager {
    pub async fn pre_compaction(
        &self,
        conversation: &str,
        context: HashMap<String, String>
    ) -> Result<PreCompactionResult, KnowledgeError> {
        println!("🔄 Running pre-compaction knowledge capture...");

        let knowledge = self.extract_critical_knowledge(conversation, &context);
        let ids = self.store_batch(knowledge).await?;

        println!("✅ Pre-compaction complete: Captured {} knowledge items", ids.len());

        Ok(PreCompactionResult {
            success: true,
            count: ids.len(),
            ids,
        })
    }
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_pre_compaction() {
    let mut km = setup_test_km().await;

    let conversation = "
        We decided to use FastAPI for the REST API.

        Implemented JWT authentication with RS256.

        Security audit found no critical issues.
    ";

    let mut context = HashMap::new();
    context.insert("project_id".into(), "test-project".into());
    context.insert("session_id".into(), "session-123".into());

    let result = km.pre_compaction(conversation, context).await.unwrap();

    assert!(result.success);
    assert!(result.count > 0);
    assert_eq!(result.ids.len(), result.count);
}
```

---

### 11. postCompaction(currentTask, context)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 297-326 | `async fn post_compaction(&self, task: &str, context: HashMap) -> Result<PostCompactionResult>` | ❌ Not implemented | Retrieve context |
| Search | Semantic search for current task | ❌ | Use search method |
| Recent knowledge | Get recent project items | ❌ | Use getProjectKnowledge |
| Summary | Generate context summary | ❌ | Call generateContextSummary |
| Return | Combined results + summary | ❌ | Result struct |

**Node.js Implementation:**
```javascript
async postCompaction(currentTask, context = {}) {
  const { project_id = 'default', limit = 10 } = context;

  const results = await this.search(currentTask, { limit, project_id });
  const recentKnowledge = await this.getProjectKnowledge(project_id, { limit: 5 });

  return {
    searchResults: results,
    recentKnowledge,
    summary: this.generateContextSummary(results, recentKnowledge)
  };
}
```

**Expected Rust:**
```rust
#[derive(Debug, Serialize)]
pub struct PostCompactionResult {
    pub search_results: Vec<SearchResult>,
    pub recent_knowledge: Vec<KnowledgeEntry>,
    pub summary: ContextSummary,
}

impl KnowledgeManager {
    pub async fn post_compaction(
        &self,
        current_task: &str,
        context: HashMap<String, String>
    ) -> Result<PostCompactionResult, KnowledgeError> {
        println!("🔄 Running post-compaction knowledge retrieval...");

        let project_id = context.get("project_id").map(|s| s.as_str()).unwrap_or("default");
        let limit = context.get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        // Search for relevant knowledge
        let search_results = self.search(SearchQuery {
            q: current_task.into(),
            limit,
            project_id: Some(project_id.into()),
            ..Default::default()
        }).await?;

        // Get recent project knowledge
        let recent_knowledge = self.get_project_knowledge(project_id, None, 5).await?;

        // Generate summary
        let summary = self.generate_context_summary(&search_results, &recent_knowledge);

        println!("✅ Post-compaction complete: Retrieved {} relevant items", search_results.len());

        Ok(PostCompactionResult {
            search_results,
            recent_knowledge,
            summary,
        })
    }
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_post_compaction() {
    let mut km = setup_test_km_with_data().await;

    let mut context = HashMap::new();
    context.insert("project_id".into(), "test-project".into());
    context.insert("limit".into(), "5".into());

    let result = km.post_compaction("authentication", context).await.unwrap();

    assert!(result.search_results.len() > 0);
    assert!(result.recent_knowledge.len() > 0);
    assert!(result.summary.total_items > 0);
}
```

---

### 12. extractCriticalKnowledge(conversation, context)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 331-380 | `fn extract_critical_knowledge(&self, conversation: &str, context: &HashMap) -> Vec<StoreRequest>` | ❌ Not implemented | Pattern matching |
| Patterns | Regex for architecture, decision, etc. | ❌ | **CRITICAL**: Must match patterns |
| Splitting | Split by `\n\n` (double newline) | ❌ | Message boundaries |
| Length filter | Skip messages <50 chars | ❌ | Avoid noise |
| Agent extraction | Regex for agent names | ❌ | Pattern: `\b(architect|python|...)\b` |

**Node.js Patterns:**
```javascript
const patterns = {
  architecture: /architecture|design pattern|system design/i,
  decision: /decided|chose|selected|will use/i,
  implementation: /implemented|built|created|added/i,
  configuration: /configured|setup|initialized/i,
  credential: /api key|secret|token|password|credential/i,
  issue: /bug|issue|problem|error|fix/i
};
```

**Expected Rust:**
```rust
use regex::Regex;

lazy_static! {
    static ref PATTERNS: HashMap<&'static str, Regex> = {
        let mut m = HashMap::new();
        m.insert("architecture", Regex::new(r"(?i)architecture|design pattern|system design").unwrap());
        m.insert("decision", Regex::new(r"(?i)decided|chose|selected|will use").unwrap());
        m.insert("implementation", Regex::new(r"(?i)implemented|built|created|added").unwrap());
        m.insert("configuration", Regex::new(r"(?i)configured|setup|initialized").unwrap());
        m.insert("credential", Regex::new(r"(?i)api key|secret|token|password|credential").unwrap());
        m.insert("issue", Regex::new(r"(?i)bug|issue|problem|error|fix").unwrap());
        m
    };

    static ref AGENT_PATTERN: Regex = Regex::new(
        r"\b(architect|python|swift|go|rust|flutter|qa|security|devops)\b"
    ).unwrap();
}

impl KnowledgeManager {
    fn extract_critical_knowledge(
        &self,
        conversation: &str,
        context: &HashMap<String, String>
    ) -> Vec<StoreRequest> {
        let project_id = context.get("project_id")
            .cloned()
            .unwrap_or_else(|| "default".into());
        let session_id = context.get("session_id")
            .cloned()
            .unwrap_or_else(|| "unknown".into());

        let mut knowledge = Vec::new();

        // Split by double newline
        let messages: Vec<&str> = conversation.split("\n\n").collect();

        for (index, message) in messages.iter().enumerate() {
            // Skip short messages
            if message.len() < 50 {
                continue;
            }

            // Detect knowledge type
            let mut type_ = "general".to_string();
            for (pattern_name, regex) in PATTERNS.iter() {
                if regex.is_match(message) {
                    type_ = pattern_name.to_string();
                    break;
                }
            }

            // Extract agent if mentioned
            let agent = AGENT_PATTERN
                .find(message)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_else(|| "unknown".into());

            knowledge.push(StoreRequest {
                text: message.trim().to_string(),
                type_,
                project_id: Some(project_id.clone()),
                session_id: session_id.clone(),
                agent,
                metadata: serde_json::json!({
                    "conversationIndex": index,
                    "extractedAt": Utc::now().to_rfc3339()
                }),
            });
        }

        println!("📊 Extracted {} knowledge items from conversation", knowledge.len());
        knowledge
    }
}
```

**Test Cases:**
```rust
#[test]
fn test_extract_knowledge_patterns() {
    let km = KnowledgeManager::new(Default::default());
    let mut context = HashMap::new();
    context.insert("project_id".into(), "test".into());

    let conversation = "
        We decided to use FastAPI for the REST API because it has great async support.

        The architect designed a microservices architecture with event-driven communication.

        We implemented JWT authentication with RS256 algorithm.

        Short.
    ";

    let knowledge = km.extract_critical_knowledge(conversation, &context);

    // Should extract 3 items (skip "Short.")
    assert_eq!(knowledge.len(), 3);

    // Check types
    assert_eq!(knowledge[0].type_, "decision");
    assert_eq!(knowledge[1].type_, "architecture");
    assert_eq!(knowledge[2].type_, "implementation");

    // Check agents
    assert_eq!(knowledge[1].agent, "architect");
}

#[test]
fn test_extract_knowledge_agent_detection() {
    let km = KnowledgeManager::new(Default::default());
    let mut context = HashMap::new();

    let conversation = "
        The security team found a vulnerability in the authentication flow.

        Python specialist implemented a fix using bcrypt for password hashing.
    ";

    let knowledge = km.extract_critical_knowledge(conversation, &context);

    assert_eq!(knowledge[0].agent, "security");
    assert_eq!(knowledge[1].agent, "python");
}
```

---

### 13. generateContextSummary(searchResults, recentKnowledge)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 385-420 | `fn generate_context_summary(&self, search: &[SearchResult], recent: &[KnowledgeEntry]) -> ContextSummary` | ❌ Not implemented | Aggregate statistics |
| Count by type | HashMap of type counts | ❌ | Iterate and count |
| Count by agent | HashMap of agent counts | ❌ | Iterate and count |
| Top decisions | First 5 decision items | ❌ | Filter + truncate text |
| Recent activity | Last 3 recent items | ❌ | Preview text |

**Node.js Implementation:**
```javascript
generateContextSummary(searchResults, recentKnowledge) {
  const summary = {
    totalItems: searchResults.length + recentKnowledge.length,
    byType: {},
    byAgent: {},
    topDecisions: [],
    recentActivity: []
  };

  const allItems = [...searchResults, ...recentKnowledge];

  allItems.forEach(item => {
    summary.byType[item.type] = (summary.byType[item.type] || 0) + 1;
    summary.byAgent[item.agent] = (summary.byAgent[item.agent] || 0) + 1;
  });

  summary.topDecisions = searchResults
    .filter(item => item.type === 'decision')
    .slice(0, 5)
    .map(item => item.text.substring(0, 100) + '...');

  summary.recentActivity = recentKnowledge
    .slice(0, 3)
    .map(item => ({
      type: item.type,
      agent: item.agent,
      timestamp: item.timestamp,
      preview: item.text.substring(0, 80) + '...'
    }));

  return summary;
}
```

**Expected Rust:**
```rust
#[derive(Debug, Serialize)]
pub struct ContextSummary {
    pub total_items: usize,
    pub by_type: HashMap<String, usize>,
    pub by_agent: HashMap<String, usize>,
    pub top_decisions: Vec<String>,
    pub recent_activity: Vec<ActivityPreview>,
}

#[derive(Debug, Serialize)]
pub struct ActivityPreview {
    pub type_: String,
    pub agent: String,
    pub timestamp: String,
    pub preview: String,
}

impl KnowledgeManager {
    fn generate_context_summary(
        &self,
        search_results: &[SearchResult],
        recent_knowledge: &[KnowledgeEntry]
    ) -> ContextSummary {
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_agent: HashMap<String, usize> = HashMap::new();

        // Count all items
        let all_items: Vec<_> = search_results.iter()
            .map(|r| (r.type_.clone(), r.agent.clone()))
            .chain(recent_knowledge.iter().map(|r| (r.type_.clone(), r.agent.clone())))
            .collect();

        for (type_, agent) in &all_items {
            *by_type.entry(type_.clone()).or_insert(0) += 1;
            *by_agent.entry(agent.clone()).or_insert(0) += 1;
        }

        // Top decisions
        let top_decisions: Vec<String> = search_results
            .iter()
            .filter(|r| r.type_ == "decision")
            .take(5)
            .map(|r| {
                let preview = if r.text.len() > 100 {
                    format!("{}...", &r.text[..100])
                } else {
                    r.text.clone()
                };
                preview
            })
            .collect();

        // Recent activity
        let recent_activity: Vec<ActivityPreview> = recent_knowledge
            .iter()
            .take(3)
            .map(|r| {
                let preview = if r.text.len() > 80 {
                    format!("{}...", &r.text[..80])
                } else {
                    r.text.clone()
                };

                ActivityPreview {
                    type_: r.type_.clone(),
                    agent: r.agent.clone(),
                    timestamp: r.timestamp.clone(),
                    preview,
                }
            })
            .collect();

        ContextSummary {
            total_items: search_results.len() + recent_knowledge.len(),
            by_type,
            by_agent,
            top_decisions,
            recent_activity,
        }
    }
}
```

**Test Cases:**
```rust
#[test]
fn test_generate_context_summary() {
    let km = KnowledgeManager::new(Default::default());

    let search_results = vec![
        SearchResult { type_: "decision".into(), agent: "architect".into(), text: "Decision 1".into(), ..Default::default() },
        SearchResult { type_: "implementation".into(), agent: "python".into(), text: "Impl 1".into(), ..Default::default() },
    ];

    let recent_knowledge = vec![
        KnowledgeEntry { type_: "issue".into(), agent: "qa".into(), text: "Issue 1".into(), ..Default::default() },
    ];

    let summary = km.generate_context_summary(&search_results, &recent_knowledge);

    assert_eq!(summary.total_items, 3);
    assert_eq!(summary.by_type.get("decision"), Some(&1));
    assert_eq!(summary.by_agent.get("architect"), Some(&1));
    assert_eq!(summary.top_decisions.len(), 1);
}
```

---

### 14. cleanup(options)

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 425-459 | `async fn cleanup(&self, older_than_days: u64, project_id: Option<&str>) -> Result<CleanupResult>` | ❌ Not implemented | Delete old entries |
| Date calculation | `cutoffDate = now - N days` | ❌ | chrono Duration |
| Retrieval | Search with dummy vector | ❌ | Workaround |
| Filtering | Filter by timestamp + project_id | ❌ | Compare dates |
| **NOTE** | LanceDB doesn't support delete | ⚠️ | **NOT IMPLEMENTED IN NODE.JS** |

**Node.js Implementation (Incomplete):**
```javascript
async cleanup(options = {}) {
  const { olderThanDays = 90, project_id = null } = options;

  const cutoffDate = new Date();
  cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

  const dummyVector = Array(384).fill(0);
  const allRecords = await this.table.search(dummyVector).limit(1000).execute();

  let toDelete = allRecords.filter(record => {
    const recordDate = new Date(record.timestamp);
    return recordDate < cutoffDate;
  });

  if (project_id) {
    toDelete = toDelete.filter(r => r.project_id === project_id);
  }

  // Note: LanceDB doesn't have built-in delete, would need to recreate table
  console.log(`⚠️  Found ${toDelete.length} old records (cleanup not yet implemented)`);

  return { count: toDelete.length };
}
```

**Expected Rust (Same Limitation):**
```rust
#[derive(Debug, Serialize)]
pub struct CleanupResult {
    pub count: usize,
    pub note: String,
}

impl KnowledgeManager {
    pub async fn cleanup(
        &self,
        older_than_days: u64,
        project_id: Option<&str>
    ) -> Result<CleanupResult, KnowledgeError> {
        println!("🧹 Cleaning up knowledge older than {} days...", older_than_days);

        let cutoff_date = Utc::now() - Duration::days(older_than_days as i64);

        // Retrieve all records (workaround)
        let dummy_vector = vec![0.0_f32; self.embedding_dim];
        let all_records = self.table
            .search(dummy_vector)
            .limit(1000)
            .execute()
            .await?;

        // Filter by date
        let mut to_delete: Vec<_> = all_records
            .into_iter()
            .filter(|r| {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(&r.timestamp) {
                    timestamp.with_timezone(&Utc) < cutoff_date
                } else {
                    false
                }
            })
            .collect();

        // Optional project filter
        if let Some(pid) = project_id {
            to_delete.retain(|r| r.project_id == pid);
        }

        let count = to_delete.len();

        eprintln!("⚠️  Found {} old records (cleanup not yet implemented)", count);

        // TODO: LanceDB doesn't support delete - need to recreate table
        // For now, just return the count

        Ok(CleanupResult {
            count,
            note: "Cleanup not yet implemented - LanceDB limitation".into(),
        })
    }
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_cleanup_identifies_old_records() {
    let mut km = setup_test_km().await;

    // Store old record (manually set timestamp)
    // (This requires a test helper to backdatecords)

    let result = km.cleanup(90, None).await.unwrap();

    // Should identify old records (but not delete yet)
    assert!(result.note.contains("not yet implemented"));
}
```

---

### 15. getStats()

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 464-505 | `async fn get_stats(&self) -> Result<KnowledgeStats>` | ❌ Not implemented | Database statistics |
| Retrieval | Search with dummy vector | ❌ | Workaround |
| Count by type | HashMap aggregation | ❌ | Iterate and count |
| Count by agent | HashMap aggregation | ❌ | Iterate and count |
| Count by project | HashMap aggregation | ❌ | Iterate and count |
| Oldest/newest | Track min/max timestamps | ❌ | Compare timestamps |

**Node.js Implementation:**
```javascript
async getStats() {
  const dummyVector = Array(384).fill(0);
  const allRecords = await this.table.search(dummyVector).limit(1000).execute();

  const stats = {
    repository: this.repoName,
    totalRecords: allRecords.length,
    byType: {},
    byAgent: {},
    byProject: {},
    oldestRecord: null,
    newestRecord: null
  };

  allRecords.forEach(record => {
    stats.byType[record.type] = (stats.byType[record.type] || 0) + 1;
    stats.byAgent[record.agent] = (stats.byAgent[record.agent] || 0) + 1;
    stats.byProject[record.project_id] = (stats.byProject[record.project_id] || 0) + 1;

    const timestamp = new Date(record.timestamp);
    if (!stats.oldestRecord || timestamp < new Date(stats.oldestRecord)) {
      stats.oldestRecord = record.timestamp;
    }
    if (!stats.newestRecord || timestamp > new Date(stats.newestRecord)) {
      stats.newestRecord = record.timestamp;
    }
  });

  return stats;
}
```

**Expected Rust:**
```rust
#[derive(Debug, Serialize)]
pub struct KnowledgeStats {
    pub repository: String,
    pub total_records: usize,
    pub by_type: HashMap<String, usize>,
    pub by_agent: HashMap<String, usize>,
    pub by_project: HashMap<String, usize>,
    pub oldest_record: Option<String>,
    pub newest_record: Option<String>,
}

impl KnowledgeManager {
    pub async fn get_stats(&self) -> Result<KnowledgeStats, KnowledgeError> {
        // Retrieve all records
        let dummy_vector = vec![0.0_f32; self.embedding_dim];
        let all_records = self.table
            .search(dummy_vector)
            .limit(1000)
            .execute()
            .await?;

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_agent: HashMap<String, usize> = HashMap::new();
        let mut by_project: HashMap<String, usize> = HashMap::new();
        let mut oldest_record: Option<String> = None;
        let mut newest_record: Option<String> = None;

        for record in &all_records {
            // Count by type
            *by_type.entry(record.type_.clone()).or_insert(0) += 1;

            // Count by agent
            *by_agent.entry(record.agent.clone()).or_insert(0) += 1;

            // Count by project
            *by_project.entry(record.project_id.clone()).or_insert(0) += 1;

            // Track oldest/newest
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(&record.timestamp) {
                let ts_str = record.timestamp.clone();

                if oldest_record.is_none() || timestamp < DateTime::parse_from_rfc3339(oldest_record.as_ref().unwrap()).unwrap() {
                    oldest_record = Some(ts_str.clone());
                }

                if newest_record.is_none() || timestamp > DateTime::parse_from_rfc3339(newest_record.as_ref().unwrap()).unwrap() {
                    newest_record = Some(ts_str);
                }
            }
        }

        Ok(KnowledgeStats {
            repository: self.repo_name.clone(),
            total_records: all_records.len(),
            by_type,
            by_agent,
            by_project,
            oldest_record,
            newest_record,
        })
    }
}
```

**Test Cases:**
```rust
#[tokio::test]
async fn test_get_stats() {
    let mut km = setup_test_km_with_data().await;

    let stats = km.get_stats().await.unwrap();

    assert!(stats.total_records > 0);
    assert!(stats.by_type.len() > 0);
    assert!(stats.by_agent.len() > 0);
    assert!(stats.oldest_record.is_some());
    assert!(stats.newest_record.is_some());
}

#[tokio::test]
async fn test_stats_aggregation() {
    let mut km = setup_test_km().await;

    // Store specific test data
    km.store(StoreRequest { type_: "decision".into(), agent: "architect".into(), ..Default::default() }).await.unwrap();
    km.store(StoreRequest { type_: "decision".into(), agent: "qa".into(), ..Default::default() }).await.unwrap();
    km.store(StoreRequest { type_: "implementation".into(), agent: "architect".into(), ..Default::default() }).await.unwrap();

    let stats = km.get_stats().await.unwrap();

    assert_eq!(stats.by_type.get("decision"), Some(&2));
    assert_eq!(stats.by_type.get("implementation"), Some(&1));
    assert_eq!(stats.by_agent.get("architect"), Some(&2));
    assert_eq!(stats.by_agent.get("qa"), Some(&1));
}
```

---

## CLI Interface

| Node.js | Rust Equivalent | Status | Critical Details |
|---------|-----------------|--------|------------------|
| Lines 517-634 | `cco/bin/knowledge-manager` (binary) | ❌ Not implemented | CLI commands |
| `store <text> [type]` | Same | ❌ | Store from command line |
| `search <query> [limit]` | Same | ❌ | Search from command line |
| `stats` | Same | ❌ | Show statistics |
| `test` | Same | ❌ | Run test suite |

**Expected Rust Binary:**
```rust
// cco/bin/knowledge-manager.rs

use cco::knowledge::KnowledgeManager;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "knowledge-manager")]
#[command(about = "Knowledge Manager - LanceDB Integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store knowledge
    Store {
        /// Text to store
        text: String,
        /// Knowledge type (decision, implementation, etc.)
        #[arg(default_value = "general")]
        type_: String,
    },
    /// Search knowledge
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(default_value = "10")]
        limit: usize,
    },
    /// Show statistics
    Stats,
    /// Run test suite
    Test,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut km = KnowledgeManager::new(Default::default());
    km.initialize().await?;

    match cli.command {
        Commands::Store { text, type_ } => {
            let id = km.store(StoreRequest {
                text,
                type_,
                agent: "cli".into(),
                ..Default::default()
            }).await?;
            println!("Stored with ID: {}", id);
        }
        Commands::Search { query, limit } => {
            let results = km.search(SearchQuery {
                q: query,
                limit,
                ..Default::default()
            }).await?;
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        Commands::Stats => {
            let stats = km.get_stats().await?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        Commands::Test => {
            run_test_suite(&mut km).await?;
        }
    }

    km.close().await?;
    Ok(())
}
```

---

## Summary

### Implementation Checklist

| Method | Lines | Priority | Complexity | Est. Hours |
|--------|-------|----------|------------|------------|
| 1. Constructor/new | 16-32 | HIGH | Low | 2 |
| 2. getRepoName | 37-40 | HIGH | Low | 0.5 |
| 3. initialize | 45-68 | HIGH | Medium | 3 |
| 4. createTable | 73-90 | HIGH | Medium | 2 |
| 5. generateEmbedding | 96-108 | **CRITICAL** | Medium | 4 |
| 6. store | 113-153 | HIGH | Medium | 3 |
| 7. storeBatch | 158-172 | MEDIUM | Low | 1 |
| 8. search | 177-225 | HIGH | High | 4 |
| 9. getProjectKnowledge | 230-274 | MEDIUM | Medium | 3 |
| 10. preCompaction | 279-292 | HIGH | Low | 2 |
| 11. postCompaction | 297-326 | HIGH | Medium | 3 |
| 12. extractCriticalKnowledge | 331-380 | MEDIUM | High | 4 |
| 13. generateContextSummary | 385-420 | LOW | Medium | 2 |
| 14. cleanup | 425-459 | LOW | Medium | 2 |
| 15. getStats | 464-505 | MEDIUM | Medium | 2 |
| CLI Interface | 517-634 | MEDIUM | Low | 2 |
| **TOTAL** | **634 lines** | | | **~40 hours** |

### Critical Path

**Week 1 (Foundation):**
1. Constructor + getRepoName + initialize (5.5 hours)
2. createTable + generateEmbedding (6 hours) **← CRITICAL**
3. store + storeBatch (4 hours)

**Week 2 (Search & Hooks):**
4. search + getProjectKnowledge (7 hours)
5. preCompaction + postCompaction (5 hours)
6. extractCriticalKnowledge (4 hours)

**Week 3 (Utilities & CLI):**
7. generateContextSummary + cleanup + getStats (6 hours)
8. CLI interface (2 hours)
9. Testing & debugging (remaining time)

---

## Test Coverage Requirements

**Minimum:**
- Unit tests for each method
- Integration tests for workflows
- Edge case tests for embeddings
- Backward compatibility tests

**Target: >90% code coverage**

---

**QA Engineer Sign-Off:**

This matrix will be used to verify 100% logic migration. Every method must have:
1. Rust equivalent implemented
2. Unit tests passing
3. Backward compatibility verified
4. Integration tests passing

**Status:** Ready for implementation to begin.

**Next Steps:**
1. Chief Architect approves embedding strategy
2. TDD Agent writes tests for each method
3. Rust Specialist implements to pass tests
4. QA Engineer verifies with this matrix

---

**END OF LOGIC MIGRATION MATRIX**
