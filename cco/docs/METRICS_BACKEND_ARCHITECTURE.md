# CCO Metrics Backend Architecture

**Version**: 2025.11.2
**Author**: Backend Architect
**Date**: 2025-01-17
**Status**: Design Ready for Implementation

---

## Executive Summary

This document defines the backend architecture for the CCO Proxy API cost monitoring daemon. The system provides real-time metrics tracking, historical analysis, and cost reporting for Claude API usage across all agent tiers (Opus/Sonnet/Haiku).

**Key Features**:
- Real-time metrics pipeline (<1s latency)
- SQLite-based persistence (no external dependencies)
- Per-tier cost analysis (Opus/Sonnet/Haiku)
- Rolling aggregations (1m/5m/10m windows)
- RESTful API for TUI dashboard
- Cross-platform (Mac/Windows)
- Self-contained single executable

---

## 1. Data Model Design

### 1.1 Database Schema (SQLite)

```sql
-- Core API call tracking table
CREATE TABLE api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,  -- Unix timestamp (milliseconds)
    session_id TEXT,             -- Session identifier (optional)
    agent_type TEXT,             -- e.g., "chief-architect", "python-specialist"
    agent_name TEXT,             -- Agent instance name (optional)
    model_requested TEXT NOT NULL,  -- Original model requested
    model_used TEXT NOT NULL,       -- Actual model used (after overrides)
    provider TEXT NOT NULL,         -- "anthropic", "openai", "ollama"

    -- Token counts
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cache_write_tokens INTEGER DEFAULT 0,  -- Claude prompt cache writes
    cache_read_tokens INTEGER DEFAULT 0,   -- Claude prompt cache reads

    -- Cost tracking
    cost_usd REAL NOT NULL,        -- Actual cost in USD
    would_be_cost_usd REAL,        -- Cost without cache (for savings calc)

    -- Performance metrics
    latency_ms INTEGER NOT NULL,   -- API call latency
    ttfb_ms INTEGER,               -- Time to first byte (streaming)

    -- Source information
    source_file TEXT,              -- File that triggered the call (if available)
    cache_hit BOOLEAN DEFAULT 0,   -- Proxy cache hit (not Claude cache)
    error_code TEXT,               -- Error code if failed

    -- Indexes for common queries
    INDEX idx_timestamp (timestamp),
    INDEX idx_model_used (model_used),
    INDEX idx_agent_type (agent_type),
    INDEX idx_session_id (session_id)
);

-- Aggregated metrics by time window
CREATE TABLE metrics_aggregated (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    window_start INTEGER NOT NULL,  -- Unix timestamp (start of window)
    window_size_seconds INTEGER NOT NULL,  -- 60, 300, 600 (1m, 5m, 10m)
    model TEXT NOT NULL,

    -- Aggregated counts
    total_calls INTEGER NOT NULL,
    total_input_tokens INTEGER NOT NULL,
    total_output_tokens INTEGER NOT NULL,
    total_cache_write_tokens INTEGER NOT NULL,
    total_cache_read_tokens INTEGER NOT NULL,

    -- Aggregated costs
    total_cost_usd REAL NOT NULL,
    total_would_be_cost_usd REAL NOT NULL,
    total_savings_usd REAL NOT NULL,

    -- Performance aggregations
    avg_latency_ms REAL NOT NULL,
    p50_latency_ms REAL,
    p95_latency_ms REAL,
    p99_latency_ms REAL,

    -- Rate metrics
    calls_per_minute REAL NOT NULL,
    tokens_per_minute REAL NOT NULL,
    cost_per_minute_usd REAL NOT NULL,

    -- Unique constraint to prevent duplicates
    UNIQUE(window_start, window_size_seconds, model),
    INDEX idx_window (window_start, window_size_seconds),
    INDEX idx_model_agg (model)
);

-- Model tier mapping (Opus/Sonnet/Haiku classification)
CREATE TABLE model_tiers (
    model TEXT PRIMARY KEY,
    tier TEXT NOT NULL,  -- "opus", "sonnet", "haiku", "other"
    provider TEXT NOT NULL,
    input_cost_per_1m REAL NOT NULL,   -- Cost per 1M input tokens
    output_cost_per_1m REAL NOT NULL,  -- Cost per 1M output tokens
    cache_write_cost_per_1m REAL DEFAULT 0,
    cache_read_cost_per_1m REAL DEFAULT 0,
    active BOOLEAN DEFAULT 1,  -- For deprecating old models

    INDEX idx_tier (tier)
);

-- Session tracking (for multi-agent workflows)
CREATE TABLE sessions (
    session_id TEXT PRIMARY KEY,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    project_name TEXT,
    total_cost_usd REAL DEFAULT 0,
    total_calls INTEGER DEFAULT 0,

    INDEX idx_started_at (started_at)
);

-- Configuration and pricing (runtime updates without code changes)
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### 1.2 In-Memory Data Structures

```rust
/// Real-time metrics aggregator (held in memory for <1s latency)
pub struct MetricsAggregator {
    // Rolling windows (ring buffers)
    pub window_1m: RingBuffer<ApiCallEvent>,   // Last 60 seconds
    pub window_5m: RingBuffer<ApiCallEvent>,   // Last 5 minutes
    pub window_10m: RingBuffer<ApiCallEvent>,  // Last 10 minutes

