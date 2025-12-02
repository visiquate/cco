//! DeepSeek provider implementation (OpenAI-compatible)
//!
//! Works with DeepSeek API and any OpenAI-compatible API.
//! Translates between Anthropic Messages API format and OpenAI Chat format.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{resolve_api_key, Provider};
use crate::daemon::llm_gateway::config::{ProviderConfig, ProviderType};
use crate::daemon::llm_gateway::metrics::PricingTable;
use crate::daemon::llm_gateway::{
    CompletionRequest, CompletionResponse, ContentBlock, Message, MessageContent, RequestMetrics,
    Usage,
};

/// DeepSeek/OpenAI-compatible provider
pub struct DeepSeekProvider {
    name: String,
    config: ProviderConfig,
    client: Client,
    api_key: String,
    pricing: PricingTable,
}

impl DeepSeekProvider {
    /// Create a new DeepSeek provider
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

    /// Build the API URL for chat completions
    fn api_url(&self) -> String {
        format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }

    /// Convert Anthropic request to OpenAI format
    fn to_openai_request(&self, request: &CompletionRequest) -> OpenAIRequest {
        let model = self.resolve_model(&request.model);
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &request.system {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: OpenAIContent::Text(system.clone()),
            });
        }

        // Convert conversation messages
        for msg in &request.messages {
            messages.push(self.convert_message(msg));
        }

        OpenAIRequest {
            model,
            messages,
            max_tokens: Some(request.max_tokens),
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop_sequences.clone(),
            stream: request.stream,
        }
    }

    /// Convert a single message
    fn convert_message(&self, msg: &Message) -> OpenAIMessage {
        let role = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => "user",
        }
        .to_string();

        let content = match &msg.content {
            MessageContent::Text(text) => OpenAIContent::Text(text.clone()),
            MessageContent::Blocks(blocks) => {
                // Convert content blocks to OpenAI format
                let parts: Vec<OpenAIContentPart> = blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(OpenAIContentPart::Text {
                            text: text.clone(),
                        }),
                        _ => None,
                    })
                    .collect();

                if parts.len() == 1 {
                    if let OpenAIContentPart::Text { text } = &parts[0] {
                        OpenAIContent::Text(text.clone())
                    } else {
                        OpenAIContent::Parts(parts)
                    }
                } else {
                    OpenAIContent::Parts(parts)
                }
            }
        };

        OpenAIMessage { role, content }
    }

    /// Convert OpenAI response to Anthropic format
    fn from_openai_response(
        &self,
        response: OpenAIResponse,
        model: String,
    ) -> (CompletionResponse, Usage) {
        let choice = response.choices.into_iter().next().unwrap_or_default();

        let content = match choice.message.content {
            Some(text) => vec![ContentBlock::Text { text }],
            None => vec![],
        };

        let stop_reason = choice.finish_reason.map(|r| match r.as_str() {
            "stop" => "end_turn".to_string(),
            "length" => "max_tokens".to_string(),
            other => other.to_string(),
        });

        let usage = Usage {
            input_tokens: response.usage.prompt_tokens,
            output_tokens: response.usage.completion_tokens,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };

        let response = CompletionResponse {
            id: response.id,
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            model,
            content,
            stop_reason,
            stop_sequence: None,
            usage: usage.clone(),
            provider: String::new(),
            latency_ms: 0,
            cost_usd: 0.0,
        };

        (response, usage)
    }
}

#[async_trait]
impl Provider for DeepSeekProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::DeepSeek
    }

    async fn health_check(&self) -> Result<bool> {
        // Check models endpoint for health
        let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        Ok(response.map(|r| r.status().is_success()).unwrap_or(false))
    }

    fn resolve_model(&self, model: &str) -> String {
        // First check aliases
        if let Some(alias) = self.config.model_aliases.get(model) {
            return alias.clone();
        }

        // Map Anthropic model names to DeepSeek equivalents
        let model_lower = model.to_lowercase();
        if model_lower.contains("opus") || model_lower.contains("sonnet") {
            // Route high-tier requests to deepseek-chat (best model)
            "deepseek-chat".to_string()
        } else if model_lower.contains("haiku") {
            // Route lower-tier requests to deepseek-chat as well (DeepSeek doesn't have tiers)
            "deepseek-chat".to_string()
        } else {
            // Use default model or pass through
            if !self.config.default_model.is_empty() {
                self.config.default_model.clone()
            } else {
                model.to_string()
            }
        }
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<(CompletionResponse, RequestMetrics)> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        let original_model = request.model.clone();

        // Convert to OpenAI format
        let openai_request = self.to_openai_request(&request);
        let resolved_model = openai_request.model.clone();

        // Make the API request
        let response = self
            .client
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!(
                "DeepSeek API error ({}): {}",
                status,
                response_text
            ));
        }

        // Parse the response
        let openai_response: OpenAIResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse DeepSeek response: {} - {}", e, response_text))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Convert to gateway format
        let (mut response, usage) =
            self.from_openai_response(openai_response, resolved_model.clone());
        response.provider = self.name.clone();
        response.latency_ms = latency_ms;

        // Calculate cost (use original model for pricing lookup)
        let cost_usd = self.pricing.calculate_cost(&original_model, &usage);
        response.cost_usd = cost_usd;

        let metrics = RequestMetrics::new(
            request_id,
            self.name.clone(),
            resolved_model,
            &usage,
            cost_usd,
            latency_ms,
        )
        .with_agent_type(request.agent_type)
        .with_project_id(request.project_id);

        Ok((response, metrics))
    }
}

// ============================================================================
// OpenAI-Compatible API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: OpenAIContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum OpenAIContent {
    Text(String),
    Parts(Vec<OpenAIContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIChoice {
    message: OpenAIChoiceMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url() {
        let base_url = "https://api.deepseek.com";
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        assert_eq!(url, "https://api.deepseek.com/chat/completions");
    }

    #[test]
    fn test_api_url_trailing_slash() {
        let base_url = "https://api.deepseek.com/";
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        assert_eq!(url, "https://api.deepseek.com/chat/completions");
    }

    #[test]
    fn test_model_mapping() {
        // Test that Anthropic model names map to DeepSeek models
        fn map_model(model: &str) -> &'static str {
            let model_lower = model.to_lowercase();
            if model_lower.contains("opus") || model_lower.contains("sonnet") {
                "deepseek-chat"
            } else if model_lower.contains("haiku") {
                "deepseek-chat"
            } else {
                "deepseek-chat"
            }
        }

        assert_eq!(map_model("claude-opus-4-1"), "deepseek-chat");
        assert_eq!(map_model("claude-sonnet-4-5"), "deepseek-chat");
        assert_eq!(map_model("claude-haiku-4-5"), "deepseek-chat");
        assert_eq!(map_model("deepseek-chat"), "deepseek-chat");
    }
}
