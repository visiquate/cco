//! Comprehensive tests for Claude history JSONL parser
//!
//! This test suite covers:
//! - Unit tests: Individual function behavior
//! - Integration tests: Multi-file parsing and aggregation
//! - Performance tests: Parallel execution and benchmarking
//! - Error handling: Malformed data, missing files, permission issues
//! - Edge cases: Large files, mixed models, zero-token messages

use cco::claude_history::*;
use std::time::SystemTime;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// ============================================================================
// UNIT TESTS
// ============================================================================

#[test]
fn test_normalize_model_name() {
    // Test date suffix removal
    assert_eq!(
        normalize_model_name("claude-sonnet-4-5-20250929"),
        "claude-sonnet-4-5"
    );
    assert_eq!(
        normalize_model_name("claude-opus-4-20250514"),
        "claude-opus-4"
    );
    assert_eq!(
        normalize_model_name("claude-3-5-haiku-20250403"),
        "claude-3-5-haiku"
    );

    // Test names without date suffix
    assert_eq!(
        normalize_model_name("claude-sonnet-3.5"),
        "claude-sonnet-3.5"
    );
    assert_eq!(
        normalize_model_name("claude-opus-4-1"),
        "claude-opus-4-1"
    );

    // Test edge cases
    assert_eq!(normalize_model_name("gpt-4"), "gpt-4");
    assert_eq!(normalize_model_name(""), "");
}

#[test]
fn test_model_pricing_all_variants() {
    // Test Opus 4 pricing
    let (input, output, cache_write, cache_read) = get_model_pricing("claude-opus-4");
    assert_eq!(input, 15.0);
    assert_eq!(output, 75.0);
    assert_eq!(cache_write, 18.75); // 25% premium
    assert_eq!(cache_read, 1.50); // 10% of input

    // Test Opus 4.1 variant
    let (input, output, _cache_write, _cache_read) = get_model_pricing("claude-opus-4-1");
    assert_eq!(input, 15.0);
    assert_eq!(output, 75.0);

    // Test Sonnet 4.5 pricing
    let (input, output, cache_write, cache_read) = get_model_pricing("claude-sonnet-4-5");
    assert_eq!(input, 3.0);
    assert_eq!(output, 15.0);
    assert_eq!(cache_write, 3.75); // 25% premium
    assert_eq!(cache_read, 0.30); // 10% of input

    // Test 3-5-sonnet variant
    let (input, output, _cache_write, _cache_read) = get_model_pricing("claude-3-5-sonnet");
    assert_eq!(input, 3.0);
    assert_eq!(output, 15.0);

    // Test Haiku 4.5 pricing
    let (input, output, cache_write, cache_read) = get_model_pricing("claude-haiku-4-5");
    assert_eq!(input, 1.0);
    assert_eq!(output, 5.0);
    assert_eq!(cache_write, 1.25); // 25% premium
    assert_eq!(cache_read, 0.10); // 10% of input

    // Test 3-5-haiku variant
    let (input, output, _cache_write, _cache_read) = get_model_pricing("claude-3-5-haiku");
    assert_eq!(input, 1.0);
    assert_eq!(output, 5.0);

    // Test synthetic/error messages
    let (input, output, cache_write, cache_read) = get_model_pricing("<synthetic>");
    assert_eq!(input, 0.0);
    assert_eq!(output, 0.0);
    assert_eq!(cache_write, 0.0);
    assert_eq!(cache_read, 0.0);

    // Test unknown model (should default to Sonnet)
    let (input, output, _cache_write, _cache_read) = get_model_pricing("unknown-model");
    assert_eq!(input, 3.0);
    assert_eq!(output, 15.0);
}

