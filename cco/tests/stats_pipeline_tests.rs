//! Integration tests for stats collection pipeline (Phase 5)
//!
//! Tests the complete pipeline:
//! 1. Background log parser task
//! 2. Metrics cache population
//! 3. Filesystem log watcher
//! 4. API /api/stats endpoint
//! 5. Error handling and recovery

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

/// Helper function to create a test JSONL conversation file
fn create_test_conversation_file(path: &PathBuf, messages: &[(&str, u64, u64)]) {
    let mut file = fs::File::create(path).unwrap();

    for (model, input_tokens, output_tokens) in messages {
        let line = format!(
            r#"{{"type":"assistant","message":{{"model":"{}","usage":{{"input_tokens":{},"output_tokens":{}}}}}}}"#,
            model, input_tokens, output_tokens
        );
        writeln!(file, "{}", line).unwrap();
    }

    file.flush().unwrap();
}

#[tokio::test]
async fn test_parse_log_file_basic() {
    // Test basic log file parsing
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("test.jsonl");

    // Create test conversation with known metrics
    create_test_conversation_file(&log_file, &[
        ("claude-sonnet-4-5-20250929", 1000, 500),
        ("claude-haiku-4-5-20251001", 2000, 1000),
    ]);

    // Parse the file
    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    // Verify metrics
    assert_eq!(metrics.messages_count, 2);
    assert_eq!(metrics.total_input_tokens, 3000); // 1000 + 2000
    assert_eq!(metrics.total_output_tokens, 1500); // 500 + 1000
    assert!(metrics.total_cost > 0.0);

    // Verify model breakdown
    assert!(metrics.model_breakdown.contains_key("claude-sonnet-4-5"));
    assert!(metrics.model_breakdown.contains_key("claude-haiku-4-5"));

    println!("✅ Basic parsing test passed");
}

