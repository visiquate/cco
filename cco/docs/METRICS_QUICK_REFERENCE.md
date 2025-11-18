# CCO Metrics Backend - Quick Reference

**For**: Developers implementing the metrics backend
**Architecture**: See `METRICS_BACKEND_ARCHITECTURE.md`
**Checklist**: See `METRICS_IMPLEMENTATION_CHECKLIST.md`

---

## 🏗️ Architecture Overview

```
┌───────────────────────────────────────────────────────────────────────┐
│                         CCO Proxy Server                              │
│                                                                       │
│  POST /v1/chat/completions  ──┐                                      │
│                                │                                      │
│                                ▼                                      │
│                          chat_completion()                            │
│                                │                                      │
│                                ├──> Calculate latency                 │
│                                ├──> Calculate cost                    │
│                                │                                      │
│                                ▼                                      │
│                    metrics_backend.record_call()                      │
│                                │                                      │
└────────────────────────────────┼───────────────────────────────────────┘
                                 │
                                 ▼
                  ┌──────────────────────────────┐
                  │     MetricsBackend           │
                  │                              │
                  │  1. Update Aggregator        │
                  │  2. Queue for BatchWriter    │
                  │  3. Update QueryCache        │
                  └──────┬─────────────┬─────────┘
                         │             │
           ┌─────────────┘             └─────────────┐
           │                                         │
           ▼                                         ▼
  ┌──────────────────┐                     ┌──────────────────┐
  │ MetricsAggregator│                     │   BatchWriter    │
  │  (in-memory)     │                     │   (async)        │
  │                  │                     │                  │
  │ - 1m window      │                     │ - Buffer: 100    │
  │ - 5m window      │                     │ - Flush: 5s      │
  │ - 10m window     │                     │ - SQLite bulk    │
  │ - Tier counters  │                     │   insert         │
  │ - Rate tracker   │                     └────────┬─────────┘
  └─────────┬────────┘                              │
            │                                       ▼
            │                              ┌──────────────────┐
            │                              │   SQLite DB      │
            │                              │   metrics.db     │
            │                              │                  │
            │                              │ - api_calls      │
            │                              │ - aggregated     │
            │                              │ - model_tiers    │
            │                              └──────────────────┘
            │
            ▼
  ┌──────────────────┐
  │   QueryCache     │
  │   (1s TTL)       │
  └─────────┬────────┘
            │
            ▼
  ┌──────────────────┐
  │   TUI Dashboard  │
  │                  │
  │ GET /api/metrics/│
  │   - stats        │
  │   - realtime     │
  │   - tiers/{tier} │
  └──────────────────┘
```

---

## 📊 Data Structures

### Core Types

```rust
// Minimal event for memory efficiency
pub struct ApiCallEvent {
    pub timestamp_ms: u64,     // Unix timestamp
    pub model: String,         // e.g., "claude-sonnet-4-5-20250929"
    pub tier: String,          // "opus", "sonnet", "haiku"
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,         // Pre-calculated
    pub latency_ms: u32,
}

// Per-tier metrics snapshot
pub struct TierMetrics {
    pub tier: String,
    pub calls: u64,
    pub cost_usd: f64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cache_savings_usd: f64,
}

// Aggregated metrics for a time window
pub struct AggregatedMetrics {
    pub window: WindowSize,      // 1m, 5m, 10m
    pub total_calls: u64,
    pub total_cost_usd: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub calls_per_minute: f64,
    pub cost_per_minute_usd: f64,
}
```

### Database Schema (Simplified)

```sql
-- Main call tracking table
CREATE TABLE api_calls (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,        -- milliseconds since epoch
    model_used TEXT NOT NULL,
    tier TEXT NOT NULL,                -- "opus", "sonnet", "haiku"
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cost_usd REAL NOT NULL,
    latency_ms INTEGER NOT NULL,

    INDEX idx_timestamp (timestamp),
    INDEX idx_tier (tier)
);

-- Pricing configuration
CREATE TABLE model_tiers (
    model TEXT PRIMARY KEY,
    tier TEXT NOT NULL,
    input_cost_per_1m REAL NOT NULL,
    output_cost_per_1m REAL NOT NULL,
    cache_read_cost_per_1m REAL DEFAULT 0,
    cache_write_cost_per_1m REAL DEFAULT 0
);
```

---

## 🔧 Implementation Snippets

### 1. Recording API Calls (Integration Point)

```rust
// In server.rs - chat_completion endpoint
async fn chat_completion(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {
    let request_start = Instant::now();

    // ... existing logic ...

    let response = state.proxy.handle_request(request.clone()).await;
    let latency_ms = request_start.elapsed().as_millis() as u64;

    // Calculate cost
    let cost_usd = state.cost_calculator.calculate_cost(
        &request.model,
        response.input_tokens,
        response.output_tokens,
    );

    // Record in metrics backend
    state.metrics_backend.record_call(ApiCallEvent {
        timestamp_ms: Utc::now().timestamp_millis() as u64,
        model: request.model.clone(),
        tier: classify_tier(&request.model),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        cost_usd,
        latency_ms: latency_ms as u32,
    }).await?;

    Ok(Json(response))
}

// Helper function
fn classify_tier(model: &str) -> String {
    if model.contains("opus") { "opus" }
    else if model.contains("sonnet") { "sonnet" }
    else if model.contains("haiku") { "haiku" }
    else { "other" }.to_string()
}
```

