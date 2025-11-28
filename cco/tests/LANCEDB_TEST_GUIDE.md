# LanceDB Knowledge Store Test Guide

## Overview

This document provides comprehensive guidance for running and maintaining the LanceDB knowledge store test suite. The tests validate all requirements for the embedded vector database implementation.

## Test File Structure

```
tests/
├── knowledge_lancedb_tests.rs      # Main comprehensive test suite
├── knowledge_store_tests.rs         # Original TDD tests (in-memory)
├── knowledge_store_integration_tests.rs  # Integration tests
├── fixtures/
│   ├── test_knowledge.json         # Sample knowledge items
│   └── test_vectors.json           # Test vector specifications
└── LANCEDB_TEST_GUIDE.md           # This file
```

## Running Tests

### Run All LanceDB Tests

```bash
# Run the entire LanceDB test suite
cargo test --test knowledge_lancedb_tests

# Run with output visible
cargo test --test knowledge_lancedb_tests -- --nocapture

# Run with logging enabled
RUST_LOG=debug cargo test --test knowledge_lancedb_tests -- --nocapture
```

### Run Specific Test Modules

```bash
# VFS storage tests
cargo test --test knowledge_lancedb_tests vfs_storage_tests

# File permissions (Unix only)
cargo test --test knowledge_lancedb_tests file_permission_tests

# Data persistence
cargo test --test knowledge_lancedb_tests data_persistence_tests

# Repository isolation
cargo test --test knowledge_lancedb_tests repository_isolation_tests

# Vector search
cargo test --test knowledge_lancedb_tests vector_search_tests

# Statistics
cargo test --test knowledge_lancedb_tests statistics_tests

# Edge cases
cargo test --test knowledge_lancedb_tests edge_case_tests
```

### Run Specific Tests

```bash
# Run a single test by name
cargo test --test knowledge_lancedb_tests test_database_stored_in_correct_directory

# Run tests matching a pattern
cargo test --test knowledge_lancedb_tests concurrent
```

### Integration Tests (Requires Daemon)

```bash
# Integration tests are ignored by default
# Run them explicitly when daemon is available
cargo test --test knowledge_lancedb_tests integration_tests -- --ignored
```

## Test Categories

### 1. VFS Storage Tests (4 tests)

Tests that validate database storage location and directory structure.

**What they test:**
- Database stored in `~/.cco/knowledge/{repo_name}/`
- Proper directory structure creation
- Cleanup functionality
- Path generation from repository names

**Key assertions:**
- Expected directory paths exist
- Nested structure created correctly
- Repository name extraction works

**Example:**
```rust
#[tokio::test]
async fn test_database_stored_in_correct_directory() {
    let store = create_test_store_with_base(base_path, "test-repo").await;
    let expected_path = base_path.join(".cco").join("knowledge").join("test-repo");
    assert!(expected_path.exists());
}
```

### 2. File Permission Tests (3 tests - Unix only)

Tests file system security through proper permission settings.

**What they test:**
- Directories have `0o700` (owner-only rwx)
- Files have `0o600` (owner-only rw)
- Recursive permission setting

**Key assertions:**
- Directory mode is exactly `0o700`
- File mode is at most `0o600`
- All levels of hierarchy have secure permissions

**Platform-specific:**
- Only runs on Unix systems (`#[cfg(unix)]`)
- Uses `std::os::unix::fs::PermissionsExt`

**Example:**
```rust
#[cfg(unix)]
#[tokio::test]
async fn test_directory_permissions_owner_only() {
    let metadata = fs::metadata(&db_path).await?;
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
}
```

### 3. Data Persistence Tests (3 tests)

Tests that data survives across daemon restarts and sessions.

**What they test:**
- Data persists after store instance dropped
- Multiple sessions accumulate data correctly
- Database recovers gracefully from crashes

**Key assertions:**
- Data accessible after restart
- Record counts accumulate correctly
- Database doesn't corrupt on unexpected shutdown

**Example:**
```rust
#[tokio::test]
async fn test_data_persists_after_restart() {
    // Phase 1: Store data
    { let mut store = create_store(); store.store(data).await?; }

    // Phase 2: Create new store and verify
    { let store = create_store(); assert!(store.get_stats().await?.total_records > 0); }
}
```

### 4. Repository Isolation Tests (5 tests)

Tests that different repositories maintain separate data spaces.

**What they test:**
- Multiple repos don't leak data between each other
- Concurrent access to different repos works
- `project_id` filtering provides proper isolation
- Separate vector spaces per repository

