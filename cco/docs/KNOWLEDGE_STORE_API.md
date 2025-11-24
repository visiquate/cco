# Knowledge Store HTTP API Reference

**Version:** 1.0.0
**Base URL:** `http://localhost:8303`
**Status:** Implementation Phase
**Last Updated:** November 18, 2025

---

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Endpoints](#endpoints)
4. [Data Models](#data-models)
5. [Error Handling](#error-handling)
6. [Rate Limits](#rate-limits)
7. [Examples](#examples)

---

## Overview

The Knowledge Store API provides vector-based knowledge management for the CCO daemon. It replaces the previous Node.js subprocess implementation with an embedded HTTP API powered by LanceDB vector database.

### What is the Knowledge Store?

The Knowledge Store enables agents to:
- **Store knowledge items** with semantic embeddings
- **Search by similarity** using vector search
- **Query by metadata** using SQL-like filters
- **Track statistics** across projects and agents
- **Cleanup old data** to manage storage

### Key Features

- **Vector Similarity Search**: Find semantically related knowledge
- **Per-Project Isolation**: Knowledge scoped to individual projects
- **Type Classification**: Organize by decision, architecture, implementation, etc.
- **Agent Attribution**: Track which agent created each item
- **Session Tracking**: Group knowledge by agent sessions
- **Metadata Storage**: Store arbitrary JSON metadata

### Architecture

```
Agent (HTTP Client)
    │
    │ POST /api/knowledge/store
    │ GET  /api/knowledge/search
    ▼
┌─────────────────────────────┐
│   CCO Daemon (Port 8303)    │
│                             │
│   ┌─────────────────────┐   │
│   │ HTTP API (Axum)     │   │
│   └──────────┬──────────┘   │
│              │              │
│   ┌──────────┼──────────┐   │
│   │ KnowledgeManager    │   │
│   │ - VectorStore       │   │
│   │ - Embeddings        │   │
│   │ - SearchEngine      │   │
│   └──────────┬──────────┘   │
│              │              │
└──────────────┼──────────────┘
               │
               ▼
    LanceDB (Embedded)
    ~/.cco/knowledge/
```

---

## Authentication

All endpoints require authentication via Bearer token.

### Token Location

```bash
# Token stored at:
~/.cco/api_token
```

### Request Header

```http
Authorization: Bearer <token>
```

### Obtaining a Token

The token is automatically generated when the CCO daemon starts:

```bash
# Start daemon
cco daemon start

# Token is written to ~/.cco/api_token
cat ~/.cco/api_token
```

### Example

```bash
TOKEN=$(cat ~/.cco/api_token)

curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $TOKEN"
```

### Security Notes

- Tokens are randomly generated on daemon startup
- Tokens are valid until daemon restarts
- Store tokens securely (file permissions: 600)
- Never commit tokens to version control

---

## Endpoints

### 1. Store Knowledge

Store a single knowledge item with semantic embedding.

**Endpoint:** `POST /api/knowledge/store`

**Request Body:**

```json
{
  "text": "String (required) - Knowledge content, any length",
  "type": "String (required) - One of: decision, architecture, implementation, configuration, credential, issue, general",
  "project_id": "String (required) - Project/repo identifier",
  "session_id": "String (required) - Agent session identifier",
  "agent": "String (required) - Agent name",
  "metadata": "Object (optional) - Additional context as JSON"
}
```

**Response (201 Created):**

```json
{
  "status": "success",
  "id": "decision-1699564213456-abc123",
  "timestamp": "2025-11-18T10:30:00Z"
}
```

**Status Codes:**
- `201 Created` - Knowledge stored successfully
- `400 Bad Request` - Missing required fields or invalid type
- `401 Unauthorized` - Missing or invalid auth token
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Decision: Use LanceDB for vector storage because it provides fast similarity search, SQL support, and Arrow-native format.",
    "type": "decision",
    "project_id": "cc-orchestra",
    "session_id": "session-123",
    "agent": "architect",
    "metadata": {
      "confidence": 0.95,
      "tags": ["vectordb", "architecture"]
    }
  }'
```

**Response:**

```json
{
  "status": "success",
  "id": "decision-1700321400123-4f8a3b2c",
  "timestamp": "2025-11-18T15:30:00Z"
}
```

---

### 2. Search Knowledge (Vector Similarity)

Search for knowledge using semantic similarity.

**Endpoint:** `GET /api/knowledge/search`

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `q` | String | Yes | - | Search query text |
| `project_id` | String | Yes | - | Project filter |
| `limit` | Integer | No | 10 | Max results (max: 100) |
| `threshold` | Float | No | 0.5 | Similarity threshold (0.0-1.0) |
| `type` | String | No | - | Filter by knowledge type |
| `agent` | String | No | - | Filter by agent name |
| `start_date` | ISO8601 | No | - | Filter by date range (start) |
| `end_date` | ISO8601 | No | - | Filter by date range (end) |

**Response (200 OK):**

```json
{
  "results": [
    {
      "id": "String",
      "text": "String",
      "type": "String",
      "project_id": "String",
      "agent": "String",
      "timestamp": "ISO8601",
      "similarity": 0.95,
      "metadata": {}
    }
  ],
  "count": 5,
  "query_time_ms": 15
}
```

**Status Codes:**
- `200 OK` - Search successful
- `400 Bad Request` - Invalid query parameters
- `401 Unauthorized` - Missing or invalid auth token
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl "http://localhost:8303/api/knowledge/search?q=vector+database+decisions&project_id=cc-orchestra&type=decision&limit=5" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

**Response:**

```json
{
  "results": [
    {
      "id": "decision-1700321400123-4f8a3b2c",
      "text": "Decision: Use LanceDB for vector storage...",
      "type": "decision",
      "project_id": "cc-orchestra",
      "agent": "architect",
      "timestamp": "2025-11-18T15:30:00Z",
      "similarity": 0.95,
      "metadata": {
        "confidence": 0.95,
        "tags": ["vectordb", "architecture"]
      }
    },
    {
      "id": "decision-1700320000456-7a2b1c3d",
      "text": "Decision: Embed LanceDB in daemon for performance...",
      "type": "decision",
      "project_id": "cc-orchestra",
      "agent": "architect",
      "timestamp": "2025-11-18T15:00:00Z",
      "similarity": 0.87,
      "metadata": {
        "tags": ["performance", "embedding"]
      }
    }
  ],
  "count": 2,
  "query_time_ms": 18
}
```

---

### 3. Batch Store

Store multiple knowledge items in a single request.

**Endpoint:** `POST /api/knowledge/store/batch`

**Request Body:**

```json
{
  "items": [
    {
      "text": "String",
      "type": "String",
      "project_id": "String",
      "session_id": "String",
      "agent": "String",
      "metadata": {}
    }
  ]
}
```

**Response (201 Created):**

```json
{
  "status": "success",
  "stored": 10,
  "failed": 0,
  "ids": ["id1", "id2", "..."]
}
```

**Status Codes:**
- `201 Created` - Batch stored successfully
- `207 Multi-Status` - Partial success (some items failed)
- `400 Bad Request` - Invalid request format
- `401 Unauthorized` - Missing or invalid auth token

**Example:**

```bash
curl -X POST http://localhost:8303/api/knowledge/store/batch \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {
        "text": "Implementation: Added vector search endpoint",
        "type": "implementation",
        "project_id": "cc-orchestra",
        "session_id": "session-123",
        "agent": "rust-specialist"
      },
      {
        "text": "Implementation: Added embedding cache",
        "type": "implementation",
        "project_id": "cc-orchestra",
        "session_id": "session-123",
        "agent": "rust-specialist"
      }
    ]
  }'
```

---

### 4. Query Knowledge (SQL-like)

Query knowledge using structured filters.

**Endpoint:** `POST /api/knowledge/query`

**Request Body:**

```json
{
  "project_id": "String (required)",
  "type": "String (optional)",
  "agent": "String (optional)",
  "session_id": "String (optional)",
  "limit": 20,
  "offset": 0,
  "order_by": "timestamp DESC"
}
```

**Response (200 OK):**

```json
{
  "results": [
    {
      "id": "String",
      "text": "String",
      "type": "String",
      "project_id": "String",
      "agent": "String",
      "session_id": "String",
      "timestamp": "ISO8601",
      "metadata": {}
    }
  ],
  "total": 42,
  "query_time_ms": 8
}
```

**Example:**

```bash
curl -X POST http://localhost:8303/api/knowledge/query \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "cc-orchestra",
    "type": "decision",
    "agent": "architect",
    "limit": 10,
    "order_by": "timestamp DESC"
  }'