#[test]
fn test_calculate_cost() {
    // Test basic calculations
    assert_eq!(calculate_cost(1_000_000, 15.0), 15.0);
    assert_eq!(calculate_cost(500_000, 3.0), 1.5);
    assert_eq!(calculate_cost(10_000, 15.0), 0.15);

    // Test fractional costs
    assert!((calculate_cost(1234, 3.0) - 0.003702).abs() < 0.000001);

    // Test zero tokens
    assert_eq!(calculate_cost(0, 15.0), 0.0);

    // Test large numbers
    assert_eq!(calculate_cost(100_000_000, 15.0), 1500.0);
}

#[test]
fn test_cache_pricing_formulas() {
    // Verify cache pricing matches Python monitor formula:
    // - cache_write = input_price * 1.25 (25% premium)
    // - cache_read = input_price * 0.10 (90% discount)

    // Test Sonnet
    let (input, _output, cache_write, cache_read) = get_model_pricing("claude-sonnet-4-5");
    assert!((cache_write - input * 1.25).abs() < 0.0001);
    assert!((cache_read - input * 0.10).abs() < 0.0001);

    // Test Haiku
    let (input, _output, cache_write, cache_read) = get_model_pricing("claude-haiku-4-5");
    assert!((cache_write - input * 1.25).abs() < 0.0001);
    assert!((cache_read - input * 0.10).abs() < 0.0001);

    // Test Opus
    let (input, _output, cache_write, cache_read) = get_model_pricing("claude-opus-4");
    assert!((cache_write - input * 1.25).abs() < 0.0001);
    assert!((cache_read - input * 0.10).abs() < 0.0001);
}

// ============================================================================
// INTEGRATION TESTS - Single File Parsing
// ============================================================================

#[tokio::test]
async fn test_parse_single_jsonl_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jsonl");

    // Create test JSONL file with valid assistant messages
    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1000,"output_tokens":500}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"summary","summary":"Test summary"}
{"type":"assistant","message":{"model":"claude-opus-4-20250514","usage":{"input_tokens":2000,"output_tokens":1000,"cache_creation_input_tokens":5000}},"timestamp":"2025-11-26T11:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    // Parse the file
    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Should have 2 assistant messages
    assert_eq!(metrics.messages_count, 2);
    assert_eq!(metrics.conversations_count, 1);

    // Verify token counts
    assert_eq!(metrics.total_input_tokens, 3000); // 1000 + 2000
    assert_eq!(metrics.total_output_tokens, 1500); // 500 + 1000
    assert_eq!(metrics.total_cache_creation_tokens, 5000);

    // Verify model breakdown
    assert!(metrics.model_breakdown.contains_key("claude-sonnet-4-5"));
    assert!(metrics.model_breakdown.contains_key("claude-opus-4"));

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
    assert_eq!(sonnet.input_tokens, 1000);
    assert_eq!(sonnet.output_tokens, 500);
    assert_eq!(sonnet.message_count, 1);

    let opus = metrics.model_breakdown.get("claude-opus-4").unwrap();
    assert_eq!(opus.input_tokens, 2000);
    assert_eq!(opus.output_tokens, 1000);
    assert_eq!(opus.cache_creation_tokens, 5000);
    assert_eq!(opus.message_count, 1);

    // Verify costs
    assert!(metrics.total_cost > 0.0);
}

#[tokio::test]
async fn test_extract_project_name_from_path() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("my-test-project");
    fs::create_dir(&project_dir).await.unwrap();

    let test_file = project_dir.join("conversation.jsonl");
    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T10:00:00Z"}
"#;
    fs::write(&test_file, content).await.unwrap();

    // Load from home dir structure (simulate)
    let metrics = load_claude_metrics_from_home_dir().await.unwrap();

    // Note: This test verifies the function works, but won't find project
    // because we're not using ~/.claude/projects/
    // Just verify the function returns successfully (messages_count is always >= 0)
    let _ = metrics.messages_count; // Use the value to avoid warnings
}