**Key assertions:**
- Each repo sees only its own data
- Stats show correct repository name
- Concurrent operations don't interfere

**Example:**
```rust
#[tokio::test]
async fn test_multiple_repo_isolation() {
    let store_a = create_store("repo-a").await;
    let store_b = create_store("repo-b").await;

    store_a.store(data_a).await?;
    store_b.store(data_b).await?;

    assert_eq!(store_a.get_stats().await?.repository, "repo-a");
    assert_eq!(store_b.get_stats().await?.repository, "repo-b");
}
```

### 5. Vector Search Tests (6 tests)

Tests semantic similarity search and filtering capabilities.

**What they test:**
- Basic vector similarity search
- Search limit parameter respected
- Filtering by `project_id`
- Filtering by `knowledge_type`
- Filtering by `agent`

**Key assertions:**
- Search returns relevant results
- Limit parameter caps results
- Filters correctly restrict results
- Results match filter criteria

**Example:**
```rust
#[tokio::test]
async fn test_filter_by_project_id() {
    store.store(item_alpha).await?;
    store.store(item_beta).await?;

    let results = store.search(SearchRequest {
        project_id: Some("alpha".to_string()),
        ..default
    }).await?;

    for result in results {
        assert_eq!(result.project_id, "alpha");
    }
}
```

### 6. Statistics Tests (7 tests)

Tests aggregation and reporting functionality.

**What they test:**
- Total record count accuracy
- Aggregation by knowledge type
- Aggregation by agent
- Aggregation by project
- Oldest record timestamp tracking
- Newest record timestamp tracking

**Key assertions:**
- Counts match stored items
- Aggregations are accurate
- Timestamps are valid RFC3339
- Newest >= Oldest

**Example:**
```rust
#[tokio::test]
async fn test_by_type_aggregation() {
    store.store(decision1).await?;
    store.store(decision2).await?;
    store.store(issue1).await?;

    let stats = store.get_stats().await?;
    assert_eq!(stats.by_type["decision"], 2);
    assert_eq!(stats.by_type["issue"], 1);
}
```

### 7. Edge Case Tests (10 tests)

Tests boundary conditions and error handling.

**What they test:**
- Empty database operations
- Large batch inserts (1000+ items)
- Concurrent writes
- Path traversal attack prevention
- Missing directory creation
- Corrupted database recovery
- Empty text rejection
- Very long text handling (1MB+)
- Special characters and Unicode

**Key assertions:**
- Operations don't panic
- Errors are handled gracefully
- No data corruption
- Security vulnerabilities prevented

**Example:**
```rust
#[tokio::test]
async fn test_large_batch_insert() {
    let requests: Vec<_> = (0..1000).map(|i| create_request(i)).collect();
    let ids = store.store_batch(requests).await?;
    assert_eq!(ids.len(), 1000);
}
```

### 8. Integration Tests (5 tests - Ignored by default)

Tests HTTP API endpoints with full daemon running.

**What they test:**
- POST /api/knowledge/store
- POST /api/knowledge/search
- GET /api/knowledge/stats
- Complete workflow (store → search → stats)
- Compaction resilience

**Running:**
```bash
cargo test --test knowledge_lancedb_tests integration_tests -- --ignored
```

**Requirements:**
- Full daemon server running
- HTTP client (reqwest) configured
- Test authentication if needed

## Test Data Fixtures

### test_knowledge.json

Contains sample knowledge items representing different types:
- Decision: "We decided to use FastAPI..."
- Implementation: "Implemented JWT authentication..."
- Issue: "Security audit found..."
- Configuration: "Configured Docker Compose..."
- Architecture: "The microservices architecture..."
- Credential: "API key for production database..."

Usage in tests:
```rust
let fixture_path = "tests/fixtures/test_knowledge.json";
let items: Vec<StoreKnowledgeRequest> = serde_json::from_str(&fs::read_to_string(fixture_path)?)?;
```

### test_vectors.json

Specifications for embedding tests:
- Expected vector dimensions (384)
- Normalization range ([-1.0, 1.0])
- Sample similarity scores
- Edge case definitions

## Common Issues and Solutions

### Issue: Permission tests fail on non-Unix systems

**Solution:** Permission tests are Unix-only. They're automatically skipped on Windows/macOS via `#[cfg(unix)]`.

### Issue: Integration tests fail with "connection refused"

