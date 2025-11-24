# Comprehensive CCO Testing Framework

**Document Version**: 2.0
**Created**: 2025-11-17
**Purpose**: Design actionable, implementation-ready test strategy addressing critical execution mode gaps
**Status**: Ready for Implementation

---

## Executive Summary

### Current Testing Gap

The existing smoke test validates **only explicit server mode** (`cco run --debug --port XXXX`), which creates a **critical blind spot**:

- **Mode 1 (Explicit Server)**: `cco run --port 3000` ✓ TESTED
- **Mode 2 (Default TUI/Daemon)**: `cco` (no args) ✗ **NOT TESTED** - CURRENT FAILURE

This mode-specific gap allowed the daemon startup failure to go undetected.

### Solution

Implement a **multi-layer testing framework** with:
1. **Coverage Matrix** - All execution modes and configuration combinations
2. **Progressive Test Phases** - Unit → Integration → E2E → Acceptance
3. **Critical Test Checklist** - Must-pass tests that catch mode-specific failures
4. **Improved Smoke Tests** - Both modes in one comprehensive script
5. **Prevention Strategy** - Catch future mode-specific failures early

---

## 1. Test Coverage Matrix

### Execution Mode Coverage

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│ EXECUTION MODE TESTING MATRIX                                                       │
├─────────────────────────────────────────────────────────────────────────────────────┤

MODE 1: Explicit Server Mode
├─ Command: cco run --debug --port 3000
├─ Current Status: ✓ TESTED (existing smoke test)
├─ Tests:
│  ├─ Server starts successfully
│  ├─ Listens on specified port (3000, 3001, 3100, etc)
│  ├─ /health endpoint responds (200, valid JSON)
│  ├─ /api/agents endpoint responds
│  ├─ /api/v1/chat endpoint accepts requests
│  ├─ WebSocket /terminal endpoint available
│  ├─ Static files served (index.html, assets)
│  ├─ Shutdown graceful (< 2 seconds)
│  ├─ Port released immediately after shutdown
│  ├─ No zombie processes remain
│  └─ Health check under load (concurrent requests)

MODE 2: Default TUI/Daemon Mode (CRITICAL GAP)
├─ Command: cco (no arguments)
├─ Current Status: ✗ NOT TESTED - **CAUSES FAILURES**
├─ Initialization Sequence (must validate order):
│  ├─ 1. TUI app initialization starts
│  ├─ 2. Daemon starts on default port (3000)
│  ├─ 3. Daemon port bound BEFORE TUI tries to connect
│  ├─ 4. TUI connects to daemon on localhost:3000
│  ├─ 5. Dashboard renders with data
│  └─ 6. Graceful shutdown of both
├─ Tests:
│  ├─ TUI process launches
│  ├─ Daemon process launches
│  ├─ Daemon listens on port 3000
│  ├─ TUI can connect to daemon
│  ├─ /api/agents endpoint accessible from TUI
│  ├─ Dashboard data loads correctly
│  ├─ Terminal component initializes
│  ├─ Keyboard input handled
│  ├─ Ctrl+C exits both TUI and daemon cleanly
│  ├─ Port 3000 released immediately
│  ├─ No hanging processes on exit
│  └─ No connection timeouts or refused connections

MODE 3: Configuration Variations
├─ Custom port: cco run --port 3100
├─ Debug mode: cco run --debug
├─ No debug: cco run (default)
├─ Explicit host: cco run --host 0.0.0.0
├─ Custom database: cco run --database-url sqlite:///custom.db
├─ All combinations tested
└─ Tests:
   ├─ Custom ports work (3000, 3001, 3100, 9000, etc)
   ├─ Debug logging enabled/disabled correctly
   ├─ Host binding works (127.0.0.1, 0.0.0.0, ::1)
   ├─ Database file created in correct location
   ├─ Environment variables override defaults
   └─ Config file parsing works correctly

MODE 4: Error & Edge Cases
├─ Port already in use
│  ├─ Kill existing process on port
│  ├─ Fall back to next available port
│  └─ Error message is clear
├─ Network errors
│  ├─ Daemon fails to bind to port
│  ├─ TUI fails to connect to daemon
│  └─ Graceful fallback or clear error
├─ Missing dependencies
│  ├─ Database file missing → create it
│  ├─ Config file missing → use defaults
│  └─ Agent config missing → load defaults
├─ Corrupted files
│  ├─ Invalid JSON in database
│  ├─ Corrupted config files
│  └─ Recovery or clear error
├─ Resource constraints
│  ├─ Low memory (< 100MB available)
│  ├─ Low disk space (< 10MB available)
│  └─ File permission issues
└─ Recovery mechanisms
   ├─ Automatic restart on crash
   ├─ State preservation across crashes
   ├─ Log rotation working
   └─ Cleanup of stale resources

MODE 5: Daemon Lifecycle (daemon mode tests)
├─ cco daemon start
├─ cco daemon stop
├─ cco daemon restart
├─ cco daemon status
├─ cco daemon logs
├─ cco daemon install (system service)
├─ cco daemon uninstall
├─ cco daemon enable (start on boot)
├─ cco daemon disable
└─ Tests:
   ├─ Daemon starts in background
   ├─ PID file created at ~/.local/share/cco/pids/
   ├─ Status command shows running state
   ├─ Logs accessible via command
   ├─ Log rotation works (10MB threshold)
   ├─ Service installation succeeds
   ├─ Service starts on boot (if enabled)
   ├─ Multiple instances can run on different ports
   └─ Graceful shutdown releases resources

MODE 6: Command Modes
├─ cco version → shows version
├─ cco health → checks running instance
├─ cco status → shows all running instances
├─ cco shutdown → graceful shutdown
├─ cco logs → show logs
├─ cco install → install binary
├─ cco update → check/install updates
├─ cco dashboard → launch TUI dashboard
└─ cco config → manage configuration

└─ All modes fail gracefully with clear error messages
```

### Configuration Coverage Matrix

```
ENVIRONMENT VARIABLES:
├─ CCO_PROJECT_PATH
│  ├─ Absolute path: /home/user/projects/my-project
│  ├─ Relative path: ./projects/my-project
│  ├─ Non-existent path: /nonexistent → clear error
│  └─ Not set → derive from cwd
├─ RUST_LOG
│  ├─ trace → verbose logs
│  ├─ debug → debug logs
│  ├─ info → info logs (default)
│  ├─ warn → warnings only
│  └─ error → errors only
├─ PORT (if supported)
├─ DATABASE_URL
│  ├─ sqlite://cco.db → local
│  ├─ postgresql://... → network
│  └─ Invalid format → clear error

