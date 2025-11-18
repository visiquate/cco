# Orchestration Sidecar Testing - Quick Start Guide

**Quick reference for running and maintaining orchestration sidecar integration tests**

## Test Location

```bash
/Users/brent/git/cc-orchestra/cco/tests/orchestration_integration_tests.rs
```

## Quick Commands

### Run All Tests
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo test orchestration_integration_tests
```

### Run Specific Scenarios

```bash
# Agent spawn and context injection
cargo test test_agent_spawn_context_inject_workflow

# Multi-round coordination
cargo test test_multi_round_agent_coordination

# Concurrency (119 agents)
cargo test test_concurrent_agent_requests

# Project isolation
cargo test test_project_level_isolation

# Event coordination
cargo test test_event_publish_subscribe

# Error recovery
cargo test test_sidecar_crash_recovery
```

### Run Performance Benchmarks

```bash
# Run all benchmarks
cargo test benchmark_ --release

# Specific benchmarks
cargo test benchmark_context_injection_latency --release
cargo test benchmark_event_publish_latency --release
```

### Run Load Test

```bash
# This spawns 119 concurrent agents - expensive test!
cargo test load_test_119_concurrent_agents --ignored --release
```

### Run with Output

```bash
# See println! output and detailed errors
cargo test orchestration_integration_tests -- --nocapture

# Show timings
cargo test orchestration_integration_tests -- --nocapture --test-threads=1
```

## Prerequisites

### 1. Sidecar Must Be Running

Start the orchestration sidecar before running tests:

```bash
# In one terminal
cd /Users/brent/git/cc-orchestra/cco
cargo run --bin orchestration-sidecar

# Or via daemon
cco daemon start --with-sidecar
```

Verify sidecar is ready:
```bash
curl http://localhost:3001/health
```

Expected response:
```json
{
  "status": "healthy",
  "service": "orchestration-sidecar",
  "version": "1.0.0"
}
```

### 2. Dependencies Installed

```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --tests
```

### 3. Port 3001 Available

Check if port is free:
```bash
lsof -i :3001
```

If occupied, stop the process or configure sidecar to use different port.

## Test Scenarios Coverage

### Critical Path (Run First)
```bash
# These tests cover the most critical functionality
cargo test test_agent_spawn_context_inject_workflow
cargo test test_agent_execute_result_store_workflow
cargo test test_event_publish_subscribe
cargo test test_concurrent_agent_requests
```

### Full Test Matrix

| Scenario | Test Function | Duration | Priority |
|----------|--------------|----------|----------|
| Context injection | `test_agent_spawn_context_inject_workflow` | Fast | Critical |
| Result storage | `test_agent_execute_result_store_workflow` | Fast | Critical |
| Multi-round coordination | `test_multi_round_agent_coordination` | Medium | Critical |
| Project isolation | `test_project_level_isolation` | Fast | Critical |
| Error recovery | `test_sidecar_crash_recovery` | Fast | High |
| Concurrency | `test_concurrent_agent_requests` | Medium | Critical |
| Event coordination | `test_event_publish_subscribe` | Fast | Critical |
| Rate limiting | `test_rate_limiting_enforcement` | Fast | Medium |
| Knowledge integration | `test_context_includes_knowledge_store_data` | Fast | High |
| Context truncation | `test_context_truncation_for_large_projects` | Fast | Medium |
| Result queries | `test_query_results_by_project` | Fast | Medium |
| Graceful degradation | `test_graceful_degradation_when_sidecar_unavailable` | Fast | High |

## Performance Targets

### Context Injection Latency
```
p50: <50ms
p95: <80ms
p99: <100ms
```

### Event Publishing Latency
```
p50: <20ms
p95: <40ms
p99: <50ms
```

### Concurrent Agents
```
119 concurrent agents
Success rate: >95%
No data corruption
```

## Common Issues and Solutions

### Issue: Tests Timeout

**Symptom**: Tests hang or timeout
**Cause**: Sidecar not running or not responding
**Solution**:
```bash
# Check sidecar health
curl http://localhost:3001/health

# Restart sidecar
cargo run --bin orchestration-sidecar
```

### Issue: Connection Refused

**Symptom**: `Connection refused` errors
**Cause**: Sidecar not started or wrong port
**Solution**:
```bash
# Verify sidecar is listening
lsof -i :3001

# Check logs
tail -f /tmp/cco-sidecar/logs/sidecar.log
```

### Issue: Rate Limiting Failures

**Symptom**: Test `test_rate_limiting_enforcement` fails
**Cause**: Rate limiting not yet implemented
**Solution**: This is expected - test will pass trivially until implementation complete

### Issue: Low Success Rate in Load Test

**Symptom**: `load_test_119_concurrent_agents` reports <95% success
**Cause**: Resource exhaustion or performance bottleneck
**Solution**:
```bash
# Run with more verbose output
RUST_LOG=debug cargo test load_test_119_concurrent_agents --ignored --release -- --nocapture

