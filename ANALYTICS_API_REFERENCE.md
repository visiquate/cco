# CCO Analytics API Reference

## New Endpoints

### 1. Per-Project Metrics Endpoint
**Route:** `GET /api/metrics/projects`

**Description:** Returns current metrics aggregated by project

**Response:**
```json
{
  "projects": {
    "Claude Orchestra": {
      "cost": 123.45,
      "tokens": 150000,
      "calls": 42,
      "last_updated": "2025-11-15T21:35:00Z"
    }
  }
}
```

**Fields:**
- `cost` (float): Total actual cost in dollars
- `tokens` (integer): Total would-be cost (as proxy for tokens)
- `calls` (integer): Total number of API calls/requests
- `last_updated` (string): ISO 8601 timestamp of last update

**Example:**
```bash
curl http://localhost:8080/api/metrics/projects | jq .
```

---

## Updated Endpoints

### 2. Server-Sent Events Stream (Updated Format)
**Route:** `GET /api/stream`

**Description:** Real-time analytics stream with project, machine, and activity data

**Response (every 5 seconds):**
```json
{
  "project": {
    "name": "Claude Orchestra",
    "cost": 123.45,
    "tokens": 150000,
    "calls": 42,
    "last_updated": "2025-11-15T21:35:00Z"
  },
  "machine": {
    "cpu": "N/A",
    "memory": "N/A",
    "uptime": 3600,
    "process_count": 5
  },
  "activity": [
    {
      "timestamp": "2025-11-15T21:35:00Z",
      "event_type": "api_call",
      "agent_name": null,
      "model": "claude-opus-4",
      "tokens": 1500,
      "latency_ms": 245
    },
    {
      "timestamp": "2025-11-15T21:34:55Z",
      "event_type": "cache_hit",
      "agent_name": null,
      "model": "claude-sonnet-4.5",
      "tokens": 800,
      "latency_ms": 12
    }
  ]
}
```

**Fields:**

#### project
- `name` (string): Project name ("Claude Orchestra")
- `cost` (float): Total actual cost in dollars
- `tokens` (integer): Total would-be cost as token count
- `calls` (integer): Total number of requests
- `last_updated` (string): ISO 8601 timestamp

#### machine
- `cpu` (string): CPU usage ("N/A" for now)
- `memory` (string): Memory usage ("N/A" for now)
- `uptime` (integer): Server uptime in seconds
- `process_count` (integer): Estimated number of active agents

#### activity (array)
Array of recent events (last 20), each containing:
- `timestamp` (string): ISO 8601 timestamp
- `event_type` (string): Type of event:
  - `"api_call"` - API request made
  - `"cache_hit"` - Response from cache
  - `"cache_miss"` - Cache miss (tracked implicitly)
  - `"model_override"` - Model was rewritten
  - `"error"` - Error occurred
- `agent_name` (string|null): Agent making request (if detected)
- `model` (string|null): Model used
- `tokens` (integer|null): Total tokens (input + output)
- `latency_ms` (integer|null): Request latency in milliseconds

**Example with curl:**
```bash
# Connect to SSE stream
curl -N http://localhost:8080/api/stream

# With timeout and count:
timeout 30 curl -N http://localhost:8080/api/stream | head -10
```

**Example with JavaScript:**
```javascript
const eventSource = new EventSource('http://localhost:8080/api/stream');

eventSource.addEventListener('analytics', (event) => {
  const data = JSON.parse(event.data);
  console.log('Project cost:', data.project.cost);
  console.log('Last 20 activities:', data.activity);
});

eventSource.onerror = (error) => {
  console.error('Stream error:', error);
  eventSource.close();
};
```

---

## Activity Event Types

### api_call
- Recorded when: API call made to external service
- Includes: model, tokens, latency_ms
- Example: Request to Claude Opus API

### cache_hit
- Recorded when: Response found in cache
- Includes: model, tokens, latency_ms (typically <20ms)
- Example: Duplicate prompt cached response

