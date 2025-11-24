# Agent Definitions API Reference

**Version**: 2.0.0
**API Version**: v1
**Last Updated**: November 15, 2025
**Status**: Stable

## Overview

The Agent Definitions API provides HTTP endpoints for discovering, querying, and managing agent configurations in CCO (Claude Code Orchestrator). This API is consumed by Claude Code to dynamically load agent definitions without hardcoding them.

## Base URL

```
http://localhost:8000
```

Configurable via `CCO_API_URL` environment variable.

## Common Response Format

All successful responses follow this format:

```json
{
  "data": {...},
  "meta": {
    "timestamp": "2025-11-15T19:30:00Z",
    "version": "2025.11.1",
    "status": "success"
  }
}
```

## Endpoints

### 1. Health Check

Check if CCO server is running and healthy.

**Request**:
```http
GET /health HTTP/1.1
Host: localhost:8000
Content-Type: application/json
```

**Response** (200 OK):
```json
{
  "status": "ok",
  "version": "2025.11.1",
  "uptime": 3600,
  "cache_stats": {
    "hit_rate": 0.75,
    "hits": 150,
    "misses": 50,
    "entries": 45
  }
}
```

**Status Codes**:
- `200 OK` - Server is healthy
- `503 Service Unavailable` - Server is starting up or unhealthy

---

### 2. List All Agents

Retrieve definitions for all available agents.

**Request**:
```http
GET /api/agents HTTP/1.1
Host: localhost:8000
Accept: application/json
```

**Optional Query Parameters**:

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `model` | string | Filter by model tier (opus, sonnet, haiku) | `?model=sonnet` |
| `category` | string | Filter by agent category | `?category=development` |
| `capability` | string | Filter by capability | `?capability=security` |
| `limit` | integer | Max results (default: 100) | `?limit=50` |
| `offset` | integer | Pagination offset (default: 0) | `?offset=20` |

**Response** (200 OK):
```json
{
  "agents": [
    {
      "name": "Chief Architect",
      "type": "chief-architect",
      "model": "opus",
      "role": "Strategic decision-making and project guidance",
      "category": "leadership",
      "capabilities": [
        "System design",
        "Architecture decisions",
        "Agent coordination",
        "Requirement analysis"
      ],
      "specialties": [
        "Microservices",
        "Cloud architecture",
        "API design"
      ],
      "autonomousAuthority": {
        "lowRisk": true,
        "mediumRisk": true,
        "highRisk": false,
        "requiresDocumentation": true
      }
    },
    {
      "name": "Python Specialist",
      "type": "python-specialist",
      "model": "haiku",
      "role": "Python implementation specialist",
      "category": "development",
      "capabilities": [
        "FastAPI/Flask development",
        "Django ORM",
        "Async/await patterns",
        "ML/AI integration"
      ],
      "specialties": [
        "python",
        "fastapi",
        "django"
      ],
      "autonomousAuthority": {
        "lowRisk": true,
        "mediumRisk": false,
        "requiresArchitectApproval": true
      }
    }
  ],
  "pagination": {
    "total": 119,
    "limit": 100,
    "offset": 0,
    "returned": 100
  },
  "meta": {
    "timestamp": "2025-11-15T19:30:45Z",
    "version": "2025.11.1"
  }
}
```

**Status Codes**:
- `200 OK` - Success
- `400 Bad Request` - Invalid query parameters
- `500 Internal Server Error` - Server error

**Example Requests**:

```bash
# Get all agents
curl http://localhost:8000/api/agents

# Get only Sonnet agents
curl http://localhost:8000/api/agents?model=sonnet

# Get development agents
curl http://localhost:8000/api/agents?category=development

# Paginate results
curl "http://localhost:8000/api/agents?limit=20&offset=20"

# Filter by capability
curl "http://localhost:8000/api/agents?capability=testing"
```

---

### 3. Get Specific Agent

Retrieve definition for a single agent.

**Request**:
```http
GET /api/agents/{agent-type} HTTP/1.1
Host: localhost:8000
Accept: application/json
```

