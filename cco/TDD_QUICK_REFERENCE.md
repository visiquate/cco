# TDD Test Suite - Quick Reference

## Test Execution Commands

```bash
# Change to project directory
cd /Users/brent/git/cc-orchestra/cco

# Run all tests
cargo test

# Run specific test file
cargo test --test daemon_manager_tests
cargo test --test api_client_tests
cargo test --test tui_integration_tests

# Run specific test by name
cargo test test_daemon_manager_check_daemon_not_running

# Run with verbose output
cargo test -- --nocapture

# Run tests and show ignored
cargo test -- --include-ignored

# Check test compilation only (no run)
cargo test --no-run
```

---

## Test Files

| File | Location | Tests | Purpose |
|------|----------|-------|---------|
| Common Utils | `tests/common/mod.rs` | 4 | Mock servers, helpers |
| Daemon Manager | `tests/daemon_manager_tests.rs` | 13 | Lifecycle management |
| API Client | `tests/api_client_tests.rs` | 16 | HTTP communication |
| TUI Integration | `tests/tui_integration_tests.rs` | 20 | TUI startup/operations |
| **TOTAL** | | **53** | |

---

## Current Status

- ✅ All tests written
- ✅ Tests compile successfully
- 🔴 Tests FAIL (expected - RED phase)
- ⏳ Awaiting implementation (GREEN phase)

---

## Implementation Priority

### Phase 1: DaemonManager Extensions
```rust
// File: src/daemon/lifecycle.rs

impl DaemonManager {
    // 1. Add health check method
    pub async fn check_health(&self) -> Result<bool> {
        // HTTP GET to http://127.0.0.1:{port}/health
        // Return true if status == "ok"
    }

    // 2. Add version fetching
    pub async fn get_running_version(&self) -> Result<String> {
        // Parse version from health response
    }

    // 3. Add version mismatch detection
    pub async fn check_version_mismatch(&self) -> Result<bool> {
        // Compare running daemon version vs binary version
    }

    // 4. Add ensure running
    pub async fn ensure_running(&self) -> Result<()> {
        // Check status, start if needed, wait for ready
    }
}
```

**Tests to pass**: 13 daemon manager tests

---

### Phase 2: ApiClient Module
```rust
// File: src/api_client.rs (NEW FILE)

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    timeout: Duration,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self { ... }
    pub async fn health(&self) -> Result<HealthResponse> { ... }
    pub async fn get_agents(&self) -> Result<Vec<Agent>> { ... }
    pub async fn stream_events(&self) -> Result<SseStream> { ... }
}

pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub port: u16,
}

pub struct Agent {
    pub name: String,
    pub type_name: String,
    pub model: String,
    pub capabilities: Vec<String>,
}
```

**Tests to pass**: 16 API client tests

---

### Phase 3: TUI Application
```rust
// File: src/tui/daemon_client.rs (NEW FILE)

pub struct TuiApp {
    current_tab: Tab,
    daemon_status: DaemonStatus,
    api_client: Option<ApiClient>,
    stream: Option<SseStream>,
    config: TuiConfig,
    should_quit: bool,
}

impl TuiApp {
    pub async fn new(config: TuiConfig) -> Result<Self> {
        // 1. Check daemon status
        // 2. Start daemon if needed
        // 3. Wait for ready
        // 4. Connect API
        // 5. Connect stream
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        // 1. Close stream
        // 2. Close API client
        // 3. Stop daemon if configured
    }
}

pub enum Tab {
    Overview,
    RealTime,
    CostAnalysis,
    SessionInfo,
}

pub enum DaemonStatus {
    Unknown,
    Starting,
    Running,
    Crashed,
    Stopped,
}
```

**Tests to pass**: 20 TUI integration tests

---

## Expected Data Formats

### Health Endpoint Response
```json
GET /health

{
  "status": "ok",
  "version": "2025.11.2",
  "uptime_seconds": 123,
  "port": 3000
}
```

### Agents Endpoint Response
```json
GET /api/agents

[
  {
    "name": "Chief Architect",
    "type_name": "system-architect",
    "model": "opus-4.1",
    "capabilities": ["architecture", "design"]
  }
]
```

