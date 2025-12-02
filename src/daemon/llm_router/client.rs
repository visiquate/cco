//! HTTP client for calling custom LLM endpoints

use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::endpoints::{EndpointConfig, EndpointType};

/// LLM request options
#[derive(Debug, Clone, Default)]
pub struct LlmOptions {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
    pub model: Option<String>,
    pub raw: serde_json::Value,
}

/// HTTP client for LLM endpoints
pub struct LlmClient {
    client: reqwest::Client,
}

impl LlmClient {
    /// Create new LLM client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Call custom LLM endpoint
    pub async fn call_endpoint(
        &self,
        endpoint: &EndpointConfig,
        prompt: &str,
        options: LlmOptions,
        bearer_token: Option<String>,
    ) -> Result<LlmResponse> {
        if !endpoint.enabled {
            return Err(anyhow!("Endpoint is not enabled"));
        }

        let endpoint_type = endpoint.detect_type();
        let url = self.build_url(endpoint, &endpoint_type)?;
        let request_body = self.build_request_body(endpoint, &endpoint_type, prompt, &options)?;

        // Build headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        // Add bearer token if available (prioritize parameter over endpoint config)
        if let Some(token) = bearer_token.or_else(|| endpoint.api_key.clone()) {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        // Add custom headers from config
        if let Some(ref custom_headers) = endpoint.headers {
            for (key, value) in custom_headers {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(value),
                ) {
                    headers.insert(name, val);
                }
            }
        }

        // Make request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("HTTP {}: {}", status, body));
        }

        let response_data: serde_json::Value = response.json().await?;

        // Extract text from response based on format
        let text = self.extract_response_text(&endpoint_type, &response_data)?;
        let model = response_data.get("model").and_then(|m| m.as_str()).map(String::from);