```

---

### 5. Get Statistics

Retrieve knowledge store statistics.

**Endpoint:** `GET /api/knowledge/stats`

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | String | No | Filter by project |

**Response (200 OK):**

```json
{
  "total_entries": 1523,
  "by_type": {
    "decision": 245,
    "architecture": 120,
    "implementation": 678,
    "configuration": 89,
    "credential": 12,
    "issue": 123,
    "general": 256
  },
  "by_agent": {
    "architect": 234,
    "python-specialist": 567,
    "qa-engineer": 234,
    "security-auditor": 156
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

**Example:**

```bash
# All projects
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"

# Specific project
curl "http://localhost:8303/api/knowledge/stats?project_id=cc-orchestra" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

---

### 6. Get Entry by ID

Retrieve a specific knowledge entry.

**Endpoint:** `GET /api/knowledge/{id}`

**Response (200 OK):**

```json
{
  "id": "decision-1700321400123-4f8a3b2c",
  "text": "Decision: Use LanceDB for vector storage...",
  "type": "decision",
  "project_id": "cc-orchestra",
  "agent": "architect",
  "session_id": "session-123",
  "timestamp": "2025-11-18T15:30:00Z",
  "metadata": {
    "confidence": 0.95
  }
}
```

**Status Codes:**
- `200 OK` - Entry found
- `404 Not Found` - Entry does not exist
- `401 Unauthorized` - Missing or invalid auth token

**Example:**

```bash
curl http://localhost:8303/api/knowledge/decision-1700321400123-4f8a3b2c \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

---

### 7. Cleanup Old Entries

Delete knowledge entries older than specified days.

**Endpoint:** `DELETE /api/knowledge/cleanup`

**Request Body:**

```json
{
  "older_than_days": 90,
  "project_id": "cc-orchestra",
  "dry_run": false
}
```

**Response (200 OK):**

```json
{
  "status": "success",
  "deleted_count": 45,
  "freed_space_mb": 2.3,
  "dry_run": false
}
```

**Status Codes:**
- `200 OK` - Cleanup successful
- `400 Bad Request` - Invalid parameters
- `401 Unauthorized` - Missing or invalid auth token

**Example:**

```bash
# Dry run first
curl -X DELETE http://localhost:8303/api/knowledge/cleanup \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "older_than_days": 90,
    "project_id": "cc-orchestra",
    "dry_run": true
  }'

# Actual cleanup
curl -X DELETE http://localhost:8303/api/knowledge/cleanup \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "older_than_days": 90,
    "project_id": "cc-orchestra",
    "dry_run": false
  }'
```

---

### 8. Health Check

Check if the knowledge store is operational.

**Endpoint:** `GET /api/knowledge/health`

**Response (200 OK):**

```json
{
  "status": "healthy",
  "database": "connected",
  "embedding_model": "loaded",
  "uptime_seconds": 3600
}
```

**Status Codes:**
- `200 OK` - Service healthy
- `503 Service Unavailable` - Service degraded or unavailable

**Example:**

```bash
curl http://localhost:8303/api/knowledge/health \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

---

## Data Models

### KnowledgeItem

```json
{
  "id": "String - Unique identifier (auto-generated)",
  "text": "String - Knowledge content (required)",
  "type": "String - Knowledge type (required)",
  "project_id": "String - Project identifier (required)",
  "session_id": "String - Agent session ID (required)",
  "agent": "String - Agent name (required)",
  "timestamp": "ISO8601 - Creation time (auto-generated)",
  "metadata": "Object - Additional context (optional)"
}
```

### KnowledgeType Enum

Valid values for the `type` field:

| Type | Description | Example Use Case |
|------|-------------|------------------|
| `decision` | Architectural decisions | "Decided to use FastAPI" |
| `architecture` | System design patterns | "Microservices architecture chosen" |
| `implementation` | Code implementation notes | "Implemented JWT authentication" |
| `configuration` | Configuration settings | "Database pool size: 10" |
| `credential` | Credential references | "API key stored in vault" |
| `issue` | Problems and bugs | "Memory leak in worker thread" |
| `general` | Uncategorized knowledge | "User feedback noted" |

### SearchRequest

```json
{
  "query": "String - Search text (required)",
  "project_id": "String - Project filter (required)",
  "limit": "Integer - Max results (default: 10, max: 100)",
  "threshold": "Float - Similarity threshold (default: 0.5)",
  "type": "String - Filter by type (optional)",
  "agent": "String - Filter by agent (optional)",
  "start_date": "ISO8601 - Date range start (optional)",
  "end_date": "ISO8601 - Date range end (optional)"
}
```

### SearchResponse

```json
{
  "results": [
    {
      "id": "String",
      "text": "String",
      "type": "String",
      "project_id": "String",
      "agent": "String",
      "timestamp": "ISO8601",
      "similarity": "Float (0.0-1.0)",
      "metadata": "Object"
    }
  ],
  "count": "Integer - Number of results returned",
  "query_time_ms": "Integer - Search performance in milliseconds"
}
```

### Metadata Structure

The `metadata` field is a JSON object with arbitrary key-value pairs:

```json
{
  "metadata": {
    "confidence": 0.95,
    "tags": ["api", "security"],
    "priority": "high",
    "related_issues": ["#123", "#456"],
    "custom_field": "custom_value"
  }
}
```

**Recommendations:**
- Use consistent keys across similar entries
- Keep values simple (strings, numbers, arrays)
- Avoid deeply nested objects
- Use tags array for categorization
- Include confidence scores for decisions

---

## Error Handling

### Error Response Format

All errors return a consistent JSON structure:

```json
{
  "error": "String - Human-readable error message",
  "code": "String - Error code for programmatic handling",
  "details": "Object - Additional error context (optional)"
}
```

### Error Codes

| HTTP Status | Error Code | Description |
|-------------|-----------|-------------|
| 400 | `INVALID_REQUEST` | Missing or invalid fields |
| 400 | `INVALID_TYPE` | Unknown knowledge type |
| 400 | `INVALID_QUERY` | Malformed search query |
| 401 | `UNAUTHORIZED` | Missing or invalid auth token |
| 404 | `NOT_FOUND` | Entry does not exist |
| 429 | `RATE_LIMIT_EXCEEDED` | Too many requests |
| 500 | `DATABASE_ERROR` | Database operation failed |
| 500 | `EMBEDDING_ERROR` | Embedding generation failed |
| 503 | `SERVICE_UNAVAILABLE` | Service temporarily unavailable |

### Example Error Responses

**400 Bad Request:**

```json
{
  "error": "Invalid knowledge type",
  "code": "INVALID_TYPE",
  "details": {
    "provided": "unknown_type",
    "valid_types": ["decision", "architecture", "implementation", "configuration", "credential", "issue", "general"]
  }
}
```

**401 Unauthorized:**

```json
{
  "error": "Missing authorization token",
  "code": "UNAUTHORIZED"
}
```

**404 Not Found:**

```json
{
  "error": "Knowledge entry not found",
  "code": "NOT_FOUND",
  "details": {
    "id": "decision-1700321400123-nonexistent"
  }
}
```

**500 Internal Server Error:**

```json
{
  "error": "Database operation failed",
  "code": "DATABASE_ERROR",
  "details": {
    "operation": "insert",
    "message": "Connection timeout"
  }
}
```

---

## Rate Limits

### Current Limits

| Limit | Value |
|-------|-------|
| Requests per minute | 100 |
| Requests per hour | 1000 |
| Concurrent connections | 50 |

### Rate Limit Headers

Responses include rate limit information:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1700321460
```

### Rate Limit Exceeded Response

```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "details": {
    "retry_after": 42
  }
}
```

**HTTP Status:** `429 Too Many Requests`

**Headers:**

```http
Retry-After: 42
```

---

## Examples

### Complete Python Client

```python
import requests
from typing import Dict, List, Optional

class KnowledgeStoreClient:
    def __init__(self, base_url: str = "http://localhost:8303", token_path: str = "~/.cco/api_token"):
        self.base_url = base_url
        with open(os.path.expanduser(token_path)) as f:
            self.token = f.read().strip()
        self.headers = {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }

    def store(
        self,
        text: str,
        type: str,
        project_id: str,
        session_id: str,
        agent: str,
        metadata: Optional[Dict] = None
    ) -> Dict:
        """Store a knowledge item."""
        response = requests.post(
            f"{self.base_url}/api/knowledge/store",
            headers=self.headers,
            json={
                "text": text,
                "type": type,
                "project_id": project_id,
                "session_id": session_id,
                "agent": agent,
                "metadata": metadata or {}
            }
        )
        response.raise_for_status()
        return response.json()

    def search(
        self,
        query: str,
        project_id: str,
        type: Optional[str] = None,
        limit: int = 10,
        threshold: float = 0.5
    ) -> Dict:
        """Search for similar knowledge."""
        params = {
            "q": query,
            "project_id": project_id,
            "limit": limit,
            "threshold": threshold
        }
        if type:
            params["type"] = type

        response = requests.get(
            f"{self.base_url}/api/knowledge/search",
            headers=self.headers,
            params=params
        )
        response.raise_for_status()
        return response.json()

    def query(
        self,
        project_id: str,
        type: Optional[str] = None,
        agent: Optional[str] = None,
        limit: int = 20
    ) -> Dict:
        """Query knowledge with filters."""
        body = {
            "project_id": project_id,
            "limit": limit
        }
        if type:
            body["type"] = type
        if agent:
            body["agent"] = agent

        response = requests.post(
            f"{self.base_url}/api/knowledge/query",
            headers=self.headers,
            json=body
        )
        response.raise_for_status()
        return response.json()

    def stats(self, project_id: Optional[str] = None) -> Dict:
        """Get knowledge store statistics."""
        params = {}
        if project_id:
            params["project_id"] = project_id

        response = requests.get(
            f"{self.base_url}/api/knowledge/stats",
            headers=self.headers,
            params=params
        )
        response.raise_for_status()
        return response.json()

# Usage example
client = KnowledgeStoreClient()

# Store knowledge
result = client.store(
    text="Decided to use Rust for daemon implementation",
    type="decision",
    project_id="cc-orchestra",
    session_id="session-abc123",
    agent="architect",
    metadata={"confidence": 0.95, "tags": ["rust", "daemon"]}
)
print(f"Stored: {result['id']}")

# Search knowledge
results = client.search(
    query="daemon implementation decisions",
    project_id="cc-orchestra",
    type="decision",
    limit=5
)
for item in results['results']:
    print(f"{item['agent']}: {item['text'][:50]}... (similarity: {item['similarity']:.2f})")

# Get statistics
stats = client.stats(project_id="cc-orchestra")
print(f"Total entries: {stats['total_entries']}")
print(f"By type: {stats['by_type']}")
```

### Bash Script Example

```bash
#!/bin/bash
# knowledge-cli.sh - Command-line interface for knowledge store

BASE_URL="http://localhost:8303"
TOKEN=$(cat ~/.cco/api_token)

# Store knowledge
store_knowledge() {
    local text="$1"
    local type="${2:-general}"
    local agent="${3:-cli}"
    local project_id=$(basename $(pwd))

    curl -X POST "$BASE_URL/api/knowledge/store" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"text\": \"$text\",
            \"type\": \"$type\",
            \"project_id\": \"$project_id\",
            \"session_id\": \"$(date +%s)\",
            \"agent\": \"$agent\"
        }" | jq .
}

# Search knowledge
search_knowledge() {
    local query="$1"
    local limit="${2:-10}"
    local project_id=$(basename $(pwd))

    curl "$BASE_URL/api/knowledge/search" \
        -H "Authorization: Bearer $TOKEN" \
        -G \
        --data-urlencode "q=$query" \
        --data-urlencode "project_id=$project_id" \
        --data-urlencode "limit=$limit" | jq .
}

# Get stats
get_stats() {
    local project_id="${1:-$(basename $(pwd))}"

    curl "$BASE_URL/api/knowledge/stats?project_id=$project_id" \
        -H "Authorization: Bearer $TOKEN" | jq .
}

# Main command dispatcher
case "$1" in
    store)
        store_knowledge "$2" "$3" "$4"
        ;;
    search)
        search_knowledge "$2" "$3"
        ;;
    stats)
        get_stats "$2"
        ;;
    *)
        echo "Usage: $0 {store|search|stats} [args...]"
        exit 1
        ;;
esac
```

### Rust Client Example

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct StoreRequest {
    text: String,
    #[serde(rename = "type")]
    knowledge_type: String,
    project_id: String,
    session_id: String,
    agent: String,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct StoreResponse {
    status: String,
    id: String,
    timestamp: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    results: Vec<KnowledgeItem>,
    count: usize,
    query_time_ms: u64,
}

#[derive(Deserialize)]
struct KnowledgeItem {
    id: String,
    text: String,
    #[serde(rename = "type")]
    knowledge_type: String,
    agent: String,
    similarity: f32,
}

struct KnowledgeClient {
    client: Client,
    base_url: String,
    token: String,
}

impl KnowledgeClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut token_path = dirs::home_dir().unwrap_or_default();
        token_path.push(".cco");
        token_path.push("api_token");

        let token = fs::read_to_string(token_path)?;

        Ok(Self {
            client: Client::new(),
            base_url: "http://localhost:8303".to_string(),
            token: token.trim().to_string(),
        })
    }

    pub async fn store(
        &self,
        text: String,
        knowledge_type: String,
        project_id: String,
        session_id: String,
        agent: String,
    ) -> Result<StoreResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .post(format!("{}/api/knowledge/store", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&StoreRequest {
                text,
                knowledge_type,
                project_id,
                session_id,
                agent,
                metadata: HashMap::new(),
            })
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn search(
        &self,
        query: String,
        project_id: String,
        limit: usize,
    ) -> Result<SearchResponse, Box<dyn std::error::Error>> {
        let response = self.client
            .get(format!("{}/api/knowledge/search", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[
                ("q", query.as_str()),
                ("project_id", project_id.as_str()),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KnowledgeClient::new()?;

    // Store knowledge
    let result = client.store(
        "Implemented vector search with LanceDB".to_string(),
        "implementation".to_string(),
        "cc-orchestra".to_string(),
        "session-123".to_string(),
        "rust-specialist".to_string(),
    ).await?;

    println!("Stored: {}", result.id);

    // Search knowledge
    let results = client.search(
        "vector search implementation".to_string(),
        "cc-orchestra".to_string(),
        5,
    ).await?;

    for item in results.results {
        println!("{}: {} (similarity: {:.2})", item.agent, item.text, item.similarity);
    }

    Ok(())
}
```

---

## Performance Tips

### Optimize Search Queries

1. **Use specific queries** - More specific text yields better results
2. **Set appropriate thresholds** - Higher threshold (0.7-0.9) for precise matches
3. **Limit results** - Only request what you need
4. **Filter by project** - Always include project_id for faster queries

### Batch Operations

```bash
# Instead of multiple single stores:
for text in "${texts[@]}"; do
    store_knowledge "$text"
done

# Use batch store:
curl -X POST "$BASE_URL/api/knowledge/store/batch" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{"items": [...]}'
```

### Connection Reuse

```python
# Reuse HTTP session for multiple requests
session = requests.Session()
session.headers.update({"Authorization": f"Bearer {token}"})

# All requests use the same connection pool
session.post(url, json=data)
session.get(url, params=params)
```

---

## Troubleshooting

### Common Issues

**1. 401 Unauthorized**

```bash
# Check token exists
ls -la ~/.cco/api_token

# Verify token content
cat ~/.cco/api_token

# Restart daemon if token missing
cco daemon restart
```

**2. Connection Refused**

```bash
# Check daemon is running
cco daemon status

# Check port
lsof -i :8303

# Start daemon if not running
cco daemon start
```

**3. Slow Search Performance**

```bash
# Check database size
curl "$BASE_URL/api/knowledge/stats" -H "Authorization: Bearer $TOKEN" | jq '.database_size_mb'

# Cleanup old entries
curl -X DELETE "$BASE_URL/api/knowledge/cleanup" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{"older_than_days": 90}'
```

**4. Invalid Type Error**

```bash
# Use valid types only
valid_types=("decision" "architecture" "implementation" "configuration" "credential" "issue" "general")

# Check your type value
echo "$type" | grep -E "^(decision|architecture|implementation|configuration|credential|issue|general)$"
```

---

## See Also

- [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md) - How to use from agents
- [Migration Guide](KNOWLEDGE_STORE_MIGRATION.md) - Migrate from Node.js version
- [Quick Start](KNOWLEDGE_STORE_QUICK_START.md) - Get started in 5 minutes
- [Architecture](KNOWLEDGE_STORE_ARCHITECTURE.md) - How it works internally
- [LanceDB Documentation](https://lancedb.github.io/lancedb/) - Underlying database

---

**Last Updated:** November 18, 2025
**API Version:** 1.0.0
**Maintained by:** CCO Team