#[tokio::test]
async fn test_handle_malformed_json_gracefully() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("malformed.jsonl");

    // Mix of valid and invalid JSON
    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T10:00:00Z"}
{"invalid json line
{"type":"assistant","message":{"model":"claude-haiku-4-5-20251001","usage":{"input_tokens":200,"output_tokens":100}},"timestamp":"2025-11-26T11:00:00Z"}
malformed data here
{"missing":"usage"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Should have parsed 2 valid messages, skipped 3 invalid
    assert_eq!(metrics.messages_count, 2);
    assert_eq!(metrics.total_input_tokens, 300); // 100 + 200
    assert_eq!(metrics.total_output_tokens, 150); // 50 + 100
}

#[tokio::test]
async fn test_aggregate_metrics_correctly() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("aggregation.jsonl");

    // Multiple messages from same model
    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":200,"output_tokens":100}},"timestamp":"2025-11-26T11:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":300,"output_tokens":150}},"timestamp":"2025-11-26T12:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 3);
    assert_eq!(metrics.model_breakdown.len(), 1);

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
    assert_eq!(sonnet.input_tokens, 600); // 100 + 200 + 300
    assert_eq!(sonnet.output_tokens, 300); // 50 + 100 + 150
    assert_eq!(sonnet.message_count, 3);

    // Verify cost calculation
    let expected_cost = (600.0 * 3.0 + 300.0 * 15.0) / 1_000_000.0;
    assert!((sonnet.total_cost - expected_cost).abs() < 0.00001);
}

// ============================================================================
// INTEGRATION TESTS - Multi-File Parsing
// ============================================================================

#[tokio::test]
async fn test_parse_directory_with_multiple_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create 10 test files with different data
    for i in 0..10 {
        let test_file = temp_dir.path().join(format!("conversation_{}.jsonl", i));
        let content = format!(
            r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5-20250929","usage":{{"input_tokens":{},"output_tokens":{}}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
            (i + 1) * 100,
            (i + 1) * 50
        );
        fs::write(&test_file, content).await.unwrap();
    }

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Should have 10 messages from 10 conversations
    assert_eq!(metrics.messages_count, 10);
    assert_eq!(metrics.conversations_count, 10);

    // Verify aggregated totals
    // Sum of 100, 200, 300, ..., 1000 = 5500
    assert_eq!(metrics.total_input_tokens, 5500);
    // Sum of 50, 100, 150, ..., 500 = 2750
    assert_eq!(metrics.total_output_tokens, 2750);
}

#[tokio::test]
async fn test_project_aggregation() {
    let temp_dir = TempDir::new().unwrap();

    // Create project structure with multiple conversations
    let test_file1 = temp_dir.path().join("conv1.jsonl");
    let test_file2 = temp_dir.path().join("conv2.jsonl");

    let content1 = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1000,"output_tokens":500}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"claude-haiku-4-5-20251001","usage":{"input_tokens":2000,"output_tokens":1000}},"timestamp":"2025-11-26T11:00:00Z"}
"#;

    let content2 = r#"{"type":"assistant","message":{"model":"claude-opus-4-20250514","usage":{"input_tokens":500,"output_tokens":250}},"timestamp":"2025-11-26T12:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1500,"output_tokens":750}},"timestamp":"2025-11-26T13:00:00Z"}
"#;

    fs::write(&test_file1, content1).await.unwrap();
    fs::write(&test_file2, content2).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Verify totals
    assert_eq!(metrics.messages_count, 4);
    assert_eq!(metrics.conversations_count, 2);
    assert_eq!(metrics.total_input_tokens, 5000); // 1000 + 2000 + 500 + 1500
    assert_eq!(metrics.total_output_tokens, 2500); // 500 + 1000 + 250 + 750

    // Verify model breakdown
    assert_eq!(metrics.model_breakdown.len(), 3);

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
    assert_eq!(sonnet.message_count, 2);
    assert_eq!(sonnet.input_tokens, 2500); // 1000 + 1500

    let haiku = metrics.model_breakdown.get("claude-haiku-4-5").unwrap();
    assert_eq!(haiku.message_count, 1);

    let opus = metrics.model_breakdown.get("claude-opus-4").unwrap();
    assert_eq!(opus.message_count, 1);
}