WORKING DIRECTORY:
├─ Start from /
├─ Start from /Users/username
├─ Start from /Users/username/projects/cco
├─ Symlinked directories
├─ NFS mounted directories
└─ Read-only directories

FILE SYSTEM STATE:
├─ ~/.local/share/cco/ missing → create it
├─ ~/.local/share/cco/pids/ missing → create it
├─ ~/.local/share/cco/logs/ missing → create it
├─ Existing PID files → cleanup stale ones
├─ Existing log files → rotation works
└─ Permission issues → clear error
```

---

## 2. Test Phases

### Phase 1: Unit Tests

**Scope**: Individual functions and components in isolation
**Framework**: Rust: `#[cfg(test)]` modules with `tokio::test`
**Execution**: `cargo test --lib`

```rust
// Examples of unit tests to implement:

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_validation() {
        // Test: Valid ports (1-65535)
        // Test: Invalid ports (0, 65536, negative)
        // Test: Reserved ports (< 1024)
    }

    #[tokio::test]
    async fn test_server_state_initialization() {
        // Test: ServerState created with correct defaults
        // Test: Cache initialized properly
        // Test: Router set up correctly
    }

    #[test]
    fn test_pid_file_parsing() {
        // Test: PID file created with correct format
        // Test: PID file read and parsed correctly
        // Test: Stale PID files detected
    }

    #[tokio::test]
    async fn test_health_endpoint_response() {
        // Test: /health returns 200 OK
        // Test: Response is valid JSON
        // Test: Contains required fields (version, status, etc)
    }

    #[test]
    fn test_configuration_loading() {
        // Test: Default config loaded when file missing
        // Test: Custom config overrides defaults
        // Test: Invalid config raises error
    }

    #[tokio::test]
    async fn test_graceful_shutdown_flag() {
        // Test: Shutdown flag set atomically
        // Test: Background tasks check flag
        // Test: Tasks exit cleanly when flag set
    }

    #[tokio::test]
    async fn test_connection_tracker() {
        // Test: Connections tracked accurately
        // Test: Rate limiting works
        // Test: Cleanup on disconnect
    }

    #[tokio::test]
    async fn test_terminal_session_creation() {
        // Test: Terminal session created
        // Test: PTY allocated correctly
        // Test: Session cleanup on close
    }

    #[tokio::test]
    async fn test_model_override_application() {
        // Test: Model override replaces default
        // Test: Missing overrides use default
        // Test: Invalid models rejected
    }
}
```

**Critical Unit Tests** (must-pass):
- Port validation (valid range, conflicts detection)
- Configuration parsing (defaults, overrides, validation)
- Graceful shutdown mechanism (flag setting and checking)
- Health endpoint response format
- PID file creation and cleanup
- Connection tracking accuracy

### Phase 2: Integration Tests

**Scope**: Multiple components working together (server + database, server + terminal, etc)
**Framework**: Rust `#[tokio::test]` in `/tests/` directory
**Execution**: `cargo test --test '*'`

```rust
// tests/mode1_server_integration_tests.rs
// Tests for explicit server mode: cco run --port XXXX

#[tokio::test]
async fn test_server_startup_and_health() {
    // Setup: Ensure port 3000 is free
    let server = start_test_server("127.0.0.1", 3000, false).await;

    // Test 1: Server started
    assert!(server.is_running());

    // Test 2: Health endpoint responds
    let health = client.get("/health").send().await.unwrap();
    assert_eq!(health.status(), 200);

    // Test 3: Response is valid JSON
    let body: HealthResponse = health.json().await.unwrap();
    assert_eq!(body.status, "ok");
    assert!(body.version.len() > 0);

    // Cleanup: Server shutdown
    server.shutdown().await;
}

#[tokio::test]
async fn test_server_endpoint_availability() {
    let server = start_test_server("127.0.0.1", 3001, false).await;

    // Test each critical endpoint
    let endpoints = vec![
        "/health",
        "/api/agents",
        "/api/v1/chat",
        "/",  // index.html
    ];

    for endpoint in endpoints {
        let response = client.get(endpoint).send().await.unwrap();
        assert!(response.status().is_success(),
                "Endpoint {} returned {}", endpoint, response.status());
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_websocket_terminal_connection() {
    let server = start_test_server("127.0.0.1", 3002, false).await;

    // Connect to WebSocket
    let ws = connect_websocket("ws://127.0.0.1:3002/terminal").await;
    assert!(ws.is_ok(), "WebSocket connection failed");

    let mut ws = ws.unwrap();

    // Send test message
    ws.send(Message::Text("echo test\n".to_string())).await.unwrap();

    // Receive response
    let response = ws.recv().await;
    assert!(response.is_some(), "No response from terminal");

    ws.close().await.unwrap();
    server.shutdown().await;
}

#[tokio::test]
async fn test_server_graceful_shutdown() {
    let server = start_test_server("127.0.0.1", 3003, false).await;

    // Let it run briefly
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Measure shutdown time
    let shutdown_start = Instant::now();
    server.shutdown().await;
    let shutdown_duration = shutdown_start.elapsed();

    // Assert: Shutdown < 2 seconds
    assert!(shutdown_duration < Duration::from_secs(2),
            "Shutdown took {:?} (expected < 2s)", shutdown_duration);

    // Assert: Port released
    tokio::time::sleep(Duration::from_millis(500)).await;
    let listener = TcpListener::bind("127.0.0.1:3003").await;
    assert!(listener.is_ok(), "Port 3003 still in use after shutdown");
}

// tests/mode2_tui_daemon_integration_tests.rs
// Tests for default TUI/daemon mode: cco (no args)

#[tokio::test]
async fn test_tui_daemon_initialization_sequence() {
    // This test verifies the critical startup order:
    // 1. TUI app init → 2. Daemon start → 3. Daemon binds port → 4. TUI connects

    let mut child = Command::new("cargo")
        .args(&["run", "--release"])
        .env("RUST_LOG", "debug")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cco");

    // Wait for daemon to be ready (max 5 seconds)
    let daemon_ready = wait_for_daemon_ready("127.0.0.1:3000", Duration::from_secs(5)).await;
    assert!(daemon_ready, "Daemon failed to start listening");

    // Verify TUI can connect
    let tui_connected = can_connect_to_daemon("127.0.0.1:3000").await;
    assert!(tui_connected, "TUI cannot connect to daemon");

    // Graceful shutdown
    child.kill().expect("Failed to kill process");
    let status = child.wait().expect("Failed to wait for exit");
    assert!(status.success() || status.code() == Some(130),
            "Exit code should be 0 or 130 (SIGINT)");
}

#[tokio::test]
async fn test_tui_dashboard_data_loading() {
    let server = start_test_tui_daemon().await;

    // Wait for dashboard to fully initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test: /api/agents endpoint accessible
    let agents = client.get("http://127.0.0.1:3000/api/agents").send().await;
    assert!(agents.is_ok(), "Failed to fetch agents");

    let agents_list = agents.unwrap().json::<Vec<Agent>>().await;
    assert!(agents_list.is_ok(), "Invalid agents JSON");
    assert!(!agents_list.unwrap().is_empty(), "Agents list empty");

    server.shutdown().await;
}

#[tokio::test]
async fn test_tui_graceful_shutdown_both_processes() {
    let start = Instant::now();
    let mut child = start_test_tui_daemon().await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send SIGINT (Ctrl+C)
    child.kill().expect("Failed to kill TUI");

    let wait_result = child.wait();
    let shutdown_duration = start.elapsed();

    assert!(shutdown_duration < Duration::from_secs(3),
            "TUI + daemon shutdown took {:?}", shutdown_duration);

    // Verify both processes are gone
    tokio::time::sleep(Duration::from_millis(500)).await;
    let listener = TcpListener::bind("127.0.0.1:3000").await;
    assert!(listener.is_ok(), "Port 3000 still in use after TUI shutdown");
}
```