### cache_miss
- Recorded when: Cache lookup fails, requires API call
- Includes: model, latency_ms
- Note: Implicit in api_call recording

### model_override
- Recorded when: Model is rewritten by proxy
- Includes: model (original model name)
- Example: Sonnet → Haiku rewrite for cost savings

### error
- Recorded when: Error occurs during processing
- Includes: agent_name (if available), model
- Future: Will include error details

---

## Usage Examples

### 1. Monitor Real-Time Project Metrics
```bash
# Get current project metrics
curl http://localhost:8080/api/metrics/projects | jq '.projects["Claude Orchestra"]'

# Watch metrics update every 5 seconds
while true; do
  curl -s http://localhost:8080/api/metrics/projects | jq '.projects["Claude Orchestra"]'
  sleep 5
done
```

### 2. Stream Activity Events to File
```bash
# Collect activity events for analysis
timeout 300 curl -N http://localhost:8080/api/stream | \
  jq '.activity[]' > activity.jsonl
```

### 3. Monitor Cache Performance
```bash
# Get current stats (old format still available)
curl http://localhost:8080/api/stats | jq '.cache'

# Results: hit_rate, hits, misses, entries, total_savings
```

### 4. Track Cost Over Time
```javascript
// Real-time cost tracking
const eventSource = new EventSource('http://localhost:8080/api/stream');
let lastCost = 0;

eventSource.addEventListener('analytics', (event) => {
  const data = JSON.parse(event.data);
  const cost = data.project.cost;
  const increment = cost - lastCost;
  
  console.log(`Cost: $${cost.toFixed(2)} (+$${increment.toFixed(2)})`);
  console.log(`Calls: ${data.project.calls}`);
  lastCost = cost;
});
```

---

## API Compatibility

### Backward Compatibility
- Existing endpoints unchanged:
  - `GET /api/stats` - Old format still available
  - `GET /api/project/stats` - Old format still available
  - `GET /api/machine/stats` - Old format still available
  - `GET /api/overrides/stats` - Unchanged

- New endpoints:
  - `GET /api/metrics/projects` - New format
  - `GET /api/stream` - Updated format (breaking change)

### Migration Guide
If using old SSE format:
```javascript
// OLD (no longer available)
eventSource.addEventListener('analytics', (event) => {
  const data = JSON.parse(event.data);
  const stats = data;  // Old StatsResponse
});

// NEW (current)
eventSource.addEventListener('analytics', (event) => {
  const data = JSON.parse(event.data);
  const project = data.project;
  const machine = data.machine;
  const activity = data.activity;
});
```

---

## Rate Limiting

### Current Behavior
- No rate limiting applied
- SSE updates every 5 seconds
- Activity buffer: max 100 events
- Activity query: immediate response

### Recommendations
- Polling: 5-10 second intervals
- SSE: Connected streaming recommended
- Batch requests: Group metrics queries

---

## Troubleshooting

### Empty Activity Array
- Possible causes:
  - No recent API calls
  - Server just started
  - Activity buffer cleared
- Solution: Make an API call to `/v1/chat/completions`

### Missing Agent Name
- Normal behavior: agent_name is null unless detected
- Detection: Happens via system message content analysis
- See: `detect_agent_from_conversation()` in server.rs

### High Latency Values
- Typical for first request: 100-500ms
- Cache hits: <20ms
- API calls: 200-1000ms depending on model
- Network: Measure with latency_ms field

---

## Performance Notes

- SSE payload: ~2-3 KB per update
- Update frequency: 5 second intervals
- Activity buffer: ~15 KB max memory
- Overhead: <1ms per event record

---

## Future Enhancements

Planned additions:
- SQLite persistence (7 days history)
- Real CPU/memory metrics (using sysinfo)
- Activity filtering by event_type
- Latency percentiles (p50, p95, p99)

Contact: Project maintainers
