# CCO Metrics Backend - System Architecture Diagrams

**Visual Reference for Implementation Team**
**Version**: 2025.11.2

---

## 1. High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Claude Code Agent                                      │
│                   (Python Specialist / Test Engineer / etc.)                     │
└───────────────────────────────────┬─────────────────────────────────────────────┘
                                    │
                                    │ POST /v1/chat/completions
                                    │ {"model": "claude-sonnet-4-5-20250929", ...}
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           CCO Proxy Server (Rust)                                │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  │  chat_completion() Handler                                               │  │
│  │                                                                          │  │
│  │  1. Start timer (for latency measurement)                               │  │
│  │  2. Model override detection (agent type → configured model)            │  │
│  │  3. Proxy request to Claude API                                         │  │
│  │  4. Calculate cost (using ModelRouter pricing)                          │  │
│  │  5. Record metrics → MetricsBackend.record_call()                       │  │
│  └────────────────────────┬─────────────────────────────────────────────────┘  │
│                           │                                                     │
│                           ▼                                                     │
│  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  │  MetricsBackend (Orchestrator)                                           │  │
│  │                                                                          │  │
│  │  • Receives: ApiCallEvent (model, tokens, cost, latency, tier)          │  │
│  │  • Distributes to:                                                       │  │
│  │    - MetricsAggregator (in-memory, real-time)                          │  │
│  │    - BatchWriter (async persistence)                                    │  │
│  │    - QueryCache (1s TTL invalidation)                                   │  │
│  └────────────────────────┬─────────────────────────────────────────────────┘  │
│                           │                                                     │
└───────────────────────────┼─────────────────────────────────────────────────────┘
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
          ▼                 ▼                 ▼
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │In-Memory│      │Batch    │      │Query    │
    │Aggrega- │      │Writer   │      │Cache    │
    │tor      │      │(Async)  │      │(1s TTL) │
    └────┬────┘      └────┬────┘      └────┬────┘
         │                │                │
         │                ▼                │
         │         ┌─────────────┐         │
         │         │  SQLite DB  │         │
         │         │ metrics.db  │         │
         │         └─────────────┘         │
         │                                 │
         └─────────────────┬───────────────┘
                           │
                           ▼
                 ┌──────────────────┐
                 │  REST API        │
                 │  /api/metrics/*  │
                 └────────┬─────────┘
                          │
                          ▼
                 ┌──────────────────┐
                 │  TUI Dashboard   │
                 │  (ratatui)       │
                 └──────────────────┘
```

---

## 2. Data Flow Sequence

```
┌──────┐     ┌──────┐     ┌──────┐     ┌──────┐     ┌──────┐     ┌──────┐
│Agent │     │Proxy │     │Backend│    │Aggr  │     │Batch │     │SQLite│
└──┬───┘     └──┬───┘     └──┬───┘     └──┬───┘     └──┬───┘     └──┬───┘
   │             │            │            │            │            │
   │ API Call    │            │            │            │            │
   ├────────────>│            │            │            │            │
   │             │            │            │            │            │
   │             │ Start timer│            │            │            │
   │             ├───────────>│            │            │            │
   │             │            │            │            │            │
   │             │ Calc cost  │            │            │            │
   │             ├───────────>│            │            │            │
   │             │            │            │            │            │
   │             │ record_call│            │            │            │
   │             ├───────────>│            │            │            │
   │             │            │            │            │            │
   │             │            │ Update ring│            │            │
   │             │            │ buffers    │            │            │
   │             │            ├───────────>│            │            │
   │             │            │            │            │            │
   │             │            │ Queue event│            │            │
   │             │            ├───────────────────────>│            │
   │             │            │            │            │            │
   │             │ Response   │            │            │            │
   │<────────────┤            │            │            │            │
   │             │            │            │            │            │
   │             │            │            │   Flush (5s interval)   │
   │             │            │            │   or 100 calls          │
   │             │            │            │            ├───────────>│
   │             │            │            │            │            │
   │             │            │            │            │   Bulk     │
   │             │            │            │            │   INSERT   │
   │             │            │            │            │<───────────┤
   │             │            │            │            │            │
   └─────        └─────       └─────       └─────       └─────       └─────

   [TUI Dashboard polls /api/metrics/stats every 1 second]

   ┌──────┐     ┌──────┐     ┌──────┐     ┌──────┐
   │  TUI │     │ API  │     │Cache │     │Aggr  │
   └──┬───┘     └──┬───┘     └──┬───┘     └──┬───┘
      │             │            │            │
      │ GET /stats  │            │            │
      ├────────────>│            │            │
      │             │            │            │
      │             │ Check cache│            │
      │             ├───────────>│            │
      │             │            │            │
      │             │   Hit (1s) │            │
      │             │<───────────┤            │
      │             │            │            │
      │ JSON (10ms) │            │            │
      │<────────────┤            │            │
      │             │            │            │
      └─────        └─────       └─────       └─────
```

---

## 3. Component Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│                         MetricsBackend                                │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Public API                                                  │    │
│  │                                                              │    │
│  │  • async fn record_call(event: ApiCallEvent) -> Result<()>  │    │
│  │  • async fn get_stats(window: WindowSize) -> StatsResponse  │    │
│  │  • async fn get_tier_stats(tier: &str) -> TierStats         │    │
│  │  • async fn health_check() -> HealthStatus                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │
│  │MetricsAggregator│  │  BatchWriter    │  │   QueryCache    │     │
│  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤     │
│  │                 │  │                 │  │                 │     │
│  │ window_1m:      │  │ buffer:         │  │ cache:          │     │
│  │  RingBuffer     │  │  Vec<Event>     │  │  DashMap        │     │
│  │                 │  │                 │  │                 │     │
│  │ window_5m:      │  │ batch_size:     │  │ ttl: 1s         │     │
│  │  RingBuffer     │  │  100            │  │                 │     │
│  │                 │  │                 │  │ get_or_compute()│     │
│  │ window_10m:     │  │ flush_interval: │  │                 │     │
│  │  RingBuffer     │  │  5s             │  │                 │     │
│  │                 │  │                 │  │                 │     │
│  │ tier_metrics:   │  │ db: Database    │  │                 │     │
│  │  DashMap        │  │                 │  │                 │     │
│  │                 │  │ flush_task()    │  │                 │     │
│  │ record(event)   │  │                 │  │                 │     │
│  │ get_snapshot()  │  │ write(event)    │  │                 │     │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘     │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  MetricsDatabase                                             │    │
│  │                                                              │    │
│  │  • pool: SqlitePool                                         │    │
│  │  • async fn write_call(event: &ApiCallEvent)                │    │
│  │  • async fn write_calls_batch(events: &[ApiCallEvent])      │    │
│  │  • async fn query_recent(limit: usize)                      │    │
│  │  • async fn query_tier_summary(window_secs: u64)            │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

---

## 4. RingBuffer Data Structure

```
┌────────────────────────────────────────────────────────────────┐
│  RingBuffer<ApiCallEvent>                                      │
│  (1-minute window example)                                     │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Time: 10:00:00 ──────────────────────────────> 10:01:00      │
│                                                                │
│  buffer: VecDeque<ApiCallEvent>                               │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐     │
│  │Event│Event│Event│Event│Event│Event│Event│Event│Event│     │
│  │ 1   │ 2   │ 3   │ 4   │ 5   │ 6   │ 7   │ 8   │ 9   │ ... │
│  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘     │
│    ↑                                                    ↑      │
│  Oldest                                              Newest    │
│  (10:00:00)                                        (10:00:59)  │
│                                                                │
│  Operations:                                                   │
│  ┌──────────────────────────────────────────────────────┐     │
│  │ push(event):                                         │     │
│  │   1. Add to back of deque                            │     │
│  │   2. While front.timestamp < now - 60s:              │     │
│  │        pop_front()  (evict expired)                  │     │
│  └──────────────────────────────────────────────────────┘     │
│                                                                │
│  ┌──────────────────────────────────────────────────────┐     │
│  │ aggregate() -> AggregatedMetrics:                    │     │
│  │   1. Iterate all events in buffer                    │     │
│  │   2. Sum: calls, costs, tokens                       │     │
│  │   3. Collect: latencies (for percentiles)            │     │
│  │   4. Return aggregated metrics                       │     │
│  └──────────────────────────────────────────────────────┘     │
│                                                                │
│  Memory: ~10 bytes per event × 10,000 events = ~100 KB        │
│          (per window, 3 windows total = ~300 KB)              │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

---

## 5. Database Schema Relationships

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Database: metrics.db                         │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│   api_calls      │
├──────────────────┤       ┌──────────────────────┐
│ id (PK)          │       │  metrics_aggregated  │
│ timestamp        │       ├──────────────────────┤
│ session_id (FK)  ├──────>│ window_start         │
│ model_used       │       │ window_size_seconds  │
│ tier             │       │ model                │
│ input_tokens     │       │ tier                 │
│ output_tokens    │       │ total_calls          │
│ cost_usd         │       │ total_cost_usd       │
│ latency_ms       │       │ avg_latency_ms       │
│ ...              │       │ calls_per_minute     │
└──────────────────┘       └──────────────────────┘
         │
         │
         ▼
┌──────────────────┐       ┌──────────────────────┐
│   sessions       │       │   model_tiers        │
├──────────────────┤       ├──────────────────────┤
│ session_id (PK)  │       │ model (PK)           │
│ started_at       │       │ tier                 │
│ ended_at         │       │ provider             │
│ total_cost_usd   │       │ input_cost_per_1m    │
│ total_calls      │       │ output_cost_per_1m   │
│ ...              │       │ cache_read_cost...   │
└──────────────────┘       └──────────────────────┘
                                      │
                                      │ Used for
                                      │ cost calc
                                      ▼
                           ┌──────────────────────┐
                           │  CostCalculator      │
                           │  (Rust component)    │
                           └──────────────────────┘

┌──────────────────┐
│     config       │
├──────────────────┤
│ key (PK)         │
│ value            │
│ description      │
│ updated_at       │
└──────────────────┘

Examples:
• batch_size = "100"
• batch_flush_seconds = "5"
• query_cache_ttl_seconds = "1"
```

---

## 6. Cost Calculation Flow

```
┌────────────────────────────────────────────────────────────────────┐
│  Cost Calculation for API Call                                    │
└────────────────────────────────────────────────────────────────────┘

Input:
┌─────────────────────────────────────────────────────────────┐
│ model: "claude-sonnet-4-5-20250929"                         │
│ input_tokens: 10,000                                        │
│ output_tokens: 3,000                                        │
│ cache_read_tokens: 9,000  (optional)                        │
│ cache_write_tokens: 0     (optional)                        │
└─────────────────────────────────────────────────────────────┘
                         │
                         ▼
        ┌─────────────────────────────────────┐
        │  1. Lookup pricing from model_tiers │
        └───────────────┬─────────────────────┘
                        │
                        ▼
        ┌─────────────────────────────────────────────────┐
        │ Pricing for "claude-sonnet-4-5-20250929":      │
        │   input_cost_per_1m = $3.00                     │
        │   output_cost_per_1m = $15.00                   │
        │   cache_read_cost_per_1m = $0.30                │
        └───────────────┬─────────────────────────────────┘
                        │
                        ▼
        ┌─────────────────────────────────────┐
        │  2. Calculate component costs       │
        └───────────────┬─────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────┐
│ Cache read cost:                                           │
│   (9,000 / 1,000,000) × $0.30 = $0.0027                   │
│                                                            │
│ New input cost:                                            │
│   ((10,000 - 9,000) / 1,000,000) × $3.00 = $0.003         │
│                                                            │
│ Output cost:                                               │
│   (3,000 / 1,000,000) × $15.00 = $0.045                   │
│                                                            │
│ Total actual cost: $0.0027 + $0.003 + $0.045 = $0.0507    │
└───────────────┬────────────────────────────────────────────┘
                │
                ▼
        ┌─────────────────────────────────────┐
        │  3. Calculate would-be cost         │
        │     (without cache)                 │
        └───────────────┬─────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────┐
│ Would-be cost (no cache):                                  │
│   (10,000 / 1,000,000) × $3.00 +                          │
│   (3,000 / 1,000,000) × $15.00 = $0.075                   │
│                                                            │
│ Savings: $0.075 - $0.0507 = $0.0243 (32% reduction)       │
└───────────────┬────────────────────────────────────────────┘
                │
                ▼
        ┌─────────────────────────────────────┐
        │  4. Return cost breakdown           │
        └───────────────┬─────────────────────┘
                        │
                        ▼
Output:
┌─────────────────────────────────────────────────────────────┐
│ {                                                           │
│   "actual_cost_usd": 0.0507,                               │
│   "would_be_cost_usd": 0.075,                              │
│   "savings_usd": 0.0243,                                   │
│   "savings_percent": 32.4                                  │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Tier Breakdown Visualization

```
┌────────────────────────────────────────────────────────────────────┐
│  API Cost Breakdown by Tier (Example Day)                         │
└────────────────────────────────────────────────────────────────────┘

Total Cost: $42.87
Total Calls: 1,523

┌─────────────────────────────────────────────────────────────────┐
│  Opus Tier (25% of calls, 75% of cost)                         │
│  ████████████████████████████████████████████████  $32.45      │
│                                                                 │
│  Model: claude-opus-4-1-20250805                               │
│  Calls: 123                                                    │
│  Tokens In: 500,000                                            │
│  Tokens Out: 250,000                                           │
│  Avg Cost/Call: $0.26                                          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  Sonnet Tier (58% of calls, 21% of cost)                      │
│  █████████████  $8.92                                          │
│                                                                 │
│  Model: claude-sonnet-4-5-20250929                             │
│  Calls: 890                                                    │
│  Tokens In: 1,500,000                                          │
│  Tokens Out: 600,000                                           │
│  Avg Cost/Call: $0.01                                          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  Haiku Tier (34% of calls, 4% of cost)                        │
│  ██  $1.50                                                     │
│                                                                 │
│  Model: claude-haiku-4-5-20251001                              │
│  Calls: 510                                                    │
│  Tokens In: 456,789                                            │
│  Tokens Out: 137,654                                           │
│  Avg Cost/Call: $0.003                                         │
└─────────────────────────────────────────────────────────────────┘

Key Insights:
• Opus is most expensive per call ($0.26 vs $0.01 vs $0.003)
• Sonnet has highest call volume (890 calls)
• Haiku is most cost-effective for high-volume tasks
• Opportunity: Migrate 10% of Opus calls → Save ~$3.25/day
```

---

## 8. Performance Optimization Flow

```
┌────────────────────────────────────────────────────────────────────┐
│  Query Performance Optimization Strategy                          │
└────────────────────────────────────────────────────────────────────┘

TUI Dashboard Request: GET /api/metrics/stats
                             │
                             ▼
                  ┌──────────────────────┐
                  │  1. Check QueryCache │
                  │     (1s TTL)         │
                  └──────────┬───────────┘
                             │
                  ┌──────────┴───────────┐
                  │                      │
         Cache Hit (98%)        Cache Miss (2%)
                  │                      │
                  ▼                      ▼
        ┌─────────────────┐   ┌──────────────────────┐
        │ Return cached   │   │ 2. Query in-memory   │
        │ JSON (<1ms)     │   │    MetricsAggregator │
        └─────────────────┘   └──────────┬───────────┘
                                         │
                                         ▼
                              ┌──────────────────────┐
                              │ 3. Compute stats     │
                              │    - Sum costs       │
                              │    - Sum tokens      │
                              │    - Calc rates      │
                              │    - Tier breakdown  │
                              └──────────┬───────────┘
                                         │
                                         ▼
                              ┌──────────────────────┐
                              │ 4. Store in cache    │
                              │    TTL = 1s          │
                              └──────────┬───────────┘
                                         │
                                         ▼
                              ┌──────────────────────┐
                              │ 5. Return JSON       │
                              │    (<10ms)           │
                              └──────────────────────┘

Performance Characteristics:
• Cache hit: <1ms (99% of requests)
• Cache miss: <10ms (1% of requests, in-memory only)
• Database query: NOT NEEDED for real-time stats
• TUI polling: 1 request/second
• Server load: Minimal (cached responses)
```

---

## 9. File Organization

```
cco/
├── src/
│   ├── metrics_backend.rs         # Main orchestrator (500 LOC)
│   │   ├── MetricsBackend
│   │   ├── record_call()
│   │   ├── get_stats()
│   │   └── health_check()
│   │
│   ├── metrics_db.rs               # SQLite persistence (300 LOC)
│   │   ├── MetricsDatabase
│   │   ├── write_call()
│   │   ├── write_calls_batch()
│   │   └── query_*()
│   │
│   ├── metrics_aggregator.rs      # In-memory aggregation (400 LOC)
│   │   ├── MetricsAggregator
│   │   ├── RingBuffer<T>
│   │   ├── record()
│   │   └── get_snapshot()
│   │
│   ├── batch_writer.rs             # Async batch writes (200 LOC)
│   │   ├── BatchWriter
│   │   ├── write()
│   │   └── run_periodic_flush()
│   │
│   ├── cost_calculator.rs          # Cost calculation (150 LOC)
│   │   ├── CostCalculator
│   │   ├── calculate_cost()
│   │   └── calculate_with_cache()
│   │
│   ├── query_cache.rs              # 1s TTL cache (100 LOC)
│   │   ├── QueryCache
│   │   └── get_or_compute()
│   │
│   └── server.rs                   # Updated endpoints (+200 LOC)
│       ├── metrics_stats()
│       ├── metrics_realtime()
│       ├── metrics_tier()
│       └── metrics_health()
│
├── migrations/
│   ├── 001_initial_schema.sql     # Core tables
│   ├── 002_seed_pricing.sql       # Pricing data
│   └── 003_add_indexes.sql        # Performance indexes
│
├── tests/
│   ├── metrics_backend_tests.rs   # Backend unit tests
│   ├── metrics_aggregator_tests.rs # Aggregation tests
│   ├── metrics_db_tests.rs        # Database tests
│   └── metrics_integration_tests.rs # End-to-end tests
│
└── docs/
    ├── METRICS_BACKEND_ARCHITECTURE.md      # Full spec (12k words)
    ├── METRICS_IMPLEMENTATION_CHECKLIST.md  # 15-day plan
    ├── METRICS_QUICK_REFERENCE.md           # Developer guide
    ├── METRICS_BACKEND_SUMMARY.md           # This summary
    └── METRICS_ARCHITECTURE_DIAGRAM.md      # This file
```

---

## 10. Error Recovery Flow

```
┌────────────────────────────────────────────────────────────────────┐
│  Error Recovery Strategy                                          │
└────────────────────────────────────────────────────────────────────┘

API Call → MetricsBackend.record_call(event)
                    │
                    ▼
         ┌──────────────────────┐
         │ 1. Update in-memory  │
         │    aggregator        │
         │    (ALWAYS SUCCEEDS) │
         └──────────┬───────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │ 2. Queue for batch   │
         │    write             │
         └──────────┬───────────┘
                    │
      ┌─────────────┴─────────────┐
      │                           │
   Success                     Failure
      │                           │
      ▼                           ▼
┌──────────┐         ┌────────────────────────┐
│ Return   │         │ 3. Try direct DB write │
│ Ok(())   │         └────────────┬───────────┘
└──────────┘                      │
                     ┌────────────┴────────────┐
                     │                         │
                  Success                  Failure
                     │                         │
                     ▼                         ▼
              ┌──────────┐       ┌──────────────────────┐
              │ Return   │       │ 4. Write to fallback │
              │ Ok(())   │       │    error log         │
              └──────────┘       │    ~/.local/share/   │
                                 │    cco/failed_       │
                                 │    metrics.jsonl     │
                                 └──────────┬───────────┘
                                            │
                                            ▼
                                 ┌──────────────────────┐
                                 │ 5. Log error         │
                                 │    Return Ok(())     │
                                 │    (non-blocking)    │
                                 └──────────────────────┘

Guarantees:
• In-memory aggregator ALWAYS updated (for real-time stats)
• API call never blocks waiting for DB write
• Failed events logged for manual recovery
• Dashboard continues working even if DB is down
```

---

**End of Architecture Diagrams**

For implementation details, see:
- `/Users/brent/git/cc-orchestra/cco/docs/METRICS_BACKEND_ARCHITECTURE.md`
- `/Users/brent/git/cc-orchestra/cco/docs/METRICS_IMPLEMENTATION_CHECKLIST.md`
- `/Users/brent/git/cc-orchestra/cco/docs/METRICS_QUICK_REFERENCE.md`
