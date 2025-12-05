//! LiteLLM HTTP client
//!
//! Provides a client for calling the LiteLLM proxy server.
//! Handles both streaming and non-streaming completion requests.

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, error, info};

use super::{CompletionRequest, CompletionResponse, RequestMetrics, Usage};

/// Type alias for a byte stream (SSE response body)
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// Streaming response with headers from upstream
pub struct LiteLLMStreamingResponse {
    /// The byte stream of SSE events
    pub stream: ByteStream,
    /// Headers from the upstream response
    pub headers: HeaderMap,
}

/// LiteLLM HTTP client for making completion requests
pub struct LiteLLMClient {
    /// Base URL of the LiteLLM proxy (e.g., "http://localhost:4000")
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
}

impl LiteLLMClient {
    /// Create a new LiteLLM client
    pub fn new(base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(600)) // 10 minute timeout for long requests
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        })
    }

    /// Execute a non-streaming completion request
    pub async fn complete(
        &self,
        request: CompletionRequest,
        client_auth: Option<String>,
        client_beta: Option<String>,
    ) -> Result<(CompletionResponse, RequestMetrics)> {
        let start = std::time::Instant::now();
        let url = format!("{}/v1/messages", self.base_url);

        debug!(
            model = %request.model,
            stream = false,
            "Sending completion request to LiteLLM"
        );

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );

        // Pass through client auth if provided
        if let Some(auth) = &client_auth {
            if auth.starts_with("Bearer ") || auth.starts_with("bearer ") {
                headers.insert(AUTHORIZATION, HeaderValue::from_str(auth)?);
            } else {
                headers.insert(
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_str(auth)?,
                );
            }
        }

        // Pass through beta header if provided
        if let Some(beta) = &client_beta {
            headers.insert(
                HeaderName::from_static("anthropic-beta"),
                HeaderValue::from_str(beta)?,
            );
        }

        // Send request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LiteLLM")?;

        let status = response.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                error = %error_text,
                "LiteLLM request failed"
            );
            anyhow::bail!("LiteLLM returned {}: {}", status, error_text);
        }

        // Parse response
        let response_body: CompletionResponse = response
            .json()
            .await
            .context("Failed to parse LiteLLM response")?;

        // Build metrics
        let metrics = RequestMetrics::new(
            uuid::Uuid::new_v4().to_string(),
            "litellm".to_string(),
            response_body.model.clone(),
            &response_body.usage,
            calculate_cost(&response_body.model, &response_body.usage),
            latency_ms,
        )
        .with_agent_type(request.agent_type.clone())
        .with_project_id(request.project_id.clone());

        info!(
            model = %response_body.model,
            input_tokens = response_body.usage.input_tokens,
            output_tokens = response_body.usage.output_tokens,
            latency_ms = latency_ms,
            "LiteLLM completion successful"
        );

        Ok((response_body, metrics))
    }

    /// Execute a streaming completion request
    ///
    /// Returns a LiteLLMStreamingResponse with byte stream and headers
    pub async fn complete_stream(
        &self,
        mut request: CompletionRequest,
        client_auth: Option<String>,
        client_beta: Option<String>,
    ) -> Result<LiteLLMStreamingResponse> {
        let url = format!("{}/v1/messages", self.base_url);

        // Ensure stream is enabled
        request.stream = true;

        debug!(
            model = %request.model,
            stream = true,
            "Sending streaming completion request to LiteLLM"
        );

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );

        // Pass through client auth if provided
        if let Some(auth) = &client_auth {
            if auth.starts_with("Bearer ") || auth.starts_with("bearer ") {
                headers.insert(AUTHORIZATION, HeaderValue::from_str(auth)?);
            } else {
                headers.insert(
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_str(auth)?,
                );
            }
        }

        // Pass through beta header if provided
        if let Some(beta) = &client_beta {
            headers.insert(
                HeaderName::from_static("anthropic-beta"),
                HeaderValue::from_str(beta)?,
            );
        }

        // Send request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send streaming request to LiteLLM")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                error = %error_text,
                "LiteLLM streaming request failed"
            );
            anyhow::bail!("LiteLLM returned {}: {}", status, error_text);
        }

        // Capture headers before consuming response body
        let headers = response.headers().clone();

        // Return the byte stream with headers
        Ok(LiteLLMStreamingResponse {
            stream: Box::pin(response.bytes_stream()),
            headers,
        })
    }

    /// Check if LiteLLM is healthy
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Calculate approximate cost based on model and token usage
///
/// Note: LiteLLM also tracks costs internally, but this provides
/// immediate cost estimates for our Rust metrics layer.
fn calculate_cost(model: &str, usage: &Usage) -> f64 {
    // Pricing per million tokens (approximate, may vary)
    let (input_price, output_price) = if model.contains("opus") {
        (15.0, 75.0) // Opus pricing
    } else if model.contains("sonnet") {
        (3.0, 15.0) // Sonnet pricing
    } else if model.contains("haiku") {
        (0.25, 1.25) // Haiku pricing
    } else if model.contains("gpt-4") {
        (10.0, 30.0) // GPT-4 pricing (approximate)
    } else if model.contains("deepseek") {
        (0.14, 0.28) // DeepSeek pricing (approximate)
    } else {
        (1.0, 2.0) // Default/unknown pricing
    };

    let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * input_price;
    let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * output_price;

    // Account for cache tokens (reduced cost)
    let cache_read_cost = if let Some(cache_read) = usage.cache_read_input_tokens {
        (cache_read as f64 / 1_000_000.0) * (input_price * 0.1) // 90% discount
    } else {
        0.0
    };

    let cache_write_cost = if let Some(cache_write) = usage.cache_creation_input_tokens {
        (cache_write as f64 / 1_000_000.0) * (input_price * 1.25) // 25% premium
    } else {
        0.0
    };

    input_cost + output_cost + cache_read_cost + cache_write_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost_opus() {
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        let cost = calculate_cost("claude-opus-4-5-20251101", &usage);
        // 1000/1M * 15 + 500/1M * 75 = 0.015 + 0.0375 = 0.0525
        assert!((cost - 0.0525).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_haiku() {
        let usage = Usage {
            input_tokens: 10000,
            output_tokens: 1000,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        let cost = calculate_cost("claude-haiku-4-5-20251001", &usage);
        // 10000/1M * 0.25 + 1000/1M * 1.25 = 0.0025 + 0.00125 = 0.00375
        assert!((cost - 0.00375).abs() < 0.0001);
    }

    #[test]
    fn test_client_creation() {
        let client = LiteLLMClient::new("http://localhost:4000");
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url(), "http://localhost:4000");
    }

    #[test]
    fn test_trailing_slash_removed() {
        let client = LiteLLMClient::new("http://localhost:4000/").unwrap();
        assert_eq!(client.base_url(), "http://localhost:4000");
    }
}
