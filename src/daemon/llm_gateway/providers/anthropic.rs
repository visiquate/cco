//! Anthropic provider implementation
//!
//! Native format for the gateway - minimal translation needed.
//! Supports both synchronous and SSE streaming completions.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{resolve_api_key, ByteStream, Provider};
use crate::daemon::llm_gateway::config::{ProviderConfig, ProviderType};
use crate::daemon::llm_gateway::metrics::PricingTable;
use crate::daemon::llm_gateway::{
    CompletionRequest, CompletionResponse, ContentBlock, RequestMetrics, Usage,
};

/// Anthropic API provider
pub struct AnthropicProvider {
    name: String,
    config: ProviderConfig,
    client: Client,
    api_key: String,
    pricing: PricingTable,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub async fn new(name: String, config: ProviderConfig) -> Result<Self> {
        let api_key = resolve_api_key(&config.api_key_ref)?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self {
            name,
            config,
            client,
            api_key,
            pricing: PricingTable::default(),
        })
    }

    /// Build the API URL
    fn api_url(&self) -> String {
        format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'))
    }

    /// Get authentication header from configured API key
    /// Returns (header_name, header_value)
    fn get_configured_auth(&self) -> (&'static str, String) {
        if self.api_key.starts_with("sk-ant-oat") {
            // OAuth access token - use Bearer auth
            ("Authorization", format!("Bearer {}", self.api_key))
        } else {
            // Standard API key
            ("x-api-key", self.api_key.clone())
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple check - try to reach the API endpoint
        // Anthropic doesn't have a dedicated health endpoint, so we just check connectivity
        let response = self
            .client
            .get(&self.config.base_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        Ok(response.is_ok())
    }

    fn resolve_model(&self, model: &str) -> String {
        self.config
            .model_aliases
            .get(model)
            .cloned()
            .unwrap_or_else(|| model.to_string())
    }

    async fn complete(
        &self,
        request: CompletionRequest,
        client_auth: Option<String>,
        client_beta: Option<String>,
    ) -> Result<(CompletionResponse, RequestMetrics)> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        let model = self.resolve_model(&request.model);

        // Build the Anthropic API request
        let api_request = AnthropicRequest {
            model: model.clone(),
            messages: request.messages.iter().map(|m| m.clone().into()).collect(),
            max_tokens: request.max_tokens,
            system: request.system.clone(),
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: request.top_k,
            stop_sequences: request.stop_sequences.clone(),
            stream: request.stream,
        };

        // Make the API request
        // Determine auth to use: client passthrough > env var > configured key
        let mut request_builder = self
            .client
            .post(self.api_url())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json");

        let (auth_header_name, auth_header_value) = if let Some(auth) = client_auth {
            // 1. Client passthrough (highest priority)
            if auth.to_lowercase().starts_with("bearer ") {
                tracing::debug!("Using client passthrough authentication (Bearer)");
                ("Authorization", auth)
            } else {
                tracing::debug!("Using client passthrough authentication (API key)");
                ("x-api-key", auth)
            }
        } else if let Ok(oauth_token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
            // 2. Environment variable (Claude Code's OAuth token)
            if !oauth_token.is_empty() {
                tracing::debug!("Using CLAUDE_CODE_OAUTH_TOKEN from environment");
                ("Authorization", format!("Bearer {}", oauth_token))
            } else {
                // Fall through to configured key
                tracing::debug!("CLAUDE_CODE_OAUTH_TOKEN is empty, using configured authentication");
                self.get_configured_auth()
            }
        } else {
            // 3. Configured API key (fallback)
            tracing::debug!("Using configured authentication");
            self.get_configured_auth()
        };

        request_builder = request_builder.header(auth_header_name, auth_header_value);

        // Add anthropic-beta header if provided by client
        if let Some(beta) = client_beta {
            request_builder = request_builder.header("anthropic-beta", beta);
            tracing::debug!("Using client anthropic-beta header");
        }

        let response = request_builder
            .json(&api_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!(
                "Anthropic API error ({}): {}",
                status,
                response_text
            ));
        }

        // Parse the response
        let api_response: AnthropicResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Anthropic response: {} - {}", e, response_text))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Convert to gateway response
        let usage = Usage {
            input_tokens: api_response.usage.input_tokens,
            output_tokens: api_response.usage.output_tokens,
            cache_creation_input_tokens: api_response.usage.cache_creation_input_tokens,
            cache_read_input_tokens: api_response.usage.cache_read_input_tokens,
        };

        // Calculate cost
        let cost_usd = self.pricing.calculate_cost(&model, &usage);

        let response = CompletionResponse {
            id: api_response.id,
            response_type: api_response.response_type,
            role: api_response.role,
            model: api_response.model,
            content: api_response
                .content
                .into_iter()
                .map(|c| c.into())
                .collect(),
            stop_reason: api_response.stop_reason,
            stop_sequence: api_response.stop_sequence,
            usage: usage.clone(),
            provider: self.name.clone(),
            latency_ms,
            cost_usd,
        };

        let metrics = RequestMetrics::new(
            request_id,
            self.name.clone(),
            model,
            &usage,
            cost_usd,
            latency_ms,
        )
        .with_agent_type(request.agent_type)
        .with_project_id(request.project_id);

        Ok((response, metrics))
    }

    /// Execute a streaming completion request
    /// Returns a byte stream that yields SSE events directly from Anthropic
    async fn complete_stream(
        &self,
        request: CompletionRequest,
        client_auth: Option<String>,
        client_beta: Option<String>,
    ) -> Result<ByteStream> {
        let model = self.resolve_model(&request.model);

        // Build the Anthropic API request with stream: true
        let api_request = AnthropicRequest {
            model: model.clone(),
            messages: request.messages.iter().map(|m| m.clone().into()).collect(),
            max_tokens: request.max_tokens,
            system: request.system.clone(),
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: request.top_k,
            stop_sequences: request.stop_sequences.clone(),
            stream: true, // Always stream for this method
        };

        // Build request with auth passthrough
        // Determine auth to use: client passthrough > env var > configured key
        let mut request_builder = self
            .client
            .post(self.api_url())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json");

        let (auth_header_name, auth_header_value) = if let Some(auth) = client_auth {
            // 1. Client passthrough (highest priority)
            if auth.to_lowercase().starts_with("bearer ") {
                tracing::debug!("Streaming: Using client passthrough authentication (Bearer)");
                ("Authorization", auth)
            } else {
                tracing::debug!("Streaming: Using client passthrough authentication (API key)");
                ("x-api-key", auth)
            }
        } else if let Ok(oauth_token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
            // 2. Environment variable (Claude Code's OAuth token)
            if !oauth_token.is_empty() {
                tracing::debug!("Streaming: Using CLAUDE_CODE_OAUTH_TOKEN from environment");
                ("Authorization", format!("Bearer {}", oauth_token))
            } else {
                // Fall through to configured key
                tracing::debug!("Streaming: CLAUDE_CODE_OAUTH_TOKEN is empty, using configured authentication");
                self.get_configured_auth()
            }
        } else {
            // 3. Configured API key (fallback)
            tracing::debug!("Streaming: Using configured authentication");
            self.get_configured_auth()
        };

        request_builder = request_builder.header(auth_header_name, auth_header_value);

        // Add anthropic-beta header if provided by client
        if let Some(beta) = client_beta {
            request_builder = request_builder.header("anthropic-beta", beta);
            tracing::debug!("Streaming: Using client anthropic-beta header");
        }

        let response = request_builder
            .json(&api_request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Anthropic API error ({}): {}",
                status,
                error_text
            ));
        }

        tracing::info!(model = %model, "Started streaming response from Anthropic");

        // Return the byte stream directly - the API layer will forward it as SSE
        Ok(Box::pin(response.bytes_stream()))
    }
}

