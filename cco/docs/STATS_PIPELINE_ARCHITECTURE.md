# Stats Pipeline Architecture

## Overview

This document describes the architecture for a real-time statistics pipeline that gathers metrics from Claude Code logs, stores them in SQLite, and serves them via the `/api/stats` endpoint with ~5 second refresh intervals.

## Component Diagram

```
+------------------------------------------------------------------+
|                         CCO Daemon                                |
+------------------------------------------------------------------+
|                                                                  |
|  +------------------+     +------------------+                   |
|  |   Log Watcher    |---->|   Log Parser     |                   |
|  | (notify/inotify) |     | (JSONL -> Struct)|                   |
|  +------------------+     +------------------+                   |
|          |                        |                              |
|          | File events            | Parsed messages              |
|          v                        v                              |
|  +------------------+     +------------------+                   |
|  |   Stats          |<----|   Stats          |                   |
|  |   Aggregator     |     |   Aggregator     |                   |
|  | (in-memory ring) |     | (calculate)      |                   |
|  +------------------+     +------------------+                   |
|          |                                                       |
|          | Aggregated stats                                      |
|          v                                                       |
|  +------------------+     +------------------+                   |
|  |  Database Writer |---->|    SQLite DB     |                   |
|  | (batch insert)   |     | (~/.cco/stats.db)|                   |
|  +------------------+     +------------------+                   |
|          |                        ^                              |
|          | Write confirmation     | Query                        |
|          v                        |                              |
|  +------------------+     +------------------+                   |
|  | Background Worker|     |   /api/stats     |                   |
|  | (5s interval)    |     |   Endpoint       |                   |
|  +------------------+     +------------------+                   |
|                                   |                              |
+-----------------------------------|------------------------------+
                                    |
                                    v
                              HTTP Response
```

## Data Flow

### 1. Log Source Discovery

Claude Code stores conversation logs in:
- **Project logs**: `~/.claude/projects/{project-name}/*.jsonl`
- **Global metrics**: `~/.claude/metrics.json`
- **Session history**: `~/.claude/history.jsonl`

The Log Watcher monitors these locations for:
1. New JSONL files (new conversations)
2. File modifications (appended messages)
3. File deletions (cleanup/rotation)

### 2. Message Flow

```
JSONL File Change
       |
       v
+------------------+
| Log Watcher      |
| - inotify/FSEvents
| - Debounce 100ms |
+------------------+
       |
       v (file path + change type)
+------------------+
| Log Parser       |
| - Read new lines |
| - Parse JSON     |
| - Extract usage  |
+------------------+
       |
       v (Vec<ParsedMessage>)
+------------------+
| Stats Aggregator |
| - Update counters|
| - Calculate cost |
| - Track sessions |
+------------------+
       |
       v (StatsSnapshot)
+------------------+
| Database Writer  |
| - Batch insert   |
| - Update summary |
+------------------+
```

### 3. API Request Flow

```
GET /api/stats
       |
       v
+------------------+
| Stats Handler    |
| - Check cache    |
| - Query if stale |
+------------------+
       |
       v (cache miss)
+------------------+
| SQLite Query     |
| - Get summary    |
| - Get recent     |
| - Get breakdown  |
+------------------+
       |
       v
+------------------+
| Response Builder |
| - Format JSON    |
| - Add metadata   |
+------------------+
       |
       v
HTTP 200 JSON
```

## Database Schema

### Tables

