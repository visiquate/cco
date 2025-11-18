# Test Strategy Quick Start Guide

**Purpose**: Actionable implementation roadmap for test strategy
**Audience**: Development team implementing tests
**Time to implement**: 2-3 weeks

---

## What's the Problem?

Current smoke test only validates **Mode 1** (explicit server):
```bash
cco run --debug --port 3000  ✓ Tested
```

But **Mode 2** (default TUI/daemon) is **NOT tested**:
```bash
cco  ← No arguments, starts TUI and daemon
     ✗ This mode failed, tests didn't catch it!
```

**Result**: Mode-specific failures go undetected until production.

---

## The Solution: 3-Layer Testing

### Layer 1: Improved Smoke Test (1-2 hours to implement)

Replace the single-mode smoke test with a script that validates **BOTH modes**:

```bash
# Current: Only validates Mode 1
cco run --debug --port 3000  ✓

# NEW: Validates Mode 1 AND Mode 2
cco run --port 3000          ✓ Mode 1
cco  (no args)               ✓ Mode 2 (CRITICAL)
```

**File to create**: `tests/smoke_test_comprehensive.sh`

See Section 4 of COMPREHENSIVE_TEST_STRATEGY.md for complete script.

**Why**: Quick validation that catches mode-specific failures immediately.

### Layer 2: Mode-Specific Integration Tests (3-4 hours)

Create separate test modules for each execution mode:

```rust
// Mode 1 tests: cco run --port XXXX
tests/integration/mode1_server_basic.rs
tests/integration/mode1_server_endpoints.rs
tests/integration/mode1_server_shutdown.rs

// Mode 2 tests: cco (no args)
tests/integration/mode2_tui_daemon.rs
tests/integration/mode2_tui_connection.rs
```

**Key Test Cases**:

Mode 1:
- Server starts on specified port
- Health endpoint responds
- All critical API endpoints accessible
- Graceful shutdown < 2 seconds
- Port released immediately

Mode 2:
- TUI process starts
- Daemon starts and listens on port 3000
- TUI can connect to daemon
- Dashboard loads data from daemon
- Both processes exit cleanly on Ctrl+C

**Why**: Integration tests catch interactions between components (TUI-daemon communication, port binding, shutdown).

### Layer 3: E2E Workflow Tests (2-3 hours)

Shell scripts that validate complete user workflows:

```bash
tests/e2e/mode1_comprehensive.sh   # Full Mode 1 workflow
tests/e2e/mode2_comprehensive.sh   # Full Mode 2 workflow
```

**What E2E tests validate**:
- Real-world command execution (not mocked)
- Complete startup-to-shutdown lifecycle
- Port binding and release
- Graceful shutdown performance
- No zombie processes

**Why**: Catches issues that unit/integration tests might miss (e.g., zombie processes, port not released).

---

## Implementation Roadmap

### Week 1: Stop the Bleeding

**Goal**: Prevent Mode 2 failures from going undetected

**Tasks**:
1. Create `smoke_test_comprehensive.sh` (1-2 hours)
   - Copy template from Section 4
   - Test Mode 1 server startup
   - Test Mode 2 TUI/daemon startup
   - Both must pass to succeed

2. Update CI/CD pipeline (1 hour)
   - Add smoke test as required step
   - Must pass before merging any PR

3. Add Mode 2 sanity tests (2 hours)
   - Basic check that `cco` command works
   - Daemon starts and listens
   - TUI connects successfully

**Why this first**: Smoke test is fastest way to catch regressions. Validates both modes in ~5 minutes.

### Week 2: Build Integration Coverage

**Goal**: Test both modes thoroughly

**Tasks**:
1. Create Mode 1 integration tests (3-4 hours)
   - Server startup on different ports (3000, 3001, 3100, etc)
   - All API endpoints respond
   - Configuration loading and override
   - Graceful shutdown performance
   - Port release verification

2. Create Mode 2 integration tests (3-4 hours)
   - TUI/daemon initialization sequence (critical!)
   - TUI connection to daemon
   - Dashboard data loading
   - Graceful shutdown of both
   - Port release after both exit

3. Update CI/CD with separate jobs (1 hour)
   - `test-mode-1` job runs Mode 1 tests
   - `test-mode-2` job runs Mode 2 tests
   - Both must pass

**Why**: Integration tests catch component interactions that unit tests miss.

### Week 3: Complete E2E Coverage

**Goal**: Real-world workflow validation