    // Current counters (updated in real-time)
    pub current_calls: AtomicU64,
    pub current_cost_usd: AtomicF64,
    pub current_tokens_in: AtomicU64,
    pub current_tokens_out: AtomicU64,

    // Per-tier breakdown (Opus/Sonnet/Haiku)
    pub tier_metrics: DashMap<String, TierMetrics>,

    // Rate calculations (calls/min, $/min)
    pub rate_tracker: RateTracker,
}

/// Ring buffer for efficient time-windowed aggregation
pub struct RingBuffer<T> {
    buffer: VecDeque<T>,
    max_size: usize,
    window_duration: Duration,
}

/// Per-tier metrics snapshot
pub struct TierMetrics {
    pub tier: String,  // "opus", "sonnet", "haiku"
    pub calls: u64,
    pub cost_usd: f64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cache_savings_usd: f64,
}

/// API call event (minimal structure for memory efficiency)
pub struct ApiCallEvent {
    pub timestamp_ms: u64,
    pub model: String,
    pub tier: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
    pub latency_ms: u32,
}

/// Rate calculation tracker
pub struct RateTracker {
    // Sliding windows for rate calculation
    pub last_minute: RingBuffer<(u64, f64)>,  // (timestamp, cost)
    pub calls_per_min: AtomicF64,
    pub cost_per_min_usd: AtomicF64,
}
```

---

## 2. Integration Points

### 2.1 CCO Proxy Integration

**Current State Analysis**:
- CCO proxy already has `AnalyticsEngine` tracking API calls
- Uses in-memory `Vec<ApiCallRecord>` with periodic JSON persistence
- Records: model, tokens, cost, cache hit/miss, activity events

**Integration Strategy**:

```rust
// Modify existing analytics.rs to emit events to metrics backend
impl AnalyticsEngine {
    pub async fn record_api_call(&self, record: ApiCallRecord) {
        // Existing behavior: store in memory
        let mut records = self.records.lock().await;
        records.push(record.clone());

        // NEW: Emit to metrics backend
        if let Some(backend) = &self.metrics_backend {
            backend.record_call(ApiCallEvent {
                timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
                model: record.model.clone(),
                tier: classify_model_tier(&record.model),
                input_tokens: record.input_tokens,
                output_tokens: record.output_tokens,
                cost_usd: record.actual_cost,
                latency_ms: 0, // TODO: Track in ChatRequest handler
            }).await;
        }
    }
}

// Classification helper
fn classify_model_tier(model: &str) -> String {
    if model.contains("opus") {
        "opus".to_string()
    } else if model.contains("sonnet") {
        "sonnet".to_string()
    } else if model.contains("haiku") {
        "haiku".to_string()
    } else {
        "other".to_string()
    }
}
```

**Injection Point**: `/v1/chat/completions` endpoint in `server.rs`

```rust
async fn chat_completion(
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {
    let request_start = std::time::Instant::now();

    // ... existing logic ...

    let response = state.proxy.handle_request(request.clone()).await;
    let latency_ms = request_start.elapsed().as_millis() as u64;

    // EXISTING: Record in analytics
    state.analytics.record_api_call(ApiCallRecord {
        // ... existing fields ...
    }).await;

    // NEW: Emit to metrics backend with latency
    state.metrics_backend.record_call(ApiCallEvent {
        timestamp_ms: Utc::now().timestamp_millis() as u64,
        model: request.model.clone(),
        tier: classify_model_tier(&request.model),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        cost_usd: cost,
        latency_ms: latency_ms as u32,
    }).await;

    Ok(Json(response))
}
```

### 2.2 Cost Calculation (Pricing Model)

**Pricing Data** (as of January 2025):

| Model | Tier | Input ($/1M) | Output ($/1M) | Cache Write ($/1M) | Cache Read ($/1M) |
|-------|------|--------------|---------------|-------------------|------------------|
| claude-opus-4-1-20250805 | Opus | $15.00 | $75.00 | $18.75 | $1.50 |
| claude-sonnet-4-5-20250929 | Sonnet | $3.00 | $15.00 | $3.75 | $0.30 |
| claude-haiku-4-5-20251001 | Haiku | $0.80 | $4.00 | $1.00 | $0.08 |

**Cost Calculation Logic**:

```rust
pub struct CostCalculator {
    pricing: DashMap<String, ModelPricing>,
}

impl CostCalculator {
    pub fn calculate_cost(&self, call: &ApiCallEvent) -> f64 {
        let pricing = match self.pricing.get(&call.model) {
            Some(p) => p,
            None => return 0.0,  // Unknown model
        };

        let input_cost = (call.input_tokens as f64 / 1_000_000.0)
            * pricing.input_cost_per_1m;
        let output_cost = (call.output_tokens as f64 / 1_000_000.0)
            * pricing.output_cost_per_1m;

        // TODO: Add cache_write_tokens and cache_read_tokens when available

        input_cost + output_cost
    }

    pub fn calculate_with_cache(&self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        cache_write_tokens: u32,
        cache_read_tokens: u32
    ) -> (f64, f64) {
        // (actual_cost, would_be_cost)
        let pricing = match self.pricing.get(model) {
            Some(p) => p,
            None => return (0.0, 0.0),
        };

        let actual_cost =
            (cache_write_tokens as f64 / 1_000_000.0) * pricing.cache_write_cost_per_1m +
            (cache_read_tokens as f64 / 1_000_000.0) * pricing.cache_read_cost_per_1m +
            ((input_tokens - cache_write_tokens - cache_read_tokens) as f64 / 1_000_000.0)
                * pricing.input_cost_per_1m +
            (output_tokens as f64 / 1_000_000.0) * pricing.output_cost_per_1m;

        let would_be_cost =
            (input_tokens as f64 / 1_000_000.0) * pricing.input_cost_per_1m +
            (output_tokens as f64 / 1_000_000.0) * pricing.output_cost_per_1m;

        (actual_cost, would_be_cost)
    }
}
```

### 2.3 Token Counting from API Responses

**Claude API Response Format**:

```json
{
  "id": "msg_...",
  "type": "message",
  "model": "claude-sonnet-4-5-20250929",
  "content": [{"type": "text", "text": "..."}],
  "usage": {
    "input_tokens": 1523,
    "output_tokens": 892,
    "cache_creation_input_tokens": 0,
    "cache_read_input_tokens": 0
  }
}
```

**Token Extraction** (already implemented in `proxy.rs`):

```rust
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    // TODO: Add cache tokens when implementing Claude cache
    // pub cache_write_tokens: u32,
    // pub cache_read_tokens: u32,
}