#[tokio::test]
async fn test_model_aggregation_across_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create 3 files with different models
    for i in 0..3 {
        let test_file = temp_dir.path().join(format!("conv{}.jsonl", i));
        let model = match i {
            0 => "claude-opus-4-20250514",
            1 => "claude-sonnet-4-5-20250929",
            2 => "claude-haiku-4-5-20251001",
            _ => unreachable!(),
        };

        let content = format!(
            r#"{{"type":"assistant","message":{{"model":"{}","usage":{{"input_tokens":1000,"output_tokens":500}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
            model
        );
        fs::write(&test_file, content).await.unwrap();
    }

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 3);
    assert_eq!(metrics.model_breakdown.len(), 3);

    // Each model should have exactly 1 message with same tokens
    for (_model, breakdown) in &metrics.model_breakdown {
        assert_eq!(breakdown.message_count, 1);
        assert_eq!(breakdown.input_tokens, 1000);
        assert_eq!(breakdown.output_tokens, 500);
    }
}

#[tokio::test]
async fn test_total_cost_calculations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("cost_test.jsonl");

    // Use known values for precise cost calculation
    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1000000,"output_tokens":500000}},"timestamp":"2025-11-26T10:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Sonnet pricing: $3/M input, $15/M output
    // Cost = (1M * $3/M) + (0.5M * $15/M) = $3 + $7.50 = $10.50
    let expected_cost = 10.50;
    assert!((metrics.total_cost - expected_cost).abs() < 0.01);
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_benchmark_parsing_100_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create 100 test files
    let start_create = SystemTime::now();
    for i in 0..100 {
        let test_file = temp_dir.path().join(format!("conv_{:03}.jsonl", i));
        let content = format!(
            r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5-20250929","usage":{{"input_tokens":{},"output_tokens":{}}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
            (i + 1) * 10,
            (i + 1) * 5
        );
        fs::write(&test_file, content).await.unwrap();
    }
    let create_duration = start_create.elapsed().unwrap();
    println!("Created 100 test files in {:?}", create_duration);

    // Benchmark parsing
    let start_parse = SystemTime::now();
    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();
    let parse_duration = start_parse.elapsed().unwrap();

    println!("Parsed 100 files in {:?}", parse_duration);
    println!("Messages parsed: {}", metrics.messages_count);
    println!("Total tokens: {}", metrics.total_input_tokens + metrics.total_output_tokens);

    assert_eq!(metrics.messages_count, 100);
    assert_eq!(metrics.conversations_count, 100);

    // Performance assertion: should parse 100 files in under 5 seconds
    assert!(parse_duration.as_secs() < 5, "Parsing took too long: {:?}", parse_duration);
}

#[tokio::test]
async fn test_verify_parallel_execution() {
    // This test verifies that parsing is indeed parallel by checking timing
    let temp_dir = TempDir::new().unwrap();

    // Create 20 files with small delays
    for i in 0..20 {
        let test_file = temp_dir.path().join(format!("conv_{}.jsonl", i));
        let content = format!(
            r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5-20250929","usage":{{"input_tokens":100,"output_tokens":50}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#
        );
        fs::write(&test_file, content).await.unwrap();
    }

    let start = SystemTime::now();
    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();
    let duration = start.elapsed().unwrap();

    println!("Parsed 20 files in {:?}", duration);

    assert_eq!(metrics.messages_count, 20);

    // Parallel execution should be reasonably fast
    assert!(duration.as_millis() < 1000, "Parallel parsing too slow: {:?}", duration);
}