**Critical Integration Tests** (must-pass):
- Server starts and health endpoint responds
- All critical endpoints accessible and return correct status
- WebSocket terminal endpoint available
- Graceful shutdown < 2 seconds, port released
- TUI/Daemon initialization sequence correct
- TUI can connect to daemon
- Both processes exit cleanly on Ctrl+C

### Phase 3: E2E Tests

**Scope**: Complete workflows (user perspective)
**Framework**: Playwright or shell scripts
**Execution**: `npm run test:e2e` or `./tests/e2e.sh`

```bash
#!/bin/bash
# tests/e2e_mode1_server.sh - End-to-end tests for Mode 1

set -e

# Test 1: Server startup and access
test_server_startup() {
    echo "E2E Test 1: Server startup and access"

    # Start server
    timeout 10 cargo run --release -- run --port 3000 &
    PID=$!

    # Wait for startup
    sleep 3

    # Access health endpoint
    HEALTH=$(curl -s http://127.0.0.1:3000/health)
    assert_contains "$HEALTH" '"status"' "Health endpoint missing status field"

    # Access dashboard
    DASHBOARD=$(curl -s http://127.0.0.1:3000/)
    assert_contains "$DASHBOARD" 'DOCTYPE' "Dashboard HTML not returned"

    # Graceful shutdown
    kill -INT $PID
    wait $PID 2>/dev/null || true

    echo "✓ PASS: Server startup and access"
}

# Test 2: Port binding variations
test_port_binding() {
    echo "E2E Test 2: Port binding variations"

    for PORT in 3000 3001 3100 9000; do
        timeout 5 cargo run --release -- run --port $PORT &
        PID=$!

        sleep 2

        if curl -s http://127.0.0.1:$PORT/health > /dev/null 2>&1; then
            echo "✓ Port $PORT: OK"
        else
            echo "✗ Port $PORT: FAILED"
            kill $PID
            return 1
        fi

        kill -INT $PID
        wait $PID 2>/dev/null || true
        sleep 1
    done

    echo "✓ PASS: Port binding variations"
}

# Test 3: Configuration precedence
test_configuration_precedence() {
    echo "E2E Test 3: Configuration precedence"

    # Test 3a: Default config
    echo "Testing default configuration..."
    timeout 5 cargo run --release -- run --debug &
    PID=$!
    sleep 2

    HEALTH=$(curl -s http://127.0.0.1:3000/health)
    assert_contains "$HEALTH" 'debug' "Debug mode not reflected"

    kill -INT $PID
    wait $PID 2>/dev/null || true

    # Test 3b: Environment variable override
    echo "Testing environment variable override..."
    export CCO_PROJECT_PATH="/tmp/test-project"
    timeout 5 cargo run --release -- run --port 3000 &
    PID=$!
    sleep 2

    kill -INT $PID
    wait $PID 2>/dev/null || true

    echo "✓ PASS: Configuration precedence"
}

# Test 4: TUI mode initialization
test_tui_mode_initialization() {
    echo "E2E Test 4: TUI mode initialization"

    # Start with no arguments (TUI/daemon mode)
    timeout 10 cargo run --release &
    PID=$!

    # Wait for daemon to be ready
    WAIT_COUNT=0
    while [ $WAIT_COUNT -lt 50 ]; do
        if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
            echo "✓ Daemon ready after $((WAIT_COUNT * 100))ms"
            break
        fi
        sleep 0.1
        WAIT_COUNT=$((WAIT_COUNT + 1))
    done

    if [ $WAIT_COUNT -ge 50 ]; then
        echo "✗ Daemon failed to start"
        kill $PID
        return 1
    fi

    # Verify API accessible
    AGENTS=$(curl -s http://127.0.0.1:3000/api/agents)
    assert_contains "$AGENTS" 'agents' "Agents API empty"

    # Graceful shutdown
    kill -INT $PID
    wait $PID 2>/dev/null || true
    sleep 1

    # Verify port released
    if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
        echo "✗ Port 3000 still in use after shutdown"
        return 1
    fi

    echo "✓ PASS: TUI mode initialization"
}

# Run all tests
echo "Starting E2E Test Suite"
test_server_startup
test_port_binding
test_configuration_precedence
test_tui_mode_initialization

echo ""
echo "✓ All E2E tests passed"
```

**Critical E2E Tests** (must-pass):
- Server starts with specified port
- Dashboard loads and renders correctly
- API endpoints respond with correct data
- WebSocket connections work
- TUI/daemon initialization sequence correct
- Graceful shutdown both modes < 2 seconds
- Port released immediately after shutdown
- No hanging processes

### Phase 4: Acceptance Tests

**Scope**: Real-world user scenarios and performance
**Framework**: Shell scripts + custom tooling
**Execution**: `./tests/acceptance.sh`