**Solution:** Integration tests require the daemon to be running. Start it first:
```bash
cargo run --bin cco daemon start
cargo test --test knowledge_lancedb_tests integration_tests -- --ignored
```

### Issue: Tests timeout

**Solution:** Increase timeout for async tests:
```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_slow_operation() { /* ... */ }
```

Or run with increased timeout:
```bash
RUST_TEST_TIMEOUT=300 cargo test
```

### Issue: Temp directory cleanup fails

**Solution:** The `TempDir` type automatically cleans up. If cleanup fails, it's usually a permission issue. Check file permissions or run with sudo (not recommended).

### Issue: LanceDB not installed

**Solution:** LanceDB is included in `Cargo.toml`. If compilation fails:
```bash
cargo clean
cargo build --release
```

## Test Coverage Report

Generate coverage report:
```bash
cargo tarpaulin --test knowledge_lancedb_tests --out Html
open tarpaulin-report.html
```

Current test coverage breakdown:
- VFS Storage: 4 tests
- File Permissions: 3 tests (Unix only)
- Data Persistence: 3 tests
- Repository Isolation: 5 tests
- Vector Search: 6 tests
- Statistics: 7 tests
- Edge Cases: 10 tests
- Integration: 5 tests (ignored)

**Total: 43 tests** (38 active + 5 integration)

## Performance Benchmarking

Run performance benchmarks:
```bash
cargo bench --bench knowledge_store_bench
```

Expected performance targets:
- Store single item: < 10ms
- Store batch (100 items): < 100ms
- Vector search: < 50ms
- Get stats: < 20ms

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: LanceDB Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run LanceDB tests
      run: cargo test --test knowledge_lancedb_tests

    - name: Run integration tests
      if: matrix.os == 'ubuntu-latest'
      run: |
        cargo run --bin cco daemon start &
        sleep 5
        cargo test --test knowledge_lancedb_tests integration_tests -- --ignored
```

## Test Maintenance

### Adding New Tests

1. Identify the test category
2. Add test function to appropriate module
3. Follow naming convention: `test_{what_is_being_tested}`
4. Use helper functions for common setup
5. Add clear assertions with descriptive messages
6. Update this documentation

Example template:
```rust
#[tokio::test]
async fn test_new_feature() {
    // Arrange
    let (_temp_dir, mut store) = create_test_store().await;

    // Act
    let result = store.new_feature().await;

    // Assert
    assert!(result.is_ok(), "New feature should work correctly");
}
```

### Updating Fixtures

When adding new test data:
1. Add to `test_knowledge.json` or `test_vectors.json`
2. Document the purpose in fixture comments
3. Update fixture loading code if structure changes
4. Regenerate any pre-computed values

## Debugging Tests

### Enable Detailed Logging

```bash
RUST_LOG=trace cargo test --test knowledge_lancedb_tests -- --nocapture
```

### Run Single Test with Debugging

```rust
#[tokio::test]
async fn test_debug_issue() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Test code with detailed logging
}
```

### Inspect Temp Directories

Prevent cleanup to inspect files:
```rust
#[tokio::test]
async fn test_inspect_files() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.into_path(); // Prevents automatic cleanup
    println!("Database at: {:?}", path);
    // Inspect files manually before they're deleted
}
```

## Next Steps

After all tests pass:

1. **Rust Specialist**: Implement LanceDB integration to make tests pass
2. **Security Auditor**: Review file permission implementation
3. **QA Engineer**: Run full test suite and report results
4. **DevOps**: Integrate tests into CI/CD pipeline
5. **Documentation**: Update API documentation with test examples

## Contact

For questions or issues with tests:
- Open an issue on GitHub
- Tag `@qa-engineer` or `@rust-specialist`
- Include test name and error output
- Attach logs with `RUST_LOG=debug`

## Appendix: Test Commands Quick Reference

```bash
# Run all tests
cargo test --test knowledge_lancedb_tests

# Run with output
cargo test --test knowledge_lancedb_tests -- --nocapture

# Run specific module
cargo test --test knowledge_lancedb_tests vfs_storage_tests

# Run specific test
cargo test --test knowledge_lancedb_tests test_database_stored_in_correct_directory

# Run integration tests
cargo test --test knowledge_lancedb_tests integration_tests -- --ignored

# Generate coverage
cargo tarpaulin --test knowledge_lancedb_tests --out Html

# Run benchmarks
cargo bench --bench knowledge_store_bench

# Debug with logging
RUST_LOG=debug cargo test --test knowledge_lancedb_tests -- --nocapture
```