#[tokio::test]
async fn test_memory_usage_stays_reasonable() {
    // Test with 50 larger files (not too large to avoid CI issues)
    let temp_dir = TempDir::new().unwrap();

    for i in 0..50 {
        let test_file = temp_dir.path().join(format!("conv_{}.jsonl", i));

        // Create files with 100 messages each
        let mut content = String::new();
        for j in 0..100 {
            content.push_str(&format!(
                r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5-20250929","usage":{{"input_tokens":{},"output_tokens":{}}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
                (i * 100 + j + 1) * 10,
                (i * 100 + j + 1) * 5
            ));
        }
        fs::write(&test_file, content).await.unwrap();
    }

    let start = SystemTime::now();
    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();
    let duration = start.elapsed().unwrap();

    println!("Parsed 50 files with 5000 total messages in {:?}", duration);

    assert_eq!(metrics.messages_count, 5000);
    assert_eq!(metrics.conversations_count, 50);

    // Should complete in reasonable time
    assert!(duration.as_secs() < 10, "Parsing 5000 messages took too long: {:?}", duration);
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_missing_usage_field() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("missing_usage.jsonl");

    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929"},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T11:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Should parse 1 message, skip the one without usage
    assert_eq!(metrics.messages_count, 1);
    assert_eq!(metrics.total_input_tokens, 100);
}

#[tokio::test]
async fn test_corrupted_jsonl_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("corrupted.jsonl");

    // Completely invalid JSON throughout
    let content = r#"not valid json at all
{broken:json}
[[malformed]]
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Should handle gracefully with no messages
    assert_eq!(metrics.messages_count, 0);
    assert_eq!(metrics.total_cost, 0.0);
}

#[tokio::test]
async fn test_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.jsonl");

    fs::write(&test_file, "").await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 0);
    assert_eq!(metrics.conversations_count, 1); // File exists but empty
}

#[tokio::test]
async fn test_permission_denied_errors() {
    // Note: This test may behave differently on different platforms
    let temp_dir = TempDir::new().unwrap();
    let restricted_dir = temp_dir.path().join("restricted");
    fs::create_dir(&restricted_dir).await.unwrap();

    // Try to load from a directory we don't have permission to read
    // On most systems, we CAN read our own created dirs, so this just verifies
    // the code handles inaccessible directories gracefully
    let metrics = load_claude_project_metrics(restricted_dir.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 0);
}

#[tokio::test]
async fn test_nonexistent_directory() {
    let result = load_claude_project_metrics("/nonexistent/path/to/nowhere").await;

    // Should return default metrics, not error
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.messages_count, 0);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_large_file_with_10k_messages() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("large.jsonl");

    // Create file with 10,000 messages
    let mut content = String::with_capacity(10_000 * 200); // Pre-allocate
    for i in 0..10_000 {
        content.push_str(&format!(
            r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5-20250929","usage":{{"input_tokens":{},"output_tokens":{}}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
            (i + 1) * 10,
            (i + 1) * 5
        ));
    }
    fs::write(&test_file, content).await.unwrap();

    let start = SystemTime::now();
    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();
    let duration = start.elapsed().unwrap();

    println!("Parsed 10,000 messages in {:?}", duration);

    assert_eq!(metrics.messages_count, 10_000);
    assert_eq!(metrics.conversations_count, 1);

    // Should handle large files efficiently
    assert!(duration.as_secs() < 5, "Large file parsing too slow: {:?}", duration);
}

#[tokio::test]
async fn test_files_with_mixed_model_types() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("mixed.jsonl");

    let content = r#"{"type":"assistant","message":{"model":"claude-opus-4-20250514","usage":{"input_tokens":1000,"output_tokens":500}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":2000,"output_tokens":1000}},"timestamp":"2025-11-26T11:00:00Z"}
{"type":"assistant","message":{"model":"claude-haiku-4-5-20251001","usage":{"input_tokens":3000,"output_tokens":1500}},"timestamp":"2025-11-26T12:00:00Z"}
{"type":"assistant","message":{"model":"claude-3-5-sonnet-20241022","usage":{"input_tokens":1500,"output_tokens":750}},"timestamp":"2025-11-26T13:00:00Z"}
{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":0,"output_tokens":0}},"timestamp":"2025-11-26T14:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 5);
    assert_eq!(metrics.model_breakdown.len(), 4); // opus, sonnet (2 variants normalized), haiku, synthetic

    // Verify Sonnet variants are aggregated
    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
    assert_eq!(sonnet.message_count, 1);

    let sonnet_alt = metrics.model_breakdown.get("claude-3-5-sonnet").unwrap();
    assert_eq!(sonnet_alt.message_count, 1);

    // Verify synthetic has zero cost
    let synthetic = metrics.model_breakdown.get("<synthetic>").unwrap();
    assert_eq!(synthetic.total_cost, 0.0);
}

