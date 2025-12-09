//! Gateway configuration
//!
//! Loaded from the `llmGateway` section of orchestra-config.json

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,

    /// Routing rules
    pub routing: RoutingConfig,

    /// Cost tracking settings
    #[serde(default)]
    pub cost_tracking: CostTrackingConfig,

    /// Audit logging settings
    #[serde(default)]
    pub audit: AuditConfig,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
            routing: RoutingConfig::default(),
            cost_tracking: CostTrackingConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Provider type
    pub provider_type: ProviderType,

    /// Base URL for API requests
    pub base_url: String,

    /// API key reference (environment variable name or keyring key)
    #[serde(default)]
    pub api_key_ref: String,

    /// Default model for this provider
    #[serde(default)]
    pub default_model: String,

    /// Model name aliases (e.g., "opus" -> "claude-opus-4-1")
    #[serde(default)]
    pub model_aliases: HashMap<String, String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Custom headers to include in requests
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Azure-specific: deployment name
    #[serde(default)]
    pub deployment: Option<String>,

    /// Azure-specific: API version
    #[serde(default)]
    pub api_version: Option<String>,
}

/// Provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Anthropic,
    Azure,
    DeepSeek,
    Ollama,
    OpenAI,
    VisiQuate,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Azure => write!(f, "azure"),
            ProviderType::DeepSeek => write!(f, "deepseek"),
            ProviderType::Ollama => write!(f, "ollama"),
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::VisiQuate => write!(f, "visiquate"),
        }
    }
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    /// Default provider when no rules match
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// Agent type to provider mapping
    #[serde(default)]
    pub agent_rules: HashMap<String, String>,

    /// Model tier to provider mapping (opus, sonnet, haiku)
    #[serde(default)]
    pub model_tier_rules: HashMap<String, String>,

    /// Fallback chain when primary provider fails
    #[serde(default)]
    pub fallback_chain: Vec<String>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            default_provider: "anthropic".to_string(),
            agent_rules: HashMap::new(),
            model_tier_rules: HashMap::new(),
            fallback_chain: vec!["anthropic".to_string()],
        }
    }
}

/// Cost tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostTrackingConfig {
    /// Enable cost tracking
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Custom pricing overrides (model -> pricing)
    #[serde(default)]
    pub pricing_overrides: HashMap<String, ModelPricing>,
}

impl Default for CostTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pricing_overrides: HashMap::new(),
        }
    }
}

/// Model pricing (USD per million tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    #[serde(default)]
    pub cache_write_per_million: Option<f64>,
    #[serde(default)]
    pub cache_read_per_million: Option<f64>,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditConfig {
    /// Enable audit logging
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Log full request bodies
    #[serde(default = "default_true")]
    pub log_request_bodies: bool,

    /// Log full response bodies
    #[serde(default = "default_true")]
    pub log_response_bodies: bool,

    /// Retention period in days
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// Database path (defaults to ~/.cco/audit.db)
    #[serde(default)]
    pub db_path: Option<PathBuf>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_request_bodies: true,
            log_response_bodies: true,
            retention_days: 30,
            db_path: None,
        }
    }
}

impl AuditConfig {
    /// Get the database path, using default if not specified
    pub fn get_db_path(&self) -> PathBuf {
        self.db_path.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cco")
                .join("audit.db")
        })
    }
}

// Default value functions for serde
fn default_true() -> bool {
    true
}

fn default_timeout_secs() -> u64 {
    300
}

fn default_max_retries() -> u32 {
    2
}

fn default_provider() -> String {
    "anthropic".to_string()
}

fn default_retention_days() -> u32 {
    30
}

/// Load gateway config from orchestra-config.json
pub fn load_from_orchestra_config(config_path: Option<PathBuf>) -> anyhow::Result<GatewayConfig> {
    let config_str = if let Some(path) = config_path {
        // Allow explicit path override for testing
        std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read config from {:?}: {}", path, e))?
    } else {
        // Use embedded config (default)
        crate::embedded_config::embedded_orchestra_config_str().to_string()
    };

    let full_config: serde_json::Value = serde_json::from_str(&config_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse config JSON: {}", e))?;

    // Extract llmGateway section
    if let Some(gateway_config) = full_config.get("llmGateway") {
        serde_json::from_value(gateway_config.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse llmGateway section: {}", e))
    } else {
        // Return default config if section doesn't exist
        Ok(GatewayConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::Anthropic.to_string(), "anthropic");
        assert_eq!(ProviderType::Azure.to_string(), "azure");
        assert_eq!(ProviderType::DeepSeek.to_string(), "deepseek");
        assert_eq!(ProviderType::Ollama.to_string(), "ollama");
    }

    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert_eq!(config.routing.default_provider, "anthropic");
        assert!(config.cost_tracking.enabled);
        assert!(config.audit.enabled);
    }

    #[test]
    fn test_audit_config_db_path() {
        let config = AuditConfig::default();
        let path = config.get_db_path();
        assert!(path.to_string_lossy().contains("audit.db"));
    }

    #[test]
    fn test_deserialize_provider_config() {
        let json = r#"{
            "enabled": true,
            "providerType": "anthropic",
            "baseUrl": "https://api.anthropic.com",
            "apiKeyRef": "ANTHROPIC_API_KEY",
            "defaultModel": "claude-sonnet-4-5-20250929",
            "timeoutSecs": 300,
            "maxRetries": 2
        }"#;

        let config: ProviderConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.provider_type, ProviderType::Anthropic);
        assert_eq!(config.base_url, "https://api.anthropic.com");
    }
}