```sql
-- Raw API call events from Claude Code logs
CREATE TABLE api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Timestamps
    timestamp DATETIME NOT NULL,
    ingested_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Source identification
    session_id TEXT,           -- Claude session UUID
    project_name TEXT,         -- Project directory name
    source_file TEXT,          -- Full path to JSONL file

    -- Model information
    model_name TEXT NOT NULL,
    model_tier TEXT NOT NULL CHECK(model_tier IN ('Opus', 'Sonnet', 'Haiku', 'Unknown')),

    -- Token breakdown
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,

    -- Cost calculation (USD)
    input_cost REAL NOT NULL DEFAULT 0.0,
    output_cost REAL NOT NULL DEFAULT 0.0,
    cache_write_cost REAL NOT NULL DEFAULT 0.0,
    cache_read_cost REAL NOT NULL DEFAULT 0.0,
    total_cost REAL NOT NULL DEFAULT 0.0,

    -- Message metadata
    message_type TEXT,         -- 'assistant', 'user', etc.
    tool_use_count INTEGER DEFAULT 0,

    -- Deduplication
    message_uuid TEXT UNIQUE   -- Prevent re-ingesting same message
);

-- Pre-aggregated hourly stats for fast queries
CREATE TABLE hourly_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Time bucket
    hour_start DATETIME NOT NULL,

    -- Aggregation key
    model_tier TEXT NOT NULL,
    project_name TEXT,         -- NULL = all projects

    -- Aggregated metrics
    call_count INTEGER NOT NULL DEFAULT 0,
    total_input_tokens INTEGER NOT NULL DEFAULT 0,
    total_output_tokens INTEGER NOT NULL DEFAULT 0,
    total_cache_write_tokens INTEGER NOT NULL DEFAULT 0,
    total_cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,

    -- Unique constraint
    UNIQUE(hour_start, model_tier, project_name)
);

-- Daily cost summaries for dashboard
CREATE TABLE daily_costs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Time bucket
    date DATE NOT NULL,

    -- Aggregation key
    model_tier TEXT,           -- NULL = all tiers
    project_name TEXT,         -- NULL = all projects

    -- Cost breakdown
    total_cost REAL NOT NULL DEFAULT 0.0,
    input_cost REAL NOT NULL DEFAULT 0.0,
    output_cost REAL NOT NULL DEFAULT 0.0,
    cache_cost REAL NOT NULL DEFAULT 0.0,

    -- Token totals
    total_tokens INTEGER NOT NULL DEFAULT 0,

    -- Unique constraint
    UNIQUE(date, model_tier, project_name)
);

-- Session tracking for conversation-level metrics
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    session_id TEXT NOT NULL UNIQUE,
    project_name TEXT,

    -- Timestamps
    first_seen DATETIME NOT NULL,
    last_seen DATETIME NOT NULL,

    -- Aggregates
    message_count INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,
    total_tokens INTEGER NOT NULL DEFAULT 0,

    -- Status
    is_active INTEGER NOT NULL DEFAULT 1
);

-- File tracking for incremental parsing
CREATE TABLE file_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    file_path TEXT NOT NULL UNIQUE,
    last_modified DATETIME NOT NULL,
    last_size INTEGER NOT NULL,
    last_line_parsed INTEGER NOT NULL DEFAULT 0,

    -- Checksum for change detection
    content_hash TEXT
);
```

### Indexes

```sql
-- api_calls indexes
CREATE INDEX idx_api_calls_timestamp ON api_calls(timestamp DESC);
CREATE INDEX idx_api_calls_session ON api_calls(session_id);
CREATE INDEX idx_api_calls_project ON api_calls(project_name);
CREATE INDEX idx_api_calls_model_tier ON api_calls(model_tier);
CREATE INDEX idx_api_calls_ingested ON api_calls(ingested_at DESC);

-- hourly_stats indexes
CREATE INDEX idx_hourly_hour ON hourly_stats(hour_start DESC);
CREATE INDEX idx_hourly_tier ON hourly_stats(model_tier);

-- daily_costs indexes
CREATE INDEX idx_daily_date ON daily_costs(date DESC);

-- sessions indexes
CREATE INDEX idx_sessions_project ON sessions(project_name);
CREATE INDEX idx_sessions_active ON sessions(is_active, last_seen DESC);
```

## API Specification

### GET /api/stats

Returns aggregated statistics for Claude Code usage.

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `day` | Time period: `hour`, `day`, `week`, `month`, `all` |
| `project` | string | null | Filter by project name |
| `model` | string | null | Filter by model tier: `opus`, `sonnet`, `haiku` |
| `from` | ISO8601 | null | Start timestamp |
| `to` | ISO8601 | null | End timestamp |

