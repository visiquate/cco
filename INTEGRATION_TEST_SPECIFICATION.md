# Integration Test Specification: Daemon/TUI Lifecycle

**Target File**: `cco/tests/integration_daemon_lifecycle.rs`

**Objective**: Create comprehensive integration tests validating the daemon startup and TUI connection sequence, including proper lifecycle management and error handling.

## Test Suite 1: Daemon Startup Tests

### Test 1a: Daemon Starts Successfully
- Spawn daemon process on default port 3000
- Verify process starts without errors
- Verify daemon is listening on the port
- Clean shutdown after test

**Expected Behavior**: Process spawns, binds to port, logs indicate startup success

### Test 1b: Health Endpoint Returns Valid Response
- Start daemon
- Make HTTP GET request to `/health`
- Parse JSON response
- Verify fields: `status: "ok"`, `version`, `cache_stats`, `uptime`
- Verify uptime is > 0
- Verify cache_stats contains: `hit_rate`, `hits`, `misses`, `entries`, `total_savings`

**Expected**: HTTP 200 with valid JSON containing health metrics

### Test 1c: Ready Endpoint Indicates Daemon Ready
- Start daemon
- Make HTTP GET request to `/ready`
- Verify response contains: `ready: true`, `version`, `timestamp`
- Verify timestamp is recent (within 5 seconds)

**Expected**: HTTP 200, JSON with `ready: true`

### Test 1d: Critical Endpoints All Respond
- Start daemon
- Make requests to each endpoint:
  - `/health` → HTTP 200
  - `/ready` → HTTP 200
  - `/api/agents` → HTTP 200 (JSON array)
  - `/api/stats` → HTTP 200 (stats JSON)
  - `/api/stream` → HTTP 200 (SSE stream)
- Verify all respond within 5 seconds

**Expected**: All endpoints accessible and responding

## Test Suite 2: TUI/Daemon Connection Tests

### Test 2a: TUI Can Connect to Daemon API
- Start daemon
- Create HTTP client
- Make GET request to `/api/agents`
- Verify connection succeeds (no timeouts)
- Verify response is valid JSON array

**Expected**: Successfully retrieve agent list

### Test 2b: Agent List Endpoint Returns Correct Structure
- Get agents list from `/api/agents`
- Verify it's a JSON array (not wrapped object)
- Verify each agent has: `name`, `description`, `type`, `model`, `category`
- Verify at least one agent exists

**Expected**: Valid agent array with proper structure

### Test 2c: Stats Endpoint Returns Metrics
- Make request to `/api/stats`
- Verify response has: `project`, `machine`, `activity`, `chart_data`
- Verify `project` has: `name`, `cost`, `tokens`, `calls`, `last_updated`
- Verify `machine` has: `cpu`, `memory`, `uptime`, `process_count`
- Verify `activity` is array

**Expected**: Complete metrics JSON with all required fields

### Test 2d: WebSocket Terminal Endpoint Exists
- Make WebSocket upgrade request to `/terminal`
- Verify upgrade succeeds (101 Switching Protocols)
- Send a test message
- Verify response is received (shell prompt or output)
- Close connection cleanly

**Expected**: WebSocket connection established and functional

## Test Suite 3: Daemon Lifecycle Tests

### Test 3a: Daemon Starts Cleanly
- Spawn daemon process
- Verify exit code is 0 (or hasn't exited)
- Verify startup logs show "listening on"
- Verify no error messages in initial startup

**Expected**: Clean startup with success indicators

### Test 3b: Daemon Handles SIGINT Gracefully
- Start daemon
- Send SIGINT (Ctrl+C) signal
- Wait for graceful shutdown (max 2 seconds)
- Verify process exits cleanly
- Verify shutdown logs indicate graceful shutdown

**Expected**: Process exits with code 0 within 2 seconds

### Test 3c: Daemon Shuts Down in < 2 Seconds
- Start daemon
- Record start time
- Send SIGINT
- Record end time
- Verify elapsed time < 2000ms

**Expected**: Shutdown completes in less than 2 seconds

### Test 3d: Port is Released After Shutdown
- Start daemon on port 3000
- Shut down daemon gracefully
- Verify port 3000 is available
- Start daemon again on same port
- Verify second daemon starts successfully

**Expected**: Port released immediately, can be reused

### Test 3e: No Zombie Processes Remain
- Start daemon
- Shut down daemon
- List all processes for user
- Verify no zombie (defunct) processes related to cco
- Verify process cleanup is complete

**Expected**: No lingering processes or zombies

## Test Suite 4: Error Handling Tests

### Test 4a: Daemon Handles Port Conflicts Gracefully
- Start daemon on port 3001
- Try to start another daemon on same port 3001
- Verify second daemon fails to start
- Verify error message indicates port in use
- Verify first daemon still running and functional

**Expected**: Second daemon fails, first unaffected, clear error message

### Test 4b: Connection Errors are Reported Properly
- Start daemon
- Make request with invalid credentials/auth
- Verify HTTP error response (4xx or 5xx)
- Verify error message is clear
- Verify daemon remains running after error

**Expected**: Proper error responses, daemon stability

### Test 4c: Invalid Requests Get Proper Errors
- Make request to `/api/invalid-endpoint`
- Verify HTTP 404 response
- Make POST request with invalid JSON
- Verify HTTP 400 or 422 response
- Make request without required headers/params
- Verify HTTP 4xx response with clear error

**Expected**: Appropriate HTTP error codes, clear error messages

## Implementation Notes

### Test Infrastructure
- Use `tokio::test` for async tests
- Use `tempfile` for temporary test directories
- Use `reqwest` HTTP client (already in Cargo.toml)
- Use `tokio::time::timeout` for timeouts
- Pattern after existing `daemon_manager_tests.rs`

### Port Selection
- Use ephemeral/high ports (13000+) to avoid conflicts
- Release ports after each test
- Verify port availability before starting

### Process Management
- Spawn daemon via `std::process::Command`
- Capture stdout/stderr for debugging
- Use `tokio::task::spawn_blocking` for process operations
- Implement proper cleanup in test teardown

### Patterns to Follow
- See `tests/daemon_manager_tests.rs` for configuration patterns
- See `src/server.rs` for endpoint documentation
- Use `common` module for shared test utilities
- Clear assertion messages for debugging failures

### Module Structure
```rust
#[cfg(test)]
mod daemon_startup_tests { }

#[cfg(test)]
mod tui_connection_tests { }

#[cfg(test)]
mod daemon_lifecycle_tests { }

#[cfg(test)]
mod error_handling_tests { }

// Common helper functions
mod test_helpers {
    fn spawn_daemon(port: u16) -> ...
    fn shutdown_daemon(process: ...) -> ...
    fn wait_for_daemon(port: u16, timeout: Duration) -> ...
    fn health_check(port: u16) -> ...
}
```

## Success Criteria

✅ All 16 tests pass consistently
✅ Tests are idempotent (can run multiple times)
✅ Clear error messages for failures
✅ Fast execution (< 30 seconds total)
✅ Proper resource cleanup
✅ No zombie processes after tests
✅ No lingering daemon instances

## Run Command

```bash
cd /Users/brent/git/cc-orchestra/cco
cargo test --test integration_daemon_lifecycle -- --nocapture
```
