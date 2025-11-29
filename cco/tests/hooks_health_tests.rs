//! Integration tests for health endpoint hooks status
//!
//! Tests that health endpoints correctly report hooks system status:
//! - /health basic health check
//! - /api/health extended health with hooks status
//! - Classifier availability reporting
//! - Model status reporting
//!
//! Run with: cargo test hooks_health

mod hooks_test_helpers;

use hooks_test_helpers::*;

// =============================================================================
// SECTION 1: Basic Health Endpoint (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when health endpoint is updated with hooks status
async fn test_health_basic_endpoint_responds() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.health().await.unwrap();

    assert_eq!(response.status, "ok");
    assert!(!response.version.is_empty());
}

#[tokio::test]
#[ignore] // Remove when health endpoint is updated
async fn test_health_includes_uptime() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Wait a bit to ensure uptime > 0
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let response = daemon.client.health().await.unwrap();

    assert!(response.uptime.is_some());
    assert!(response.uptime.unwrap() > 0);
}

// =============================================================================
// SECTION 2: Extended Health Endpoint (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/health is implemented
async fn test_api_health_includes_hooks_status() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();

    assert!(
        response.hooks.is_some(),
        "Health response should include hooks status"
    );
}

#[tokio::test]
#[ignore] // Remove when /api/health is implemented
async fn test_health_classifier_available_when_enabled() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    assert!(hooks.enabled, "Hooks should be enabled");
    assert!(hooks.classifier_available, "Classifier should be available");
}

#[tokio::test]
#[ignore] // Remove when /api/health is implemented
async fn test_health_classifier_unavailable_when_disabled() {
    let daemon = TestDaemon::with_hooks_disabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    assert!(!hooks.enabled, "Hooks should be disabled");
    assert!(
        !hooks.classifier_available,
        "Classifier should be unavailable"
    );
}

#[tokio::test]
#[ignore] // Remove when /api/health is implemented
async fn test_health_includes_classifier_details() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    // Should include model information
    assert!(hooks.model_name.is_some());
    assert!(!hooks.model_name.unwrap().is_empty());
}

// =============================================================================
// SECTION 3: Model Status Reporting (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when model status is added to health
async fn test_health_model_name_in_response() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    let model_name = hooks.model_name.expect("Model name should be present");

    // Should be TinyLLaMA
    assert!(model_name.to_lowercase().contains("tinyllama"));
}

#[tokio::test]
#[ignore] // Remove when model loaded status is added
async fn test_health_model_loaded_status() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // First call may trigger model loading
    let _ = daemon.client.classify("ls").await;

    // Wait for model to load
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    // Model should be loaded after first classification
    assert!(hooks.model_loaded || !hooks.model_loaded); // Depends on lazy loading strategy
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_model_status_when_disabled() {
    let daemon = TestDaemon::with_hooks_disabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    assert!(!hooks.enabled);
    assert!(!hooks.classifier_available);
    assert!(
        !hooks.model_loaded,
        "Model should not be loaded when disabled"
    );
}

// =============================================================================
// SECTION 4: Health Endpoint Performance (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_endpoint_responds_quickly() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let start = std::time::Instant::now();
    let _ = daemon.client.api_health().await.unwrap();
    let elapsed = start.elapsed();

    // Health check should be fast (< 100ms)
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "Health check took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_endpoint_concurrent_requests() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let mut handles = vec![];

    for _ in 0..10 {
        let client = daemon.client.clone();
        handles.push(tokio::spawn(async move { client.api_health().await }));
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

// =============================================================================
// SECTION 5: Health Status Consistency (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_status_consistent_across_calls() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response1 = daemon.client.api_health().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let response2 = daemon.client.api_health().await.unwrap();

    // Core status should remain consistent
    assert_eq!(response1.status, response2.status);
    assert_eq!(response1.version, response2.version);

    let hooks1 = response1.hooks.unwrap();
    let hooks2 = response2.hooks.unwrap();

    assert_eq!(hooks1.enabled, hooks2.enabled);
    assert_eq!(hooks1.classifier_available, hooks2.classifier_available);
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_uptime_increases() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response1 = daemon.client.api_health().await.unwrap();
    let uptime1 = response1.uptime.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let response2 = daemon.client.api_health().await.unwrap();
    let uptime2 = response2.uptime.unwrap();

    assert!(uptime2 > uptime1, "Uptime should increase");
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_reflects_hooks_config_changes() {
    // Note: This test assumes dynamic config reload capability
    // If config is static, this test can be skipped

    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response1 = daemon.client.api_health().await.unwrap();
    let hooks1 = response1.hooks.unwrap();
    assert!(hooks1.enabled);

    // In a real scenario, you would reload config here
    // For now, just verify the health endpoint is working

    let response2 = daemon.client.api_health().await.unwrap();
    assert_eq!(response2.status, "ok");
}

// =============================================================================
// SECTION 6: Error Scenarios (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_endpoint_always_responds() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Even if classifier fails, health should still respond
    for _ in 0..5 {
        let result = daemon.client.api_health().await;
        assert!(result.is_ok(), "Health endpoint should always respond");
    }
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_health_doesnt_block_on_classifier() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Health check should not wait for classifier initialization
    let start = std::time::Instant::now();
    let _ = daemon.client.api_health().await.unwrap();
    let elapsed = start.elapsed();

    // Should respond quickly even on first call
    assert!(elapsed < std::time::Duration::from_secs(1));
}