impl ProxyServer {
    pub async fn handle_request(&self, request: ChatRequest) -> ChatResponse {
        // ... API call logic ...

        ChatResponse {
            id: response_id,
            model: request.model,
            content: extracted_text,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            usage,
            from_cache: false,
        }
    }
}
```

---

## 3. Persistence Strategy

### 3.1 SQLite Database Configuration

**File Location**:
- **macOS/Linux**: `~/.local/share/cco/metrics.db`
- **Windows**: `%LOCALAPPDATA%\cco\metrics.db`

```rust
pub struct MetricsDatabase {
    pool: sqlx::SqlitePool,
}

impl MetricsDatabase {
    pub async fn initialize() -> anyhow::Result<Self> {
        let db_path = get_metrics_db_path()?;

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Create connection pool
        let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
        // Create tables (idempotent)
        sqlx::query(include_str!("../migrations/001_initial_schema.sql"))
            .execute(pool)
            .await?;

        // Seed default pricing
        sqlx::query(include_str!("../migrations/002_seed_pricing.sql"))
            .execute(pool)
            .await?;

        Ok(())
    }
}

fn get_metrics_db_path() -> anyhow::Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?;

    Ok(data_dir.join("cco").join("metrics.db"))
}
```

### 3.2 Write Strategy

**Write Batching** (for performance):

```rust
pub struct BatchWriter {
    buffer: Arc<Mutex<Vec<ApiCallEvent>>>,
    db: MetricsDatabase,
    batch_size: usize,
    flush_interval: Duration,
}

