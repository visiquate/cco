# Hooks Infrastructure Test Specification

## Overview

This document specifies comprehensive tests for Phase 1 Hook Infrastructure. The hooks system enables lifecycle management and extensibility for the CCO daemon.

## Architecture Under Test

The hooks infrastructure consists of three core modules:

1. **types.rs** - Hook data structures and configuration
2. **registry.rs** - Thread-safe hook registration and retrieval
3. **executor.rs** - Hook execution with timeout, retry, and panic recovery

## Test Coverage Requirements

- **Target Coverage**: 95%+ for hooks modules
- **Total Test Cases**: 40+ test cases
- **Test Types**: Unit (30+), Integration (10+)
- **Safety Focus**: Thread safety, panic recovery, timeout enforcement

## 1. Unit Tests for types.rs

### 1.1 HookPayload Tests (5 tests)

```rust
#[test]
fn test_hook_payload_construction()
// Verify HookPayload can be constructed with command and context

#[test]
fn test_hook_payload_serialization()
// Verify HookPayload serializes/deserializes correctly (serde)

#[test]
fn test_hook_payload_with_empty_context()
// Verify HookPayload handles empty context HashMap

#[test]
fn test_hook_payload_with_complex_context()
// Verify HookPayload handles nested JSON in context values

#[test]
fn test_hook_payload_command_types()
// Verify all command variants work: "start", "stop", "status", custom
```

### 1.2 HookType Tests (3 tests)

```rust
#[test]
fn test_hook_type_variants()
// Verify all HookType enum variants: PreStart, PostStart, PreStop, PostStop, Custom

#[test]
fn test_hook_type_display()
// Verify HookType Display trait for logging

#[test]
fn test_hook_type_equality()
// Verify HookType PartialEq implementation
```

### 1.3 HookConfig Tests (8 tests)

```rust
#[test]
fn test_hook_config_default_values()
// Verify HookConfig::default() has sensible defaults:
// - timeout: Duration::from_secs(5)
// - retries: 2
// - enabled: true

#[test]
fn test_hook_config_from_json()
// Parse HookConfig from JSON:
// {"timeout_secs": 10, "retries": 3, "enabled": true}

#[test]
fn test_hook_config_from_toml()
// Parse HookConfig from TOML:
// timeout_secs = 10
// retries = 3
// enabled = true

#[test]
fn test_hook_config_validation_timeout_zero()
// Reject config with timeout = 0 (validation error)

#[test]
fn test_hook_config_validation_timeout_negative()
// Reject config with negative timeout (if applicable)

#[test]
fn test_hook_config_validation_excessive_retries()
// Reject config with retries > 10 (prevent infinite loops)

#[test]
fn test_hook_config_disabled_hook()
// Verify enabled=false hooks are skipped during execution

#[test]
fn test_hook_config_timeout_conversion()
// Verify timeout_secs correctly converts to Duration
```

## 2. Unit Tests for registry.rs

### 2.1 HookRegistry Basic Operations (6 tests)

```rust
#[test]
fn test_registry_register_hook()
// Register a hook and verify it's stored

#[test]
fn test_registry_get_hooks_existing()
// Retrieve hooks for a registered HookType

#[test]
fn test_registry_get_hooks_nonexistent()
// Retrieve hooks for non-existent HookType returns empty Vec

#[test]
fn test_registry_clear_all_hooks()
// Clear all hooks and verify registry is empty

#[test]
fn test_registry_multiple_hooks_same_type()
// Register multiple hooks for same HookType, all should be retrieved

#[test]
fn test_registry_hooks_different_types()
// Register hooks for different types, verify isolation
```

### 2.2 HookRegistry Thread Safety (4 tests)

```rust
#[tokio::test]
async fn test_registry_concurrent_registration()
// Spawn 10 threads, each registering 10 hooks concurrently
// Verify total count = 100, no race conditions

#[tokio::test]
async fn test_registry_concurrent_reads()
// Register hooks, then spawn 20 threads reading concurrently
// Verify all threads get correct results

#[tokio::test]
async fn test_registry_concurrent_read_write()
// Mix of threads registering and reading simultaneously
// Verify data consistency

#[tokio::test]
async fn test_registry_arc_sharing()
// Verify HookRegistry can be wrapped in Arc and shared
// Test with Arc::new(HookRegistry::new())
```

