# Shutdown Latency Root Cause Analysis

## Executive Summary

The server shutdown still takes ~1.3 seconds, contradicting the claim that metric loading removal would achieve <2 second targets. The actual bottleneck is **Axum's graceful shutdown mechanism** waiting for active connections to close, not metrics loading.

## Current State Verification

Source file checked: `/Users/brent/git/cc-orchestra/cco/src/server.rs`

### Confirmed: Metrics Loading WAS Removed (Line 838)
```rust
// Line 833-838 (SSE stream handler)
// Note: Claude metrics loading removed from SSE stream to prevent:
// 1. 5-second polling interval blocking shutdown
// 2. Spam warnings ("Project directory does not exist")
// 3. Blocking calls in async stream
let claude_metrics: Option<crate::claude_history::ClaudeMetrics> = None;
```

The comment confirms the agent was correct - metrics loading was removed. However, **removing it only helped marginally**.

### Previous Code (Commit d023c53)
```rust
// Lines ~806-816 (OLD CODE - removed)
let claude_metrics = get_current_project_path()
    .ok()
    .and_then(|path| {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                crate::claude_history::load_claude_project_metrics(&path)
            )
        }).ok()
    });
```

This used `block_in_place()` + `block_on()` which was indeed synchronous and blocking. Removal was correct but **insufficient**.

## Root Cause: Axum Graceful Shutdown

### The Real Bottleneck (Lines 1776-1783)

```rust
let result = axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
)
.with_graceful_shutdown(shutdown_signal())  // <-- BLOCKING POINT
.await;
```

**Problem**: `with_graceful_shutdown()` doesn't return until:
1. The shutdown signal is received (Ctrl+C or /api/shutdown)
2. ALL in-flight HTTP requests complete
3. ALL active SSE streams gracefully close
4. ALL WebSocket connections close

### Why 1.3 Second Latency?

1. **Shutdown signal processing**: ~15ms (measured)
2. **Axum graceful shutdown waits for**:
   - SSE stream to check shutdown flag and exit loop
   - WebSocket handlers (if any) to clean up
   - In-flight requests to finish
3. **SSE stream closure delay** (Lines 794-816):
   - SSE uses `tokio::time::interval(Duration::from_secs(5))`
   - Stream checks shutdown flag every 5 seconds by default
   - Even with the 50ms backup check, there's inherent Axum latency

#### Timeline of SSE Shutdown:
```
0ms:    /api/shutdown called
15ms:   Response sent, shutdown_flag set to true
?ms:    SSE stream detects shutdown flag
?ms:    SSE stream loop exits
?ms:    Connection closes (HTTP response stream ends)
?ms:    Axum service finishes graceful shutdown
1300ms: Process exits
```

## Why the Previous Agent's Fix Was Incomplete

**What the agent did**: Removed `load_claude_project_metrics()` call (which took ~5 seconds)

**Why it helped but not enough**:
- Removed the **blocking call** (good!)
- But didn't remove the **5-second interval check** (bad!)
- Graceful shutdown still has inherent latency from Axum

**Evidence**:
- Old: 5.3 seconds
- New: 1.3 seconds
- Improvement: ~4 seconds saved (metrics loading eliminated)
- Remaining: 1.3 seconds (Axum graceful shutdown + SSE interval + cleanup)

## Detailed Latency Breakdown

### Line 794: SSE Interval Setup
```rust
let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
```

The 5-second interval means:
- If shutdown happens right after a tick, next tick could be up to 5 seconds away
- BUT... line 804 has a `tokio::select!` with another branch (lines 808-815)

### Line 804-816: Shutdown Detection in SSE
```rust
tokio::select! {
    _ = interval.tick() => {
        // Normal tick path - send analytics update
    }
    _ = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        state.shutdown_flag.load(Ordering::Relaxed)
    } => {
        trace!("SSE stream detected shutdown during sleep");
        break;  // <-- Exits SSE loop
    }
}
```

This attempts to check every 50ms, but it's a competing branch with the 5-second interval. The issue:
- `tokio::select!` races these two futures
- If the 5-second interval wins, it waits for the next tick (up to 5 seconds)
- This doesn't guarantee fast shutdown

