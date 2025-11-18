# Orchestration Sidecar Test Documentation

**Status**: RED PHASE (All tests should fail - this is correct!)
**Test Suite**: 83 comprehensive tests
**Created**: 2025-11-18
**TDD Methodology**: Tests written BEFORE implementation

## Overview

This test suite defines the complete contract for the Orchestration Sidecar implementation. Following strict TDD principles, all 83 tests are written first and should fail until implementation is complete.

## Test Structure

```
orchestration_sidecar_tests.rs (83 tests)
├── server_tests (15 tests)         - HTTP API endpoints
├── broker_tests (12 tests)         - Knowledge broker and context
├── event_bus_tests (15 tests)      - Pub-sub messaging
├── storage_tests (10 tests)        - Result persistence
├── injector_tests (8 tests)        - Context injection
├── cli_tests (8 tests)             - CLI wrapper lifecycle
└── integration_tests (15 tests)    - End-to-end workflows
```

## Module Breakdown

### Module 1: HTTP Server Tests (15 tests)

These tests define the contract for the REST API server.

**Coverage:**
- ✅ GET /api/context/:issue_id/:agent_type returns proper structure
- ✅ POST /api/results stores correctly
- ✅ POST /api/events/:event_type publishes correctly
- ✅ GET /api/events/wait/:event_type subscribes (long-polling)
- ✅ GET /health returns healthy status
- ✅ GET /status returns detailed info
- ✅ Valid JWT token accepted
- ✅ Invalid JWT token rejected (401)
- ✅ Missing Authorization header rejected (401)
- ✅ Rate limiting enforced (429 after threshold)
- ✅ CORS headers present
- ✅ Error responses have proper status codes (404, 400)
- ✅ Context truncation when large
- ✅ Concurrent requests handled properly
- ✅ Security headers present (X-Content-Type-Options, X-Frame-Options)

**Expected Behavior:**
- All endpoints return proper JSON responses
- Authentication enforced on all protected endpoints
- Rate limiting prevents abuse
- CORS allows browser access
- Performance: <100ms response time (p99)

### Module 2: Knowledge Broker Tests (12 tests)

Tests for intelligent context gathering and caching.

**Coverage:**
- ✅ Auto-discovers agent context from files
- ✅ Gathers relevant knowledge from store
- ✅ Includes project structure
- ✅ Includes previous agent outputs
- ✅ Truncates large context intelligently
- ✅ Handles missing files gracefully
- ✅ Caches context appropriately (LRU)
- ✅ Evicts stale cache entries
- ✅ Enforces project-level isolation
- ✅ Validates agent permissions
- ✅ Tracks context access metrics
- ✅ Handles concurrent context requests safely

**Expected Behavior:**
- Context tailored to agent type
- Smart caching improves performance
- No cross-project data leakage
- >70% cache hit rate under load

### Module 3: Event Bus Tests (15 tests)

Tests for the pub-sub event coordination system.

**Coverage:**
- ✅ Publishes events to topics
- ✅ Subscribes to topics
- ✅ Multiple subscribers receive same event
- ✅ Topic filtering works
- ✅ Event timeout handling (long-polling)
- ✅ Dead letter queue for failed events
- ✅ Event retention (24h cleanup)
- ✅ Circular buffer doesn't lose old events (10,000 capacity)
- ✅ Concurrent publishes are safe
- ✅ Circular buffer capacity respected
- ✅ Event ordering guarantees per topic
- ✅ Event correlation IDs work
- ✅ Event publisher verification via JWT
- ✅ Event replay capability for debugging
- ✅ Event TTL enforcement

**Expected Behavior:**
- Reliable pub-sub messaging
- Events retained for 24 hours
- Circular buffer prevents unbounded growth
- Order preserved per topic
- <50ms event publish latency

### Module 4: Result Storage Tests (10 tests)

Tests for persistent result storage.

**Coverage:**
- ✅ Stores result with complete metadata
- ✅ Retrieves result by ID
- ✅ Queries results by agent type
- ✅ Queries results by project ID
- ✅ Results expire after retention period
- ✅ Concurrent writes don't corrupt data
- ✅ File storage format is valid JSON
- ✅ Large results handled (>10MB)
- ✅ Metadata validation
- ✅ Automatic cleanup of old results

**Expected Behavior:**
- Results stored in project-isolated directories
- JSON format for easy inspection
- Automatic cleanup prevents disk bloat
- Thread-safe concurrent access

### Module 5: Context Injector Tests (8 tests)

Tests for intelligent context gathering and injection.