### 2. MetricsBackend (Main Component)

```rust
pub struct MetricsBackend {
    aggregator: Arc<MetricsAggregator>,
    batch_writer: Arc<BatchWriter>,
    db: Arc<MetricsDatabase>,
    query_cache: Arc<QueryCache>,
}

impl MetricsBackend {
    pub async fn initialize() -> anyhow::Result<Self> {
        let db = Arc::new(MetricsDatabase::initialize().await?);
        let aggregator = Arc::new(MetricsAggregator::new());
        let batch_writer = Arc::new(BatchWriter::new(db.clone(), 100));
        let query_cache = Arc::new(QueryCache::new());

        // Spawn background flush task
        let writer_clone = batch_writer.clone();
        tokio::spawn(async move {
            writer_clone.run_periodic_flush().await;
        });

        Ok(Self {
            aggregator,
            batch_writer,
            db,
            query_cache,
        })
    }

    pub async fn record_call(&self, event: ApiCallEvent) -> anyhow::Result<()> {
        // Update in-memory aggregator (always succeeds)
        self.aggregator.record(event.clone());

        // Queue for batch write (async, non-blocking)
        self.batch_writer.write(event).await?;

        Ok(())
    }

    pub async fn get_stats(&self, window: WindowSize) -> anyhow::Result<StatsResponse> {
        let cache_key = format!("stats:{:?}", window);

        self.query_cache.get_or_compute(&cache_key, || async {
            let agg_metrics = self.aggregator.get_snapshot(window);
            let tier_metrics = self.aggregator.get_tier_breakdown();

            Ok(serde_json::json!({
                "summary": agg_metrics,
                "tiers": tier_metrics,
            }))
        }).await
    }
}
```

### 3. MetricsAggregator (Ring Buffers)

```rust
pub struct MetricsAggregator {
    window_1m: Arc<Mutex<RingBuffer<ApiCallEvent>>>,
    window_5m: Arc<Mutex<RingBuffer<ApiCallEvent>>>,
    window_10m: Arc<Mutex<RingBuffer<ApiCallEvent>>>,
    tier_metrics: Arc<DashMap<String, TierMetrics>>,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            window_1m: Arc::new(Mutex::new(RingBuffer::new(Duration::from_secs(60)))),
            window_5m: Arc::new(Mutex::new(RingBuffer::new(Duration::from_secs(300)))),
            window_10m: Arc::new(Mutex::new(RingBuffer::new(Duration::from_secs(600)))),
            tier_metrics: Arc::new(DashMap::new()),
        }
    }

    pub fn record(&self, event: ApiCallEvent) {
        // Update all ring buffers
        tokio::spawn({
            let windows = vec![
                self.window_1m.clone(),
                self.window_5m.clone(),
                self.window_10m.clone(),
            ];
            let event_clone = event.clone();

            async move {
                for window in windows {
                    window.lock().await.push(event_clone.clone());
                }
            }
        });

        // Update tier metrics
        self.tier_metrics.entry(event.tier.clone())
            .and_modify(|m| {
                m.calls += 1;
                m.cost_usd += event.cost_usd;
                m.tokens_in += event.input_tokens as u64;
                m.tokens_out += event.output_tokens as u64;
            })
            .or_insert(TierMetrics {
                tier: event.tier.clone(),
                calls: 1,
                cost_usd: event.cost_usd,
                tokens_in: event.input_tokens as u64,
                tokens_out: event.output_tokens as u64,
                cache_savings_usd: 0.0,
            });
    }

    pub async fn get_snapshot(&self, window: WindowSize) -> AggregatedMetrics {
        let buffer = match window {
            WindowSize::OneMinute => &self.window_1m,
            WindowSize::FiveMinutes => &self.window_5m,
            WindowSize::TenMinutes => &self.window_10m,
        };

        buffer.lock().await.aggregate()
    }
}
```

### 4. BatchWriter (Async Persistence)

```rust
pub struct BatchWriter {
    buffer: Arc<Mutex<Vec<ApiCallEvent>>>,
    db: Arc<MetricsDatabase>,
    batch_size: usize,
}

impl BatchWriter {
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

        self.db.write_calls_batch(&buffer).await?;
        buffer.clear();

        Ok(())
    }

    pub async fn run_periodic_flush(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let mut buffer = self.buffer.lock().await;
            if let Err(e) = self.flush_internal(&mut buffer).await {
                tracing::error!("Failed to flush batch: {}", e);
            }
        }
    }
}
```

### 5. REST API Endpoints

