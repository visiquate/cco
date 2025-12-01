//! Endpoint configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LLM endpoint type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EndpointType {
    Ollama,
    OpenAI,
    Generic,
}

/// LLM endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub enabled: bool,
    pub url: String,
    #[serde(rename = "type")]
    pub endpoint_type: Option<EndpointType>,
    #[serde(rename = "defaultModel")]
    pub default_model: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "additionalParams")]
    pub additional_params: Option<serde_json::Value>,
}

impl EndpointConfig {
    /// Detect endpoint type from URL if not explicitly set
    pub fn detect_type(&self) -> EndpointType {
        if let Some(ref t) = self.endpoint_type {
            return t.clone();
        }

        // Auto-detect from URL or model name
        if self.url.contains("ollama") {
            return EndpointType::Ollama;
        }

        if let Some(ref model) = self.default_model {
            if model.contains("qwen") || model.contains("llama") || model.contains("codestral") {
                return EndpointType::Ollama;
            }
        }

        EndpointType::Generic
    }

    /// Get the API path based on endpoint type
    pub fn get_api_path(&self) -> String {
        match self.detect_type() {
            EndpointType::Ollama => "/api/generate".to_string(),
            EndpointType::OpenAI => "/v1/completions".to_string(),
            EndpointType::Generic => {
                // Use the path from URL if present
                if let Ok(parsed_url) = url::Url::parse(&self.url) {
                    parsed_url.path().to_string()
                } else {
                    "/".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ollama_from_url() {
        let config = EndpointConfig {
            enabled: true,
            url: "http://localhost:11434/api/generate".to_string(),
            endpoint_type: None,
            default_model: None,
            api_key: None,
            temperature: None,
            max_tokens: None,
            headers: None,
            additional_params: None,
        };

        assert_eq!(config.detect_type(), EndpointType::Ollama);
    }

    #[test]
    fn test_detect_ollama_from_model() {
        let config = EndpointConfig {
            enabled: true,
            url: "http://coder.example.com".to_string(),
            endpoint_type: None,
            default_model: Some("qwen2.5-coder:32b-instruct".to_string()),
            api_key: None,
            temperature: None,
            max_tokens: None,
            headers: None,
            additional_params: None,
        };

        assert_eq!(config.detect_type(), EndpointType::Ollama);
    }

    #[test]
    fn test_explicit_type_takes_precedence() {
        let config = EndpointConfig {
            enabled: true,
            url: "http://localhost:11434".to_string(),
            endpoint_type: Some(EndpointType::OpenAI),
            default_model: Some("qwen2.5-coder:32b-instruct".to_string()),
            api_key: None,
            temperature: None,
            max_tokens: None,
            headers: None,
            additional_params: None,
        };

        assert_eq!(config.detect_type(), EndpointType::OpenAI);
    }
}
