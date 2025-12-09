//! LLM Provider implementations
//!
//! Each provider translates between the gateway's Anthropic-compatible format
//! and the provider's native API format.

pub mod anthropic;
pub mod azure;
pub mod deepseek;
pub mod ollama;
pub mod visiquate;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use reqwest::header::HeaderMap;

use super::config::{ProviderConfig, ProviderType};
use super::{CompletionRequest, CompletionResponse, RequestMetrics};

/// Type alias for a byte stream (SSE response body)
pub type ByteStream = std::pin::Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// Streaming response with headers from the upstream provider
/// This allows forwarding important headers (like request-id) to the client
pub struct StreamingResponse {
    /// The byte stream of SSE events
    pub stream: ByteStream,
    /// Headers from the upstream provider response
    pub headers: HeaderMap,
}

/// Provider trait for LLM backends
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Check if the provider is healthy/available
    async fn health_check(&self) -> Result<bool>;

    /// Execute a completion request
    /// Returns (response, metrics) tuple
    async fn complete(
        &self,
        request: CompletionRequest,
        client_auth: Option<String>,
        client_beta: Option<String>,
    ) -> Result<(CompletionResponse, RequestMetrics)>;

    /// Execute a streaming completion request
    /// Returns a StreamingResponse containing the byte stream and upstream headers
    /// Default implementation returns an error (not all providers support streaming)
    async fn complete_stream(
        &self,
        _request: CompletionRequest,
        _client_auth: Option<String>,
        _client_beta: Option<String>,
    ) -> Result<StreamingResponse> {
        Err(anyhow!(
            "Streaming not supported by provider: {}",
            self.name()
        ))
    }

    /// Get the resolved model name (handling aliases)
    fn resolve_model(&self, model: &str) -> String;
}

/// Registry of available providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Create a registry from provider configurations
    pub async fn from_config(configs: &HashMap<String, ProviderConfig>) -> Result<Self> {
        let mut registry = Self::new();

        for (name, config) in configs {
            if !config.enabled {
                tracing::info!(provider = %name, "Provider disabled, skipping");
                continue;
            }

            let provider: Arc<dyn Provider> = match config.provider_type {
                ProviderType::Anthropic => {
                    Arc::new(anthropic::AnthropicProvider::new(name.clone(), config.clone()).await?)
                }
                ProviderType::Azure => {
                    Arc::new(azure::AzureProvider::new(name.clone(), config.clone()).await?)
                }
                ProviderType::DeepSeek => {
                    Arc::new(deepseek::DeepSeekProvider::new(name.clone(), config.clone()).await?)
                }
                ProviderType::Ollama => {
                    Arc::new(ollama::OllamaProvider::new(name.clone(), config.clone()).await?)
                }
                ProviderType::OpenAI => {
                    // OpenAI uses the same format as DeepSeek (OpenAI-compatible)
                    Arc::new(deepseek::DeepSeekProvider::new(name.clone(), config.clone()).await?)
                }
                ProviderType::VisiQuate => {
                    Arc::new(visiquate::VisiquateProvider::new(name.clone(), config.clone()).await?)
                }
            };

            registry.register(name.clone(), provider);
            tracing::info!(provider = %name, provider_type = ?config.provider_type, "Registered provider");
        }

        Ok(registry)
    }

    /// Register a provider
    pub fn register(&mut self, name: String, provider: Arc<dyn Provider>) {
        self.providers.insert(name, provider);
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Result<Arc<dyn Provider>> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Provider not found: {}", name))
    }

    /// List all registered provider names
    pub fn list(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check health of all providers
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for (name, provider) in &self.providers {
            let healthy = provider.health_check().await.unwrap_or(false);
            results.insert(name.clone(), healthy);
        }

        results
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to resolve API key from environment or config
///
/// For Anthropic provider, falls back to CLAUDE_CODE_OAUTH_TOKEN if ANTHROPIC_API_KEY
/// is not set. This allows seamless integration when running under Claude Code.
pub fn resolve_api_key(api_key_ref: &str) -> Result<String> {
    if api_key_ref.is_empty() {
        return Ok(String::new());
    }

    // Extract the actual variable name
    let var_name = if api_key_ref.starts_with("env:") {
        &api_key_ref[4..]
    } else if api_key_ref.starts_with('$') {
        &api_key_ref[1..]
    } else {
        api_key_ref
    };

    // Try to get the primary environment variable
    match std::env::var(var_name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => {
            // Special fallback: if ANTHROPIC_API_KEY not found or empty, try CLAUDE_CODE_OAUTH_TOKEN
            // This allows the gateway to work seamlessly when running under Claude Code
            if var_name == "ANTHROPIC_API_KEY" {
                if let Ok(oauth_token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
                    if !oauth_token.is_empty() {
                        tracing::info!("ANTHROPIC_API_KEY not set, using CLAUDE_CODE_OAUTH_TOKEN");
                        return Ok(oauth_token);
                    }
                }
            }
            Err(anyhow!("Environment variable {} not found", var_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = ProviderRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_registry_get_missing() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("nonexistent").is_err());
    }

    #[test]
    fn test_resolve_api_key_env_prefix() {
        std::env::set_var("TEST_API_KEY_123", "test-value");
        let result = resolve_api_key("env:TEST_API_KEY_123");
        assert_eq!(result.unwrap(), "test-value");
        std::env::remove_var("TEST_API_KEY_123");
    }

    #[test]
    fn test_resolve_api_key_dollar_prefix() {
        std::env::set_var("TEST_API_KEY_456", "test-value-2");
        let result = resolve_api_key("$TEST_API_KEY_456");
        assert_eq!(result.unwrap(), "test-value-2");
        std::env::remove_var("TEST_API_KEY_456");
    }

    #[test]
    fn test_resolve_api_key_direct() {
        std::env::set_var("TEST_API_KEY_789", "test-value-3");
        let result = resolve_api_key("TEST_API_KEY_789");
        assert_eq!(result.unwrap(), "test-value-3");
        std::env::remove_var("TEST_API_KEY_789");
    }

    #[test]
    fn test_resolve_api_key_missing() {
        let result = resolve_api_key("DEFINITELY_NOT_SET_XYZ_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_api_key_empty() {
        let result = resolve_api_key("");
        assert_eq!(result.unwrap(), "");
    }
}