## 3. Unit Tests for executor.rs

### 3.1 HookExecutor Basic Execution (5 tests)

```rust
#[tokio::test]
async fn test_execute_successful_hook()
// Hook that completes successfully, verify Result::Ok

#[tokio::test]
async fn test_execute_failing_hook()
// Hook that returns Err, verify executor handles gracefully

#[tokio::test]
async fn test_execute_hook_with_payload()
// Verify hook receives correct HookPayload (command, context)

#[tokio::test]
async fn test_execute_hook_return_value()
// Verify hook return value is captured correctly

#[tokio::test]
async fn test_execute_async_hook()
// Test async hook execution with .await
```

### 3.2 HookExecutor Timeout Tests (4 tests)

```rust
#[tokio::test]
async fn test_execute_hook_timeout()
// Hook that sleeps for 10 seconds, timeout=5s
// Verify hook times out and returns error

#[tokio::test]
async fn test_execute_hook_within_timeout()
// Hook that completes in 2 seconds, timeout=5s
// Verify hook succeeds

#[tokio::test]
async fn test_timeout_accuracy()
// Measure actual timeout duration, should be ~5s ±100ms

#[tokio::test]
async fn test_timeout_cleanup()
// Verify timed-out hook task is properly cancelled/cleaned up
```

### 3.3 HookExecutor Retry Logic (5 tests)

```rust
#[tokio::test]
async fn test_retry_successful_on_second_attempt()
// Hook fails first time, succeeds second time
// Verify retry happens and hook eventually succeeds

#[tokio::test]
async fn test_retry_exhaustion()
// Hook always fails, retries=2
// Verify hook is retried 2 times, then returns final error

#[tokio::test]
async fn test_retry_count_tracking()
// Verify retry counter increments correctly

#[tokio::test]
async fn test_no_retry_on_immediate_success()
// Hook succeeds first time
// Verify no retries are attempted

#[tokio::test]
async fn test_retry_delay()
// Optional: verify delay between retries (if implemented)
```

### 3.4 HookExecutor Panic Recovery (4 tests)

```rust
#[tokio::test]
async fn test_hook_panic_recovery()
// Hook that panics via panic!()
// Verify executor catches panic and returns error, doesn't crash daemon

#[tokio::test]
async fn test_hook_panic_message()
// Hook panics with specific message
// Verify panic message is captured in error

#[tokio::test]
async fn test_hook_panic_no_retry()
// Hook panics - should NOT retry (panic = permanent failure)

#[tokio::test]
async fn test_multiple_hook_panic_isolation()
// Execute 3 hooks: [success, panic, success]
// Verify panic in hook 2 doesn't affect hooks 1 and 3
```

### 3.5 HookExecutor Concurrent Execution (3 tests)

```rust
#[tokio::test]
async fn test_execute_multiple_hooks_parallel()
// Register 5 hooks, execute concurrently
// Verify all 5 complete in roughly same time (parallel)

#[tokio::test]
async fn test_concurrent_execution_isolation()
// Execute hooks concurrently, verify they don't interfere

#[tokio::test]
async fn test_concurrent_execution_error_handling()
// Mix of successful and failing hooks executing concurrently
// Verify each result is correctly captured
```

## 4. Integration Tests (daemon integration)

File: `cco/tests/hooks_integration_tests.rs`

### 4.1 Daemon Lifecycle Integration (5 tests)

```rust
#[tokio::test]
async fn test_daemon_hook_registry_initialization()
// Verify DaemonManager initializes HookRegistry on startup

#[tokio::test]
async fn test_hooks_config_loaded_from_daemon_config()
// Create daemon config with hooks section
// Verify hooks are registered from config

#[tokio::test]
async fn test_missing_hooks_config_graceful()
// Daemon config without hooks section
// Verify daemon starts successfully (hooks are optional)

#[tokio::test]
async fn test_hooks_available_during_lifecycle()
// Start daemon, verify hooks can be accessed
// Stop daemon, verify cleanup happens

#[tokio::test]
async fn test_pre_start_post_start_hooks_fire()
// Register PreStart and PostStart hooks
// Start daemon, verify both hooks executed in order
```

### 4.2 Hook Execution During Daemon Operations (5 tests)