**Coverage:**
- ✅ Finds relevant files for agent type
- ✅ Includes project config files
- ✅ Includes previous agent outputs
- ✅ Truncates intelligently (keeps most recent)
- ✅ Excludes sensitive files (.env, credentials)
- ✅ Handles missing directories gracefully
- ✅ Performance: fast context gathering (<100ms)
- ✅ Extracts git context (history, status)

**Expected Behavior:**
- Context tailored to agent's needs
- Smart truncation preserves important info
- No sensitive data leakage
- Fast context injection

### Module 6: CLI Wrapper Tests (8 tests)

Tests for the CLI wrapper that manages sidecar lifecycle.

**Coverage:**
- ✅ Starts sidecar on first invocation
- ✅ Sets environment variables for agents
- ✅ Passes through to Claude Code
- ✅ Graceful shutdown handling
- ✅ Handles stale PID files
- ✅ Port conflict handling
- ✅ Process cleanup on exit
- ✅ Agent spawn script generation

**Expected Behavior:**
- Transparent wrapper around Claude Code
- Automatic sidecar startup
- Clean shutdown and cleanup
- Handles edge cases gracefully

### Module 7: Integration Tests (15 tests)

End-to-end workflow tests.

**Coverage:**
- ✅ Full workflow: spawn → context → execute → result
- ✅ Multi-round agent interactions
- ✅ Event-driven agent coordination
- ✅ Error recovery and resilience
- ✅ Load testing: 119 concurrent agents
- ✅ Sidecar unavailability scenarios
- ✅ Knowledge isolation enforcement
- ✅ Project-level access control
- ✅ Security: agent token validation
- ✅ Graceful degradation under pressure
- ✅ Context cache hit rate optimization
- ✅ Chief Architect workflow
- ✅ Parallel agent execution
- ✅ Agent failure notification
- ✅ Performance: sub-100ms response (p99)

**Expected Behavior:**
- Complete workflows function correctly
- 119 concurrent agents supported
- >95% success rate under load
- Graceful error recovery
- Performance targets met

## Test Fixtures

Located in: `/Users/brent/git/cc-orchestra/cco/tests/fixtures/orchestration/`

### Files:

1. **agent_profiles.json** - 5 sample agent profiles
   - Chief Architect (Opus, high authority)
   - Python Specialist (Haiku, medium authority)
   - Test Engineer (Sonnet, medium authority)
   - Security Auditor (Sonnet, high authority)
   - Documentation Expert (Haiku, low authority)

2. **sample_context.json** - Example context structure
   - Project structure
   - Relevant files
   - Git context
   - Previous agent outputs
   - Metadata

3. **sample_events.json** - 5 example events
   - architecture_defined
   - implementation_complete
   - testing_complete
   - security_audit_complete
   - agent_failed (error case)

4. **sample_results.json** - 4 example results
   - Successful Python implementation
   - Successful test engineering
   - Successful security audit
   - Failed implementation (error case)

## Running Tests

