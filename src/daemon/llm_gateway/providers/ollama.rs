//! Ollama provider implementation
//!
//! Supports both Ollama's native API and OpenAI-compatible API.
//! Ollama is typically local and doesn't require authentication.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::Provider;
use crate::daemon::llm_gateway::config::{ProviderConfig, ProviderType};
use crate::daemon::llm_gateway::metrics::PricingTable;
use crate::daemon::llm_gateway::{
    CompletionRequest, CompletionResponse, ContentBlock, Message, MessageContent, RequestMetrics,
    Usage,
};

/// Ollama provider
pub struct OllamaProvider {
    name: String,
    config: ProviderConfig,
    client: Client,
    pricing: PricingTable,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub async fn new(name: String, config: ProviderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self {
            name,
            config,
            client,
            pricing: PricingTable::default(),
        })
    }

    /// Build the API URL for chat completions (using OpenAI-compatible endpoint)
    fn api_url(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }

    /// Build the API URL for the native Ollama chat endpoint
    fn native_api_url(&self) -> String {
        format!("{}/api/chat", self.config.base_url.trim_end_matches('/'))
    }

    /// Convert Anthropic request to OpenAI format (for OpenAI-compatible endpoint)
    fn to_openai_request(&self, request: &CompletionRequest) -> OllamaOpenAIRequest {
        let model = self.resolve_model(&request.model);
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &request.system {
            let system_text = match system {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => {
                    // Extract text from array of content blocks
                    arr.iter()
                        .filter_map(|v| v.get("text").and_then(|t| t.as_str()))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                _ => system.to_string(),
            };
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: system_text,
            });
        }

        // Convert conversation messages
        for msg in &request.messages {
            messages.push(self.convert_message(msg));
        }

        OllamaOpenAIRequest {
            model,
            messages,
            stream: false, // Ollama streaming handled differently
            options: Some(OllamaOptions {
                num_predict: Some(request.max_tokens as i32),
                temperature: request.temperature,
                top_p: request.top_p,
                top_k: request.top_k.map(|k| k as i32),
            }),
        }
    }

    /// Convert a single message
    fn convert_message(&self, msg: &Message) -> OllamaMessage {
        let role = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => "user",
        }
        .to_string();

        let content = match &msg.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Blocks(blocks) => {
                // Concatenate text blocks
                blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        OllamaMessage { role, content }
    }

    /// Convert Ollama response to Anthropic format
    fn from_ollama_response(
        &self,
        response: OllamaOpenAIResponse,
        model: String,
    ) -> (CompletionResponse, Usage) {
        let choice = response.choices.into_iter().next().unwrap_or_default();

        let content = match choice.message.content {
            Some(text) if !text.is_empty() => vec![ContentBlock::Text { text }],
            _ => vec![],
        };

        let stop_reason = choice.finish_reason.map(|r| match r.as_str() {
            "stop" => "end_turn".to_string(),
            "length" => "max_tokens".to_string(),
            other => other.to_string(),
        });

        // Ollama may not always return usage stats
        let usage = response
            .usage
            .map(|u| Usage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            })
            .unwrap_or_default();

        let response = CompletionResponse {
            id: response
                .id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            model,
            content,
            stop_reason,
            stop_sequence: None,
            usage: usage.clone(),
            provider: String::new(),
            latency_ms: 0,
            cost_usd: 0.0, // Local models are free
        };

        (response, usage)
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }

    async fn health_check(&self) -> Result<bool> {
        // Ollama has a dedicated health/version endpoint
        let url = format!("{}/api/version", self.config.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
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

        // Map Anthropic model names to Ollama equivalents
        let model_lower = model.to_lowercase();
        if model_lower.contains("opus") {
            // Route opus-tier requests to largest available model
            "llama3.3:70b".to_string()
        } else if model_lower.contains("sonnet") {
            // Route sonnet-tier requests to medium model
            "llama3.2:latest".to_string()
        } else if model_lower.contains("haiku") {
            // Route haiku-tier requests to smaller model
            "llama3.2:3b".to_string()
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
        _client_auth: Option<String>,
        _client_beta: Option<String>,
    ) -> Result<(CompletionResponse, RequestMetrics)> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        let original_model = request.model.clone();

        // Convert to OpenAI format
        let ollama_request = self.to_openai_request(&request);
        let resolved_model = ollama_request.model.clone();

        // Make the API request (try OpenAI-compatible endpoint first)
        let response = self
            .client
            .post(self.api_url())
            .header("content-type", "application/json")
            .json(&ollama_request)
            .send()
            .await;

        // If OpenAI endpoint fails, try native endpoint
        let response = match response {
            Ok(r) if r.status().is_success() => r,
            _ => {
                // Fallback to native Ollama API
                return self.complete_native(request, &request_id, start).await;
            }
        };

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Ollama API error ({}): {}", status, response_text));
        }

        // Parse the response
        let ollama_response: OllamaOpenAIResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Ollama response: {} - {}", e, response_text))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Convert to gateway format
        let (mut response, usage) =
            self.from_ollama_response(ollama_response, resolved_model.clone());
        response.provider = self.name.clone();
        response.latency_ms = latency_ms;

        // Ollama is free (local), but we can still track "cost" for comparison
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

impl OllamaProvider {
    /// Fallback to native Ollama API
    async fn complete_native(
        &self,
        request: CompletionRequest,
        request_id: &str,
        start: std::time::Instant,
    ) -> Result<(CompletionResponse, RequestMetrics)> {
        let model = self.resolve_model(&request.model);
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &request.system {
            let system_text = match system {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => {
                    // Extract text from array of content blocks
                    arr.iter()
                        .filter_map(|v| v.get("text").and_then(|t| t.as_str()))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                _ => system.to_string(),
            };
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: system_text,
            });
        }

        // Convert conversation messages
        for msg in &request.messages {
            messages.push(self.convert_message(msg));
        }

        let native_request = OllamaNativeRequest {
            model: model.clone(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                num_predict: Some(request.max_tokens as i32),
                temperature: request.temperature,
                top_p: request.top_p,
                top_k: request.top_k.map(|k| k as i32),
            }),
        };

        let response = self
            .client
            .post(self.native_api_url())
            .header("content-type", "application/json")
            .json(&native_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!(
                "Ollama native API error ({}): {}",
                status,
                response_text
            ));
        }

        let native_response: OllamaNativeResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                anyhow!(
                    "Failed to parse Ollama native response: {} - {}",
                    e,
                    response_text
                )
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        let content = if native_response.message.content.is_empty() {
            vec![]
        } else {
            vec![ContentBlock::Text {
                text: native_response.message.content,
            }]
        };

        // Estimate tokens (Ollama native may not provide exact counts)
        let usage = Usage {
            input_tokens: native_response.prompt_eval_count.unwrap_or(0),
            output_tokens: native_response.eval_count.unwrap_or(0),
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };

        let cost_usd = self.pricing.calculate_cost(&model, &usage);

        let response = CompletionResponse {
            id: request_id.to_string(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            model: model.clone(),
            content,
            stop_reason: if native_response.done {
                Some("end_turn".to_string())
            } else {
                None
            },
            stop_sequence: None,
            usage: usage.clone(),
            provider: self.name.clone(),
            latency_ms,
            cost_usd,
        };

        let metrics = RequestMetrics::new(
            request_id.to_string(),
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
// Ollama API Types
// ============================================================================

/// OpenAI-compatible request format (Ollama /v1/chat/completions)
#[derive(Debug, Serialize)]
struct OllamaOpenAIRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Native Ollama request format (/api/chat)
#[derive(Debug, Serialize)]
struct OllamaNativeRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
}

/// OpenAI-compatible response format
#[derive(Debug, Deserialize)]
struct OllamaOpenAIResponse {
    id: Option<String>,
    choices: Vec<OllamaChoice>,
    usage: Option<OllamaUsage>,
}

#[derive(Debug, Deserialize, Default)]
struct OllamaChoice {
    message: OllamaChoiceMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct OllamaChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct OllamaUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// Native Ollama response format
#[derive(Debug, Deserialize)]
struct OllamaNativeResponse {
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_url() {
        let base_url = "http://localhost:11434";
        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
        assert_eq!(url, "http://localhost:11434/v1/chat/completions");
    }

    #[test]
    fn test_native_api_url() {
        let base_url = "http://localhost:11434";
        let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
        assert_eq!(url, "http://localhost:11434/api/chat");
    }

    #[test]
    fn test_model_mapping() {
        fn map_model(model: &str) -> &'static str {
            let model_lower = model.to_lowercase();
            if model_lower.contains("opus") {
                "llama3.3:70b"
            } else if model_lower.contains("sonnet") {
                "llama3.2:latest"
            } else if model_lower.contains("haiku") {
                "llama3.2:3b"
            } else {
                "llama3.2:latest"
            }
        }

        assert_eq!(map_model("claude-opus-4-1"), "llama3.3:70b");
        assert_eq!(map_model("claude-sonnet-4-5"), "llama3.2:latest");
        assert_eq!(map_model("claude-haiku-4-5"), "llama3.2:3b");
        assert_eq!(map_model("llama3.2"), "llama3.2:latest");
    }
}
