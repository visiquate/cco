//! Intelligent routing engine
//!
//! Routes requests to providers based on:
//! 1. Agent type rules (chief-architect → anthropic)
//! 2. Model tier rules (opus/sonnet/haiku → anthropic)
//! 3. Default provider fallback

use super::config::RoutingConfig;
use super::CompletionRequest;

/// Routing decision
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// Provider to use
    pub provider: String,
    /// Reason for the routing decision
    pub reason: String,
    /// Fallback providers if primary fails
    pub fallbacks: Vec<String>,
}

/// Routing engine
pub struct RoutingEngine {
    config: RoutingConfig,
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new(config: RoutingConfig) -> Self {
        Self { config }
    }

    /// Route a completion request to a provider
    pub fn route(&self, request: &CompletionRequest) -> RouteDecision {
        // 1. Check agent type rules
        if let Some(agent_type) = &request.agent_type {
            let agent_lower = agent_type.to_lowercase();
            if let Some(provider) = self.config.agent_rules.get(&agent_lower) {
                return RouteDecision {
                    provider: provider.clone(),
                    reason: format!("agent_rule:{}", agent_type),
                    fallbacks: self.get_fallbacks(provider),
                };
            }
        }

        // 2. Check model tier rules
        let tier = extract_model_tier(&request.model);
        if let Some(provider) = self.config.model_tier_rules.get(&tier) {
            return RouteDecision {
                provider: provider.clone(),
                reason: format!("model_tier:{}", tier),
                fallbacks: self.get_fallbacks(provider),
            };
        }

        // 3. Try to infer agent type from system prompt or messages
        if let Some(inferred_agent) = self.infer_agent_type(request) {
            if let Some(provider) = self.config.agent_rules.get(&inferred_agent) {
                return RouteDecision {
                    provider: provider.clone(),
                    reason: format!("inferred_agent:{}", inferred_agent),
                    fallbacks: self.get_fallbacks(provider),
                };
            }
        }

        // 4. Default provider
        RouteDecision {
            provider: self.config.default_provider.clone(),
            reason: "default".to_string(),
            fallbacks: self.get_fallbacks(&self.config.default_provider),
        }
    }

    /// Get fallback providers excluding the primary
    fn get_fallbacks(&self, primary: &str) -> Vec<String> {
        self.config
            .fallback_chain
            .iter()
            .filter(|p| *p != primary)
            .cloned()
            .collect()
    }

    /// Try to infer agent type from request content
    fn infer_agent_type(&self, request: &CompletionRequest) -> Option<String> {
        // Check system prompt for agent type indicators
        if let Some(system) = &request.system {
            let system_text = match system {
                serde_json::Value::String(s) => s.as_str(),
                serde_json::Value::Array(arr) => {
                    // Extract text from first content block
                    if let Some(first) = arr.first() {
                        if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                            return self.detect_agent_from_text(text);
                        }
                    }
                    return None;
                }
                _ => return None,
            };
            return self.detect_agent_from_text(system_text);
        }

        // Check first user message
        for msg in &request.messages {
            if msg.role == "user" {
                if let super::MessageContent::Text(text) = &msg.content {
                    if let Some(agent) = self.detect_agent_from_text(text) {
                        return Some(agent);
                    }
                }
            }
        }

        None
    }

    /// Detect agent type from text content
    fn detect_agent_from_text(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        // Check for explicit agent type mentions
        let agent_patterns: &[(&str, &[&str])] = &[
            ("chief-architect", &["chief architect", "system architect", "architecture design"]),
            ("code-reviewer", &["code review", "review the code", "reviewing code"]),
            ("test-engineer", &["test engineer", "write tests", "testing"]),
            ("security-auditor", &["security audit", "security review", "vulnerability"]),
            ("python-specialist", &["python", "fastapi", "django", "flask"]),
            ("rust-specialist", &["rust", "cargo", "tokio", "axum"]),
            ("go-specialist", &["golang", "go ", "goroutine"]),
            ("technical-researcher", &["research", "investigate", "analyze"]),
        ];

        for (agent, patterns) in agent_patterns {
            for pattern in *patterns {
                if text_lower.contains(*pattern) {
                    // Only return if this agent has a routing rule
                    if self.config.agent_rules.contains_key(*agent) {
                        return Some(agent.to_string());
                    }
                }
            }
        }

        None
    }
}

