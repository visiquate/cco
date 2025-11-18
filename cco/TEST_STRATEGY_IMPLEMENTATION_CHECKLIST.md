# Test Strategy Implementation Checklist

**Purpose**: Day-by-day implementation guide for comprehensive testing framework
**Target**: 2-3 weeks to full coverage
**Status**: Ready to implement

---

## Week 1: Stop the Bleeding (Create Smoke Test)

### Day 1: Create Smoke Test Script

**File**: `tests/smoke_test_comprehensive.sh`
**Time**: 1-2 hours
**Critical**: Yes - blocks all other work if tests fail

- [ ] Create `/Users/brent/git/cc-orchestra/cco/tests/smoke_test_comprehensive.sh`
  - [ ] Copy template from `COMPREHENSIVE_TEST_STRATEGY.md` Section 4
  - [ ] Update absolute paths:
    - [ ] `/Users/brent/git/cc-orchestra/cco` as base directory
    - [ ] Log files in `/tmp/`
  - [ ] Update port numbers (use 3000, 3001, 3002 for tests)
  - [ ] Add Mode 1 tests (explicit server)
    - [ ] Server starts: `timeout 8 cargo run --release -- run --port 3000`
    - [ ] Health endpoint: `curl http://127.0.0.1:3000/health`
    - [ ] API endpoints: `curl http://127.0.0.1:3000/api/agents`
    - [ ] Shutdown timing: Measure Ctrl+C to exit < 2s
    - [ ] Port release: Verify no `lsof -i :3000` after shutdown
  - [ ] Add Mode 2 tests (TUI/daemon - CRITICAL!)
    - [ ] Start with no args: `timeout 10 cargo run --release`
    - [ ] Wait for daemon ready (poll health endpoint, max 10s)
    - [ ] Verify health endpoint accessible
    - [ ] Verify API endpoints accessible
    - [ ] Test graceful shutdown
    - [ ] Verify port released

- [ ] Make script executable
  ```bash
  chmod +x tests/smoke_test_comprehensive.sh
  ```

- [ ] Test locally
  ```bash
  cd /Users/brent/git/cc-orchestra/cco
  ./tests/smoke_test_comprehensive.sh
  ```

- [ ] Verify output
  - [ ] Shows Mode 1 tests running
  - [ ] Shows Mode 2 tests running
  - [ ] All tests pass or all fail (no partial)
  - [ ] Clear PASS/FAIL summary at end

### Day 2: Integrate Smoke Test into CI/CD

**File**: `.github/workflows/test.yml`
**Time**: 1 hour

- [ ] Update `.github/workflows/test.yml`
  - [ ] Add new job `smoke-test-both-modes`:
    ```yaml
    smoke-test-both-modes:
      runs-on: ubuntu-latest
      name: "Smoke Test: Both Modes"
      needs: [build]  # Depends on successful build
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo build --release
        - run: chmod +x tests/smoke_test_comprehensive.sh
        - run: ./tests/smoke_test_comprehensive.sh
    ```
  - [ ] Make smoke test a required check for PRs

- [ ] Commit changes
  ```bash
  git add .github/workflows/test.yml
  git commit -m "test: add smoke test for both execution modes"
  ```

- [ ] Verify in GitHub
  - [ ] Workflow runs on new PR
  - [ ] Smoke test job appears in checks
  - [ ] Smoke test passes

### Day 3: Verify Smoke Test Works

**Time**: 30 minutes

- [ ] Run smoke test locally 3 times
  ```bash
  for i in 1 2 3; do
    echo "Run $i..."
    tests/smoke_test_comprehensive.sh || exit 1
  done
  ```

- [ ] Verify all checks pass
  - [ ] Mode 1 startup test passes
  - [ ] Mode 1 port binding test passes
  - [ ] Mode 1 API endpoint tests pass
  - [ ] Mode 1 shutdown test passes
  - [ ] Mode 2 startup test passes
  - [ ] Mode 2 API endpoint tests pass
  - [ ] Mode 2 shutdown test passes
  - [ ] No zombie processes remain
  - [ ] Port 3000 released after each mode

- [ ] Check smoke test is in PR checks
  - [ ] Create test PR
  - [ ] Verify smoke test runs
  - [ ] Verify smoke test passes
  - [ ] Merge PR

**Week 1 Goal**: Smoke test prevents future Mode 2 failures!

---

## Week 2: Integration Coverage

### Day 4: Create Mode 1 Integration Tests

**File**: `tests/integration/mode1_server_basic.rs`
**Time**: 2-3 hours

- [ ] Create `/Users/brent/git/cc-orchestra/cco/tests/integration/`
  - [ ] Create `mode1_server_basic.rs`
  - [ ] Create `mod.rs` with module declarations