**Tasks**:
1. Create Mode 1 E2E test (2 hours)
   - Real binary execution
   - Complete startup-shutdown lifecycle
   - Port binding and release
   - Check for zombie processes

2. Create Mode 2 E2E test (2 hours)
   - Real binary execution
   - TUI and daemon startup
   - TUI-daemon communication
   - Both processes exit cleanly
   - No orphaned processes

3. Add performance benchmarks (1 hour)
   - Startup time < 2 seconds
   - Shutdown time < 2 seconds
   - Memory usage < 100MB

**Why**: E2E tests validate real-world scenarios that mocked tests might miss.

---

## Quick Implementation Checklist

### Immediate (This Week)

- [ ] Create `smoke_test_comprehensive.sh`
  - [ ] Copy template from Section 4 of main strategy
  - [ ] Update paths and port numbers
  - [ ] Test Mode 1: `cco run --port 3000`
  - [ ] Test Mode 2: `cco` (no args)
  - [ ] Verify both modes pass

- [ ] Update GitHub Actions (`.github/workflows/test.yml`)
  - [ ] Add smoke test as required job
  - [ ] Smoke test must pass before merge
  - [ ] Runs on all PRs and main branch pushes

- [ ] Test the smoke test locally
  - [ ] Run: `./tests/smoke_test_comprehensive.sh`
  - [ ] Verify all checks pass
  - [ ] Verify both Mode 1 and Mode 2 are tested

### Short Term (Next 2 Weeks)

- [ ] Create Mode 1 integration tests
  - [ ] File: `tests/integration/mode1_server_basic.rs`
  - [ ] Copy template from Section 2
  - [ ] Implement test functions for Mode 1 scenarios

- [ ] Create Mode 2 integration tests
  - [ ] File: `tests/integration/mode2_tui_daemon.rs`
  - [ ] Copy template from Section 2
  - [ ] Critical: Test daemon starts before TUI connects

- [ ] Update CI/CD
  - [ ] Separate `test-mode-1` and `test-mode-2` jobs
  - [ ] Both must pass before smoke test
  - [ ] Smoke test runs last as final check

- [ ] Verify integration tests run locally
  - [ ] Run: `cargo test --test '*mode1*'`
  - [ ] Run: `cargo test --test '*mode2*'`
  - [ ] All tests pass

### Long Term (After 3 Weeks)

- [ ] Create E2E workflow tests
  - [ ] Mode 1: `tests/e2e/mode1_comprehensive.sh`
  - [ ] Mode 2: `tests/e2e/mode2_comprehensive.sh`
  - [ ] Real binary execution (not mocked)

- [ ] Add performance benchmarks
  - [ ] Startup time validation
  - [ ] Shutdown time validation
  - [ ] Memory usage monitoring

- [ ] Acceptance tests
  - [ ] 5-minute stability test
  - [ ] Load testing (100+ concurrent)
  - [ ] Resource usage monitoring

---

## File Organization

Create this structure in `/Users/brent/git/cc-orchestra/cco/tests/`:

```
tests/
├── smoke_test_comprehensive.sh          ← Main smoke test (IMMEDIATE)
│
├── integration/
│   ├── mod.rs                          ← Integration test module
│   ├── mode1_server_basic.rs           ← Mode 1 tests (WEEK 2)
│   ├── mode1_server_endpoints.rs       ← Mode 1 API tests
│   ├── mode1_server_shutdown.rs        ← Mode 1 shutdown tests
│   ├── mode2_tui_daemon.rs             ← Mode 2 tests (WEEK 2)
│   ├── mode2_tui_connection.rs         ← Mode 2 connection tests
│   └── both_modes_stability.rs         ← Stress tests for both
│
├── e2e/
│   ├── mode1_comprehensive.sh          ← Mode 1 E2E (WEEK 3)
│   ├── mode2_comprehensive.sh          ← Mode 2 E2E (WEEK 3)
│   └── helpers.sh                      ← Shared helper functions
│
└── common/
    ├── test_helpers.rs                 ← Shared Rust utilities
    └── fixtures/                       ← Test data files
        ├── config.json
        ├── agents.json
        └── sample_project/
```

---

## Critical Tests (Must Have)

These tests are **non-negotiable** - they catch the most critical failures:

### Startup Tests
```
[ ] cco run --port 3000 starts successfully
[ ] cco (no args) TUI/daemon starts successfully
[ ] Daemon binds port 3000 BEFORE TUI tries to connect
[ ] TUI can connect to daemon within 5 seconds
```