impl BatchWriter {
    pub fn new(db: MetricsDatabase, batch_size: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
            db,
            batch_size,
            flush_interval: Duration::from_secs(5),
        }
    }

    pub async fn write(&self, event: ApiCallEvent) -> anyhow::Result<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(event);

        if buffer.len() >= self.batch_size {
            self.flush_internal(&mut buffer).await?;
        }

        Ok(())
    }

    async fn flush_internal(&self, buffer: &mut Vec<ApiCallEvent>) -> anyhow::Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        // Bulk insert
        let mut tx = self.db.pool.begin().await?;

        for event in buffer.drain(..) {
            sqlx::query(
                "INSERT INTO api_calls (timestamp, model_used, tier, input_tokens,
                 output_tokens, cost_usd, latency_ms)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(event.timestamp_ms as i64)
            .bind(&event.model)
            .bind(&event.tier)
            .bind(event.input_tokens as i64)
            .bind(event.output_tokens as i64)
            .bind(event.cost_usd)
            .bind(event.latency_ms as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // Background task to flush periodically
    pub async fn run_periodic_flush(&self) {
        let mut interval = tokio::time::interval(self.flush_interval);

        loop {
            interval.tick().await;

            let mut buffer = self.buffer.lock().await;
            if let Err(e) = self.flush_internal(&mut buffer).await {
                tracing::error!("Failed to flush metrics batch: {}", e);
            }
        }
    }
}
```

### 3.3 Archival Strategy

**Data Retention Policy**:

| Data Type | Retention Period | Aggregation Level |
|-----------|------------------|-------------------|
| Raw API calls | 7 days | Individual records |
| 1-minute aggregates | 30 days | Per-model summary |
| 5-minute aggregates | 90 days | Per-model summary |
| 10-minute aggregates | 1 year | Per-model summary |
| Daily summaries | Forever | Per-tier totals |

**Archival Task** (runs daily):

```rust
pub async fn archive_old_data(db: &MetricsDatabase) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let seven_days_ago = now - (7 * 24 * 60 * 60 * 1000);

    // Archive raw calls to aggregated metrics before deletion
    sqlx::query(
        "INSERT INTO metrics_aggregated (window_start, window_size_seconds, model,
         total_calls, total_input_tokens, total_output_tokens, total_cost_usd,
         calls_per_minute, cost_per_minute_usd)
         SELECT
           (timestamp / 86400000) * 86400000 as day_start,
           86400 as window_size,
           model_used,
           COUNT(*) as calls,
           SUM(input_tokens),
           SUM(output_tokens),
           SUM(cost_usd),
           COUNT(*) / 1440.0 as calls_per_min,
           SUM(cost_usd) / 1440.0 as cost_per_min
         FROM api_calls
         WHERE timestamp < ?
         GROUP BY day_start, model_used"
    )
    .bind(seven_days_ago)
    .execute(&db.pool)
    .await?;

    // Delete archived raw calls
    sqlx::query("DELETE FROM api_calls WHERE timestamp < ?")
        .bind(seven_days_ago)
        .execute(&db.pool)
        .await?;

    Ok(())
}
```

---

## 4. Real-Time Metrics Pipeline

### 4.1 Event Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     CCO Proxy Server                            │
│                                                                 │
│  /v1/chat/completions                                          │
│         │                                                       │
│         ├─> handle_request() ──────┐                          │
│         │                           │                          │
│         │                           ▼                          │
│         │                    AnalyticsEngine                   │
│         │                           │                          │
│         │                           ├──> record_api_call()     │
│         │                           │                          │
│         │                           ▼                          │
│         │                    MetricsBackend                    │
│         │                           │                          │
└─────────┼───────────────────────────┼──────────────────────────┘
          │                           │
          │                           ▼
          │                  ┌─────────────────┐
          │                  │ MetricsAggregator│
          │                  │  (in-memory)    │
          │                  │                 │
          │                  │ - 1m window     │
          │                  │ - 5m window     │
          │                  │ - 10m window    │
          │                  │ - Rate tracker  │
          │                  │ - Tier counters │
          │                  └────────┬────────┘
          │                           │
          │                           ├──> Update ring buffers
          │                           ├──> Increment counters
          │                           ├──> Calculate rates
          │                           │
          ▼                           ▼
  ┌────────────────┐        ┌────────────────┐
  │  BatchWriter   │        │  QueryCache    │
  │                │        │                │
  │ - Buffer calls │        │ - 1s TTL       │
  │ - Flush every  │        │ - Pre-computed │
  │   5s or 100    │        │   responses    │
  │   calls        │        └────────┬───────┘
  └───────┬────────┘                 │
          │                          │
          ▼                          │
  ┌────────────────┐                 │
  │  SQLite DB     │                 │
  │  metrics.db    │                 │
  │                │                 │
  │ - api_calls    │                 │
  │ - aggregated   │                 │
  └────────────────┘                 │
                                     │
                                     ▼
                          ┌──────────────────┐
                          │   TUI Dashboard   │
                          │   (via REST API)  │
                          │                   │
                          │ GET /api/stats    │
                          │ GET /api/tiers    │
                          │ GET /api/realtime │
                          └───────────────────┘
```

### 4.2 Aggregation Windows

**Ring Buffer Implementation**:

```rust
impl RingBuffer<ApiCallEvent> {
    pub fn new(window_duration: Duration) -> Self {
        Self {
            buffer: VecDeque::new(),
            max_size: 10_000,  // Prevent unbounded growth
            window_duration,
        }
    }

    pub fn push(&mut self, event: ApiCallEvent) {
        // Add new event
        self.buffer.push_back(event);

        // Remove expired events
        let cutoff = event.timestamp_ms - self.window_duration.as_millis() as u64;
        while let Some(front) = self.buffer.front() {
            if front.timestamp_ms < cutoff {
                self.buffer.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn aggregate(&self) -> AggregatedMetrics {
        let mut total_calls = 0u64;
        let mut total_cost = 0.0f64;
        let mut total_tokens_in = 0u64;
        let mut total_tokens_out = 0u64;
        let mut latencies = Vec::new();

        for event in &self.buffer {
            total_calls += 1;
            total_cost += event.cost_usd;
            total_tokens_in += event.input_tokens as u64;
            total_tokens_out += event.output_tokens as u64;
            latencies.push(event.latency_ms);
        }

        // Calculate percentiles
        latencies.sort_unstable();
        let p50 = percentile(&latencies, 50);
        let p95 = percentile(&latencies, 95);
        let p99 = percentile(&latencies, 99);

        AggregatedMetrics {
            total_calls,
            total_cost_usd: total_cost,
            total_tokens_in,
            total_tokens_out,
            avg_latency_ms: latencies.iter().sum::<u32>() as f64 / latencies.len() as f64,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
        }
    }
}

fn percentile(sorted: &[u32], p: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (sorted.len() * p) / 100;
    sorted[idx.min(sorted.len() - 1)] as f64
}
```