- [ ] Implement Mode 1 tests in `mode1_server_basic.rs`
  - [ ] Test: Server starts on specified port
    ```rust
    #[tokio::test]
    async fn test_mode1_server_startup_on_custom_port() {
        // Start server on port 3000
        // Verify process running
        // Verify health endpoint responds
    }
    ```

  - [ ] Test: API endpoints accessible
    ```rust
    #[tokio::test]
    async fn test_mode1_api_endpoints_accessible() {
        // Start server
        // Test /health endpoint
        // Test /api/agents endpoint
        // Test /api/v1/chat endpoint
    }
    ```

  - [ ] Test: Graceful shutdown < 2 seconds
    ```rust
    #[tokio::test]
    async fn test_mode1_graceful_shutdown_timing() {
        // Start server
        // Measure Ctrl+C to exit time
        // Assert time < 2 seconds
        // Verify port released
    }
    ```

  - [ ] Test: Port binding on different ports
    ```rust
    #[tokio::test]
    async fn test_mode1_port_binding_variations() {
        // Test ports 3000, 3001, 3100
        // Verify each binds successfully
    }
    ```

- [ ] Run tests
  ```bash
  cargo test --test '*mode1*'
  ```

- [ ] Verify all tests pass
  - [ ] Server startup test passes
  - [ ] API endpoint test passes
  - [ ] Shutdown timing test passes
  - [ ] Port binding test passes

### Day 5: Create Mode 2 Integration Tests (CRITICAL)

**File**: `tests/integration/mode2_tui_daemon.rs`
**Time**: 2-3 hours

**This is the MOST CRITICAL test file - catches daemon startup failures**

- [ ] Create `mode2_tui_daemon.rs`

- [ ] Implement Mode 2 tests
  - [ ] Test: TUI/daemon initialization sequence (CRITICAL!)
    ```rust
    #[tokio::test]
    async fn test_mode2_tui_daemon_initialization_sequence() {
        // Start with no args: cargo run --release
        // Wait for daemon to bind port 3000 (max 10s)
        // Verify daemon is listening
        // Verify TUI can connect
        // Graceful shutdown
        // Verify port released
    }
    ```
    **This is the TEST that should have caught the Mode 2 failure!**

  - [ ] Test: Daemon binds BEFORE TUI connects
    ```rust
    #[tokio::test]
    async fn test_mode2_daemon_ready_before_tui() {
        // Start process
        // Poll health endpoint every 100ms
        // Assert daemon ready within 5s
        // Assert TUI can connect
    }
    ```

  - [ ] Test: TUI-daemon communication
    ```rust
    #[tokio::test]
    async fn test_mode2_tui_daemon_communication() {
        // Start TUI/daemon
        // Access /api/agents from daemon
        // Verify dashboard data loads
    }
    ```

  - [ ] Test: Graceful shutdown both processes
    ```rust
    #[tokio::test]
    async fn test_mode2_graceful_shutdown_both_processes() {
        // Start TUI/daemon
        // Send Ctrl+C
        // Measure shutdown time
        // Assert time < 3 seconds
        // Verify port released
        // Verify no zombie processes
    }
    ```

- [ ] Run tests
  ```bash
  cargo test --test '*mode2*'
  ```

- [ ] Verify all tests pass
  - [ ] Initialization sequence test passes
  - [ ] Daemon ready test passes
  - [ ] Communication test passes
  - [ ] Shutdown test passes

### Day 6: Update CI/CD Pipeline

**File**: `.github/workflows/test.yml`
**Time**: 1 hour

- [ ] Update workflow with separate Mode 1 and Mode 2 jobs
  ```yaml
  jobs:
    test-mode-1:
      runs-on: ubuntu-latest
      name: "Test Mode 1: Explicit Server"
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo build --release
        - run: cargo test --test '*mode1*'

    test-mode-2:
      runs-on: ubuntu-latest
      name: "Test Mode 2: TUI/Daemon"
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo build --release
        - run: cargo test --test '*mode2*'

    smoke-test-both-modes:
      runs-on: ubuntu-latest
      name: "Smoke Test: Both Modes"
      needs: [test-mode-1, test-mode-2]
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo build --release
        - run: ./tests/smoke_test_comprehensive.sh
  ```

- [ ] Verify all three jobs appear in PR checks
  - [ ] Test Mode 1 job
  - [ ] Test Mode 2 job
  - [ ] Smoke test job

- [ ] All jobs must pass for PR approval

### Day 7: Verification & Cleanup

**Time**: 1 hour

- [ ] Run all Mode 1 tests locally
  ```bash
  cargo test --test '*mode1*'
  ```

- [ ] Run all Mode 2 tests locally
  ```bash
  cargo test --test '*mode2*'
  ```