### SSE Stream Format
```
GET /api/stream

event: message
data: {"type": "api_call", "model": "opus-4", "tokens": 1000}

event: metrics
data: {"total_cost": 5.25, "call_count": 42}

```

---

## Configuration Values

### Retry Configuration
- Max retries: 5
- Initial delay: 100ms
- Backoff multiplier: 2x
- Sequence: 100ms → 200ms → 400ms → 800ms → 1600ms
- Max delay: Configurable cap

### TUI Configuration
```rust
TuiConfig {
    daemon_url: "http://127.0.0.1:3000",
    daemon_ready_timeout: Duration::from_secs(30),
    stop_daemon_on_exit: true,
    auto_start_daemon: true,
    health_check_interval: Duration::from_secs(5),
}
```

### Daemon Configuration
```rust
DaemonConfig {
    port: 3000,
    host: "127.0.0.1",
    // ... existing fields
}
```

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit TUI |
| `r` | Restart daemon |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |

---

## Tab Navigation

```
Overview → Real-time → Cost Analysis → Session Info → (back to Overview)
```

---

## Dependencies Needed

### Add to Cargo.toml

```toml
[dependencies]
# Existing dependencies already include:
# - reqwest (HTTP client)
# - tokio (async runtime)
# - serde, serde_json (JSON serialization)
# - anyhow (error handling)

# May need to add:
reqwest-eventsource = "0.4"  # Already present - SSE support
```

### Dev Dependencies (Optional for Mocking)

```toml
[dev-dependencies]
mockito = "1.2"      # HTTP mocking
# OR
wiremock = "0.6"     # Alternative
```

---

## Test Patterns

### Arrange-Act-Assert
```rust
#[tokio::test]
async fn test_something() {
    // Arrange
    let config = DaemonConfig::default();
    let manager = DaemonManager::new(config);

    // Act
    let result = manager.check_health().await;

    // Assert
    assert!(result.is_ok());
}
```

### Error Testing
```rust
#[tokio::test]
async fn test_error_case() {
    let result = something_that_should_fail().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("expected error"));
}
```

### Mock Server (When Added)
```rust
#[tokio::test]
async fn test_with_mock() {
    let mock_server = MockDaemonServer::start().await.unwrap();

    // Use mock_server.port or mock_server.base_url
    let client = ApiClient::new(&mock_server.base_url);

    // ... test code
}
```

---

## Common Gotchas

1. **Async Tests**: Use `#[tokio::test]` not `#[test]`
2. **Port Conflicts**: Tests may conflict if running in parallel - use unique ports
3. **Cleanup**: Always clean up daemon processes in tests
4. **Timeouts**: Use reasonable timeouts (not too short, not too long)
5. **Mocking**: Some tests will need HTTP mocking for full isolation

---

## Progress Tracking

Check implementation progress:

```bash
# Run tests and count passing
cargo test 2>&1 | grep "test result"

# Example output:
# test result: FAILED. 0 passed; 53 failed; 0 ignored; 0 measured; 0 filtered out
#                       ^^^^^^^^ <- Track this number
```

Goal: Get to `53 passed; 0 failed`

---

## Documentation

- **Full Summary**: `TDD_TEST_SUITE_SUMMARY.md`
- **Deliverables**: `TDD_DELIVERABLES.md`
- **This Reference**: `TDD_QUICK_REFERENCE.md`

---

## Next Steps

1. ✅ Tests written (COMPLETE)
2. ⏳ Implement DaemonManager extensions
3. ⏳ Create ApiClient module
4. ⏳ Create TUI integration
5. ⏳ Run tests until all pass
6. ⏳ Refactor and optimize

**Current Phase**: 🔴 RED (tests fail)
**Next Phase**: 🟢 GREEN (make tests pass)
**Final Phase**: 🔵 REFACTOR (optimize)

---

## Questions?

- Check TODO comments in test files for implementation hints
- Review expected behavior in test assertions
- Look at existing code in `src/daemon/lifecycle.rs` for patterns
- All data formats are documented in test files

---

**Status**: Ready for implementation
**Test Count**: 53 tests
**Coverage Target**: 90%+