### 4.3 Rate Calculation

**Calls per Minute**:

```rust
impl RateTracker {
    pub fn update(&mut self, timestamp_ms: u64, cost_usd: f64) {
        // Add to sliding window
        self.last_minute.push((timestamp_ms, cost_usd));

        // Calculate rate based on time span
        let oldest = self.last_minute.buffer.front().unwrap().0;
        let newest = self.last_minute.buffer.back().unwrap().0;
        let time_span_minutes = (newest - oldest) as f64 / 60_000.0;

        if time_span_minutes > 0.0 {
            let calls = self.last_minute.buffer.len() as f64;
            let total_cost: f64 = self.last_minute.buffer.iter()
                .map(|(_, cost)| cost)
                .sum();

            self.calls_per_min.store(calls / time_span_minutes, Ordering::Relaxed);
            self.cost_per_min_usd.store(total_cost / time_span_minutes, Ordering::Relaxed);
        }
    }
}
```

---

## 5. API for TUI Dashboard

### 5.1 REST Endpoints

**Summary Stats** (`GET /api/stats`):

```json
{
  "summary": {
    "total_calls": 1523,
    "total_cost_usd": 42.87,
    "total_tokens_in": 2_456_789,
    "total_tokens_out": 987_654,
    "cache_savings_usd": 12.34,
    "avg_latency_ms": 1450,
    "calls_per_minute": 3.2,
    "cost_per_minute_usd": 0.089
  },
  "tiers": [
    {
      "tier": "opus",
      "calls": 123,
      "cost_usd": 32.45,
      "tokens_in": 500_000,
      "tokens_out": 250_000,
      "percentage_of_total_cost": 75.7
    },
    {
      "tier": "sonnet",
      "calls": 890,
      "cost_usd": 8.92,
      "tokens_in": 1_500_000,
      "tokens_out": 600_000,
      "percentage_of_total_cost": 20.8
    },
    {
      "tier": "haiku",
      "calls": 510,
      "cost_usd": 1.50,
      "tokens_in": 456_789,
      "tokens_out": 137_654,
      "percentage_of_total_cost": 3.5
    }
  ],
  "window": "last_10_minutes"
}
```

**Real-Time Updates** (`GET /api/realtime`):

```json
{
  "current_rate": {
    "calls_per_minute": 4.7,
    "cost_per_minute_usd": 0.15,
    "tokens_per_minute": 125_000
  },
  "last_call": {
    "timestamp": "2025-01-17T10:32:45Z",
    "model": "claude-sonnet-4-5-20250929",
    "tier": "sonnet",
    "cost_usd": 0.023,
    "latency_ms": 1230
  },
  "windows": {
    "1m": {
      "calls": 5,
      "cost_usd": 0.15,
      "avg_latency_ms": 1350
    },
    "5m": {
      "calls": 23,
      "cost_usd": 0.72,
      "avg_latency_ms": 1420
    },
    "10m": {
      "calls": 48,
      "cost_usd": 1.45,
      "avg_latency_ms": 1380
    }
  }
}
```

**Recent Calls** (`GET /api/calls/recent?limit=20`):

```json
{
  "calls": [
    {
      "id": 98765,
      "timestamp": "2025-01-17T10:32:45.123Z",
      "model": "claude-opus-4-1-20250805",
      "tier": "opus",
      "agent_type": "chief-architect",
      "input_tokens": 5234,
      "output_tokens": 892,
      "cost_usd": 0.145,
      "latency_ms": 2340,
      "cache_hit": false
    }
    // ... 19 more
  ]
}
```

**By-Tier Breakdown** (`GET /api/tiers/{tier}`):

```json
{
  "tier": "sonnet",
  "models": [
    {
      "model": "claude-sonnet-4-5-20250929",
      "calls": 890,
      "cost_usd": 8.92,
      "tokens_in": 1_500_000,
      "tokens_out": 600_000
    }
  ],
  "total_cost_usd": 8.92,
  "percentage_of_total": 20.8,
  "top_agents": [
    {
      "agent_type": "python-specialist",
      "calls": 423,
      "cost_usd": 4.23
    },
    {
      "agent_type": "test-engineer",
      "calls": 312,
      "cost_usd": 3.12
    }
  ]
}
```

### 5.2 Update Frequency

**Query Strategy**:

```rust
pub struct QueryCache {
    cache: DashMap<String, (Instant, serde_json::Value)>,
    ttl: Duration,
}

impl QueryCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            ttl: Duration::from_secs(1),  // 1-second TTL for real-time feel
        }
    }

    pub async fn get_or_compute<F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> anyhow::Result<serde_json::Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<serde_json::Value>>,
    {
        // Check cache
        if let Some(entry) = self.cache.get(key) {
            let (cached_at, value) = entry.value();
            if cached_at.elapsed() < self.ttl {
                return Ok(value.clone());
            }
        }

        // Compute fresh value
        let value = compute().await?;
        self.cache.insert(key.to_string(), (Instant::now(), value.clone()));

        Ok(value)
    }
}
```