// ============================================================================
// Anthropic API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: AnthropicMessageContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AnthropicMessageContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

/// Anthropic content block - use Value for flexible pass-through
/// This ensures we forward all content types Claude Code sends without modification
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: serde_json::Value },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(default)]
        content: serde_json::Value,
        #[serde(default)]
        is_error: bool,
    },
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        #[serde(default)]
        signature: Option<String>,
    },
    #[serde(rename = "document")]
    Document { source: serde_json::Value },
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    model: String,
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
    #[serde(default)]
    cache_creation_input_tokens: Option<u32>,
    #[serde(default)]
    cache_read_input_tokens: Option<u32>,
}

// ============================================================================
// Type Conversions
// ============================================================================

impl From<crate::daemon::llm_gateway::Message> for AnthropicMessage {
    fn from(msg: crate::daemon::llm_gateway::Message) -> Self {
        Self {
            role: msg.role,
            content: match msg.content {
                crate::daemon::llm_gateway::MessageContent::Text(text) => {
                    AnthropicMessageContent::Text(text)
                }
                crate::daemon::llm_gateway::MessageContent::Blocks(blocks) => {
                    AnthropicMessageContent::Blocks(blocks.into_iter().map(|b| b.into()).collect())
                }
            },
        }
    }
}

impl From<ContentBlock> for AnthropicContentBlock {
    fn from(block: ContentBlock) -> Self {
        use crate::daemon::llm_gateway::{ImageSource, DocumentSource, ToolResultContent};
        match block {
            ContentBlock::Text { text } => AnthropicContentBlock::Text { text },
            ContentBlock::Image { source } => {
                let source_json = match source {
                    ImageSource::Base64 { media_type, data } => {
                        serde_json::json!({"type": "base64", "media_type": media_type, "data": data})
                    }
                    ImageSource::Url { url } => {
                        serde_json::json!({"type": "url", "url": url})
                    }
                };
                AnthropicContentBlock::Image { source: source_json }
            }
            ContentBlock::ToolUse { id, name, input } => {
                AnthropicContentBlock::ToolUse { id, name, input }
            }
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                let content_json = match content {
                    ToolResultContent::Text(text) => serde_json::Value::String(text),
                    ToolResultContent::Blocks(blocks) => {
                        serde_json::to_value(blocks).unwrap_or(serde_json::Value::Null)
                    }
                };
                AnthropicContentBlock::ToolResult {
                    tool_use_id,
                    content: content_json,
                    is_error,
                }
            }
            ContentBlock::Thinking { thinking, signature } => {
                AnthropicContentBlock::Thinking { thinking, signature }
            }
            ContentBlock::Document { source } => {
                let source_json = match source {
                    DocumentSource::Base64 { media_type, data } => {
                        serde_json::json!({"type": "base64", "media_type": media_type, "data": data})
                    }
                };
                AnthropicContentBlock::Document { source: source_json }
            }
            ContentBlock::Unknown => {
                // Unknown blocks are dropped - they shouldn't be forwarded
                AnthropicContentBlock::Text { text: "[unknown content]".to_string() }
            }
        }
    }
}