**Response:**

```json
{
  "status": "ok",
  "generated_at": "2025-11-26T19:30:00Z",
  "period": {
    "type": "day",
    "start": "2025-11-26T00:00:00Z",
    "end": "2025-11-26T23:59:59Z"
  },
  "summary": {
    "total_cost_usd": 12.45,
    "total_calls": 1234,
    "total_tokens": 2456789,
    "tokens_by_type": {
      "input": 1234567,
      "output": 987654,
      "cache_write": 123456,
      "cache_read": 111112
    },
    "cache_savings_usd": 3.21,
    "avg_cost_per_call": 0.0101
  },
  "by_model_tier": {
    "Opus": {
      "call_count": 50,
      "total_cost": 8.50,
      "total_tokens": 500000,
      "avg_tokens_per_call": 10000
    },
    "Sonnet": {
      "call_count": 800,
      "total_cost": 3.50,
      "total_tokens": 1500000,
      "avg_tokens_per_call": 1875
    },
    "Haiku": {
      "call_count": 384,
      "total_cost": 0.45,
      "total_tokens": 456789,
      "avg_tokens_per_call": 1189
    }
  },
  "by_project": {
    "cc-orchestra": {
      "call_count": 600,
      "total_cost": 7.20
    },
    "other-project": {
      "call_count": 634,
      "total_cost": 5.25
    }
  },
  "time_series": {
    "granularity": "hour",
    "data": [
      {
        "timestamp": "2025-11-26T00:00:00Z",
        "cost": 0.52,
        "calls": 45
      },
      {
        "timestamp": "2025-11-26T01:00:00Z",
        "cost": 0.31,
        "calls": 28
      }
    ]
  },
  "recent_calls": [
    {
      "timestamp": "2025-11-26T19:28:45Z",
      "model": "claude-sonnet-4-5",
      "project": "cc-orchestra",
      "tokens": 15234,
      "cost": 0.0456
    }
  ],
  "sessions": {
    "active_count": 3,
    "today_count": 12,
    "total_count": 156
  }
}
```

### GET /api/stats/live

Server-Sent Events endpoint for real-time stats updates.

**Response (SSE stream):**

```
event: stats-update
data: {"type":"call","model":"sonnet","tokens":1234,"cost":0.005}

event: stats-update
data: {"type":"summary","total_cost_today":12.67,"calls_today":1256}
```

### GET /api/stats/projects

Returns list of tracked projects with basic stats.

**Response:**

```json
{
  "projects": [
    {
      "name": "cc-orchestra",
      "path": "/Users/brent/git/cc-orchestra",
      "total_cost": 45.67,
      "total_calls": 3456,
      "last_activity": "2025-11-26T19:28:00Z"
    }
  ]
}
```

## Component Specifications

### 1. Log Watcher

**Module:** `src/daemon/stats/watcher.rs`

**Responsibilities:**
- Monitor `~/.claude/projects/` directory tree
- Detect new, modified, and deleted JSONL files
- Debounce rapid file changes (100ms)
- Maintain file cursor positions for incremental reads

**Key Types:**

```rust
pub struct LogWatcher {
    /// Watcher handle
    watcher: RecommendedWatcher,

    /// File state tracking
    file_states: Arc<RwLock<HashMap<PathBuf, FileState>>>,

    /// Event channel
    event_tx: mpsc::Sender<LogEvent>,
}

pub enum LogEvent {
    FileCreated(PathBuf),
    FileModified(PathBuf),
    FileDeleted(PathBuf),
}

pub struct FileState {
    pub last_modified: SystemTime,
    pub last_size: u64,
    pub last_line: usize,
    pub content_hash: Option<String>,
}
```

**Implementation Notes:**
- Use `notify` crate for cross-platform file watching
- On macOS: FSEvents (efficient for entire directory trees)
- On Linux: inotify (requires explicit watch per directory)
- Buffer file events with 100ms debounce to handle rapid writes
- Track file cursor to only parse new lines on modification

### 2. Log Parser

**Module:** `src/daemon/stats/parser.rs`