**TUI Polling Interval**: 1 second (matches cache TTL)

---

## 6. Configuration Management

### 6.1 API Key Handling

**Storage**: Environment variable (secure, cross-platform)

```bash
# User sets API key
export ANTHROPIC_API_KEY="sk-ant-..."

# CCO reads on startup
cco daemon start --port 3000
```

**Runtime Access**:

```rust
pub struct ApiConfig {
    pub anthropic_api_key: String,
    pub openai_api_key: Option<String>,
}

impl ApiConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")
                .context("ANTHROPIC_API_KEY not set")?,
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        })
    }
}
```

### 6.2 Log File Locations

**Platform-Specific Paths**:

```rust
pub fn get_log_path() -> anyhow::Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get local data directory"))?;

    // macOS: ~/Library/Application Support/cco/logs/
    // Windows: %LOCALAPPDATA%\cco\logs\
    Ok(data_dir.join("cco").join("logs").join("metrics.log"))
}
```

**Log Rotation** (prevent unbounded growth):

```rust
use tracing_subscriber::fmt::time::ChronoLocal;

pub fn setup_logging() -> anyhow::Result<()> {
    let log_path = get_log_path()?;

    // Create directory
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Configure appender with rotation (10MB max, 5 files)
    let file_appender = tracing_appender::rolling::daily(
        log_path.parent().unwrap(),
        log_path.file_name().unwrap()
    );

    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::rfc_3339())
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    Ok(())
}
```

### 6.3 Cost Pricing Configuration

**Configuration File** (`~/.config/cco/pricing.toml`):

```toml
# Cost pricing configuration (updated manually or via CLI)

[[models]]
name = "claude-opus-4-1-20250805"
tier = "opus"
provider = "anthropic"
input_cost_per_1m = 15.00
output_cost_per_1m = 75.00
cache_write_cost_per_1m = 18.75
cache_read_cost_per_1m = 1.50

[[models]]
name = "claude-sonnet-4-5-20250929"
tier = "sonnet"
provider = "anthropic"
input_cost_per_1m = 3.00
output_cost_per_1m = 15.00
cache_write_cost_per_1m = 3.75
cache_read_cost_per_1m = 0.30

[[models]]
name = "claude-haiku-4-5-20251001"
tier = "haiku"
provider = "anthropic"
input_cost_per_1m = 0.80
output_cost_per_1m = 4.00
cache_write_cost_per_1m = 1.00
cache_read_cost_per_1m = 0.08
```

**Runtime Updates** (via CLI):

```bash
# Update pricing for a model
cco config pricing update claude-opus-4-1-20250805 \
  --input 15.00 \
  --output 75.00 \
  --cache-write 18.75 \
  --cache-read 1.50

# List current pricing
cco config pricing list
```

---

## 7. Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                    API Call Flow                                 │
└──────────────────────────────────────────────────────────────────┘

1. Claude Code Agent → CCO Proxy
   POST /v1/chat/completions
   {
     "model": "claude-sonnet-4-5-20250929",
     "messages": [...]
   }

2. CCO Proxy → Claude API
   (with model override if configured)

3. Claude API → CCO Proxy
   {
     "usage": {
       "input_tokens": 1523,
       "output_tokens": 892,
       "cache_creation_input_tokens": 0,
       "cache_read_input_tokens": 0
     }
   }

4. CCO Proxy → MetricsBackend.record_call()
   ApiCallEvent {
     timestamp_ms: 1705484565123,
     model: "claude-sonnet-4-5-20250929",
     tier: "sonnet",
     input_tokens: 1523,
     output_tokens: 892,
     cost_usd: 0.018,  // Calculated
     latency_ms: 1450
   }

5. MetricsBackend → In-Memory Aggregator
   - Update 1m/5m/10m ring buffers
   - Increment tier counters (Opus/Sonnet/Haiku)
   - Calculate rate (calls/min, $/min)

6. MetricsBackend → BatchWriter
   - Buffer event (flush every 5s or 100 calls)

7. BatchWriter → SQLite Database
   INSERT INTO api_calls (...) VALUES (...)

8. TUI Dashboard → CCO Proxy API
   GET /api/stats  (every 1 second)

9. CCO Proxy API → QueryCache
   - Check cache (1s TTL)
   - If expired: query in-memory aggregator + DB
   - Return JSON response

10. TUI Dashboard → Render UI
    - Display real-time costs
    - Show tier breakdown (Opus/Sonnet/Haiku)
    - Graph rate over time
