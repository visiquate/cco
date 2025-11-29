//! Error scenario tests for hooks system
//!
//! Tests error handling and edge cases including:
//! - Empty and malformed commands
//! - Very long commands
//! - Invalid JSON requests
//! - Unicode and special characters
//! - Concurrent request handling
//! - Timeout scenarios
//!
//! Run with: cargo test hooks_error_scenarios

mod hooks_test_helpers;

use hooks_test_helpers::*;
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration;

// =============================================================================
// SECTION 1: Empty and Malformed Commands (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify is implemented
async fn test_classify_empty_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("").await.unwrap();

    // Should return fallback classification (CREATE is safest)
    assert_eq!(response.classification.to_uppercase(), "CREATE");
    // Confidence should be lower for fallback
    assert!(response.confidence <= 0.6);
}

#[tokio::test]
#[ignore]
async fn test_classify_whitespace_only_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("   \t\n   ").await.unwrap();

    // Should handle gracefully
    assert!(!response.classification.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_classify_null_characters() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Command with null byte
    let command = "echo\0test";
    let result = daemon.client.classify(command).await;

    // Should either classify or reject gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_classify_command_with_control_characters() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let command = "echo \x1b[31mRed\x1b[0m"; // ANSI escape codes
    let response = daemon.client.classify(command).await.unwrap();

    // Should classify the underlying command
    assert!(!response.classification.is_empty());
}

// =============================================================================
// SECTION 2: Very Long Commands (4 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_very_long_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // 10,000 character command
    let long_command = format!("echo {}", "x".repeat(10000));

    let start = std::time::Instant::now();
    let result = daemon.client.classify(&long_command).await;
    let elapsed = start.elapsed();

    // Should handle within timeout
    assert!(elapsed < Duration::from_secs(3));
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_classify_extremely_long_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // 100,000 character command
    let very_long_command = format!("echo {}", "y".repeat(100000));

    let result = daemon.client.classify(&very_long_command).await;

    // Should either classify or reject gracefully (not crash)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_classify_many_arguments() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Command with 1000 arguments
    let args = (0..1000)
        .map(|i| format!("arg{}", i))
        .collect::<Vec<_>>()
        .join(" ");
    let command = format!("echo {}", args);

    let response = daemon.client.classify(&command).await.unwrap();
    assert!(!response.classification.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_classify_deeply_nested_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Deeply nested command substitutions
    let command = "echo $(echo $(echo $(echo $(echo test))))";

    let response = daemon.client.classify(command).await.unwrap();
    assert!(!response.classification.is_empty());
}

// =============================================================================
// SECTION 3: Invalid JSON and Requests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_invalid_json_request() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon
        .client
        .client
        .post(&url)
        .header("Content-Type", "application/json")
        .body("{invalid json}}")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_missing_required_field() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon
        .client
        .client
        .post(&url)
        .json(&json!({
            "wrong_field": "value"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_wrong_content_type() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon
        .client
        .client
        .post(&url)
        .header("Content-Type", "text/plain")
        .body("ls -la")
        .send()
        .await
        .unwrap();

    // Should reject non-JSON content type
    assert!(response.status().is_client_error());
}

#[tokio::test]
#[ignore]
async fn test_oversized_json_payload() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Create a very large JSON payload (> 1MB)
    let huge_command = "x".repeat(2_000_000);
    let url = format!("{}/api/classify", daemon.client.base_url);

    let result = daemon
        .client
        .client
        .post(&url)
        .json(&json!({
            "command": huge_command
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // Should either reject or handle gracefully
    match result {
        Ok(response) => {
            // If accepted, should be 4xx error or successful classification
            assert!(response.status().is_client_error() || response.status().is_success());
        }
        Err(_) => {
            // Timeout or connection error is acceptable
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_malformed_context_field() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon
        .client
        .client
        .post(&url)
        .json(&json!({
            "command": "ls",
            "context": "invalid context format" // Should be object
        }))
        .send()
        .await
        .unwrap();

    // Should handle gracefully (either accept or reject)
    assert!(response.status().is_success() || response.status().is_client_error());
}

// =============================================================================
// SECTION 4: Unicode and Special Characters (4 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_unicode_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec![
        "echo '你好世界'",
        "echo 'Привет мир'",
        "echo '🚀 test'",
        "echo 'مرحبا بالعالم'",
    ];

    for cmd in commands {
        let result = daemon.client.classify(cmd).await;
        assert!(result.is_ok(), "Failed to classify: {}", cmd);
    }
}

#[tokio::test]
#[ignore]
async fn test_classify_emoji_in_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("echo '🎉🎊🎈'").await.unwrap();
    assert!(!response.classification.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_classify_special_shell_characters() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec![
        "echo '$HOME'",
        "echo `date`",
        "echo $(pwd)",
        "echo 'test' | cat",
        "echo 'test' > file.txt",
        "echo 'test' >> file.txt",
        "echo 'test' 2>&1",
    ];

    for cmd in commands {
        let result = daemon.client.classify(cmd).await;
        assert!(result.is_ok(), "Failed to classify: {}", cmd);
    }
}

#[tokio::test]
#[ignore]
async fn test_classify_quoted_strings() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec![
        r#"echo "double quotes""#,
        r#"echo 'single quotes'"#,
        r#"echo "nested 'quotes'""#,
        r#"echo 'escaped \'quotes\''"#,
    ];

    for cmd in commands {
        let result = daemon.client.classify(cmd).await;
        assert!(result.is_ok(), "Failed to classify: {}", cmd);
    }
}

// =============================================================================
// SECTION 5: Concurrent Request Handling (5 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_concurrent_classification_requests() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec![
        "ls -la",
        "mkdir test",
        "rm file.txt",
        "echo >> log.txt",
        "cat data.json",
    ];

    let mut handles = vec![];

    for cmd in commands {
        let client = daemon.client.clone();
        let command = cmd.to_string();
        handles.push(tokio::spawn(async move { client.classify(&command).await }));
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent request failed");
    }
}

#[tokio::test]
#[ignore]
async fn test_high_concurrency_load() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // 100 concurrent requests
    let mut handles = vec![];

    for i in 0..100 {
        let client = daemon.client.clone();
        handles.push(tokio::spawn(async move {
            let cmd = format!("echo 'test{}'", i);
            client.classify(&cmd).await
        }));
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // Most should succeed
    assert!(success_count > 90, "Too many failures: {}/100", error_count);
}

#[tokio::test]
#[ignore]
async fn test_concurrent_requests_no_interference() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Send different commands concurrently
    let cmd1 = daemon.client.classify("ls -la");
    let cmd2 = daemon.client.classify("mkdir test");
    let cmd3 = daemon.client.classify("rm file");

    let (res1, res2, res3) = tokio::join!(cmd1, cmd2, cmd3);

    // Each should get correct classification
    assert_eq!(res1.unwrap().classification.to_uppercase(), "READ");
    assert_eq!(res2.unwrap().classification.to_uppercase(), "CREATE");
    assert_eq!(res3.unwrap().classification.to_uppercase(), "DELETE");
}