**Responsibilities:**
- Parse JSONL log files line by line
- Extract assistant messages with usage data
- Calculate costs using model pricing
- Handle malformed JSON gracefully

**Key Types:**

```rust
pub struct LogParser {
    /// Model pricing lookup
    pricing: ModelPricing,
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub project_name: Option<String>,
    pub message_uuid: String,
    pub model_name: String,
    pub model_tier: ModelTier,
    pub tokens: TokenBreakdown,
    pub cost: CostBreakdown,
    pub message_type: String,
    pub tool_use_count: u32,
}

#[derive(Debug, Clone)]
pub struct TokenBreakdown {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
}

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
    pub total: f64,
}
```

**Parsing Logic:**

```rust
impl LogParser {
    pub fn parse_line(&self, line: &str, source: &Path) -> Option<ParsedMessage> {
        // 1. Parse JSON
        let msg: ClaudeMessage = serde_json::from_str(line).ok()?;

        // 2. Filter for assistant messages with usage
        if msg.message_type != "assistant" {
            return None;
        }

        let content = msg.message?;
        let usage = content.usage?;
        let model = content.model?;

        // 3. Calculate costs
        let tier = ModelTier::from_model_name(&model)?;
        let pricing = tier.pricing();

        let tokens = TokenBreakdown {
            input: usage.input_tokens.unwrap_or(0),
            output: usage.output_tokens.unwrap_or(0),
            cache_creation: usage.cache_creation_input_tokens.unwrap_or(0),
            cache_read: usage.cache_read_input_tokens.unwrap_or(0),
        };

        let cost = tokens.calculate_cost(&pricing);

        // 4. Extract metadata
        let project_name = extract_project_name(source);

        Some(ParsedMessage {
            timestamp: msg.timestamp.parse().ok()?,
            session_id: msg.session_id,
            project_name,
            message_uuid: msg.uuid,
            model_name: model,
            model_tier: tier,
            tokens,
            cost,
            message_type: "assistant".into(),
            tool_use_count: count_tool_uses(&content),
        })
    }
}
```

### 3. Stats Aggregator

**Module:** `src/daemon/stats/aggregator.rs`

**Responsibilities:**
- Maintain in-memory running totals
- Calculate session-level statistics
- Generate time-bucketed aggregations
- Provide thread-safe access to stats

**Key Types:**

```rust
pub struct StatsAggregator {
    /// Current period stats (reset periodically)
    current: Arc<RwLock<PeriodStats>>,

    /// Historical stats ring buffer
    history: Arc<RwLock<VecDeque<PeriodStats>>>,

    /// Session tracking
    sessions: Arc<RwLock<HashMap<String, SessionStats>>>,

    /// Per-project stats
    projects: Arc<RwLock<HashMap<String, ProjectStats>>>,
}

#[derive(Debug, Clone, Default)]
pub struct PeriodStats {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    pub total_calls: u64,
    pub total_cost: f64,
    pub total_tokens: u64,

    pub by_tier: HashMap<ModelTier, TierStats>,
    pub by_project: HashMap<String, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct TierStats {
    pub call_count: u64,
    pub total_cost: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_read_tokens: u64,
}
```

**Aggregation Methods:**

```rust
impl StatsAggregator {
    /// Process a new parsed message
    pub async fn record(&self, msg: ParsedMessage) {
        let mut current = self.current.write().await;

        // Update totals
        current.total_calls += 1;
        current.total_cost += msg.cost.total;
        current.total_tokens += msg.tokens.total();

        // Update tier breakdown
        let tier = current.by_tier.entry(msg.model_tier).or_default();
        tier.call_count += 1;
        tier.total_cost += msg.cost.total;
        tier.input_tokens += msg.tokens.input;
        tier.output_tokens += msg.tokens.output;
        tier.cache_write_tokens += msg.tokens.cache_creation;
        tier.cache_read_tokens += msg.tokens.cache_read;

        // Update project breakdown
        if let Some(project) = &msg.project_name {
            *current.by_project.entry(project.clone()).or_default() += 1;
        }

        // Update session
        if let Some(session_id) = &msg.session_id {
            let mut sessions = self.sessions.write().await;
            let session = sessions.entry(session_id.clone()).or_insert_with(|| {
                SessionStats::new(session_id.clone(), msg.project_name.clone())
            });
            session.record_call(msg.cost.total, msg.tokens.total());
        }
    }

    /// Get current snapshot for API response
    pub async fn snapshot(&self) -> StatsSnapshot {
        let current = self.current.read().await;
        let sessions = self.sessions.read().await;

        StatsSnapshot {
            summary: current.clone(),
            active_sessions: sessions.values()
                .filter(|s| s.is_active())
                .count(),
            recent_calls: self.get_recent_calls(20).await,
        }
    }
}
```

