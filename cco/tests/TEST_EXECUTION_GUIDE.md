// Test Execution Guide for FUSE VFS and CLI Enhancements

# CCO Test Execution Guide

**Purpose:** Step-by-step guide for running the comprehensive test suite
**Audience:** QA Engineers, Developers, CI/CD Systems
**Prerequisites:** Rust 1.70+, FUSE libraries installed, CCO compiled

---

## Quick Start

### Run All Tests
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo test --all
```

### Run with Verbose Output
```bash
cargo test --all -- --nocapture --test-threads=1
```

### Run Specific Test File
```bash
cargo test --test fuse_vfs_phase1_tests
```

---

## Test Suite Organization

### 1. FUSE VFS Tests (Phases 1-4)

#### Phase 1: Mount/Unmount Tests
```bash
# Run all Phase 1 tests
cargo test --test fuse_vfs_phase1_tests

# Run specific unit test
cargo test --test fuse_vfs_phase1_tests test_mount_creates_directory

# Run all mount tests
cargo test --test fuse_vfs_phase1_tests mount

# Run all error path tests
cargo test --test fuse_vfs_phase1_tests error
```

**Expected Results:**
- All tests should pass
- No errors or warnings
- Test execution time: <5 seconds

#### Phase 2: Encryption & Sealing Tests
```bash
# Run all Phase 2 tests
cargo test --test fuse_vfs_phase2_tests

# Run encryption unit tests
cargo test --test fuse_vfs_phase2_tests aes256gcm

# Run sealing integration tests
cargo test --test fuse_vfs_phase2_tests seal

# Run security tests
cargo test --test fuse_vfs_phase2_tests key_derivation
```

**Expected Results:**
- All encryption tests pass
- Performance: Seal/unseal <50ms each
- No timing attack vulnerabilities

#### Phase 3: Anti-Debugging Tests
```bash
# Run all Phase 3 tests
cargo test --test fuse_vfs_phase3_tests

# Run debugger detection tests
cargo test --test fuse_vfs_phase3_tests ptrace

# Run memory protection tests
cargo test --test fuse_vfs_phase3_tests guard_pages

# Skip manual security tests
cargo test --test fuse_vfs_phase3_tests -- --skip ignore
```

**Expected Results:**
- Automated tests pass
- Manual tests documented for security audit
- No false positives in normal execution

#### Phase 4: Monitoring & Health Tests
```bash
# Run all Phase 4 tests
cargo test --test fuse_vfs_phase4_tests

# Run health endpoint tests
cargo test --test fuse_vfs_phase4_tests health_endpoint

# Run metrics tests
cargo test --test fuse_vfs_phase4_tests prometheus_metrics

# Run performance tests
cargo test --test fuse_vfs_phase4_tests latency
```

**Expected Results:**
- All health checks pass
- Metrics exported correctly
- Performance targets met:
  - File read: <2ms
  - Unsealing: <5ms
  - 100 concurrent reads: <1s
  - 1000+ reads/second

---

### 2. CLI Launcher Tests

```bash
# Run all CLI tests
cargo test --test cli_launcher_tests

# Run Phase 1: Launcher module tests
cargo test --test cli_launcher_tests ensure_daemon

# Run Phase 2: TUI subcommand tests
cargo test --test cli_launcher_tests launch_tui

# Run Phase 3: CLI routing tests
cargo test --test cli_launcher_tests cli_parsing

# Run Phase 4: Full test suite
cargo test --test cli_launcher_tests e2e
```

**Expected Results:**
- All routing tests pass
- Launcher startup: <3 seconds
- VFS health check: <100ms
- CLI parsing: <50ms

---

## Coverage Analysis

### Install Coverage Tools
```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Or use llvm-cov (alternative)
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

### Generate Coverage Report
```bash
# Using tarpaulin
cargo tarpaulin \
  --out Html \
  --output-dir coverage \
  --exclude-files "tests/*" \
  --target-dir target/tarpaulin

# View report
open coverage/index.html
```

### Coverage by Component
```bash
# FUSE VFS Phase 1 coverage
cargo tarpaulin --test fuse_vfs_phase1_tests --out Html

# FUSE VFS Phase 2 coverage
cargo tarpaulin --test fuse_vfs_phase2_tests --out Html

# CLI Launcher coverage
cargo tarpaulin --test cli_launcher_tests --out Html
```

**Coverage Targets:**
- Overall: 90%+
- FUSE VFS: 90%+ per phase
- CLI Launcher: 90%+
- Critical paths: 100%