```bash
#!/bin/bash
# tests/acceptance.sh - Acceptance tests for production readiness

# Test 1: 5-minute stability test
test_stability_5min() {
    echo "Acceptance Test 1: 5-minute stability"

    timeout 305 cargo run --release -- run --port 3000 &
    PID=$!

    sleep 3

    # Make requests every second
    ERROR_COUNT=0
    for i in {1..300}; do
        if ! curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
            ((ERROR_COUNT++))
        fi
        sleep 1
    done

    kill -INT $PID
    wait $PID 2>/dev/null || true

    if [ $ERROR_COUNT -eq 0 ]; then
        echo "✓ PASS: No errors in 5 minutes"
    else
        echo "✗ FAIL: $ERROR_COUNT errors in 5 minutes"
        return 1
    fi
}

# Test 2: Load test (100 concurrent requests)
test_concurrent_load() {
    echo "Acceptance Test 2: Concurrent load (100 requests)"

    timeout 30 cargo run --release -- run --port 3000 &
    PID=$!

    sleep 3

    # Send 100 concurrent requests
    for i in {1..100}; do
        curl -s http://127.0.0.1:3000/api/agents > /dev/null 2>&1 &
    done

    # Wait for all background jobs
    wait

    # Verify server still responsive
    if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
        echo "✓ PASS: Server responsive after 100 concurrent requests"
    else
        echo "✗ FAIL: Server became unresponsive"
        kill $PID
        return 1
    fi

    kill -INT $PID
    wait $PID 2>/dev/null || true
}

# Test 3: Multiple instances
test_multiple_instances() {
    echo "Acceptance Test 3: Multiple instances on different ports"

    # Start 3 instances on different ports
    for PORT in 3000 3001 3002; do
        timeout 30 cargo run --release -- run --port $PORT &
    done

    sleep 3

    # Verify all responding
    for PORT in 3000 3001 3002; do
        if ! curl -s http://127.0.0.1:$PORT/health > /dev/null 2>&1; then
            echo "✗ FAIL: Instance on port $PORT not responding"
            pkill -f "cargo run.*--port"
            return 1
        fi
    done

    echo "✓ PASS: All 3 instances responding"

    # Shutdown all
    pkill -f "cargo run.*--port"
    sleep 2
}

# Test 4: Resource usage
test_resource_usage() {
    echo "Acceptance Test 4: Resource usage"

    timeout 30 cargo run --release -- run --port 3000 &
    PID=$!

    sleep 3

    # Check memory usage (should be < 100MB)
    MEMORY=$(ps aux | grep $PID | grep -v grep | awk '{print $6}')

    if [ "$MEMORY" -lt 100000 ]; then
        echo "✓ PASS: Memory usage acceptable ($MEMORY KB)"
    else
        echo "✗ FAIL: Memory usage high ($MEMORY KB)"
    fi

    kill -INT $PID
    wait $PID 2>/dev/null || true
}

# Run all tests
run_all_acceptance_tests() {
    test_stability_5min || return 1
    test_concurrent_load || return 1
    test_multiple_instances || return 1
    test_resource_usage || return 1

    echo ""
    echo "✓ All acceptance tests passed"
}

run_all_acceptance_tests
```

**Critical Acceptance Tests** (must-pass):
- Server stable for 5 minutes without errors
- Handles 100 concurrent requests
- Multiple instances can run on different ports
- Resource usage acceptable (< 100MB memory)
- Performance benchmarks met (response time < 1s avg)

---

## 3. Critical Test Checklist

This checklist identifies **must-pass tests** that catch mode-specific failures:

```
STARTUP TESTS (Mode-Specific)
├─ [✓] Mode 1: cco run --port 3000 starts successfully
├─ [✓] Mode 2: cco (no args) TUI/daemon mode starts
├─ [✓] Mode 2: Daemon starts BEFORE TUI tries to connect
├─ [✓] Mode 2: TUI can connect to daemon on localhost:3000
├─ [✓] Mode 2: Connection succeeds within 5 seconds
├─ [✓] Both modes: Health endpoint accessible
├─ [✓] Both modes: /api/agents endpoint accessible
├─ [✓] Both modes: Required agents are loaded
└─ [✓] Both modes: Version string displayed

PORT & BINDING TESTS
├─ [✓] Port 3000 binds successfully
├─ [✓] Custom port (3001, 3100, etc) binds
├─ [✓] Localhost (127.0.0.1) binding works
├─ [✓] All interfaces (0.0.0.0) binding works
├─ [✓] IPv6 (::1) binding works (if supported)
├─ [✓] Port already in use detected
├─ [✓] Clear error when port in use
└─ [✓] Fallback mechanism works (if implemented)

SHUTDOWN TESTS (Most Critical)
├─ [✓] Ctrl+C (SIGINT) initiates shutdown
├─ [✓] Shutdown completes within 2 seconds
├─ [✓] No hanging processes after shutdown
├─ [✓] Port immediately available for reuse
├─ [✓] "Shutting down gracefully" message in logs
├─ [✓] PID file removed
├─ [✓] Database connections closed
├─ [✓] Multiple Ctrl+C handled gracefully
├─ [✓] SIGTERM also works (kill -TERM)
└─ [✓] No data corruption on shutdown

DAEMON/TUI COMMUNICATION TESTS (Mode 2 Specific)
├─ [✓] TUI process starts
├─ [✓] Daemon process starts
├─ [✓] Daemon listens on port 3000
├─ [✓] TUI connects to localhost:3000
├─ [✓] Connection timeout set (5s max)
├─ [✓] Reconnection works if daemon restarts
├─ [✓] Dashboard data syncs from daemon
├─ [✓] Terminal data flows through daemon
├─ [✓] Both processes shutdown together
└─ [✓] No orphaned processes

HEALTH CHECK TESTS
├─ [✓] /health endpoint exists
├─ [✓] Returns HTTP 200 OK
├─ [✓] Returns valid JSON
├─ [✓] Contains 'status' field
├─ [✓] Contains 'version' field
├─ [✓] Contains 'uptime' field
├─ [✓] Responds within 100ms
├─ [✓] Works under load (100+ concurrent)
└─ [✓] Accessible immediately after startup

API ENDPOINT TESTS
├─ [✓] GET /api/agents returns list
├─ [✓] GET /api/agents response is valid JSON
├─ [✓] Agents have required fields (name, type, model)
├─ [✓] POST /api/v1/chat accepts requests
├─ [✓] Chat responses are valid JSON
├─ [✓] POST /api/v1/chat/models returns model list
├─ [✓] GET / returns HTML (index.html)
├─ [✓] Static files served (CSS, JS, images)
├─ [✓] 404 for non-existent endpoints
└─ [✓] CORS headers present for cross-origin

LOGGING TESTS
├─ [✓] Startup messages appear once
├─ [✓] No spam logging (e.g., CCO_PROJECT_PATH every 5s)
├─ [✓] Debug flag increases verbosity
├─ [✓] RUST_LOG environment variable respected
├─ [✓] Log level: trace, debug, info, warn, error working
├─ [✓] No ERROR level messages during normal operation
├─ [✓] No repeating warning messages
├─ [✓] Logs are structured and parseable
└─ [✓] Log rotation works (10MB threshold)

CONFIGURATION TESTS
├─ [✓] Default config used when no args
├─ [✓] Command-line args override defaults
├─ [✓] Environment variables work
├─ [✓] Config file loading works
├─ [✓] CCO_PROJECT_PATH environment variable respected
├─ [✓] Database URL configuration works
├─ [✓] Cache size configuration works
├─ [✓] Cache TTL configuration works
└─ [✓] Invalid config raises clear error

STRESS/EDGE CASE TESTS
├─ [✓] 5 minute stability test (no crashes)
├─ [✓] 100 concurrent requests handled
├─ [✓] Memory usage < 100MB
├─ [✓] Handle missing database file (create it)
├─ [✓] Handle missing config file (use defaults)
├─ [✓] Handle corrupted PID file (cleanup)
├─ [✓] Handle permission errors (clear message)
├─ [✓] Low disk space handling
└─ [✓] Low memory handling

INTEGRATION TESTS (Both Modes Together)
├─ [✓] Start Mode 1, verify stable
├─ [✓] Stop Mode 1 cleanly
├─ [✓] Start Mode 2, verify TUI/daemon working
├─ [✓] Stop Mode 2 cleanly
├─ [✓] Start multiple instances (different ports)
├─ [✓] All instances shutdown together
├─ [✓] No port conflicts
└─ [✓] No resource leaks
```

