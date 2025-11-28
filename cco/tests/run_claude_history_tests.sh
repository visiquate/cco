#!/bin/bash
# Quick test runner for Claude history parser tests

set -e

echo "=========================================="
echo "Claude History Parser Test Suite"
echo "=========================================="
echo ""

# Unit tests
echo "Running unit tests..."
cargo test --test claude_history_tests test_normalize_model_name --quiet
cargo test --test claude_history_tests test_model_pricing_all_variants --quiet
cargo test --test claude_history_tests test_calculate_cost --quiet
cargo test --test claude_history_tests test_cache_pricing_formulas --quiet
echo "✓ Unit tests passed"
echo ""

# Integration tests (single file)
echo "Running single-file integration tests..."
cargo test --test claude_history_tests test_parse_single_jsonl_file --quiet
cargo test --test claude_history_tests test_handle_malformed_json_gracefully --quiet
cargo test --test claude_history_tests test_aggregate_metrics_correctly --quiet
echo "✓ Single-file integration tests passed"
echo ""

# Integration tests (multi-file)
echo "Running multi-file integration tests..."
cargo test --test claude_history_tests test_parse_directory_with_multiple_files --quiet
cargo test --test claude_history_tests test_project_aggregation --quiet
cargo test --test claude_history_tests test_model_aggregation_across_files --quiet
echo "✓ Multi-file integration tests passed"
echo ""

# Error handling
echo "Running error handling tests..."
cargo test --test claude_history_tests test_missing_usage_field --quiet
cargo test --test claude_history_tests test_corrupted_jsonl_file --quiet
cargo test --test claude_history_tests test_empty_file --quiet
cargo test --test claude_history_tests test_nonexistent_directory --quiet
echo "✓ Error handling tests passed"
echo ""

# Edge cases
echo "Running edge case tests..."
cargo test --test claude_history_tests test_files_with_mixed_model_types --quiet
cargo test --test claude_history_tests test_zero_token_messages --quiet
cargo test --test claude_history_tests test_cache_tokens_parsing --quiet
echo "✓ Edge case tests passed"
echo ""

# Performance tests
echo "Running performance tests..."
echo "(These may take a minute...)"
cargo test --test claude_history_tests test_benchmark_parsing_100_files --quiet
cargo test --test claude_history_tests test_verify_parallel_execution --quiet
echo "✓ Performance tests passed"
echo ""

echo "=========================================="
echo "All tests passed!"
echo "=========================================="
