//! HTTP proxy server for multi-model LLM requests

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Chat message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Chat request
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

/// Mock proxy server for testing
pub struct ProxyServer {
    cache: Arc<Mutex<HashMap<String, CachedResponseData>>>,
    api_calls: Arc<Mutex<Vec<ChatRequest>>>,
}

impl ProxyServer {
    /// Create a new proxy server
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            api_calls: Arc::new(Mutex::new(Vec::new())),
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

    /// Handle a request (check cache, record API call, return response)
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

        let cache = self.cache.lock().await;

        if let Some(cached) = cache.get(&cache_key) {
            // Cache hit
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

        // Cache miss - record API call
        drop(cache);
        let mut api_calls = self.api_calls.lock().await;
        api_calls.push(request.clone());
        drop(api_calls);

        // Simulate API response
        let response = ChatResponse {
            id: format!("api-{}", uuid_v4()),
            model: request.model.clone(),
            content: "This is a simulated response".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
            from_cache: false,
        };

        // Store in cache
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

        // First request - cache miss
        let response1 = proxy.handle_request(request.clone()).await;
        assert!(!response1.from_cache);
        assert_eq!(proxy.get_api_call_count().await, 1);

        // Second identical request - cache hit
        let response2 = proxy.handle_request(request).await;
        assert!(response2.from_cache);
        assert_eq!(proxy.get_api_call_count().await, 1);

        assert_eq!(response1.content, response2.content);
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

        let response = proxy.handle_request(request).await;
        assert!(!response.from_cache);
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

        let response1 = proxy.handle_request(base_request.clone()).await;
        assert!(!response1.from_cache);

        let mut request2 = base_request;
        request2.model = "claude-sonnet-3.5".to_string();
        let response2 = proxy.handle_request(request2).await;
        assert!(!response2.from_cache);

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

        let response1 = proxy.handle_request(base_request.clone()).await;
        assert!(!response1.from_cache);

        let mut request2 = base_request;
        request2.temperature = Some(1.5);
        let response2 = proxy.handle_request(request2).await;
        assert!(!response2.from_cache);

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

        let response1 = proxy.handle_request(request.clone()).await;
        assert!(!response1.from_cache);
        assert_eq!(proxy.get_api_call_count().await, 1);

        proxy.clear().await;

        let response2 = proxy.handle_request(request).await;
        assert!(!response2.from_cache);

        // Cache was cleared, so second request is a cache miss
        // API call count accumulates (not cleared by clear())
        assert_eq!(proxy.get_api_call_count().await, 2);
    }
}
