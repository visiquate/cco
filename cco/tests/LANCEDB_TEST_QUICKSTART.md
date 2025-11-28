# LanceDB Tests Quick Start

## Run All Tests

```bash
cargo test --test knowledge_lancedb_tests
```

## Run By Category

```bash
# VFS Storage (4 tests)
cargo test --test knowledge_lancedb_tests vfs_storage_tests

# File Permissions - Unix only (3 tests)
cargo test --test knowledge_lancedb_tests file_permission_tests

# Data Persistence (3 tests)
cargo test --test knowledge_lancedb_tests data_persistence_tests

# Repository Isolation (5 tests)
cargo test --test knowledge_lancedb_tests repository_isolation_tests

# Vector Search (6 tests)
cargo test --test knowledge_lancedb_tests vector_search_tests

# Statistics (7 tests)
cargo test --test knowledge_lancedb_tests statistics_tests

# Edge Cases (10 tests)
cargo test --test knowledge_lancedb_tests edge_case_tests

# Integration - requires daemon (5 tests)
cargo test --test knowledge_lancedb_tests integration_tests -- --ignored
```

## Run Single Test

```bash
cargo test --test knowledge_lancedb_tests test_database_stored_in_correct_directory
```

## With Output

```bash
cargo test --test knowledge_lancedb_tests -- --nocapture
```

## With Logging

```bash
RUST_LOG=debug cargo test --test knowledge_lancedb_tests -- --nocapture
```

## Test Summary

- **Total Tests:** 43
  - Active: 38
  - Ignored (integration): 5

- **Categories:**
  - VFS Storage: 4
  - Permissions: 3 (Unix)
  - Persistence: 3
  - Isolation: 5
  - Search: 6
  - Statistics: 7
  - Edge Cases: 10
  - Integration: 5 (ignored)

## Files Created

1. `tests/knowledge_lancedb_tests.rs` - Main test suite
2. `tests/fixtures/test_knowledge.json` - Sample data
3. `tests/fixtures/test_vectors.json` - Vector specs
4. `tests/LANCEDB_TEST_GUIDE.md` - Full documentation
5. `tests/LANCEDB_TEST_REPORT.md` - Implementation report
6. `tests/LANCEDB_TEST_QUICKSTART.md` - This file

## Next Steps

1. Rust specialist implements LanceDB integration
2. Run tests to validate implementation
3. Fix any failures
4. Run integration tests with daemon
5. Generate coverage report

## Coverage Report

```bash
cargo tarpaulin --test knowledge_lancedb_tests --out Html
open tarpaulin-report.html
```
