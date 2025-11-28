# Claude History Parser Tests - Quick Reference

## Quick Start

```bash
# Run all tests
cargo test --test claude_history_tests

# Run with output
cargo test --test claude_history_tests -- --nocapture

# Run test script
./tests/run_claude_history_tests.sh
```

## Test Categories

| Category | Count | Command |
|----------|-------|---------|
| **All Tests** | 33 | `cargo test --test claude_history_tests` |
| Unit | 4 | `cargo test --test claude_history_tests test_normalize` |
| Integration | 9 | `cargo test --test claude_history_tests test_parse` |
| Performance | 3 | `cargo test --test claude_history_tests benchmark` |
| Error Handling | 5 | `cargo test --test claude_history_tests test_corrupted` |
| Edge Cases | 6 | `cargo test --test claude_history_tests test_zero` |

## Key Test Commands

```bash
# Model normalization
cargo test --test claude_history_tests test_normalize_model_name

# Pricing validation
cargo test --test claude_history_tests test_model_pricing_all_variants

# Parse single file
cargo test --test claude_history_tests test_parse_single_jsonl_file

# Parse multiple files
cargo test --test claude_history_tests test_parse_directory_with_multiple_files

# Error handling
cargo test --test claude_history_tests test_handle_malformed_json_gracefully

# Performance (100 files)
cargo test --test claude_history_tests test_benchmark_parsing_100_files

# Large file (10K messages)
cargo test --test claude_history_tests test_large_file_with_10k_messages

# Cache pricing
cargo test --test claude_history_tests test_cache_tokens_parsing

# Incremental parsing
cargo test --test claude_history_tests test_incremental_parsing_from_offset
```

## Test Files

| File | Purpose |
|------|---------|
| `tests/claude_history_tests.rs` | Main test suite (33 tests) |
| `tests/fixtures/claude_history/*.jsonl` | Test data fixtures |
| `tests/CLAUDE_HISTORY_TEST_GUIDE.md` | Comprehensive documentation |
| `tests/CLAUDE_HISTORY_TEST_SUMMARY.md` | Test summary and metrics |
| `tests/run_claude_history_tests.sh` | Quick test runner script |

## Performance Expectations

| Test | Expected Time |
|------|--------------|
| 100 files | < 5 seconds |
| 10K messages (single file) | < 5 seconds |
| 5K messages (50 files) | < 10 seconds |
| Incremental (no changes) | < 10ms |

## Common Issues

### Test Fails: "assertion failed"
- Check expected vs actual values in output
- Verify pricing table matches current rates
- Check test fixture files not corrupted

### Test Fails: "file not found"
- Run from project root: `cd cco/`
- Check fixtures exist in `tests/fixtures/claude_history/`
- Temp directory cleanup may have failed

### Test Slow
- Check system resources (CPU, memory)
- Verify parallel execution working
- Profile with `cargo test -- --nocapture`

## Test Structure

```rust
#[tokio::test]
async fn test_name() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jsonl");

    // Create test data
    fs::write(&test_file, "...").await.unwrap();

    // Execute
    let metrics = load_claude_project_metrics(...)
        .await
        .unwrap();

    // Assert
    assert_eq!(metrics.messages_count, expected);
}
```

## Coverage Summary

- **Total Tests**: 33
- **Code Coverage**: 100% of public API
- **Lines of Test Code**: ~1,600
- **Test Fixtures**: 6 files
- **Documentation Pages**: 3 guides

## Pricing Table (for reference)

| Model | Input/M | Output/M | Cache Write/M | Cache Read/M |
|-------|---------|----------|--------------|-------------|
| Opus 4 | $15 | $75 | $18.75 | $1.50 |
| Sonnet 4.5 | $3 | $15 | $3.75 | $0.30 |
| Haiku 4.5 | $1 | $5 | $1.25 | $0.10 |
| Synthetic | $0 | $0 | $0 | $0 |

## Quick Test Examples

### Test One Function
```bash
cargo test --test claude_history_tests test_normalize_model_name
```

### Test with Debug Output
```bash
cargo test --test claude_history_tests test_parse_single_jsonl_file -- --nocapture
```

### Test Performance Only
```bash
cargo test --test claude_history_tests benchmark -- --nocapture
```

### Test Error Handling
```bash
cargo test --test claude_history_tests test_corrupted --quiet
cargo test --test claude_history_tests test_missing --quiet
```

## Useful cargo test flags

| Flag | Purpose |
|------|---------|
| `--nocapture` | Show println! output |
| `--quiet` | Minimal output |
| `--no-fail-fast` | Run all tests even if some fail |
| `-- --test-threads=1` | Run tests serially |

## Getting Help

- **Test Guide**: `tests/CLAUDE_HISTORY_TEST_GUIDE.md`
- **Test Summary**: `tests/CLAUDE_HISTORY_TEST_SUMMARY.md`
- **Implementation**: `src/claude_history.rs`
- **Example Usage**: `test_metrics.rs`