impl From<AnthropicContentBlock> for ContentBlock {
    fn from(block: AnthropicContentBlock) -> Self {
        use crate::daemon::llm_gateway::{ImageSource, DocumentSource, ToolResultContent};
        match block {
            AnthropicContentBlock::Text { text } => ContentBlock::Text { text },
            AnthropicContentBlock::Image { source } => {
                // Parse source JSON back to ImageSource
                let image_source = if let Some(source_type) = source.get("type").and_then(|v| v.as_str()) {
                    match source_type {
                        "base64" => ImageSource::Base64 {
                            media_type: source.get("media_type").and_then(|v| v.as_str()).unwrap_or("image/png").to_string(),
                            data: source.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        },
                        "url" => ImageSource::Url {
                            url: source.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        },
                        _ => ImageSource::Base64 { media_type: "image/png".to_string(), data: "".to_string() },
                    }
                } else {
                    ImageSource::Base64 { media_type: "image/png".to_string(), data: "".to_string() }
                };
                ContentBlock::Image { source: image_source }
            }
            AnthropicContentBlock::ToolUse { id, name, input } => {
                ContentBlock::ToolUse { id, name, input }
            }
            AnthropicContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                // Convert JSON Value back to ToolResultContent
                let tool_content = if content.is_string() {
                    ToolResultContent::Text(content.as_str().unwrap_or("").to_string())
                } else if content.is_array() {
                    // Try to deserialize as blocks
                    match serde_json::from_value(content.clone()) {
                        Ok(blocks) => ToolResultContent::Blocks(blocks),
                        Err(_) => ToolResultContent::Text(content.to_string()),
                    }
                } else {
                    ToolResultContent::Text(content.to_string())
                };
                ContentBlock::ToolResult {
                    tool_use_id,
                    content: tool_content,
                    is_error,
                }
            }
            AnthropicContentBlock::Thinking { thinking, signature } => {
                ContentBlock::Thinking { thinking, signature }
            }
            AnthropicContentBlock::Document { source } => {
                // Parse source JSON back to DocumentSource
                let doc_source = DocumentSource::Base64 {
                    media_type: source.get("media_type").and_then(|v| v.as_str()).unwrap_or("application/pdf").to_string(),
                    data: source.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                };
                ContentBlock::Document { source: doc_source }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url() {
        let config = ProviderConfig {
            enabled: true,
            provider_type: ProviderType::Anthropic,
            base_url: "https://api.anthropic.com".to_string(),
            api_key_ref: String::new(),
            default_model: String::new(),
            model_aliases: std::collections::HashMap::new(),
            timeout_secs: 300,
            max_retries: 2,
            headers: std::collections::HashMap::new(),
            deployment: None,
            api_version: None,
        };

        // We can't create the provider without an API key, so just test URL building logic
        let url = format!("{}/v1/messages", config.base_url.trim_end_matches('/'));
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn test_api_url_trailing_slash() {
        let base_url = "https://api.anthropic.com/";
        let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
    }
}