---

## 4. Improved Smoke Test

This script replaces the existing smoke test and validates **BOTH execution modes**:

```bash
#!/bin/bash
# tests/smoke_test_comprehensive.sh
# Comprehensive smoke test validating both execution modes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_TESTS=0
PASSED=0
FAILED=0

# Helper functions
print_header() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo ""
}

test_passed() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
    ((TOTAL_TESTS++))
}

test_failed() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED++))
    ((TOTAL_TESTS++))
}

test_warning() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
}

cleanup() {
    # Kill all background cco processes
    pkill -f "cargo run.*--port" 2>/dev/null || true
    pkill -f "cco run" 2>/dev/null || true
    sleep 1
}

trap cleanup EXIT

print_header "COMPREHENSIVE CCO SMOKE TEST"
echo "Testing Mode 1 (Explicit Server) and Mode 2 (TUI/Daemon)"
echo ""

# ============================================================================
# MODE 1: EXPLICIT SERVER MODE
# ============================================================================

print_header "MODE 1: EXPLICIT SERVER MODE"

# Test 1.1: Server starts successfully
((TOTAL_TESTS++))
echo "Test 1.1: Server startup on default port (3000)..."
export NO_BROWSER=1

timeout 8 cargo run --release -- run --port 3000 2>&1 > /tmp/mode1_test.log &
PID=$!

sleep 3

if ps -p $PID > /dev/null 2>&1; then
    test_passed "Server process running (PID: $PID)"
else
    test_failed "Server process not running"
    exit 1
fi

# Test 1.2: Health endpoint responds
((TOTAL_TESTS++))
echo "Test 1.2: Health endpoint responds..."

HEALTH=$(curl -s http://127.0.0.1:3000/health)

if echo "$HEALTH" | jq . > /dev/null 2>&1; then
    test_passed "Health endpoint returns valid JSON"
else
    test_failed "Health endpoint response invalid"
    kill $PID
    exit 1
fi

# Test 1.3: /api/agents endpoint accessible
((TOTAL_TESTS++))
echo "Test 1.3: /api/agents endpoint accessible..."

AGENTS=$(curl -s http://127.0.0.1:3000/api/agents)

if echo "$AGENTS" | jq . > /dev/null 2>&1; then
    test_passed "/api/agents endpoint returns valid JSON"
else
    test_failed "/api/agents endpoint invalid"
    kill $PID
    exit 1
fi

# Test 1.4: Graceful shutdown < 2 seconds
((TOTAL_TESTS++))
echo "Test 1.4: Graceful shutdown performance..."

SHUTDOWN_START=$(date +%s%N)
kill -INT $PID 2>/dev/null || true
SHUTDOWN_SIGNAL=$(date +%s%N)

wait $PID 2>/dev/null || true
SHUTDOWN_COMPLETE=$(date +%s%N)

SHUTDOWN_MS=$(((SHUTDOWN_COMPLETE - SHUTDOWN_SIGNAL) / 1000000))

if [ $SHUTDOWN_MS -lt 2000 ]; then
    test_passed "Shutdown completed in ${SHUTDOWN_MS}ms (< 2000ms)"
else
    test_failed "Shutdown took ${SHUTDOWN_MS}ms (expected < 2000ms)"
fi

# Test 1.5: Port released after shutdown
((TOTAL_TESTS++))
echo "Test 1.5: Port released after shutdown..."

sleep 1

if lsof -i :3000 > /dev/null 2>&1; then
    test_failed "Port 3000 still in use"
else
    test_passed "Port 3000 released successfully"
fi

# Test 1.6: Check logs for errors
((TOTAL_TESTS++))
echo "Test 1.6: Log file analysis..."

if grep -q "ERROR\|FAIL\|PANIC" /tmp/mode1_test.log; then
    test_warning "Error messages found in logs"
else
    test_passed "No error messages in logs"
fi

# ============================================================================
# MODE 2: DEFAULT TUI/DAEMON MODE (CRITICAL TEST)
# ============================================================================

print_header "MODE 2: DEFAULT TUI/DAEMON MODE"

# Test 2.1: TUI/Daemon startup
((TOTAL_TESTS++))
echo "Test 2.1: TUI/Daemon startup (no arguments)..."

timeout 10 cargo run --release 2>&1 > /tmp/mode2_test.log &
PID=$!

# Wait for daemon to be ready (critical: daemon must bind BEFORE TUI connects)
READY=0
for i in {1..50}; do
    if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
        READY=1
        break
    fi
    sleep 0.1
done

if [ $READY -eq 1 ]; then
    test_passed "Daemon ready on port 3000 (after $((i * 100))ms)"
else
    test_failed "Daemon failed to start or bind to port 3000"
    kill $PID
    exit 1
fi

# Test 2.2: TUI process started
((TOTAL_TESTS++))
echo "Test 2.2: TUI process is running..."

if ps -p $PID > /dev/null 2>&1; then
    test_passed "TUI/Daemon process running (PID: $PID)"
else
    test_failed "TUI/Daemon process not running"
    exit 1
fi

# Test 2.3: Health endpoint accessible from Mode 2
((TOTAL_TESTS++))
echo "Test 2.3: Health endpoint accessible in Mode 2..."

HEALTH=$(curl -s http://127.0.0.1:3000/health)

if echo "$HEALTH" | jq . > /dev/null 2>&1; then
    test_passed "Health endpoint responds in TUI/Daemon mode"
else
    test_failed "Health endpoint not accessible in TUI/Daemon mode"
    kill $PID
    exit 1
fi

# Test 2.4: /api/agents accessible from Mode 2
((TOTAL_TESTS++))
echo "Test 2.4: /api/agents accessible in Mode 2..."

AGENTS=$(curl -s http://127.0.0.1:3000/api/agents)

if echo "$AGENTS" | jq . > /dev/null 2>&1; then
    test_passed "/api/agents accessible in TUI/Daemon mode"
else
    test_failed "/api/agents not accessible in TUI/Daemon mode"
    kill $PID
    exit 1
fi

# Test 2.5: Graceful shutdown Mode 2
((TOTAL_TESTS++))
echo "Test 2.5: Graceful shutdown of TUI/Daemon..."

SHUTDOWN_START=$(date +%s%N)
kill -INT $PID 2>/dev/null || true
SHUTDOWN_SIGNAL=$(date +%s%N)

wait $PID 2>/dev/null || true
SHUTDOWN_COMPLETE=$(date +%s%N)

SHUTDOWN_MS=$(((SHUTDOWN_COMPLETE - SHUTDOWN_SIGNAL) / 1000000))

if [ $SHUTDOWN_MS -lt 3000 ]; then
    test_passed "TUI/Daemon shutdown in ${SHUTDOWN_MS}ms (< 3000ms)"
else
    test_failed "TUI/Daemon shutdown took ${SHUTDOWN_MS}ms (expected < 3000ms)"
fi

# Test 2.6: Port released after Mode 2 shutdown
((TOTAL_TESTS++))
echo "Test 2.6: Port 3000 released after Mode 2 shutdown..."

sleep 1

if lsof -i :3000 > /dev/null 2>&1; then
    test_failed "Port 3000 still in use after Mode 2 shutdown"
else
    test_passed "Port 3000 released after Mode 2 shutdown"
fi

# ============================================================================
# SUMMARY
# ============================================================================

print_header "SMOKE TEST SUMMARY"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED ($PASSED/$TOTAL_TESTS)${NC}"
    echo ""
    echo "Both execution modes validated:"
    echo "  Mode 1: cco run --port 3000 ✓"
    echo "  Mode 2: cco (TUI/Daemon) ✓"
    exit 0
else
    echo -e "${RED}✗ TESTS FAILED ($FAILED/$TOTAL_TESTS failed, $PASSED/$TOTAL_TESTS passed)${NC}"
    echo ""
    echo "Check logs:"
    echo "  Mode 1: /tmp/mode1_test.log"
    echo "  Mode 2: /tmp/mode2_test.log"
    exit 1
fi
```

