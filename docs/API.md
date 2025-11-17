# CCO API Documentation

Complete API reference for CCO endpoints, including model override statistics and analytics.

## Base URL

```
http://localhost:3000
```

## Authentication

No authentication required. In production, implement authentication/authorization as needed.

## Health Check Endpoint

### GET /health

Check if CCO is running and get basic status information.

**Request:**
```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "ok",
  "uptime": 3600,
  "overrides_enabled": true,
  "override_rules": 3,
  "cache_hits": 1234,
  "cache_misses": 567
}
```

**Status Codes:**
- `200 OK` - CCO is healthy
- `503 Service Unavailable` - CCO is unavailable

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Always "ok" if service is up |
| `uptime` | number | Seconds since CCO started |
| `overrides_enabled` | boolean | Whether model overrides are active |
| `override_rules` | number | Number of configured override rules |
| `cache_hits` | number | Total cache hits since startup |
| `cache_misses` | number | Total cache misses since startup |

## Model Override Statistics Endpoint

### GET /api/overrides/stats

Get current model override statistics, showing which models have been overridden and how many times.

**Request:**
```bash
curl http://localhost:3000/api/overrides/stats
```

**Response (with overrides active):**
```json
{
  "total_overrides": 47,
  "overrides_by_model": {
    "claude-sonnet-4.5-20250929": {
      "overridden_to": "claude-haiku-4-5-20251001",
      "count": 47,
      "percentage": 100,
      "cost_saved": "$18.50"
    }
  }
}
```

**Response (with no overrides active):**
```json
{
  "total_overrides": 0,
  "overrides_enabled": false,
  "message": "Model overrides are not enabled"
}
```

**Status Codes:**
- `200 OK` - Statistics retrieved successfully

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `total_overrides` | number | Total model overrides applied since startup |
| `overrides_enabled` | boolean | Whether overrides are globally enabled |
| `overrides_by_model` | object | Breakdown of overrides by original model |

**Per-Model Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `overridden_to` | string | The replacement model name |
| `count` | number | How many times this override was applied |
| `percentage` | number | Percentage of all overrides |
| `cost_saved` | string | Estimated cost saved (informational) |

## Cache Statistics Endpoint

### GET /api/cache/stats

Get cache performance metrics.

**Request:**
```bash
curl http://localhost:3000/api/cache/stats
```

**Response:**
```json
{
  "cache_hits": 1234,
  "cache_misses": 567,
  "hit_rate": 0.685,
  "total_requests": 1801,
  "memory_usage_bytes": 52428800,
  "cached_items": 342
}
```

**Status Codes:**
- `200 OK` - Cache stats retrieved successfully

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `cache_hits` | number | Number of successful cache hits |
| `cache_misses` | number | Number of cache misses |
| `hit_rate` | number | Cache hit rate (0-1, where 1 = 100%) |
| `total_requests` | number | Total requests processed |
| `memory_usage_bytes` | number | Memory used by cache (bytes) |
| `cached_items` | number | Number of items in cache |

## Machine-Wide Analytics Endpoint

### GET /api/machine/stats

Get comprehensive analytics across all projects and models.

**Request:**
```bash
curl http://localhost:3000/api/machine/stats | jq
```

**Response:**
```json
{
  "totalCost": 1234.56,
  "activeProjects": 7,
  "totalCalls": 45678,
  "totalTokens": 12345678,
  "projects": [
    {
      "name": "Project A",
      "calls": 1234,
      "inputTokens": 50000,
      "outputTokens": 30000,
      "cost": 123.45,
      "lastActivity": "2024-11-15T10:30:00Z"
    }
  ],
  "models": [
    {
      "name": "claude-opus-4.1",
      "calls": 500,
      "inputTokens": 100000,
      "outputTokens": 50000,
      "cost": 456.78
    }
  ],
  "chartData": {
    "costOverTime": [
      {
        "date": "2024-11-15T00:00:00Z",
        "cost": 45.23
      }
    ],
    "costByProject": [
      {
        "project": "Project A",
        "cost": 456.78
      }
    ],
    "modelDistribution": [
      {
        "model": "claude-opus-4.1",
        "count": 500
      }
    ]
  }
}
```