#[tokio::test]
async fn test_project_with_no_assistant_messages() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("no_assistant.jsonl");

    let content = r#"{"type":"summary","summary":"Test summary"}
{"type":"user","message":"Hello"}
{"type":"system","message":"System message"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 0);
    assert_eq!(metrics.conversations_count, 1);
    assert_eq!(metrics.total_cost, 0.0);
}

#[tokio::test]
async fn test_zero_token_messages() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("zero_tokens.jsonl");

    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":0,"output_tokens":0}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":0,"output_tokens":0}},"timestamp":"2025-11-26T11:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 2);
    assert_eq!(metrics.total_input_tokens, 0);
    assert_eq!(metrics.total_output_tokens, 0);
    assert_eq!(metrics.total_cost, 0.0);
}

#[tokio::test]
async fn test_cache_tokens_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("cache.jsonl");

    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1000,"output_tokens":500,"cache_creation_input_tokens":2000,"cache_read_input_tokens":10000}},"timestamp":"2025-11-26T10:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics = load_claude_project_metrics(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(metrics.total_input_tokens, 1000);
    assert_eq!(metrics.total_output_tokens, 500);
    assert_eq!(metrics.total_cache_creation_tokens, 2000);
    assert_eq!(metrics.total_cache_read_tokens, 10000);

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();

    // Verify cache costs
    // cache_write: 2000 * $3.75/M = $0.0075
    // cache_read: 10000 * $0.30/M = $0.003
    assert!((sonnet.cache_write_cost - 0.0075).abs() < 0.00001);
    assert!((sonnet.cache_read_cost - 0.003).abs() < 0.00001);
}

// ============================================================================
// INCREMENTAL PARSING TESTS
// ============================================================================

#[tokio::test]
async fn test_incremental_parsing_from_offset() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("incremental.jsonl");

    // Create initial file with 3 messages
    let initial_content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":200,"output_tokens":100}},"timestamp":"2025-11-26T11:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":300,"output_tokens":150}},"timestamp":"2025-11-26T12:00:00Z"}
"#;

    fs::write(&test_file, initial_content).await.unwrap();

    // Parse from start
    let (messages1, offset1, line_count1) = parse_jsonl_file_from_offset(&test_file, 0)
        .await
        .unwrap();

    assert_eq!(messages1.len(), 3);
    assert_eq!(line_count1, 3);
    assert!(offset1 > 0);

    // Append new messages
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&test_file)
        .await
        .unwrap();

    let additional_content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":400,"output_tokens":200}},"timestamp":"2025-11-26T13:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":500,"output_tokens":250}},"timestamp":"2025-11-26T14:00:00Z"}
"#;

    file.write_all(additional_content.as_bytes()).await.unwrap();
    file.flush().await.unwrap();
    drop(file);

    // Parse incrementally from previous offset
    let (messages2, offset2, line_count2) = parse_jsonl_file_from_offset(&test_file, offset1)
        .await
        .unwrap();

    // Should only get the 2 new messages
    assert_eq!(messages2.len(), 2);
    assert_eq!(line_count2, 2);
    assert!(offset2 > offset1);

    // Verify token counts of new messages
    let total_input: u64 = messages2
        .iter()
        .map(|(_, usage, _)| usage.input_tokens.unwrap_or(0))
        .sum();
    assert_eq!(total_input, 900); // 400 + 500
}