```rust
// In server.rs

// Summary stats endpoint
async fn metrics_stats(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<StatsResponse>, ServerError> {
    let stats = state.metrics_backend
        .get_stats(WindowSize::TenMinutes)
        .await?;

    Ok(Json(stats))
}

// Real-time updates endpoint
async fn metrics_realtime(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<RealtimeResponse>, ServerError> {
    let snapshot_1m = state.metrics_backend.aggregator
        .get_snapshot(WindowSize::OneMinute)
        .await;

    let rate = state.metrics_backend.aggregator
        .get_current_rate();

    Ok(Json(RealtimeResponse {
        current_rate: rate,
        window_1m: snapshot_1m,
    }))
}

// Add routes
let app = Router::new()
    .route("/api/metrics/stats", get(metrics_stats))
    .route("/api/metrics/realtime", get(metrics_realtime))
    .route("/api/metrics/tiers/:tier", get(metrics_tier))
    .route("/api/metrics/recent", get(metrics_recent))
    .route("/api/metrics/health", get(metrics_health))
    .with_state(state);
```

---

## 💰 Cost Calculation

### Pricing Table (January 2025)

| Model | Tier | Input ($/1M) | Output ($/1M) | Cache Read ($/1M) |
|-------|------|--------------|---------------|-------------------|
| claude-opus-4-1-20250805 | Opus | $15.00 | $75.00 | $1.50 |
| claude-sonnet-4-5-20250929 | Sonnet | $3.00 | $15.00 | $0.30 |
| claude-haiku-4-5-20251001 | Haiku | $0.80 | $4.00 | $0.08 |

### Calculation Formula

```rust
pub fn calculate_cost(
    input_tokens: u32,
    output_tokens: u32,
    input_cost_per_1m: f64,
    output_cost_per_1m: f64,
) -> f64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * input_cost_per_1m;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * output_cost_per_1m;

    input_cost + output_cost
}

// Example: Opus call (5k input, 2k output)
let cost = calculate_cost(5000, 2000, 15.00, 75.00);
// Result: $0.225 ($0.075 input + $0.150 output)
```

---

## 📁 File Structure

```
cco/
├── src/
│   ├── metrics_backend.rs      # Main backend (record_call, get_stats)
│   ├── metrics_db.rs           # SQLite persistence
│   ├── metrics_aggregator.rs   # In-memory ring buffers
│   ├── batch_writer.rs         # Async batch writes
│   ├── cost_calculator.rs      # Cost calculation
│   ├── query_cache.rs          # 1s TTL cache
│   └── server.rs               # API endpoints (updated)
├── migrations/
│   ├── 001_initial_schema.sql
│   └── 002_seed_pricing.sql
└── tests/
    ├── metrics_backend_tests.rs
    └── metrics_integration_tests.rs
```

---

## 🧪 Testing Commands

```bash
# Unit tests
cargo test --test metrics_backend_tests
cargo test --test metrics_aggregator_tests
cargo test --test cost_calculator_tests

# Integration tests
cargo test --test metrics_integration_tests

# Benchmarks
cargo bench --bench metrics_write_bench
cargo bench --bench metrics_query_bench

# Coverage
cargo tarpaulin --out Html --output-dir coverage

# Run server
cargo run -- daemon start --port 3000

# Test API
curl http://localhost:3000/api/metrics/stats | jq
curl http://localhost:3000/api/metrics/realtime | jq
curl http://localhost:3000/api/metrics/health | jq
```

---

## 🚨 Common Issues & Solutions

### Issue 1: Database locked

**Symptom**: "database is locked" error

**Solution**: Enable WAL mode
```sql
PRAGMA journal_mode=WAL;
```

### Issue 2: High memory usage

**Symptom**: >100MB memory for ring buffers

**Solution**: Reduce ring buffer size or eviction window
```rust
// Reduce from 10k to 5k max events
RingBuffer::with_capacity(5000, Duration::from_secs(60))
```

### Issue 3: Slow queries

**Symptom**: API response >10ms

**Solution**: Add indexes
```sql
CREATE INDEX idx_timestamp_tier ON api_calls(timestamp, tier);
CREATE INDEX idx_model_timestamp ON api_calls(model_used, timestamp);
```

### Issue 4: Batch flush timeout

**Symptom**: Events not persisting within 5 seconds

**Solution**: Reduce batch size
```rust
BatchWriter::new(db, 50)  // Reduce from 100 to 50
```

---

## 📈 Performance Targets

| Metric | Target | How to Verify |
|--------|--------|---------------|
| Record latency | <1ms | Benchmark: 10k calls in <10s |
| Query latency | <10ms | `curl` response time |
| Memory usage | <100MB | `htop` or `ps aux` |
| Write throughput | 10k calls/min | Load test with `ab` |
| Database size | <500MB/month | Check `metrics.db` size |

---

## 🔗 Integration Checklist

- [ ] Update `ServerState` with `metrics_backend: Arc<MetricsBackend>`
- [ ] Modify `chat_completion` to call `record_call()`
- [ ] Add `/api/metrics/*` routes to server
- [ ] Update `analytics.rs` to use metrics backend
- [ ] Test with TUI dashboard

---

## 📞 Support

- Architecture questions → See `METRICS_BACKEND_ARCHITECTURE.md`
- Implementation tasks → See `METRICS_IMPLEMENTATION_CHECKLIST.md`
- API reference → See `METRICS_API_REFERENCE.md` (to be created)