**Path Parameters**:

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `agent-type` | string | Yes | Agent type identifier | `python-specialist` |

**Response** (200 OK):
```json
{
  "agent": {
    "name": "Python Specialist",
    "type": "python-specialist",
    "model": "haiku",
    "role": "Python implementation specialist",
    "description": "Expert Python developer specializing in FastAPI, Django, and modern async patterns",
    "category": "development",
    "languages": ["python"],
    "frameworks": ["FastAPI", "Django"],
    "capabilities": [
      "FastAPI/Flask REST API development",
      "Django ORM and models",
      "Async/await pattern implementation",
      "Data processing with Pandas",
      "ML/AI integration",
      "Error handling and logging"
    ],
    "specialties": [
      "python",
      "fastapi",
      "django",
      "asyncio",
      "data-processing",
      "ml-integration"
    ],
    "responsibilities": [
      "Implement Python backend services",
      "Create REST APIs with FastAPI",
      "Write async-first Python code",
      "Handle data processing tasks",
      "Integrate ML models"
    ],
    "autonomousAuthority": {
      "lowRisk": true,
      "mediumRisk": false,
      "highRisk": false,
      "requiresArchitectApproval": true,
      "requiresDocumentation": false
    },
    "agentFile": "~/.claude/agents/python-specialist.md",
    "tags": ["backend", "python", "api"]
  },
  "meta": {
    "timestamp": "2025-11-15T19:31:00Z",
    "version": "2025.11.1"
  }
}
```

**Error Response** (404 Not Found):
```json
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent type 'unknown-agent' not found",
    "type": "unknown-agent"
  },
  "meta": {
    "timestamp": "2025-11-15T19:31:05Z",
    "version": "2025.11.1"
  }
}
```

**Status Codes**:
- `200 OK` - Agent found
- `404 Not Found` - Agent type doesn't exist
- `500 Internal Server Error` - Server error

**Example Requests**:

```bash
# Get Python specialist
curl http://localhost:8000/api/agents/python-specialist

# Get Go specialist
curl http://localhost:8000/api/agents/go-specialist

# Get security auditor
curl http://localhost:8000/api/agents/security-auditor

# Parse with jq
curl -s http://localhost:8000/api/agents/python-specialist | jq '.agent.capabilities'
```

---

### 4. Get Agent by Category

Retrieve all agents in a specific category.

**Request**:
```http
GET /api/agents/category/{category-name} HTTP/1.1
Host: localhost:8000
Accept: application/json
```

**Path Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `category-name` | string | Category name (development, infrastructure, security, etc.) |

**Available Categories**:
- `leadership` - Chief Architect
- `coding` - Language specialists
- `integration` - API integration agents
- `development` - Frontend/backend developers
- `data` - Data and database agents
- `infrastructure` - DevOps and cloud agents
- `security` - Security and compliance agents
- `ai-ml` - AI and ML agents
- `mcp` - Model Context Protocol agents
- `documentation` - Documentation agents
- `research` - Research agents
- `support` - Support and utility agents
- `business` - Business agents

**Response** (200 OK):
```json
{
  "category": "development",
  "agents": [
    {
      "name": "Frontend Developer",
      "type": "frontend-developer",
      "model": "haiku",
      "role": "Frontend development specialist"
    },
    {
      "name": "Backend Architect",
      "type": "backend-architect",
      "model": "sonnet",
      "role": "Backend system architecture"
    }
  ],
  "count": 12,
  "meta": {
    "timestamp": "2025-11-15T19:31:30Z",
    "version": "2025.11.1"
  }
}
```

**Example Requests**:

```bash
# Get all development agents
curl http://localhost:8000/api/agents/category/development

# Get all security agents
curl http://localhost:8000/api/agents/category/security

# Get all infrastructure agents
curl http://localhost:8000/api/agents/category/infrastructure
```

---

### 5. Get Agents by Model

Retrieve all agents using a specific model tier.

**Request**:
```http
GET /api/agents/model/{model-name} HTTP/1.1
Host: localhost:8000
Accept: application/json
```