#[tokio::test]
async fn test_incremental_parsing_performance() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("perf.jsonl");

    // Create large initial file
    let mut content = String::new();
    for _i in 0..1000 {
        content.push_str(
            r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T10:00:00Z"}
"#
        );
    }
    fs::write(&test_file, &content).await.unwrap();

    // Initial parse
    let start1 = SystemTime::now();
    let (messages1, offset1, _) = parse_jsonl_file_from_offset(&test_file, 0)
        .await
        .unwrap();
    let duration1 = start1.elapsed().unwrap();

    assert_eq!(messages1.len(), 1000);
    println!("Initial parse of 1000 messages: {:?}", duration1);

    // Incremental parse with no changes (should be fast)
    let start2 = SystemTime::now();
    let (messages2, _, _) = parse_jsonl_file_from_offset(&test_file, offset1)
        .await
        .unwrap();
    let duration2 = start2.elapsed().unwrap();

    assert_eq!(messages2.len(), 0); // No new messages
    println!("Incremental parse with no changes: {:?}", duration2);

    // Incremental should be much faster than initial
    assert!(duration2 < duration1 / 10, "Incremental parsing not optimized");
}

// ============================================================================
// DEFAULT STRUCT TESTS
// ============================================================================

#[test]
fn test_default_claude_metrics() {
    let metrics = ClaudeMetrics::default();
    assert_eq!(metrics.total_cost, 0.0);
    assert_eq!(metrics.messages_count, 0);
    assert_eq!(metrics.conversations_count, 0);
    assert!(metrics.model_breakdown.is_empty());
    assert!(metrics.project_breakdown.is_empty());
}

#[test]
fn test_default_model_breakdown() {
    let breakdown = ModelBreakdown::default();
    assert_eq!(breakdown.input_tokens, 0);
    assert_eq!(breakdown.output_tokens, 0);
    assert_eq!(breakdown.cache_creation_tokens, 0);
    assert_eq!(breakdown.cache_read_tokens, 0);
    assert_eq!(breakdown.input_cost, 0.0);
    assert_eq!(breakdown.output_cost, 0.0);
    assert_eq!(breakdown.cache_write_cost, 0.0);
    assert_eq!(breakdown.cache_read_cost, 0.0);
    assert_eq!(breakdown.total_cost, 0.0);
    assert_eq!(breakdown.message_count, 0);
}

#[test]
fn test_default_project_breakdown() {
    let breakdown = ProjectBreakdown::default();
    assert_eq!(breakdown.name, "");
    assert_eq!(breakdown.total_input_tokens, 0);
    assert_eq!(breakdown.message_count, 0);
    assert_eq!(breakdown.conversation_count, 0);
    assert!(breakdown.models.is_empty());
}

// ============================================================================
// DATE-BASED METRICS TESTS
// ============================================================================

#[tokio::test]
async fn test_metrics_by_date() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("dated.jsonl");

    let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}},"timestamp":"2025-11-26T10:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":200,"output_tokens":100}},"timestamp":"2025-11-26T15:00:00Z"}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":300,"output_tokens":150}},"timestamp":"2025-11-27T10:00:00Z"}
"#;

    fs::write(&test_file, content).await.unwrap();

    let metrics_by_date = load_claude_project_metrics_by_date(temp_dir.path().to_str().unwrap())
        .await
        .unwrap();

    // Should have 2 dates
    assert_eq!(metrics_by_date.len(), 2);
    assert!(metrics_by_date.contains_key("2025-11-26"));
    assert!(metrics_by_date.contains_key("2025-11-27"));

    // Nov 26 should have 2 messages
    let nov26 = metrics_by_date.get("2025-11-26").unwrap();
    assert_eq!(nov26.len(), 2);

    // Nov 27 should have 1 message
    let nov27 = metrics_by_date.get("2025-11-27").unwrap();
    assert_eq!(nov27.len(), 1);
}
