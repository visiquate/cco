# Shutdown Performance Analysis - Technical Investigation

**Date**: 2025-11-17
**Investigator**: QA Engineer
**Priority**: CRITICAL
**Issue**: Shutdown takes 3-5 seconds instead of target < 2 seconds

---

## Problem Statement

When Ctrl+C is pressed, the application takes approximately 4 seconds to fully shutdown instead of the target 2 seconds. The delay appears to be a consistent 2-second gap between signal reception and shutdown initiation.

**Test Evidence**:
```
Log timestamps from test run:
02:14:54.219680Z - Received Ctrl+C signal
02:14:54.219710Z - Initiating graceful shutdown...
[2-SECOND GAP]
02:14:56.228839Z - Signaling background tasks to shutdown...
02:14:56.263819Z - Cleaning up PID file...
02:14:56.264001Z - Server shut down gracefully

Total shutdown time: ~5 seconds
Expected: < 2 seconds
```

---

## Code Flow Analysis

### Current Shutdown Flow (from server.rs)

```
1. User presses Ctrl+C
   ↓
2. shutdown_signal() → tokio::signal::ctrl_c() detects signal
   ↓
3. Logs "Received Ctrl+C signal" (line 1839)
   ↓
4. Logs "Initiating graceful shutdown..." (line 1846)
   ↓
5. shutdown_signal() returns from async function
   ↓
6. .with_graceful_shutdown(shutdown_signal()) triggers
   [AXUM HANDLES GRACEFUL SHUTDOWN HERE - ~2 SECOND DELAY]
   ↓
7. Server.await completes (line 1783)
   ↓
8. Logs "Signaling background tasks to shutdown..." (line 1786)
   ↓
9. Sets shutdown_flag = true (line 1787)
   ↓
10. Waits for metrics task with 500ms timeout (line 1793-1803)
    ↓
11. Logs "Cleaning up PID file..." (line 1806)
    ↓
12. Logs "Server shut down gracefully" (line 1811)
```

---

## Root Cause Hypothesis

The 2-second delay occurs between steps 4 and 8, which suggests:

### **Most Likely**: Axum Graceful Shutdown Implementation

The `.with_graceful_shutdown()` middleware in Axum may have:
- A hardcoded 2-second grace period for in-flight requests
- Connection draining with timeout
- Socket cleanup delay

**Evidence**:
- The delay is consistent (~2 seconds)
- Logs show all shutdown messages are logged in rapid succession AFTER the delay
- The signal handler completes quickly (log lines 1839-1846 are immediate)

### **Secondary Possibility**: TCP Socket Cleanup

The SO_REUSEADDR socket option is being set, but there might be:
- Pending connections being drained
- Socket state transitions taking time
- Kernel TCP cleanup operations

**Code Location**: server.rs lines 1737-1760 (socket setup with libc::setsockopt)

### **Low Probability**: Background Tasks Blocking

The metrics task is given 500ms to shutdown, but it's checked AFTER the 2-second delay, so this is not the cause.

---

## Axum Version and Behavior

The project uses Axum 0.7.9. From Axum documentation:

> `.with_graceful_shutdown()` allows graceful shutdown by providing a future that completes when shutdown should begin. The server will wait for all in-flight connections to complete (or timeout).

**Potential Issue**: Axum might have a default grace period timeout, or it's waiting for connection draining.

---

## Investigation Steps for Rust Specialist

### 1. Check Axum Graceful Shutdown

**Question**: Does Axum 0.7.9 have a default grace period or timeout?

```rust
.with_graceful_shutdown(shutdown_signal())
```

Look for:
- Default timeout configurations
- Whether in-flight requests are blocking
- If there's a way to skip grace period for inactive servers

### 2. Check for Hidden Sleeps

```bash
grep -n "sleep\|Duration::from_secs(2)\|2000" src/server.rs
```

Look for any 2-second sleeps that might be in the shutdown path.

### 3. Test with Minimal Server

Create a minimal Axum server to verify if the 2-second delay is inherent to Axum or specific to this implementation.

### 4. Check Connection Tracking

The new `ConnectionTracker` was added in this commit. Check if it's:
- Holding connections open
- Waiting for connections to drop
- Blocking the shutdown path

---

## Specific Code Sections to Review

### Section 1: Signal Handler (Lines 1817-1847)
```rust
async fn shutdown_signal() {
    // Should complete immediately - appears to be fine
}
```

### Section 2: Server Startup with Graceful Shutdown (Lines 1778-1783)
```rust
let result = axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
)
.with_graceful_shutdown(shutdown_signal())  // <-- POTENTIAL ISSUE HERE
.await;
```

**Action**: Check if there are configuration options for graceful shutdown timeouts

### Section 3: Socket Options (Lines 1737-1760)
```rust
#[cfg(unix)]
{
    use std::os::unix::io::AsRawFd;
    // Socket option setting code
}
```

**Action**: Verify socket options aren't causing delays

---

## Performance Impact

**Current Behavior**:
- Shutdown: ~4 seconds
- User experience: Slow to respond to kill signals
- Deployment: Container orchestration might forcefully kill pods

**Target Behavior**:
- Shutdown: < 2 seconds
- User experience: Responsive to Ctrl+C
- Deployment: Clean shutdowns within grace period

---

## Potential Fixes (for Rust Specialist to evaluate)

### Option 1: Configure Axum Timeout
Check if Axum's graceful shutdown has a timeout parameter and reduce it:
```rust
// Pseudocode - actual API may differ
.with_graceful_shutdown(
    shutdown_signal(),
    Some(Duration::from_secs(1))  // 1-second grace period instead of default
)
```

### Option 2: Immediate Shutdown
If graceful shutdown isn't needed:
```rust
// Skip the .with_graceful_shutdown() call entirely
// Still notify tasks with shutdown_flag
```

### Option 3: Custom Graceful Shutdown
Implement custom shutdown logic without Axum's built-in mechanism.

### Option 4: Async Signal Timing
Verify the signal handler is actually being called immediately and there's no async delay.

---

## Testing the Fix

Once the Rust Specialist implements a fix, QA will verify:

```bash
for run in 1 2 3 4 5; do
    START=$(date +%s%N | cut -b1-13)
    cargo run --release -- run --debug --port 3005 &
    PID=$!
    sleep 3
    kill -INT $PID
    wait $PID
    END=$(date +%s%N | cut -b1-13)
    DURATION=$((END - START))
    echo "Run $run: ${DURATION}ms"
done
```

**Success Criteria**: All runs < 2000ms

---

## Additional Notes

### Why This Matters
1. **Kubernetes Deployments**: Default grace period is often 30 seconds, but custom periods might be 5-10 seconds
2. **CI/CD Pipelines**: Tests and deployments expect fast startup/shutdown
3. **Development Experience**: Developers expect Ctrl+C to be responsive

### Non-Critical Observations
- Port is released correctly after shutdown
- No zombie processes
- Graceful shutdown message is being logged
- All other functionality is working

---

## Escalation

This issue should be escalated to:
- **Rust Specialist**: For debugging and fixing the root cause
- **Performance Reviewer**: To verify the fix under load

Once fixed, QA will re-run the comprehensive test suite.

---

**Status**: AWAITING RUST SPECIALIST INVESTIGATION AND FIX