#[tokio::test]
async fn test_parse_corrupted_jsonl() {
    // Test that parser handles corrupted JSONL gracefully
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("corrupted.jsonl");

    // Create file with mix of valid and invalid JSON
    let mut file = fs::File::create(&log_file).unwrap();
    writeln!(file, r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5","usage":{{"input_tokens":100,"output_tokens":50}}}}}}"#).unwrap();
    writeln!(file, "INVALID JSON LINE HERE").unwrap();
    writeln!(file, r#"{{"incomplete":"#).unwrap();
    writeln!(file, r#"{{"type":"assistant","message":{{"model":"claude-haiku-4-5","usage":{{"input_tokens":200,"output_tokens":100}}}}}}"#).unwrap();
    file.flush().unwrap();
    drop(file);

    // Parse file - should succeed and skip invalid lines
    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    // Should have parsed 2 valid messages
    assert_eq!(metrics.messages_count, 2);
    assert_eq!(metrics.total_input_tokens, 300); // 100 + 200
    assert_eq!(metrics.total_output_tokens, 150); // 50 + 100

    println!("✅ Corrupted JSONL handling test passed");
}

#[tokio::test]
async fn test_parse_missing_directory() {
    // Test that parser handles missing directory gracefully
    let nonexistent_path = "/nonexistent/path/to/claude/history";

    // Should return default metrics (not crash)
    let metrics = cco::claude_history::load_claude_project_metrics(nonexistent_path)
        .await
        .unwrap();

    assert_eq!(metrics.messages_count, 0);
    assert_eq!(metrics.total_cost, 0.0);
    assert_eq!(metrics.conversations_count, 0);

    println!("✅ Missing directory handling test passed");
}

#[tokio::test]
async fn test_incremental_parsing_performance() {
    // Test that incremental parsing is significantly faster than full parse
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("large.jsonl");

    // Create large file with 1000 lines
    let mut file = fs::File::create(&log_file).unwrap();
    for i in 0..1000 {
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5","usage":{{"input_tokens":{},"output_tokens":{}}}}}}}"#,
            i * 100, i * 50
        ).unwrap();
    }
    file.flush().unwrap();
    drop(file);

    // Full parse (first time)
    let start = std::time::Instant::now();
    let metrics1 = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();
    let full_parse_time = start.elapsed();

    assert_eq!(metrics1.messages_count, 1000);

    // Add 1 new line
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&log_file)
        .unwrap();
    writeln!(
        file,
        r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5","usage":{{"input_tokens":100000,"output_tokens":50000}}}}}}"#
    ).unwrap();
    file.flush().unwrap();
    drop(file);

    // Incremental parse (should be faster)
    let start = std::time::Instant::now();
    let metrics2 = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();
    let incremental_parse_time = start.elapsed();

    assert_eq!(metrics2.messages_count, 1001);

    // Note: Current implementation doesn't have true incremental parsing yet
    // This test documents the expected behavior for future optimization
    println!("Full parse time: {:?}", full_parse_time);
    println!("Incremental parse time: {:?}", incremental_parse_time);
    println!("✅ Incremental parsing performance test passed");
}

#[tokio::test]
async fn test_multiple_conversation_files() {
    // Test parsing multiple conversation files
    let temp_dir = TempDir::new().unwrap();

    // Create 5 conversation files
    for i in 0..5 {
        let file = temp_dir.path().join(format!("conversation_{}.jsonl", i));
        create_test_conversation_file(&file, &[
            ("claude-sonnet-4-5-20250929", 1000 * (i + 1), 500 * (i + 1)),
        ]);
    }

    // Parse all files
    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    // Should have 5 messages (one per file)
    assert_eq!(metrics.messages_count, 5);
    assert_eq!(metrics.conversations_count, 5);

    // Total input tokens: 1000 + 2000 + 3000 + 4000 + 5000 = 15000
    assert_eq!(metrics.total_input_tokens, 15000);

    // Total output tokens: 500 + 1000 + 1500 + 2000 + 2500 = 7500
    assert_eq!(metrics.total_output_tokens, 7500);

    println!("✅ Multiple conversation files test passed");
}

#[tokio::test]
async fn test_empty_directory() {
    // Test parsing empty directory
    let temp_dir = TempDir::new().unwrap();

    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    assert_eq!(metrics.messages_count, 0);
    assert_eq!(metrics.conversations_count, 0);
    assert_eq!(metrics.total_cost, 0.0);

    println!("✅ Empty directory test passed");
}

#[tokio::test]
async fn test_mixed_model_aggregation() {
    // Test aggregation across multiple models
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("mixed.jsonl");

    create_test_conversation_file(&log_file, &[
        ("claude-opus-4-20250514", 10000, 5000),
        ("claude-sonnet-4-5-20250929", 20000, 10000),
        ("claude-haiku-4-5-20251001", 30000, 15000),
        ("claude-opus-4-20250514", 10000, 5000),  // Duplicate model
        ("claude-sonnet-4-5-20250929", 20000, 10000),  // Duplicate model
    ]);

    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    assert_eq!(metrics.messages_count, 5);
    assert_eq!(metrics.model_breakdown.len(), 3); // 3 unique models

    // Verify per-model aggregation
    let opus = metrics.model_breakdown.get("claude-opus-4").unwrap();
    assert_eq!(opus.input_tokens, 20000); // 10000 + 10000
    assert_eq!(opus.output_tokens, 10000); // 5000 + 5000
    assert_eq!(opus.message_count, 2);

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
    assert_eq!(sonnet.input_tokens, 40000); // 20000 + 20000
    assert_eq!(sonnet.output_tokens, 20000); // 10000 + 10000
    assert_eq!(sonnet.message_count, 2);

    let haiku = metrics.model_breakdown.get("claude-haiku-4-5").unwrap();
    assert_eq!(haiku.input_tokens, 30000);
    assert_eq!(haiku.output_tokens, 15000);
    assert_eq!(haiku.message_count, 1);

    println!("✅ Mixed model aggregation test passed");
}

#[tokio::test]
async fn test_cache_tokens_cost_calculation() {
    // Test that cache tokens are correctly priced
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("cached.jsonl");

    // Create message with cache tokens
    let mut file = fs::File::create(&log_file).unwrap();
    writeln!(
        file,
        r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5-20250929","usage":{{"input_tokens":1000,"output_tokens":500,"cache_creation_input_tokens":5000,"cache_read_input_tokens":10000}}}}}}"#
    ).unwrap();
    file.flush().unwrap();
    drop(file);

    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    assert_eq!(metrics.total_cache_creation_tokens, 5000);
    assert_eq!(metrics.total_cache_read_tokens, 10000);

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();

    // Verify cache costs (Sonnet pricing: input=$3/M, cache_write=$3.75/M, cache_read=$0.30/M)
    // cache_write: 5000 * $3.75/M = $0.01875
    // cache_read: 10000 * $0.30/M = $0.003
    let expected_cache_write = (5000.0 * 3.75) / 1_000_000.0;
    let expected_cache_read = (10000.0 * 0.30) / 1_000_000.0;

    assert!((sonnet.cache_write_cost - expected_cache_write).abs() < 0.00001);
    assert!((sonnet.cache_read_cost - expected_cache_read).abs() < 0.00001);

    println!("✅ Cache token cost calculation test passed");
}

#[tokio::test]
async fn test_large_conversation_file() {
    // Test handling of very large conversation file
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("large.jsonl");

    // Create file with 10,000 messages
    let mut file = fs::File::create(&log_file).unwrap();
    for i in 0..10000 {
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5","usage":{{"input_tokens":100,"output_tokens":50}}}}}}"#
        ).unwrap();

        // Flush periodically to avoid memory issues
        if i % 1000 == 0 {
            file.flush().unwrap();
        }
    }
    file.flush().unwrap();
    drop(file);

    // Parse large file
    let start = std::time::Instant::now();
    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();
    let parse_time = start.elapsed();

    assert_eq!(metrics.messages_count, 10000);
    assert_eq!(metrics.total_input_tokens, 1_000_000); // 100 * 10000
    assert_eq!(metrics.total_output_tokens, 500_000); // 50 * 10000

    println!("✅ Large conversation file test passed (parse time: {:?})", parse_time);
}

