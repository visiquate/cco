# Knowledge Store Developer Guide

**Guide for developers implementing and maintaining the Knowledge Store**

**Version:** 1.0.0
**Last Updated:** November 28, 2025
**Audience:** Rust developers, system architects

---

## Table of Contents

1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [Adding Knowledge Items](#adding-knowledge-items)
4. [Searching Knowledge](#searching-knowledge)
5. [Integrating with Agents](#integrating-with-agents)
6. [Code Examples](#code-examples)
7. [Testing Guide](#testing-guide)
8. [Debugging Tips](#debugging-tips)
9. [Contributing Changes](#contributing-changes)
10. [Performance Tuning](#performance-tuning)

---

## Overview

### Purpose

This guide helps developers understand and work with the Knowledge Store codebase, including:
- How to extend functionality
- How to add new features
- How to test changes
- How to debug issues

### Prerequisites

- Rust 1.70+ installed
- Understanding of Tokio async/await
- Familiarity with HTTP APIs
- Knowledge of vector databases (helpful but not required)

### Key Files

| File | Lines | Purpose |
|------|-------|---------|
| `src/daemon/knowledge/mod.rs` | 131 | Module definition & re-exports |
| `src/daemon/knowledge/models.rs` | 260 | Data structures (16 models) |
| `src/daemon/knowledge/store.rs` | 499 | Core business logic |
| `src/daemon/knowledge/api.rs` | 330 | HTTP endpoints (8 handlers) |
| `src/daemon/knowledge/embedding.rs` | 110 | Vector generation |

---

## Project Structure

### Module Organization

```
src/daemon/knowledge/
├── mod.rs
│   └── Re-exports public API
│
├── models.rs
│   ├── KnowledgeItem (internal representation)
│   ├── StoreKnowledgeRequest (API input)
│   ├── SearchRequest (API input)
│   ├── SearchResult (API output)
│   ├── PreCompactionRequest/Response
│   ├── PostCompactionRequest/Response
│   ├── StatsResponse
│   └── Various enums and types
│
├── store.rs
│   ├── KnowledgeStore struct
│   ├── Core methods (store, search, etc.)
│   ├── Helper methods (extract_critical_knowledge, etc.)
│   ├── Utility functions (cosine_similarity)
│   └── Tests
│
├── api.rs
│   ├── HTTP route handlers
│   ├── ApiError implementation
│   ├── Request validation
│   ├── Response formatting
│   └── Tests
│
└── embedding.rs
    ├── generate_embedding() function
    ├── Deterministic vector generation
    └── Tests
```

### Dependencies

**In Cargo.toml:**

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
chrono = "0.4"
uuid = { version = "1.0", features = ["v4"] }
regex = "1.9"
anyhow = "1.0"
tracing = "0.1"
dirs = "5.0"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

---

## Adding Knowledge Items

### Adding to Store

**Method Signature:**
```rust
pub async fn store(&mut self, request: StoreKnowledgeRequest) -> Result<StoreKnowledgeResponse>
```

**Process:**

1. **Validate Input**
   ```rust
   if request.text.is_empty() {
       anyhow::bail!("Knowledge text is required");
   }
   ```

2. **Generate Embedding**
   ```rust
   let vector = generate_embedding(&request.text);
   ```

3. **Prepare Metadata**
   ```rust
   let knowledge_type = request.knowledge_type
       .unwrap_or_else(|| "decision".to_string());

   let id = format!(
       "{}-{}-{}",
       knowledge_type,
       Utc::now().timestamp(),
       uuid::Uuid::new_v4().to_string()[..7].to_string()
   );
   ```

4. **Create Item**
   ```rust
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
   ```

5. **Store in Memory**
   ```rust
   self.items.push(item);
   ```

### Batch Operations

**Method Signature:**
```rust
pub async fn store_batch(&mut self, requests: Vec<StoreKnowledgeRequest>) -> Result<Vec<String>>
```

**Implementation Pattern:**
```rust
pub async fn store_batch(&mut self, requests: Vec<StoreKnowledgeRequest>) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for request in requests {
        match self.store(request).await {
            Ok(response) => ids.push(response.id),
            Err(e) => warn!("Failed to store item: {}", e),
        }
    }
    Ok(ids)
}
```

**Key Points:**
- Sequential processing (could optimize with parallelization)
- Collects successes even if some fail
- Logs failures for debugging

---

## Searching Knowledge

### Vector Similarity Search

**Method Signature:**
```rust
pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>>
```

**Algorithm:**

1. **Generate Query Embedding**
   ```rust
   let query_vector = generate_embedding(&request.query);
   ```

2. **Calculate Similarities**
   ```rust
   let mut results: Vec<(f32, &KnowledgeItem)> = self
       .items
       .iter()
       .map(|item| {
           let similarity = cosine_similarity(&query_vector, &item.vector);
           (similarity, item)
       })
       .collect();
   ```

3. **Filter by Metadata**
   ```rust
   results.retain(|(_, item)| {
       if let Some(ref project_id) = request.project_id {
           if &item.project_id != project_id {
               return false;
           }
       }
       // ... other filters
       true
   });
   ```

4. **Sort and Limit**
   ```rust
   results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
   let search_results: Vec<SearchResult> = results
       .into_iter()
       .take(request.limit)
       .map(|(score, item)| SearchResult {
           // ... convert to SearchResult
           score,
       })
       .collect();
   ```

### Cosine Similarity

**Implementation:**
```rust
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
```

**Score Interpretation:**
- 1.0 = identical vectors
- 0.5-0.9 = very similar
- 0.0-0.5 = somewhat related
- Negative = opposite direction

---

## Integrating with Agents

### From Python Agents

**HTTP Client Wrapper:**
```python
import requests
import os
from typing import Dict, List, Optional

class KnowledgeClient:
    def __init__(self):
        token_path = os.path.expanduser("~/.cco/api_token")
        with open(token_path) as f:
            self.token = f.read().strip()

        self.base_url = "http://localhost:8303"
        self.headers = {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }

    def store(self, text: str, type: str, agent: str,
              project_id: Optional[str] = None) -> Dict:
        """Store a knowledge item"""
        response = requests.post(
            f"{self.base_url}/api/knowledge/store",
            headers=self.headers,
            json={
                "text": text,
                "type": type,
                "agent": agent,
                "project_id": project_id or self.get_project_id(),
            }
        )
        response.raise_for_status()
        return response.json()

    def search(self, query: str, limit: int = 10) -> Dict:
        """Search for knowledge"""
        response = requests.post(
            f"{self.base_url}/api/knowledge/search",
            headers=self.headers,
            json={
                "query": query,
                "limit": limit,
            }
        )
        response.raise_for_status()
        return response.json()

    def get_stats(self) -> Dict:
        """Get statistics"""
        response = requests.get(
            f"{self.base_url}/api/knowledge/stats",
            headers=self.headers,
        )
        response.raise_for_status()
        return response.json()

    @staticmethod
    def get_project_id() -> str:
        """Get project ID from current directory"""
        return os.path.basename(os.getcwd())
```

**Usage:**
```python
from knowledge_client import KnowledgeClient

client = KnowledgeClient()

# Store
response = client.store(
    text="We decided to use FastAPI for REST API",
    type="decision",
    agent="architect"
)
print(f"Stored: {response['id']}")

# Search
results = client.search("FastAPI REST decision")
for item in results:
    print(f"- {item['text']} (score: {item['score']:.2f})")

# Stats
stats = client.get_stats()
print(f"Total items: {stats['total_records']}")
```

### From Rust Agents

**Using reqwest:**
```rust
use reqwest::Client;
use serde_json::json;

pub struct KnowledgeClient {
    client: Client,
    base_url: String,
    token: String,
}

impl KnowledgeClient {
    pub async fn new() -> anyhow::Result<Self> {
        let token_path = dirs::home_dir()
            .unwrap()
            .join(".cco/api_token");

        let token = std::fs::read_to_string(token_path)?
            .trim()
            .to_string();

        Ok(Self {
            client: Client::new(),
            base_url: "http://localhost:8303".to_string(),
            token,
        })
    }

    pub async fn store(&self, text: String, r#type: String, agent: String) -> anyhow::Result<String> {
        let url = format!("{}/api/knowledge/store", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&json!({
                "text": text,
                "type": r#type,
                "agent": agent,
            }))
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        Ok(body["id"].as_str().unwrap_or("unknown").to_string())
    }

    pub async fn search(&self, query: String, limit: usize) -> anyhow::Result<Vec<serde_json::Value>> {
        let url = format!("{}/api/knowledge/search", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&json!({
                "query": query,
                "limit": limit,
            }))
            .send()
            .await?;

        let body: Vec<serde_json::Value> = response.json().await?;
        Ok(body)
    }
}
```

---

## Code Examples

### Example 1: Extracting Critical Knowledge

**Scenario:** Before conversation compaction, extract important decisions and architecture notes

```rust
pub fn extract_critical_knowledge(
    &self,
    conversation: &str,
    project_id: &str,
    session_id: &str,
) -> Vec<StoreKnowledgeRequest> {
    let patterns: HashMap<&str, regex::Regex> = [
        ("architecture", regex::Regex::new(r"(?i)architecture|design pattern").unwrap()),
        ("decision", regex::Regex::new(r"(?i)decided|chose|selected").unwrap()),
        ("implementation", regex::Regex::new(r"(?i)implemented|built|created").unwrap()),
    ]
    .iter()
    .map(|(k, v)| (*k, v.clone()))
    .collect();

    let messages: Vec<&str> = conversation.split("\n\n").collect();
    let mut knowledge = Vec::new();

    for message in messages {
        if message.len() < 50 {
            continue;  // Skip short messages
        }

        let knowledge_type = patterns
            .iter()
            .find(|(_, regex)| regex.is_match(message))
            .map(|(k, _)| k.to_string())
            .unwrap_or_else(|| "general".to_string());

        knowledge.push(StoreKnowledgeRequest {
            text: message.trim().to_string(),
            knowledge_type: Some(knowledge_type),
            project_id: Some(project_id.to_string()),
            session_id: Some(session_id.to_string()),
            agent: None,
            metadata: None,
        });
    }

    knowledge
}
```

### Example 2: Retrieving Post-Compaction Context

```rust
pub async fn post_compaction(&self, request: PostCompactionRequest) -> Result<PostCompactionResponse> {
    // Search for semantically similar knowledge
    let search_request = SearchRequest {
        query: request.current_task.clone(),
        limit: request.limit,
        threshold: 0.5,
        project_id: request.project_id.clone(),
        knowledge_type: None,
        agent: None,
    };

    let search_results = self.search(search_request).await?;

    // Get most recent items
    let project_id = request.project_id.unwrap_or_else(|| "default".to_string());
    let recent_knowledge = self.get_project_knowledge(&project_id, None, 5).await?;

    // Generate summary
    let summary = self.generate_context_summary(&search_results, &recent_knowledge);

    Ok(PostCompactionResponse {
        search_results,
        recent_knowledge,
        summary,
    })
}
```

### Example 3: Custom Filtering

```rust
pub async fn search_by_agent(
    &self,
    agent: &str,
    project_id: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let mut results: Vec<&KnowledgeItem> = self
        .items
        .iter()
        .filter(|item| item.agent == agent && item.project_id == project_id)
        .collect();

    // Sort by timestamp (newest first)
    results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let search_results: Vec<SearchResult> = results
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
            score: 1.0,  // Perfect match
        })
        .collect();

    Ok(search_results)
}
```

---

## Testing Guide

### Unit Tests

**In `store.rs`:**

```rust
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
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let mut store = KnowledgeStore::new(
            temp_dir.path(),
            Some(temp_dir.path()),
            None,
        );
        store.initialize().await.unwrap();

        let request = StoreKnowledgeRequest {
            text: "Test knowledge".to_string(),
            knowledge_type: Some("decision".to_string()),
            project_id: None,
            session_id: None,
            agent: None,
            metadata: None,
        };

        let response = store.store(request).await.unwrap();
        assert!(!response.id.is_empty());
        assert!(response.stored);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }
}
```

### Integration Tests

**In `tests/knowledge_store_integration_tests.rs`:**

```rust
#[tokio::test]
async fn test_store_search_workflow() {
    let mut store = create_test_store().await;

    // Store an item
    let store_response = store.store(StoreKnowledgeRequest {
        text: "Architecture: Use microservices".to_string(),
        knowledge_type: Some("architecture".to_string()),
        project_id: Some("test-project".to_string()),
        session_id: Some("session-1".to_string()),
        agent: Some("architect".to_string()),
        metadata: None,
    }).await.unwrap();

    // Search for it
    let results = store.search(SearchRequest {
        query: "microservices architecture".to_string(),
        limit: 10,
        threshold: 0.0,  // Accept all results
        project_id: Some("test-project".to_string()),
        knowledge_type: None,
        agent: None,
    }).await.unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].id, store_response.id);
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_store_and_retrieve

# Run with output
cargo test -- --nocapture

# Run only integration tests
cargo test --test knowledge_store_integration_tests

# Run with coverage
cargo tarpaulin --out Html
```

---

## Debugging Tips

### Enable Detailed Logging

**In code:**
```rust
use tracing::{debug, info, warn, error};

debug!("Detailed debug info: {:?}", variable);
info!("User action: Stored knowledge {}", id);
warn!("Potential issue: High latency {} ms", latency);
error!("Failed to store: {:?}", error);
```

**Enable in runtime:**
```bash
RUST_LOG=debug cargo run
RUST_LOG=cco::daemon::knowledge=trace cargo run
```

### Common Issues

**Issue 1: "Text field exceeds 10 MB limit"**
```rust
// Check size before sending
let size = request.text.len();
if size > 10 * 1024 * 1024 {
    eprintln!("Request too large: {} MB", size / 1024 / 1024);
}
```

**Issue 2: "Request contains sensitive data"**
```rust
// Test credential detection
let detector = CredentialDetector::new();
if detector.contains_credentials(&text) {
    println!("Credential pattern detected!");
}
```

**Issue 3: "Metadata validation failed"**
```rust
// Validate JSON before sending
let metadata = serde_json::json!({
    "key": "value",
    "nested": {"field": 123}
});

match serde_json::to_string(&metadata) {
    Ok(json_str) => println!("Valid: {}", json_str),
    Err(e) => eprintln!("Invalid JSON: {}", e),
}
```

### Performance Profiling

**Measure operation latency:**
```rust
let start = std::time::Instant::now();
let result = store.search(request).await?;
let elapsed = start.elapsed();
println!("Search took: {:?}", elapsed);
```

**Memory usage:**
```bash
# Monitor memory during test
/usr/bin/time -v cargo test 2>&1 | grep "Maximum resident"
```

---

## Contributing Changes

### Code Style

**Follow Rust conventions:**
- Use 4-space indentation
- Use `SCREAMING_SNAKE_CASE` for constants
- Use snake_case for functions/variables
- Use PascalCase for types/structs

**Example:**
```rust
const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;

pub struct KnowledgeStore {
    items: Vec<KnowledgeItem>,
}

pub async fn store_knowledge(text: String) -> Result<String> {
    if text.len() > MAX_TEXT_SIZE {
        anyhow::bail!("Text too large");
    }
    // ...
    Ok(id)
}
```

### Documentation

**Add doc comments:**
```rust
/// Store a single knowledge item
///
/// # Arguments
/// * `request` - StoreKnowledgeRequest with text and metadata
///
/// # Returns
/// * `Ok(StoreKnowledgeResponse)` - Item stored successfully
/// * `Err(anyhow::Error)` - Storage failed
///
/// # Example
/// ```no_run
/// let store = KnowledgeStore::new(path, None, None);
/// let response = store.store(request).await?;
/// ```
pub async fn store(&mut self, request: StoreKnowledgeRequest) -> Result<StoreKnowledgeResponse>
```

### Testing Requirements

- Minimum 80% code coverage
- Test all error paths
- Test edge cases (empty input, max size, etc.)
- Integration tests for workflows

### Pull Request Process

1. Create feature branch: `git checkout -b feature/my-feature`
2. Make changes with tests
3. Run tests: `cargo test`
4. Format code: `cargo fmt`
5. Lint: `cargo clippy`
6. Commit with descriptive message
7. Push and create PR

---

## Performance Tuning

### Current Bottlenecks

| Component | Issue | Impact |
|-----------|-------|--------|
| Linear search | O(n) scan | Slow with 100K+ items |
| In-memory storage | RAM limited | Can't store millions |
| No indexing | All items scanned | Sublinear not possible |

### Optimization Opportunities

1. **Vector Indexing**
   ```
   Current:  O(n) linear scan
   Optimized: O(log n) with HNSW index
   Benefit: 100x faster searches with 1M items
   ```

2. **Disk Persistence**
   ```
   Current:   All in memory, lost on restart
   Optimized: Disk-backed with caching
   Benefit: Unlimited items, survives restarts
   ```

3. **Batch Operations**
   ```
   Current:  Sequential processing
   Optimized: Parallel with rayon
   Benefit: 4-8x faster batch operations
   ```

4. **Query Optimization**
   ```
   Current:  Full filter after search
   Optimized: Filter before similarity calc
   Benefit: Skip unnecessary calculations
   ```

### Monitoring Performance

**Add metrics:**
```rust
pub async fn store(&mut self, request: StoreKnowledgeRequest) -> Result<StoreKnowledgeResponse> {
    let start = std::time::Instant::now();

    // ... store logic ...

    let elapsed = start.elapsed();
    metrics::histogram!("knowledge_store_operation", elapsed.as_secs_f64(),
                       "operation" => "store");

    Ok(response)
}
```

---

## Related Documentation

- [Architecture Guide](KNOWLEDGE_STORE_ARCHITECTURE.md) - System design
- [API Reference](KNOWLEDGE_STORE_API.md) - Endpoint specifications
- [Security Guide](KNOWLEDGE_STORE_SECURITY.md) - Security practices
- [Troubleshooting Guide](KNOWLEDGE_STORE_TROUBLESHOOTING.md) - Problem solving

---

**Last Updated:** November 28, 2025
**Version:** 1.0.0
**Maintained by:** CCO Development Team
