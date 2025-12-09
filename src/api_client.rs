//! API Client for CCO Daemon Communication
//!
//! Provides HTTP client with retry logic, exponential backoff, and SSE stream support
//! for communicating with the CCO daemon's REST API.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Default request timeout
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default maximum retries
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Initial backoff delay
const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

/// Maximum backoff delay
const MAX_BACKOFF: Duration = Duration::from_secs(2);

/// API client for daemon communication
#[derive(Clone)]
pub struct ApiClient {
    pub base_url: String,
    client: Client,
    max_retries: u32,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Health check endpoint
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        self.get_with_retry(&url).await
    }

    /// Ready check endpoint (faster than health check - just checks daemon is responsive)
    pub async fn ready(&self) -> Result<serde_json::Value> {
        let url = format!("{}/ready", self.base_url);
        self.get_with_retry(&url).await
    }

    /// Get agents list
    pub async fn get_agents(&self) -> Result<Vec<Agent>> {
        let url = format!("{}/api/agents", self.base_url);
        self.get_with_retry(&url).await
    }

    /// Get statistics
    pub async fn get_stats(&self) -> Result<Stats> {
        let url = format!("{}/api/stats", self.base_url);
        self.get_with_retry(&url).await
    }

    /// Generic GET request with retry logic
    pub async fn get_with_retry<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let mut last_error = None;
        let mut backoff = INITIAL_BACKOFF;

        for attempt in 1..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    // Handle HTTP errors
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();

                        last_error = Some(anyhow::anyhow!(
                            "HTTP error {}: {}",
                            status,
                            if error_text.is_empty() {
                                status.canonical_reason().unwrap_or("Unknown error")
                            } else {
                                &error_text
                            }
                        ));

                        // Retry on 5xx errors
                        if status.is_server_error() && attempt < self.max_retries {
                            sleep(backoff).await;
                            backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
                            continue;
                        }

                        return Err(last_error.unwrap());
                    }

                    // Parse JSON response
                    match response.json::<T>().await {
                        Ok(data) => return Ok(data),
                        Err(e) => {
                            return Err(anyhow::anyhow!("JSON parse error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Request failed: {}", e));

                    // Retry on connection errors
                    if attempt < self.max_retries {
                        sleep(backoff).await;
                        backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Request failed after {} retries", self.max_retries)
        }))
    }
}

/// Health response from /health endpoint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub uptime_seconds: u64,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub hooks: Option<HooksHealthStatus>,
}

/// Hooks system health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HooksHealthStatus {
    pub enabled: bool,
    pub classifier_available: bool,
    pub model_loaded: bool,
    pub model_name: String,
    #[serde(default)]
    pub classification_latency_ms: Option<u32>,
}

/// Agent information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub type_name: String,
    pub model: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Statistics information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub total_requests: u64,
    #[serde(default)]
    pub total_cost: f64,
    #[serde(default)]
    pub cache_hits: u64,
    #[serde(default)]
    pub cache_misses: u64,
    /// Token breakdown by model tier (Haiku, Sonnet, Opus)
    #[serde(default)]
    pub token_breakdown: std::collections::HashMap<String, ModelTokens>,
}

/// Token breakdown for a specific model tier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModelTokens {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

/// Calculate exponential backoff delay
pub fn calculate_backoff(attempt: u32, initial: Duration, max: Duration) -> Duration {
    if attempt == 0 {
        return initial;
    }

    let delay_ms = initial.as_millis() * (2_u128.pow(attempt - 1));
    let delay = Duration::from_millis(delay_ms.min(u64::MAX as u128) as u64);

    std::cmp::min(delay, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("http://127.0.0.1:13109".to_string());
        assert_eq!(client.base_url, "http://127.0.0.1:13109");
        assert_eq!(client.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn test_api_client_with_timeout() {
        let client = ApiClient::new("http://127.0.0.1:13109".to_string())
            .with_timeout(Duration::from_secs(10));
        assert_eq!(client.base_url, "http://127.0.0.1:13109");
    }

    #[test]
    fn test_api_client_with_max_retries() {
        let client = ApiClient::new("http://127.0.0.1:13109".to_string()).with_max_retries(5);
        assert_eq!(client.max_retries, 5);
    }

    #[test]
    fn test_calculate_backoff() {
        let initial = Duration::from_millis(100);
        let max = Duration::from_millis(1000);

        assert_eq!(
            calculate_backoff(1, initial, max),
            Duration::from_millis(100)
        );
        assert_eq!(
            calculate_backoff(2, initial, max),
            Duration::from_millis(200)
        );
        assert_eq!(
            calculate_backoff(3, initial, max),
            Duration::from_millis(400)
        );
        assert_eq!(
            calculate_backoff(4, initial, max),
            Duration::from_millis(800)
        );
        assert_eq!(
            calculate_backoff(5, initial, max),
            Duration::from_millis(1000)
        ); // Capped at max
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{
            "status": "ok",
            "version": "2025.11.2",
            "uptime_seconds": 123,
            "port": 13109
        }"#;

        let health: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.version, "2025.11.2");
        assert_eq!(health.uptime_seconds, 123);
        assert_eq!(health.port, 13109);
    }

    #[test]
    fn test_agent_deserialization() {
        let json = r#"{
            "name": "Chief Architect",
            "type_name": "system-architect",
            "model": "opus-4.1",
            "capabilities": ["architecture", "design"]
        }"#;

        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.name, "Chief Architect");
        assert_eq!(agent.type_name, "system-architect");
        assert_eq!(agent.model, "opus-4.1");
        assert_eq!(agent.capabilities.len(), 2);
    }

    #[test]
    fn test_stats_deserialization() {
        let json = r#"{
            "total_requests": 100,
            "total_cost": 1.23,
            "cache_hits": 50,
            "cache_misses": 50
        }"#;

        let stats: Stats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.total_requests, 100);
        assert_eq!(stats.total_cost, 1.23);
        assert_eq!(stats.cache_hits, 50);
        assert_eq!(stats.cache_misses, 50);
    }
}
