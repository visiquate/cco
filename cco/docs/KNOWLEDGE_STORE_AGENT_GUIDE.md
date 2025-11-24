# Knowledge Store Agent Integration Guide

**For:** Agent Developers
**Version:** 1.0.0
**Last Updated:** November 18, 2025

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [HTTP Client Setup](#http-client-setup)
3. [Common Patterns](#common-patterns)
4. [Error Handling](#error-handling)
5. [Best Practices](#best-practices)
6. [Language Examples](#language-examples)
7. [Testing](#testing)

---

## Quick Start

### What Changed?

**Before (Node.js subprocess):**
```bash
node ~/git/cc-orchestra/src/knowledge-manager.js store "knowledge text" decision architect
```

**After (HTTP API):**
```bash
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "knowledge text",
    "type": "decision",
    "agent": "architect",
    "project_id": "current-project",
    "session_id": "session-123"
  }'
```

### Why the Change?

- **Faster**: No process spawn overhead (50-200ms → 1-5ms)
- **More reliable**: No subprocess failures
- **Better concurrency**: Thousands of simultaneous requests
- **Richer API**: Vector search, SQL queries, batch operations
- **Single binary**: No Node.js dependency

---

## HTTP Client Setup

### Authentication Token

All requests require a Bearer token from `~/.cco/api_token`:

```bash
# Read token
TOKEN=$(cat ~/.cco/api_token)

# Use in request
curl -H "Authorization: Bearer $TOKEN" ...
```

### Base URL

```
http://localhost:8303
```

The CCO daemon runs on port 8303 by default.

### Required Headers

```http
Authorization: Bearer <token>
Content-Type: application/json
```

---

## Common Patterns

### Pattern 1: Store Knowledge After Task

Store what you learned or decided during task execution.

**Python Example:**

```python
import requests
import os

def store_knowledge(text, type, agent):
    """Store knowledge after completing a task."""
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    response = requests.post(
        "http://localhost:8303/api/knowledge/store",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json"
        },
        json={
            "text": text,
            "type": type,
            "project_id": os.path.basename(os.getcwd()),
            "session_id": os.environ.get("AGENT_SESSION_ID", "default"),
            "agent": agent,
            "metadata": {}
        }
    )
    return response.json()

# Usage in agent
result = implement_feature()
store_knowledge(
    text=f"Implemented {result.feature} using {result.technology}",
    type="implementation",
    agent="python-specialist"
)
```

### Pattern 2: Search Before Implementation

Search for relevant decisions or patterns before starting work.

**Python Example:**

```python
def search_knowledge(query, type=None):
    """Search for relevant knowledge before starting work."""
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    params = {
        "q": query,
        "project_id": os.path.basename(os.getcwd()),
        "limit": 5
    }
    if type:
        params["type"] = type

    response = requests.get(
        "http://localhost:8303/api/knowledge/search",
        headers={"Authorization": f"Bearer {token}"},
        params=params
    )
    return response.json()

# Usage in agent
print("Searching for authentication decisions...")
results = search_knowledge("authentication security", type="decision")

for item in results['results']:
    print(f"Found: {item['text'][:100]}...")
    print(f"Agent: {item['agent']}, Similarity: {item['similarity']:.2f}")
    print()

# Use findings to inform implementation
if results['results']:
    print("Using previous authentication decisions as reference...")
```

### Pattern 3: Batch Store Multiple Items

Store multiple knowledge items at once for efficiency.

**Python Example:**

```python
def batch_store_knowledge(items):
    """Store multiple knowledge items efficiently."""
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    project_id = os.path.basename(os.getcwd())
    session_id = os.environ.get("AGENT_SESSION_ID", "default")

    formatted_items = [
        {
            "text": item["text"],
            "type": item["type"],
            "project_id": project_id,
            "session_id": session_id,
            "agent": item["agent"],
            "metadata": item.get("metadata", {})
        }
        for item in items
    ]

    response = requests.post(
        "http://localhost:8303/api/knowledge/store/batch",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json"
        },
        json={"items": formatted_items}
    )
    return response.json()

# Usage in agent
knowledge_items = [
    {
        "text": "Added user authentication endpoint",
        "type": "implementation",
        "agent": "python-specialist"
    },
    {
        "text": "Added JWT token validation",
        "type": "implementation",
        "agent": "python-specialist"
    },
    {
        "text": "Added rate limiting middleware",
        "type": "implementation",
        "agent": "python-specialist"
    }
]

result = batch_store_knowledge(knowledge_items)
print(f"Stored {result['stored']} items")
```

### Pattern 4: Query by Agent or Type

Find all knowledge from a specific agent or of a specific type.

**Python Example:**

```python
def query_by_agent(agent, limit=20):
    """Query all knowledge from a specific agent."""
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    response = requests.post(
        "http://localhost:8303/api/knowledge/query",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json"
        },
        json={
            "project_id": os.path.basename(os.getcwd()),
            "agent": agent,
            "limit": limit,
            "order_by": "timestamp DESC"
        }
    )
    return response.json()

# Usage: Review what architect decided
architect_decisions = query_by_agent("architect")
print(f"Architect made {architect_decisions['total']} entries:")
for entry in architect_decisions['results']:
    print(f"- {entry['text'][:80]}...")
```

### Pattern 5: Check Statistics Before Cleanup

Review knowledge statistics before implementing changes.

**Python Example:**

```python
def get_project_stats():
    """Get knowledge statistics for current project."""
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    response = requests.get(
        "http://localhost:8303/api/knowledge/stats",
        headers={"Authorization": f"Bearer {token}"},
        params={"project_id": os.path.basename(os.getcwd())}
    )
    return response.json()

# Usage
stats = get_project_stats()
print(f"Total knowledge entries: {stats['total_entries']}")
print(f"By type: {stats['by_type']}")
print(f"Database size: {stats['database_size_mb']} MB")

# Decide if cleanup needed
if stats['database_size_mb'] > 50:
    print("Consider running cleanup...")
```

---

## Error Handling

### Retry Strategy

Implement exponential backoff for transient failures:

```python
import time

def store_with_retry(text, type, agent, max_retries=3):
    """Store knowledge with automatic retry on failure."""
    retry_delay = 1  # seconds

    for attempt in range(max_retries):
        try:
            return store_knowledge(text, type, agent)
        except requests.exceptions.RequestException as e:
            if attempt < max_retries - 1:
                print(f"Store failed (attempt {attempt + 1}), retrying in {retry_delay}s...")
                time.sleep(retry_delay)
                retry_delay *= 2  # Exponential backoff
            else:
                print(f"Store failed after {max_retries} attempts: {e}")
                raise

# Usage
try:
    result = store_with_retry(
        "Important decision that must be stored",
        "decision",
        "architect"
    )
    print(f"Stored: {result['id']}")
except Exception as e:
    print(f"Failed to store knowledge: {e}")
    # Continue with agent work - knowledge storage is not critical path
```

### Handle Specific Errors

```python
def safe_store_knowledge(text, type, agent):
    """Store knowledge with comprehensive error handling."""
    try:
        response = requests.post(
            "http://localhost:8303/api/knowledge/store",
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json"
            },
            json={
                "text": text,
                "type": type,
                "project_id": os.path.basename(os.getcwd()),
                "session_id": os.environ.get("AGENT_SESSION_ID", "default"),
                "agent": agent
            },
            timeout=5  # 5 second timeout
        )

        if response.status_code == 401:
            print("ERROR: Invalid auth token. Check ~/.cco/api_token")
            return None
        elif response.status_code == 400:
            error = response.json()
            print(f"ERROR: Invalid request: {error['error']}")
            return None
        elif response.status_code == 429:
            print("ERROR: Rate limit exceeded. Waiting...")
            time.sleep(60)  # Wait 1 minute
            return safe_store_knowledge(text, type, agent)  # Retry
        elif response.status_code >= 500:
            print(f"ERROR: Server error: {response.status_code}")
            return None

        response.raise_for_status()
        return response.json()

    except requests.exceptions.ConnectionError:
        print("ERROR: Cannot connect to daemon. Is it running?")
        print("Run: cco daemon status")
        return None
    except requests.exceptions.Timeout:
        print("ERROR: Request timeout. Daemon may be overloaded.")
        return None
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}")
        return None

# Usage
result = safe_store_knowledge("Decision text", "decision", "architect")
if result:
    print(f"Success: {result['id']}")
else:
    print("Failed to store, but continuing with task...")
```

### Graceful Degradation

```python
def search_with_fallback(query, type=None):
    """Search knowledge with fallback to empty results."""
    try:
        return search_knowledge(query, type)
    except Exception as e:
        print(f"Knowledge search failed: {e}")
        print("Continuing without historical context...")
        return {"results": [], "count": 0, "query_time_ms": 0}

# Usage - agent continues even if search fails
results = search_with_fallback("authentication patterns")
if results['count'] > 0:
    print(f"Found {results['count']} relevant patterns")
else:
    print("No historical patterns found, implementing from scratch")
```

---

## Best Practices

### 1. Always Include Project ID

```python
# Get current project from working directory
import os
project_id = os.path.basename(os.getcwd())

# Or from environment variable
project_id = os.environ.get("PROJECT_ID", "unknown")

# Or from git remote
import subprocess
result = subprocess.run(
    ["git", "remote", "get-url", "origin"],
    capture_output=True,
    text=True
)
project_id = result.stdout.strip().split("/")[-1].replace(".git", "")
```

### 2. Use Session IDs for Grouping

```python
import uuid

# Generate session ID at agent startup
session_id = str(uuid.uuid4())

# Store in environment
os.environ["AGENT_SESSION_ID"] = session_id

# Use in all knowledge operations
store_knowledge(..., session_id=session_id)
```

### 3. Add Useful Metadata

```python
metadata = {
    "confidence": 0.95,  # How confident in this knowledge
    "tags": ["api", "security"],  # Categories
    "file": "src/auth.py",  # Related file
    "line": 42,  # Related line number
    "commit": "abc123",  # Related commit
    "related_issues": ["#123", "#456"]  # Related issues
}

store_knowledge(
    text="Implemented JWT authentication",
    type="implementation",
    agent="python-specialist",
    metadata=metadata
)
```

### 4. Search Before Storing Duplicates

```python
def store_unique_knowledge(text, type, agent):
    """Only store if similar knowledge doesn't exist."""
    # Search for similar entries
    results = search_knowledge(text, type=type)

    # Check similarity threshold
    if results['count'] > 0 and results['results'][0]['similarity'] > 0.9:
        print(f"Similar knowledge already exists: {results['results'][0]['id']}")
        return results['results'][0]

    # Store new knowledge
    return store_knowledge(text, type, agent)
```

### 5. Use Appropriate Types

```python
# Good - specific types
store_knowledge("Decided to use FastAPI", "decision", agent)
store_knowledge("Implemented /auth endpoint", "implementation", agent)
store_knowledge("Found memory leak in worker", "issue", agent)

# Bad - everything as general
store_knowledge("Decided to use FastAPI", "general", agent)
store_knowledge("Implemented /auth endpoint", "general", agent)
```

### 6. Batch When Possible

```python
# Bad - multiple HTTP requests
for item in items:
    store_knowledge(item.text, item.type, agent)

# Good - single batch request
batch_store_knowledge(items)
```

### 7. Set Appropriate Search Limits

```python
# For quick context - low limit
search_knowledge("authentication", limit=3)

# For comprehensive analysis - higher limit
search_knowledge("security decisions", limit=20)

# Don't request more than needed
# Bad: search_knowledge("query", limit=100)  # Usually wasteful
```

---

## Language Examples

### Python (Requests)

```python
import requests
import os

class KnowledgeStore:
    def __init__(self):
        with open(os.path.expanduser("~/.cco/api_token")) as f:
            self.token = f.read().strip()
        self.base_url = "http://localhost:8303"
        self.project_id = os.path.basename(os.getcwd())
        self.session_id = os.environ.get("AGENT_SESSION_ID", "default")

    def store(self, text, type, agent, metadata=None):
        response = requests.post(
            f"{self.base_url}/api/knowledge/store",
            headers={
                "Authorization": f"Bearer {self.token}",
                "Content-Type": "application/json"
            },
            json={
                "text": text,
                "type": type,
                "project_id": self.project_id,
                "session_id": self.session_id,
                "agent": agent,
                "metadata": metadata or {}
            }
        )
        response.raise_for_status()
        return response.json()

    def search(self, query, type=None, limit=10):
        params = {
            "q": query,
            "project_id": self.project_id,
            "limit": limit
        }
        if type:
            params["type"] = type

        response = requests.get(
            f"{self.base_url}/api/knowledge/search",
            headers={"Authorization": f"Bearer {self.token}"},
            params=params
        )
        response.raise_for_status()
        return response.json()

# Usage
ks = KnowledgeStore()
ks.store("Implemented feature X", "implementation", "python-specialist")
results = ks.search("feature implementation")
```

### Bash

```bash
#!/bin/bash

# Configuration
BASE_URL="http://localhost:8303"
TOKEN=$(cat ~/.cco/api_token)
PROJECT_ID=$(basename $(pwd))
SESSION_ID=${AGENT_SESSION_ID:-default}

# Store knowledge
store_knowledge() {
    local text="$1"
    local type="$2"
    local agent="$3"

    curl -X POST "$BASE_URL/api/knowledge/store" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"text\": \"$text\",
            \"type\": \"$type\",
            \"project_id\": \"$PROJECT_ID\",
            \"session_id\": \"$SESSION_ID\",
            \"agent\": \"$agent\"
        }"
}

# Search knowledge
search_knowledge() {
    local query="$1"
    local limit="${2:-10}"

    curl "$BASE_URL/api/knowledge/search" \
        -H "Authorization: Bearer $TOKEN" \
        -G \
        --data-urlencode "q=$query" \
        --data-urlencode "project_id=$PROJECT_ID" \
        --data-urlencode "limit=$limit"
}

# Usage
store_knowledge "Implemented feature X" "implementation" "bash-agent"
search_knowledge "feature implementation" 5
```

### JavaScript/Node.js

```javascript
const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');

class KnowledgeStore {
    constructor() {
        const tokenPath = path.join(os.homedir(), '.cco', 'api_token');
        this.token = fs.readFileSync(tokenPath, 'utf8').trim();
        this.baseUrl = 'http://localhost:8303';
        this.projectId = path.basename(process.cwd());
        this.sessionId = process.env.AGENT_SESSION_ID || 'default';
    }

    async store(text, type, agent, metadata = {}) {
        const response = await axios.post(
            `${this.baseUrl}/api/knowledge/store`,
            {
                text,
                type,
                project_id: this.projectId,
                session_id: this.sessionId,
                agent,
                metadata
            },
            {
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            }
        );
        return response.data;
    }

    async search(query, type = null, limit = 10) {
        const params = {
            q: query,
            project_id: this.projectId,
            limit
        };
        if (type) params.type = type;

        const response = await axios.get(
            `${this.baseUrl}/api/knowledge/search`,
            {
                headers: { 'Authorization': `Bearer ${this.token}` },
                params
            }
        );
        return response.data;
    }
}

// Usage
const ks = new KnowledgeStore();
await ks.store('Implemented feature X', 'implementation', 'js-specialist');
const results = await ks.search('feature implementation');
```

### Rust

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

pub struct KnowledgeStore {
    client: Client,
    base_url: String,
    token: String,
    project_id: String,
    session_id: String,
}

impl KnowledgeStore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut token_path = dirs::home_dir().unwrap_or_default();
        token_path.push(".cco");
        token_path.push("api_token");
        let token = fs::read_to_string(token_path)?.trim().to_string();

        let project_id = std::env::current_dir()?
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let session_id = std::env::var("AGENT_SESSION_ID")
            .unwrap_or_else(|_| "default".to_string());

        Ok(Self {
            client: Client::new(),
            base_url: "http://localhost:8303".to_string(),
            token,
            project_id,
            session_id,
        })
    }

    pub async fn store(
        &self,
        text: String,
        knowledge_type: String,
        agent: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.client
            .post(format!("{}/api/knowledge/store", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&StoreRequest {
                text,
                knowledge_type,
                project_id: self.project_id.clone(),
                session_id: self.session_id.clone(),
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
        limit: usize,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.client
            .get(format!("{}/api/knowledge/search", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[
                ("q", query.as_str()),
                ("project_id", self.project_id.as_str()),
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
    let ks = KnowledgeStore::new()?;
    ks.store(
        "Implemented feature X".to_string(),
        "implementation".to_string(),
        "rust-specialist".to_string(),
    ).await?;

    let results = ks.search("feature implementation".to_string(), 10).await?;
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
```

---

## Testing

### Unit Test Example (Python)

```python
import unittest
from unittest.mock import patch, Mock
import requests

class TestKnowledgeStore(unittest.TestCase):
    def setUp(self):
        self.ks = KnowledgeStore()

    @patch('requests.post')
    def test_store_success(self, mock_post):
        # Mock successful response
        mock_response = Mock()
        mock_response.status_code = 201
        mock_response.json.return_value = {
            "status": "success",
            "id": "test-123",
            "timestamp": "2025-11-18T10:00:00Z"
        }
        mock_post.return_value = mock_response

        # Test store
        result = self.ks.store("test text", "decision", "test-agent")

        # Verify
        self.assertEqual(result['status'], 'success')
        self.assertEqual(result['id'], 'test-123')
        mock_post.assert_called_once()

    @patch('requests.get')
    def test_search_success(self, mock_get):
        # Mock successful search
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "results": [
                {
                    "id": "test-123",
                    "text": "test knowledge",
                    "similarity": 0.95
                }
            ],
            "count": 1,
            "query_time_ms": 10
        }
        mock_get.return_value = mock_response

        # Test search
        results = self.ks.search("test query")

        # Verify
        self.assertEqual(results['count'], 1)
        self.assertEqual(results['results'][0]['similarity'], 0.95)
        mock_get.assert_called_once()

if __name__ == '__main__':
    unittest.main()
```

### Integration Test Example

```python
import pytest
import uuid

@pytest.fixture
def knowledge_store():
    return KnowledgeStore()

def test_store_and_search_integration(knowledge_store):
    # Store unique knowledge
    unique_text = f"Test knowledge {uuid.uuid4()}"
    store_result = knowledge_store.store(
        unique_text,
        "general",
        "test-agent"
    )

    assert store_result['status'] == 'success'
    knowledge_id = store_result['id']

    # Search for it
    search_results = knowledge_store.search(unique_text)

    assert search_results['count'] > 0
    assert any(r['id'] == knowledge_id for r in search_results['results'])
    assert search_results['results'][0]['similarity'] > 0.9

def test_batch_store(knowledge_store):
    # Create batch
    items = [
        {"text": f"Test {i}", "type": "general", "agent": "test"}
        for i in range(5)
    ]

    result = knowledge_store.batch_store(items)

    assert result['stored'] == 5
    assert result['failed'] == 0
    assert len(result['ids']) == 5
```

---

## Connection Management

### Connection Pool Configuration

```python
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

class KnowledgeStore:
    def __init__(self):
        # Read token
        with open(os.path.expanduser("~/.cco/api_token")) as f:
            self.token = f.read().strip()

        # Create session with connection pooling
        self.session = requests.Session()

        # Configure retry strategy
        retry_strategy = Retry(
            total=3,
            backoff_factor=1,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["GET", "POST", "DELETE"]
        )

        adapter = HTTPAdapter(
            max_retries=retry_strategy,
            pool_connections=10,
            pool_maxsize=10
        )

        self.session.mount("http://", adapter)
        self.session.headers.update({
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        })

        self.base_url = "http://localhost:8303"
        self.project_id = os.path.basename(os.getcwd())
        self.session_id = os.environ.get("AGENT_SESSION_ID", "default")

    def store(self, text, type, agent, metadata=None):
        response = self.session.post(
            f"{self.base_url}/api/knowledge/store",
            json={
                "text": text,
                "type": type,
                "project_id": self.project_id,
                "session_id": self.session_id,
                "agent": agent,
                "metadata": metadata or {}
            },
            timeout=5
        )
        response.raise_for_status()
        return response.json()
```

### Health Check Before Operations

```python
def check_health(self):
    """Check if knowledge store is available."""
    try:
        response = self.session.get(
            f"{self.base_url}/api/knowledge/health",
            timeout=2
        )
        return response.status_code == 200
    except:
        return False

# Usage
if ks.check_health():
    ks.store("knowledge", "general", "agent")
else:
    print("Knowledge store unavailable, skipping...")
```

---

## See Also

- [API Reference](KNOWLEDGE_STORE_API.md) - Complete API documentation
- [Migration Guide](KNOWLEDGE_STORE_MIGRATION.md) - Migrate from Node.js version
- [Quick Start](KNOWLEDGE_STORE_QUICK_START.md) - Get started in 5 minutes
- [Architecture](KNOWLEDGE_STORE_ARCHITECTURE.md) - How it works internally

---

**Last Updated:** November 18, 2025
**Version:** 1.0.0
**Maintained by:** CCO Team