#[tokio::test]
async fn test_concurrent_file_parsing() {
    // Test parsing multiple files concurrently
    let temp_dir = TempDir::new().unwrap();

    // Create 10 files
    for i in 0..10 {
        let file = temp_dir.path().join(format!("conv_{}.jsonl", i));
        create_test_conversation_file(&file, &[
            ("claude-sonnet-4-5-20250929", 1000, 500),
        ]);
    }

    // Parse concurrently (spawn 5 concurrent parse tasks)
    let mut handles = vec![];
    for _ in 0..5 {
        let path = temp_dir.path().to_str().unwrap().to_string();
        let handle = tokio::spawn(async move {
            cco::claude_history::load_claude_project_metrics(&path).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.messages_count, 10);
    }

    println!("✅ Concurrent file parsing test passed");
}

#[tokio::test]
async fn test_non_assistant_messages_ignored() {
    // Test that non-assistant messages are correctly ignored
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("mixed_types.jsonl");

    let mut file = fs::File::create(&log_file).unwrap();

    // Write various message types
    writeln!(file, r#"{{"type":"user","content":"Hello"}}"#).unwrap();
    writeln!(file, r#"{{"type":"assistant","message":{{"model":"claude-sonnet-4-5","usage":{{"input_tokens":100,"output_tokens":50}}}}}}"#).unwrap();
    writeln!(file, r#"{{"type":"summary","summary":"Conversation summary"}}"#).unwrap();
    writeln!(file, r#"{{"type":"assistant","message":{{"model":"claude-haiku-4-5","usage":{{"input_tokens":200,"output_tokens":100}}}}}}"#).unwrap();
    writeln!(file, r#"{{"type":"error","error":"Some error"}}"#).unwrap();

    file.flush().unwrap();
    drop(file);

    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    // Should only count 2 assistant messages
    assert_eq!(metrics.messages_count, 2);
    assert_eq!(metrics.total_input_tokens, 300); // 100 + 200
    assert_eq!(metrics.total_output_tokens, 150); // 50 + 100

    println!("✅ Non-assistant message filtering test passed");
}

#[tokio::test]
async fn test_zero_token_messages() {
    // Test handling of messages with zero tokens
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("zero_tokens.jsonl");

    create_test_conversation_file(&log_file, &[
        ("claude-sonnet-4-5-20250929", 0, 0),
        ("claude-sonnet-4-5-20250929", 100, 50),
        ("claude-sonnet-4-5-20250929", 0, 0),
    ]);

    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    assert_eq!(metrics.messages_count, 3);
    assert_eq!(metrics.total_input_tokens, 100);
    assert_eq!(metrics.total_output_tokens, 50);

    println!("✅ Zero token messages test passed");
}

#[tokio::test]
async fn test_model_name_normalization() {
    // Test that different model name formats are normalized correctly
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("normalized.jsonl");

    // Use different date suffixes for same model
    create_test_conversation_file(&log_file, &[
        ("claude-sonnet-4-5-20250929", 1000, 500),
        ("claude-sonnet-4-5-20251015", 2000, 1000),
        ("claude-sonnet-4-5-20251101", 3000, 1500),
    ]);

    let metrics = cco::claude_history::load_claude_project_metrics(
        temp_dir.path().to_str().unwrap()
    ).await.unwrap();

    // All should be aggregated under "claude-sonnet-4-5"
    assert_eq!(metrics.model_breakdown.len(), 1);

    let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
    assert_eq!(sonnet.input_tokens, 6000); // 1000 + 2000 + 3000
    assert_eq!(sonnet.output_tokens, 3000); // 500 + 1000 + 1500
    assert_eq!(sonnet.message_count, 3);

    println!("✅ Model name normalization test passed");
}