---

## Performance Benchmarking

### Install Benchmark Tools
```bash
cargo install cargo-criterion
```

### Run Performance Tests
```bash
# File read latency
cargo test --test fuse_vfs_phase4_tests \
  test_file_read_latency_less_than_2ms \
  -- --ignored --nocapture

# Unsealing latency
cargo test --test fuse_vfs_phase4_tests \
  test_unsealing_latency_less_than_5ms \
  -- --ignored --nocapture

# Concurrent reads
cargo test --test fuse_vfs_phase4_tests \
  test_100_concurrent_reads \
  -- --ignored --nocapture

# Throughput
cargo test --test fuse_vfs_phase4_tests \
  test_1000_file_reads_per_second \
  -- --ignored --nocapture

# Launcher startup
cargo test --test cli_launcher_tests \
  test_launcher_startup_under_3_seconds \
  -- --ignored --nocapture
```

**Performance Targets:**
| Metric | Target | Measured |
|--------|--------|----------|
| VFS mount | <100ms | TBD |
| VFS unmount | <50ms | TBD |
| File read (small) | <1ms | TBD |
| File read (large) | <10ms | TBD |
| Seal agent file | <50ms | TBD |
| Unseal agent file | <50ms | TBD |
| Launcher startup | <3s | TBD |
| VFS health check | <100ms | TBD |
| CLI parsing | <50ms | TBD |

---

## End-to-End Testing

### Prerequisites
```bash
# Stop any running daemon
cco daemon stop

# Clean test environment
rm -rf /var/run/cco
rm -rf ~/.cco/test_*
```

### E2E Test Scenarios

#### Scenario 1: Fresh Installation
```bash
# 1. Build CCO
cargo build --release

# 2. Run E2E test
cargo test --test cli_launcher_tests \
  test_e2e_clean_environment_daemon_not_running \
  -- --nocapture

# Expected:
# - Daemon starts automatically
# - VFS mounts at /var/run/cco/
# - Health check passes
# - Claude Code launches (mock)
```

#### Scenario 2: Daemon Crash Recovery
```bash
# 1. Start daemon
cco daemon start

# 2. Crash daemon (kill -9)
pkill -9 cco

# 3. Run E2E test
cargo test --test cli_launcher_tests \
  test_e2e_daemon_crash_recovery \
  -- --nocapture

# Expected:
# - Detects daemon crash
# - Restarts daemon
# - Remounts VFS
# - Claude Code launches
```

#### Scenario 3: Multiple Simultaneous Sessions
```bash
# Run E2E test
cargo test --test cli_launcher_tests \
  test_e2e_multiple_simultaneous_sessions \
  -- --nocapture

# Expected:
# - Multiple cco instances can run
# - No conflicts or errors
# - Shared VFS access works
```

---

## Integration Testing

### Prerequisites for Integration Tests
```bash
# Ensure daemon is running
cco daemon start

# Verify VFS is mounted
ls /var/run/cco/

# Check health
cat /var/run/cco/health
```

### Run Integration Tests
```bash
# FUSE VFS integration tests
cargo test --test fuse_vfs_phase1_tests daemon_lifecycle
cargo test --test fuse_vfs_phase2_tests seal

# CLI integration tests
cargo test --test cli_launcher_tests full_workflow
cargo test --test cli_launcher_tests tui_daemon_auto_start
```

---

## Manual Testing (Security Audits)

### Manual Test 1: Debugger Detection
```bash
# Start daemon
cco daemon start

# Attach debugger
lldb -p $(pgrep cco)

# Expected: Daemon detects debugger and logs warning
# Check logs:
tail -f ~/.cco/daemon.log
```

### Manual Test 2: Memory Dump Analysis
```bash
# Start daemon
cco daemon start

# Create memory dump
gcore $(pgrep cco)

# Search for plaintext agent definitions
strings core.$(pgrep cco) | grep -i "agent"

# Expected: No plaintext agent definitions visible
```

### Manual Test 3: Cross-Machine Unsealing
```bash
# On Machine 1:
cco daemon start
cp /var/run/cco/agents.sealed /tmp/agents.sealed

# Transfer /tmp/agents.sealed to Machine 2

# On Machine 2:
cco daemon start
# Try to unseal Machine 1's file
cco unseal /tmp/agents.sealed

# Expected: Unsealing fails (machine-specific keys)
```

### Manual Test 4: VFS Path Traversal
```bash
# Try to access files outside VFS
cat /var/run/cco/../../../etc/passwd

# Expected: Access denied or ENOENT
```

