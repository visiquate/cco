//! Core routing logic for LLM endpoint selection

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::endpoints::EndpointConfig;

/// LLM routing configuration from orchestra-config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRoutingConfig {
    pub endpoints: HashMap<String, EndpointConfig>,
    #[serde(default)]
    pub rules: HashMap<String, String>,
}

/// Routing decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub endpoint: String,
    pub use_claude_code: bool,
    pub url: Option<String>,
    pub reason: String,
}

/// LLM Router - routes tasks to appropriate endpoints
pub struct LlmRouter {
    config: LlmRoutingConfig,
    credential_store: Option<Arc<keyring::Entry>>,
}

impl LlmRouter {
    /// Create new router with configuration
    pub fn new(config: LlmRoutingConfig) -> Self {
        Self {
            config,
            credential_store: None,
        }
    }

    /// Load router from orchestra config file
    pub fn from_orchestra_config(config_path: Option<PathBuf>) -> Result<Self> {
        let config_str = if let Some(path) = config_path {
            // Allow explicit path override for testing
            std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read orchestra config: {:?}", path))?
        } else {
            // Use embedded config (default)
            crate::embedded_config::embedded_orchestra_config_str().to_string()
        };

        let full_config: serde_json::Value =
            serde_json::from_str(&config_str).context("Failed to parse orchestra config JSON")?;

        // Extract llmRouting section
        let routing_config = if let Some(llm_routing) = full_config.get("llmRouting") {
            serde_json::from_value(llm_routing.clone())
                .context("Failed to parse llmRouting section")?
        } else {
            // Default empty config if no llmRouting section
            LlmRoutingConfig {
                endpoints: HashMap::new(),
                rules: HashMap::new(),
            }
        };

        Ok(Self::new(routing_config))
    }

    /// Create router with credential store access
    pub fn with_credentials(config: LlmRoutingConfig, service: &str, key: &str) -> Self {
        let credential_store = keyring::Entry::new(service, key).ok().map(Arc::new);
        Self {
            config,
            credential_store,
        }
    }

    /// Route a task based on agent and task type
    pub fn route_task(&self, agent_type: &str, task_type: Option<&str>) -> RoutingDecision {
        // Architecture and planning always go to Claude
        if self.is_architecture_task(agent_type, task_type) {
            return RoutingDecision {
                endpoint: "claude".to_string(),
                use_claude_code: true,
                url: None,
                reason: "Architecture and planning tasks use Claude".to_string(),
            };
        }

        // Coding tasks can be routed to custom endpoint
        if self.is_coding_task(agent_type, task_type) {
            if let Some(coding_endpoint) = self.config.endpoints.get("coding") {
                if coding_endpoint.enabled {
                    return RoutingDecision {
                        endpoint: "custom".to_string(),
                        use_claude_code: false,
                        url: Some(coding_endpoint.url.clone()),
                        reason: "Coding tasks routed to custom LLM".to_string(),
                    };
                }
            }
        }

        // Default to Claude via Claude Code
        RoutingDecision {
            endpoint: "claude".to_string(),
            use_claude_code: true,
            url: None,
            reason: "Default routing to Claude".to_string(),
        }
    }

    /// Check if task is architecture/planning related
    fn is_architecture_task(&self, agent_type: &str, task_type: Option<&str>) -> bool {
        const ARCHITECTURE_TYPES: &[&str] = &[
            "system-architect",
            "chief-architect",
            "architecture",
            "specification",
            "planner",
        ];

        const ARCHITECTURE_TASKS: &[&str] = &[
            "design",
            "architecture",
            "planning",
            "specification",
            "requirements",
            "coordination",
        ];

        if ARCHITECTURE_TYPES.contains(&agent_type) {
            return true;
        }

        if let Some(task) = task_type {
            let task_lower = task.to_lowercase();
            if ARCHITECTURE_TASKS.iter().any(|t| task_lower.contains(t)) {
                return true;
            }
        }

        false
    }

    /// Check if task is coding related
    fn is_coding_task(&self, agent_type: &str, task_type: Option<&str>) -> bool {
        const CODING_TYPES: &[&str] = &[
            "python-expert",
            "python-specialist",
            "ios-developer",
            "swift-specialist",
            "backend-dev",
            "mobile-developer",
            "coder",
            "frontend-dev",
            "deployment-engineer",
            "go-specialist",
            "rust-specialist",
            "flutter-specialist",
            "tdd-coding-agent",
        ];

        const CODING_TASKS: &[&str] = &[
            "implement",
            "code",
            "develop",
            "build",
            "write code",
            "programming",
            "refactor",
        ];

        if CODING_TYPES.contains(&agent_type) {
            return true;
        }

        if let Some(task) = task_type {
            let task_lower = task.to_lowercase();
            if CODING_TASKS.iter().any(|t| task_lower.contains(t)) {
                return true;
            }
        }

        false
    }