        Ok(LlmResponse {
            text,
            model,
            raw: response_data,
        })
    }

    /// Build full URL for request
    fn build_url(&self, endpoint: &EndpointConfig, endpoint_type: &EndpointType) -> Result<String> {
        let base_url = endpoint.url.trim_end_matches('/');

        match endpoint_type {
            EndpointType::Ollama => Ok(format!("{}/api/generate", base_url)),
            EndpointType::OpenAI => {
                // Use URL as-is if it already has a path, otherwise append /v1/completions
                if base_url.contains("/v1/") || base_url.ends_with("/completions") {
                    Ok(base_url.to_string())
                } else {
                    Ok(format!("{}/v1/completions", base_url))
                }
            }
            EndpointType::Generic => Ok(base_url.to_string()),
        }
    }

    /// Build request body based on endpoint type
    fn build_request_body(
        &self,
        endpoint: &EndpointConfig,
        endpoint_type: &EndpointType,
        prompt: &str,
        options: &LlmOptions,
    ) -> Result<serde_json::Value> {
        let model = options
            .model
            .clone()
            .or_else(|| endpoint.default_model.clone())
            .unwrap_or_else(|| "default".to_string());

        let temperature = options
            .temperature
            .or(endpoint.temperature)
            .unwrap_or(0.7);

        let max_tokens = options.max_tokens.or(endpoint.max_tokens).unwrap_or(4096);

        match endpoint_type {
            EndpointType::Ollama => {
                let mut body = json!({
                    "model": model,
                    "prompt": prompt,
                    "stream": false,
                    "options": {
                        "temperature": temperature,
                        "num_predict": max_tokens,
                    }
                });

                // Merge additional params if present
                if let Some(ref params) = endpoint.additional_params {
                    if let Some(obj) = body.as_object_mut() {
                        if let Some(params_obj) = params.as_object() {
                            for (key, value) in params_obj {
                                obj.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }

                Ok(body)
            }
            EndpointType::OpenAI | EndpointType::Generic => {
                let mut body = json!({
                    "prompt": prompt,
                    "model": model,
                    "temperature": temperature,
                    "max_tokens": max_tokens,
                });

                // Merge additional params if present
                if let Some(ref params) = endpoint.additional_params {
                    if let Some(obj) = body.as_object_mut() {
                        if let Some(params_obj) = params.as_object() {
                            for (key, value) in params_obj {
                                obj.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }

                Ok(body)
            }
        }
    }

    /// Extract response text from response data based on format
    fn extract_response_text(
        &self,
        endpoint_type: &EndpointType,
        response: &serde_json::Value,
    ) -> Result<String> {
        match endpoint_type {
            EndpointType::Ollama => {
                // Ollama format: {"response": "text"}
                response
                    .get("response")
                    .and_then(|r| r.as_str())
                    .map(String::from)
                    .ok_or_else(|| anyhow!("No 'response' field in Ollama response"))
            }
            EndpointType::OpenAI | EndpointType::Generic => {
                // OpenAI format: {"choices": [{"text": "..."} or {"message": {"content": "..."}}]}
                if let Some(choices) = response.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        // Try text field first
                        if let Some(text) = choice.get("text").and_then(|t| t.as_str()) {
                            return Ok(text.to_string());
                        }
                        // Try message.content field
                        if let Some(content) = choice
                            .get("message")
                            .and_then(|m| m.get("content"))
                            .and_then(|c| c.as_str())
                        {
                            return Ok(content.to_string());
                        }
                    }
                }

                // Fallback: return entire response as JSON string
                Ok(serde_json::to_string_pretty(response)?)
            }
        }
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ollama_url() {
        let client = LlmClient::new();
        let endpoint = EndpointConfig {
            enabled: true,
            url: "http://localhost:11434".to_string(),
            endpoint_type: Some(EndpointType::Ollama),
            default_model: None,
            api_key: None,
            temperature: None,
            max_tokens: None,
            headers: None,
            additional_params: None,
        };

        let url = client.build_url(&endpoint, &EndpointType::Ollama).unwrap();
        assert_eq!(url, "http://localhost:11434/api/generate");
    }

    #[test]
    fn test_build_openai_url() {
        let client = LlmClient::new();
        let endpoint = EndpointConfig {
            enabled: true,
            url: "https://api.openai.com".to_string(),
            endpoint_type: Some(EndpointType::OpenAI),
            default_model: None,
            api_key: None,
            temperature: None,
            max_tokens: None,
            headers: None,
            additional_params: None,
        };

        let url = client.build_url(&endpoint, &EndpointType::OpenAI).unwrap();
        assert_eq!(url, "https://api.openai.com/v1/completions");
    }

    #[test]
    fn test_build_ollama_request_body() {
        let client = LlmClient::new();
        let endpoint = EndpointConfig {
            enabled: true,
            url: "http://localhost:11434".to_string(),
            endpoint_type: Some(EndpointType::Ollama),
            default_model: Some("qwen2.5-coder:32b-instruct".to_string()),
            api_key: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            headers: None,
            additional_params: None,
        };

        let body = client
            .build_request_body(
                &endpoint,
                &EndpointType::Ollama,
                "Test prompt",
                &LlmOptions::default(),
            )
            .unwrap();

        assert_eq!(body["model"], "qwen2.5-coder:32b-instruct");
        assert_eq!(body["prompt"], "Test prompt");
        assert_eq!(body["stream"], false);
        // Use approximate comparison for float to handle f32 precision
        let temp = body["options"]["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001, "Temperature {} is not close to 0.7", temp);
        assert_eq!(body["options"]["num_predict"], 4096);
    }

    #[test]
    fn test_extract_ollama_response() {
        let client = LlmClient::new();
        let response = json!({
            "model": "qwen2.5-coder:32b-instruct",
            "response": "This is the response text"
        });

        let text = client
            .extract_response_text(&EndpointType::Ollama, &response)
            .unwrap();
        assert_eq!(text, "This is the response text");
    }

    #[test]
    fn test_extract_openai_response_text_field() {
        let client = LlmClient::new();
        let response = json!({
            "choices": [
                {"text": "This is the response"}
            ]
        });

        let text = client
            .extract_response_text(&EndpointType::OpenAI, &response)
            .unwrap();
        assert_eq!(text, "This is the response");
    }

    #[test]
    fn test_extract_openai_response_message_field() {
        let client = LlmClient::new();
        let response = json!({
            "choices": [
                {"message": {"content": "This is the message content"}}
            ]
        });

        let text = client
            .extract_response_text(&EndpointType::OpenAI, &response)
            .unwrap();
        assert_eq!(text, "This is the message content");
    }
}
