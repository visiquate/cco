# Classifier TDD Quick Start Guide

## Overview

The classifier system has **226 comprehensive tests** already written following TDD principles. This guide shows how to use them during implementation.

## Quick Commands

### Run All Hooks Tests
```bash
cd /Users/brent/git/cc-orchestra/cco
VERSION_DATE=2025.11.24 cargo test hooks_
```

### Run Specific Test Files
```bash
# CRUD classification tests (43 tests)
VERSION_DATE=2025.11.24 cargo test --test hooks_classification_accuracy_tests

# API endpoint tests (21 tests)
VERSION_DATE=2025.11.24 cargo test --test hooks_api_classify_tests

# Permission system tests (12 tests)
VERSION_DATE=2025.11.24 cargo test --test hooks_permission_tests

# Integration tests (22 tests)
VERSION_DATE=2025.11.24 cargo test --test hooks_integration_tests
```

### Run Single Test
```bash
VERSION_DATE=2025.11.24 cargo test test_classify_ls_command -- --exact --nocapture
```

### Run Currently Ignored Tests
```bash
# These will fail until implementation is complete
VERSION_DATE=2025.11.24 cargo test hooks_ -- --ignored
```

---

## TDD Workflow

### Phase 1: RED (Current - Complete ✓)

**Status**: All 226 tests written and marked with `#[ignore]`

```rust
#[tokio::test]
#[ignore] // Remove when /api/classify is implemented
async fn test_classify_ls_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("ls -la").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}
```

**Current State**:
- ✅ Tests define expected behavior
- ✅ Test infrastructure complete
- ✅ Helper functions implemented
- ✅ Mock structures in place

### Phase 2: GREEN (Next - Implementation)

**Goal**: Make tests pass one by one

**Workflow**:
1. Pick a test file to implement (start with classification accuracy)
2. Remove `#[ignore]` from ONE test
3. Run that test - it should FAIL (RED)
4. Implement minimal code to make it pass
5. Run test - it should PASS (GREEN)
6. Repeat for next test

**Example**:
```rust
// Step 1: Remove #[ignore] from ONE test
#[tokio::test]
// #[ignore] // <-- REMOVED
async fn test_classify_ls_command() {
    // ... test code ...
}

// Step 2: Run test
// cargo test test_classify_ls_command -- --exact
// Expected: FAIL (no implementation yet)

// Step 3: Implement classifier
// ... add code to classify "ls" as READ ...

// Step 4: Run test again
// Expected: PASS (implementation works)
```

### Phase 3: REFACTOR (Final - Optimization)

**Goal**: Improve code quality while keeping tests green

**Activities**:
- Optimize performance
- Reduce code duplication
- Improve error handling
- Add inline documentation
- Run tests after each change to ensure they still pass

---

## Test Structure

### Test File Organization

```
tests/
├── hooks_test_helpers.rs              ← Shared utilities
├── hooks_classification_accuracy_tests.rs  ← CRUD accuracy (43)
├── hooks_api_classify_tests.rs        ← API endpoint (21)
├── hooks_permission_tests.rs          ← Permission flow (12)
├── hooks_integration_tests.rs         ← End-to-end (22)
├── hooks_error_scenarios_tests.rs     ← Error cases (27)
└── ... (other test files)
```

### Test Helpers Available

```rust
use hooks_test_helpers::*;

// Start test daemon
let daemon = TestDaemon::with_hooks_enabled().await?;
let daemon = TestDaemon::with_hooks_disabled().await?;

// Make classification request
let response = daemon.client.classify("ls -la").await?;

// Assert classification
assert_classification(&response, "READ", 0.8);

// Get test configuration
let config = test_hooks_config();
```

---

## Key Test Patterns

### Pattern 1: Basic Classification Test

```rust
#[tokio::test]
#[ignore] // Remove when ready
async fn test_classify_read_command() {
    // Arrange
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Act
    let response = daemon.client.classify("ls -la").await.unwrap();

    // Assert
    assert_eq!(response.classification.to_uppercase(), "READ");
    assert!(response.confidence >= 0.8);
    assert!(response.reasoning.is_some());
}
```

### Pattern 2: Error Handling Test

