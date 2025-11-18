# Knowledge Store Architecture

**Technical Documentation**
**Version:** 1.0.0
**Last Updated:** November 18, 2025

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Components](#architecture-components)
3. [Data Flow](#data-flow)
4. [Database Design](#database-design)
5. [Embedding Strategy](#embedding-strategy)
6. [Per-Project Isolation](#per-project-isolation)
7. [Concurrency Model](#concurrency-model)
8. [Performance Characteristics](#performance-characteristics)

---

## System Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│              Agent Ecosystem                        │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ Agent A  │  │ Agent B  │  │ Agent C  │         │
│  │ (Python) │  │  (Rust)  │  │  (Bash)  │         │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘         │
│       │             │             │                │
│       └─────────────┼─────────────┘                │
│                     │                              │
└─────────────────────┼──────────────────────────────┘
                      │ HTTP/JSON
                      │ POST /api/knowledge/store
                      │ GET  /api/knowledge/search
                      ▼
        ┌─────────────────────────────────────────┐
        │   CCO Daemon (Single Rust Binary)       │
        │   Port: 8303                            │
        │                                         │
        │  ┌─────────────────────────────────┐   │
        │  │   HTTP Server (Axum)            │   │
        │  │   - Authentication              │   │
        │  │   - Rate limiting               │   │
        │  │   - Request validation          │   │
        │  └──────────────┬──────────────────┘   │
        │                 │                       │
        │  ┌──────────────┴──────────────────┐   │
        │  │  Knowledge Manager              │   │
        │  │  (Orchestration Layer)          │   │
        │  │                                 │   │
        │  │  ┌──────────────────────────┐   │   │
        │  │  │  Vector Store            │   │   │
        │  │  │  - LanceDB wrapper       │   │   │
        │  │  │  - CRUD operations       │   │   │
        │  │  │  - Schema management     │   │   │
        │  │  └──────────────────────────┘   │   │
        │  │                                 │   │
        │  │  ┌──────────────────────────┐   │   │
        │  │  │  Embedding Generator     │   │   │
        │  │  │  - Text → Vector (384D)  │   │   │
        │  │  │  - LRU cache             │   │   │
        │  │  │  - Batch processing      │   │   │
        │  │  └──────────────────────────┘   │   │
        │  │                                 │   │
        │  │  ┌──────────────────────────┐   │   │
        │  │  │  Search Engine           │   │   │
        │  │  │  - Vector similarity     │   │   │
        │  │  │  - Metadata filtering    │   │   │
        │  │  │  - Ranking               │   │   │
        │  │  └──────────────────────────┘   │   │
        │  └─────────────────────────────────┘   │
        │                                         │
        └─────────────────┬───────────────────────┘
                          │
                          ▼
        ┌─────────────────────────────────────────┐
        │  LanceDB (Embedded Vector Database)     │
        │  Location: ~/.cco/knowledge/            │
        │                                         │
        │  ┌──────────────┐  ┌──────────────┐    │
        │  │ Project A    │  │ Project B    │    │
        │  │ (Arrow)      │  │ (Arrow)      │    │
        │  └──────────────┘  └──────────────┘    │
        │                                         │
        │  Storage Format: Apache Arrow/Parquet   │
        └─────────────────────────────────────────┘
```

### Design Principles

1. **Embedded**: No external dependencies, single binary
2. **Fast**: <10ms for most operations
3. **Concurrent**: Thousands of simultaneous requests
4. **Isolated**: Per-project knowledge separation
5. **Type-safe**: Rust guarantees memory and thread safety
6. **Observable**: Built-in metrics and health checks

---

## Architecture Components

### 1. HTTP Server (Axum)

**Responsibilities:**
- Accept HTTP requests on port 8303
- Authenticate requests via Bearer tokens
- Validate request payloads
- Route to appropriate handlers
- Format responses
- Handle errors gracefully

**Technology:** Axum web framework (Rust)

**Key Features:**
- Async I/O with Tokio runtime
- Zero-copy parsing where possible
- Connection pooling
- Request/response middleware

**Code Structure:**
```rust
// cco/src/knowledge/api.rs

use axum::{Router, routing::{get, post, delete}};

pub fn knowledge_routes() -> Router<Arc<DaemonState>> {
    Router::new()
        .route("/store", post(store_knowledge))
        .route("/store/batch", post(batch_store_knowledge))
        .route("/search", get(search_knowledge))
        .route("/query", post(query_knowledge))
        .route("/stats", get(get_statistics))
        .route("/health", get(health_check))
        .route("/cleanup", delete(cleanup_old))
        .route("/:id", get(get_by_id))
        .layer(auth_middleware())
}
```

### 2. Knowledge Manager

**Responsibilities:**
- Orchestrate between VectorStore, Embedding, and Search
- Implement business logic
- Handle retry logic
- Coordinate transactions
- Aggregate statistics

**Technology:** Rust struct with Arc-wrapped components

**Code Structure:**
```rust
// cco/src/knowledge/manager.rs

pub struct KnowledgeManager {
    store: Arc<VectorStore>,
    embedding: Arc<EmbeddingGenerator>,
    search: Arc<SearchEngine>,
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
}

impl KnowledgeManager {
    pub async fn store(&self, req: StoreRequest) -> Result<String, KnowledgeError> {
        // 1. Generate embedding
        let vector = self.embedding.generate(&req.text).await?;

        // 2. Create entry
        let entry = KnowledgeEntry { /* ... */ };

        // 3. Store in database
        self.store.insert(entry).await
    }

    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse, KnowledgeError> {
        // 1. Generate query embedding
        let vector = self.embedding.generate(&query.q).await?;

        // 2. Vector search
        let results = self.search.vector_search(vector, query).await?;

        Ok(SearchResponse { results, /* ... */ })
    }
}
```

### 3. Vector Store (LanceDB Wrapper)

**Responsibilities:**
- Manage LanceDB connection
- Define Arrow schema
- Insert entries (single and batch)
- Execute vector searches
- Apply filters
- Delete entries

**Technology:** LanceDB Rust SDK (0.22.3)

**Schema Definition:**
```rust
// cco/src/knowledge/store.rs

use arrow_schema::{Schema, Field, DataType};

pub fn knowledge_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("text", DataType::Utf8, false),
        Field::new("type", DataType::Utf8, false),
        Field::new("project_id", DataType::Utf8, false),
        Field::new("session_id", DataType::Utf8, false),
        Field::new("agent", DataType::Utf8, false),
        Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, None), false),
        Field::new("metadata", DataType::Utf8, false), // JSON string
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                384  // Embedding dimension
            ),
            false
        ),
    ]))
}
```

**Operations:**
```rust
impl VectorStore {
    pub async fn insert(&self, entry: KnowledgeEntry) -> Result<String, KnowledgeError> {
        // Convert to Arrow RecordBatch
        let batch = self.entry_to_batch(entry)?;

        // Insert into LanceDB
        self.table.add(batch).await?;

        Ok(entry.id)
    }

