# LanceDB Knowledge Manager API Contract

## Base URL
```
http://localhost:9898/api/knowledge
```

## Authentication
Currently no authentication required (localhost only). Future enhancement: API keys for agent identification.

## Endpoints

### 1. Store Knowledge Item

Store a single knowledge item with automatic embedding generation.

**Endpoint:** `POST /api/knowledge/store`

**Request:**
```json
{
  "text": "We decided to use FastAPI for the REST API because it has automatic OpenAPI documentation and excellent async support.",
  "type": "decision",
  "project_id": "cc-orchestra",
  "session_id": "session-2025-01-01",
  "agent": "architect",
  "metadata": {
    "confidence": 0.95,
    "alternatives_considered": ["Django", "Flask", "Starlette"],
    "reasoning": "Performance and developer experience"
  }
}
```

**Response (Success):**
```json
{
  "id": "decision-1763449000123-abc7def",
  "success": true
}
```

**Response (Error):**
```json
{
  "error": "Failed to store knowledge: Database connection error",
  "success": false
}
```

**curl Example:**
```bash
curl -X POST http://localhost:9898/api/knowledge/store \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Architecture decision: Use microservices pattern",
    "type": "architecture",
    "agent": "chief-architect",
    "project_id": "my-project"
  }'
```

### 2. Batch Store Knowledge

Store multiple knowledge items in a single request for efficiency.

**Endpoint:** `POST /api/knowledge/store-batch`

**Request:**
```json
{
  "items": [
    {
      "text": "Implemented JWT authentication with RS256 algorithm",
      "type": "implementation",
      "project_id": "cc-orchestra",
      "session_id": "session-2025-01-01",
      "agent": "python-specialist",
      "metadata": {}
    },
    {
      "text": "Security audit found no critical vulnerabilities",
      "type": "issue",
      "project_id": "cc-orchestra",
      "session_id": "session-2025-01-01",
      "agent": "security-auditor",
      "metadata": {
        "severity": "low",
        "recommendations": ["Add rate limiting"]
      }
    }
  ]
}
```

**Response:**
```json
{
  "ids": [
    "implementation-1763449000456-xyz123",
    "issue-1763449000789-mno456"
  ],
  "stored": 2,
  "failed": 0,
  "success": true
}
```

### 3. Search Knowledge

Perform semantic similarity search on stored knowledge.

**Endpoint:** `POST /api/knowledge/search`

**Request:**
```json
{
  "query": "authentication JWT FastAPI security",
  "limit": 10,
  "threshold": 0.5,
  "project_id": "cc-orchestra",
  "type": "decision",
  "agent": null
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "decision-1763449000123-abc7def",
      "text": "We decided to use FastAPI for the REST API...",
      "type": "decision",
      "project_id": "cc-orchestra",
      "session_id": "session-2025-01-01",
      "agent": "architect",
      "timestamp": "2025-01-01T12:00:00Z",
      "metadata": {
        "confidence": 0.95
      },
      "score": 0.892
    },
    {
      "id": "implementation-1763449000456-xyz123",
      "text": "Implemented JWT authentication with RS256...",
      "type": "implementation",
      "project_id": "cc-orchestra",
      "session_id": "session-2025-01-01",
      "agent": "python-specialist",
      "timestamp": "2025-01-01T12:05:00Z",
      "metadata": {},
      "score": 0.756
    }
  ]
}
```

**curl Example:**
```bash
curl -X POST http://localhost:9898/api/knowledge/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "database architecture PostgreSQL",
    "limit": 5,
    "project_id": "cc-orchestra"
  }'
```

### 4. Get Project Knowledge

Retrieve all knowledge items for a specific project.

**Endpoint:** `GET /api/knowledge/project/{project_id}`

**Query Parameters:**
- `type` (optional): Filter by knowledge type
- `limit` (optional, default: 100): Maximum items to return

