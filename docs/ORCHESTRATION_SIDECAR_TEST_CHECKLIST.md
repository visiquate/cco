# Orchestration Sidecar Test Execution Checklist

**Use this checklist when testing the orchestration sidecar implementation**

## Pre-Test Setup

### Environment Preparation
- [ ] Sidecar compiled successfully (`cargo build --release --bin orchestration-sidecar`)
- [ ] Port 3001 is available (`lsof -i :3001` shows nothing)
- [ ] Test dependencies installed (`cargo build --tests`)
- [ ] Temporary directories writable (`/tmp/cco-sidecar/`)
- [ ] Sufficient disk space (>1GB free)
- [ ] Sufficient memory (>2GB free)

### Sidecar Startup
- [ ] Sidecar starts without errors
- [ ] Health endpoint responds (`curl http://localhost:3001/health`)
- [ ] Status endpoint responds (`curl http://localhost:3001/status`)
- [ ] Logs directory created (`/tmp/cco-sidecar/logs/`)
- [ ] No error messages in logs

Expected health response:
```json
{
  "status": "healthy",
  "service": "orchestration-sidecar",
  "version": "1.0.0",
  "uptime_seconds": 5,
  "checks": {
    "storage": "healthy",
    "event_bus": "healthy",
    "memory_usage_mb": 50,
    "active_agents": 0,
    "event_queue_size": 0
  }
}
```

## Test Execution Phases

### Phase 1: Smoke Tests (Fast - 30 seconds)

Run basic health and connectivity tests:

```bash
cargo test test_health_endpoint test_status_endpoint
```

**Expected Results**:
- [ ] `test_health_endpoint` - PASSED
- [ ] `test_status_endpoint` - PASSED

**If Failed**: Check sidecar startup, verify port 3001 is listening

---

### Phase 2: Core Workflow Tests (1-2 minutes)

Test fundamental agent workflows:

```bash
cargo test test_agent_spawn_context_inject_workflow
cargo test test_agent_execute_result_store_workflow
cargo test test_context_includes_all_required_fields
cargo test test_result_retrieval_by_id
```

**Expected Results**:
- [ ] `test_agent_spawn_context_inject_workflow` - PASSED
  - Context returned in <100ms
  - All required fields present
- [ ] `test_agent_execute_result_store_workflow` - PASSED
  - Results stored successfully
  - Result ID returned
- [ ] `test_context_includes_all_required_fields` - PASSED
  - project_structure present
  - relevant_files present
  - git_context present
  - metadata present
- [ ] `test_result_retrieval_by_id` - PASSED
  - Results queryable
  - Status shows stored results

**If Failed**: Check context gatherer implementation, result storage paths

---

### Phase 3: Event Coordination Tests (1 minute)

Test event bus functionality:

```bash
cargo test test_event_publish_subscribe
cargo test test_multiple_subscribers_receive_event
cargo test test_multi_round_agent_coordination
```

**Expected Results**:
- [ ] `test_event_publish_subscribe` - PASSED
  - Events published in <50ms
  - Subscribers receive events
- [ ] `test_multiple_subscribers_receive_event` - PASSED
  - All subscribers notified
  - Event data intact
- [ ] `test_multi_round_agent_coordination` - PASSED
  - 3-agent workflow completes
  - Events trigger next agents
  - Correlation IDs maintained

**If Failed**: Check event bus implementation, long-polling logic

---

### Phase 4: Isolation and Security Tests (1 minute)

Test project isolation and security:

```bash
cargo test test_project_level_isolation
cargo test test_graceful_degradation_when_sidecar_unavailable
```

**Expected Results**:
- [ ] `test_project_level_isolation` - PASSED
  - Project A and B contexts separated
  - No cross-project data leakage
- [ ] `test_graceful_degradation_when_sidecar_unavailable` - PASSED
  - Connection errors handled gracefully
  - No panics or crashes