    pub async fn vector_search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: Vec<Filter>,
    ) -> Result<Vec<SearchResult>, KnowledgeError> {
        // Build LanceDB query
        let query = self.table
            .vector_search(query_vector)
            .limit(limit);

        // Apply filters
        let query = filters.iter().fold(query, |q, f| {
            q.filter(f.to_sql())
        });

        // Execute
        let results = query.execute().await?;

        Ok(results)
    }
}
```

### 4. Embedding Generator

**Responsibilities:**
- Load embedding model
- Convert text to vectors
- Cache embeddings
- Batch processing

**Technology:** Sentence-transformers model (via rust-bert or candle)

**Model:** `all-MiniLM-L6-v2` (384 dimensions)

**Implementation:**
```rust
// cco/src/knowledge/embedding.rs

pub struct EmbeddingGenerator {
    model: Arc<SentenceTransformer>,
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
    dimension: usize,
}

impl EmbeddingGenerator {
    pub async fn new() -> Result<Self, KnowledgeError> {
        // Load model from disk or download
        let model = SentenceTransformer::load("all-MiniLM-L6-v2")?;

        Ok(Self {
            model: Arc::new(model),
            cache: Arc::new(RwLock::new(LruCache::new(1000))),
            dimension: 384,
        })
    }

    pub async fn generate(&self, text: &str) -> Result<Vec<f32>, KnowledgeError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.peek(text) {
                return Ok(cached.clone());
            }
        }

        // Generate embedding
        let embedding = self.model.encode(text).await?;

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.put(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    pub async fn generate_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, KnowledgeError> {
        // Batch encoding for efficiency
        self.model.encode_batch(texts).await
    }
}
```

### 5. Search Engine

**Responsibilities:**
- Execute vector similarity searches
- Apply metadata filters
- Rank results by similarity
- Implement hybrid search (vector + SQL)

**Implementation:**
```rust
// cco/src/knowledge/search.rs