**Request:**
```
GET /api/knowledge/project/cc-orchestra?type=decision&limit=50
```

**Response:**
```json
{
  "items": [
    {
      "id": "decision-1763449000123-abc7def",
      "text": "We decided to use FastAPI...",
      "type": "decision",
      "project_id": "cc-orchestra",
      "session_id": "session-2025-01-01",
      "agent": "architect",
      "timestamp": "2025-01-01T12:00:00Z",
      "metadata": {}
    }
  ],
  "count": 1
}
```

### 5. Pre-Compaction Hook

Extract and store critical knowledge before conversation compaction.

**Endpoint:** `POST /api/knowledge/pre-compaction`

**Request:**
```json
{
  "conversation": "Chief Architect: We need to design a microservices architecture...\n\nPython Specialist: I'll implement the API gateway using FastAPI...\n\nSecurity Auditor: Ensure all services use mTLS for communication...",
  "context": {
    "project_id": "cc-orchestra",
    "session_id": "session-2025-01-01"
  }
}
```

**Response:**
```json
{
  "success": true,
  "count": 5,
  "ids": [
    "architecture-1763449001000-aaa111",
    "decision-1763449001001-bbb222",
    "implementation-1763449001002-ccc333",
    "configuration-1763449001003-ddd444",
    "issue-1763449001004-eee555"
  ]
}
```

### 6. Post-Compaction Hook

Retrieve relevant context after conversation compaction.

**Endpoint:** `POST /api/knowledge/post-compaction`

**Request:**
```json
{
  "current_task": "Implementing user authentication with JWT tokens",
  "context": {
    "project_id": "cc-orchestra",
    "limit": 10
  }
}
```

**Response:**
```json
{
  "search_results": [
    {
      "id": "decision-123",
      "text": "Use JWT with RS256...",
      "type": "decision",
      "score": 0.89
    }
  ],
  "recent_knowledge": [
    {
      "id": "implementation-456",
      "text": "Created auth endpoints...",
      "type": "implementation",
      "timestamp": "2025-01-01T14:00:00Z"
    }
  ],
  "summary": {
    "total_items": 15,
    "by_type": {
      "decision": 5,
      "architecture": 3,
      "implementation": 4,
      "issue": 3
    },
    "by_agent": {
      "architect": 8,
      "python-specialist": 7
    },
    "top_decisions": [
      "Use FastAPI for REST API...",
      "Implement JWT authentication...",
      "Deploy with Docker..."
    ],
    "recent_activity": [
      {
        "type": "implementation",
        "agent": "python-specialist",
        "timestamp": "2025-01-01T14:00:00Z",
        "preview": "Created auth endpoints with JWT validation..."
      }
    ]
  }
}
```

### 7. Get Statistics

Get comprehensive statistics about the knowledge base.

**Endpoint:** `GET /api/knowledge/stats`

**Response:**
```json
{
  "repository": "cc-orchestra",
  "total_records": 247,
  "by_type": {
    "decision": 45,
    "architecture": 38,
    "implementation": 89,
    "configuration": 23,
    "credential": 12,
    "issue": 28,
    "general": 12
  },
  "by_agent": {
    "chief-architect": 83,
    "python-specialist": 67,
    "security-auditor": 34,
    "qa-engineer": 29,
    "devops-engineer": 34
  },
  "by_project": {
    "cc-orchestra": 180,
    "test-project": 67
  },
  "oldest_record": "2024-12-01T08:00:00Z",
  "newest_record": "2025-01-01T15:30:00Z"
}
```

### 8. Cleanup Old Knowledge

Remove knowledge items older than specified days.

**Endpoint:** `POST /api/knowledge/cleanup`

**Request:**
```json
{
  "older_than_days": 90,
  "project_id": "cc-orchestra"
}
```

**Response:**
```json
{
  "count": 42,
  "success": true,
  "message": "Removed 42 knowledge items older than 90 days"
}
```

## Error Responses