### Shutdown Tests
```
[ ] Ctrl+C initiates shutdown (Mode 1)
[ ] Ctrl+C initiates shutdown (Mode 2)
[ ] Shutdown completes within 2 seconds
[ ] Port 3000 is free after shutdown (can bind again)
[ ] No zombie processes remain
```

### API Tests
```
[ ] GET /health returns 200 OK with valid JSON
[ ] GET /api/agents returns list of agents
[ ] POST /api/v1/chat accepts requests
[ ] All endpoints work in both Mode 1 and Mode 2
```

### Mode Separation Tests
```
[ ] Mode 1 (explicit server) works independent of Mode 2
[ ] Mode 2 (TUI/daemon) works independent of Mode 1
[ ] Tests explicitly document which mode they test
[ ] CI/CD runs separate test jobs for each mode
```

---

## Testing Both Modes: Example Pattern

Use this pattern in all tests to make mode-specific testing explicit:

```rust
// ✓ GOOD: Clear which mode is being tested
#[tokio::test]
async fn test_mode1_server_startup() {
    // Explicitly test Mode 1: cco run --port XXXX
    let server = start_test_server_mode1("127.0.0.1", 3000).await;
    // ... test code ...
}

#[tokio::test]
async fn test_mode2_tui_daemon_startup() {
    // Explicitly test Mode 2: cco (no args)
    let app = start_test_tui_daemon_mode2().await;
    // ... test code ...
}

// ✗ BAD: Ambiguous which mode is tested
#[tokio::test]
async fn test_server_startup() {
    // Which mode? Unclear!
    let server = start_server().await;
}
```

---

## CI/CD Pipeline Update

Update `.github/workflows/test.yml`:

```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  # New: Separate Mode 1 tests
  test-mode-1:
    runs-on: ubuntu-latest
    name: "Test Mode 1: Explicit Server"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: cargo test --lib mode1
      - run: cargo test --test '*mode1*'

  # New: Separate Mode 2 tests
  test-mode-2:
    runs-on: ubuntu-latest
    name: "Test Mode 2: TUI/Daemon"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: cargo test --lib mode2
      - run: cargo test --test '*mode2*'

  # New: Smoke test (both modes)
  smoke-test-both-modes:
    runs-on: ubuntu-latest
    name: "Smoke Test: Both Modes"
    needs: [test-mode-1, test-mode-2]  # Run after other tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: chmod +x tests/smoke_test_comprehensive.sh
      - run: ./tests/smoke_test_comprehensive.sh
```

---

## Validation Checklist

After implementing, verify:

### Smoke Test
- [ ] Runs in < 5 minutes
- [ ] Tests Mode 1: `cco run --port 3000`
- [ ] Tests Mode 2: `cco` (no args)
- [ ] All checks pass or all fail (no partial passes)
- [ ] Clear output showing which tests passed/failed