pub struct SearchEngine {
    store: Arc<VectorStore>,
    embedding: Arc<EmbeddingGenerator>,
}

impl SearchEngine {
    pub async fn vector_search(
        &self,
        query_vector: Vec<f32>,
        params: SearchQuery,
    ) -> Result<Vec<SearchResult>, KnowledgeError> {
        // Build filters
        let mut filters = vec![
            Filter::Eq("project_id", params.project_id),
        ];

        if let Some(type_) = params.knowledge_type {
            filters.push(Filter::Eq("type", type_.as_str()));
        }

        if let Some(agent) = params.agent {
            filters.push(Filter::Eq("agent", agent));
        }

        // Execute search
        let results = self.store
            .vector_search(query_vector, params.limit, filters)
            .await?;

        // Filter by threshold
        let filtered: Vec<_> = results
            .into_iter()
            .filter(|r| r.similarity >= params.threshold)
            .collect();

        Ok(filtered)
    }
}
```

---

## Data Flow

### Store Knowledge Flow

```
┌──────────┐
│  Agent   │
└─────┬────┘
      │ 1. HTTP POST /api/knowledge/store
      │    {"text": "...", "type": "decision", ...}
      ▼
┌────────────────────────────────────────┐
│  HTTP Handler (api.rs)                 │
│  - Validate request                    │
│  - Check authentication                │
└─────┬──────────────────────────────────┘
      │ 2. StoreRequest
      ▼
┌────────────────────────────────────────┐
│  KnowledgeManager                      │
│  - Orchestrate components              │
└─────┬──────────────────────────────────┘
      │ 3. Generate embedding
      ▼
┌────────────────────────────────────────┐
│  EmbeddingGenerator                    │
│  - Check cache                         │
│  - Encode text → 384D vector           │
└─────┬──────────────────────────────────┘
      │ 4. Vec<f32>
      ▼
┌────────────────────────────────────────┐
│  KnowledgeManager                      │
│  - Create KnowledgeEntry               │
│  - Add vector, metadata                │
└─────┬──────────────────────────────────┘
      │ 5. KnowledgeEntry
      ▼
┌────────────────────────────────────────┐
│  VectorStore                           │
│  - Convert to Arrow RecordBatch        │
│  - Insert into LanceDB table           │
└─────┬──────────────────────────────────┘
      │ 6. Write to disk
      ▼
┌────────────────────────────────────────┐
│  LanceDB Storage                       │
│  ~/.cco/knowledge/{project}/           │
│  - Write Arrow/Parquet files           │
└─────┬──────────────────────────────────┘
      │ 7. Success
      ▼
┌────────────────────────────────────────┐
│  HTTP Response                         │
│  {"status": "success", "id": "..."}    │
└────────────────────────────────────────┘
```

### Search Knowledge Flow

```
┌──────────┐
│  Agent   │
└─────┬────┘
      │ 1. HTTP GET /api/knowledge/search?q=...
      ▼
┌────────────────────────────────────────┐
│  HTTP Handler                          │
│  - Parse query params                  │
│  - Validate                            │
└─────┬──────────────────────────────────┘
      │ 2. SearchQuery
      ▼
┌────────────────────────────────────────┐
│  SearchEngine                          │
│  - Prepare query                       │
└─────┬──────────────────────────────────┘
      │ 3. Generate query embedding
      ▼
┌────────────────────────────────────────┐
│  EmbeddingGenerator                    │
│  - Encode query text                   │
└─────┬──────────────────────────────────┘
      │ 4. Query vector
      ▼
┌────────────────────────────────────────┐
│  VectorStore                           │
│  - Build LanceDB query                 │
│  - Apply filters (project, type, etc.) │
│  - Execute vector search               │
└─────┬──────────────────────────────────┘
      │ 5. Vector similarity search
      ▼
┌────────────────────────────────────────┐
│  LanceDB                               │
│  - Compute cosine similarity           │
│  - Rank by score                       │
│  - Return top K                        │
└─────┬──────────────────────────────────┘
      │ 6. Raw results
      ▼