/// Extract model tier from model name
fn extract_model_tier(model: &str) -> String {
    let model_lower = model.to_lowercase();

    if model_lower.contains("opus") {
        "opus".to_string()
    } else if model_lower.contains("sonnet") {
        "sonnet".to_string()
    } else if model_lower.contains("haiku") {
        "haiku".to_string()
    } else if model_lower.contains("gpt-4") {
        "gpt4".to_string()
    } else if model_lower.contains("gpt-3.5") {
        "gpt35".to_string()
    } else if model_lower.contains("deepseek") {
        "deepseek".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_config() -> RoutingConfig {
        let mut agent_rules = HashMap::new();
        agent_rules.insert("chief-architect".to_string(), "anthropic".to_string());
        agent_rules.insert("code-reviewer".to_string(), "azure".to_string());
        agent_rules.insert("python-specialist".to_string(), "deepseek".to_string());

        let mut model_tier_rules = HashMap::new();
        model_tier_rules.insert("opus".to_string(), "anthropic".to_string());
        model_tier_rules.insert("sonnet".to_string(), "anthropic".to_string());
        model_tier_rules.insert("haiku".to_string(), "anthropic".to_string());

        RoutingConfig {
            default_provider: "anthropic".to_string(),
            agent_rules,
            model_tier_rules,
            fallback_chain: vec![
                "anthropic".to_string(),
                "azure".to_string(),
                "deepseek".to_string(),
            ],
        }
    }

    fn test_request(agent_type: Option<&str>, model: &str) -> CompletionRequest {
        CompletionRequest {
            model: model.to_string(),
            messages: vec![],
            max_tokens: 1000,
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: false,
            agent_type: agent_type.map(String::from),
            project_id: None,
        }
    }

    #[test]
    fn test_route_by_agent_type() {
        let engine = RoutingEngine::new(test_config());

        let request = test_request(Some("chief-architect"), "claude-sonnet-4-5");
        let decision = engine.route(&request);

        assert_eq!(decision.provider, "anthropic");
        assert!(decision.reason.contains("agent_rule"));
    }

    #[test]
    fn test_route_by_model_tier() {
        let engine = RoutingEngine::new(test_config());

        let request = test_request(None, "claude-opus-4-1");
        let decision = engine.route(&request);

        assert_eq!(decision.provider, "anthropic");
        assert!(decision.reason.contains("model_tier"));
    }

    #[test]
    fn test_route_default() {
        let engine = RoutingEngine::new(test_config());

        let request = test_request(None, "some-unknown-model");
        let decision = engine.route(&request);

        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.reason, "default");
    }

    #[test]
    fn test_route_code_reviewer_to_azure() {
        let engine = RoutingEngine::new(test_config());

        let request = test_request(Some("code-reviewer"), "claude-sonnet-4-5");
        let decision = engine.route(&request);

        assert_eq!(decision.provider, "azure");
    }

    #[test]
    fn test_fallbacks_exclude_primary() {
        let engine = RoutingEngine::new(test_config());

        let request = test_request(Some("chief-architect"), "claude-sonnet-4-5");
        let decision = engine.route(&request);

        assert!(!decision.fallbacks.contains(&"anthropic".to_string()));
        assert!(decision.fallbacks.contains(&"azure".to_string()));
    }

    #[test]
    fn test_extract_model_tier() {
        assert_eq!(extract_model_tier("claude-opus-4-1"), "opus");
        assert_eq!(extract_model_tier("claude-sonnet-4-5-20250929"), "sonnet");
        assert_eq!(extract_model_tier("claude-haiku-4-5"), "haiku");
        assert_eq!(extract_model_tier("gpt-4-turbo"), "gpt4");
        assert_eq!(extract_model_tier("random-model"), "unknown");
    }

    #[test]
    fn test_agent_type_case_insensitive() {
        let engine = RoutingEngine::new(test_config());

        let request = test_request(Some("CHIEF-ARCHITECT"), "claude-sonnet-4-5");
        let decision = engine.route(&request);

        assert_eq!(decision.provider, "anthropic");
    }
}