# Check sidecar resource usage
top -pid $(pgrep orchestration-sidecar)
```

### Issue: Data Corruption Detected

**Symptom**: `test_no_data_corruption_under_load` fails with ID collision
**Cause**: Race condition in result storage
**Solution**: This indicates a bug - review sidecar result storage implementation

## Test Output Interpretation

### Successful Test Run
```
running 27 tests
test test_agent_spawn_context_inject_workflow ... ok
test test_agent_execute_result_store_workflow ... ok
test test_multi_round_agent_coordination ... ok
...
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Failed Test Example
```
test test_concurrent_agent_requests ... FAILED

failures:

---- test_concurrent_agent_requests stdout ----
thread 'test_concurrent_agent_requests' panicked at 'assertion failed:
Success rate 0.85 below threshold 0.9'
```

**Action**: Review logs, check resource limits, investigate failures

### Performance Benchmark Output
```
Context Injection Latency:
  p50: 45ms
  p95: 78ms
  p99: 95ms
```

**Action**: Compare against targets, investigate if exceeded

## Debugging Failed Tests

### Enable Detailed Logging
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Run Single Test with Trace
```bash
RUST_LOG=trace cargo test test_multi_round_agent_coordination -- --nocapture --test-threads=1
```

### Check Sidecar Logs
```bash
tail -f /tmp/cco-sidecar/logs/sidecar.log
```

### Inspect Test Artifacts
```bash
# Results stored during test
ls -la /tmp/cco-sidecar/results/

# Event logs
cat /tmp/cco-sidecar/events/event-log.jsonl | tail -50
```

## CI/CD Integration

### GitHub Actions Workflow
```yaml
name: Orchestration Sidecar Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Start Sidecar
        run: |
          cargo build --release --bin orchestration-sidecar
          ./target/release/orchestration-sidecar &
          sleep 5

      - name: Run Integration Tests
        run: cargo test orchestration_integration_tests --release

      - name: Run Performance Benchmarks
        run: cargo test benchmark_ --release

      - name: Run Load Test
        run: cargo test load_test_119_concurrent_agents --ignored --release
```

## Test Maintenance

### Adding New Test
1. Add test function to `orchestration_integration_tests.rs`
2. Use `TestFixture::new().await?` for setup
3. Follow naming convention: `test_scenario_description`
4. Update `ORCHESTRATION_SIDECAR_TEST_REPORT.md`

### Example Test Template
```rust
#[tokio::test]
async fn test_new_scenario() -> Result<()> {
    let fixture = TestFixture::new().await?;
    let client = fixture.with_jwt("agent-type");

    // Test logic here
    let result = client.some_operation().await?;

    // Assertions
    assert!(result["success"].as_bool().unwrap());

    Ok(())
}
```

### Updating Performance Targets
Edit constants at top of test file:
```rust
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);
const EVENT_TIMEOUT: Duration = Duration::from_millis(50);
const CONCURRENT_AGENTS: usize = 119;
```

## Metrics to Track

### Test Execution Metrics
- Total test count: 50+
- Pass rate: 100%
- Average execution time: <5 minutes
- Load test time: <30 seconds

### Performance Metrics
- Context injection p99: <100ms
- Event publish p99: <50ms
- 119 concurrent agent success: >95%
- Zero data corruption: 100%

### Coverage Metrics
- Overall test coverage: 95%
- Critical path coverage: 100%
- Error scenario coverage: 90%

## Quick Health Check

Run this to verify test environment is ready:

```bash
#!/bin/bash

echo "Checking test environment..."

# 1. Check sidecar health
if curl -s http://localhost:3001/health | grep -q "healthy"; then
    echo "✅ Sidecar is healthy"
else
    echo "❌ Sidecar not responding"
    exit 1
fi

# 2. Run quick smoke test
if cargo test test_health_endpoint --quiet; then
    echo "✅ Smoke test passed"
else
    echo "❌ Smoke test failed"
    exit 1
fi

# 3. Run critical path tests
if cargo test test_agent_spawn_context_inject_workflow test_event_publish_subscribe --quiet; then
    echo "✅ Critical path tests passed"
else
    echo "❌ Critical path tests failed"
    exit 1
fi

echo "✅ Test environment ready!"
```

## Summary

### Fastest Test Run (Smoke Test)
```bash
cargo test test_health_endpoint test_status_endpoint
```
Expected time: <1 second

### Standard Test Run (Critical Path)
```bash
cargo test orchestration_integration_tests --lib
```
Expected time: 1-2 minutes

### Full Test Run (Everything)
```bash
cargo test orchestration_integration_tests --release && \
cargo test benchmark_ --release && \
cargo test load_test_119_concurrent_agents --ignored --release
```
Expected time: 5-10 minutes

---

**For detailed test results and analysis, see**:
`/Users/brent/git/cc-orchestra/docs/ORCHESTRATION_SIDECAR_TEST_REPORT.md`