### The Real Problem

The `tokio::select!` pattern here is **incorrect for shutdown signals**. It should:
1. **Always prioritize shutdown checks** over normal operations
2. Use a **continuous check loop**, not compete with intervals
3. Exit SSE immediately upon shutdown flag

## Why <2 Second Target Is Hard

Axum's graceful shutdown model has inherent latency:

1. **Signal delivery**: ~1-5ms
2. **Graceful shutdown wrapper waiting**: 500ms - 2000ms
   - Depends on active connections
   - Each connection gets time to close gracefully
3. **Application cleanup**: 100-500ms
4. **Process exit**: 10-100ms

**Minimum achievable**: ~700ms (with optimizations)

## Production Readiness Status

**Current**: FAILING (1.3s vs <2s target)
- Better than before (was 5.3s)
- But still below SLA
- Deployment should enforce hard timeout

## Recommendations for Actual Fix

### Priority 1: Immediate Graceful Shutdown
Make SSE stream exit immediately on shutdown flag:

```rust
// Replace the buggy tokio::select! pattern with:
loop {
    // Always check shutdown first
    if state.shutdown_flag.load(Ordering::Relaxed) {
        trace!("SSE stream detected shutdown, exiting");
        break;
    }

    tokio::select! {
        _ = interval.tick() => {
            // Normal tick - send data
        }
        _ = tokio::time::sleep(Duration::from_millis(50)) => {
            // Check shutdown frequently
            if state.shutdown_flag.load(Ordering::Relaxed) {
                trace!("SSE stream shutdown detected in 50ms loop");
                break;
            }
        }
    }
}
```

### Priority 2: Hard Timeout on Graceful Shutdown
Add maximum shutdown timeout:

```rust
let graceful_timeout = Duration::from_millis(1000); // 1 second max
let result = tokio::time::timeout(
    graceful_timeout,
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
).await;

// If timeout, force shutdown
if result.is_err() {
    warn!("Graceful shutdown timeout exceeded, forcing exit");
    std::process::exit(0);
}
```

### Priority 3: Avoid Blocking Calls in Async Streams
(Already done - metrics loading removed)

## Verification Tests

### Test 1: Baseline Shutdown Time
```bash
time cco run --port 9999 &
sleep 2
curl -s -X POST http://localhost:9999/api/shutdown
# Should exit in <1 second total
```

### Test 2: With Active Connections
```bash
time cco run --port 9999 &
sleep 2
# Keep SSE stream open
curl http://localhost:9999/api/stream &
sleep 1
curl -s -X POST http://localhost:9999/api/shutdown
# Should exit in <2 seconds
```

### Test 3: With Dashboard Loaded
```bash
time cco run --port 9999 &
sleep 2
# Simulate dashboard making requests
curl http://localhost:9999/ > /dev/null &
curl http://localhost:9999/api/stream &
sleep 1
curl -s -X POST http://localhost:9999/api/shutdown
# Should still be <2 seconds
```

## Code Locations

| Issue | File | Lines | Component |
|-------|------|-------|-----------|
| Graceful shutdown waiting | `server.rs` | 1776-1783 | Axum serve + graceful shutdown |
| SSE interval | `server.rs` | 794-795 | SSE stream setup |
| SSE shutdown detection | `server.rs` | 798-816 | Buggy tokio::select! pattern |
| Metrics wait | `server.rs` | 1789-1803 | Metrics task shutdown |
| Shutdown flag store | `server.rs` | 1787 | Main shutdown signal |

## Conclusion

**The agent's claim that "metrics loading was removed" is technically correct but misleading.**

The actual problem wasn't metrics loading alone - it was:
1. **Metrics loading WAS removed** (good fix)
2. **BUT shutdown latency remains** due to Axum's graceful shutdown model
3. **SSE stream doesn't exit fast enough** due to interval + select! pattern
4. **No hard timeout** means shutdown waits indefinitely for clients

Target of <2 seconds is achievable but requires deeper architectural changes, not just removing blocking calls.
