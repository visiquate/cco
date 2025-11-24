# Metrics Cache Implementation - Test Report

## Executive Summary

**Issue**: `/api/stats` endpoint takes ~15 seconds to respond due to scanning and parsing all Claude project JSONL files on every request.

**Root Cause**: The `load_all_projects_metrics()` function in `server.rs`:
- Scans `~/.claude/projects/` directory
- Iterates through ALL project subdirectories
- Parses ALL `.jsonl` files in each project
- Aggregates metrics from scratch on EVERY request
- No caching mechanism

**Current Implementation**: In-memory circular buffers for `recent_activities` and `recent_decisions` (100 items max), but **NO caching** for the expensive metrics aggregation.

## Problem Analysis

### Current Code Flow

```rust
async fn stats(State(state): State<Arc<DaemonState>>) -> Json<StatsResponse> {
    // ❌ This runs on EVERY request
    let (claude_metrics, mut cost_by_project) = match load_all_projects_metrics().await {
        Ok((metrics, projects)) => (metrics, projects),
        Err(e) => (crate::claude_history::ClaudeMetrics::default(), Vec::new())
    };
    // ... rest of response building
}
```

### `load_all_projects_metrics()` Performance

Located in `server.rs` lines 696-824:

```rust
async fn load_all_projects_metrics() -> anyhow::Result<(
    crate::claude_history::ClaudeMetrics,
    Vec<ProjectChartData>,
)> {
    // 1. List ~/.claude/projects/ directory
    // 2. For each project subdirectory:
    //    - Call load_claude_project_metrics()
    //    - Which reads ALL .jsonl files
    //    - Parses EVERY line
    //    - Aggregates tokens and costs
    // 3. Deduplicates canonical paths
    // 4. Returns aggregated results
}
```

**Complexity**: O(P × F × L) where:
- P = number of projects (10-100+)
- F = number of .jsonl files per project (10-1000+)
- L = number of lines per file (100-10,000+)

**Measured Performance**: ~15 seconds with typical project history

## Required Solution: Metrics Cache

### Design Requirements

1. **Cache Structure**
   - Store last aggregated metrics
   - Store timestamp of last refresh
   - Thread-safe (Arc<Mutex<>>)
   - Part of DaemonState

2. **Refresh Strategy**
   - Lazy refresh: Check age on request, refresh if stale
   - Background refresh: Periodic task updates cache
   - TTL: 5 seconds (configurable)

3. **Concurrency**
   - Multiple requests can read cached data simultaneously
   - Only one refresh operation at a time
   - Non-blocking reads during refresh

### Proposed Implementation

```rust
// In DaemonState
pub struct MetricsCache {
    data: Arc<Mutex<CachedMetricsData>>,
    refresh_interval: Duration,
}

struct CachedMetricsData {
    metrics: crate::claude_history::ClaudeMetrics,
    projects: Vec<ProjectChartData>,
    last_updated: Instant,
}

impl MetricsCache {
    fn new(refresh_interval_secs: u64) -> Self { ... }

    async fn get_or_refresh(&self) -> (ClaudeMetrics, Vec<ProjectChartData>) {
        let data = self.data.lock().unwrap();
        if data.last_updated.elapsed() < self.refresh_interval {
            return (data.metrics.clone(), data.projects.clone());
        }
        drop(data);

        // Refresh logic
        let (new_metrics, new_projects) = load_all_projects_metrics().await?;

        let mut data = self.data.lock().unwrap();
        data.metrics = new_metrics;
        data.projects = new_projects;
        data.last_updated = Instant::now();

        (data.metrics.clone(), data.projects.clone())
    }
}
```

## Test Plan

### 1. Build & Compile Tests ✅
```bash
cargo build --release
cargo test --lib
```

**Status**: ✅ PASS
- Clean build with 1 warning (deprecated before_exec)
- All library tests pass

### 2. Unit Tests for MetricsCache

**Required Tests**:
- [ ] `test_cache_new()` - Creates empty cache
- [ ] `test_cache_first_access()` - First call loads metrics
- [ ] `test_cache_hit()` - Second call within TTL returns cached data
- [ ] `test_cache_expiration()` - After TTL, refreshes data
- [ ] `test_cache_concurrent_reads()` - Multiple readers don't block
- [ ] `test_cache_single_refresh()` - Only one refresh at a time
- [ ] `test_cache_thread_safety()` - Stress test with 100 concurrent requests