```

---

## 8. Implementation Roadmap

### Phase 1: Core Backend (Week 1)

- [ ] SQLite schema implementation
- [ ] MetricsBackend struct with in-memory aggregator
- [ ] Integration with existing AnalyticsEngine
- [ ] BatchWriter for persistence
- [ ] Basic REST API endpoints

**Deliverables**:
- `src/metrics_backend.rs` (500 LOC)
- `src/metrics_db.rs` (300 LOC)
- `migrations/001_initial_schema.sql`
- Updated `server.rs` with `/api/stats` endpoint

### Phase 2: Real-Time Aggregation (Week 2)

- [ ] Ring buffer implementation
- [ ] Rate tracker (calls/min, $/min)
- [ ] Per-tier breakdown (Opus/Sonnet/Haiku)
- [ ] Query cache (1s TTL)
- [ ] REST API for real-time updates

**Deliverables**:
- `src/metrics_aggregator.rs` (400 LOC)
- `src/rate_tracker.rs` (200 LOC)
- Updated API endpoints (`/api/realtime`, `/api/tiers`)

### Phase 3: Configuration & Pricing (Week 3)

- [ ] TOML-based pricing configuration
- [ ] CLI commands for pricing updates
- [ ] Platform-specific paths (logs, DB)
- [ ] Log rotation
- [ ] Configuration validation

**Deliverables**:
- `src/config.rs` (300 LOC)
- `pricing.toml` template
- CLI subcommands (`cco config pricing`)

### Phase 4: Testing & Optimization (Week 4)

- [ ] Unit tests (90%+ coverage)
- [ ] Integration tests with mock API
- [ ] Performance benchmarks (10k calls/min target)
- [ ] Database indexing optimization
- [ ] Memory profiling and optimization

**Deliverables**:
- `tests/metrics_backend_tests.rs`
- `benches/metrics_performance.rs`
- Performance report

---

## 9. Scalability & Performance

### 9.1 Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Latency (record call) | <1ms | Must not slow down proxy |
| Latency (query API) | <10ms | Real-time dashboard feel |
| Throughput (writes) | 10k calls/min | Support heavy agent usage |
| Memory (in-memory buffer) | <100 MB | Ring buffers + tier counters |
| Database size (30 days) | <500 MB | With 1M calls (estimate) |

### 9.2 Optimization Strategies

**1. Batch Writes**:
- Buffer 100 calls or 5 seconds (whichever comes first)
- Reduces SQLite transaction overhead (100x speedup)

**2. Query Caching**:
- 1-second TTL for `/api/stats` endpoint
- Pre-computed responses in memory
- Prevents redundant DB queries

**3. Ring Buffer Eviction**:
- Automatic cleanup of expired events
- Bounded memory usage (max 10k events per window)

**4. Index Optimization**:
- Indexes on `timestamp`, `model_used`, `tier`
- Covering indexes for common queries
- Vacuum DB weekly to reclaim space

**5. In-Memory Aggregation**:
- All real-time queries served from memory
- DB only for historical analysis (>10 minutes)

---

## 10. Security Considerations

### 10.1 API Key Storage

- **NEVER** store API keys in database
- Use environment variables only
- Warn if API key found in config files
- Validate key format (prevent injection)

### 10.2 Database Security

- File permissions: `0600` (owner read/write only)
- No remote access (SQLite file-based)
- Input sanitization (SQLx prepared statements)
- Rate limiting on API endpoints

### 10.3 Log Security

- No API keys in logs (redact if present)
- No sensitive prompt content
- Rotate logs to prevent disk exhaustion
- File permissions: `0600`

---

## 11. Error Handling

### 11.1 Database Errors

```rust
impl MetricsBackend {
    pub async fn record_call(&self, event: ApiCallEvent) -> anyhow::Result<()> {
        // Try to write to batch
        if let Err(e) = self.batch_writer.write(event.clone()).await {
            tracing::warn!("Failed to write to batch, falling back to direct write: {}", e);

            // Fallback: direct DB write
            if let Err(e2) = self.db.write_call(&event).await {
                tracing::error!("Failed to persist metrics event: {}", e2);

                // Last resort: write to error log for manual recovery
                self.write_to_error_log(&event)?;
            }
        }

        // Always update in-memory aggregator (even if persistence fails)
        self.aggregator.record(event);

        Ok(())
    }
}
```

### 11.2 API Errors

```rust
async fn stats_handler(State(state): State<Arc<ServerState>>)
    -> Result<Json<StatsResponse>, ServerError>
{
    match state.metrics_backend.get_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            tracing::error!("Failed to fetch stats: {}", e);

            // Return partial data from in-memory aggregator
            let partial_stats = state.metrics_backend.aggregator.snapshot();
            Ok(Json(StatsResponse::partial(partial_stats)))
        }
    }
}
```

---

## 12. Monitoring & Observability

### 12.1 Health Metrics

```rust
pub struct MetricsBackendHealth {
    pub buffer_size: usize,           // Current batch buffer size
    pub buffer_capacity: usize,       // Max buffer size
    pub last_flush_at: Instant,       // Last successful flush
    pub flush_error_count: u64,       // Failed flush attempts
    pub db_size_bytes: u64,           // Database file size
    pub oldest_record_age_secs: u64,  // Age of oldest record
}