---

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: CCO Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable, 1.70.0]

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true

      - name: Install FUSE (Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get install -y fuse3 libfuse3-dev

      - name: Install FUSE (macOS)
        if: runner.os == 'macOS'
        run: brew install macfuse

      - name: Build
        run: cargo build --all --verbose

      - name: Run Unit Tests
        run: cargo test --lib --verbose

      - name: Run Integration Tests
        run: cargo test --test '*' --verbose

      - name: Coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --exclude-files 'tests/*'

      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

      - name: Performance Tests
        run: |
          cargo test test_file_read_latency_less_than_2ms -- --ignored
          cargo test test_launcher_startup_under_3_seconds -- --ignored
```

---

## Troubleshooting

### Test Failures

#### "VFS not mounted" errors
```bash
# Check if daemon is running
cco daemon status

# Restart daemon
cco daemon restart

# Verify VFS mount
ls /var/run/cco/
```

#### "Permission denied" errors
```bash
# Check permissions
ls -la /var/run/cco/

# On macOS, approve FUSE in System Preferences
# On Linux, check fuse group membership
groups
```

#### "Timeout" errors
```bash
# Increase test timeout
cargo test -- --test-threads=1 --nocapture

# Check system load
top
```

#### "Encryption key mismatch" errors
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Restart daemon
cco daemon restart
```

### Performance Issues

#### Slow test execution
```bash
# Run tests in parallel (default)
cargo test

# Run tests sequentially (for debugging)
cargo test -- --test-threads=1

# Skip slow integration tests
cargo test --lib
```

#### Coverage generation timeout
```bash
# Increase timeout
cargo tarpaulin --timeout 600

# Run coverage for specific component
cargo tarpaulin --test fuse_vfs_phase1_tests
```

---

## Test Maintenance

### Adding New Tests
```rust
// Add to existing test file
#[tokio::test]
async fn test_new_feature() {
    // Arrange
    let setup = test_setup();

    // Act
    let result = feature_under_test().await;

    // Assert
    assert!(result.is_ok());
}
```

### Updating Test Expectations
```bash
# Re-run tests to get new baselines
cargo test --test fuse_vfs_phase4_tests -- --nocapture

# Update performance targets in test code
```

### Removing Obsolete Tests
```bash
# Comment out deprecated tests
# Mark with #[ignore] or #[cfg(feature = "deprecated")]

# Remove from coverage calculations
cargo tarpaulin --exclude-files 'tests/deprecated/*'
```

---

## Success Criteria Checklist

### FUSE VFS
- [ ] All Phase 1 tests passing (20+ tests)
- [ ] All Phase 2 tests passing (25+ tests)
- [ ] All Phase 3 tests passing (20+ tests)
- [ ] All Phase 4 tests passing (25+ tests)
- [ ] Overall coverage 90%+
- [ ] All performance targets met
- [ ] All security tests documented

### CLI Launcher
- [ ] All Phase 1 tests passing (10+ tests)
- [ ] All Phase 2 tests passing (7+ tests)
- [ ] All Phase 3 tests passing (13+ tests)
- [ ] All Phase 4 tests passing (11+ tests)
- [ ] Overall coverage 90%+
- [ ] Launcher startup <3s
- [ ] All backward compatibility tests passing

### Integration
- [ ] Daemon lifecycle tests passing
- [ ] VFS + CLI integration working
- [ ] E2E workflows validated
- [ ] Manual security audit complete

### CI/CD
- [ ] GitHub Actions workflow configured
- [ ] Automated tests run on every PR
- [ ] Coverage reports generated
- [ ] Performance benchmarks tracked

---

## Reporting

### Generate Test Report
```bash
# Run all tests with JSON output
cargo test --all -- -Z unstable-options --format json > test-results.json

# Generate HTML report
# (requires cargo-test-report)
cargo install cargo-test-report
cargo test-report --json test-results.json --html test-report.html

# View report
open test-report.html
```

### Coverage Report
```bash
# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage
open coverage/index.html

# Coverage by file
cargo tarpaulin --out Json | jq '.files'
```

### Performance Report
```bash
# Run performance tests
cargo test --test fuse_vfs_phase4_tests latency -- --ignored --nocapture > perf-results.txt

# Parse results
cat perf-results.txt | grep "Average latency"
```

---

**Last Updated:** 2025-11-17
**Status:** Test Execution Guide Complete
**Next Review:** After Phase 1 implementation