### 4. Database Writer

**Module:** `src/daemon/stats/database.rs`

**Responsibilities:**
- Persist parsed messages to SQLite
- Update aggregation tables
- Handle deduplication via message UUID
- Batch writes for efficiency

**Key Types:**

```rust
pub struct StatsDatabase {
    pool: SqlitePool,

    /// Write buffer for batching
    buffer: Arc<Mutex<Vec<ParsedMessage>>>,

    /// Flush interval
    flush_interval: Duration,
}

impl StatsDatabase {
    /// Initialize database with schema
    pub async fn new(path: PathBuf) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}", path.display()))
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations/stats").run(&pool).await?;

        Ok(Self {
            pool,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
            flush_interval: Duration::from_secs(5),
        })
    }

    /// Buffer a message for batch insert
    pub async fn buffer_message(&self, msg: ParsedMessage) {
        let mut buffer = self.buffer.lock().await;
        buffer.push(msg);

        // Auto-flush if buffer is large
        if buffer.len() >= 100 {
            drop(buffer);
            self.flush().await.ok();
        }
    }

    /// Flush buffer to database
    pub async fn flush(&self) -> Result<()> {
        let messages: Vec<ParsedMessage> = {
            let mut buffer = self.buffer.lock().await;
            std::mem::take(&mut *buffer)
        };

        if messages.is_empty() {
            return Ok(());
        }

        // Batch insert
        let mut tx = self.pool.begin().await?;

        for msg in &messages {
            sqlx::query(r#"
                INSERT OR IGNORE INTO api_calls (
                    timestamp, session_id, project_name, source_file,
                    model_name, model_tier,
                    input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
                    input_cost, output_cost, cache_write_cost, cache_read_cost, total_cost,
                    message_type, tool_use_count, message_uuid
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&msg.timestamp)
            .bind(&msg.session_id)
            .bind(&msg.project_name)
            // ... bind all fields
            .execute(&mut *tx)
            .await?;
        }

        // Update hourly aggregates
        self.update_hourly_stats(&mut tx, &messages).await?;

        tx.commit().await?;

        Ok(())
    }
}
```

### 5. Background Worker

**Module:** `src/daemon/stats/worker.rs`

**Responsibilities:**
- Orchestrate periodic stats updates
- Coordinate log watching and parsing
- Trigger database flushes
- Manage component lifecycle

**Key Types:**

```rust
pub struct StatsWorker {
    watcher: LogWatcher,
    parser: LogParser,
    aggregator: Arc<StatsAggregator>,
    database: Arc<StatsDatabase>,

    /// Update interval
    interval: Duration,

    /// Shutdown signal
    shutdown: broadcast::Receiver<()>,
}

impl StatsWorker {
    pub async fn run(mut self) -> Result<()> {
        let mut interval = tokio::time::interval(self.interval);

        // Channel for log events
        let (event_tx, mut event_rx) = mpsc::channel(1000);

        // Start file watcher
        self.watcher.start(event_tx)?;

        loop {
            tokio::select! {
                // Handle file events
                Some(event) = event_rx.recv() => {
                    self.handle_log_event(event).await?;
                }

                // Periodic flush
                _ = interval.tick() => {
                    self.database.flush().await?;
                    self.update_aggregations().await?;
                }

                // Shutdown signal
                _ = self.shutdown.recv() => {
                    info!("Stats worker shutting down");
                    self.database.flush().await?;
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_log_event(&self, event: LogEvent) -> Result<()> {
        match event {
            LogEvent::FileModified(path) => {
                // Read new lines from file
                let new_lines = self.watcher.read_new_lines(&path).await?;

                for line in new_lines {
                    if let Some(msg) = self.parser.parse_line(&line, &path) {
                        self.aggregator.record(msg.clone()).await;
                        self.database.buffer_message(msg).await;
                    }
                }
            }
            LogEvent::FileCreated(path) => {
                // Parse entire new file
                self.parse_entire_file(&path).await?;
            }
            LogEvent::FileDeleted(path) => {
                // Remove from file state tracking
                self.watcher.remove_file(&path).await;
            }
        }

        Ok(())
    }
}
```