```rust
#[tokio::test]
async fn test_pre_stop_post_stop_hooks_fire()
// Register PreStop and PostStop hooks
// Stop daemon, verify both hooks executed

#[tokio::test]
async fn test_custom_hook_registration()
// Register custom hook via API/config
// Verify it's available and executable

#[tokio::test]
async fn test_hook_failure_doesnt_block_daemon()
// Register failing PreStart hook
// Verify daemon still starts (hook failures are logged, not fatal)

#[tokio::test]
async fn test_hook_execution_logged()
// Execute hooks, verify log entries created
// Check log for: hook name, duration, result

#[tokio::test]
async fn test_hook_metrics_tracked()
// If metrics enabled, verify hook execution metrics:
// - total_hooks_executed counter
// - hook_duration_seconds histogram
// - hook_errors_total counter
```

## 5. Mock Strategies

### 5.1 Test Hook Helpers

```rust
/// Mock hook that simulates work
async fn mock_working_hook(payload: HookPayload) -> Result<(), HookError> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(())
}

/// Mock hook that always fails
async fn mock_failing_hook(payload: HookPayload) -> Result<(), HookError> {
    Err(HookError::ExecutionFailed("Mock failure".to_string()))
}

/// Mock hook that panics
async fn mock_panicking_hook(payload: HookPayload) -> Result<(), HookError> {
    panic!("Mock panic");
}

/// Mock hook that times out
async fn mock_timeout_hook(payload: HookPayload) -> Result<(), HookError> {
    tokio::time::sleep(Duration::from_secs(30)).await;
    Ok(())
}

/// Mock hook with retry tracking
struct RetryTracker {
    attempt: Arc<AtomicUsize>,
    fail_until: usize,
}

impl RetryTracker {
    async fn hook(&self, payload: HookPayload) -> Result<(), HookError> {
        let attempt = self.attempt.fetch_add(1, Ordering::SeqCst);
        if attempt < self.fail_until {
            Err(HookError::ExecutionFailed(format!("Attempt {}", attempt)))
        } else {
            Ok(())
        }
    }
}
```

### 5.2 Test Timing Helpers

```rust
/// Measure hook execution duration
async fn measure_duration<F, Fut>(f: F) -> (Duration, Result<(), HookError>)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), HookError>>,
{
    let start = Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    (duration, result)
}

/// Assert duration is within tolerance
fn assert_duration_near(actual: Duration, expected: Duration, tolerance_ms: u64) {
    let diff = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };
    assert!(
        diff.as_millis() <= tolerance_ms as u128,
        "Duration {:?} not within {}ms of {:?}",
        actual,
        tolerance_ms,
        expected
    );
}
```

## 6. Error Scenario Tests

### 6.1 Configuration Errors (3 tests)

```rust
#[test]
fn test_invalid_timeout_zero()
// Config with timeout = 0 → HookConfigError::InvalidTimeout

#[test]
fn test_invalid_retries_excessive()
// Config with retries = 100 → HookConfigError::ExcessiveRetries

#[test]
fn test_malformed_config_json()
// Malformed JSON → HookConfigError::ParseError
```

### 6.2 Execution Errors (4 tests)

```rust
#[tokio::test]
async fn test_hook_not_found()
// Try to execute non-existent hook → HookError::NotFound

#[tokio::test]
async fn test_hook_disabled()
// Execute disabled hook (config.enabled=false) → skipped silently

#[tokio::test]
async fn test_hook_timeout_error()
// Hook times out → HookError::Timeout

#[tokio::test]
async fn test_hook_panic_error()
// Hook panics → HookError::Panic(message)
```

## 7. Safety & Invariant Tests

### 7.1 Memory Safety (3 tests)

```rust
#[tokio::test]
async fn test_no_memory_leak_on_hook_failure()
// Execute failing hook 1000 times
// Verify memory usage doesn't grow

#[tokio::test]
async fn test_no_memory_leak_on_timeout()
// Execute timing-out hook 100 times
// Verify tasks are properly cleaned up

#[tokio::test]
async fn test_hook_registry_memory_cleanup()
// Register 1000 hooks, clear, repeat 100 times
// Verify memory usage stable
```

### 7.2 Thread Safety (2 tests)

