//! Multi-model routing with cost calculation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider types
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Ollama,
    LocalAI,
    VLLM,
    TGI,
    #[serde(untagged)]
    Custom(String),
}

/// Model pricing information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model: String,
    pub provider: String,
    pub input_cost: f64,       // $ per 1M tokens
    pub output_cost: f64,      // $ per 1M tokens
    pub cache_write_cost: f64, // $ per 1M tokens (Claude cache only)
    pub cache_read_cost: f64,  // $ per 1M tokens (Claude cache only)
}

/// Route rule for model -> provider mapping
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteRule {
    pub pattern: String,
    pub provider: String,
    pub endpoint: String,
    pub priority: u32,
}

/// Routing configuration
#[derive(Clone)]
pub struct RouterConfig {
    pub routes: Vec<RouteRule>,
    pub pricing: HashMap<String, ModelPricing>,
}

impl Default for RouterConfig {
    /// Create default routing configuration
    fn default() -> Self {
        let mut pricing = HashMap::new();

        // Claude models - Opus 4
        pricing.insert(
            "claude-opus-4".to_string(),
            ModelPricing {
                model: "claude-opus-4".to_string(),
                provider: "anthropic".to_string(),
                input_cost: 15.0,
                output_cost: 75.0,
                cache_write_cost: 18.75,
                cache_read_cost: 1.5,
            },
        );

        // Claude Sonnet 4.5 (new naming convention)
        pricing.insert(
            "claude-sonnet-4".to_string(),
            ModelPricing {
                model: "claude-sonnet-4".to_string(),
                provider: "anthropic".to_string(),
                input_cost: 3.0,
                output_cost: 15.0,
                cache_write_cost: 3.75,
                cache_read_cost: 0.3,
            },
        );

        pricing.insert(
            "claude-sonnet-4-5-20251001".to_string(),
            ModelPricing {
                model: "claude-sonnet-4-5-20251001".to_string(),
                provider: "anthropic".to_string(),
                input_cost: 3.0,
                output_cost: 15.0,
                cache_write_cost: 3.75,
                cache_read_cost: 0.3,
            },
        );

        // Claude Sonnet 3.5 (legacy naming)
        pricing.insert(
            "claude-sonnet-3.5".to_string(),
            ModelPricing {
                model: "claude-sonnet-3.5".to_string(),
                provider: "anthropic".to_string(),
                input_cost: 3.0,
                output_cost: 15.0,
                cache_write_cost: 3.75,
                cache_read_cost: 0.3,
            },
        );

        // Claude Haiku 4.5 (new naming convention)
        pricing.insert(
            "claude-haiku-4".to_string(),
            ModelPricing {
                model: "claude-haiku-4".to_string(),
                provider: "anthropic".to_string(),
                input_cost: 1.0,
                output_cost: 5.0,
                cache_write_cost: 1.25,
                cache_read_cost: 0.1,
            },
        );

        pricing.insert(
            "claude-haiku-4-5-20251001".to_string(),
            ModelPricing {
                model: "claude-haiku-4-5-20251001".to_string(),
                provider: "anthropic".to_string(),
                input_cost: 1.0,
                output_cost: 5.0,
                cache_write_cost: 1.25,
                cache_read_cost: 0.1,
            },
        );

        // OpenAI models
        pricing.insert(
            "gpt-4".to_string(),
            ModelPricing {
                model: "gpt-4".to_string(),
                provider: "openai".to_string(),
                input_cost: 30.0,
                output_cost: 60.0,
                cache_write_cost: 0.0,
                cache_read_cost: 0.0,
            },
        );

        // Ollama models (self-hosted, free)
        pricing.insert(
            "ollama/llama3-70b".to_string(),
            ModelPricing {
                model: "ollama/llama3-70b".to_string(),
                provider: "ollama".to_string(),
                input_cost: 0.0,
                output_cost: 0.0,
                cache_write_cost: 0.0,
                cache_read_cost: 0.0,
            },
        );

        pricing.insert(
            "ollama/mistral".to_string(),
            ModelPricing {
                model: "ollama/mistral".to_string(),
                provider: "ollama".to_string(),
                input_cost: 0.0,
                output_cost: 0.0,
                cache_write_cost: 0.0,
                cache_read_cost: 0.0,
            },
        );

        let routes = vec![
            RouteRule {
                pattern: "^claude-".to_string(),
                provider: "anthropic".to_string(),
                endpoint: "https://api.anthropic.com/v1".to_string(),
                priority: 1,
            },
            RouteRule {
                pattern: "^gpt-".to_string(),
                provider: "openai".to_string(),
                endpoint: "https://api.openai.com/v1".to_string(),
                priority: 1,
            },
            RouteRule {
                pattern: "^ollama/".to_string(),
                provider: "ollama".to_string(),
                endpoint: "http://localhost:11434".to_string(),
                priority: 1,
            },
        ];

        Self { routes, pricing }
    }
}