All endpoints may return these standard error responses:

### 400 Bad Request
```json
{
  "error": "Invalid request: missing required field 'text'",
  "success": false
}
```

### 500 Internal Server Error
```json
{
  "error": "Database connection failed",
  "success": false
}
```

### 503 Service Unavailable
```json
{
  "error": "Knowledge base is initializing, please try again",
  "success": false
}
```

## Rate Limiting

- Maximum 100 requests per second per IP
- Batch operations count as single request
- Uses existing ConnectionTracker from CCO

## Request/Response Headers

**Required Request Headers:**
```
Content-Type: application/json
```

**Response Headers:**
```
Content-Type: application/json
X-Knowledge-Version: 1.0.0
```

## WebSocket Support (Future)

Future enhancement for real-time knowledge updates:
```
ws://localhost:9898/api/knowledge/stream
```

## Agent Integration Examples

### JavaScript/Node.js Agent
```javascript
const axios = require('axios');

class KnowledgeClient {
  constructor(baseUrl = 'http://localhost:9898') {
    this.baseUrl = baseUrl;
  }

  async store(text, type, agent, metadata = {}) {
    const response = await axios.post(
      `${this.baseUrl}/api/knowledge/store`,
      {
        text,
        type,
        agent,
        project_id: process.env.PROJECT_ID || 'default',
        session_id: process.env.SESSION_ID || 'unknown',
        metadata
      }
    );
    return response.data.id;
  }

  async search(query, options = {}) {
    const response = await axios.post(
      `${this.baseUrl}/api/knowledge/search`,
      { query, ...options }
    );
    return response.data.results;
  }
}
```

### Python Agent
```python
import requests
import os

class KnowledgeClient:
    def __init__(self, base_url='http://localhost:9898'):
        self.base_url = base_url

    def store(self, text, type='general', agent='unknown', metadata=None):
        response = requests.post(
            f'{self.base_url}/api/knowledge/store',
            json={
                'text': text,
                'type': type,
                'agent': agent,
                'project_id': os.getenv('PROJECT_ID', 'default'),
                'session_id': os.getenv('SESSION_ID', 'unknown'),
                'metadata': metadata or {}
            }
        )
        return response.json()['id']

    def search(self, query, **options):
        response = requests.post(
            f'{self.base_url}/api/knowledge/search',
            json={'query': query, **options}
        )
        return response.json()['results']
```

## Performance Benchmarks

Expected performance characteristics:

| Operation | Target Latency | Throughput |
|-----------|---------------|------------|
| Store Single | < 10ms | 1000/sec |
| Store Batch (100) | < 100ms | 100/sec |
| Search (10 results) | < 50ms | 500/sec |
| Get Project Knowledge | < 100ms | 100/sec |
| Pre-compaction | < 500ms | 10/sec |
| Post-compaction | < 200ms | 50/sec |
| Stats | < 50ms | 100/sec |
| Cleanup | < 1000ms | 1/sec |

## Monitoring Endpoints

### Health Check
```
GET /health
```

Response includes knowledge store status:
```json
{
  "status": "ok",
  "knowledge_store": {
    "initialized": true,
    "database_connected": true,
    "total_records": 247
  }
}
```

## Migration from Node.js

For agents currently using the Node.js file-based API:

**Before (File-based):**
```javascript
node ~/git/cc-orchestra/src/knowledge-manager.js store "text" --type decision --agent architect
```

**After (HTTP API):**
```javascript
curl -X POST http://localhost:9898/api/knowledge/store \
  -H "Content-Type: application/json" \
  -d '{"text": "text", "type": "decision", "agent": "architect"}'
```

## Version History

- **v1.0.0** (2025-01): Initial implementation with 8 endpoints
- **v1.1.0** (Future): WebSocket streaming support
- **v1.2.0** (Future): Authentication and rate limiting
- **v2.0.0** (Future): Real embedding models instead of SHA256