### 6. API Endpoint

**Module:** `src/daemon/stats/api.rs`

**Responsibilities:**
- Handle HTTP requests for stats
- Query database and aggregator
- Format JSON responses
- Implement caching layer

**Implementation:**

```rust
/// Stats API handler
pub async fn get_stats(
    State(state): State<Arc<DaemonState>>,
    Query(params): Query<StatsQueryParams>,
) -> Result<Json<StatsResponse>, AppError> {
    let stats_service = state.stats_service
        .as_ref()
        .ok_or(AppError::ServiceUnavailable("Stats service not available"))?;

    // Check cache
    let cache_key = params.cache_key();
    if let Some(cached) = stats_service.get_cached(&cache_key).await {
        return Ok(Json(cached));
    }

    // Build time range
    let (start, end) = params.time_range()?;

    // Query aggregated stats
    let summary = stats_service.get_summary(start, end, &params).await?;
    let by_tier = stats_service.get_by_tier(start, end, &params).await?;
    let by_project = stats_service.get_by_project(start, end, &params).await?;
    let time_series = stats_service.get_time_series(start, end, &params).await?;
    let recent = stats_service.get_recent_calls(20).await?;
    let sessions = stats_service.get_session_summary().await?;

    let response = StatsResponse {
        status: "ok".into(),
        generated_at: Utc::now(),
        period: PeriodInfo {
            period_type: params.period.clone(),
            start,
            end,
        },
        summary,
        by_model_tier: by_tier,
        by_project,
        time_series,
        recent_calls: recent,
        sessions,
    };

    // Cache response
    stats_service.cache_response(&cache_key, &response, Duration::from_secs(5)).await;

    Ok(Json(response))
}
```

## Implementation Phases

### Phase 1: Basic /api/stats Endpoint with Mock Data
**Effort: 2-3 hours**

**Deliverables:**
- New `src/daemon/stats/mod.rs` module structure
- Mock `StatsResponse` struct with static data
- `/api/stats` endpoint returning mock JSON
- Basic integration with existing router

**Tasks:**
1. Create stats module directory structure
2. Define API response types
3. Add mock endpoint to server.rs router
4. Write integration test

### Phase 2: SQLite Schema and Basic Storage
**Effort: 4-6 hours**

**Deliverables:**
- SQLite migrations for stats tables
- `StatsDatabase` struct with CRUD operations
- Basic insert and query methods
- Database initialization on daemon startup

**Tasks:**
1. Create migrations directory
2. Write schema creation SQL
3. Implement StatsDatabase with sqlx
4. Add database to DaemonState
5. Write unit tests for database operations

### Phase 3: Log Parsing and Ingestion
**Effort: 6-8 hours**

**Deliverables:**
- `LogParser` with JSONL parsing
- Cost calculation logic
- One-time full scan of existing logs
- Insert parsed messages to database

**Tasks:**
1. Implement JSONL parser (reuse existing claude_history.rs)
2. Add cost calculation (reuse existing pricing logic)
3. Create CLI command for initial scan: `cco stats scan`
4. Populate database with historical data
5. Update /api/stats to query real data

### Phase 4: Background Updates and Live Refresh
**Effort: 8-10 hours**