**If Failed**: Check JWT validation, project scoping in storage

---

### Phase 5: Error Recovery Tests (1 minute)

Test error handling and recovery:

```bash
cargo test test_sidecar_crash_recovery
cargo test test_request_retry_after_timeout
```

**Expected Results**:
- [ ] `test_sidecar_crash_recovery` - PASSED
  - Health checks work
  - Recovery possible
- [ ] `test_request_retry_after_timeout` - PASSED
  - Timeouts handled gracefully
  - Error messages clear

**If Failed**: Check health endpoint, error response formatting

---

### Phase 6: Concurrency Tests (2-3 minutes)

Test concurrent access and load handling:

```bash
cargo test test_concurrent_agent_requests
cargo test test_no_data_corruption_under_load
```

**Expected Results**:
- [ ] `test_concurrent_agent_requests` - PASSED
  - 119 concurrent agents handled
  - Success rate >90%
  - Response times acceptable
- [ ] `test_no_data_corruption_under_load` - PASSED
  - No result ID collisions
  - All unique data preserved
  - No race conditions

**If Failed**: Check locking mechanisms, connection pool settings

**Metrics to Record**:
- Success rate: ____%
- Average response time: ____ms
- p99 response time: ____ms
- Memory usage during test: ____MB

---

### Phase 7: Optional Tests (1 minute)

Test additional features:

```bash
cargo test test_rate_limiting_enforcement
cargo test test_context_includes_knowledge_store_data
cargo test test_context_truncation_for_large_projects
cargo test test_query_results_by_project
```

**Expected Results**:
- [ ] `test_rate_limiting_enforcement` - PASSED or SKIPPED (if not implemented)
- [ ] `test_context_includes_knowledge_store_data` - PASSED
- [ ] `test_context_truncation_for_large_projects` - PASSED
- [ ] `test_query_results_by_project` - PASSED

**If Failed**: Check specific implementation, may be expected if feature not yet complete

---

### Phase 8: Performance Benchmarks (2-3 minutes)

Run performance benchmarks:

```bash
cargo test benchmark_context_injection_latency --release -- --nocapture
cargo test benchmark_event_publish_latency --release -- --nocapture
```

**Expected Results**:
- [ ] `benchmark_context_injection_latency` - PASSED
  - p50: <50ms
  - p95: <80ms
  - p99: <100ms
- [ ] `benchmark_event_publish_latency` - PASSED
  - p50: <20ms
  - p95: <40ms
  - p99: <50ms

**Metrics to Record**:

Context Injection:
- p50: ____ms
- p95: ____ms
- p99: ____ms

Event Publishing:
- p50: ____ms
- p95: ____ms
- p99: ____ms

**If Failed**: Investigate performance bottlenecks, check resource limits

---

### Phase 9: Load Test (5-10 minutes)

Run comprehensive load test:

```bash
cargo test load_test_119_concurrent_agents --ignored --release -- --nocapture
```

**Expected Results**:
- [ ] `load_test_119_concurrent_agents` - PASSED
  - 119 concurrent agents spawned
  - Success rate >95%
  - Total time <30 seconds
  - No memory leaks
  - All results stored correctly

**Metrics to Record**:
- Total agents: 119
- Successful: ____
- Failed: ____
- Success rate: ____%
- Total time: ____s
- Average time per agent: ____ms
- Peak memory usage: ____MB
- Memory at end: ____MB

**If Failed**: Check resource limits, investigate failures, review logs

---

## Post-Test Verification

### Result Validation
- [ ] All test results logged
- [ ] Performance metrics recorded
- [ ] No error messages in sidecar logs
- [ ] Resource cleanup completed
- [ ] Temporary files removed

### Log Analysis
```bash
# Check for errors in sidecar logs
grep -i error /tmp/cco-sidecar/logs/sidecar.log

# Check for warnings
grep -i warn /tmp/cco-sidecar/logs/sidecar.log

# Review event log
tail -100 /tmp/cco-sidecar/events/event-log.jsonl
```

