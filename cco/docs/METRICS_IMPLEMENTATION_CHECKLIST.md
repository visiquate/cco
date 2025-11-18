# Metrics Backend Implementation Checklist

**Project**: CCO API Cost Monitoring Daemon
**Architecture**: See `METRICS_BACKEND_ARCHITECTURE.md`
**Target Date**: Week of 2025-01-20

---

## Phase 1: Database Foundation (Day 1-2)

### Tasks

- [ ] Create SQLite migration files
  - [ ] `migrations/001_initial_schema.sql` (api_calls, metrics_aggregated, model_tiers, sessions, config tables)
  - [ ] `migrations/002_seed_pricing.sql` (Opus/Sonnet/Haiku pricing data)
  - [ ] `migrations/003_add_indexes.sql` (performance indexes)

- [ ] Implement `MetricsDatabase` struct
  - [ ] Connection pool initialization
  - [ ] Platform-specific path resolution (Mac/Windows)
  - [ ] Migration runner
  - [ ] Health check query

- [ ] Create `src/metrics_db.rs`
  - [ ] `write_call(event: &ApiCallEvent)` - Insert single call
  - [ ] `write_calls_batch(events: &[ApiCallEvent])` - Bulk insert
  - [ ] `get_calls_recent(limit: usize)` - Recent calls query
  - [ ] `get_tier_summary(window_secs: u64)` - Tier breakdown
  - [ ] Database cleanup/archival functions

**Verification**:
```bash
cargo test --test metrics_db_tests
sqlite3 ~/.local/share/cco/metrics.db ".schema"
sqlite3 ~/.local/share/cco/metrics.db "SELECT COUNT(*) FROM api_calls;"
```

---

## Phase 2: In-Memory Aggregation (Day 3-4)

### Tasks

- [ ] Implement `MetricsAggregator` struct
  - [ ] Ring buffers (1m/5m/10m windows)
  - [ ] Per-tier counters (Opus/Sonnet/Haiku)
  - [ ] Rate tracker (calls/min, $/min)
  - [ ] Thread-safe operations (Arc/Mutex/AtomicU64)

- [ ] Create `src/metrics_aggregator.rs`
  - [ ] `record_call(event: ApiCallEvent)` - Update all windows
  - [ ] `get_snapshot(window: WindowSize)` - Get aggregated metrics
  - [ ] `get_tier_breakdown()` - Per-tier stats
  - [ ] `get_current_rate()` - Real-time rate

- [ ] Implement `RingBuffer<ApiCallEvent>`
  - [ ] Time-based eviction (auto-cleanup expired events)
  - [ ] Efficient push/pop operations
  - [ ] Aggregation functions (sum, avg, percentiles)

**Verification**:
```bash
cargo test --test metrics_aggregator_tests
# Load test: 1000 calls/sec for 60 seconds
cargo bench --bench metrics_aggregator_bench
```

---

## Phase 3: Backend Integration (Day 5-6)

### Tasks

- [ ] Create `MetricsBackend` struct
  - [ ] Combine database + aggregator + batch writer
  - [ ] Error recovery (fallback to logs if DB unavailable)
  - [ ] Health monitoring

- [ ] Implement `BatchWriter`
  - [ ] Buffering (100 calls or 5 seconds)
  - [ ] Periodic flush task (tokio spawn)
  - [ ] Transaction handling

- [ ] Create `src/metrics_backend.rs`
  - [ ] `initialize()` - Setup DB + aggregator + batch writer
  - [ ] `record_call(event: ApiCallEvent)` - Main entry point
  - [ ] `get_stats(window: WindowSize)` - Query stats
  - [ ] `get_tier_stats(tier: &str)` - Per-tier stats
  - [ ] `health_check()` - Backend health

**Verification**:
```bash
cargo test --test metrics_backend_tests
# End-to-end test: Record 100 calls, verify DB + aggregator consistency
cargo test --test e2e_metrics_test
```

---

## Phase 4: Server Integration (Day 7-8)

### Tasks

- [ ] Update `ServerState` in `server.rs`
  ```rust
  pub struct ServerState {
      // ... existing fields ...
      pub metrics_backend: Arc<MetricsBackend>,
  }
  ```

- [ ] Modify `chat_completion` endpoint
  - [ ] Extract latency measurement
  - [ ] Call `metrics_backend.record_call()` after response
  - [ ] Include tier classification

- [ ] Add REST API endpoints
  - [ ] `GET /api/metrics/stats` - Summary stats
  - [ ] `GET /api/metrics/realtime` - Real-time updates
  - [ ] `GET /api/metrics/tiers/{tier}` - Per-tier breakdown
  - [ ] `GET /api/metrics/recent?limit=20` - Recent calls
  - [ ] `GET /api/metrics/health` - Backend health

- [ ] Implement `QueryCache` with 1s TTL
  - [ ] Cache `/api/metrics/stats` responses
  - [ ] Prevent redundant DB queries

**Verification**:
```bash
# Start server
cargo run -- daemon start --port 3000

# Test endpoints
curl http://localhost:3000/api/metrics/stats | jq
curl http://localhost:3000/api/metrics/realtime | jq
curl http://localhost:3000/api/metrics/tiers/sonnet | jq
curl http://localhost:3000/api/metrics/health | jq

# Load test
ab -n 1000 -c 10 http://localhost:3000/api/metrics/stats
```

---

## Phase 5: Configuration & CLI (Day 9-10)

### Tasks

- [ ] Create pricing configuration template
  - [ ] `~/.config/cco/pricing.toml` with Opus/Sonnet/Haiku defaults
  - [ ] TOML parsing and validation

