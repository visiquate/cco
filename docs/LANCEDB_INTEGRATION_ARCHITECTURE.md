# LanceDB Integration Architecture Design

**Date:** November 18, 2025
**Project:** CCO Daemon Embedded VectorDB
**Status:** Architecture Specification

---

## Table of Contents

1. [System Architecture](#1-system-architecture)
2. [Module Structure](#2-module-structure)
3. [API Specification](#3-api-specification)
4. [Data Model](#4-data-model)
5. [Agent Communication Flow](#5-agent-communication-flow)
6. [Error Handling](#6-error-handling)
7. [Concurrency Patterns](#7-concurrency-patterns)
8. [Code Architecture](#8-code-architecture)

---

## 1. System Architecture

### 1.1 Current Architecture (Node.js Separate Process)

```
┌─────────────────────────────────────────────────────┐
│                 Agent Ecosystem                     │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ Agent A  │  │ Agent B  │  │ Agent C  │         │
│  │ (Python) │  │  (QA)    │  │(Security)│         │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘         │
│       │             │             │                │
│       └─────────────┼─────────────┘                │
│                     │                              │
└─────────────────────┼──────────────────────────────┘
                      │
                      │ subprocess spawn
                      │ node knowledge-manager.js
                      ▼
        ┌─────────────────────────────┐
        │  knowledge-manager.js       │
        │  (Node.js Process)          │
        │                             │
        │  ├── CLI Interface          │
        │  ├── LanceDB (vectordb)     │
        │  ├── Simple Embedding       │
        │  └── Local Storage          │
        └──────────────┬──────────────┘
                       │
                       ▼
        ┌─────────────────────────────┐
        │  LanceDB Storage            │
        │  ~/data/knowledge/{repo}/   │
        │  (Arrow/Parquet files)      │
        └─────────────────────────────┘

Issues:
❌ Process spawn overhead (50-200ms)
❌ Separate Node.js dependency
❌ No concurrent access control
❌ No authentication
❌ Limited to CLI interface
```

### 1.2 Proposed Architecture (Rust Embedded)

```
┌─────────────────────────────────────────────────────┐
│                 Agent Ecosystem                     │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ Agent A  │  │ Agent B  │  │ Agent C  │         │
│  │ (Python) │  │  (QA)    │  │(Security)│         │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘         │
│       │             │             │                │
│       └─────────────┼─────────────┘                │
│                     │                              │
└─────────────────────┼──────────────────────────────┘
                      │
                      │ HTTP/JSON API
                      │ POST /api/knowledge/store
                      │ GET  /api/knowledge/search
                      ▼
        ┌─────────────────────────────────────────┐
        │      CCO Daemon (Single Rust Binary)    │
        │                                         │
        │  ┌─────────────────────────────────┐   │
        │  │   HTTP Server (Axum)            │   │
        │  │   - Port 3000                   │   │
        │  │   - Auth middleware             │   │
        │  │   - Rate limiting               │   │
        │  └──────────────┬──────────────────┘   │
        │                 │                       │
        │  ┌──────────────┼──────────────────┐   │
        │  │  Knowledge Service (NEW)        │   │
        │  │                                 │   │
        │  │  ├── KnowledgeManager           │   │
        │  │  ├── VectorStore (LanceDB)      │   │
        │  │  ├── EmbeddingGenerator         │   │
        │  │  └── SearchEngine               │   │
        │  └──────────────┬──────────────────┘   │
        │                 │                       │
        │  ┌──────────────┼──────────────────┐   │
        │  │  Existing Systems               │   │
        │  │                                 │   │
        │  │  ├── Hooks System               │   │
        │  │  ├── CRUD Classifier            │   │
        │  │  ├── TinyLLaMA Model (600MB)    │   │
        │  │  ├── Audit DB (SQLite)          │   │
        │  │  └── Cache (Moka)               │   │
        │  └─────────────────────────────────┘   │
        └─────────────────┬───────────────────────┘
                          │
                          ▼
        ┌─────────────────────────────────────────┐
        │  LanceDB Storage (Embedded)             │
        │  ~/.cco/knowledge/{repo}/               │
        │  (Arrow/Parquet files)                  │
        └─────────────────────────────────────────┘

Benefits:
✅ Single binary distribution
✅ No process spawn overhead
✅ Concurrent access with Tokio
✅ Built-in authentication
✅ HTTP API (future: gRPC)
✅ Type-safe Rust implementation
```

### 1.3 System Components

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **HTTP Server** | Axum | REST API endpoints |
| **Knowledge Manager** | Rust | Business logic layer |
| **Vector Store** | LanceDB (Rust) | Vector similarity search |
| **Embedding Generator** | sentence-transformers (via bindings) | Text → vector embeddings |
| **Search Engine** | LanceDB + SQL | Hybrid vector + metadata search |
| **Auth Middleware** | Axum middleware | Token-based authentication |
| **Persistence** | LanceDB (Arrow/Parquet) | Durable storage |

---

## 2. Module Structure

### 2.1 File Organization

```
cco/src/
├── knowledge/
│   ├── mod.rs              # Module exports and public API
│   ├── manager.rs          # KnowledgeManager (orchestration)
│   ├── store.rs            # VectorStore (LanceDB wrapper)
│   ├── embedding.rs        # EmbeddingGenerator (text → vectors)
│   ├── search.rs           # SearchEngine (query execution)
│   ├── models.rs           # Data models (KnowledgeEntry, etc.)
│   ├── api.rs              # HTTP API handlers
│   └── error.rs            # Error types
│
├── daemon/
│   ├── mod.rs
│   ├── server.rs           # Add knowledge routes
│   └── ...
│
└── main.rs                 # Initialize knowledge system
```

### 2.2 Module Responsibilities

**`knowledge/mod.rs`**
- Public API exports
- Module initialization
- Configuration management

**`knowledge/manager.rs`**
- `KnowledgeManager` struct
- High-level operations (store, search, query)
- Coordinates between store, embedding, and search

**`knowledge/store.rs`**
- `VectorStore` struct
- LanceDB connection management
- CRUD operations on vector database
- Schema management

**`knowledge/embedding.rs`**
- `EmbeddingGenerator` struct
- Text → vector transformation
- Model loading and inference
- Caching embeddings

**`knowledge/search.rs`**
- `SearchEngine` struct
- Vector similarity search
- Metadata filtering
- Hybrid search (vector + text)

**`knowledge/models.rs`**
- `KnowledgeEntry` - Main data model
- `SearchQuery` - Search parameters
- `SearchResult` - Search response
- `KnowledgeType` - Enum for knowledge types

**`knowledge/api.rs`**
- HTTP request handlers
- Request validation
- Response formatting
- Error mapping

**`knowledge/error.rs`**
- `KnowledgeError` - Custom error type
- Error conversion (LanceDB → KnowledgeError)
- Error responses

---

## 3. API Specification

### 3.1 REST Endpoints

#### 3.1.1 Store Knowledge

**Endpoint:** `POST /api/knowledge/store`

**Request:**
```json
{
  "text": "We decided to use FastAPI for the REST API because...",
  "knowledge_type": "decision",
  "project_id": "cc-orchestra",
  "session_id": "session-12345",
  "agent": "architect",
  "metadata": {
    "tags": ["api", "framework"],
    "confidence": 0.95
  }
}
```

**Response:**
```json
{
  "status": "success",
  "id": "decision-1699564213456-abc123",
  "timestamp": "2025-11-18T10:30:00Z"
}
```

**Status Codes:**
- `201 Created` - Knowledge stored successfully
- `400 Bad Request` - Invalid request format
- `401 Unauthorized` - Missing or invalid auth token
- `500 Internal Server Error` - Database error

#### 3.1.2 Search Knowledge (Vector Similarity)

**Endpoint:** `GET /api/knowledge/search`

**Query Parameters:**
- `q` (required) - Search query text
- `limit` (optional) - Max results (default: 10, max: 100)
- `threshold` (optional) - Similarity threshold (default: 0.5)
- `project_id` (optional) - Filter by project
- `knowledge_type` (optional) - Filter by type
- `agent` (optional) - Filter by agent

**Example:**
```
GET /api/knowledge/search?q=authentication&limit=5&project_id=cc-orchestra
```

**Response:**
```json
{
  "results": [
    {
      "id": "decision-1699564213456-abc123",
      "text": "We decided to use JWT for authentication...",
      "knowledge_type": "decision",
      "project_id": "cc-orchestra",
      "agent": "architect",
      "timestamp": "2025-11-18T10:30:00Z",
      "similarity_score": 0.92,
      "metadata": {
        "tags": ["auth", "security"]
      }
    }
  ],
  "total": 1,
  "query_time_ms": 15
}
```

**Status Codes:**
- `200 OK` - Search successful
- `400 Bad Request` - Invalid query parameters
- `401 Unauthorized` - Missing or invalid auth token
- `500 Internal Server Error` - Database error

#### 3.1.3 Query Knowledge (SQL)

**Endpoint:** `POST /api/knowledge/query`

**Request:**
```json
{
  "project_id": "cc-orchestra",
  "knowledge_type": "decision",
  "agent": "architect",
  "limit": 20,
  "order_by": "timestamp DESC"
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "decision-1699564213456-abc123",
      "text": "We decided to use JWT for authentication...",
      "knowledge_type": "decision",
      "project_id": "cc-orchestra",
      "agent": "architect",
      "timestamp": "2025-11-18T10:30:00Z"
    }
  ],
  "total": 15,
  "query_time_ms": 8
}
```

#### 3.1.4 Get Statistics

**Endpoint:** `GET /api/knowledge/stats`

**Query Parameters:**
- `project_id` (optional) - Filter by project

**Response:**
```json
{
  "total_entries": 1523,
  "by_type": {
    "decision": 245,
    "implementation": 678,
    "issue": 123,
    "pattern": 89,
    "general": 388
  },
  "by_agent": {
    "architect": 234,
    "python": 567,
    "qa": 234,
    "security": 156
  },
  "by_project": {
    "cc-orchestra": 1023,
    "other-project": 500
  },
  "oldest_entry": "2025-01-15T08:00:00Z",
  "newest_entry": "2025-11-18T10:30:00Z",
  "database_size_mb": 12.5
}
```

#### 3.1.5 Delete Old Entries

**Endpoint:** `DELETE /api/knowledge/cleanup`

**Request:**
```json
{
  "older_than_days": 90,
  "project_id": "cc-orchestra"  // optional
}
```

**Response:**
```json
{
  "status": "success",
  "deleted_count": 45,
  "freed_space_mb": 2.3
}
```

### 3.2 Authentication

**Method:** Bearer Token (JWT or API Key)

**Header:**
```
Authorization: Bearer <token>
```

**Token Generation:**
- Issued by daemon on startup
- Stored in `~/.cco/api_token`
- Rotated every 24 hours (optional)

**Implementation:**
```rust
// Axum middleware
async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    match token {
        Some(t) if validate_token(t) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

### 3.3 Rate Limiting

**Limits:**
- 100 requests/minute per client
- 1000 requests/hour per client

**Implementation:** Tower rate limiting middleware

---

## 4. Data Model

### 4.1 KnowledgeEntry

**Rust Definition:**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Unique identifier (generated)
    pub id: String,

    /// 384-dimensional embedding vector
    pub vector: Vec<f32>,

    /// Raw text content
    pub text: String,

    /// Type of knowledge
    pub knowledge_type: KnowledgeType,

    /// Project/repository identifier
    pub project_id: String,

    /// Session identifier
    pub session_id: String,

    /// Agent that created this entry
    pub agent: String,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Additional structured metadata
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeType {
    Architecture,
    Decision,
    Implementation,
    Configuration,
    Credential,
    Issue,
    Pattern,
    General,
}

impl KnowledgeEntry {
    /// Generate a unique ID
    pub fn generate_id(knowledge_type: KnowledgeType) -> String {
        format!(
            "{}-{}-{}",
            knowledge_type.as_str(),
            Utc::now().timestamp_millis(),
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        )
    }
}

impl KnowledgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Architecture => "architecture",
            Self::Decision => "decision",
            Self::Implementation => "implementation",
            Self::Configuration => "configuration",
            Self::Credential => "credential",
            Self::Issue => "issue",
            Self::Pattern => "pattern",
            Self::General => "general",
        }
    }
}
```

### 4.2 SearchQuery

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    /// Search text
    pub q: String,

    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Similarity threshold (0.0-1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f32,

    /// Filter by project
    pub project_id: Option<String>,

    /// Filter by knowledge type
    pub knowledge_type: Option<KnowledgeType>,

    /// Filter by agent
    pub agent: Option<String>,
}

fn default_limit() -> usize { 10 }
fn default_threshold() -> f32 { 0.5 }
```

### 4.3 SearchResult

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Knowledge entry
    #[serde(flatten)]
    pub entry: KnowledgeEntry,

    /// Similarity score (0.0-1.0)
    pub similarity_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    /// Matching results
    pub results: Vec<SearchResult>,

    /// Total count (before limit)
    pub total: usize,

    /// Query execution time (ms)
    pub query_time_ms: u64,
}
```

### 4.4 Arrow Schema

**LanceDB Table Schema:**
```rust
use arrow_schema::{Schema, Field, DataType};
use std::sync::Arc;

pub fn knowledge_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("text", DataType::Utf8, false),
        Field::new("knowledge_type", DataType::Utf8, false),
        Field::new("project_id", DataType::Utf8, false),
        Field::new("session_id", DataType::Utf8, false),
        Field::new("agent", DataType::Utf8, false),
        Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, None), false),
        Field::new("metadata", DataType::Utf8, false), // JSON string
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                384 // embedding dimension
            ),
            false
        ),
    ]))
}
```

---

## 5. Agent Communication Flow

### 5.1 Store Knowledge Flow

```
┌─────────┐
│ Agent A │
└────┬────┘
     │
     │ 1. Generate knowledge text
     │    "We decided to use FastAPI..."
     │
     │ 2. HTTP POST /api/knowledge/store
     │    {
     │      "text": "...",
     │      "knowledge_type": "decision",
     │      "agent": "architect"
     │    }
     ▼
┌──────────────────────────────┐
│   CCO Daemon                 │
│                              │
│  ┌────────────────────────┐  │
│  │ HTTP Handler           │  │
│  │ (knowledge/api.rs)     │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 3. Validate request
│             │ 4. Call KnowledgeManager
│             ▼                │
│  ┌────────────────────────┐  │
│  │ KnowledgeManager       │  │
│  │ (knowledge/manager.rs) │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 5. Generate embedding
│             ▼                │
│  ┌────────────────────────┐  │
│  │ EmbeddingGenerator     │  │
│  │ (knowledge/embedding)  │  │
│  │                        │  │
│  │ text → [0.1, 0.2, ...] │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 6. Create entry
│             ▼                │
│  ┌────────────────────────┐  │
│  │ VectorStore            │  │
│  │ (knowledge/store.rs)   │  │
│  │                        │  │
│  │ ┌────────────────────┐ │  │
│  │ │ LanceDB            │ │  │
│  │ │ table.add(batch)   │ │  │
│  │ └────────────────────┘ │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 7. Persist to disk
│             ▼                │
│       Arrow/Parquet files    │
│       ~/.cco/knowledge/      │
│                              │
│             │ 8. Return success
│             ▼                │
│  ┌────────────────────────┐  │
│  │ HTTP Response          │  │
│  │ { "status": "success", │  │
│  │   "id": "..." }        │  │
│  └────────────────────────┘  │
└──────────────┬───────────────┘
               │
               │ 9. Response
               ▼
          ┌─────────┐
          │ Agent A │
          └─────────┘
```

### 5.2 Search Knowledge Flow

```
┌─────────┐
│ Agent B │
└────┬────┘
     │
     │ 1. Need to find info about auth
     │
     │ 2. HTTP GET /api/knowledge/search
     │    ?q=authentication&limit=5
     ▼
┌──────────────────────────────┐
│   CCO Daemon                 │
│                              │
│  ┌────────────────────────┐  │
│  │ HTTP Handler           │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 3. Parse query params
│             ▼                │
│  ┌────────────────────────┐  │
│  │ SearchEngine           │  │
│  │ (knowledge/search.rs)  │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 4. Generate query vector
│             ▼                │
│  ┌────────────────────────┐  │
│  │ EmbeddingGenerator     │  │
│  │ "authentication"       │  │
│  │   → [0.15, 0.23, ...]  │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 5. Vector similarity search
│             ▼                │
│  ┌────────────────────────┐  │
│  │ VectorStore            │  │
│  │                        │  │
│  │ ┌────────────────────┐ │  │
│  │ │ LanceDB            │ │  │
│  │ │ vector_search()    │ │  │
│  │ │ + metadata filter  │ │  │
│  │ └────────────────────┘ │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 6. Rank by similarity
│             │ 7. Apply filters
│             │ 8. Limit results
│             ▼                │
│  ┌────────────────────────┐  │
│  │ SearchResults          │  │
│  │ [                      │  │
│  │   {text: "...", score} │  │
│  │ ]                      │  │
│  └──────────┬─────────────┘  │
│             │                │
│             │ 9. Format response
│             ▼                │
│  ┌────────────────────────┐  │
│  │ HTTP Response          │  │
│  │ {                      │  │
│  │   "results": [...],    │  │
│  │   "total": 5,          │  │
│  │   "query_time_ms": 15  │  │
│  │ }                      │  │
│  └────────────────────────┘  │
└──────────────┬───────────────┘
               │
               │ 10. Results
               ▼
          ┌─────────┐
          │ Agent B │
          └─────────┘
```

### 5.3 Agent CLI Wrapper (Backward Compatibility)

**Option 1: HTTP CLI Wrapper**
```bash
#!/bin/bash
# ~/.claude/bin/knowledge-manager (replaces Node.js version)

DAEMON_URL="http://127.0.0.1:3000"
TOKEN=$(cat ~/.cco/api_token)

case "$1" in
  store)
    TEXT="$2"
    TYPE="${3:-general}"
    AGENT="${4:-cli}"

    curl -s -X POST "$DAEMON_URL/api/knowledge/store" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"text\": \"$TEXT\",
        \"knowledge_type\": \"$TYPE\",
        \"agent\": \"$AGENT\",
        \"project_id\": \"$(basename $(pwd))\"
      }" | jq .
    ;;

  search)
    QUERY="$2"
    LIMIT="${3:-10}"

    curl -s "$DAEMON_URL/api/knowledge/search?q=$QUERY&limit=$LIMIT" \
      -H "Authorization: Bearer $TOKEN" | jq .
    ;;

  stats)
    curl -s "$DAEMON_URL/api/knowledge/stats" \
      -H "Authorization: Bearer $TOKEN" | jq .
    ;;

  *)
    echo "Usage: knowledge-manager {store|search|stats}"
    exit 1
    ;;
esac
```

**Option 2: Direct HTTP Calls in Agent Instructions**
```markdown
# Agent Instructions (e.g., architect.md)

## Knowledge Management

To store knowledge:
```bash
curl -X POST http://127.0.0.1:3000/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Architecture decision: ...",
    "knowledge_type": "decision",
    "agent": "architect",
    "project_id": "current-project"
  }'
```

To search knowledge:
```bash
curl "http://127.0.0.1:3000/api/knowledge/search?q=authentication&limit=5" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```
```

---

## 6. Error Handling

### 6.1 Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KnowledgeError {
    #[error("LanceDB error: {0}")]
    Database(#[from] lancedb::Error),

    #[error("Embedding generation failed: {0}")]
    Embedding(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Entry not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for KnowledgeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            KnowledgeError::InvalidQuery(msg) => (StatusCode::BAD_REQUEST, msg),
            KnowledgeError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### 6.2 Error Recovery

**Strategy:**
- Retry transient failures (3 attempts with exponential backoff)
- Log all errors with context
- Return user-friendly error messages
- Fallback to degraded mode (e.g., skip embedding if model unavailable)

**Example:**
```rust
async fn store_with_retry(
    entry: KnowledgeEntry,
    max_retries: usize,
) -> Result<String, KnowledgeError> {
    let mut attempts = 0;
    let mut backoff = Duration::from_millis(100);

    loop {
        match self.store.insert(entry.clone()).await {
            Ok(id) => return Ok(id),
            Err(e) if attempts < max_retries => {
                warn!("Store failed (attempt {}): {}", attempts + 1, e);
                tokio::time::sleep(backoff).await;
                backoff *= 2;
                attempts += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 7. Concurrency Patterns

### 7.1 Thread Safety

**LanceDB Connection:**
- LanceDB connections are thread-safe (Arc-wrapped)
- Single connection shared across all requests
- Connection pooling handled internally by LanceDB

**Shared State:**
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct KnowledgeManager {
    store: Arc<VectorStore>,
    embedding: Arc<EmbeddingGenerator>,
    search: Arc<SearchEngine>,
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>, // Embedding cache
}

impl Clone for KnowledgeManager {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            embedding: Arc::clone(&self.embedding),
            search: Arc::clone(&self.search),
            cache: Arc::clone(&self.cache),
        }
    }
}
```

### 7.2 Async/Await Patterns

**All I/O operations are async:**
```rust
impl KnowledgeManager {
    pub async fn store(&self, req: StoreRequest) -> Result<String, KnowledgeError> {
        // 1. Generate embedding (async)
        let vector = self.embedding.generate(&req.text).await?;

        // 2. Create entry
        let entry = KnowledgeEntry {
            id: KnowledgeEntry::generate_id(req.knowledge_type),
            vector,
            text: req.text,
            knowledge_type: req.knowledge_type,
            project_id: req.project_id,
            session_id: req.session_id,
            agent: req.agent,
            timestamp: Utc::now(),
            metadata: req.metadata,
        };

        // 3. Insert (async)
        self.store.insert(entry).await
    }

    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse, KnowledgeError> {
        let start = std::time::Instant::now();

        // 1. Generate query vector (async)
        let query_vector = self.embedding.generate(&query.q).await?;

        // 2. Execute search (async)
        let results = self.search.vector_search(query_vector, query).await?;

        let query_time_ms = start.elapsed().as_millis() as u64;

        Ok(SearchResponse {
            results,
            total: results.len(),
            query_time_ms,
        })
    }
}
```

### 7.3 Caching Strategy

**Embedding Cache:**
- Cache embeddings for frequently searched queries
- LRU eviction policy
- Cache size: 1000 entries
- TTL: 1 hour

```rust
use lru::LruCache;

pub struct EmbeddingGenerator {
    model: Arc<SentenceTransformer>,
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
}

impl EmbeddingGenerator {
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
}
```

---

## 8. Code Architecture

### 8.1 Service Traits

```rust
use async_trait::async_trait;

#[async_trait]
pub trait VectorDatabase: Send + Sync {
    async fn insert(&self, entry: KnowledgeEntry) -> Result<String, KnowledgeError>;
    async fn insert_batch(&self, entries: Vec<KnowledgeEntry>) -> Result<Vec<String>, KnowledgeError>;
    async fn vector_search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, KnowledgeError>;
    async fn query(&self, filter: QueryFilter) -> Result<Vec<KnowledgeEntry>, KnowledgeError>;
    async fn delete(&self, id: &str) -> Result<(), KnowledgeError>;
    async fn delete_old(&self, days: u64) -> Result<usize, KnowledgeError>;
    async fn stats(&self) -> Result<DatabaseStats, KnowledgeError>;
}

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn encode(&self, text: &str) -> Result<Vec<f32>, KnowledgeError>;
    async fn encode_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, KnowledgeError>;
    fn dimension(&self) -> usize;
}
```

### 8.2 Initialization

```rust
// cco/src/lib.rs

pub mod knowledge;

// cco/src/knowledge/mod.rs

mod manager;
mod store;
mod embedding;
mod search;
mod models;
mod api;
mod error;

pub use manager::KnowledgeManager;
pub use models::{KnowledgeEntry, KnowledgeType, SearchQuery, SearchResponse};
pub use api::knowledge_routes;
pub use error::KnowledgeError;

use std::path::PathBuf;

pub async fn initialize_knowledge_system(
    db_path: PathBuf,
) -> Result<KnowledgeManager, KnowledgeError> {
    // 1. Initialize vector store
    let store = VectorStore::new(db_path).await?;

    // 2. Initialize embedding generator
    let embedding = EmbeddingGenerator::new().await?;

    // 3. Initialize search engine
    let search = SearchEngine::new(store.clone(), embedding.clone());

    // 4. Create manager
    Ok(KnowledgeManager::new(store, embedding, search))
}
```

### 8.3 Daemon Integration

```rust
// cco/src/daemon/server.rs

use crate::knowledge::{initialize_knowledge_system, knowledge_routes, KnowledgeManager};

pub struct DaemonState {
    // ... existing fields ...
    pub knowledge_manager: Arc<KnowledgeManager>,
}

impl DaemonState {
    pub async fn new(config: DaemonConfig) -> anyhow::Result<Self> {
        // ... existing initialization ...

        // Initialize knowledge system
        let knowledge_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cco")
            .join("knowledge");

        let knowledge_manager = Arc::new(
            initialize_knowledge_system(knowledge_path).await?
        );

        Ok(Self {
            // ... existing fields ...
            knowledge_manager,
        })
    }
}

pub async fn run_daemon_server(config: DaemonConfig) -> anyhow::Result<()> {
    // ... existing code ...

    let app = Router::new()
        // Existing routes
        .route("/health", get(health_check))
        .route("/api/classify", post(classify_command))
        // ... other routes ...

        // NEW: Knowledge routes
        .nest("/api/knowledge", knowledge_routes())

        .with_state(state.clone());

    // ... rest of server setup ...
}
```

### 8.4 Example Code Snippets

**Complete store handler:**
```rust
// cco/src/knowledge/api.rs

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct StoreRequest {
    pub text: String,
    pub knowledge_type: KnowledgeType,
    pub project_id: String,
    pub session_id: String,
    pub agent: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Serialize)]
pub struct StoreResponse {
    pub status: String,
    pub id: String,
    pub timestamp: String,
}

pub async fn store_knowledge(
    State(manager): State<Arc<KnowledgeManager>>,
    Json(req): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, KnowledgeError> {
    let id = manager.store(req).await?;

    Ok(Json(StoreResponse {
        status: "success".to_string(),
        id,
        timestamp: Utc::now().to_rfc3339(),
    }))
}

pub fn knowledge_routes() -> Router<Arc<DaemonState>> {
    Router::new()
        .route("/store", post(store_knowledge))
        .route("/search", get(search_knowledge))
        .route("/query", post(query_knowledge))
        .route("/stats", get(get_statistics))
        .route("/cleanup", delete(cleanup_old))
        .layer(auth_middleware)
}
```

---

## Appendix: Migration Checklist

### Pre-Migration

- [ ] Backup existing Node.js knowledge database
- [ ] Document current agent usage patterns
- [ ] Test LanceDB Rust SDK with sample data
- [ ] Benchmark performance (Node.js baseline)

### Implementation

- [ ] Add `lancedb` dependency to Cargo.toml
- [ ] Create `cco/src/knowledge/` module structure
- [ ] Implement data models and error types
- [ ] Implement VectorStore (LanceDB wrapper)
- [ ] Implement EmbeddingGenerator (proper model, not hash)
- [ ] Implement SearchEngine
- [ ] Implement KnowledgeManager (orchestration)
- [ ] Add HTTP API endpoints
- [ ] Add authentication middleware
- [ ] Integration with DaemonState

### Testing

- [ ] Unit tests for each module
- [ ] Integration tests for API endpoints
- [ ] Migration test (Node.js data → Rust)
- [ ] Performance benchmarks
- [ ] Load testing (concurrent requests)

### Deployment

- [ ] Create migration script
- [ ] Update agent instructions
- [ ] Deploy updated daemon
- [ ] Monitor for errors
- [ ] Validate data integrity
- [ ] Deprecate Node.js knowledge-manager

---

**Architecture Complete** - Ready for implementation.