```rust
#[tokio::test]
async fn test_concurrent_hook_registration_no_data_race()
// Use thread sanitizer or manual verification
// Ensure no data races in HookRegistry

#[tokio::test]
async fn test_concurrent_execution_no_deadlock()
// Execute many hooks concurrently
// Verify no deadlocks occur
```

### 7.3 Timeout Enforcement (2 tests)

```rust
#[tokio::test]
async fn test_timeout_always_respected()
// Hook with 10s sleep, timeout=5s
// Verify timeout fires at 5s ±100ms

#[tokio::test]
async fn test_timeout_task_cancellation()
// Verify timed-out task is actually cancelled (not running in background)
```

## 8. Daemon Integration Safety

### 8.1 Daemon Stability (3 tests)

```rust
#[tokio::test]
async fn test_hook_failure_never_crashes_daemon()
// Hook that panics/fails
// Verify daemon process stays alive

#[tokio::test]
async fn test_hook_timeout_never_blocks_daemon()
// Hook that times out
// Verify daemon remains responsive

#[tokio::test]
async fn test_multiple_hook_errors_daemon_stable()
// 5 hooks all fail/panic/timeout
// Verify daemon continues operating normally
```

## 9. Test Execution Strategy

### 9.1 Test Organization

```
cco/tests/
├── hooks_unit_tests.rs          # All unit tests (types, registry, executor)
├── hooks_integration_tests.rs   # Daemon integration tests
└── HOOKS_TEST_SPECIFICATION.md  # This document
```

### 9.2 Test Running

```bash
# Run all hooks tests
cargo test hooks --lib --tests

# Run only unit tests
cargo test hooks_unit --lib

# Run only integration tests
cargo test hooks_integration --test hooks_integration_tests

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage/hooks
```

### 9.3 CI/CD Integration

```yaml
# .github/workflows/hooks-tests.yml
name: Hooks Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run hooks tests
        run: cargo test hooks
      - name: Check coverage
        run: cargo tarpaulin --out Lcov --output-dir coverage
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## 10. Test Coverage Report Template

After implementation, provide a report in this format:

```markdown
# Hooks Test Coverage Report

## Summary
- **Total Tests**: 42
- **Passing**: 42
- **Failing**: 0
- **Code Coverage**: 97.3%

## Coverage by Module
- types.rs: 98.5%
- registry.rs: 96.2%
- executor.rs: 97.1%

## Coverage by Category
- Unit Tests: 32 (100% passing)
- Integration Tests: 10 (100% passing)
- Safety Tests: 7 (100% passing)
- Error Tests: 7 (100% passing)

## Untested Code Paths
1. executor.rs line 234: Rare panic recovery branch (0.2% of executions)
2. registry.rs line 156: Arc unwrap in drop (unreachable in practice)

## Performance Benchmarks
- Hook registration: 0.05ms avg
- Hook execution (success): 0.12ms avg
- Hook execution (timeout): 5.001s (within tolerance)
- Concurrent registration (1000 hooks): 45ms

## Recommendations
1. Add fuzz testing for HookPayload serialization
2. Add property-based testing for retry logic
3. Add benchmark suite for performance regression detection
```

## 11. Dependencies Required

Add to `cco/Cargo.toml`:

```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"
proptest = "1.4"       # For property-based testing
criterion = "0.5"      # For benchmarking
```

## 12. Success Criteria

Tests are considered complete and successful when:

1. ✅ All 40+ test cases implemented and passing
2. ✅ Code coverage ≥ 95% for hooks modules
3. ✅ All safety invariants tested (thread safety, panic recovery, timeouts)
4. ✅ Integration tests verify daemon integration works
5. ✅ No memory leaks detected
6. ✅ All error scenarios handled gracefully
7. ✅ Documentation updated with test results
8. ✅ CI/CD pipeline includes hooks tests

## Next Steps

1. **Rust Specialist**: Implement hooks modules (types, registry, executor)
2. **Test Engineer**: Implement tests based on this specification
3. **Security Auditor**: Review panic recovery and timeout enforcement
4. **DevOps Engineer**: Add hooks tests to CI/CD pipeline
5. **Documentation Lead**: Update user docs with hooks usage examples

---

**Test Specification Version**: 1.0
**Last Updated**: 2025-11-17
**Owner**: Test Engineer
**Reviewers**: Rust Specialist, Chief Architect, Security Auditor