```rust
#[tokio::test]
#[ignore]
async fn test_classify_endpoint_invalid_json() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon.client.client
        .post(&url)
        .body("invalid json {{{")
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

### Pattern 3: Concurrent Test

```rust
#[tokio::test]
#[ignore]
async fn test_classify_multiple_requests_concurrently() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec!["ls -la", "mkdir test", "rm file.txt"];
    let mut handles = vec![];

    for cmd in commands {
        let client = daemon.client.clone();
        let command = cmd.to_string();
        handles.push(tokio::spawn(async move {
            client.classify(&command).await
        }));
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
```

---

## Implementation Order (Recommended)

### 1. Start with Core Classification (Week 1)
```bash
# Focus on these tests first
cargo test --test hooks_classification_accuracy_tests
```

**Files to implement**:
- `src/daemon/hooks/classifier.rs` - Core CRUD classifier
- `src/daemon/hooks/types.rs` - Classification types

**Tests to pass**:
- 43 classification accuracy tests
- Simple READ/CREATE/UPDATE/DELETE commands

### 2. Add API Endpoints (Week 1-2)
```bash
cargo test --test hooks_api_classify_tests
```

**Files to implement**:
- `src/daemon/hooks/api.rs` - HTTP handlers
- `src/daemon/hooks/routes.rs` - Axum routes

**Tests to pass**:
- 21 API endpoint tests
- Request/response format
- Error handling (400, 503, 405)

### 3. Implement Permission System (Week 2)
```bash
cargo test --test hooks_permission_tests
```

**Files to implement**:
- `src/daemon/hooks/permissions.rs` - Permission logic
- `src/daemon/hooks/database.rs` - SQLite persistence

**Tests to pass**:
- 12 permission system tests
- Auto-allow READ
- Require confirmation for C/U/D

### 4. Integration Testing (Week 3)
```bash
cargo test --test hooks_integration_tests
```

**Files to integrate**:
- `src/daemon/hooks/lifecycle.rs` - Daemon integration
- `src/daemon/hooks/health.rs` - Health checks

**Tests to pass**:
- 22 integration tests
- End-to-end workflows

### 5. Error Scenarios (Week 3)
```bash
cargo test --test hooks_error_scenarios_tests
```

**Files to enhance**:
- All files - add error handling
- Graceful degradation
- Timeout enforcement

**Tests to pass**:
- 27 error scenario tests

---

## Test Expectations

### Classification Response Format

```json
{
  "classification": "READ",
  "confidence": 0.85,
  "reasoning": "Command 'ls -la' is a read-only directory listing operation",
  "timestamp": "2025-11-24T09:00:00Z"
}
```

### Permission Response Format

```json
{
  "decision": "APPROVED",
  "reasoning": "Safe read-only operation",
  "timestamp": "2025-11-24T09:00:00Z"
}
```

### HTTP Status Codes Expected

| Code | Meaning | Test Scenarios |
|------|---------|----------------|
| 200 | Success | Valid classification requests |
| 400 | Bad Request | Invalid JSON, missing fields |
| 405 | Method Not Allowed | GET on POST endpoint |
| 429 | Too Many Requests | Rate limit exceeded (100/min) |
| 503 | Service Unavailable | Classifier disabled/unavailable |

---

## Verification Checklist

Before merging classifier implementation:

- [ ] All 226 tests have `#[ignore]` removed
- [ ] All tests pass: `cargo test hooks_`
- [ ] No warnings during compilation
- [ ] Code coverage >= 90% (use `cargo tarpaulin`)
- [ ] Performance: Classification < 3 seconds
- [ ] Concurrent safety: 10+ parallel requests work
- [ ] Database persistence: Decisions survive restart
- [ ] Error handling: All error scenarios handled gracefully

---

## Troubleshooting

### Tests Don't Compile

**Issue**: Missing types or modules

**Fix**: Ensure all types are defined:
```rust
// In src/daemon/hooks/types.rs
pub struct ClassifyResponse {
    pub classification: String,
    pub confidence: f32,
    pub reasoning: Option<String>,
    pub timestamp: Option<String>,
}
```

### Tests Timeout

**Issue**: Daemon not starting or classifier too slow

**Fix**: Check daemon startup and add timeout:
```rust
let daemon = TestDaemon::with_hooks_enabled().await?;
daemon.client.wait_for_ready(Duration::from_secs(10)).await?;
```

### Database Errors

**Issue**: SQLite database not initialized

**Fix**: Create database in test setup:
```rust
// Create temp database for testing
let db_path = temp_dir.path().join("test.db");
sqlx::SqliteConnection::connect(&format!("sqlite:{}", db_path.display())).await?;
```

### Flaky Concurrent Tests

**Issue**: Race conditions in concurrent requests

**Fix**: Use proper synchronization:
```rust
use tokio::sync::Semaphore;
let semaphore = Arc::new(Semaphore::new(10));
```

---

## Test Coverage Report

Current coverage by category:

```
CRUD Classification:    43 tests ✓
API Endpoints:          21 tests ✓
Error Scenarios:        27 tests ✓
Integration:            22 tests ✓
Unit Tests:             24 tests ✓
Permission System:      12 tests ✓
Daemon Lifecycle:       20 tests ✓
Other:                  57 tests ✓
─────────────────────────────────
TOTAL:                 226 tests ✓
```

---

## Resources

### Documentation
- [Full Test Coverage Report](./TDD_CLASSIFIER_TEST_COVERAGE.md)
- [Test Helpers API](./tests/hooks_test_helpers.rs)
- [CRUD Classification Guide](../CLAUDE.md)

### Key Files
- Test Files: `/Users/brent/git/cc-orchestra/cco/tests/hooks_*.rs`
- Implementation: `/Users/brent/git/cc-orchestra/cco/src/daemon/hooks/`
- Configuration: `/Users/brent/git/cc-orchestra/cco/Cargo.toml`

### Commands Reference
```bash
# Run all tests
VERSION_DATE=2025.11.24 cargo test

# Run hooks tests only
VERSION_DATE=2025.11.24 cargo test hooks_

# Run with output
VERSION_DATE=2025.11.24 cargo test hooks_ -- --nocapture

# Run ignored tests (will fail until implemented)
VERSION_DATE=2025.11.24 cargo test hooks_ -- --ignored

# Check test coverage
cargo tarpaulin --out Html --output-dir coverage
```

---

**TDD Status**: RED phase complete ✓
**Next Step**: Begin GREEN phase implementation
**Goal**: Make all 226 tests pass

