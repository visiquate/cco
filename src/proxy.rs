//! HTTP proxy server for multi-model LLM requests
//!
//! Makes real HTTP calls to Claude (Anthropic) and other LLM APIs.
//! Routes requests based on model and caches responses using SHA256 keys.

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Chat message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Chat request (OpenAI-compatible format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Usage information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Chat response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub usage: Usage,
    #[serde(default)]
    pub from_cache: bool,
}

/// Cached response
#[derive(Clone, Debug)]
pub struct CachedResponseData {
    pub model: String,
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Real HTTP proxy server for multi-model LLM requests
pub struct ProxyServer {
    cache: Arc<Mutex<HashMap<String, CachedResponseData>>>,
    api_calls: Arc<Mutex<Vec<ChatRequest>>>,
    client: Client,
}

impl ProxyServer {
    /// Create a new proxy server
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            api_calls: Arc::new(Mutex::new(Vec::new())),
            client: Client::new(),
        }
    }

    /// Generate cache key using SHA256
    pub fn generate_cache_key(
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(prompt.as_bytes());
        hasher.update(temperature.to_le_bytes());
        hasher.update(max_tokens.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get API key from environment
    fn get_api_key(&self, provider: &str) -> Option<String> {
        match provider {
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
            "azure" => std::env::var("AZURE_OPENAI_API_KEY").ok(),
            "deepseek" => std::env::var("DEEPSEEK_API_KEY").ok(),
            "custom" => std::env::var("CODER_LLM_TOKEN").ok(),
            _ => None,
        }
    }

    /// Determine which provider should handle this model
    fn get_provider(&self, model: &str) -> &'static str {
        if model.contains("claude") {
            "anthropic"
        } else if model.contains("gpt") {
            "azure"
        } else if model.contains("deepseek") || model.contains("DeepSeek") {
            "deepseek"
        } else {
            "custom"
        }
    }

    /// Make a real API call to Anthropic Claude API
    async fn call_anthropic(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let api_key = self
            .get_api_key("anthropic")
            .ok_or_else(|| anyhow!("ANTHROPIC_API_KEY not set"))?;

        let url = "https://api.anthropic.com/v1/messages";

        // Build request payload
        let payload = json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "messages": request.messages.iter().map(|m| json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "temperature": request.temperature.unwrap_or(1.0),
        });

        debug!("Calling Anthropic API with model: {}", request.model);

        let response = self
            .client
            .post(url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to call Anthropic API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Anthropic API error {}: {}",
                status,
                error_text
            ));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Anthropic response: {}", e))?;

        // Extract content from Anthropic response
        let content = response_json
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No content in Anthropic response"))?
            .to_string();

        let usage = response_json.get("usage").ok_or_else(|| anyhow!("No usage in response"))?;
        let input_tokens = usage
            .get("input_tokens")
            .and_then(|t| t.as_u64())
            .ok_or_else(|| anyhow!("No input_tokens in usage"))? as u32;
        let output_tokens = usage
            .get("output_tokens")
            .and_then(|t| t.as_u64())
            .ok_or_else(|| anyhow!("No output_tokens in usage"))? as u32;

        let id = response_json
            .get("id")
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string();

        info!(
            "Anthropic API success: model={}, tokens=({}/{})",
            request.model, input_tokens, output_tokens
        );

        Ok(ChatResponse {
            id,
            model: request.model.clone(),
            content,
            input_tokens,
            output_tokens,
            usage: Usage {
                input_tokens,
                output_tokens,
            },
            from_cache: false,
        })
    }

    /// Make a real API call to Azure OpenAI
    async fn call_azure(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let api_key = self
            .get_api_key("azure")
            .ok_or_else(|| anyhow!("AZURE_OPENAI_API_KEY not set"))?;

        // Azure endpoint format: https://{resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions
        let url = "https://cco-resource.openai.azure.com/openai/deployments/gpt-5-1-mini/chat/completions?api-version=2024-05-01-preview";

        // Build request payload (OpenAI-compatible)
        let payload = json!({
            "messages": request.messages.iter().map(|m| json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "temperature": request.temperature.unwrap_or(1.0),
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        debug!("Calling Azure API with model: {}", request.model);

        let response = self
            .client
            .post(url)
            .header("api-key", &api_key)
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to call Azure API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Azure API error {}: {}", status, error_text));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Azure response: {}", e))?;

        // Extract content from Azure response
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No content in Azure response"))?
            .to_string();

        let usage = response_json.get("usage").ok_or_else(|| anyhow!("No usage in response"))?;
        let input_tokens = usage
            .get("prompt_tokens")
            .and_then(|t| t.as_u64())
            .ok_or_else(|| anyhow!("No prompt_tokens in usage"))? as u32;
        let output_tokens = usage
            .get("completion_tokens")
            .and_then(|t| t.as_u64())
            .ok_or_else(|| anyhow!("No completion_tokens in usage"))? as u32;

        let id = response_json
            .get("id")
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string();

        info!(
            "Azure API success: model={}, tokens=({}/{})",
            request.model, input_tokens, output_tokens
        );

        Ok(ChatResponse {
            id,
            model: request.model.clone(),
            content,
            input_tokens,
            output_tokens,
            usage: Usage {
                input_tokens,
                output_tokens,
            },
            from_cache: false,
        })
    }

    /// Make a real API call to DeepSeek V3.1 API
    async fn call_deepseek(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let api_key = self
            .get_api_key("deepseek")
            .ok_or_else(|| anyhow!("DEEPSEEK_API_KEY not set"))?;

        // DeepSeek endpoint via Azure (as provided)
        let url = "https://cco-resource.cognitiveservices.azure.com/openai/deployments/DeepSeek-V3.1/chat/completions?api-version=2024-05-01-preview";

        // Build request payload (OpenAI-compatible format)
        let payload = json!({
            "messages": request.messages.iter().map(|m| json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "temperature": request.temperature.unwrap_or(1.0),
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        debug!("Calling DeepSeek API with model: {}", request.model);

        let response = self
            .client
            .post(url)
            .header("api-key", &api_key)
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to call DeepSeek API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("DeepSeek API error {}: {}", status, error_text));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse DeepSeek response: {}", e))?;

        // Extract content from Azure OpenAI-compatible response
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No content in DeepSeek response"))?
            .to_string();

        let usage = response_json.get("usage").ok_or_else(|| anyhow!("No usage in response"))?;
        let input_tokens = usage
            .get("prompt_tokens")
            .and_then(|t| t.as_u64())
            .ok_or_else(|| anyhow!("No prompt_tokens in usage"))? as u32;
        let output_tokens = usage
            .get("completion_tokens")
            .and_then(|t| t.as_u64())
            .ok_or_else(|| anyhow!("No completion_tokens in usage"))? as u32;

        let id = response_json
            .get("id")
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string();

        info!(
            "DeepSeek API success: model={}, tokens=({}/{})",
            request.model, input_tokens, output_tokens
        );

        Ok(ChatResponse {
            id,
            model: request.model.clone(),
            content,
            input_tokens,
            output_tokens,
            usage: Usage {
                input_tokens,
                output_tokens,
            },
            from_cache: false,
        })
    }

    /// Handle a request (check cache, make real API call, cache response)
    pub async fn handle_request(&self, request: ChatRequest) -> ChatResponse {
        let prompt = request
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let cache_key = Self::generate_cache_key(
            &request.model,
            &prompt,
            request.temperature.unwrap_or(1.0),
            request.max_tokens.unwrap_or(4096),
        );

        // Check cache first
        let cache = self.cache.lock().await;
        if let Some(cached) = cache.get(&cache_key) {
            debug!("Cache hit for model: {}", request.model);
            return ChatResponse {
                id: format!("cache-{}", uuid_v4()),
                model: cached.model.clone(),
                content: cached.content.clone(),
                input_tokens: cached.input_tokens,
                output_tokens: cached.output_tokens,
                usage: Usage {
                    input_tokens: cached.input_tokens,
                    output_tokens: cached.output_tokens,
                },
                from_cache: true,
            };
        }
        drop(cache);

        // Record API call
        let mut api_calls = self.api_calls.lock().await;
        api_calls.push(request.clone());
        drop(api_calls);

        // Make real API call based on model
        let result = match self.get_provider(&request.model) {
            "anthropic" => self.call_anthropic(&request).await,
            "azure" => self.call_azure(&request).await,
            "deepseek" => self.call_deepseek(&request).await,
            "custom" => {
                warn!("Custom LLM provider not yet implemented");
                Err(anyhow!("Custom LLM provider not yet implemented"))
            }
            _ => Err(anyhow!("Unknown provider for model: {}", request.model)),
        };

        // Handle API call result
        let response = match result {
            Ok(response) => {
                // Cache successful response
                let mut cache = self.cache.lock().await;
                cache.insert(
                    cache_key,
                    CachedResponseData {
                        model: response.model.clone(),
                        content: response.content.clone(),
                        input_tokens: response.input_tokens,
                        output_tokens: response.output_tokens,
                    },
                );
                drop(cache);

                info!(
                    "API call successful and cached: model={}, tokens=({}/{})",
                    request.model, response.input_tokens, response.output_tokens
                );

                response
            }
            Err(e) => {
                error!("API call failed: {}", e);
                // Create error response
                let error_response = ChatResponse {
                    id: format!("error-{}", uuid_v4()),
                    model: request.model.clone(),
                    content: format!("Error: {}", e),
                    input_tokens: 0,
                    output_tokens: 0,
                    usage: Usage {
                        input_tokens: 0,
                        output_tokens: 0,
                    },
                    from_cache: false,
                };

                // Also cache error responses for consistency
                let mut cache = self.cache.lock().await;
                cache.insert(
                    cache_key,
                    CachedResponseData {
                        model: error_response.model.clone(),
                        content: error_response.content.clone(),
                        input_tokens: error_response.input_tokens,
                        output_tokens: error_response.output_tokens,
                    },
                );

                error_response
            }
        };

        response
    }

    /// Get number of API calls made
    pub async fn get_api_call_count(&self) -> usize {
        let api_calls = self.api_calls.lock().await;
        api_calls.len()
    }

    /// Clear cache (keep API call log for audit)
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }
}

impl Default for ProxyServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple UUID v4 implementation
fn uuid_v4() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:016x}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cache_key() {
        let key1 = ProxyServer::generate_cache_key("claude-opus-4", "Hello", 0.7, 4096);
        let key2 = ProxyServer::generate_cache_key("claude-opus-4", "Hello", 0.7, 4096);
        let key3 = ProxyServer::generate_cache_key("claude-opus-4", "Goodbye", 0.7, 4096);

        // Same inputs should produce same key
        assert_eq!(key1, key2);

        // Different inputs should produce different keys
        assert_ne!(key1, key3);

        // Key should be hex-encoded SHA256 (64 chars)
        assert_eq!(key1.len(), 64);
    }

    #[tokio::test]
    async fn test_proxy_cache_hit_path() {
        let proxy = ProxyServer::new();

        let request = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "What is 2+2?".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        // First request - cache miss (will fail without API key, but still tests cache behavior)
        let _response1 = proxy.handle_request(request.clone()).await;
        assert_eq!(proxy.get_api_call_count().await, 1);

        // Second identical request - cache hit (even errors are cached)
        let response2 = proxy.handle_request(request).await;
        assert!(response2.from_cache);
        assert_eq!(proxy.get_api_call_count().await, 1);
    }

    #[tokio::test]
    async fn test_proxy_cache_miss_path() {
        let proxy = ProxyServer::new();

        let request = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let _response = proxy.handle_request(request).await;
        assert_eq!(proxy.get_api_call_count().await, 1);
    }

    #[tokio::test]
    async fn test_proxy_cache_isolation_by_model() {
        let proxy = ProxyServer::new();

        let base_request = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Same question".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let _response1 = proxy.handle_request(base_request.clone()).await;
        assert_eq!(proxy.get_api_call_count().await, 1);

        let mut request2 = base_request;
        request2.model = "claude-sonnet-3.5".to_string();
        let _response2 = proxy.handle_request(request2).await;
        assert_eq!(proxy.get_api_call_count().await, 2);
    }

    #[tokio::test]
    async fn test_proxy_cache_sensitivity_to_temperature() {
        let proxy = ProxyServer::new();

        let base_request = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(0.5),
            max_tokens: Some(4096),
        };

        let _response1 = proxy.handle_request(base_request.clone()).await;
        assert_eq!(proxy.get_api_call_count().await, 1);

        let mut request2 = base_request;
        request2.temperature = Some(1.5);
        let _response2 = proxy.handle_request(request2).await;
        assert_eq!(proxy.get_api_call_count().await, 2);
    }

    #[tokio::test]
    async fn test_proxy_cache_clear() {
        let proxy = ProxyServer::new();

        let request = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let _response1 = proxy.handle_request(request.clone()).await;
        assert_eq!(proxy.get_api_call_count().await, 1);

        proxy.clear().await;

        let _response2 = proxy.handle_request(request).await;
        // Cache was cleared, so second request is a cache miss
        // API call count accumulates (not cleared by clear())
        assert_eq!(proxy.get_api_call_count().await, 2);
    }

    #[test]
    fn test_provider_detection() {
        let proxy = ProxyServer::new();

        assert_eq!(proxy.get_provider("claude-opus-4"), "anthropic");
        assert_eq!(proxy.get_provider("claude-sonnet"), "anthropic");
        assert_eq!(proxy.get_provider("gpt-4"), "azure");
        assert_eq!(proxy.get_provider("gpt-5-1"), "azure");
        assert_eq!(proxy.get_provider("deepseek"), "deepseek");
        assert_eq!(proxy.get_provider("DeepSeek-V3.1"), "deepseek");
        assert_eq!(proxy.get_provider("other-model"), "custom");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.content, "Hello, world!");
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "claude-opus-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "claude-opus-4");
        assert_eq!(deserialized.temperature, Some(0.7));
        assert_eq!(deserialized.max_tokens, Some(2048));
    }
}
