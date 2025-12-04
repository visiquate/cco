//! Embedded orchestra configuration
//!
//! Provides compile-time embedded orchestra-config.json for use throughout the codebase.
//! This ensures the binary works without needing the source directory present.

use serde_json::Value;
use std::sync::OnceLock;

// Import types from daemon modules
use crate::daemon::llm_gateway::config::GatewayConfig;
use crate::daemon::llm_router::router::LlmRoutingConfig;

/// Raw embedded config JSON string
const ORCHESTRA_CONFIG_STR: &str = include_str!("../config/orchestra-config.json");

/// Parsed config (lazy initialized)
static ORCHESTRA_CONFIG: OnceLock<Value> = OnceLock::new();

/// Initialize and get the parsed config
fn get_parsed_config() -> &'static Value {
    ORCHESTRA_CONFIG.get_or_init(|| {
        serde_json::from_str(ORCHESTRA_CONFIG_STR)
            .expect("Embedded orchestra-config.json should be valid JSON")
    })
}

/// Get the raw embedded config JSON string
pub fn embedded_orchestra_config_str() -> &'static str {
    ORCHESTRA_CONFIG_STR
}

/// Get the parsed embedded config
pub fn embedded_orchestra_config() -> &'static Value {
    get_parsed_config()
}

/// Get LLM routing config from embedded config
///
/// Extracts the `llmRouting` section from orchestra-config.json.
/// Returns None if the section doesn't exist.
pub fn get_llm_routing_config() -> Option<LlmRoutingConfig> {
    let config = embedded_orchestra_config();

    // Extract llmRouting section
    config.get("llmRouting").and_then(|routing_section| {
        serde_json::from_value(routing_section.clone())
            .map_err(|e| {
                eprintln!("Warning: Failed to parse llmRouting section: {}", e);
                e
            })
            .ok()
    })
}

/// Get gateway config from embedded config
///
/// Extracts the `llmGateway` section from orchestra-config.json.
/// Returns default config if the section is missing or invalid.
pub fn get_gateway_config() -> GatewayConfig {
    let config = embedded_orchestra_config();

    // Extract llmGateway section
    if let Some(gateway_section) = config.get("llmGateway") {
        serde_json::from_value(gateway_section.clone())
            .map_err(|e| {
                eprintln!("Warning: Failed to parse llmGateway section, using defaults: {}", e);
                e
            })
            .unwrap_or_default()
    } else {
        // Return default config if section doesn't exist
        GatewayConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_config_str_not_empty() {
        let config_str = embedded_orchestra_config_str();
        assert!(!config_str.is_empty());
        assert!(config_str.contains("claude-orchestra"));
    }

    #[test]
    fn test_embedded_config_parses() {
        let config = embedded_orchestra_config();
        assert!(config.is_object());

        // Verify top-level fields exist
        assert!(config.get("name").is_some());
        assert!(config.get("version").is_some());
        assert!(config.get("architect").is_some());
    }

    #[test]
    fn test_get_llm_routing_config() {
        let _routing = get_llm_routing_config();
        // May or may not exist in config - just verify it doesn't panic
        // The function returns Some(config) or None - both are valid outcomes
    }

    #[test]
    fn test_get_gateway_config() {
        let _gateway = get_gateway_config();
        // Should always return a valid config (default or from file)
        // Just verify it doesn't panic - the config is valid if we get here
    }

    #[test]
    fn test_lazy_initialization() {
        // First access initializes
        let _config1 = embedded_orchestra_config();
        // Second access reuses cached value
        let _config2 = embedded_orchestra_config();
        // Should be same reference (Lazy guarantees single initialization)
    }
}