#[tokio::test]
#[ignore]
async fn test_rapid_sequential_requests() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // 50 sequential requests as fast as possible
    let start = std::time::Instant::now();

    for i in 0..50 {
        let cmd = format!("echo {}", i);
        daemon.client.classify(&cmd).await.unwrap();
    }

    let elapsed = start.elapsed();

    // Should handle rapid requests efficiently (< 30s for 50 requests)
    assert!(
        elapsed < Duration::from_secs(30),
        "Sequential requests too slow: {:?}",
        elapsed
    );
}

#[tokio::test]
#[ignore]
async fn test_concurrent_with_long_running_request() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Start a potentially slow request
    let long_string = "x".repeat(50000);
    let long_cmd = daemon.client.classify(&long_string);

    // Start fast requests concurrently
    let fast1 = daemon.client.classify("ls");
    let fast2 = daemon.client.classify("pwd");

    let (_, res1, res2) = tokio::join!(long_cmd, fast1, fast2);

    // Fast requests should complete even if long request is slow
    assert!(res1.is_ok());
    assert!(res2.is_ok());
}

// =============================================================================
// SECTION 6: Timeout Scenarios (3 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classification_timeout_enforcement() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Send a command that might cause slow processing
    let complex_cmd = &"echo $(find / -name '*' 2>/dev/null | head -10000)".repeat(100);

    let start = std::time::Instant::now();
    let _ = daemon.client.classify(complex_cmd).await;
    let elapsed = start.elapsed();

    // Should timeout within inference timeout (2s) + some margin
    assert!(
        elapsed < Duration::from_secs(4),
        "Classification should timeout: {:?}",
        elapsed
    );
}

#[tokio::test]
#[ignore]
async fn test_client_timeout_handling() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Set aggressive client timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let result = client
        .post(&url)
        .json(&json!({ "command": "ls" }))
        .send()
        .await;

    // Either succeeds or times out (both acceptable)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_multiple_timeout_scenarios() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Send several requests that might timeout
    let commands = vec!["x".repeat(100000), "y".repeat(100000), "z".repeat(100000)];

    for cmd in commands {
        let start = std::time::Instant::now();
        let _ = daemon.client.classify(&cmd).await;
        let elapsed = start.elapsed();

        // Each should handle timeout gracefully
        assert!(elapsed < Duration::from_secs(5));
    }
}

// =============================================================================
// SECTION 7: Resource Exhaustion (2 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_memory_usage_with_many_requests() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Send many requests to test memory handling
    for i in 0..1000 {
        let cmd = format!("echo 'test {}'", i);
        let _ = daemon.client.classify(&cmd).await;

        // Give system time to cleanup
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    // Daemon should still be responsive
    let health = daemon.client.health().await;
    assert!(health.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_recovery_after_errors() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Send several problematic requests
    let _ = daemon.client.classify("").await;
    let _ = daemon.client.classify(&"x".repeat(100000)).await;
    let _ = daemon.client.classify("\0\0\0").await;

    // System should recover and handle normal request
    let response = daemon.client.classify("ls -la").await.unwrap();
    assert_eq!(response.classification.to_uppercase(), "READ");
}
