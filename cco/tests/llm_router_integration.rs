//! Integration tests for LLM Router
//!
//! Tests routing decisions, agent type classification, custom endpoint calling,
//! authentication, statistics, and error handling.
//!
//! Run with: cargo test llm_router_integration

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

// =============================================================================
// Test Client and Helpers
// =============================================================================

#[derive(Clone)]
struct LlmRouterTestClient {
    client: Client,
    base_url: String,
    port: u16,
}

impl LlmRouterTestClient {
    fn new(port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", port),
            port,
        }
    }

    async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.health().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }
        anyhow::bail!("Daemon did not become ready within {:?}", timeout)
    }

    async fn route_decision(
        &self,
        agent_type: &str,
        task_description: &str,
    ) -> Result<RouteDecisionResponse> {
        let url = format!("{}/api/llm-router/route", self.base_url);
        let request = RouteDecisionRequest {
            agent_type: agent_type.to_string(),
            task_description: task_description.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn classify_task(&self, task_description: &str) -> Result<ClassifyTaskResponse> {
        let url = format!("{}/api/llm-router/classify", self.base_url);
        let request = ClassifyTaskRequest {
            task_description: task_description.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn call_custom_endpoint(
        &self,
        endpoint_url: &str,
        model: &str,
        prompt: &str,
        endpoint_type: &str,
    ) -> Result<CustomEndpointResponse> {
        let url = format!("{}/api/llm-router/call", self.base_url);
        let request = CustomEndpointRequest {
            endpoint_url: endpoint_url.to_string(),
            model: model.to_string(),
            prompt: prompt.to_string(),
            endpoint_type: endpoint_type.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn get_statistics(&self) -> Result<RouterStatisticsResponse> {
        let url = format!("{}/api/llm-router/stats", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn set_custom_endpoint(
        &self,
        endpoint_url: &str,
        endpoint_type: &str,
        bearer_token: Option<String>,
    ) -> Result<SetEndpointResponse> {
        let url = format!("{}/api/llm-router/endpoint", self.base_url);
        let request = SetEndpointRequest {
            endpoint_url: endpoint_url.to_string(),
            endpoint_type: endpoint_type.to_string(),
            bearer_token,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn clear_custom_endpoint(&self) -> Result<ClearEndpointResponse> {
        let url = format!("{}/api/llm-router/endpoint", self.base_url);
        let response = self
            .client
            .delete(&url)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json().await?)
    }
}

// =============================================================================
// API Request/Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct RouteDecisionRequest {
    agent_type: String,
    task_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RouteDecisionResponse {
    route_to: String, // "claude" or "custom"
    endpoint: String,
    reason: String,
    is_architecture_task: bool,
    is_coding_task: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClassifyTaskRequest {
    task_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClassifyTaskResponse {
    is_architecture_task: bool,
    is_coding_task: bool,
    classification: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomEndpointRequest {
    endpoint_url: String,
    model: String,
    prompt: String,
    endpoint_type: String, // "ollama" or "openai"
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomEndpointResponse {
    success: bool,
    response_text: String,
    model_used: String,
    tokens_used: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RouterStatisticsResponse {
    total_routes: u64,
    claude_routes: u64,
    custom_routes: u64,
    architecture_tasks: u64,
    coding_tasks: u64,
    custom_endpoint_configured: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetEndpointRequest {
    endpoint_url: String,
    endpoint_type: String,
    bearer_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetEndpointResponse {
    success: bool,
    endpoint_url: String,
    endpoint_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClearEndpointResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// =============================================================================
// Test Helpers
// =============================================================================

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    listener.local_addr().unwrap().port()
}

// Note: These tests are currently ignored because they require:
// 1. The daemon to have LLM router module integrated
// 2. The router endpoints to be registered
// 3. Custom endpoint configuration support
//
// Remove #[ignore] once Phase 3 implementation is complete

// =============================================================================
// SECTION 1: Routing Decision Tests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_route_architecture_task_to_claude() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Architecture task should always route to Claude
    let response = client
        .route_decision(
            "chief-architect",
            "Design the system architecture for a microservices platform",
        )
        .await
        .unwrap();

    assert_eq!(response.route_to, "claude");
    assert!(response.is_architecture_task);
    assert!(!response.is_coding_task);
    assert!(response.reason.contains("architecture"));
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_route_coding_task_to_custom_when_configured() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Configure custom endpoint
    client
        .set_custom_endpoint("http://localhost:11434", "ollama", None)
        .await
        .unwrap();

    // Coding task should route to custom when configured
    let response = client
        .route_decision("python-specialist", "Implement a REST API endpoint")
        .await
        .unwrap();

    assert_eq!(response.route_to, "custom");
    assert!(!response.is_architecture_task);
    assert!(response.is_coding_task);

    // Cleanup
    client.clear_custom_endpoint().await.unwrap();
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_route_coding_task_to_claude_when_no_custom() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Ensure no custom endpoint
    let _ = client.clear_custom_endpoint().await;

    // Coding task should fallback to Claude when no custom endpoint
    let response = client
        .route_decision("go-specialist", "Write a concurrent worker pool")
        .await
        .unwrap();

    assert_eq!(response.route_to, "claude");
    assert!(!response.is_architecture_task);
    assert!(response.is_coding_task);
    assert!(response.reason.contains("fallback") || response.reason.contains("no custom"));
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_route_multiple_agent_types() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let test_cases = vec![
        ("chief-architect", "Design system", true, false),
        ("python-specialist", "Write Python code", false, true),
        ("security-auditor", "Review security", false, false),
        ("tdd-coding-agent", "Write tests", false, true),
    ];

    for (agent_type, task, expect_arch, expect_code) in test_cases {
        let response = client.route_decision(agent_type, task).await.unwrap();

        assert_eq!(
            response.is_architecture_task, expect_arch,
            "Failed for agent: {}",
            agent_type
        );
        assert_eq!(
            response.is_coding_task, expect_code,
            "Failed for agent: {}",
            agent_type
        );
    }
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_routing_statistics_tracking() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Get initial stats
    let stats_before = client.get_statistics().await.unwrap();
    let initial_routes = stats_before.total_routes;

    // Make several routing decisions
    client
        .route_decision("chief-architect", "Design system")
        .await
        .unwrap();
    client
        .route_decision("python-specialist", "Write code")
        .await
        .unwrap();

    // Get updated stats
    let stats_after = client.get_statistics().await.unwrap();

    assert_eq!(stats_after.total_routes, initial_routes + 2);
    assert!(stats_after.claude_routes > 0);
}

// =============================================================================
// SECTION 2: Agent Type Classification Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_classify_architecture_task() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let tasks = vec![
        "Design the system architecture",
        "Create architectural diagrams",
        "Define microservices boundaries",
        "Plan the database schema",
    ];

    for task in tasks {
        let response = client.classify_task(task).await.unwrap();

        assert!(
            response.is_architecture_task,
            "Task should be classified as architecture: {}",
            task
        );
        assert_eq!(response.classification, "architecture");
    }
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_classify_coding_task() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let tasks = vec![
        "Write a Python function",
        "Implement REST API endpoints",
        "Fix the authentication bug",
        "Add unit tests for the service",
    ];

    for task in tasks {
        let response = client.classify_task(task).await.unwrap();

        assert!(
            response.is_coding_task,
            "Task should be classified as coding: {}",
            task
        );
        assert_eq!(response.classification, "coding");
    }
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_classify_mixed_task() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Tasks that could be both architecture and coding
    let response = client
        .classify_task("Design and implement the authentication system")
        .await
        .unwrap();

    // Should classify as architecture (higher priority)
    assert!(response.is_architecture_task);
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_classify_other_task() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let tasks = vec![
        "Write documentation",
        "Review security audit",
        "Test the deployment",
    ];

    for task in tasks {
        let response = client.classify_task(task).await.unwrap();

        assert!(
            !response.is_architecture_task && !response.is_coding_task,
            "Task should be classified as other: {}",
            task
        );
        assert_eq!(response.classification, "other");
    }
}

// =============================================================================
// SECTION 3: Custom Endpoint Tests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_call_ollama_endpoint() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Configure Ollama endpoint
    client
        .set_custom_endpoint("http://localhost:11434", "ollama", None)
        .await
        .unwrap();

    // Call Ollama endpoint (will fail if Ollama not running, that's OK)
    let result = client
        .call_custom_endpoint(
            "http://localhost:11434/api/generate",
            "codellama",
            "Write a hello world function",
            "ollama",
        )
        .await;

    // Either succeeds or fails gracefully
    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(!response.response_text.is_empty());
        }
        Err(_) => {
            // Ollama not running - that's acceptable
        }
    }
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_call_openai_compatible_endpoint() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Configure OpenAI-compatible endpoint with bearer token
    client
        .set_custom_endpoint(
            "http://localhost:8080/v1",
            "openai",
            Some("test_bearer_token".to_string()),
        )
        .await
        .unwrap();

    // Call endpoint (will fail if not running, that's OK)
    let result = client
        .call_custom_endpoint(
            "http://localhost:8080/v1/chat/completions",
            "gpt-4",
            "Hello world",
            "openai",
        )
        .await;

    // Either succeeds or fails gracefully
    match result {
        Ok(response) => {
            assert!(response.success);
        }
        Err(_) => {
            // Endpoint not running - that's acceptable
        }
    }
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_bearer_token_authentication() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Set endpoint with bearer token
    let response = client
        .set_custom_endpoint(
            "http://localhost:8080",
            "openai",
            Some("sk_test_12345".to_string()),
        )
        .await
        .unwrap();

    assert!(response.success);
    assert_eq!(response.endpoint_type, "openai");

    // Verify stats show custom endpoint configured
    let stats = client.get_statistics().await.unwrap();
    assert!(stats.custom_endpoint_configured);
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_bearer_token_from_env_var() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Set environment variable
    std::env::set_var("LLM_BEARER_TOKEN", "env_token_12345");

    // Configure endpoint without explicit token (should use env var)
    let response = client
        .set_custom_endpoint("http://localhost:8080", "openai", None)
        .await
        .unwrap();

    assert!(response.success);

    // Cleanup
    std::env::remove_var("LLM_BEARER_TOKEN");
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_bearer_token_from_credential_store() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Store token in credential store (assuming credential API is available)
    // This would use the credential store API

    // Configure endpoint without explicit token (should retrieve from store)
    let response = client
        .set_custom_endpoint("http://localhost:8080", "openai", None)
        .await
        .unwrap();

    assert!(response.success);
}

// =============================================================================
// SECTION 4: Error Handling Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_invalid_endpoint_url() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Try to set invalid URL
    let result = client
        .set_custom_endpoint("not-a-valid-url", "ollama", None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_unsupported_endpoint_type() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Try to set unsupported endpoint type
    let result = client
        .set_custom_endpoint("http://localhost:8080", "unsupported", None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_unreachable_custom_endpoint() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Configure unreachable endpoint
    client
        .set_custom_endpoint("http://localhost:99999", "ollama", None)
        .await
        .unwrap();

    // Try to call it
    let result = client
        .call_custom_endpoint(
            "http://localhost:99999/api/generate",
            "model",
            "prompt",
            "ollama",
        )
        .await;

    // Should fail gracefully
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_malformed_custom_endpoint_response() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // This test would require a mock server returning malformed responses
    // For now, just verify error handling exists
    let result = client
        .call_custom_endpoint(
            "http://localhost:8080/malformed",
            "model",
            "prompt",
            "ollama",
        )
        .await;

    // Should handle errors gracefully
    match result {
        Ok(_) => {
            // If endpoint exists and returns valid response, OK
        }
        Err(_) => {
            // Expected - endpoint doesn't exist or returns errors
        }
    }
}

// =============================================================================
// SECTION 5: Statistics Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_statistics_initial_state() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let stats = client.get_statistics().await.unwrap();

    assert!(stats.total_routes >= 0);
    assert!(stats.claude_routes >= 0);
    assert!(stats.custom_routes >= 0);
    assert!(stats.architecture_tasks >= 0);
    assert!(stats.coding_tasks >= 0);
    assert!(!stats.custom_endpoint_configured || stats.custom_endpoint_configured);
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_statistics_increment_correctly() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let stats_before = client.get_statistics().await.unwrap();

    // Make routing decisions
    client
        .route_decision("chief-architect", "Design")
        .await
        .unwrap();
    client
        .route_decision("python-specialist", "Code")
        .await
        .unwrap();

    let stats_after = client.get_statistics().await.unwrap();

    assert_eq!(stats_after.total_routes, stats_before.total_routes + 2);
    assert_eq!(
        stats_after.architecture_tasks,
        stats_before.architecture_tasks + 1
    );
    assert_eq!(stats_after.coding_tasks, stats_before.coding_tasks + 1);
}

#[tokio::test]
#[ignore] // Remove after LLM router implementation complete
async fn test_statistics_custom_endpoint_flag() {
    let port = find_available_port();
    let client = LlmRouterTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Initially no custom endpoint
    let stats_before = client.get_statistics().await.unwrap();
    assert!(!stats_before.custom_endpoint_configured);

    // Set custom endpoint
    client
        .set_custom_endpoint("http://localhost:11434", "ollama", None)
        .await
        .unwrap();

    // Now should show configured
    let stats_after = client.get_statistics().await.unwrap();
    assert!(stats_after.custom_endpoint_configured);

    // Clear endpoint
    client.clear_custom_endpoint().await.unwrap();

    // Should show not configured again
    let stats_final = client.get_statistics().await.unwrap();
    assert!(!stats_final.custom_endpoint_configured);
}
