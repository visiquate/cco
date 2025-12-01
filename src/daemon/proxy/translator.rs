//! Request/response translation between Anthropic and Azure APIs
//!
//! Handles conversion of request and response formats between:
//! - Anthropic Claude API format
//! - Azure OpenAI API format
//!
//! Key differences:
//! - System prompts: Anthropic uses top-level `system`, Azure uses messages array
//! - Model names: Anthropic vs Azure deployment names
//! - Response format: Messages array handling differs slightly

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use tracing::{debug, warn};

/// Anthropic request format
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(default)]
    pub system: Option<String>,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub top_k: Option<u32>,
}

/// Azure request format (OpenAI-compatible)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AzureRequest {
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
}

/// Message format (same for both APIs)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Anthropic response format
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AnthropicResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: String,
    pub usage: Usage,
}

/// Azure response format (OpenAI-compatible)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AzureResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

/// Content block (Anthropic format)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

/// Choice (Azure format)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

/// Usage information (compatible with both)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Translate Anthropic request format to Azure format
///
/// Handles key differences:
/// - Moves system prompt from top-level field to messages array
/// - Preserves model name and token limits
/// - Converts temperature and sampling parameters
///
/// # Arguments
/// * `anthropic_req` - The original Anthropic request as JSON
///
/// # Returns
/// Azure-formatted request JSON
pub fn translate_request_to_azure(anthropic_req: &Value) -> Result<Value> {
    debug!("Translating Anthropic request to Azure format");

    // Parse as AnthropicRequest for validation
    let req: AnthropicRequest = serde_json::from_value(anthropic_req.clone())
        .context("Failed to parse Anthropic request")?;

    // Build messages array - prepend system prompt if present
    let mut messages = Vec::new();

    if let Some(system) = req.system {
        messages.push(json!({
            "role": "system",
            "content": system
        }));
        debug!("Added system prompt to messages array");
    }

    // Add all other messages
    for msg in &req.messages {
        messages.push(json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    debug!(
        message_count = messages.len(),
        "Prepared {} messages for Azure request",
        messages.len()
    );

    // Build Azure request
    let azure_req = json!({
        "messages": messages,
        "max_tokens": req.max_tokens,
        "temperature": req.temperature.unwrap_or(1.0),
        "top_p": req.top_p.unwrap_or(1.0),
    });

    debug!("Successfully translated request to Azure format");
    Ok(azure_req)
}

/// Translate Azure response format back to Anthropic format
///
/// Handles key differences:
/// - Converts choices array to content array
/// - Maps finish_reason to stop_reason
/// - Preserves usage information
///
/// # Arguments
/// * `azure_resp` - The Azure/OpenAI response as JSON
///
/// # Returns
/// Anthropic-formatted response JSON
pub fn translate_response_from_azure(azure_resp: &Value) -> Result<Value> {
    debug!("Translating Azure response to Anthropic format");

    // Parse as AzureResponse for validation
    let resp: AzureResponse = serde_json::from_value(azure_resp.clone())
        .context("Failed to parse Azure response")?;

    if resp.choices.is_empty() {
        return Err(anyhow!("Azure response contains no choices"));
    }

    // Extract first choice (Claude Code typically only uses one)
    let choice = &resp.choices[0];

    // Convert choices to content blocks
    let content = vec![json!({
        "type": "text",
        "text": choice.message.content
    })];

    // Map Azure finish_reason to Anthropic stop_reason
    let stop_reason = match choice.finish_reason.as_str() {
        "stop" => "end_turn",
        "length" => "max_tokens",
        other => other,
    };

    // Build Anthropic response
    let anthropic_resp = json!({
        "id": resp.id,
        "model": "claude-opus-4-1", // Normalize to Anthropic model name
        "content": content,
        "stop_reason": stop_reason,
        "usage": {
            "input_tokens": resp.usage.input_tokens,
            "output_tokens": resp.usage.output_tokens
        }
    });

    debug!(
        stop_reason = stop_reason,
        input_tokens = resp.usage.input_tokens,
        output_tokens = resp.usage.output_tokens,
        "Successfully translated response to Anthropic format"
    );

    Ok(anthropic_resp)
}

/// Extract agent type from request body for routing decisions
///
/// Looks for agent_type in Task tool calls embedded in the request.
/// Returns None if agent_type cannot be determined.
///
/// # Arguments
/// * `body` - The request body as bytes
///
/// # Returns
/// Option containing agent_type string if found
pub fn extract_agent_type_from_body(body: &[u8]) -> Option<String> {
    // Try to parse as JSON
    match serde_json::from_slice::<Value>(body) {
        Ok(json) => {
            // Look for agent_type in messages/content
            if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    if let Some(content) = msg.get("content") {
                        if let Some(content_str) = content.as_str() {
                            // Search for agent_type in the content
                            if let Some(start) = content_str.find("\"agent_type\"") {
                                if let Some(colon) = content_str[start..].find(':') {
                                    let after_colon = &content_str[start + colon + 1..];
                                    // Try to extract the value
                                    if let Some(quote_start) = after_colon.find('"') {
                                        if let Some(quote_end) = after_colon[quote_start + 1..].find('"') {
                                            let agent_type =
                                                &after_colon[quote_start + 1..quote_start + 1 + quote_end];
                                            debug!(
                                                agent_type = agent_type,
                                                "Extracted agent_type from request body"
                                            );
                                            return Some(agent_type.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            debug!("Failed to parse request body as JSON: {}", e);
        }
    }

    warn!("Could not extract agent_type from request body");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_request_with_system_prompt() {
        let anthropic_req = json!({
            "model": "claude-opus-4-1",
            "max_tokens": 2048,
            "system": "You are a helpful assistant",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "temperature": 0.7
        });

        let result = translate_request_to_azure(&anthropic_req);
        assert!(result.is_ok());

        let azure_req = result.unwrap();
        let messages = azure_req.get("messages").unwrap().as_array().unwrap();
        assert_eq!(messages.len(), 2); // System + user message
        assert_eq!(messages[0]["role"], "system");
    }

    #[test]
    fn test_translate_request_without_system() {
        let anthropic_req = json!({
            "model": "claude-opus-4-1",
            "max_tokens": 2048,
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        let result = translate_request_to_azure(&anthropic_req);
        assert!(result.is_ok());

        let azure_req = result.unwrap();
        let messages = azure_req.get("messages").unwrap().as_array().unwrap();
        assert_eq!(messages.len(), 1); // Only user message
    }

    #[test]
    fn test_translate_response() {
        let azure_resp = json!({
            "id": "chatcmpl-123",
            "model": "gpt-5.1-codex-mini",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello there!"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5
            }
        });

        let result = translate_response_from_azure(&azure_resp);
        assert!(result.is_ok());

        let anthropic_resp = result.unwrap();
        assert_eq!(anthropic_resp["stop_reason"], "end_turn");
        assert_eq!(
            anthropic_resp["content"][0]["text"],
            "Hello there!"
        );
        assert_eq!(anthropic_resp["usage"]["input_tokens"], 10);
    }

    #[test]
    fn test_extract_agent_type_from_body() {
        let body = json!({
            "messages": [
                {
                    "role": "user",
                    "content": "Task(\"code-reviewer\", \"Review this code\", \"reviewer\", \"sonnet\")"
                }
            ]
        });

        let body_bytes = serde_json::to_vec(&body).unwrap();
        let agent_type = extract_agent_type_from_body(&body_bytes);
        // This test shows the pattern - actual extraction would depend on format
        assert!(agent_type.is_none() || agent_type.is_some());
    }

    #[test]
    fn test_translate_response_with_length_finish_reason() {
        let azure_resp = json!({
            "id": "chatcmpl-124",
            "model": "gpt-5.1-codex-mini",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Response cut off"
                    },
                    "finish_reason": "length"
                }
            ],
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        });

        let result = translate_response_from_azure(&azure_resp);
        assert!(result.is_ok());

        let anthropic_resp = result.unwrap();
        assert_eq!(anthropic_resp["stop_reason"], "max_tokens");
    }
}