- [ ] Run smoke test
  ```bash
  ./tests/smoke_test_comprehensive.sh
  ```

- [ ] Create verification PR
  - [ ] All 3 jobs pass
  - [ ] No warnings or errors
  - [ ] Merge PR

**Week 2 Goal**: Both modes have comprehensive integration test coverage!

---

## Week 3: E2E & Performance Tests

### Day 8: Create Mode 1 E2E Tests

**File**: `tests/e2e/mode1_comprehensive.sh`
**Time**: 2 hours

- [ ] Create `/Users/brent/git/cc-orchestra/cco/tests/e2e/`

- [ ] Create `mode1_comprehensive.sh`
  - [ ] Test 1: Server startup and health
    - [ ] Start: `cargo run --release -- run --port 3000`
    - [ ] Verify: Health endpoint responds
    - [ ] Verify: Dashboard HTML served
  - [ ] Test 2: Port binding variations
    - [ ] Test ports 3000, 3001, 3100
    - [ ] Verify each starts successfully
  - [ ] Test 3: Graceful shutdown
    - [ ] Start server
    - [ ] Send Ctrl+C
    - [ ] Measure shutdown time
    - [ ] Verify time < 2 seconds
  - [ ] Test 4: Port release
    - [ ] After shutdown, verify can bind again

- [ ] Make executable
  ```bash
  chmod +x tests/e2e/mode1_comprehensive.sh
  ```

- [ ] Run test
  ```bash
  ./tests/e2e/mode1_comprehensive.sh
  ```

### Day 9: Create Mode 2 E2E Tests

**File**: `tests/e2e/mode2_comprehensive.sh`
**Time**: 2 hours

- [ ] Create `mode2_comprehensive.sh`
  - [ ] Test 1: TUI/daemon startup
    - [ ] Start: `cargo run --release` (no args)
    - [ ] Wait for daemon ready (max 10s)
    - [ ] Verify health endpoint
  - [ ] Test 2: Daemon listening
    - [ ] Verify port 3000 bound
    - [ ] Verify TUI process running
  - [ ] Test 3: Dashboard data loading
    - [ ] Verify /api/agents accessible
    - [ ] Verify returns valid JSON
  - [ ] Test 4: Graceful shutdown both
    - [ ] Send Ctrl+C
    - [ ] Measure shutdown time
    - [ ] Verify time < 3 seconds
  - [ ] Test 5: Port release
    - [ ] After shutdown, verify can bind

- [ ] Make executable
  ```bash
  chmod +x tests/e2e/mode2_comprehensive.sh
  ```

- [ ] Run test
  ```bash
  ./tests/e2e/mode2_comprehensive.sh
  ```

### Day 10: Performance Benchmarks

**Time**: 2 hours

- [ ] Create acceptance tests for performance
  - [ ] Startup time benchmark
    - [ ] Mode 1: < 2 seconds
    - [ ] Mode 2: < 5 seconds
  - [ ] Shutdown time benchmark
    - [ ] Mode 1: < 2 seconds
    - [ ] Mode 2: < 3 seconds
  - [ ] Memory usage
    - [ ] Both modes: < 100MB
  - [ ] Responsiveness
    - [ ] Health endpoint: < 100ms
    - [ ] API endpoints: < 500ms

- [ ] Create stability test
  - [ ] Run for 5 minutes
  - [ ] Make request every second
  - [ ] Count errors (should be 0)
  - [ ] Monitor memory (should stay constant)

- [ ] Create load test
  - [ ] 100 concurrent requests
  - [ ] Measure response times
  - [ ] Count errors (should be 0)
  - [ ] Verify server remains responsive

### Day 11: Documentation & PR Template

**Time**: 1 hour

- [ ] Create PR template with mode testing checklist
  - [ ] Create `.github/PULL_REQUEST_TEMPLATE.md`
  - [ ] Add testing requirements section:
    ```markdown
    ## Testing

    - [ ] Mode 1 tests pass: `cargo test --test '*mode1*'`
    - [ ] Mode 2 tests pass: `cargo test --test '*mode2*'`
    - [ ] Smoke test passes: `./tests/smoke_test_comprehensive.sh`
    - [ ] Manual test Mode 1: `cco run --port 3000`
    - [ ] Manual test Mode 2: `cco` (no args)

    ## Mode-Specific Questions

    - [ ] Does this change affect Mode 1 (explicit server)?
    - [ ] Does this change affect Mode 2 (TUI/daemon)?
    - [ ] Does this change affect daemon startup?
    - [ ] Does this change affect graceful shutdown?
    ```

- [ ] Update README with test instructions
  - [ ] Run all tests: `cargo test --lib && cargo test --test '*'`
  - [ ] Run smoke test: `./tests/smoke_test_comprehensive.sh`
  - [ ] Run Mode 1: `cargo run --release -- run --port 3000`
  - [ ] Run Mode 2: `cargo run --release`