**Log Review**:
- [ ] No unexpected errors
- [ ] No memory warnings
- [ ] No connection errors
- [ ] Event log shows expected activity

### Resource Cleanup
```bash
# Check for leaked resources
lsof -i :3001

# Check disk usage
du -sh /tmp/cco-sidecar/

# Check for temp files
ls -la /tmp/agent-context-*
```

**Cleanup Verification**:
- [ ] No orphaned processes
- [ ] Disk usage reasonable (<100MB)
- [ ] No temp files leaked
- [ ] Memory returned to baseline

## Test Results Summary

### Overall Results
- Total tests run: _____
- Passed: _____
- Failed: _____
- Skipped: _____
- Success rate: _____%

### Performance Summary
| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Context injection p99 | <100ms | ____ms | ☐ PASS ☐ FAIL |
| Event publish p99 | <50ms | ____ms | ☐ PASS ☐ FAIL |
| 119 concurrent success | >95% | ____% | ☐ PASS ☐ FAIL |
| Data corruption | 0% | ____% | ☐ PASS ☐ FAIL |

### Critical Path Status
- [ ] Agent spawn and context injection - WORKING
- [ ] Result storage and retrieval - WORKING
- [ ] Event coordination - WORKING
- [ ] Multi-round agent flow - WORKING
- [ ] Project isolation - WORKING
- [ ] Error recovery - WORKING
- [ ] Concurrency handling - WORKING
- [ ] Performance targets met - WORKING

## Issues Found

### Issue 1
**Test**: __________________
**Symptom**: __________________
**Severity**: ☐ Critical ☐ High ☐ Medium ☐ Low
**Status**: ☐ Open ☐ Fixed ☐ Investigating

### Issue 2
**Test**: __________________
**Symptom**: __________________
**Severity**: ☐ Critical ☐ High ☐ Medium ☐ Low
**Status**: ☐ Open ☐ Fixed ☐ Investigating

### Issue 3
**Test**: __________________
**Symptom**: __________________
**Severity**: ☐ Critical ☐ High ☐ Medium ☐ Low
**Status**: ☐ Open ☐ Fixed ☐ Investigating

## Sign-Off

### Test Execution
- **Executed by**: __________________
- **Date**: __________________
- **Duration**: __________________
- **Environment**: ☐ Development ☐ Staging ☐ Production

### Results
- **Overall Status**: ☐ PASS ☐ FAIL ☐ PARTIAL
- **Critical Issues**: _____
- **High Priority Issues**: _____
- **Medium Priority Issues**: _____
- **Low Priority Issues**: _____

### Approval
- **QA Engineer**: __________________
- **Date**: __________________
- **Approved for Release**: ☐ Yes ☐ No ☐ With Conditions

**Conditions (if applicable)**:
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________

## Next Steps

### If All Tests Passed
- [ ] Document performance baselines
- [ ] Update architecture documentation
- [ ] Prepare deployment guide
- [ ] Schedule production deployment

### If Tests Failed
- [ ] Review failed test logs
- [ ] Identify root causes
- [ ] Create bug reports
- [ ] Schedule fixes
- [ ] Re-run affected tests

### Follow-Up Actions
1. _________________________________________________________________
2. _________________________________________________________________
3. _________________________________________________________________

---

**Checklist Version**: 1.0.0
**Last Updated**: November 18, 2025
**Related Documents**:
- `/Users/brent/git/cc-orchestra/docs/ORCHESTRATION_SIDECAR_TEST_REPORT.md`
- `/Users/brent/git/cc-orchestra/docs/ORCHESTRATION_SIDECAR_TESTING_QUICK_START.md`
- `/Users/brent/git/cc-orchestra/docs/ORCHESTRATION_SIDECAR_ARCHITECTURE.md`