**Deliverables:**
- `LogWatcher` with file system monitoring
- `StatsWorker` background task
- Incremental parsing of new log entries
- Real-time stats updates

**Tasks:**
1. Add `notify` dependency
2. Implement LogWatcher with debouncing
3. Create StatsWorker async task
4. Integrate with daemon lifecycle
5. Test with live Claude Code session

### Phase 5: Performance Optimization
**Effort: 4-6 hours**

**Deliverables:**
- Response caching layer
- Pre-aggregated hourly/daily tables
- Query optimization
- Metrics and monitoring

**Tasks:**
1. Add in-memory cache with TTL
2. Implement hourly stats aggregation
3. Add indexes for common queries
4. Add timing metrics to endpoints
5. Performance testing and tuning

## Risks and Mitigation

### Risk 1: Log File Format Changes
**Risk Level:** Medium

Claude Code may change its JSONL format in updates.

**Mitigation:**
- Use lenient JSON parsing with defaults
- Log unknown fields for monitoring
- Design parser to gracefully handle missing fields
- Version the parser and support multiple formats

### Risk 2: Large Log Files
**Risk Level:** Medium

Active users may have very large JSONL files (100MB+).

**Mitigation:**
- Use streaming line-by-line parsing
- Track file position to only read new lines
- Implement file rotation awareness
- Set reasonable buffer sizes

### Risk 3: Database Growth
**Risk Level:** Low

Stats database could grow large over time.

**Mitigation:**
- Automatic cleanup of old raw events (>30 days)
- Keep aggregated data longer (hourly: 90 days, daily: forever)
- Implement vacuum/optimize on schedule
- Add configurable retention policies

### Risk 4: Race Conditions
**Risk Level:** Low

Multiple processes reading/writing to files and database.

**Mitigation:**
- Use SQLite WAL mode for concurrent reads
- File locking for watcher state
- Atomic batch operations
- Proper async synchronization

### Risk 5: Resource Usage
**Risk Level:** Low

Background worker consuming too much CPU/memory.

**Mitigation:**
- Debounce file events (100ms)
- Batch database writes
- Use ring buffers with fixed capacity
- Configurable update interval (default 5s)

## Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Database | SQLite + sqlx | Already used for audit.rs, embedded, no separate process |
| File Watching | notify crate | Cross-platform (FSEvents on macOS, inotify on Linux) |
| Async Runtime | tokio | Already used throughout codebase |
| JSON Parsing | serde_json | Already used, robust error handling |
| Caching | mini-moka or simple HashMap+TTL | Lightweight, in-process |
| Time Handling | chrono | Already used throughout codebase |

## Integration Points

### DaemonState Extension

```rust
pub struct DaemonState {
    // ... existing fields ...

    /// Stats service (optional, enabled by config)
    pub stats_service: Option<Arc<StatsService>>,
}
```

### Router Extension

```rust
fn create_router(state: Arc<DaemonState>) -> Router {
    let mut router = Router::new()
        // ... existing routes ...
        ;

    // Add stats routes if enabled
    if state.stats_service.is_some() {
        router = router
            .route("/api/stats", get(get_stats))
            .route("/api/stats/live", get(stats_live_sse))
            .route("/api/stats/projects", get(get_projects));

        info!("Stats API endpoints enabled");
    }

    router.with_state(state)
}
```

### Configuration

```toml
# ~/.cco/config.toml

[stats]
enabled = true
update_interval_secs = 5
retention_days = 30
database_path = "~/.cco/stats.db"

[stats.watch]
paths = ["~/.claude/projects"]
debounce_ms = 100
```

## Success Metrics

1. **Latency**: `/api/stats` responds in <100ms for cached data
2. **Freshness**: Stats reflect new log entries within 10 seconds
3. **Accuracy**: Cost calculations match Claude Code billing
4. **Resource Usage**: <50MB memory, <1% CPU at idle
5. **Reliability**: Zero data loss on daemon restart

---

*Document Version: 1.0*
*Created: 2025-11-26*
*Author: Backend Architecture Agent*