**Usage**:
```bash
chmod +x tests/smoke_test_comprehensive.sh
./tests/smoke_test_comprehensive.sh
```

---

## 5. Prevention Strategy

### Problem Statement

The current single-mode smoke test allowed Mode 2 (TUI/Daemon) to fail silently because tests only validated Mode 1 (explicit server).

### Solution: Multi-Layered Prevention

#### Layer 1: Test Naming Convention

```
tests/
├── unit/
│   ├── config_parsing.rs          # Tests for configuration module
│   ├── port_validation.rs         # Port handling unit tests
│   └── graceful_shutdown.rs       # Shutdown mechanism tests
│
├── integration/
│   ├── mode1_server_basic.rs      # Mode 1: Basic server operations
│   ├── mode1_server_endpoints.rs  # Mode 1: API endpoints
│   ├── mode2_tui_daemon.rs        # Mode 2: TUI/Daemon initialization
│   └── mode2_tui_connection.rs    # Mode 2: TUI connects to daemon
│
└── e2e/
    ├── mode1_comprehensive.sh     # Mode 1: End-to-end workflow
    ├── mode2_comprehensive.sh     # Mode 2: End-to-end workflow
    └── smoke_test_both_modes.sh   # Both modes in one test
```

**Key Principle**: Test file naming INCLUDES the mode being tested:
- `mode1_*` tests ONLY Mode 1
- `mode2_*` tests ONLY Mode 2
- `smoke_test_both_modes.sh` tests BOTH modes

This naming prevents someone from thinking a single test covers all modes.

#### Layer 2: Test Documentation

```rust
// In every test file, document which mode it tests

#[cfg(test)]
mod mode1_server_tests {
    //! IMPORTANT: These tests validate MODE 1 ONLY (cco run --port XXXX)
    //!
    //! Mode 1: Explicit server mode
    //! Command: cco run --port 3000 --debug
    //!
    //! Related tests:
    //! - See mode2_tui_daemon.rs for Mode 2 tests (TUI/Daemon mode)
    //! - See smoke_test_both_modes.sh for combined testing

    #[tokio::test]
    async fn test_server_starts_on_specified_port() {
        // ... test code ...
    }
}

#[cfg(test)]
mod mode2_tui_daemon_tests {
    //! IMPORTANT: These tests validate MODE 2 ONLY (cco with no args)
    //!
    //! Mode 2: Default TUI/Daemon mode
    //! Command: cco (no arguments)
    //!
    //! Related tests:
    //! - See mode1_server_tests.rs for Mode 1 tests (explicit server)
    //! - See smoke_test_both_modes.sh for combined testing

    #[tokio::test]
    async fn test_tui_daemon_initialization_sequence() {
        // ... test code ...
    }
}
```

#### Layer 3: CI/CD Integration