    /// Get routing statistics
    pub fn get_stats(&self) -> RoutingStats {
        let endpoints: Vec<EndpointInfo> = self
            .config
            .endpoints
            .iter()
            .map(|(name, config)| EndpointInfo {
                name: name.clone(),
                enabled: config.enabled,
                url: config.url.clone(),
            })
            .collect();

        let coding_endpoint_msg = if let Some(coding) = self.config.endpoints.get("coding") {
            if coding.enabled {
                format!("Route to {}", coding.url)
            } else {
                "Route to Claude (custom endpoint disabled)".to_string()
            }
        } else {
            "Route to Claude (custom endpoint not configured)".to_string()
        };

        RoutingStats {
            endpoints,
            rules: self.config.rules.clone(),
            architecture_tasks: "Always route to Claude".to_string(),
            coding_tasks: coding_endpoint_msg,
        }
    }

    /// Get bearer token from environment or credential store
    pub async fn get_bearer_token(&self, hostname: &str) -> Option<String> {
        // Special handling for coder.visiquate.com
        if hostname == "coder.visiquate.com" {
            // Try environment variable first
            if let Ok(token) = std::env::var("CODER_LLM_TOKEN") {
                return Some(token);
            }

            // Fallback to credential store
            if let Some(ref store) = self.credential_store {
                if let Ok(token) = store.get_password() {
                    return Some(token);
                }
            }
        }

        None
    }

    /// Get endpoint configuration
    pub fn get_endpoint(&self, name: &str) -> Option<&EndpointConfig> {
        self.config.endpoints.get(name)
    }
}

/// Endpoint information for stats
#[derive(Debug, Clone, Serialize)]
pub struct EndpointInfo {
    pub name: String,
    pub enabled: bool,
    pub url: String,
}

/// Routing statistics
#[derive(Debug, Clone, Serialize)]
pub struct RoutingStats {
    pub endpoints: Vec<EndpointInfo>,
    pub rules: HashMap<String, String>,
    pub architecture_tasks: String,
    pub coding_tasks: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> LlmRoutingConfig {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "coding".to_string(),
            EndpointConfig {
                enabled: true,
                url: "http://coder.visiquate.com/api/generate".to_string(),
                endpoint_type: Some(crate::daemon::llm_router::endpoints::EndpointType::Ollama),
                default_model: Some("qwen2.5-coder:32b-instruct".to_string()),
                api_key: None,
                temperature: Some(0.7),
                max_tokens: Some(4096),
                headers: None,
                additional_params: None,
            },
        );

        LlmRoutingConfig {
            endpoints,
            rules: HashMap::new(),
        }
    }

    #[test]
    fn test_architecture_task_routing() {
        let router = LlmRouter::new(test_config());

        let decision = router.route_task("chief-architect", Some("design system"));
        assert!(decision.use_claude_code);
        assert_eq!(decision.endpoint, "claude");
    }

    #[test]
    fn test_coding_task_with_custom_endpoint() {
        let router = LlmRouter::new(test_config());

        let decision = router.route_task("python-specialist", Some("implement API"));
        assert!(!decision.use_claude_code);
        assert_eq!(decision.endpoint, "custom");
        assert!(decision.url.is_some());
    }

    #[test]
    fn test_coding_task_without_custom_endpoint() {
        let mut config = test_config();
        config.endpoints.get_mut("coding").unwrap().enabled = false;

        let router = LlmRouter::new(config);

        let decision = router.route_task("python-specialist", Some("implement API"));
        assert!(decision.use_claude_code);
        assert_eq!(decision.endpoint, "claude");
    }

    #[test]
    fn test_is_architecture_task() {
        let router = LlmRouter::new(test_config());

        assert!(router.is_architecture_task("chief-architect", None));
        assert!(router.is_architecture_task("system-architect", None));
        assert!(router.is_architecture_task("unknown", Some("design architecture")));
        assert!(!router.is_architecture_task("python-specialist", Some("implement feature")));
    }

    #[test]
    fn test_is_coding_task() {
        let router = LlmRouter::new(test_config());

        assert!(router.is_coding_task("python-specialist", None));
        assert!(router.is_coding_task("rust-specialist", None));
        assert!(router.is_coding_task("unknown", Some("implement the feature")));
        assert!(!router.is_coding_task("chief-architect", Some("design system")));
    }
}
