//! Shared test utilities for hooks integration tests
//!
//! Provides helpers for:
//! - Starting test daemons with hooks enabled
//! - Creating test configurations
//! - Asserting classification results
//! - Managing temporary test environments

use anyhow::Result;
use cco::daemon::config::DaemonConfig;
use cco::daemon::hooks::{ClassificationResult, CrudClassification, HooksConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

/// Test client for interacting with daemon API
#[derive(Clone)]
pub struct TestClient {
    pub client: Client,
    pub base_url: String,
    pub port: u16,
}

impl TestClient {
    /// Create a new test client
    pub fn new(port: u16) -> Self {
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

    /// Call /api/classify endpoint
    pub async fn classify(&self, command: &str) -> Result<ClassifyResponse> {
        let url = format!("{}/api/classify", self.base_url);
        let request = ClassifyRequest {
            command: command.to_string(),
            context: None,
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

    /// Call /health endpoint
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    /// Call /api/health endpoint (extended health with hooks status)
    pub async fn api_health(&self) -> Result<ApiHealthResponse> {
        let url = format!("{}/api/health", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    /// Wait for daemon to become ready
    pub async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if self.health().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }

        anyhow::bail!("Daemon did not become ready within {:?}", timeout)
    }
}

/// Classification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyRequest {
    pub command: String,
    pub context: Option<serde_json::Value>,
}

/// Classification response from /api/classify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyResponse {
    pub classification: String,
    pub confidence: f32,
    #[serde(default)]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Health response from /health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: Option<u64>,
}

/// Extended health response from /api/health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: Option<u64>,
    #[serde(default)]
    pub hooks: Option<HooksStatus>,
}

/// Hooks status in health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksStatus {
    pub enabled: bool,
    pub classifier_available: bool,
    #[serde(default)]
    pub model_name: Option<String>,
    #[serde(default)]
    pub model_loaded: bool,
}

/// Test daemon handle
pub struct TestDaemon {
    pub port: u16,
    pub config_dir: TempDir,
    pub client: TestClient,
    #[allow(dead_code)]
    process_handle: Option<tokio::process::Child>,
}

impl TestDaemon {
    /// Start a test daemon with custom configuration
    pub async fn start(config: DaemonConfig) -> Result<Self> {
        let config_dir = TempDir::new()?;
        let port = find_available_port()?;

        // Save configuration
        let config_path = config_dir.path().join("config.toml");
        config.save(&config_path)?;

        // Start daemon process (stub for now - actual implementation would spawn daemon)
        // In real implementation:
        // let process = tokio::process::Command::new("cco")
        //     .args(&["daemon", "start", "--config", config_path.to_str().unwrap()])
        //     .spawn()?;

        let client = TestClient::new(port);

        // Wait for daemon to become ready
        // client.wait_for_ready(Duration::from_secs(10)).await?;

        Ok(Self {
            port,
            config_dir,
            client,
            process_handle: None,
        })
    }

    /// Start a test daemon with hooks enabled
    pub async fn with_hooks_enabled() -> Result<Self> {
        let mut config = DaemonConfig::default();
        config.hooks.enabled = true;
        Self::start(config).await
    }

    /// Start a test daemon with hooks disabled
    pub async fn with_hooks_disabled() -> Result<Self> {
        let mut config = DaemonConfig::default();
        config.hooks.enabled = false;
        Self::start(config).await
    }

    /// Get configuration directory path
    pub fn config_dir(&self) -> PathBuf {
        self.config_dir.path().to_path_buf()
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        // Cleanup: kill daemon process if running
        if let Some(mut process) = self.process_handle.take() {
            let _ = process.start_kill();
        }
    }
}

/// Find an available TCP port
pub fn find_available_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Assert that a classification result matches expected values
pub fn assert_classification(
    result: &ClassifyResponse,
    expected_classification: &str,
    min_confidence: f32,
) {
    assert_eq!(
        result.classification.to_uppercase(),
        expected_classification.to_uppercase(),
        "Classification mismatch"
    );
    assert!(
        result.confidence >= min_confidence,
        "Confidence {:.2} below minimum {:.2}",
        result.confidence,
        min_confidence
    );
}

/// Create a test hooks configuration
pub fn test_hooks_config() -> HooksConfig {
    HooksConfig {
        enabled: true,
        timeout_ms: 5000,
        max_retries: 2,
        llm: Default::default(),
        permissions: Default::default(),
        callbacks: Default::default(),
    }
}

/// Create a test daemon configuration with hooks
pub fn create_daemon_config_with_hooks() -> DaemonConfig {
    let mut config = DaemonConfig::default();
    config.hooks = test_hooks_config();
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port() {
        let port = find_available_port().unwrap();
        assert!(port > 0);
        // port is u16, so it's always <= 65535
    }

    #[test]
    fn test_test_client_creation() {
        let client = TestClient::new(3000);
        assert_eq!(client.port, 3000);
        assert_eq!(client.base_url, "http://127.0.0.1:3000");
    }

    #[test]
    fn test_hooks_config_creation() {
        let config = test_hooks_config();
        assert!(config.enabled);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_daemon_config_with_hooks() {
        let config = create_daemon_config_with_hooks();
        assert!(config.hooks.enabled);
    }

    #[test]
    fn test_assert_classification_success() {
        let response = ClassifyResponse {
            classification: "READ".to_string(),
            confidence: 0.95,
            reasoning: None,
            timestamp: None,
        };

        assert_classification(&response, "READ", 0.8);
    }

    #[test]
    #[should_panic(expected = "Classification mismatch")]
    fn test_assert_classification_fail_mismatch() {
        let response = ClassifyResponse {
            classification: "CREATE".to_string(),
            confidence: 0.95,
            reasoning: None,
            timestamp: None,
        };

        assert_classification(&response, "READ", 0.8);
    }

    #[test]
    #[should_panic(expected = "Confidence")]
    fn test_assert_classification_fail_confidence() {
        let response = ClassifyResponse {
            classification: "READ".to_string(),
            confidence: 0.5,
            reasoning: None,
            timestamp: None,
        };

        assert_classification(&response, "READ", 0.8);
    }
}