impl MetricsBackend {
    pub async fn health_check(&self) -> MetricsBackendHealth {
        // Monitor metrics backend health
    }
}
```

**Health Endpoint** (`GET /api/metrics/health`):

```json
{
  "status": "healthy",
  "buffer": {
    "size": 42,
    "capacity": 100,
    "usage_percent": 42.0
  },
  "database": {
    "size_mb": 127.4,
    "oldest_record_age_hours": 156.2
  },
  "last_flush": "2025-01-17T10:32:40Z",
  "flush_errors": 0
}
```

### 12.2 Alerts

**Trigger Conditions**:

| Alert | Condition | Action |
|-------|-----------|--------|
| High buffer usage | >90% capacity | Log warning + force flush |
| Flush failures | >5 consecutive | Alert + fallback to logs |
| Database size | >1 GB | Trigger archival task |
| Stale data | Oldest record >7 days | Run cleanup task |

---

## Appendix A: File Structure

```
cco/
├── src/
│   ├── metrics_backend.rs      # Main metrics backend (500 LOC)
│   ├── metrics_db.rs           # SQLite persistence (300 LOC)
│   ├── metrics_aggregator.rs   # In-memory aggregation (400 LOC)
│   ├── rate_tracker.rs         # Rate calculations (200 LOC)
│   ├── config.rs               # Configuration management (300 LOC)
│   └── server.rs               # Updated with new endpoints
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_seed_pricing.sql
│   └── 003_add_indexes.sql
├── tests/
│   ├── metrics_backend_tests.rs
│   └── metrics_integration_tests.rs
└── benches/
    └── metrics_performance.rs
```

**Total Estimated LOC**: ~2,000 lines (backend only, excluding TUI)

---

## Appendix B: Cost Calculation Examples

### Example 1: Opus Call (No Cache)

```
Model: claude-opus-4-1-20250805
Input tokens: 5,000
Output tokens: 2,000

Cost = (5,000 / 1,000,000) * $15.00 + (2,000 / 1,000,000) * $75.00
     = $0.075 + $0.150
     = $0.225
```

### Example 2: Sonnet Call (90% Cache Read)

```
Model: claude-sonnet-4-5-20250929
Input tokens: 10,000
Cache read tokens: 9,000
New input tokens: 1,000
Output tokens: 3,000

Cost = (9,000 / 1,000,000) * $0.30 +  # Cache read
       (1,000 / 1,000,000) * $3.00 +  # New input
       (3,000 / 1,000,000) * $15.00   # Output
     = $0.0027 + $0.003 + $0.045
     = $0.0507

Would-be cost (no cache):
     = (10,000 / 1,000,000) * $3.00 + (3,000 / 1,000,000) * $15.00
     = $0.03 + $0.045
     = $0.075

Savings = $0.075 - $0.0507 = $0.0243 (32% reduction)
```

### Example 3: Haiku Call (High Volume)

```
Model: claude-haiku-4-5-20251001
Input tokens: 1,500
Output tokens: 500

Cost = (1,500 / 1,000,000) * $0.80 + (500 / 1,000,000) * $4.00
     = $0.0012 + $0.002
     = $0.0032

If 1,000 calls per day:
Daily cost = $0.0032 * 1,000 = $3.20
Monthly cost = $3.20 * 30 = $96.00
```

---

## Appendix C: Database Queries

### Query 1: Tier Breakdown (Last 24 Hours)

```sql
SELECT
  tier,
  COUNT(*) as calls,
  SUM(input_tokens) as total_input,
  SUM(output_tokens) as total_output,
  SUM(cost_usd) as total_cost,
  AVG(latency_ms) as avg_latency
FROM api_calls
WHERE timestamp > (strftime('%s', 'now') - 86400) * 1000
GROUP BY tier
ORDER BY total_cost DESC;
```

### Query 2: Top Models by Cost (Last 7 Days)

```sql
SELECT
  model_used,
  tier,
  COUNT(*) as calls,
  SUM(cost_usd) as total_cost,
  SUM(cost_usd) * 100.0 / (SELECT SUM(cost_usd) FROM api_calls
                             WHERE timestamp > (strftime('%s', 'now') - 604800) * 1000) as percent_of_total
FROM api_calls
WHERE timestamp > (strftime('%s', 'now') - 604800) * 1000
GROUP BY model_used, tier
ORDER BY total_cost DESC
LIMIT 10;
```

### Query 3: Hourly Cost Trend (Last 24 Hours)

```sql
SELECT
  (timestamp / 3600000) * 3600000 as hour_start,
  tier,
  COUNT(*) as calls,
  SUM(cost_usd) as cost
FROM api_calls
WHERE timestamp > (strftime('%s', 'now') - 86400) * 1000
GROUP BY hour_start, tier
ORDER BY hour_start ASC;
```

---

## Conclusion

This architecture provides a scalable, performant, and maintainable backend for CCO's API cost monitoring daemon. Key strengths:

1. **Real-time Performance**: <1s latency from API call to dashboard
2. **Self-Contained**: No external dependencies (SQLite, no network services)
3. **Cross-Platform**: Works on Mac/Windows with platform-specific paths
4. **Scalable**: Handles 10k calls/min with batching and caching
5. **Observable**: Built-in health checks and error recovery

**Ready for Implementation**: All integration points defined, data models specified, and API contracts documented.

**Next Steps**: Coordinate with TUI Developer for dashboard implementation and Chief Architect for final review.
