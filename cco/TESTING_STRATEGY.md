# CCO Testing Strategy and Best Practices

**Version**: 2025.11
**Last Updated**: November 17, 2025
**Status**: Critical Documentation

---

## Table of Contents

1. [Testing Gap Analysis](#testing-gap-analysis)
2. [CCO Testing Procedures](#cco-testing-procedures)
3. [Best Practices](#best-practices)
4. [Quick Reference](#quick-reference)
5. [Lessons Learned](#lessons-learned)
6. [Code Review Checklist](#code-review-checklist)

---

## Testing Gap Analysis

### What Happened

A critical testing gap was discovered during E2E testing of the CCO (Command Orchestrator) application. The E2E test suite successfully validated the **explicit server mode** (`cco run`) but **failed to test the default TUI/daemon mode** that actual users interact with.

### The Mistake

**Root Cause**: The E2E tests only exercised one execution path:
- ✅ Tested: `cco run` (explicit server mode with visible terminal output)
- ❌ Missed: Default TUI/daemon mode (actual user workflow)

The test coverage created a false sense of security because:

```
Application runs fine in explicit server mode
    ↓
Test passes in explicit server mode
    ↓
Assumption: Application works correctly everywhere
    ↓
❌ WRONG: Daemon/TUI mode has subtle initialization bugs
```

### The Impact

**Production Failure**: Real users started the application using the default behavior (`cco` without arguments), which enters TUI/daemon mode. This mode has:

- Different initialization sequence
- Different port binding behavior
- Different shutdown mechanisms
- Different readiness state detection

The bug that was missed:

```
Explicit mode: cco run
  ↓ Starts server immediately
  ↓ Binds port 8080
  ↓ Server listens before returning
  ✅ Tests pass

Default mode: cco (or cco start)
  ↓ Spawns daemon in background
  ↓ Port binding happens asynchronously
  ↓ Returns immediately (daemon not yet ready)
  ❌ Tests don't capture this timing issue
```

### The Lesson

**Critical Testing Principle**: Different code paths must be tested independently.

```
Multiple execution modes = Multiple test scenarios
- Not optional
- Not "nice to have"
- Essential for complete coverage
```

Application complexity grows with execution modes:

```
Simple CLI app
  └─ One execution path
  └─ One test suite sufficient

Server with daemon mode
  ├─ Explicit server mode (cco run)
  ├─ Implicit daemon mode (cco)
  ├─ TUI interaction mode
  └─ Each has different initialization and lifecycle
  └─ Each requires dedicated testing
```

### The Fix

**Comprehensive Testing Strategy**: Test all execution modes and all lifecycle stages:

```
Before Testing
  ├─ Clean state (no running instances)
  ├─ Clear ports
  └─ Fresh configuration

Execution Mode 1: Explicit Server Mode
  ├─ Start: cco run
  ├─ Verify: Port binding happens
  ├─ Verify: Server responds to requests
  └─ Shutdown: Clean termination

Execution Mode 2: Default TUI/Daemon Mode
  ├─ Start: cco (background)
  ├─ Wait: Daemon initialization
  ├─ Verify: Port is bound and listening
  ├─ Verify: Client can connect
  └─ Shutdown: Graceful daemon termination

Execution Mode 3: Custom Configuration
  ├─ Start: cco --config custom.toml
  ├─ Verify: Config is loaded correctly
  ├─ Verify: Correct port and settings applied
  └─ Shutdown: Clean termination

After Testing
  ├─ Port released
  ├─ No orphaned processes
  └─ Clean state for next test
```

### Updated Definition of "Fully Tested"

Before: "Tests pass for main code paths"

After: "Tests pass for ALL code paths including:
- [ ] All execution modes
- [ ] All initialization sequences
- [ ] All shutdown scenarios
- [ ] Error conditions and edge cases
- [ ] Lifecycle from startup to shutdown
- [ ] Resource cleanup (ports, processes, files)"

---

## CCO Testing Procedures

### Prerequisites

Before running tests, ensure your environment is clean:

```bash
# Kill any running CCO instances
pkill -f "cco" || true

# Clear test ports (if using fixed ports in tests)
lsof -i :8080 | grep -v COMMAND | awk '{print $2}' | xargs kill -9 || true

# Verify clean state
ps aux | grep -i cco
# Should show no running CCO processes (only grep itself)
```

### Running Unit Tests

Unit tests verify individual functions and modules in isolation.

```bash
# Run all unit tests with output
cargo test --lib --verbose

# Run specific unit test module
cargo test --lib analytics --verbose

# Run single test function
cargo test --lib analytics::test_cost_calculation --verbose

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo test --lib --verbose

# Run with logging output
RUST_LOG=debug cargo test --lib --verbose -- --nocapture
```

**Expected Output**:
```
test analytics::test_cost_calculation ... ok
test analytics::test_edge_cases ... ok

test result: ok. X passed; 0 failed; 0 ignored; 0 measured; Y filtered out
```

### Running Integration Tests

Integration tests verify multiple components working together.

```bash
# Run all integration tests
cargo test --test '*' --verbose

# Run specific integration test file
cargo test --test daemon_lifecycle_tests --verbose

# Run integration tests with port auto-selection
cargo test --test daemon_lifecycle_tests -- --test-threads=1 --nocapture

# Run with output to see detailed logs
RUST_LOG=debug cargo test --test daemon_lifecycle_tests --verbose -- --nocapture
```

**Key Integration Test Files**:
- `daemon_lifecycle_tests.rs` - Daemon startup/shutdown
- `terminal_comprehensive.rs` - TUI/daemon mode
- `tui_integration_tests.rs` - TUI functionality
- `phase1a_integration_tests.rs` - Complete workflows

### Running E2E Tests (Playwright)

E2E tests verify complete user workflows from UI interaction.

#### Critical: Test Both Execution Modes

```bash
# Test 1: Explicit Server Mode (cco run)
# ==========================================

# Start server in explicit mode
cd /Users/brent/git/cc-orchestra/cco
cco run &
SERVER_PID=$!

# Wait for server to be fully ready
sleep 2

# Verify port is listening
netstat -tuln | grep 8080

# Run Playwright tests
npx playwright test tests/terminal_keyboard_e2e.spec.js --headed

# Shutdown
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true

# Verify port is released
sleep 1
netstat -tuln | grep 8080 || echo "Port released successfully"


# Test 2: Default TUI/Daemon Mode (cco)
# =======================================

# Kill any running instances
pkill -f "cco" || true
sleep 1

# Start application in default mode (daemon)
cco &
APP_PID=$!

# CRITICAL: Wait for daemon to initialize
# This is where the bug was - tests must wait for daemon readiness
sleep 3

# Verify daemon is running
ps -p $APP_PID > /dev/null && echo "Daemon running" || echo "Daemon failed to start"

# Verify port is listening (this was failing before)
lsof -i :8080 | grep -q LISTEN && echo "Port is listening" || echo "Port not listening - BUG!"

# Run same tests against daemon mode
npx playwright test tests/terminal_keyboard_e2e.spec.js --headed

# Graceful shutdown
kill $APP_PID
wait $APP_PID 2>/dev/null || true

# Verify port is released
sleep 1
lsof -i :8080 || echo "Port released successfully"
```

#### Automated E2E Test Script

Create `/Users/brent/git/cc-orchestra/cco/run-all-e2e-tests.sh`:

```bash
#!/bin/bash
set -e

MODES=("explicit" "daemon")
RESULTS=()

for MODE in "${MODES[@]}"; do
    echo "=========================================="
    echo "Testing: $MODE mode"
    echo "=========================================="

    # Clean state
    pkill -f "cco" || true
    sleep 2

    if [ "$MODE" = "explicit" ]; then
        # Explicit server mode
        cco run &
        PID=$!
        echo "Started in explicit mode (PID: $PID)"
        sleep 2
    else
        # Daemon mode
        cco &
        PID=$!
        echo "Started in daemon mode (PID: $PID)"
        sleep 3
    fi

    # Verify port is listening
    if lsof -i :8080 | grep -q LISTEN; then
        echo "✅ Port 8080 is listening"
        RESULTS+=("$MODE: PASS")

        # Run tests
        npx playwright test tests/terminal_keyboard_e2e.spec.js
        TEST_RESULT=$?
        if [ $TEST_RESULT -eq 0 ]; then
            echo "✅ Tests passed for $MODE mode"
        else
            echo "❌ Tests failed for $MODE mode"
            RESULTS+=("$MODE: TEST FAIL")
        fi
    else
        echo "❌ Port not listening in $MODE mode - CRITICAL BUG"
        RESULTS+=("$MODE: FAIL")
    fi

    # Shutdown
    kill $PID 2>/dev/null || true
    wait $PID 2>/dev/null || true
    sleep 1
done

# Report
echo ""
echo "=========================================="
echo "Test Results"
echo "=========================================="
for result in "${RESULTS[@]}"; do
    echo "$result"
done
```

### Running Smoke Tests Locally

Quick validation tests to catch obvious breakage.

```bash
# Smoke test script
#!/bin/bash
set -e

echo "Smoke Test: Basic Application Startup"
echo "====================================="

# Kill any existing processes
pkill -f "cco" || true
sleep 1

# Test 1: Explicit mode starts
echo "1. Starting in explicit mode..."
cco run &
PID=$!
sleep 2

if ps -p $PID > /dev/null; then
    echo "   ✅ Process running"
else
    echo "   ❌ Process failed to start"
    exit 1
fi

# Test 2: Port is bound
if lsof -i :8080 | grep -q LISTEN; then
    echo "   ✅ Port 8080 is listening"
else
    echo "   ❌ Port not listening"
    kill $PID 2>/dev/null || true
    exit 1
fi

# Test 3: Health check
if curl -s http://localhost:8080/health | grep -q "status"; then
    echo "   ✅ Health endpoint responding"
else
    echo "   ❌ Health endpoint not responding"
    kill $PID 2>/dev/null || true
    exit 1
fi

kill $PID
wait $PID 2>/dev/null || true

# Test 4: Daemon mode starts
echo ""
echo "2. Starting in daemon mode..."
cco &
PID=$!
sleep 3

if ps -p $PID > /dev/null; then
    echo "   ✅ Daemon running"
else
    echo "   ❌ Daemon failed to start"
    exit 1
fi

if lsof -i :8080 | grep -q LISTEN; then
    echo "   ✅ Port bound by daemon"
else
    echo "   ❌ Port not bound"
    kill $PID 2>/dev/null || true
    exit 1
fi

kill $PID
wait $PID 2>/dev/null || true
sleep 1

# Test 5: Clean shutdown
if ! lsof -i :8080 2>/dev/null | grep -q LISTEN; then
    echo "   ✅ Port released after shutdown"
else
    echo "   ❌ Port still in use after shutdown"
    exit 1
fi

echo ""
echo "✅ All smoke tests passed"
```

### Running Full Test Suite Before Commit

Complete test validation before pushing code.

```bash
#!/bin/bash
set -e

echo "Full Test Suite Validation"
echo "==========================="

# Phase 1: Unit tests
echo ""
echo "Phase 1: Unit Tests"
echo "-------------------"
cargo test --lib --verbose

# Phase 2: Integration tests
echo ""
echo "Phase 2: Integration Tests"
echo "---------------------------"
cargo test --test '*' --verbose

# Phase 3: Specific daemon/TUI tests
echo ""
echo "Phase 3: Daemon/TUI Integration"
echo "--------------------------------"
cargo test --test daemon_lifecycle_tests --verbose -- --nocapture
cargo test --test tui_integration_tests --verbose -- --nocapture

# Phase 4: Build check
echo ""
echo "Phase 4: Build Verification"
echo "-----------------------------"
cargo build --release

# Phase 5: E2E tests (Playwright)
echo ""
echo "Phase 5: End-to-End Tests"
echo "-------------------------"

# Clean state
pkill -f "cco" || true
sleep 2

# Test explicit mode
echo "Testing explicit server mode..."
cco run &
SERVER_PID=$!
sleep 2
npx playwright test tests/ --headed
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

# Test daemon mode
echo "Testing daemon mode..."
pkill -f "cco" || true
sleep 2
cco &
DAEMON_PID=$!
sleep 3
npx playwright test tests/ --headed
kill $DAEMON_PID 2>/dev/null || true
wait $DAEMON_PID 2>/dev/null || true

echo ""
echo "✅ Full test suite completed successfully"
```

### Interpreting Test Results

#### Successful Test Run

```
test result: ok. 47 passed; 0 failed; 2 ignored; 0 measured

What this means:
- 47 tests passed ✅
- 0 tests failed ✅
- 2 tests ignored (marked #[ignore]) - these are normal
- 0 tests failed for timing/measurement reasons
```

#### Failed Test - Port Binding

```
test daemon_lifecycle::test_port_binding ... FAILED

---- daemon_lifecycle::test_port_binding stdout ----
thread 'daemon_lifecycle::test_port_binding' panicked at 'Port not listening: 8080'

What this means:
- Daemon started but didn't bind to expected port
- Likely causes:
  a) Port already in use (kill previous process)
  b) Daemon initialization incomplete (increase wait time)
  c) Daemon failed silently (check logs with RUST_LOG=debug)
```

#### Failed Test - Daemon Readiness

```
test daemon_lifecycle::test_client_connection ... FAILED

---- daemon_lifecycle::test_client_connection stdout ----
thread 'daemon_lifecycle::test_client_connection' panicked at 'Connection refused'

What this means:
- Daemon started but not ready for connections
- The timing issue described in this document
- Fix: Increase wait time before test attempts connection
- Or: Implement daemon readiness detection
```

#### Failed Test - Execution Mode Difference

```
test e2e::test_terminal_input_explicit ... ok
test e2e::test_terminal_input_daemon ... FAILED

What this means:
- Same test passes in explicit mode but fails in daemon mode
- Different initialization sequence or behavior in daemon mode
- CRITICAL: This is the gap we're trying to prevent
```

### Debugging Test Failures

#### 1. Run with Full Logging

```bash
# Show all debug output
RUST_LOG=debug,cco=trace cargo test --lib analytics --verbose -- --nocapture

# Trace specific module
RUST_LOG=cco::daemon=trace cargo test daemon_lifecycle --verbose -- --nocapture
```

#### 2. Run Single Test with Backtrace

```bash
# Full backtrace on panic
RUST_BACKTRACE=full cargo test --lib test_name --verbose -- --nocapture

# Short backtrace (usually sufficient)
RUST_BACKTRACE=1 cargo test --lib test_name --verbose -- --nocapture
```

#### 3. Inspect System State During Test

```bash
# In test or in separate terminal:
# Check running processes
watch -n 0.1 'ps aux | grep cco'

# Check ports
watch -n 0.1 'lsof -i :8080'

# Check network connections
watch -n 0.1 'netstat -an | grep 8080'
```

#### 4. Add Debug Output to Test

```rust
// In test file
#[tokio::test]
async fn test_daemon_starts() {
    println!("Starting test...");

    let daemon = start_daemon().await;
    println!("Daemon started: {:?}", daemon);

    sleep(Duration::from_secs(2)).await;
    println!("Checking port...");

    assert!(is_port_listening(8080), "Port should be listening");
    println!("✅ Port is listening");
}
```

#### 5. Replay Exact Test Scenario Manually

```bash
# If test fails, manually reproduce the exact steps

# Example: test_daemon_port_binding fails
# 1. Kill any running instances
pkill -f "cco" || true
sleep 2

# 2. Start exactly as test does
cco &
DAEMON_PID=$!
sleep 3

# 3. Check exact conditions
lsof -i :8080
ps -p $DAEMON_PID

# 4. Manually try what test does
curl http://localhost:8080/health

# 5. Check logs
journalctl -u cco --no-pager | tail -20

# Clean up
kill $DAEMON_PID 2>/dev/null || true
```

---

## Best Practices

### 1. Always Test Multiple Execution Modes

**The Core Lesson from This Gap**

```rust
// DON'T: Only test one mode
#[test]
fn test_server() {
    start_server_explicit_mode();
    assert!(endpoint_responds());
}

// DO: Test all modes
#[tokio::test]
async fn test_explicit_mode() {
    let server = start_server_explicit_mode().await;
    assert!(is_port_listening().await);
    assert!(endpoint_responds().await);
    server.shutdown().await;
}

#[tokio::test]
async fn test_daemon_mode() {
    let daemon = start_daemon().await;
    sleep(Duration::from_secs(2)).await;  // Wait for init
    assert!(is_port_listening().await);   // CRITICAL check
    assert!(endpoint_responds().await);
    daemon.shutdown().await;
}
```

### 2. Test Both Happy Path and Error Cases

```rust
// Happy path: Everything works
#[tokio::test]
async fn test_start_success() {
    let daemon = start_daemon_with_config(valid_config()).await;
    assert!(daemon.is_running());
}

// Error case: Invalid config
#[tokio::test]
async fn test_start_with_invalid_config() {
    let result = start_daemon_with_config(invalid_config()).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ConfigError::Invalid);
}

// Error case: Port already in use
#[tokio::test]
async fn test_port_already_in_use() {
    let _first = start_daemon_on_port(8080).await;
    let result = start_daemon_on_port(8080).await;
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("in use"));
}
```

### 3. Verify Startup Sequences (Not Just Endpoints)

```rust
// DON'T: Just check endpoint
#[tokio::test]
async fn test_startup_wrong() {
    start_daemon();
    assert!(endpoint_responds());
}

// DO: Verify entire startup sequence
#[tokio::test]
async fn test_startup_correct() {
    // Step 1: Process starts
    let daemon = start_daemon().await;
    assert!(daemon.process_running());

    // Step 2: Port binds
    assert!(port_is_bound(8080));

    // Step 3: Service becomes ready
    let mut retries = 0;
    loop {
        if endpoint_responds().await {
            break;
        }
        retries += 1;
        assert!(retries < 10, "Service took too long to become ready");
        sleep(Duration::from_millis(100)).await;
    }

    // Step 4: All health checks pass
    let health = get_health_status().await;
    assert_eq!(health.status, "healthy");
    assert!(health.all_checks_pass());
}
```

### 4. Check Port Binding and Release

```rust
// Before test: Port must be free
#[tokio::test]
async fn test_port_management() {
    // CRITICAL: Assert port is free before starting
    assert!(!port_is_in_use(8080), "Port should be free before test");

    let daemon = start_daemon().await;
    sleep(Duration::from_secs(1)).await;

    // CRITICAL: Assert port IS bound during operation
    assert!(port_is_in_use(8080), "Port should be bound during operation");

    daemon.shutdown().await;
    sleep(Duration::from_millis(500)).await;

    // CRITICAL: Assert port is released after shutdown
    assert!(!port_is_in_use(8080), "Port should be released after shutdown");
}

fn port_is_in_use(port: u16) -> bool {
    // Use netstat or lsof to check
    let output = std::process::Command::new("lsof")
        .args(&["-i", &format!(":{}", port)])
        .output()
        .expect("lsof failed");
    output.status.success()
}
```

### 5. Validate Daemon Readiness Before Client Connections

```rust
// DON'T: Connect immediately
#[tokio::test]
async fn test_wrong_timing() {
    start_daemon();
    // Daemon might not be ready yet!
    let result = connect_to_daemon().await;  // May fail!
}

// DO: Wait for daemon readiness
#[tokio::test]
async fn test_correct_timing() {
    start_daemon().await;

    // CRITICAL: Wait for daemon to be fully initialized
    wait_for_daemon_ready(Duration::from_secs(5)).await;

    // NOW it's safe to connect
    let result = connect_to_daemon().await;
    assert!(result.is_ok());
}

async fn wait_for_daemon_ready(timeout: Duration) {
    let start = Instant::now();
    loop {
        // Check all readiness indicators
        if port_is_listening(8080) &&
           health_endpoint_responds().await &&
           database_is_connected().await {
            break;  // Daemon is ready
        }

        if start.elapsed() > timeout {
            panic!("Daemon not ready after {:?}", timeout);
        }

        sleep(Duration::from_millis(100)).await;
    }
}
```

### 6. Test Graceful Shutdown

```rust
// Graceful shutdown
#[tokio::test]
async fn test_graceful_shutdown() {
    let daemon = start_daemon().await;
    sleep(Duration::from_secs(1)).await;

    // Send shutdown signal
    daemon.signal_shutdown().await;

    // Wait for graceful shutdown (with timeout)
    let shutdown_result = tokio::time::timeout(
        Duration::from_secs(5),
        daemon.wait_for_shutdown()
    ).await;

    assert!(shutdown_result.is_ok(), "Shutdown should complete");
    assert!(!daemon.process_running(), "Process should be stopped");
}

// Forced shutdown if graceful times out
#[tokio::test]
async fn test_forced_shutdown() {
    let mut daemon = start_daemon().await;
    daemon.signal_shutdown().await;

    // Wait briefly for graceful
    sleep(Duration::from_secs(1)).await;

    // If still running, force kill
    if daemon.process_running() {
        daemon.force_kill().await;
    }

    sleep(Duration::from_millis(500)).await;
    assert!(!daemon.process_running());
}
```

### 7. Avoid Hardcoded Ports and Timeouts

```rust
// DON'T: Hardcode port and timeout
#[tokio::test]
async fn test_wrong_hardcoding() {
    start_daemon_on_port(8080);  // What if 8080 is in use?
    sleep(Duration::from_secs(1));  // What if system is slow?
}

// DO: Use dynamic allocation
#[tokio::test]
async fn test_right_dynamic() {
    // Get free port
    let port = allocate_free_port().unwrap();

    // Start daemon on free port
    let daemon = start_daemon_on_port(port).await;

    // Wait for readiness dynamically
    wait_for_daemon_ready_on_port(port, Duration::from_secs(10)).await;

    // Test using the allocated port
    assert!(is_port_listening(port).await);
}

fn allocate_free_port() -> Result<u16> {
    // Bind to port 0 (OS picks free port)
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}
```

### 8. Use Cleanup Fixtures to Prevent Test Pollution

```rust
// Setup and teardown for each test
async fn run_test_with_cleanup<F>(test_fn: F)
where
    F: FnOnce() -> BoxFuture<'static, ()>,
{
    // Setup: Clean state
    cleanup_all_instances().await;
    cleanup_ports().await;
    cleanup_config_files().await;

    // Run test
    test_fn().await;

    // Teardown: Clean state
    cleanup_all_instances().await;
    cleanup_ports().await;
}

// Example test using fixture
#[tokio::test]
async fn test_with_cleanup() {
    run_test_with_cleanup(|| Box::pin(async {
        let daemon = start_daemon().await;
        // ... test code ...
    })).await;
}

// Or use a setup struct
struct TestContext {
    daemon: Option<DaemonHandle>,
}

impl TestContext {
    async fn new() -> Self {
        cleanup_all_instances().await;
        Self { daemon: None }
    }

    async fn start_daemon(&mut self) {
        self.daemon = Some(start_daemon().await);
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup happens automatically when context is dropped
        if let Some(daemon) = self.daemon.take() {
            tokio::spawn(async move {
                daemon.force_kill().await;
            });
        }
    }
}

#[tokio::test]
async fn test_with_context() {
    let mut ctx = TestContext::new().await;
    ctx.start_daemon().await;
    // Test automatically cleans up when ctx is dropped
}
```

---

## Quick Reference

### Common Test Commands

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test '*'

# Run specific test file
cargo test --test daemon_lifecycle_tests

# Run specific test function
cargo test test_daemon_starts

# Run with output
cargo test -- --nocapture --test-threads=1

# Run with logging
RUST_LOG=debug cargo test --verbose -- --nocapture

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo test --verbose

# Check compilation without running
cargo test --no-run

# Run Playwright tests
npx playwright test tests/terminal_keyboard_e2e.spec.js

# Run Playwright in headed mode (see browser)
npx playwright test tests/terminal_keyboard_e2e.spec.js --headed

# Run single Playwright test
npx playwright test tests/terminal_keyboard_e2e.spec.js -g "specific test name"
```

### Test Modes Checklist

When testing any feature, verify ALL modes:

```
FEATURE TESTING CHECKLIST
════════════════════════════════════════════════════════════

Feature: ___________________________

[ ] Explicit Server Mode (cco run)
    [ ] Starts without errors
    [ ] Port binds correctly
    [ ] Health check passes
    [ ] Feature works as expected
    [ ] Shutdown is clean

[ ] Default Daemon Mode (cco)
    [ ] Starts in background
    [ ] Daemon readiness detected
    [ ] Port binds correctly
    [ ] Health check passes
    [ ] Feature works as expected
    [ ] Shutdown is graceful

[ ] Custom Configuration
    [ ] Config loads correctly
    [ ] Feature respects config
    [ ] Settings applied properly

[ ] Error Conditions
    [ ] Invalid input handled
    [ ] Port already in use caught
    [ ] Config errors reported
    [ ] Graceful failure recovery

[ ] Resource Cleanup
    [ ] Port released after shutdown
    [ ] No orphaned processes
    [ ] Temporary files cleaned
    [ ] State reset for next test
```

### Critical Validations

Always verify these before marking tests as complete:

```bash
# Before test run
echo "Pre-test checks:"
pkill -f "cco" || echo "No processes to kill"
sleep 1
netstat -tuln | grep 8080 || echo "Port 8080 is free"

# During test
echo "Runtime checks:"
ps aux | grep cco | grep -v grep || echo "Daemon process check"
lsof -i :8080 | grep LISTEN || echo "Port binding check"

# After test
echo "Post-test checks:"
pkill -f "cco" || echo "Cleanup: kill running instances"
sleep 1
lsof -i :8080 || echo "Port 8080 is free - success"
ps aux | grep cco | grep -v grep || echo "No orphaned processes"
```

---

## Lessons Learned

### Discovery

**What We Discovered**: The E2E test suite was incomplete and false-positive.

```
Initial Assumption: "Tests pass → Application is working"
Reality: "Tests pass in explicit mode → Application works in explicit mode"
                                    (But might not work in daemon mode!)
```

### Root Cause Analysis

```
Why the gap existed:
├─ Explicit mode (cco run) is easier to test
│  └─ Direct control over process
│  └─ Synchronous startup
│  └─ Output visible in test logs
│
└─ Daemon mode (cco) is harder to test
   ├─ Process runs in background
   ├─ Asynchronous port binding
   ├─ Readiness detection is non-obvious
   └─ First instinct: "Skip it - too complex"
```

### Why Tests Were Insufficient

1. **Single Path Testing**: Only tested the explicit server mode
2. **False Confidence**: Passing tests didn't catch real-world failures
3. **Incomplete Definition**: "Tested" didn't include all code paths
4. **No Lifecycle Testing**: Didn't verify startup → ready → shutdown cycle
5. **Missing Async Verification**: Didn't wait for daemon readiness

### Prevention Strategy

**Going Forward**: Multiple defense layers

```
Layer 1: Development (Developer responsibility)
├─ Write tests for ALL code paths
├─ Include multiple execution modes
└─ Verify startup and shutdown sequences

Layer 2: Code Review (Reviewer responsibility)
├─ Check test coverage for all modes
├─ Look for hardcoded modes/assumptions
└─ Verify lifecycle testing exists

Layer 3: CI/CD (Pipeline responsibility)
├─ Run full test suite (all modes)
├─ Check for test coverage regression
└─ Fail if any mode untested

Layer 4: Acceptance (Definition of Done)
├─ "Tested" = All execution modes verified
├─ "Tested" = Happy path AND error cases
├─ "Tested" = Lifecycle from startup to shutdown
└─ "Tested" = Resource cleanup verified
```

### Updated Definition of "Fully Tested"

**Old Definition** (insufficient):
> "Tests pass for the happy path"

**New Definition** (complete):
> "Application has comprehensive test coverage including:
> - All execution modes (explicit, daemon, custom config)
> - All initialization paths
> - All shutdown scenarios
> - Error handling and edge cases
> - Lifecycle management (startup → ready → operation → shutdown)
> - Resource cleanup (ports, processes, temporary files)
> - Async operation verification and timing
> - Health checks and readiness detection"

### Acceptance Criteria for Testing

Before marking any feature as "tested":

- [ ] Unit tests written for individual functions
- [ ] Integration tests written for multi-component workflows
- [ ] All execution modes tested with dedicated tests
- [ ] Error cases and edge cases covered
- [ ] Async operations properly awaited and verified
- [ ] Port binding verified (before, during, after)
- [ ] Process lifecycle tested (start, run, shutdown)
- [ ] Graceful shutdown verified
- [ ] Resource cleanup verified
- [ ] No hardcoded timeouts or ports
- [ ] No test interdependencies or pollution
- [ ] Test results reproducible across environments
- [ ] Documentation updated with test procedures

### Team Takeaways

1. **Code paths need dedicated tests**: Don't assume "if it works here, it works everywhere"
2. **Async operations need explicit verification**: Waiting isn't optional, it's required
3. **Test coverage is multi-dimensional**: Different modes = different tests
4. **Definition of done must be explicit**: Document what "tested" means
5. **Testing is part of the feature**: Not an afterthought or optional

---

## Code Review Checklist

Use this checklist when reviewing PRs to prevent similar gaps:

### Testing Requirements

```markdown
## Testing Review Checklist

- [ ] **Unit Tests Written**: All new functions have unit test coverage
- [ ] **Integration Tests**: Multi-component interactions are tested
- [ ] **Execution Mode Coverage**:
  - [ ] Explicit server mode tested (if applicable)
  - [ ] Daemon/background mode tested (if applicable)
  - [ ] Custom configuration mode tested (if applicable)

- [ ] **Lifecycle Testing**:
  - [ ] Startup sequence verified
  - [ ] Initialization order validated
  - [ ] Shutdown gracefully tested
  - [ ] Resource cleanup verified

- [ ] **Error Case Coverage**:
  - [ ] Invalid inputs handled
  - [ ] Configuration errors caught
  - [ ] Resource unavailable scenarios tested
  - [ ] Timeout edge cases covered

- [ ] **Async Operation Verification**:
  - [ ] No race conditions
  - [ ] Proper timeouts used
  - [ ] Readiness explicitly verified
  - [ ] No "wait and hope" patterns

- [ ] **Resource Management**:
  - [ ] Port binding verified
  - [ ] Port release after shutdown verified
  - [ ] No orphaned processes
  - [ ] Temporary files cleaned up

- [ ] **Test Quality**:
  - [ ] No hardcoded ports/timeouts
  - [ ] No test interdependencies
  - [ ] No pollution between tests
  - [ ] Cleanup fixtures implemented
  - [ ] Tests are reproducible

- [ ] **Documentation**:
  - [ ] Test procedures documented
  - [ ] Setup/teardown documented
  - [ ] Manual test steps documented (if needed)
  - [ ] Test execution time reasonable
```

### Code Review Comments

**When you see a test gap**, use these comments:

```markdown
### Issue: Only explicit mode tested
This PR only tests `cco run` mode but the application also supports daemon mode.
Consider adding tests for:
- Default daemon mode (cco)
- Daemon readiness detection
- Async port binding verification

Reference: See TESTING_STRATEGY.md section "Best Practices #1"

Suggested fix:
```rust
#[tokio::test]
async fn test_daemon_mode() {
    let daemon = start_daemon().await;
    sleep(Duration::from_secs(2)).await;  // Wait for init
    assert!(is_port_listening().await);   // Verify readiness
    assert!(endpoint_responds().await);   // Verify functionality
    daemon.shutdown().await;
}
```
```

```markdown
### Issue: Missing startup sequence verification
This test only checks the endpoint but doesn't verify the full startup sequence.
Consider adding checks for:
- Process startup
- Port binding
- Service readiness
- All health checks pass

Reference: See TESTING_STRATEGY.md section "Best Practices #3"
```

```markdown
### Issue: No port cleanup verification
This test doesn't verify that the port is released after shutdown.
Add verification:
- Port should be in use during operation
- Port should be free after shutdown
- Consider timeouts for port release

Reference: See TESTING_STRATEGY.md section "Best Practices #4"
```

```markdown
### Issue: Hardcoded timeout/port
This test uses hardcoded values which can cause flakiness.
Consider:
- Using dynamic port allocation (port 0)
- Using adaptive timeouts with retry loops
- Checking readiness dynamically instead of sleeping

Reference: See TESTING_STRATEGY.md section "Best Practices #7"
```

### Approval Criteria

**Approve the PR only if:**

- ✅ All execution modes are tested
- ✅ Lifecycle (startup → operation → shutdown) is verified
- ✅ Error cases are covered
- ✅ Resource cleanup is verified
- ✅ Test quality is high (no flakiness, no pollution)
- ✅ New tests follow established patterns
- ✅ Documentation is updated
- ✅ No test coverage regression

**Request changes if:**

- ❌ Tests only cover one execution mode
- ❌ Async operations not properly awaited
- ❌ Port binding not verified
- ❌ Shutdown not tested
- ❌ Hardcoded ports/timeouts without reason
- ❌ Missing error case coverage
- ❌ Test pollution or interdependencies
- ❌ Documentation not updated

---

## Appendix: Reference Documentation

### Files Referenced in This Document

- **Test Files**: `/Users/brent/git/cc-orchestra/cco/tests/`
- **Source Code**: `/Users/brent/git/cc-orchestra/cco/src/`
- **Build Configuration**: `/Users/brent/git/cc-orchestra/cco/Cargo.toml`
- **E2E Tests**: `/Users/brent/git/cc-orchestra/cco/tests/*.spec.js`

### Environment Variables for Testing

```bash
# Logging
RUST_LOG=debug              # Enable debug logging
RUST_LOG=cco=trace         # Trace specific module
RUST_LOG=debug,cco=trace   # Combined

# Backtrace on panic
RUST_BACKTRACE=1           # Short backtrace
RUST_BACKTRACE=full        # Full backtrace

# Test execution
RUST_TEST_THREADS=1        # Single-threaded (easier to debug)
```

### Useful Tools

```bash
# Process management
ps aux | grep cco           # Find running processes
pkill -f "cco"              # Kill processes by pattern

# Port management
lsof -i :8080               # List processes on port
netstat -tuln | grep 8080   # Check port status

# Logging
journalctl -u cco           # System journal logs
tail -f /var/log/cco.log    # Follow log file

# Performance
time cargo test             # Measure test execution time
watch -n 1 'test command'   # Monitor command output
```

### Related Documentation

- **Development Guide**: See project README
- **Architecture Documentation**: See ARCHITECTURE.md
- **Daemon Implementation**: See src/daemon/mod.rs
- **TUI Implementation**: See src/tui/mod.rs

---

**Last Updated**: November 17, 2025
**Version**: 1.0
**Status**: Active - Baseline for all future testing
