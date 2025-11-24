# CCO Analytics Pipeline Architecture

## Overview
Complete redesign of the CCO dashboard analytics system to fix critical bugs and provide comprehensive metrics tracking with persistence.

## Critical Issues Resolved

### 1. SSE Data Format Mismatch
**Issue**: Server sends `{ cache, models, totals }` but frontend expects `{ project, machine, activity }`
**Solution**: Restructure SSE payload to match frontend expectations

### 2. Missing Activity Tracking
**Issue**: Only API calls tracked, no error/cache/model events
**Solution**: Implement comprehensive event tracking system

### 3. Timestamp DOM Bug
**Issue**: Timestamps computed but not displayed
**Solution**: Fix DOM update logic in dashboard.js

### 4. Project Breakdown Missing
**Issue**: No per-project metrics available
**Solution**: Add project aggregation and filtering

### 5. Data Persistence Missing
**Issue**: Metrics lost on server restart
**Solution**: SQLite storage with 7-day retention

## Data Structures

### SSE Event Format
```json
{
  "project": {
    "name": "current_project_name",
    "cost": 123.45,
    "tokens": 50000,
    "calls": 100,
    "lastUpdated": "2025-11-15T10:30:00Z",
    "costTrend": 5.2,
    "tokensTrend": 3.1,
    "callsTrend": 2.5,
    "avgTime": 245
  },
  "machine": {
    "cpu": 45.2,
    "memory": 2048,
    "uptime": 3600,
    "processCount": 5,
    "totalCost": 456.78,
    "activeProjects": 3,
    "totalCalls": 1500,
    "totalTokens": 750000
  },
  "activity": {
    "type": "api_call",
    "timestamp": "2025-11-15T10:30:00Z",
    "data": {}
  }
}
```

### Activity Event Types

#### API Call
```json
{
  "type": "api_call",
  "timestamp": "2025-11-15T10:30:00Z",
  "data": {
    "model": "claude-opus-4.1",
    "project": "project_name",
    "inputTokens": 1000,
    "outputTokens": 500,
    "latency": 245,
    "cost": 52.50,
    "success": true
  }
}
```

#### Cache Hit/Miss
```json
{
  "type": "cache_hit",
  "timestamp": "2025-11-15T10:30:00Z",
  "data": {
    "model": "claude-opus-4.1",
    "project": "project_name",
    "savings": 52.50,
    "cacheKey": "hash_key"
  }
}
```

#### Error
```json
{
  "type": "error",
  "timestamp": "2025-11-15T10:30:00Z",
  "data": {
    "error": "Rate limit exceeded",
    "model": "claude-opus-4.1",
    "project": "project_name",
    "context": "API call failed"
  }
}
```

#### Model Override
```json
{
  "type": "model_override",
  "timestamp": "2025-11-15T10:30:00Z",
  "data": {
    "originalModel": "claude-opus-4.1",
    "overrideModel": "claude-sonnet-4.5",
    "reason": "opus_exhausted",
    "project": "project_name"
  }
}
```

## Storage Schema

### SQLite Tables

```sql
-- Main metrics table
CREATE TABLE metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL,
    event_type TEXT NOT NULL,
    project TEXT,
    model TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost REAL,
    savings REAL,
    latency INTEGER,
    cache_hit BOOLEAN,
    error TEXT,
    metadata JSON,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX idx_metrics_project ON metrics(project);
CREATE INDEX idx_metrics_model ON metrics(model);
CREATE INDEX idx_metrics_event_type ON metrics(event_type);

-- Project aggregates for fast queries
CREATE TABLE project_aggregates (
    project TEXT PRIMARY KEY,
    total_cost REAL DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    total_calls INTEGER DEFAULT 0,
    total_errors INTEGER DEFAULT 0,
    cache_hits INTEGER DEFAULT 0,
    cache_misses INTEGER DEFAULT 0,
    avg_latency REAL DEFAULT 0,
    last_updated DATETIME
);

-- System state for recovery
CREATE TABLE system_state (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## API Endpoints

### GET /api/metrics/current
Returns current real-time metrics with proper format for frontend

**Response:**
```json
{
  "project": { /* current project stats */ },
  "machine": { /* system-wide stats */ },
  "activity": [ /* last 50 events */ ]
}
```

### GET /api/metrics/projects
Returns per-project breakdown

**Response:**
```json
{
  "projects": [
    {
      "name": "project_name",
      "cost": 123.45,
      "tokens": 50000,
      "calls": 100,
      "errors": 5,
      "cacheHitRate": 85.5,
      "avgLatency": 245,
      "lastActivity": "2025-11-15T10:30:00Z"
    }
  ]
}
```

### GET /api/metrics/activity
Query activity log with filters

**Query Parameters:**
- `limit` (default: 100, max: 1000)
- `offset` (default: 0)
- `type` (filter by event type)
- `project` (filter by project)
- `from` (ISO date)
- `to` (ISO date)

**Response:**
```json
{
  "events": [ /* activity events */ ],
  "total": 5000,
  "hasMore": true
}
```

### GET /api/metrics/models
Returns model usage breakdown

**Response:**
```json
{
  "models": [
    {
      "model": "claude-opus-4.1",
      "requests": 100,
      "cost": 525.00,
      "tokens": 150000,
      "cacheHitRate": 70.5,
      "avgLatency": 300,
      "errors": 2
    }
  ]
}
```

## Implementation Components

### 1. Analytics Module (analytics.rs)

**New Types:**
```rust
pub enum EventType {
    ApiCall,
    CacheHit,
    CacheMiss,
    Error,
    ModelOverride,
    System,
}

