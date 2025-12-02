//! Azure OpenAI provider implementation
//!
//! Translates between Anthropic Messages API format and Azure OpenAI Chat format.

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

/// Azure OpenAI provider
pub struct AzureProvider {
    name: String,
    config: ProviderConfig,
    client: Client,
    api_key: String,
    pricing: PricingTable,
}

impl AzureProvider {
    /// Create a new Azure OpenAI provider
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
        let base = self.config.base_url.trim_end_matches('/');
        let deployment = self
            .config
            .deployment
            .as_deref()
            .unwrap_or("gpt-4");
        let api_version = self
            .config
            .api_version
            .as_deref()
            .unwrap_or("2024-02-15-preview");

        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            base, deployment, api_version
        )
    }

    /// Convert Anthropic request to Azure OpenAI format
    fn to_azure_request(&self, request: &CompletionRequest) -> AzureRequest {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &request.system {
            messages.push(AzureMessage {
                role: "system".to_string(),
                content: AzureContent::Text(system.clone()),
            });
        }

        // Convert conversation messages
        for msg in &request.messages {
            messages.push(self.convert_message(msg));
        }

        AzureRequest {
            messages,
            max_tokens: Some(request.max_tokens),
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop_sequences.clone(),
            stream: request.stream,
        }
    }

    /// Convert a single message
    fn convert_message(&self, msg: &Message) -> AzureMessage {
        let role = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => "user",
        }
        .to_string();

        let content = match &msg.content {
            MessageContent::Text(text) => AzureContent::Text(text.clone()),
            MessageContent::Blocks(blocks) => {
                // Convert content blocks to OpenAI format
                let parts: Vec<AzureContentPart> = blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(AzureContentPart::Text {
                            text: text.clone(),
                        }),
                        // Tool use/result blocks need special handling
                        _ => None,
                    })
                    .collect();

                if parts.len() == 1 {
                    if let AzureContentPart::Text { text } = &parts[0] {
                        AzureContent::Text(text.clone())
                    } else {
                        AzureContent::Parts(parts)
                    }
                } else {
                    AzureContent::Parts(parts)
                }
            }
        };

        AzureMessage { role, content }
    }

    /// Convert Azure response to Anthropic format
    fn from_azure_response(
        &self,
        response: AzureResponse,
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
            "content_filter" => "content_filter".to_string(),
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
            provider: String::new(), // Set by caller
            latency_ms: 0,           // Set by caller
            cost_usd: 0.0,           // Set by caller
        };

        (response, usage)
    }
}

#[async_trait]
impl Provider for AzureProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Azure
    }

    async fn health_check(&self) -> Result<bool> {
        // Azure doesn't have a health endpoint, check connectivity
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
    ) -> Result<(CompletionResponse, RequestMetrics)> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        let model = self.resolve_model(&request.model);

        // Convert to Azure format
        let azure_request = self.to_azure_request(&request);

        // Make the API request
        let response = self
            .client
            .post(self.api_url())
            .header("api-key", &self.api_key)
            .header("content-type", "application/json")
            .json(&azure_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Azure API error ({}): {}", status, response_text));
        }

        // Parse the response
        let azure_response: AzureResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Azure response: {} - {}", e, response_text))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Convert to gateway format
        let (mut response, usage) = self.from_azure_response(azure_response, model.clone());
        response.provider = self.name.clone();
        response.latency_ms = latency_ms;

        // Calculate cost
        let cost_usd = self.pricing.calculate_cost(&model, &usage);
        response.cost_usd = cost_usd;

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
}

// ============================================================================
// Azure OpenAI API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct AzureRequest {
    messages: Vec<AzureMessage>,
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
struct AzureMessage {
    role: String,
    content: AzureContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AzureContent {
    Text(String),
    Parts(Vec<AzureContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum AzureContentPart {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct AzureResponse {
    id: String,
    choices: Vec<AzureChoice>,
    usage: AzureUsage,
}

#[derive(Debug, Deserialize, Default)]
struct AzureChoice {
    message: AzureChoiceMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct AzureChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct AzureUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url() {
        let config = ProviderConfig {
            enabled: true,
            provider_type: ProviderType::Azure,
            base_url: "https://myresource.openai.azure.com".to_string(),
            api_key_ref: String::new(),
            default_model: String::new(),
            model_aliases: std::collections::HashMap::new(),
            timeout_secs: 300,
            max_retries: 2,
            headers: std::collections::HashMap::new(),
            deployment: Some("gpt-4-turbo".to_string()),
            api_version: Some("2024-02-15-preview".to_string()),
        };

        let base = config.base_url.trim_end_matches('/');
        let deployment = config.deployment.as_deref().unwrap_or("gpt-4");
        let api_version = config.api_version.as_deref().unwrap_or("2024-02-15-preview");

        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            base, deployment, api_version
        );

        assert_eq!(
            url,
            "https://myresource.openai.azure.com/openai/deployments/gpt-4-turbo/chat/completions?api-version=2024-02-15-preview"
        );
    }

    #[test]
    fn test_stop_reason_mapping() {
        fn map_stop_reason(r: &str) -> &str {
            match r {
                "stop" => "end_turn",
                "length" => "max_tokens",
                "content_filter" => "content_filter",
                _ => "unknown",
            }
        }

        assert_eq!(map_stop_reason("stop"), "end_turn");
        assert_eq!(map_stop_reason("length"), "max_tokens");
        assert_eq!(map_stop_reason("content_filter"), "content_filter");
        assert_eq!(map_stop_reason("unknown"), "unknown");
    }
}
