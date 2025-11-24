# Knowledge Store Quick Start Guide

**Get started in 5 minutes**

---

## Prerequisites

1. CCO daemon running on port 8303
2. API token at `~/.cco/api_token`
3. HTTP client (curl, Python requests, etc.)

---

## Quick Test

### 1. Check Connection

```bash
curl http://localhost:8303/api/knowledge/health \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

Expected:
```json
{"status": "healthy", "database": "connected"}
```

### 2. Store Knowledge

```bash
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Quick start test - knowledge store works!",
    "type": "general",
    "project_id": "test-project",
    "session_id": "quickstart",
    "agent": "test-user"
  }'
```

Expected:
```json
{
  "status": "success",
  "id": "general-1700321400123-abc123",
  "timestamp": "2025-11-18T15:30:00Z"
}
```

### 3. Search Knowledge

```bash
curl "http://localhost:8303/api/knowledge/search?q=quick+start&project_id=test-project&limit=5" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

Expected:
```json
{
  "results": [
    {
      "id": "general-1700321400123-abc123",
      "text": "Quick start test - knowledge store works!",
      "type": "general",
      "similarity": 0.95,
      "agent": "test-user"
    }
  ],
  "count": 1,
  "query_time_ms": 12
}
```

### 4. Get Statistics

```bash
curl "http://localhost:8303/api/knowledge/stats?project_id=test-project" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

Expected:
```json
{
  "total_entries": 1,
  "by_type": {"general": 1},
  "by_agent": {"test-user": 1}
}
```

---

## Python Quick Start

### Install Dependencies

```bash
pip install requests
```

### Create Client

```python
# knowledge_client.py
import requests
import os

class KnowledgeClient:
    def __init__(self):
        with open(os.path.expanduser("~/.cco/api_token")) as f:
            self.token = f.read().strip()
        self.base_url = "http://localhost:8303"
        self.headers = {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }

    def store(self, text, type, agent, project_id, session_id="default"):
        return requests.post(
            f"{self.base_url}/api/knowledge/store",
            headers=self.headers,
            json={
                "text": text,
                "type": type,
                "project_id": project_id,
                "session_id": session_id,
                "agent": agent
            }
        ).json()

    def search(self, query, project_id, limit=10):
        return requests.get(
            f"{self.base_url}/api/knowledge/search",
            headers=self.headers,
            params={"q": query, "project_id": project_id, "limit": limit}
        ).json()
```

### Use It

```python
from knowledge_client import KnowledgeClient

client = KnowledgeClient()

# Store
result = client.store(
    "Implemented authentication with JWT",
    "implementation",
    "python-specialist",
    "my-project"
)
print(f"Stored: {result['id']}")

# Search
results = client.search("authentication", "my-project")
for item in results['results']:
    print(f"Found: {item['text']} (similarity: {item['similarity']:.2f})")
```

---

## Bash Quick Start

### Create Wrapper Script

```bash
#!/bin/bash
# knowledge.sh

TOKEN=$(cat ~/.cco/api_token)
BASE_URL="http://localhost:8303"
PROJECT_ID=$(basename $(pwd))

store() {
    curl -X POST "$BASE_URL/api/knowledge/store" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"text\": \"$1\",
            \"type\": \"$2\",
            \"project_id\": \"$PROJECT_ID\",
            \"session_id\": \"bash\",
            \"agent\": \"$3\"
        }"
}

search() {
    curl "$BASE_URL/api/knowledge/search" \
        -H "Authorization: Bearer $TOKEN" \
        -G \
        --data-urlencode "q=$1" \
        --data-urlencode "project_id=$PROJECT_ID" \
        --data-urlencode "limit=${2:-10}"
}

case "$1" in
    store) store "$2" "$3" "$4" ;;
    search) search "$2" "$3" ;;
    *) echo "Usage: $0 {store|search} [args...]" ;;
esac
```

### Use It

```bash
chmod +x knowledge.sh

# Store knowledge
./knowledge.sh store "Implemented feature X" implementation python-specialist

# Search knowledge
./knowledge.sh search "feature implementation" 5
```

---

## Common Patterns

### Pattern 1: Store After Task

```python
# After completing a task, store what you learned
result = implement_feature()

client.store(
    f"Implemented {result.feature} using {result.tech}",
    "implementation",
    "my-agent",
    "my-project"
)
```

### Pattern 2: Search Before Implementation

```python
# Before starting, search for relevant knowledge
results = client.search("authentication patterns", "my-project")

if results['count'] > 0:
    print("Found existing patterns:")
    for item in results['results']:
        print(f"  - {item['text']}")
else:
    print("No existing patterns, implementing from scratch")
```

### Pattern 3: Track Decisions

```python
# Store important decisions
client.store(
    "Decision: Use PostgreSQL instead of MongoDB for better transaction support",
    "decision",
    "architect",
    "my-project"
)
```

---

## Knowledge Types

Use the right type for each knowledge item:

| Type | Use For | Example |
|------|---------|---------|
| `decision` | Architecture decisions | "Decided to use FastAPI" |
| `architecture` | System design | "Microservices architecture" |
| `implementation` | Code details | "Implemented JWT auth" |
| `configuration` | Settings | "Database pool: 10 connections" |
| `credential` | Credential references | "API key in vault" |
| `issue` | Problems/bugs | "Memory leak in worker" |
| `general` | Other knowledge | "User feedback noted" |

---

## Troubleshooting

### Connection Refused

```bash
# Check daemon is running
cco daemon status

# Start if not running
cco daemon start
```

### 401 Unauthorized

```bash
# Check token exists
cat ~/.cco/api_token

# Restart daemon to regenerate
cco daemon restart
```

### No Results Found

```bash
# Make sure project_id matches exactly
# Bad:  "my-project" vs "my_project"
# Good: Use same project_id for store and search
```

---

## Next Steps

1. **Read full API reference**: [KNOWLEDGE_STORE_API.md](KNOWLEDGE_STORE_API.md)
2. **Integrate in your agent**: [KNOWLEDGE_STORE_AGENT_GUIDE.md](KNOWLEDGE_STORE_AGENT_GUIDE.md)
3. **Migrate from Node.js**: [KNOWLEDGE_STORE_MIGRATION.md](KNOWLEDGE_STORE_MIGRATION.md)
4. **Understand architecture**: [KNOWLEDGE_STORE_ARCHITECTURE.md](KNOWLEDGE_STORE_ARCHITECTURE.md)

---

**Ready to use! Start storing and searching knowledge.**