┌────────────────────────────────────────┐
│  SearchEngine                          │
│  - Filter by threshold                 │
│  - Format results                      │
└─────┬──────────────────────────────────┘
      │ 7. SearchResponse
      ▼
┌────────────────────────────────────────┐
│  HTTP Response                         │
│  {"results": [...], "count": 5}        │
└────────────────────────────────────────┘
```

---

## Database Design

### Storage Format

**Format:** Apache Arrow with Parquet on-disk storage

**Why Arrow?**
- Columnar format (efficient for vector operations)
- Zero-copy deserialization
- Fast filtering and aggregation
- Standard format (portable across languages)

### Schema

```
KnowledgeEntry {
    id: String                    // Unique identifier
    vector: FixedSizeList<f32>   // 384-dimensional embedding
    text: String                  // Original text
    type: String                  // Knowledge type (enum as string)
    project_id: String            // Project identifier
    session_id: String            // Agent session ID
    agent: String                 // Agent name
    timestamp: Timestamp          // Creation time (milliseconds)
    metadata: String              // JSON-encoded metadata
}
```

### Indexes

LanceDB automatically creates indexes:

1. **Vector Index**: IVF-PQ (Inverted File with Product Quantization)
   - Fast approximate nearest neighbor search
   - Configurable accuracy vs speed tradeoff

2. **Scalar Indexes**: B-tree indexes on frequently filtered columns
   - `project_id` (for per-project queries)
   - `type` (for type-filtered searches)
   - `agent` (for agent-filtered queries)
   - `timestamp` (for time-range queries)

### File Organization

```
~/.cco/knowledge/
├── cc-orchestra/
│   ├── data.lance               # Main data file
│   ├── _versions/               # Version metadata
│   └── _indices/                # Vector and scalar indexes
├── other-project/
│   ├── data.lance
│   ├── _versions/
│   └── _indices/
└── ...
```

---

## Embedding Strategy

### Model Selection

**Chosen Model:** `all-MiniLM-L6-v2`

**Specifications:**
- Dimension: 384
- Size: ~80 MB
- Speed: ~30ms per encoding (CPU)
- Quality: State-of-the-art for semantic similarity

**Why This Model?**
- Good balance of size, speed, and quality
- Widely used and battle-tested
- Pre-trained on diverse corpus
- Works well for technical text

### SHA256-Based Fallback (Deprecated)

The previous Node.js implementation used SHA256 hash truncation:

```javascript
// Old approach (deprecated)
const hash = crypto.createHash('sha256').update(text).digest();
const vector = Array.from(hash.slice(0, 128)).map(b => b / 255);
```

**Why Changed?**
- Poor semantic similarity (hash-based is too brittle)
- No understanding of meaning
- Only works for exact/near-exact matches

**Migration:** Existing hash-based embeddings are re-generated using the new model.

### Caching Strategy

**LRU Cache:**
- Size: 1000 entries
- Key: Text content (hash of text)
- Value: 384D vector

**Benefits:**
- Avoid re-encoding frequently searched queries
- Reduce model inference overhead
- Thread-safe with RwLock

**Cache Hit Rate:** Expected 60-80% for typical agent usage

---

## Per-Project Isolation

### Design Goal

Each project's knowledge should be isolated:
- Searches don't leak across projects
- Statistics are project-specific
- Cleanup is per-project

### Implementation

**1. Directory Structure:**
```
~/.cco/knowledge/
├── project-a/    # Separate LanceDB database
├── project-b/    # Separate LanceDB database
└── project-c/    # Separate LanceDB database
```

**2. Query Filtering:**
```rust
// All queries include project_id filter
let results = store.search(query_vector)
    .filter("project_id = 'project-a'")  // Enforced at database level
    .execute()
    .await?;
