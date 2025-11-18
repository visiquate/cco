# Knowledge Store Migration Guide

**From:** Node.js subprocess (`knowledge-manager.js`)
**To:** HTTP API (embedded in CCO daemon)
**Version:** 1.0.0
**Last Updated:** November 18, 2025

---

## Table of Contents

1. [Migration Overview](#migration-overview)
2. [Key Differences](#key-differences)
3. [Before and After Comparison](#before-and-after-comparison)
4. [Migration Steps](#migration-steps)
5. [Code Migration Examples](#code-migration-examples)
6. [Data Migration](#data-migration)
7. [Troubleshooting](#troubleshooting)
8. [Rollback Plan](#rollback-plan)

---

## Migration Overview

### What's Changing?

The knowledge management system is moving from a **Node.js subprocess** to an **HTTP API** embedded in the CCO daemon (Rust).

### Why Migrate?

| Aspect | Node.js (Old) | HTTP API (New) | Improvement |
|--------|--------------|----------------|-------------|
| **Performance** | 50-200ms per call | 1-5ms per call | **10-40x faster** |
| **Concurrency** | Single-threaded | Thousands concurrent | **100x better** |
| **Distribution** | Node.js required | Single binary | **Simpler** |
| **Features** | Basic search | Vector search + SQL | **More powerful** |
| **Memory** | ~100 MB overhead | ~20 MB overhead | **5x less** |
| **Reliability** | Process spawn failures | HTTP client | **More reliable** |

### Timeline

```
Week 1: Prepare
  - Review documentation
  - Update code
  - Test locally

Week 2: Deploy
  - Migrate data
  - Deploy updated agents
  - Monitor

Week 3: Cleanup
  - Remove old code
  - Archive Node.js version
```

---

## Key Differences

### 1. Access Method

**Before:** Subprocess spawn
```bash
node ~/git/cc-orchestra/src/knowledge-manager.js store "text" decision architect
```

**After:** HTTP POST
```bash
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{"text": "...", "type": "decision", "agent": "architect"}'
```

### 2. Authentication

**Before:** No authentication (local subprocess)

**After:** Bearer token required
```bash
TOKEN=$(cat ~/.cco/api_token)
curl -H "Authorization: Bearer $TOKEN" ...
```

### 3. Data Format

**Before:** Command-line arguments
```bash
knowledge-manager store "text" type agent
```

**After:** JSON request body
```json
{
  "text": "text",
  "type": "type",
  "agent": "agent",
  "project_id": "project",
  "session_id": "session"
}
```

### 4. Response Format

**Before:** Plain text output
```
Stored knowledge with ID: decision-123
```

**After:** JSON response
```json
{
  "status": "success",
  "id": "decision-1700321400123-4f8a3b2c",
  "timestamp": "2025-11-18T15:30:00Z"
}
```

### 5. Search Results

**Before:** Text lines
```
decision-123 | 0.89 | Decided to use FastAPI
decision-456 | 0.76 | Authentication strategy chosen
```

**After:** JSON array
```json
{
  "results": [
    {
      "id": "decision-123",
      "text": "Decided to use FastAPI",
      "similarity": 0.89,
      "agent": "architect"
    }
  ],
  "count": 2,
  "query_time_ms": 15
}
```

---

## Before and After Comparison

### Storing Knowledge

**Before (Node.js):**
```python
import subprocess

def store_knowledge(text, type, agent):
    subprocess.run([
        'node',
        '~/git/cc-orchestra/src/knowledge-manager.js',
        'store',
        text,
        type,
        agent
    ])
```

**After (HTTP API):**
```python
import requests
import os

def store_knowledge(text, type, agent):
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    requests.post(
        'http://localhost:8303/api/knowledge/store',
        headers={'Authorization': f'Bearer {token}'},
        json={
            'text': text,
            'type': type,
            'agent': agent,
            'project_id': os.path.basename(os.getcwd()),
            'session_id': os.environ.get('AGENT_SESSION_ID', 'default')
        }
    )
```

### Searching Knowledge

**Before (Node.js):**
```python
import subprocess

def search_knowledge(query):
    result = subprocess.run(
        ['node', '~/git/cc-orchestra/src/knowledge-manager.js', 'search', query],
        capture_output=True,
        text=True
    )
    # Parse text output
    lines = result.stdout.strip().split('\n')
    return [parse_line(line) for line in lines]
```

**After (HTTP API):**
```python
import requests
import os

def search_knowledge(query):
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    response = requests.get(
        'http://localhost:8303/api/knowledge/search',
        headers={'Authorization': f'Bearer {token}'},
        params={
            'q': query,
            'project_id': os.path.basename(os.getcwd()),
            'limit': 10
        }
    )
    return response.json()['results']
```

### Getting Statistics

**Before (Node.js):**
```python
def get_stats():
    result = subprocess.run(
        ['node', '~/git/cc-orchestra/src/knowledge-manager.js', 'stats'],
        capture_output=True,
        text=True
    )
    # Parse text output manually
    lines = result.stdout.split('\n')
    stats = {}
    for line in lines:
        if ':' in line:
            key, value = line.split(':', 1)
            stats[key.strip()] = value.strip()
    return stats
```

**After (HTTP API):**
```python
def get_stats():
    with open(os.path.expanduser("~/.cco/api_token")) as f:
        token = f.read().strip()

    response = requests.get(
        'http://localhost:8303/api/knowledge/stats',
        headers={'Authorization': f'Bearer {token}'},
        params={'project_id': os.path.basename(os.getcwd())}
    )
    return response.json()
```

---

## Migration Steps

### Step 1: Verify Prerequisites

```bash
# Check CCO daemon is running
cco daemon status

# Verify API token exists
ls -la ~/.cco/api_token

# Test health endpoint
curl http://localhost:8303/api/knowledge/health \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

Expected output:
```json
{
  "status": "healthy",
  "database": "connected",
  "embedding_model": "loaded",
  "uptime_seconds": 3600
}
```

### Step 2: Update Dependencies

**Python:**
```bash
# Add requests if not already installed
pip install requests
```

**JavaScript/Node.js:**
```bash
# Add axios if not already installed
npm install axios
```

**Rust:**
```toml
# Add to Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

### Step 3: Create HTTP Client Wrapper

Create a reusable client module in your agent code:

**Python (`knowledge_client.py`):**
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
        self.project_id = os.path.basename(os.getcwd())
        self.session_id = os.environ.get("AGENT_SESSION_ID", "default")

    def store(self, text: str, type: str, agent: str, metadata: Optional[Dict] = None):
        response = requests.post(
            f"{self.base_url}/api/knowledge/store",
            headers=self.headers,
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

    def search(self, query: str, type: Optional[str] = None, limit: int = 10):
        params = {
            "q": query,
            "project_id": self.project_id,
            "limit": limit
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

    def stats(self):
        response = requests.get(
            f"{self.base_url}/api/knowledge/stats",
            headers=self.headers,
            params={"project_id": self.project_id}
        )
        response.raise_for_status()
        return response.json()
```

### Step 4: Replace Old Code

**Find all uses of old knowledge manager:**

```bash
# Find subprocess calls
grep -r "subprocess.*knowledge-manager" .
grep -r "node.*knowledge-manager" .

# Find direct script calls
grep -r "knowledge-manager.js" .
```

**Replace with HTTP client:**

```python
# Before
import subprocess
subprocess.run(['node', '~/git/cc-orchestra/src/knowledge-manager.js', ...])

# After
from knowledge_client import KnowledgeClient
client = KnowledgeClient()
client.store(...)
```

### Step 5: Test Locally

```python
# Test script
from knowledge_client import KnowledgeClient

client = KnowledgeClient()

# Test store
result = client.store(
    "Migration test",
    "general",
    "test-agent"
)
print(f"Stored: {result['id']}")

# Test search
results = client.search("migration test")
print(f"Found {results['count']} results")
assert results['count'] > 0

# Test stats
stats = client.stats()
print(f"Total entries: {stats['total_entries']}")

print("All tests passed!")
```

### Step 6: Deploy Updated Agent

```bash
# Commit changes
git add .
git commit -m "migrate: Switch to knowledge store HTTP API"

# Deploy
git push
```

### Step 7: Monitor

```bash
# Check agent logs for errors
tail -f ~/.cco/logs/agent.log

# Monitor daemon logs
tail -f ~/.cco/logs/daemon.log

# Check knowledge store health
curl http://localhost:8303/api/knowledge/health \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

### Step 8: Remove Old Code

After successful deployment (wait 1-2 weeks):

```bash
# Remove old subprocess calls
# Remove Node.js knowledge-manager.js references
# Update documentation
# Archive old code in git history
```

---

## Code Migration Examples

### Example 1: Simple Store

**Before:**
```python
import subprocess

subprocess.run([
    'node',
    os.path.expanduser('~/git/cc-orchestra/src/knowledge-manager.js'),
    'store',
    'Implemented feature X',
    'implementation',
    'python-specialist'
])
```

**After:**
```python
from knowledge_client import KnowledgeClient

client = KnowledgeClient()
client.store(
    'Implemented feature X',
    'implementation',
    'python-specialist'
)
```

### Example 2: Search with Parsing

**Before:**
```python
import subprocess

result = subprocess.run(
    ['node', '~/git/cc-orchestra/src/knowledge-manager.js', 'search', 'authentication'],
    capture_output=True,
    text=True
)

# Parse text output
for line in result.stdout.strip().split('\n'):
    if line:
        parts = line.split('|')
        id = parts[0].strip()
        score = float(parts[1].strip())
        text = parts[2].strip()
        print(f"Found: {text} (score: {score})")
```

**After:**
```python
from knowledge_client import KnowledgeClient

client = KnowledgeClient()
results = client.search('authentication')

for item in results['results']:
    print(f"Found: {item['text']} (similarity: {item['similarity']})")
```

### Example 3: Error Handling

**Before:**
```python
import subprocess

try:
    result = subprocess.run(
        ['node', '~/git/cc-orchestra/src/knowledge-manager.js', 'store', ...],
        check=True,
        capture_output=True
    )
except subprocess.CalledProcessError as e:
    print(f"Failed: {e.stderr}")
```

**After:**
```python
from knowledge_client import KnowledgeClient
import requests

client = KnowledgeClient()

try:
    result = client.store(...)
    print(f"Stored: {result['id']}")
except requests.HTTPError as e:
    print(f"Failed: {e.response.json()['error']}")
except requests.ConnectionError:
    print("Cannot connect to daemon")
```

### Example 4: Batch Operations

**Before:**
```python
for item in items:
    subprocess.run([
        'node',
        '~/git/cc-orchestra/src/knowledge-manager.js',
        'store',
        item.text,
        item.type,
        item.agent
    ])
```

**After:**
```python
from knowledge_client import KnowledgeClient

client = KnowledgeClient()

# Option 1: Use batch endpoint (more efficient)
batch_items = [
    {
        "text": item.text,
        "type": item.type,
        "agent": item.agent,
        "project_id": client.project_id,
        "session_id": client.session_id
    }
    for item in items
]

response = requests.post(
    f"{client.base_url}/api/knowledge/store/batch",
    headers=client.headers,
    json={"items": batch_items}
)

# Option 2: Loop with HTTP (still faster than subprocess)
for item in items:
    client.store(item.text, item.type, item.agent)
```

---

## Data Migration

### Automatic Data Migration

The existing LanceDB data format is **compatible** between Node.js and Rust implementations. No manual data migration is required.

**Location:**
```
Before: ~/git/cc-orchestra/data/knowledge/{project}/
After:  ~/.cco/knowledge/{project}/
```

**Migration Steps:**

1. **Stop Node.js knowledge-manager** (if running as daemon)
2. **Copy data** to new location:
   ```bash
   mkdir -p ~/.cco/knowledge
   cp -r ~/git/cc-orchestra/data/knowledge/* ~/.cco/knowledge/
   ```
3. **Verify copy**:
   ```bash
   # Check file count
   find ~/git/cc-orchestra/data/knowledge -type f | wc -l
   find ~/.cco/knowledge -type f | wc -l
   # Should match
   ```
4. **Test with new API**:
   ```bash
   curl http://localhost:8303/api/knowledge/stats \
     -H "Authorization: Bearer $(cat ~/.cco/api_token)"
   # Should show existing entries
   ```

### Manual Data Migration (if needed)

If data format changes between versions:

```python
# migration_script.py
from knowledge_client import KnowledgeClient
import json
import os

# Read old data
old_data_dir = os.path.expanduser("~/git/cc-orchestra/data/knowledge")

client = KnowledgeClient()

for project_dir in os.listdir(old_data_dir):
    project_path = os.path.join(old_data_dir, project_dir)
    if not os.path.isdir(project_path):
        continue

    print(f"Migrating project: {project_dir}")

    # Read old format (example)
    data_file = os.path.join(project_path, "knowledge.json")
    if not os.path.exists(data_file):
        continue

    with open(data_file) as f:
        old_entries = json.load(f)

    # Migrate each entry
    for entry in old_entries:
        try:
            client.store(
                text=entry['text'],
                type=entry.get('type', 'general'),
                agent=entry.get('agent', 'unknown'),
                metadata=entry.get('metadata', {})
            )
            print(f"  Migrated: {entry['id']}")
        except Exception as e:
            print(f"  Failed: {entry['id']} - {e}")

    print(f"Completed: {project_dir}")
```

---

## Troubleshooting

### Issue 1: "Connection refused"

**Symptom:**
```
requests.exceptions.ConnectionError: Connection refused
```

**Solution:**
```bash
# Check daemon status
cco daemon status

# Start daemon if not running
cco daemon start

# Verify port
lsof -i :8303
```

### Issue 2: "401 Unauthorized"

**Symptom:**
```
HTTP 401: Unauthorized
```

**Solution:**
```bash
# Check token file exists
ls -la ~/.cco/api_token

# Regenerate token
cco daemon restart

# Verify token is being read correctly
cat ~/.cco/api_token
echo -n "Bearer $(cat ~/.cco/api_token)" | wc -c  # Should be > 10
```

### Issue 3: "Invalid knowledge type"

**Symptom:**
```json
{
  "error": "Invalid knowledge type",
  "code": "INVALID_TYPE"
}
```

**Solution:**
```python
# Use valid types only
VALID_TYPES = [
    "decision",
    "architecture",
    "implementation",
    "configuration",
    "credential",
    "issue",
    "general"
]

# Check type before storing
if type not in VALID_TYPES:
    type = "general"  # Default fallback
```

### Issue 4: "No results found" after migration

**Symptom:**
Search returns no results after migration.

**Solution:**
```bash
# Check data was copied
ls -la ~/.cco/knowledge/

# Check stats show entries
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"

# Verify project_id matches
# Before: might be "cc-orchestra"
# After: must match exactly

# Debug search
curl "http://localhost:8303/api/knowledge/search?q=test&project_id=EXACT_PROJECT_ID" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

### Issue 5: Performance slower than expected

**Symptom:**
HTTP requests taking >100ms.

**Solution:**
```python
# Use connection pooling
import requests
from requests.adapters import HTTPAdapter

session = requests.Session()
adapter = HTTPAdapter(pool_connections=10, pool_maxsize=10)
session.mount('http://', adapter)

# Reuse session for all requests
session.post(url, ...)
session.get(url, ...)

# Check daemon health
curl http://localhost:8303/api/knowledge/health
```

---

## Rollback Plan

If migration encounters critical issues:

### Step 1: Restore Old Code

```bash
# Revert to previous commit
git revert HEAD

# Or restore from backup
git checkout previous-stable-commit
```

### Step 2: Restore Old Data (if needed)

```bash
# If data was moved
cp -r ~/.cco/knowledge/* ~/git/cc-orchestra/data/knowledge/

# Verify
ls -la ~/git/cc-orchestra/data/knowledge/
```

### Step 3: Restart Old System

```bash
# Test old knowledge manager still works
node ~/git/cc-orchestra/src/knowledge-manager.js stats

# If running as daemon, restart it
# (implementation-specific)
```

### Step 4: Document Issues

```markdown
## Rollback Report

**Date:** 2025-11-XX
**Reason:** [Specific issue encountered]

**Issues:**
1. [Issue 1]
2. [Issue 2]

**Impact:** [Describe impact]

**Next Steps:**
1. [Fix approach]
2. [Timeline]
```

---

## Success Checklist

Migration is successful when:

- [ ] All agents can store knowledge via HTTP API
- [ ] Search returns expected results
- [ ] Statistics show all migrated entries
- [ ] Performance is better than subprocess (< 10ms per call)
- [ ] No errors in daemon logs
- [ ] No subprocess calls remaining in codebase
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Team trained on new API

---

## See Also

- [API Reference](KNOWLEDGE_STORE_API.md) - Complete API documentation
- [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md) - How to use from agents
- [Quick Start](KNOWLEDGE_STORE_QUICK_START.md) - Get started in 5 minutes
- [Architecture](KNOWLEDGE_STORE_ARCHITECTURE.md) - How it works internally

---

**Last Updated:** November 18, 2025
**Version:** 1.0.0
**Maintained by:** CCO Team