```yaml
# .github/workflows/test.yml - Ensures both modes are tested

name: Test Suite

on: [push, pull_request]

jobs:
  test-mode-1-server:
    runs-on: ubuntu-latest
    name: "Test Mode 1: Explicit Server"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --lib mode1
      - run: cargo test --test '*mode1*'
      - run: ./tests/e2e/mode1_comprehensive.sh

  test-mode-2-tui-daemon:
    runs-on: ubuntu-latest
    name: "Test Mode 2: TUI/Daemon"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --lib mode2
      - run: cargo test --test '*mode2*'
      - run: ./tests/e2e/mode2_comprehensive.sh

  smoke-test-both-modes:
    runs-on: ubuntu-latest
    name: "Smoke Test: Both Modes"
    needs: [test-mode-1-server, test-mode-2-tui-daemon]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: ./tests/smoke_test_comprehensive.sh
```

#### Layer 4: Code Review Checklist

Add to pull request template:

```markdown
# Code Review Checklist

## Testing Requirements

- [ ] **Mode 1 Tests**: Run `cargo test --test '*mode1*'` - all pass
- [ ] **Mode 2 Tests**: Run `cargo test --test '*mode2*'` - all pass
- [ ] **Smoke Test**: Run `./tests/smoke_test_comprehensive.sh` - all pass
- [ ] **Manual Test Mode 1**: `cco run --port 3000` works
- [ ] **Manual Test Mode 2**: `cco` (no args) works
- [ ] No hardcoded mode assumptions
- [ ] Changes work with BOTH execution modes

## Mode-Specific Questions

- [ ] Does this change affect Mode 1 startup? (cco run --port XXXX)
- [ ] Does this change affect Mode 2 startup? (cco with no args)
- [ ] Does this change affect TUI/daemon communication?
- [ ] Does this change affect graceful shutdown?
- [ ] Could this change cause mode-specific failures?

If YES to any question:
- [ ] Both Mode 1 and Mode 2 tests added
- [ ] Smoke test validates both modes
- [ ] Manual testing completed for both modes
```

#### Layer 5: Regression Test Procedures

```bash
#!/bin/bash
# After any major change, run full test suite

echo "Running Regression Test Suite..."
echo ""

# Unit tests (fast)
echo "1. Unit Tests (Mode 1 & Mode 2)..."
cargo test --lib || exit 1
echo "   ✓ Unit tests passed"

# Integration tests (medium)
echo "2. Integration Tests (Mode 1 & Mode 2)..."
cargo test --test '*mode1*' || exit 1
cargo test --test '*mode2*' || exit 1
echo "   ✓ Integration tests passed"

# E2E tests (slow)
echo "3. E2E Tests (Mode 1)..."
./tests/e2e/mode1_comprehensive.sh || exit 1
echo "   ✓ Mode 1 E2E passed"

echo "4. E2E Tests (Mode 2)..."
./tests/e2e/mode2_comprehensive.sh || exit 1
echo "   ✓ Mode 2 E2E passed"

# Smoke test (quick validation)
echo "5. Smoke Test (Both Modes)..."
./tests/smoke_test_comprehensive.sh || exit 1
echo "   ✓ Smoke test passed"

echo ""
echo "✓ All regression tests passed"
```

---

## 6. Test Execution Matrix

### Local Development

```bash
# Quick validation (2-3 minutes)
cargo test --lib                    # Unit tests only

# Full validation (10-15 minutes)
cargo test --lib
cargo test --test '*'               # All integration tests
./tests/smoke_test_comprehensive.sh

# Specific mode testing
cargo test --lib mode1              # Mode 1 tests only
cargo test --test '*mode2*'         # Mode 2 tests only

# E2E for specific mode
./tests/e2e/mode1_comprehensive.sh
./tests/e2e/mode2_comprehensive.sh
```

### CI/CD Pipeline

```
Commit → Compile → Unit Tests → Integration Tests → E2E Tests
                        ↓              ↓                ↓
                  Mode 1 & 2    Mode 1 & 2      Mode 1 & 2
                   (parallel)    (parallel)      (sequential)
                        ↓              ↓                ↓
                      Pass          Pass    →   Smoke Test Both Modes
                        ↓              ↓                ↓
                     Report         Report          Pass → Merge ✓
```

### Test Timings

| Test Phase | Mode 1 | Mode 2 | Total | Environment |
|-----------|--------|--------|-------|-------------|
| Unit tests | 1m | 1m | 2m | Local / CI |
| Integration | 2m | 2m | 4m | Local / CI |
| E2E | 3m | 3m | 6m | Local / CI |
| Smoke (both) | - | - | 2m | Local / CI |
| **Full suite** | - | - | **8-10m** | Local / CI |
| Manual validation | 10m | 10m | 20m | Local only |

---

## 7. Test Implementation Timeline

### Phase A: Immediate (Week 1)

**Priority**: Eliminate mode-specific blind spot

1. Create improved smoke test (`smoke_test_comprehensive.sh`)
   - Validates both Mode 1 AND Mode 2
   - Runs in CI/CD immediately
   - Prevents regressions

2. Add Mode 2 integration tests (`tests/mode2_tui_daemon.rs`)
   - Test TUI/daemon initialization sequence
   - Test TUI connection to daemon
   - Test graceful shutdown of both

3. Update CI/CD pipeline
   - Add separate Mode 1 and Mode 2 test jobs
   - Add smoke test as required job
   - All three jobs must pass to merge

### Phase B: Short-term (Week 2-3)

**Priority**: Build comprehensive test coverage

1. Expand Mode 1 integration tests
   - All API endpoints
   - Configuration variations
   - Error cases

2. Expand Mode 2 integration tests
   - TUI/daemon communication
   - Dashboard data sync
   - Terminal data flow

3. Create E2E test scripts
   - Mode 1 complete workflow
   - Mode 2 complete workflow
   - Cross-mode compatibility

### Phase C: Long-term (Week 4+)

**Priority**: Production-ready test suite

1. Acceptance tests
   - 5-minute stability
   - Load testing (100+ concurrent)
   - Resource usage monitoring
   - Performance benchmarks

2. Automated test result reporting
   - Dashboard showing test trends
   - Historical failure analysis
   - Performance regression detection

3. Test maintenance and optimization
   - Reduce test execution time
   - Improve test reliability
   - Add edge case coverage

---

## 8. Success Criteria

### Immediate Success (This Week)