**Path Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `model-name` | string | Model tier: `opus`, `sonnet`, or `haiku` |

**Model Information**:

| Model | Count | Cost | Use Case |
|-------|-------|------|----------|
| `opus` | 1 | $15/M input, $75/M output | Strategic decisions, architecture |
| `sonnet` | ~37 | $3/M input, $15/M output | Complex coding, code review |
| `haiku` | ~81 | $0.80/M input, $4/M output | Basic coding, documentation |

**Response** (200 OK):
```json
{
  "model": "sonnet",
  "agents": [
    {
      "name": "API Explorer",
      "type": "api-explorer",
      "model": "sonnet",
      "role": "API exploration and integration analysis"
    },
    {
      "name": "Backend Architect",
      "type": "backend-architect",
      "model": "sonnet",
      "role": "Backend system architecture"
    }
  ],
  "count": 37,
  "meta": {
    "timestamp": "2025-11-15T19:31:45Z",
    "version": "2025.11.1"
  }
}
```

**Example Requests**:

```bash
# Get Opus agents (strategic)
curl http://localhost:8000/api/agents/model/opus

# Get Sonnet agents (intelligent)
curl http://localhost:8000/api/agents/model/sonnet

# Get Haiku agents (basic)
curl http://localhost:8000/api/agents/model/haiku
```

---

### 6. Override Agent Model Assignment

Change the model assigned to an agent at runtime (for development/testing).

**Request**:
```http
POST /api/models/override HTTP/1.1
Host: localhost:8000
Content-Type: application/json

{
  "agent-type": "model-name",
  "chief-architect": "sonnet",
  "python-specialist": "sonnet"
}
```

**Request Body**:
```json
{
  "python-specialist": "sonnet",
  "go-specialist": "sonnet",
  "test-engineer": "opus"
}
```

**Response** (200 OK):
```json
{
  "overrides": {
    "python-specialist": "sonnet",
    "go-specialist": "sonnet",
    "test-engineer": "opus"
  },
  "appliedAt": "2025-11-15T19:32:00Z",
  "meta": {
    "timestamp": "2025-11-15T19:32:00Z",
    "version": "2025.11.1"
  }
}
```

**Error Response** (400 Bad Request):
```json
{
  "error": {
    "code": "INVALID_MODEL",
    "message": "Model 'invalid-model' not recognized. Use: opus, sonnet, haiku",
    "validModels": ["opus", "sonnet", "haiku"]
  },
  "meta": {
    "timestamp": "2025-11-15T19:32:05Z",
    "version": "2025.11.1"
  }
}
```

**Status Codes**:
- `200 OK` - Overrides applied successfully
- `400 Bad Request` - Invalid model or agent type
- `500 Internal Server Error` - Server error

**Example Requests**:

```bash
# Upgrade Python specialist to Sonnet
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{"python-specialist": "sonnet"}'

# Override multiple agents
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{
    "python-specialist": "sonnet",
    "go-specialist": "sonnet",
    "test-engineer": "sonnet"
  }'

# Reset to defaults (use empty object)
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

### 7. Get Current Model Overrides

Retrieve all currently active model overrides.

**Request**:
```http
GET /api/models/override HTTP/1.1
Host: localhost:8000
Accept: application/json
```

**Response** (200 OK):
```json
{
  "overrides": {
    "python-specialist": "sonnet",
    "go-specialist": "sonnet",
    "test-engineer": "sonnet"
  },
  "totalOverrides": 3,
  "meta": {
    "timestamp": "2025-11-15T19:32:30Z",
    "version": "2025.11.1"
  }
}
```

**Example Requests**:

```bash
# Get current overrides
curl http://localhost:8000/api/models/override

# Check if specific agent is overridden
curl -s http://localhost:8000/api/models/override | jq '.overrides."python-specialist"'
```

---

## Error Handling

### Error Response Format

All error responses include consistent error information:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "Additional context"
    }
  },
  "meta": {
    "timestamp": "2025-11-15T19:33:00Z",
    "version": "2025.11.1"
  }
}
```