### Run All Tests
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo test orchestration_sidecar_tests --lib
```

### Run Specific Module
```bash
cargo test orchestration_sidecar_tests::server_tests --lib
cargo test orchestration_sidecar_tests::broker_tests --lib
cargo test orchestration_sidecar_tests::event_bus_tests --lib
```

### Run Integration Tests
```bash
cargo test orchestration_sidecar_tests::integration_tests --lib
```

### Expected Results (RED PHASE)

**All tests should FAIL** - This is correct and expected!

```
test result: FAILED. 0 passed; 83 failed; 0 ignored; 0 measured
```

This is the RED phase of TDD. Each failing test defines what needs to be implemented.

## Implementation Guidance

### Test-Driven Implementation Order

Follow this order to implement the sidecar:

1. **Phase 1: HTTP Server (Week 1)**
   - Implement basic server with Axum
   - Add health/status endpoints
   - Implement JWT authentication
   - Add CORS and security headers
   - Tests should start passing: 5-7 tests ✓

2. **Phase 2: Storage Layer (Week 1-2)**
   - Implement result storage (JSON files)
   - Add in-memory context cache
   - Implement cleanup routines
   - Tests should start passing: 8-10 tests ✓

3. **Phase 3: Event Bus (Week 2)**
   - Implement pub-sub with tokio channels
   - Add circular buffer for event log
   - Implement long-polling subscriptions
   - Add dead letter queue
   - Tests should start passing: 12-15 tests ✓

4. **Phase 4: Context System (Week 3)**
   - Implement knowledge broker
   - Add context injector
   - Implement smart truncation
   - Add project isolation
   - Tests should start passing: 18-20 tests ✓

5. **Phase 5: API Endpoints (Week 3-4)**
   - Complete all REST endpoints
   - Add rate limiting
   - Implement context/results endpoints
   - Tests should start passing: 30+ tests ✓

6. **Phase 6: CLI Wrapper (Week 4)**
   - Implement agent spawn scripts
   - Add lifecycle management
   - Implement graceful shutdown
   - Tests should start passing: 8+ tests ✓

7. **Phase 7: Integration (Week 4)**
   - Wire all components together
   - Add error recovery
   - Implement graceful degradation
   - All 83 tests should pass ✓✓✓

## Test Assertions Reference

### HTTP Status Codes
- `200` - Success
- `400` - Bad Request (invalid data)
- `401` - Unauthorized (missing/invalid JWT)
- `404` - Not Found (endpoint doesn't exist)
- `429` - Too Many Requests (rate limited)
- `500` - Internal Server Error

### JSON Response Structures

**Context Response:**
```json
{
  "issue_id": "string",
  "agent_type": "string",
  "context": { ... },
  "truncated": boolean,
  "truncation_strategy": "string|null",
  "timestamp": "ISO8601"
}
```

**Result Storage Response:**
```json
{
  "id": "string",
  "stored": boolean,
  "storage_path": "string",
  "next_agents": ["string"],
  "event_published": boolean
}
```

**Event Publish Response:**
```json
{
  "event_id": "string",
  "published": boolean,
  "subscribers_notified": ["string"],
  "timestamp": "ISO8601"
}
```

**Health Response:**
```json
{
  "status": "healthy",
  "service": "orchestration-sidecar",
  "version": "string",
  "uptime_seconds": number,
  "checks": { ... }
}
```

## Performance Targets

| Metric | Target | Test Verification |
|--------|--------|-------------------|
| Context injection | <100ms (p99) | `benchmark_context_injection_latency` |
| Event publish | <50ms (p99) | `benchmark_event_publish_latency` |
| Concurrent agents | 119 simultaneous | `load_test_119_concurrent_agents` |
| Success rate | >95% under load | `test_concurrent_agent_requests` |
| Cache hit rate | >70% | `test_context_cache_hit_rate` |
| Memory usage | <1GB | `test_health_endpoint` |

## Security Checklist

Tests verify these security requirements:

- [x] JWT authentication on all protected endpoints
- [x] Project-level isolation enforced
- [x] Rate limiting prevents abuse
- [x] Input validation and sanitization
- [x] Secure token generation (RSA-256)
- [x] Event publisher verification
- [x] Resource limits (memory, CPU)
- [x] Audit logging for security events
- [x] HTTPS support (TLS 1.3)
- [x] No sensitive data in logs
- [x] CORS properly configured
- [x] Security headers present

## Debugging Failed Tests

### Common Issues During Implementation

1. **Server not starting** → Check port 3001 availability
2. **JWT validation failing** → Verify token format and signing
3. **Context not found** → Check project path and file discovery
4. **Events not received** → Verify pub-sub implementation and channels
5. **Results not stored** → Check filesystem permissions and paths
6. **Rate limiting not working** → Verify per-agent rate limit tracking

### Debug Commands

```bash
# Check if sidecar is running
curl http://localhost:3001/health

# View status
curl http://localhost:3001/status

# Test JWT authentication
curl -H "Authorization: Bearer test-token" \
  http://localhost:3001/api/context/issue-1/python-specialist

# Monitor logs
tail -f /tmp/cco-sidecar/sidecar.log
```

## Success Criteria

The RED phase is complete when:

1. ✅ All 83 tests compile successfully
2. ✅ All 83 tests run (and fail as expected)
3. ✅ Test fixtures are available
4. ✅ Documentation is complete
5. ✅ Implementation team understands contract

The GREEN phase is complete when:

1. ✅ All 83 tests pass
2. ✅ Performance targets met
3. ✅ Security requirements verified
4. ✅ 119 concurrent agents supported
5. ✅ Integration tests pass end-to-end

## Next Steps

1. **Implementation Team**: Start with Phase 1 (HTTP Server)
2. **Run tests frequently**: `cargo test orchestration_sidecar_tests --lib`
3. **Track progress**: Watch test count change from 0/83 → 83/83
4. **Refactor continuously**: Once tests pass, improve code quality
5. **Performance testing**: Run benchmarks and load tests

## Related Documentation

- Architecture: `/Users/brent/git/cc-orchestra/docs/ORCHESTRATION_SIDECAR_ARCHITECTURE.md`
- Integration Tests: `/Users/brent/git/cc-orchestra/cco/tests/orchestration_integration_tests.rs`
- Test Fixtures: `/Users/brent/git/cc-orchestra/cco/tests/fixtures/orchestration/`

---

**TDD Principle**: "Red → Green → Refactor"

We are currently in the **RED phase**. All tests failing is a success!
