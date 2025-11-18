//! Integration tests for /api/classify endpoint
//!
//! Tests all aspects of the CRUD classification API endpoint including:
//! - Basic CRUD operation classification (READ, CREATE, UPDATE, DELETE)
//! - Timeout enforcement
//! - Error handling (invalid JSON, malformed requests)
//! - Service unavailability scenarios
//! - Response format validation
//!
//! Run with: cargo test hooks_api_classify

mod hooks_test_helpers;

use hooks_test_helpers::*;
use reqwest::StatusCode;
use serde_json::json;

// =============================================================================
// SECTION 1: Basic CRUD Classification Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_read_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("ls -la").await.unwrap();

    assert_eq!(response.classification.to_uppercase(), "READ");
    assert!(response.confidence > 0.8, "Confidence too low: {}", response.confidence);
    assert!(response.reasoning.is_some());
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_create_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("mkdir newdir").await.unwrap();

    assert_eq!(response.classification.to_uppercase(), "CREATE");
    assert!(response.confidence > 0.7);
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_update_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("echo 'text' >> file.txt").await.unwrap();

    assert_eq!(response.classification.to_uppercase(), "UPDATE");
    assert!(response.confidence > 0.7);
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_delete_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("rm -rf directory").await.unwrap();

    assert_eq!(response.classification.to_uppercase(), "DELETE");
    assert!(response.confidence > 0.8);
}

// =============================================================================
// SECTION 2: Timeout and Performance Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_endpoint_timeout() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Very long/complex command that might take time to classify
    let long_command = "echo ".to_string() + &"x".repeat(10000);

    let start = std::time::Instant::now();
    let result = daemon.client.classify(&long_command).await;
    let elapsed = start.elapsed();

    // Should complete within 2 seconds (inference timeout)
    assert!(elapsed < std::time::Duration::from_secs(3));

    // Should still return a result (fallback if needed)
    assert!(result.is_ok(), "Classify should return fallback on timeout");
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_multiple_requests_concurrently() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec![
        "ls -la",
        "mkdir test",
        "rm file.txt",
        "echo >> log.txt",
        "cat file.txt",
    ];

    let mut handles = vec![];

    for cmd in commands {
        let client = daemon.client.clone();
        let command = cmd.to_string();
        handles.push(tokio::spawn(async move {
            client.classify(&command).await
        }));
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent request failed");
    }
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_response_time_acceptable() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let start = std::time::Instant::now();
    daemon.client.classify("git status").await.unwrap();
    let elapsed = start.elapsed();

    // First request may be slower (model loading), but should still be < 3s
    assert!(elapsed < std::time::Duration::from_secs(3));

    // Subsequent requests should be faster
    let start = std::time::Instant::now();
    daemon.client.classify("git log").await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed < std::time::Duration::from_secs(2));
}

// =============================================================================
// SECTION 3: Error Handling Tests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
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

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_endpoint_missing_command_field() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon.client.client
        .post(&url)
        .json(&json!({
            "context": "test"
            // Missing "command" field
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: serde_json::Value = response.json().await.unwrap();
    assert!(error["error"].as_str().unwrap().contains("command"));
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_endpoint_empty_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("").await.unwrap();

    // Should return fallback classification (CREATE is safest)
    assert_eq!(response.classification.to_uppercase(), "CREATE");
    assert!(response.confidence >= 0.0); // Fallback has low confidence
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_endpoint_classifier_unavailable() {
    let daemon = TestDaemon::with_hooks_disabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon.client.client
        .post(&url)
        .json(&json!({
            "command": "ls -la"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let error: serde_json::Value = response.json().await.unwrap();
    assert!(error["error"].as_str().unwrap().to_lowercase().contains("classifier"));
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_endpoint_wrong_http_method() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);

    // Try GET instead of POST
    let response = daemon.client.client
        .get(&url)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// =============================================================================
// SECTION 4: Response Format Validation (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_response_has_required_fields() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("ls").await.unwrap();

    // Required fields
    assert!(!response.classification.is_empty());
    assert!(response.confidence >= 0.0 && response.confidence <= 1.0);

    // Optional fields may be present
    // reasoning and timestamp are optional
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_response_classification_valid() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("git status").await.unwrap();

    let valid_classifications = vec!["READ", "CREATE", "UPDATE", "DELETE"];
    assert!(
        valid_classifications.contains(&response.classification.to_uppercase().as_str()),
        "Invalid classification: {}",
        response.classification
    );
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_response_confidence_in_range() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let commands = vec!["ls", "mkdir test", "rm file", "echo >> log"];

    for cmd in commands {
        let response = daemon.client.classify(cmd).await.unwrap();
        assert!(
            response.confidence >= 0.0 && response.confidence <= 1.0,
            "Confidence out of range for '{}': {}",
            cmd,
            response.confidence
        );
    }
}

// =============================================================================
// SECTION 5: Special Characters and Edge Cases (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_unicode_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("echo '你好世界' > file.txt").await.unwrap();

    // Should classify correctly despite unicode
    assert_eq!(response.classification.to_uppercase(), "CREATE");
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_command_with_pipes() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("cat file.txt | grep pattern").await.unwrap();

    // Piped read commands are still READ
    assert_eq!(response.classification.to_uppercase(), "READ");
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_command_with_redirects() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Output redirect creates a file
    let response = daemon.client.classify("echo 'test' > output.txt").await.unwrap();
    assert_eq!(response.classification.to_uppercase(), "CREATE");

    // Append redirect updates a file
    let response = daemon.client.classify("echo 'more' >> output.txt").await.unwrap();
    assert_eq!(response.classification.to_uppercase(), "UPDATE");
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_very_long_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Command with 5000 characters
    let long_args = "arg=value ".repeat(500);
    let long_command = format!("echo {}", long_args);

    let start = std::time::Instant::now();
    let response = daemon.client.classify(&long_command).await.unwrap();
    let elapsed = start.elapsed();

    // Should handle gracefully
    assert!(!response.classification.is_empty());
    // Should timeout before 3 seconds
    assert!(elapsed < std::time::Duration::from_secs(3));
}

// =============================================================================
// SECTION 6: Context Parameter Tests (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_with_context() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let url = format!("{}/api/classify", daemon.client.base_url);
    let response = daemon.client.client
        .post(&url)
        .json(&json!({
            "command": "ls -la",
            "context": {
                "cwd": "/home/user",
                "user": "testuser"
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: ClassifyResponse = response.json().await.unwrap();
    assert_eq!(result.classification.to_uppercase(), "READ");
}

#[tokio::test]
#[ignore] // Remove when /api/classify endpoint is implemented
async fn test_classify_without_context() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Context is optional
    let response = daemon.client.classify("ls -la").await.unwrap();
    assert_eq!(response.classification.to_uppercase(), "READ");
}