### 3. Performance Benchmark

**Baseline (Current)**:
```bash
$ time curl -s http://127.0.0.1:3000/api/stats | jq '.project.name'
"CCO Daemon"
15.394s total  # ❌ TOO SLOW
```

**Target (With Cache)**:
```bash
$ time curl -s http://127.0.0.1:3000/api/stats | jq '.project.name'
"CCO Daemon"
<0.010s total  # ✅ 1500x FASTER
```

**Load Test**:
```bash
# Apache Bench - 1000 requests, 10 concurrent
ab -n 1000 -c 10 http://127.0.0.1:3000/api/stats

# Expected with cache:
# - Requests per second: >500
# - Mean time: <20ms
# - p95: <50ms
# - p99: <100ms
# - No timeouts
```

### 4. TUI Timeout Verification

**Test Procedure**:
1. Start daemon with new code
2. Run: `cco monitor`
3. Observe metrics update every 5 seconds
4. Run for 5+ minutes continuously

**Expected Results**:
- ✅ No timeout errors
- ✅ Smooth data updates
- ✅ Consistent 5-second refresh cycle
- ✅ No UI freezes or hangs

### 5. Integration Tests

**Test Scenarios**:
```bash
# Scenario 1: Rapid sequential requests
for i in {1..10}; do
    curl -s http://127.0.0.1:3000/api/stats > /dev/null
done

# Scenario 2: Concurrent requests
for i in {1..10}; do
    curl -s http://127.0.0.1:3000/api/stats > /dev/null &
done
wait

# Scenario 3: Verify data consistency
curl -s http://127.0.0.1:3000/api/stats | jq '.project.cost' # A
sleep 6  # Wait past TTL
curl -s http://127.0.0.1:3000/api/stats | jq '.project.cost' # B
# A and B should match (unless new activity)
```

### 6. Memory Leak Test

**Test Procedure**:
```bash
# Monitor daemon memory over 1 hour
ps aux | grep "cco daemon" | awk '{print $6}'  # Initial RSS

# Generate load
while true; do
    curl -s http://127.0.0.1:3000/api/stats > /dev/null
    sleep 1
done &

# After 1 hour
ps aux | grep "cco daemon" | awk '{print $6}'  # Final RSS
```

**Expected Results**:
- Initial memory: ~40-60 MB
- Final memory: ~50-70 MB (bounded growth only)
- No continuous growth pattern
- Stable after initial warmup

## Current Test Results

### Build Status: ✅ PASS
```
Finished `release` profile [optimized] target(s) in 1.05s
```

### Performance Status: ❌ FAIL
```
Current: 15.394s per request
Target:  <0.010s per request
Gap:     1539x too slow
```

### Root Cause Confirmed
The `/api/stats` endpoint has **NO caching** - it re-parses all project history files on every single request.

## Recommendations

### Immediate Action Required
1. **Implement MetricsCache** in `DaemonState`
2. **Add cache field** with 5-second TTL
3. **Modify stats() handler** to use `cache.get_or_refresh()`
4. **Add unit tests** for cache behavior
5. **Benchmark before/after** to verify 100x improvement

### Expected Impact
- Response time: 15s → <10ms (1500x faster)
- TUI responsiveness: No more timeouts
- Server load: 99.9% reduction in I/O
- User experience: Instant metrics

### Code Changes Required
- `src/daemon/server.rs`: Add `MetricsCache` struct and impl
- `src/daemon/server.rs`: Add cache field to `DaemonState`
- `src/daemon/server.rs`: Modify `stats()` to use cache
- `tests/metrics_cache_tests.rs`: Comprehensive test suite

## Next Steps
1. ✅ Problem analysis complete
2. ✅ Test plan documented
3. ⏭️  Implement MetricsCache
4. ⏭️  Write unit tests
5. ⏭️  Benchmark improvements
6. ⏭️  Verify TUI compatibility
7. ⏭️  Memory leak testing

## Appendix: Circular Buffer Status

The existing circular buffers (`recent_activities`, `recent_decisions`) are **working correctly**:
- Capacity: 100 items max
- Tested in: `daemon::server::tests::test_circular_buffer_capacity`
- Purpose: Store recent events, NOT metrics aggregation
- Performance: O(1) access, bounded memory

**Note**: These are **NOT** a metrics cache - they store individual events, not aggregated metrics.