pub struct ActivityEvent {
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub project: Option<String>,
    pub model: Option<String>,
    pub data: serde_json::Value,
}

pub struct ProjectMetrics {
    pub name: String,
    pub cost: f64,
    pub tokens: u64,
    pub calls: u64,
    pub errors: u64,
    pub cache_hit_rate: f64,
    pub avg_latency: f64,
    pub last_activity: DateTime<Utc>,
}
```

**New Methods:**
```rust
impl AnalyticsEngine {
    pub async fn track_event(&self, event: ActivityEvent);
    pub async fn get_project_metrics(&self, project: &str) -> ProjectMetrics;
    pub async fn get_activity_log(&self, filter: ActivityFilter) -> Vec<ActivityEvent>;
    pub async fn persist_to_disk(&self);
    pub async fn load_from_disk(&self);
}
```

### 2. Server Module (server.rs)

**SSE Handler Fix:**
- Transform analytics data to match frontend format
- Include project, machine, and activity fields
- Ensure proper JSON structure

**New Endpoints:**
- Implement `/api/metrics/projects`
- Implement `/api/metrics/activity`
- Implement `/api/metrics/models`
- Fix `/api/stream` SSE format

### 3. Frontend (dashboard.js)

**Required Fixes:**
1. Update `handleAnalyticsUpdate()` for new format
2. Fix timestamp DOM updates
3. Add project selector dropdown
4. Implement activity filtering
5. Add error notifications

## Persistence Strategy

### Startup Sequence
1. Check for existing SQLite database
2. Run migrations if needed
3. Load last 7 days of data
4. Populate in-memory caches
5. Calculate initial aggregates
6. Start SSE streaming

### Runtime Operations
1. Buffer events (batch size: 100 or 5 seconds)
2. Write to SQLite in background
3. Update aggregates every minute
4. Broadcast SSE immediately
5. Checkpoint every hour

### Cleanup Tasks
1. Daily: Purge data > 7 days old
2. Weekly: VACUUM database
3. Monthly: Archive summaries to JSON

## Performance Targets

- API Response Time: < 100ms
- SSE Latency: < 50ms
- SQLite Write: < 10ms batched
- Memory Usage: < 100MB for 7 days
- Startup Time: < 2 seconds

## Testing Requirements

### Unit Tests
- Analytics module event tracking
- SQLite operations
- Data transformation
- Aggregation logic

### Integration Tests
- API endpoint responses
- SSE event streaming
- Database persistence
- Error handling

### Frontend Tests
- DOM updates
- Event handling
- Chart rendering
- Filter operations

## Migration Plan

1. **Deploy Backend Changes**
   - Add SQLite dependency
   - Implement new analytics module
   - Update server endpoints
   - Test data persistence

2. **Update Frontend**
   - Fix data format handling
   - Add new UI components
   - Test real-time updates

3. **Data Migration**
   - No historical data to migrate
   - Start fresh with new schema
   - Monitor for issues

## Success Metrics

- ✅ All 5 bugs fixed
- ✅ Data persists across restarts
- ✅ Real-time updates working
- ✅ Project breakdown available
- ✅ Activity log with filters
- ✅ < 100ms API responses
- ✅ 7-day data retention