```

**3. Statistics:**
```rust
// Stats aggregated per project
pub async fn get_stats(&self, project_id: Option<String>) -> Result<Stats> {
    match project_id {
        Some(id) => self.stats_for_project(id).await,
        None => self.stats_all_projects().await,
    }
}
```

### Benefits

- **Security**: Projects can't access each other's knowledge
- **Performance**: Smaller index sizes per project
- **Scalability**: Can shard across projects if needed
- **Cleanup**: Delete entire project without affecting others

---

## Concurrency Model

### Async/Await with Tokio

All I/O operations are asynchronous:

```rust
pub async fn store(&self, req: StoreRequest) -> Result<String> {
    // Non-blocking embedding generation
    let vector = self.embedding.generate(&req.text).await?;

    // Non-blocking database insert
    self.store.insert(entry).await?;

    Ok(id)
}
```

### Shared State

Components are wrapped in `Arc` (atomic reference counting):

```rust
pub struct KnowledgeManager {
    store: Arc<VectorStore>,          // Shared across requests
    embedding: Arc<EmbeddingGenerator>, // Shared across requests
    search: Arc<SearchEngine>,         // Shared across requests
    cache: Arc<RwLock<LruCache<...>>>, // Thread-safe cache
}
```

### Lock Strategy

**Read-Write Locks (RwLock):**
- Multiple concurrent readers
- Exclusive write access
- Used for embedding cache

```rust
// Read (shared)
let cache = self.cache.read().await;
if let Some(vector) = cache.get(text) { ... }

// Write (exclusive)
let mut cache = self.cache.write().await;
cache.put(text, vector);
```

### Connection Pooling

LanceDB connections are pooled internally:
- Single connection per database
- Thread-safe
- Handles concurrent queries efficiently

---

## Performance Characteristics

### Benchmarks (Expected)

| Operation | Time | Throughput |
|-----------|------|------------|
| Store (single) | <10ms | 100 ops/sec |
| Store (batch of 100) | <500ms | 200 items/sec |
| Search (10 results) | <15ms | 66 searches/sec |
| Search (100K entries) | <100ms | 10 searches/sec |
| Generate embedding | <30ms | 33 encodings/sec |
| Cache hit | <1ms | 1000 hits/sec |

### Scalability

**Single Instance:**
- 10K entries: <10ms search
- 100K entries: <50ms search
- 1M entries: <200ms search

**Multiple Projects:**
- Linear scaling (separate databases)
- No cross-project overhead

### Memory Usage

| Component | Memory |
|-----------|--------|
| Embedding model | ~80 MB |
| LanceDB index (10K entries) | ~20 MB |
| HTTP server | ~10 MB |
| Cache (1000 entries) | ~1.5 MB |
| **Total** | **~110 MB** |

### Optimization Opportunities

1. **GPU Acceleration**: Use CUDA for embedding generation (10x faster)
2. **Quantization**: Reduce vector precision (384 x f32 → 384 x i8)
3. **Approximate Search**: Trade accuracy for speed (IVF-PQ tuning)
4. **Batch Processing**: Process multiple requests together

---

## API Design Rationale

### Why HTTP Instead of gRPC?

**Chosen:** HTTP/JSON

**Reasons:**
1. **Simpler**: Works with curl, Postman, any HTTP client
2. **Debugging**: Easy to inspect and test
3. **Language Agnostic**: Python, Bash, Rust, JavaScript all have HTTP clients
4. **Tooling**: Mature ecosystem (proxies, load balancers, etc.)

**Future:** Can add gRPC alongside HTTP if needed

### Why REST Instead of GraphQL?

**Chosen:** REST

**Reasons:**
1. **Simple Operations**: CRUD doesn't need GraphQL complexity
2. **Performance**: Direct endpoint mapping is faster
3. **Caching**: HTTP caching works well with REST
4. **Familiarity**: More developers know REST

### Authentication Design

**Chosen:** Bearer token (file-based)

**Why Not:**
- OAuth2: Too complex for local daemon
- mTLS: Overkill for localhost
- API Key Header: Bearer is standard

**Token Generation:**
```rust
// On daemon startup
let token = generate_random_token(32);
fs::write("~/.cco/api_token", token)?;
```

**Token Validation:**
```rust
async fn auth_middleware(req: Request, next: Next) -> Result<Response> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    match token {
        Some(t) if validate_token(t) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

---

## See Also

- [API Reference](KNOWLEDGE_STORE_API.md) - Complete API documentation
- [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md) - How to use from agents
- [Migration Guide](KNOWLEDGE_STORE_MIGRATION.md) - Migrate from Node.js version
- [Quick Start](KNOWLEDGE_STORE_QUICK_START.md) - Get started in 5 minutes

---

**Last Updated:** November 18, 2025
**Version:** 1.0.0
**Maintained by:** CCO Team