### Common Error Codes

| Code | Status | Description |
|------|--------|-------------|
| `AGENT_NOT_FOUND` | 404 | Requested agent type doesn't exist |
| `INVALID_PARAMETER` | 400 | Invalid query parameter |
| `INVALID_MODEL` | 400 | Invalid model assignment |
| `SERVER_ERROR` | 500 | Internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Server starting up or overloaded |

### Error Example

```bash
$ curl http://localhost:8000/api/agents/nonexistent

{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent type 'nonexistent' not found"
  }
}
```

---

## Request/Response Examples

### Example 1: Discovery Workflow

```bash
# 1. Check server health
curl http://localhost:8000/health

# 2. Get all agents
curl http://localhost:8000/api/agents | jq '.agents | length'

# 3. Get specific agent
curl http://localhost:8000/api/agents/python-specialist | jq '.agent | {name, model, capabilities}'

# 4. Check overrides
curl http://localhost:8000/api/models/override | jq '.overrides'
```

### Example 2: Filter and Query

```bash
# Get all Sonnet agents
curl "http://localhost:8000/api/agents?model=sonnet" | jq '.agents | length'

# Get development agents only
curl "http://localhost:8000/api/agents/category/development" | jq '.agents | map(.name)'

# Get agents by capability
curl "http://localhost:8000/api/agents?capability=testing" | jq '.agents'
```

### Example 3: Override Management

```bash
# View current overrides
curl http://localhost:8000/api/models/override

# Apply overrides for testing
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{"test-engineer": "sonnet"}'

# Verify override was applied
curl "http://localhost:8000/api/agents/test-engineer" | jq '.agent.model'

# Reset overrides
curl -X POST http://localhost:8000/api/models/override \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

## Rate Limiting

Currently, there is no rate limiting on the API. Future versions may implement:

- Per-IP rate limits (requests/second)
- Token-based authentication
- Cost-aware rate limiting

---

## Versioning

API versioning follows semantic versioning:

- **Current Version**: 1.0.0
- **API Endpoint**: `/api/v1/agents` (v1 implied in `/api/agents`)

All responses include version information in metadata:
```json
{
  "meta": {
    "version": "2025.11.1",
    "apiVersion": "1.0.0"
  }
}
```

---

## Best Practices

### 1. Cache Agent Definitions Locally

```javascript
// Cache after fetching
const agents = await fetch('/api/agents').then(r => r.json());
localStorage.setItem('agents', JSON.stringify(agents));

// Use cache if fetch fails
try {
  return await fetch('/api/agents');
} catch {
  return JSON.parse(localStorage.getItem('agents'));
}
```

### 2. Implement Retry Logic

```javascript
async function fetchWithRetry(url, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fetch(url, { timeout: 5000 });
    } catch (e) {
      if (i === maxRetries - 1) throw e;
      await new Promise(r => setTimeout(r, 1000 * (i + 1)));
    }
  }
}
```

### 3. Validate Agent Definitions

```javascript
function validateAgent(agent) {
  const required = ['name', 'type', 'model', 'role'];
  return required.every(field => field in agent);
}
```

### 4. Handle Model Fallback

```javascript
function getEffectiveModel(agent, overrides) {
  return overrides[agent.type] || agent.model;
}
```

---

## Related Documentation

- `/Users/brent/.claude/AGENT_DEFINITIONS_ARCHITECTURE.md` - System architecture
- `/Users/brent/git/cc-orchestra/config/orchestra-config.json` - Agent definitions
- `/Users/brent/git/cc-orchestra/cco/src/server.rs` - Implementation
- `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md` - Agent delegation rules

---

## Support and Issues

For issues with the Agent Definitions API:

1. **Check CCO status**: `curl http://localhost:8000/health`
2. **Review logs**: `tail -f ~/.local/share/cco/logs/cco-8000.log`
3. **Verify configuration**: Check `/Users/brent/git/cc-orchestra/config/orchestra-config.json`
4. **Rebuild if needed**: `cd cco && cargo build --release`