- [ ] Implement CLI commands
  - [ ] `cco config pricing list` - Show current pricing
  - [ ] `cco config pricing update <model>` - Update pricing
  - [ ] `cco config pricing reset` - Reset to defaults

- [ ] Add configuration management
  - [ ] `src/config.rs` - Load/save pricing config
  - [ ] Hot-reload pricing without restart
  - [ ] Validation (prevent negative costs)

**Verification**:
```bash
# List pricing
cco config pricing list

# Update pricing
cco config pricing update claude-opus-4-1-20250805 \
  --input 15.00 --output 75.00

# Verify update
cco config pricing list | grep opus
```

---

## Phase 6: Testing & Optimization (Day 11-14)

### Tasks

- [ ] Unit tests (target: 90%+ coverage)
  - [ ] `tests/metrics_db_tests.rs` - Database operations
  - [ ] `tests/metrics_aggregator_tests.rs` - Aggregation logic
  - [ ] `tests/metrics_backend_tests.rs` - Backend integration
  - [ ] `tests/cost_calculator_tests.rs` - Cost calculations

- [ ] Integration tests
  - [ ] `tests/metrics_integration_tests.rs` - End-to-end flow
  - [ ] Mock API responses
  - [ ] Verify DB persistence + in-memory consistency

- [ ] Performance benchmarks
  - [ ] `benches/metrics_write_bench.rs` - Write throughput (target: 10k/min)
  - [ ] `benches/metrics_query_bench.rs` - Query latency (target: <10ms)
  - [ ] Memory profiling (target: <100MB for ring buffers)

- [ ] Optimization
  - [ ] Database indexing (verify with EXPLAIN QUERY PLAN)
  - [ ] Batch size tuning (100 calls vs 5 seconds)
  - [ ] Query cache effectiveness (measure hit rate)

**Verification**:
```bash
# Run all tests
cargo test --all

# Coverage report
cargo tarpaulin --out Html --output-dir coverage

# Benchmarks
cargo bench --bench metrics_write_bench
cargo bench --bench metrics_query_bench

# Memory profiling
valgrind --tool=massif target/release/cco daemon start
```

---

## Phase 7: Documentation & Deployment (Day 15)

### Tasks

- [ ] Update `README.md` with metrics features
- [ ] Create user guide for metrics API
- [ ] Document pricing configuration
- [ ] Add troubleshooting guide

**Deliverables**:
- [ ] `docs/METRICS_USER_GUIDE.md`
- [ ] `docs/METRICS_API_REFERENCE.md`
- [ ] `docs/METRICS_TROUBLESHOOTING.md`

---

## Success Criteria

| Metric | Target | Verification |
|--------|--------|--------------|
| Write latency | <1ms | Benchmark: 10k calls in <10s |
| Query latency | <10ms | API response time <10ms |
| Memory usage | <100MB | Ring buffers + tier counters |
| Database size | <500MB/month | 1M calls = ~400MB |
| Test coverage | >90% | `cargo tarpaulin` |
| API uptime | 99.9% | No crashes under load test |

---

## Integration Points Checklist

- [x] `analytics.rs` - Emit events to metrics backend
- [ ] `server.rs` - Add `/api/metrics/*` endpoints
- [ ] `router.rs` - Use cost calculator from metrics backend
- [ ] `cache.rs` - Report cache savings to metrics
- [ ] `agents_config.rs` - Track per-agent costs

---

## Deployment Checklist

- [ ] Database migration on first run (auto-create schema)
- [ ] Platform-specific paths (Mac: `~/Library/Application Support/cco/`, Windows: `%LOCALAPPDATA%\cco\`)
- [ ] Log rotation (daily, max 10 files)
- [ ] Graceful shutdown (flush batch buffer)
- [ ] Error recovery (write to fallback log if DB fails)

---

## Rollback Plan

If metrics backend causes issues:

1. **Disable metrics recording**:
   ```rust
   // In server.rs
   if let Some(backend) = &state.metrics_backend {
       let _ = backend.record_call(event).await;  // Ignore errors
   }
   ```

2. **Fall back to existing analytics**:
   - Keep `AnalyticsEngine` working as-is
   - Metrics backend is additive, not replacement

3. **Database cleanup**:
   ```bash
   rm ~/.local/share/cco/metrics.db
   ```

---

## Post-Implementation Tasks

- [ ] Create automated tests in CI/CD
- [ ] Add Prometheus exporter (future enhancement)
- [ ] Implement cost alerts (notify if >$X/day)
- [ ] Add GraphQL API (alternative to REST)
- [ ] Support for other providers (OpenAI, Ollama)

---

## Timeline Summary

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. Database Foundation | 2 days | SQLite schema, MetricsDatabase |
| 2. In-Memory Aggregation | 2 days | MetricsAggregator, RingBuffer |
| 3. Backend Integration | 2 days | MetricsBackend, BatchWriter |
| 4. Server Integration | 2 days | REST API endpoints, QueryCache |
| 5. Configuration & CLI | 2 days | Pricing config, CLI commands |
| 6. Testing & Optimization | 4 days | Tests, benchmarks, optimization |
| 7. Documentation | 1 day | User guides, API reference |
| **Total** | **15 days** | **Production-ready metrics backend** |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Database corruption | Regular backups, WAL mode, fsync on commit |
| Memory leaks (ring buffers) | Bounded buffers, eviction policy, memory tests |
| Performance degradation | Batching, indexing, query caching, benchmarks |
| Data loss on crash | Batch flush timeout (5s), graceful shutdown |
| API key exposure | Environment variables only, never log keys |

---

## Contact

- **Backend Architect**: This document
- **Chief Architect**: Final review and approval
- **TUI Developer**: Dashboard integration
- **Test Engineer**: Quality assurance and validation