### Day 14: Final Verification

**Time**: 1 hour

- [ ] Run complete test suite
  ```bash
  cargo test --lib                        # Unit tests
  cargo test --test '*'                   # Integration tests
  ./tests/smoke_test_comprehensive.sh     # Smoke test
  ./tests/e2e/mode1_comprehensive.sh      # Mode 1 E2E
  ./tests/e2e/mode2_comprehensive.sh      # Mode 2 E2E
  ```

- [ ] Verify all tests pass
  - [ ] All unit tests pass
  - [ ] All Mode 1 integration tests pass
  - [ ] All Mode 2 integration tests pass
  - [ ] Smoke test passes (both modes)
  - [ ] Mode 1 E2E passes
  - [ ] Mode 2 E2E passes

- [ ] Verify CI/CD pipeline
  - [ ] Create PR with test changes
  - [ ] Verify all 3 jobs run (Mode 1, Mode 2, Smoke)
  - [ ] All jobs pass
  - [ ] Merge PR

**Week 3 Goal**: Complete E2E coverage validated!

---

## Post-Implementation (Ongoing)

### Code Review Checklist

Add to every code review:

- [ ] Tests explicitly indicate which mode: `test_mode1_*` or `test_mode2_*`
- [ ] Mode 1 tests still pass: `cargo test --test '*mode1*'`
- [ ] Mode 2 tests still pass: `cargo test --test '*mode2*'`
- [ ] Smoke test still passes: `./tests/smoke_test_comprehensive.sh`
- [ ] No hardcoded mode assumptions
- [ ] All changes tested in BOTH modes

### Regression Testing

After any major change:

```bash
# Unit tests (fast)
cargo test --lib                          # < 1 min

# Integration tests
cargo test --test '*mode1*'               # < 2 min
cargo test --test '*mode2*'               # < 2 min

# E2E tests (slow)
./tests/e2e/mode1_comprehensive.sh        # < 1 min
./tests/e2e/mode2_comprehensive.sh        # < 1 min

# Smoke test
./tests/smoke_test_comprehensive.sh       # < 2 min

# Total: ~10 minutes
```

### Metrics to Track

Track these metrics monthly:

| Metric | Baseline | Target |
|--------|----------|--------|
| Mode 1 tests passing | 100% | 100% |
| Mode 2 tests passing | TBD | 100% |
| Smoke test time | TBD | < 5 min |
| PR approval rate | TBD | 100% |
| Post-merge failures | TBD | 0% |
| Mode-specific bugs | Many | 0 |

---

## Success Checklist

### Week 1 Complete
- [ ] Smoke test created and passing
- [ ] Smoke test added to CI/CD
- [ ] Both Mode 1 and Mode 2 validated in smoke test
- [ ] No blind spots in execution mode coverage

### Week 2 Complete
- [ ] Mode 1 integration tests created and passing
- [ ] Mode 2 integration tests created and passing (CRITICAL)
- [ ] CI/CD has separate jobs for Mode 1 and Mode 2
- [ ] All integration tests pass locally and in CI/CD

### Week 3 Complete
- [ ] Mode 1 E2E tests created and passing
- [ ] Mode 2 E2E tests created and passing
- [ ] Performance benchmarks established
- [ ] All tests passing in all environments

### Final Validation
- [ ] Zero mode-specific test failures
- [ ] New code requires Mode 1 AND Mode 2 testing
- [ ] PR template includes mode testing checklist
- [ ] Team understands dual-mode testing approach

---

## Quick Reference Commands

```bash
# Build
cargo build --release

# All tests
cargo test --lib && cargo test --test '*'

# Mode 1 only
cargo test --test '*mode1*'

# Mode 2 only
cargo test --test '*mode2*'

# Smoke test
./tests/smoke_test_comprehensive.sh

# E2E Mode 1
./tests/e2e/mode1_comprehensive.sh

# E2E Mode 2
./tests/e2e/mode2_comprehensive.sh

# Manual test Mode 1
cargo run --release -- run --port 3000

# Manual test Mode 2
cargo run --release
```

---

## Support Files

- **Main Strategy**: `COMPREHENSIVE_TEST_STRATEGY.md`
- **Quick Start**: `TEST_STRATEGY_QUICK_START.md`
- **Visual Summary**: `TEST_STRATEGY_VISUAL_SUMMARY.md`
- **This Checklist**: `TEST_STRATEGY_IMPLEMENTATION_CHECKLIST.md`

---

**Status**: Ready to implement
**Total Effort**: 2-3 weeks
**Expected Outcome**: Zero future mode-specific test failures
**Owner**: QA Engineer / Development Team