- [x] **Smoke test validates both modes** - identifies mode-specific failures
- [x] **Mode 2 tests created** - daemon startup and TUI connection tested
- [x] **CI/CD updated** - separate Mode 1/Mode 2 jobs
- [x] **No mode-specific blind spots** - all execution paths covered

### Short-term Success (2-3 Weeks)

- [x] **80%+ code coverage** - unit and integration tests
- [x] **All critical tests pass** - see Checklist section
- [x] **Zero mode-specific failures** - both modes validated
- [x] **Clear test naming** - `mode1_*` and `mode2_*` distinguish tests

### Long-term Success (Production-Ready)

- [x] **5-minute stability** - no crashes or errors
- [x] **Load testing** - handles 100+ concurrent requests
- [x] **Performance benchmarks** - response times < 1s average
- [x] **Resource efficiency** - memory < 100MB, CPU < 20%
- [x] **Zero regressions** - all tests pass on every commit

---

## 9. Risk Mitigation

### Known Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Mode 2 startup race condition | Critical | Daemon must bind BEFORE TUI connects; explicit wait logic |
| Port binding failure | High | Test port availability; detect and report clearly |
| Graceful shutdown timeout | High | Test < 2s shutdown; no hanging processes |
| Logging spam | Medium | Monitor log output; fail if spam detected |
| TUI/daemon communication | Medium | Test connection establishment; validate protocol |
| Configuration precedence | Low | Test environment vars, CLI args, config files |

### Testing Safeguards

1. **Dual-mode testing** - Every feature test in both modes
2. **Startup sequence validation** - Critical initialization order tested
3. **Resource cleanup** - Port release and process cleanup verified
4. **Error handling** - All error paths tested
5. **Integration points** - TUI-daemon communication thoroughly tested

---

## 10. Quick Reference

### Run All Tests

```bash
# Build
cargo build --release

# Unit tests (fast)
cargo test --lib

# Integration tests
cargo test --test '*'

# Smoke test (quick validation)
./tests/smoke_test_comprehensive.sh

# Manual Mode 1
cargo run --release -- run --port 3000

# Manual Mode 2
cargo run --release  # no arguments
```

### Test Files Structure

```
tests/
├── integration/
│   ├── mode1_server_basic.rs       # Mode 1 startup & endpoints
│   ├── mode1_server_shutdown.rs    # Mode 1 graceful shutdown
│   ├── mode2_tui_daemon.rs         # Mode 2 initialization
│   ├── mode2_tui_connection.rs     # Mode 2 TUI-daemon comm
│   └── both_modes_stability.rs     # Both modes stress tests
│
├── e2e/
│   ├── mode1_comprehensive.sh      # Mode 1 workflow
│   ├── mode2_comprehensive.sh      # Mode 2 workflow
│   └── smoke_test_comprehensive.sh # Both modes validation
│
└── common/
    ├── test_helpers.rs             # Shared test utilities
    └── fixtures/                   # Test data and configs
```

---

## Appendix A: Critical Execution Paths

### Mode 1: Explicit Server Flow

```
User: cco run --port 3000
  ↓
Parse CLI args (port=3000)
  ↓
Initialize logging (RUST_LOG from --debug)
  ↓
Create ServerState (cache, router, analytics)
  ↓
Bind TCP listener on 127.0.0.1:3000 ← CRITICAL: Must succeed
  ↓
Write PID file
  ↓
Start HTTP server (Axum)
  ↓
Print "Server running at http://127.0.0.1:3000"
  ↓
Listen for connections...
  ↓
[User presses Ctrl+C]
  ↓
Set shutdown_flag = true
  ↓
Close listener (no new connections)
  ↓
Wait for active connections to complete (< 2s timeout)
  ↓
Close database and cache
  ↓
Remove PID file
  ↓
Exit(0) ← CRITICAL: Must release port immediately
```

### Mode 2: TUI/Daemon Flow

```
User: cco (no arguments)
  ↓
Try to create TuiApp
  ├─ Yes: Proceed with TUI mode
  └─ No: Fall back to daemon mode
  ↓
Initialize TUI subsystems (crossterm, ratatui)
  ↓
Spawn daemon task:
  ├─ Parse daemon config
  ├─ Create ServerState
  ├─ Bind TCP listener on 127.0.0.1:3000 ← CRITICAL: Before TUI connects
  ├─ Write PID file
  └─ Start HTTP server
  ↓
Wait for daemon to be ready (health check)
  ↓
TUI connects to http://127.0.0.1:3000 ← CRITICAL: Daemon must be listening
  ↓
TUI renders dashboard (gets /api/agents, etc from daemon)
  ↓
Loop: Handle user input, update dashboard
  ↓
[User presses Ctrl+C or quits]
  ↓
TUI requests shutdown from daemon
  ↓
Daemon:
  ├─ Set shutdown_flag = true
  ├─ Close HTTP listener
  ├─ Wait for active connections (< 2s)
  ├─ Close database
  ├─ Remove PID file
  └─ Signal completion
  ↓
TUI cleanup
  ↓
Exit(0) ← CRITICAL: Must release port immediately
```

### Critical Failure Points

**Mode 1 Failures**:
1. Port binding fails (port in use)
2. Graceful shutdown hangs > 2s
3. Port not released after shutdown
4. Process doesn't exit (becomes zombie)

**Mode 2 Failures**:
1. Daemon fails to start (TUI mode fallback should work)
2. Daemon binds port AFTER TUI tries to connect (timeout)
3. TUI cannot connect to daemon (clear error)
4. Graceful shutdown of both takes > 3s
5. Port not released after both exit
6. Orphaned processes remain

---

## Appendix B: Test Data and Fixtures

### Minimal Configuration for Testing

```json
{
  "agents": [
    {
      "name": "Test Agent",
      "type": "test",
      "model": "gpt-4",
      "description": "Test agent for validation"
    }
  ],
  "cache": {
    "max_size": 1073741824,
    "ttl_seconds": 3600
  },
  "server": {
    "host": "127.0.0.1",
    "port": 3000
  }
}
```

### Test Database Schema

```sql
-- Minimal analytics table
CREATE TABLE api_calls (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME,
    endpoint TEXT,
    status_code INTEGER,
    response_time_ms INTEGER
);

CREATE TABLE activity_events (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME,
    event_type TEXT,
    details TEXT
);
```

---

**Document Status**: Ready for Implementation
**Next Step**: Create smoke_test_comprehensive.sh and mode2 integration tests
**Estimated Implementation Time**: 2-3 weeks for full coverage
**Owner**: QA Engineer / Test Engineer