### Integration Tests
- [ ] Mode 1 tests isolated (don't require Mode 2)
- [ ] Mode 2 tests isolated (don't require Mode 1)
- [ ] Test names clearly indicate which mode: `test_mode1_*` / `test_mode2_*`
- [ ] All tests pass locally and in CI/CD

### CI/CD Pipeline
- [ ] Mode 1 tests run separately
- [ ] Mode 2 tests run separately
- [ ] Smoke test runs after both pass
- [ ] PR requires all three job groups to pass

### Coverage
- [ ] Mode 1 startup tested
- [ ] Mode 2 startup tested
- [ ] Daemon port binding tested (Mode 2)
- [ ] TUI-daemon connection tested (Mode 2)
- [ ] Graceful shutdown < 2s (both modes)
- [ ] Port release verified (both modes)
- [ ] No zombie processes (both modes)

---

## Example: Minimal Mode 2 Test

Here's a minimal Mode 2 integration test to get started:

```rust
// tests/integration/mode2_tui_daemon.rs

#[cfg(test)]
mod mode2_tests {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    use std::thread;

    /// Test Mode 2: cco (no args) - TUI/Daemon initialization
    ///
    /// This is the CRITICAL test that catches Mode 2 failures
    #[tokio::test]
    async fn test_mode2_tui_daemon_startup() {
        // Spawn the cco binary with NO arguments (Mode 2)
        let mut child = Command::new("cargo")
            .args(&["run", "--release"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn cco");

        // CRITICAL: Wait for daemon to be ready
        // Daemon MUST bind port 3000 BEFORE TUI tries to connect
        let start = Instant::now();
        let mut daemon_ready = false;

        while start.elapsed() < Duration::from_secs(10) {
            if can_connect_to_daemon().await {
                daemon_ready = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        assert!(daemon_ready, "Daemon failed to start and bind port 3000");

        // CRITICAL: Verify daemon is listening
        let health = reqwest::Client::new()
            .get("http://127.0.0.1:3000/health")
            .send()
            .await;

        assert!(health.is_ok(), "Daemon health endpoint not accessible");

        // Graceful shutdown
        child.kill().expect("Failed to kill process");
        let status = child.wait().expect("Failed to wait for exit");

        assert!(
            status.success() || status.code() == Some(130),
            "Exit code should be 0 or 130 (SIGINT): {:?}",
            status.code()
        );

        // Verify port released
        tokio::time::sleep(Duration::from_millis(500)).await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await;
        assert!(
            listener.is_ok(),
            "Port 3000 still in use after shutdown"
        );
    }

    async fn can_connect_to_daemon() -> bool {
        reqwest::Client::new()
            .get("http://127.0.0.1:3000/health")
            .timeout(Duration::from_secs(1))
            .send()
            .await
            .is_ok()
    }
}
```

Run it:
```bash
cargo test --test '*' test_mode2_tui_daemon_startup -- --nocapture
```

---

## Key Metrics to Track

After implementing tests, track these metrics:

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Mode 1 tests passing | 100% | `cargo test --test '*mode1*'` |
| Mode 2 tests passing | 100% | `cargo test --test '*mode2*'` |
| Smoke test time | < 5 min | Time `smoke_test_comprehensive.sh` |
| Test coverage | > 80% | `cargo tarpaulin` or `cargo llvm-cov` |
| CI/CD pass rate | 100% | Check GitHub Actions |
| Zero mode-specific failures | ∞ days | Track bug reports |

---

## Success Indicators

You've successfully implemented the test strategy when:

1. **Smoke test runs both modes** (< 5 minutes)
   - Mode 1: `cco run --port 3000` passes
   - Mode 2: `cco (no args)` passes
   - Both modes must pass for CI/CD approval

2. **CI/CD has separate Mode 1 and Mode 2 jobs**
   - Mode 1 tests isolated
   - Mode 2 tests isolated
   - Smoke test validates both together

3. **Mode 2 failures are caught immediately**
   - Any daemon startup failure detected
   - Any TUI-daemon connection failure detected
   - Any graceful shutdown failure detected

4. **Tests are documented**
   - Test names indicate which mode: `mode1_*` or `mode2_*`
   - Each test file documents its mode at the top
   - PR template includes mode testing checklist

5. **Zero regressions**
   - Mode 1 never breaks (Mode 1 tests always pass)
   - Mode 2 never breaks (Mode 2 tests always pass)
   - Both modes always work (smoke test always passes)

---

## Common Mistakes to Avoid

❌ **DON'T**: Single smoke test that only validates Mode 1
❌ **DON'T**: Mix Mode 1 and Mode 2 tests in one function
❌ **DON'T**: Assume Mode 1 tests cover Mode 2
❌ **DON'T**: Skip testing TUI-daemon communication
❌ **DON'T**: Ignore shutdown timing (< 2 seconds required)
❌ **DON'T**: Forget to verify port is released
❌ **DON'T**: Use generic test names that don't indicate mode

✅ **DO**: Create separate Mode 1 and Mode 2 test files
✅ **DO**: Name tests explicitly: `test_mode1_*` and `test_mode2_*`
✅ **DO**: Test both modes in smoke test
✅ **DO**: Test TUI-daemon initialization sequence
✅ **DO**: Verify graceful shutdown < 2 seconds
✅ **DO**: Verify port is released immediately
✅ **DO**: Document which mode each test covers

---

## Support Resources

- **Main Strategy**: See `COMPREHENSIVE_TEST_STRATEGY.md` in this directory
- **Code Examples**: Section 2 of main strategy has test code templates
- **CI/CD Template**: Section 2 shows GitHub Actions configuration
- **Test Scripts**: Section 4 has complete smoke test script
- **Quick Checklist**: Section 3 has critical tests list

---

**Status**: Ready to implement
**Estimated effort**: 2-3 weeks for complete coverage
**Expected outcome**: Zero mode-specific test failures in future