**Status Codes:**
- `200 OK` - Analytics retrieved successfully

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `totalCost` | number | Total cost across all projects |
| `activeProjects` | number | Number of active projects |
| `totalCalls` | number | Total API calls made |
| `totalTokens` | number | Total tokens consumed |
| `projects` | array | Per-project breakdown |
| `models` | array | Per-model breakdown |
| `chartData` | object | Formatted data for charts |

## Project-Specific Analytics Endpoint

### GET /api/project/stats

Get analytics for the current project.

**Request:**
```bash
curl http://localhost:3000/api/project/stats
```

**Response:**
```json
{
  "cost": 45.67,
  "costTrend": {
    "value": 5.2,
    "period": "24h"
  },
  "tokens": 123456,
  "tokensTrend": {
    "value": -2.1,
    "period": "24h"
  },
  "calls": 89,
  "callsTrend": {
    "value": 12.3,
    "period": "24h"
  },
  "avgTime": 245,
  "timeTrend": {
    "value": -8.5,
    "period": "24h"
  }
}
```

**Status Codes:**
- `200 OK` - Project stats retrieved successfully

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `cost` | number | Total cost for this project |
| `costTrend` | object | Cost trend (value=percent change, period=time window) |
| `tokens` | number | Total tokens used |
| `tokensTrend` | object | Token trend |
| `calls` | number | Total API calls |
| `callsTrend` | object | Call trend |
| `avgTime` | number | Average response time (ms) |
| `timeTrend` | object | Response time trend |

## Streaming Endpoint

### GET /api/stream

Server-Sent Events (SSE) stream for real-time analytics updates.

**Request:**
```bash
curl http://localhost:3000/api/stream
```

**Response Format (Server-Sent Events):**
```
event: analytics
data: {"type":"stats","payload":{...}}

event: model_override
data: {"model":"claude-sonnet-4.5","override_to":"claude-haiku-4-5"}

event: cache_hit
data: {"cache_key":"hash_value"}
```

**Event Types:**

| Event | Data | Description |
|-------|------|-------------|
| `analytics` | Full stats object | Periodic analytics update |
| `model_override` | Override details | When a model override occurs |
| `cache_hit` | Cache key | When cache is hit |
| `cache_miss` | Request details | When cache is missed |
| `error` | Error message | When an error occurs |

**Usage Example (JavaScript):**
```javascript
const stream = new EventSource('http://localhost:3000/api/stream');

stream.addEventListener('model_override', (event) => {
  const data = JSON.parse(event.data);
  console.log(`Override: ${data.model} → ${data.override_to}`);
});

stream.addEventListener('error', (event) => {
  console.error('Stream error:', event);
  stream.close();
});
```

## WebSocket Terminal Endpoint

### WS /terminal

WebSocket connection for interactive terminal commands.

**Request:**
```bash
# Via websocat or similar tool
websocat ws://localhost:3000/terminal

# Or use JavaScript
const ws = new WebSocket('ws://localhost:3000/terminal');
ws.onmessage = (event) => console.log(event.data);
ws.send('help');
```

**Message Format:**

**Request (Client → Server):**
```json
{
  "type": "command",
  "command": "cache stats"
}
```

**Response (Server → Client):**
```json
{
  "type": "output",
  "content": "Cache hits: 1234\nCache misses: 567"
}
```

**Available Commands:**
- `help` - Show available commands
- `cache stats` - Show cache statistics
- `cache clear` - Clear cache
- `stats` - Show current statistics
- `overrides` - Show override statistics
- `exit` - Close connection

## Error Responses

All endpoints return consistent error responses.

**Error Response Format:**
```json
{
  "error": "Error description",
  "code": "ERROR_CODE",
  "timestamp": "2024-11-15T10:30:00Z",
  "details": {}
}
```

**Common Error Codes:**

| Code | Status | Description |
|------|--------|-------------|
| `INVALID_REQUEST` | 400 | Request is malformed |
| `NOT_FOUND` | 404 | Resource not found |
| `INTERNAL_ERROR` | 500 | Internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