/// Model router for finding routes and calculating costs
#[derive(Clone)]
pub struct ModelRouter {
    config: RouterConfig,
}

impl ModelRouter {
    /// Create a new router with default configuration
    pub fn new() -> Self {
        Self {
            config: RouterConfig::default(),
        }
    }

    /// Find route for a model using regex pattern matching
    pub fn find_route(&self, model: &str) -> Option<&RouteRule> {
        use regex::Regex;
        self.config.routes.iter().find(|route| {
            if let Ok(re) = Regex::new(&route.pattern) {
                re.is_match(model)
            } else {
                false
            }
        })
    }

    /// Get pricing for a model
    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.config.pricing.get(model)
    }

    /// Calculate cost for a request
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        self.config.pricing.get(model).map(|pricing| {
            let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_cost;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
            input_cost + output_cost
        })
    }

    /// Calculate cost with Claude prompt cache
    pub fn calculate_cost_with_claude_cache(
        &self,
        model: &str,
        cache_write_tokens: u32,
        cached_tokens: u32,
        new_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        self.config.pricing.get(model).map(|pricing| {
            let cache_write_cost =
                (cache_write_tokens as f64 / 1_000_000.0) * pricing.cache_write_cost;
            let cache_read_cost = (cached_tokens as f64 / 1_000_000.0) * pricing.cache_read_cost;
            let new_input_cost = (new_tokens as f64 / 1_000_000.0) * pricing.input_cost;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
            cache_write_cost + cache_read_cost + new_input_cost + output_cost
        })
    }

    /// Calculate cache savings for proxy cache hit
    pub fn calculate_cache_savings(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<(f64, f64, f64)> {
        // Returns (actual_cost, would_be_cost, savings)
        self.config.pricing.get(model).map(|pricing| {
            let would_be_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_cost
                + (output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
            (0.0, would_be_cost, would_be_cost) // Proxy cache = 0 cost
        })
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_claude_opus_model() {
        let router = ModelRouter::new();
        let route = router.find_route("claude-opus-4");
        assert!(route.is_some());
        assert_eq!(route.unwrap().provider, "anthropic");
    }

    #[test]
    fn test_route_openai_gpt4_model() {
        let router = ModelRouter::new();
        let route = router.find_route("gpt-4");
        assert!(route.is_some());
        assert_eq!(route.unwrap().provider, "openai");
    }

    #[test]
    fn test_route_ollama_model() {
        let router = ModelRouter::new();
        let route = router.find_route("ollama/llama3-70b");
        assert!(route.is_some());
        assert_eq!(route.unwrap().endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_route_unknown_model() {
        let router = ModelRouter::new();
        let route = router.find_route("unknown-model-xyz");
        assert!(route.is_none());
    }

    #[test]
    fn test_cost_calculation_claude_opus() {
        let router = ModelRouter::new();
        let cost = router.calculate_cost("claude-opus-4", 1_000_000, 500_000);
        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // 1M input * $15/M + 500K output * $75/M = $15 + $37.50 = $52.50
        assert!((cost_value - 52.5).abs() < 0.01);
    }

    #[test]
    fn test_cost_calculation_ollama_free() {
        let router = ModelRouter::new();
        let cost = router.calculate_cost("ollama/llama3-70b", 1_000_000, 500_000);
        assert!(cost.is_some());
        assert_eq!(cost.unwrap(), 0.0);
    }

    #[test]
    fn test_proxy_cache_savings_claude_opus() {
        let router = ModelRouter::new();
        let savings = router.calculate_cache_savings("claude-opus-4", 1_000_000, 500_000);
        assert!(savings.is_some());
        let (actual_cost, would_be_cost, savings_amount) = savings.unwrap();
        assert_eq!(actual_cost, 0.0);
        assert!((would_be_cost - 52.5).abs() < 0.01);
        assert!((savings_amount - 52.5).abs() < 0.01);
    }

    #[test]
    fn test_claude_cache_savings_with_90_percent_cached() {
        let router = ModelRouter::new();
        let total_input = 1_000_000u32;
        let cache_write_tokens = 0u32; // No cache write in this scenario
        let cached_tokens = 900_000u32;
        let new_tokens = 100_000u32;
        let output_tokens = 500_000u32;

        let actual_cost = router
            .calculate_cost_with_claude_cache(
                "claude-opus-4",
                cache_write_tokens,
                cached_tokens,
                new_tokens,
                output_tokens,
            )
            .unwrap();

        let would_be_cost = router
            .calculate_cost("claude-opus-4", total_input, output_tokens)
            .unwrap();

        let savings = would_be_cost - actual_cost;

        // Cache write: 0K * $18.75/M = $0.00
        // Cache read: 900K * $1.5/M = $1.35
        // New input: 100K * $15/M = $1.50
        // Output: 500K * $75/M = $37.50
        // Actual: $40.35
        // Would-be: $52.50
        // Savings: $12.15
        assert!((actual_cost - 40.35).abs() < 0.01);
        assert!((savings - 12.15).abs() < 0.01);
    }
}