**Example Error Response:**
```json
{
  "error": "Invalid query parameter",
  "code": "INVALID_REQUEST",
  "timestamp": "2024-11-15T10:30:00Z",
  "details": {
    "parameter": "limit",
    "reason": "must be a positive integer"
  }
}
```

## Rate Limiting

No rate limiting by default. In production, implement rate limiting as needed:

```
Recommended: 100 requests/second per client IP
Burst limit: 500 requests/second
```

## CORS Headers

CORS is disabled by default. Enable if needed for cross-origin requests:

```bash
# Example response headers
Access-Control-Allow-Origin: http://localhost:3000
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

## Example Workflows

### Workflow 1: Monitor Override Activity

Monitor model overrides in real-time:

```bash
# Shell script to monitor overrides
while true; do
  clear
  echo "Model Override Statistics"
  echo "======================="
  curl -s http://localhost:3000/api/overrides/stats | jq
  echo ""
  echo "Last updated: $(date)"
  sleep 5
done
```

### Workflow 2: Track Cost Savings

Calculate cost savings from overrides:

```bash
#!/bin/bash
# Get current statistics
STATS=$(curl -s http://localhost:3000/api/overrides/stats)

# Extract override count
OVERRIDES=$(echo $STATS | jq '.total_overrides')
SAVED=$(echo $STATS | jq -r '.overrides_by_model[] | .cost_saved' | head -1)

echo "Total overrides: $OVERRIDES"
echo "Estimated savings: $SAVED"
```

### Workflow 3: Dashboard Integration

Integrate CCO stats into your monitoring system:

```python
import requests
import json
from datetime import datetime

def get_override_stats():
    """Fetch override statistics from CCO."""
    response = requests.get('http://localhost:3000/api/overrides/stats')
    if response.status_code == 200:
        return response.json()
    return None

def get_health():
    """Check CCO health."""
    response = requests.get('http://localhost:3000/health')
    if response.status_code == 200:
        return response.json()
    return None

# Usage
stats = get_override_stats()
health = get_health()

print(f"CCO Status: {health['status']}")
print(f"Overrides Enabled: {health['overrides_enabled']}")
print(f"Total Overrides: {stats['total_overrides']}")
```

## API Versioning

Current API version: `v1`

All endpoints are backward compatible. Future breaking changes will use versioned paths (e.g., `/api/v2/...`).

## Performance Considerations

**Latency:**
- Health check: < 1ms
- Stats endpoints: < 5ms
- Stream events: < 100ms

**Throughput:**
- Supports 1000+ requests/second
- Concurrent connections: 500+

**Caching:**
- API responses are not cached by default
- Implement client-side caching for `/api/stats` endpoints (5-10 second TTL)

## Security

**In Production:**
1. Require authentication for all endpoints
2. Implement rate limiting (100 req/sec per client)
3. Use HTTPS instead of HTTP
4. Validate all query parameters
5. Never expose sensitive data (API keys)
6. Log all API access

**Example Authentication (Bearer Token):**
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:3000/api/overrides/stats
```

## Troubleshooting

### Endpoint Returns 404

**Problem:** API endpoint not found.

**Solution:**
```bash
# Verify CCO is running
curl http://localhost:3000/health

# Check correct URL
curl http://localhost:3000/api/overrides/stats
```

### Endpoint Returns 500

**Problem:** Internal server error.

**Solution:**
```bash
# Check logs
journalctl -u cco -f

# Restart CCO
sudo systemctl restart cco

# Verify health
curl http://localhost:3000/health
```

### Streaming Connection Drops

**Problem:** SSE stream closes unexpectedly.

**Solution:**
```bash
# Client should implement reconnection logic
# Server timeout: typically 60 seconds of inactivity
# Implement heartbeat: send ":" as keepalive every 30 seconds
```

## Related Documentation

1. **[User Guide](./MODEL_OVERRIDE_USER_GUIDE.md)** - How to use model overrides
2. **[Operator Guide](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)** - Deployment and management
3. **[Configuration Reference](./MODEL_OVERRIDE_CONFIG_REFERENCE.md)** - Configuration options

---

**API Ready for Integration?** Use the examples above to integrate CCO into your monitoring and analytics systems.
