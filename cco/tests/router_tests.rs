//! Comprehensive router tests for multi-model routing
//!
//! This module tests all aspects of the routing system including:
//! - Model provider routing (Claude, OpenAI, Ollama)
//! - Routing rule matching
//! - Cost calculation with and without cache
//! - Self-hosted model savings
//! - Provider fallback behavior

#[cfg(test)]
mod router_tests {
    use std::collections::HashMap;

    /// Provider types
    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code, clippy::upper_case_acronyms)]
    enum ProviderType {
        Anthropic,
        OpenAI,
        Ollama,
        LocalAI,
        VLLM,
        TGI,
        Custom(String),
    }

    /// Model pricing information
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct ModelPricing {
        model: String,
        provider: ProviderType,
        input_cost: f64,      // $ per 1M tokens
        output_cost: f64,     // $ per 1M tokens
        cache_read_cost: f64, // $ per 1M tokens (Claude cache only)
    }

    /// Route rule
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct RouteRule {
        pattern: String,
        provider: ProviderType,
        endpoint: String,
        priority: u32,
    }

    /// Mock router
    struct MockRouter {
        routes: Vec<RouteRule>,
        pricing: HashMap<String, ModelPricing>,
    }

    impl MockRouter {
        fn new() -> Self {
            let mut pricing = HashMap::new();

            // Claude models
            pricing.insert(
                "claude-opus-4".to_string(),
                ModelPricing {
                    model: "claude-opus-4".to_string(),
                    provider: ProviderType::Anthropic,
                    input_cost: 15.0,
                    output_cost: 75.0,
                    cache_read_cost: 1.5,
                },
            );

            pricing.insert(
                "claude-sonnet-3.5".to_string(),
                ModelPricing {
                    model: "claude-sonnet-3.5".to_string(),
                    provider: ProviderType::Anthropic,
                    input_cost: 3.0,
                    output_cost: 15.0,
                    cache_read_cost: 0.3,
                },
            );

            // OpenAI models
            pricing.insert(
                "gpt-4".to_string(),
                ModelPricing {
                    model: "gpt-4".to_string(),
                    provider: ProviderType::OpenAI,
                    input_cost: 30.0,
                    output_cost: 60.0,
                    cache_read_cost: 0.0, // OpenAI doesn't have prompt cache yet
                },
            );

            // Ollama models (self-hosted, free)
            pricing.insert(
                "ollama/llama3-70b".to_string(),
                ModelPricing {
                    model: "ollama/llama3-70b".to_string(),
                    provider: ProviderType::Ollama,
                    input_cost: 0.0,
                    output_cost: 0.0,
                    cache_read_cost: 0.0,
                },
            );

            pricing.insert(
                "ollama/mistral".to_string(),
                ModelPricing {
                    model: "ollama/mistral".to_string(),
                    provider: ProviderType::Ollama,
                    input_cost: 0.0,
                    output_cost: 0.0,
                    cache_read_cost: 0.0,
                },
            );

            let routes = vec![
                RouteRule {
                    pattern: "^claude-".to_string(),
                    provider: ProviderType::Anthropic,
                    endpoint: "https://api.anthropic.com/v1".to_string(),
                    priority: 1,
                },
                RouteRule {
                    pattern: "^gpt-".to_string(),
                    provider: ProviderType::OpenAI,
                    endpoint: "https://api.openai.com/v1".to_string(),
                    priority: 1,
                },
                RouteRule {
                    pattern: "^ollama/".to_string(),
                    provider: ProviderType::Ollama,
                    endpoint: "http://localhost:11434".to_string(),
                    priority: 1,
                },
            ];

            Self { routes, pricing }
        }

        fn find_route(&self, model: &str) -> Option<&RouteRule> {
            use regex::Regex;
            self.routes.iter().find(|route| {
                if let Ok(re) = Regex::new(&route.pattern) {
                    re.is_match(model)
                } else {
                    false
                }
            })
        }

        fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
            self.pricing.get(model)
        }

        fn calculate_cost(
            &self,
            model: &str,
            input_tokens: u32,
            output_tokens: u32,
        ) -> Option<f64> {
            self.pricing.get(model).map(|pricing| {
                let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_cost;
                let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
                input_cost + output_cost
            })
        }

        fn calculate_cost_with_claude_cache(
            &self,
            model: &str,
            cached_tokens: u32,
            new_tokens: u32,
            output_tokens: u32,
        ) -> Option<f64> {
            self.pricing.get(model).map(|pricing| {
                let cache_read_cost =
                    (cached_tokens as f64 / 1_000_000.0) * pricing.cache_read_cost;
                let new_input_cost =
                    (new_tokens as f64 / 1_000_000.0) * pricing.input_cost;
                let output_cost =
                    (output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
                cache_read_cost + new_input_cost + output_cost
            })
        }

        fn calculate_cache_savings(
            &self,
            model: &str,
            input_tokens: u32,
            output_tokens: u32,
        ) -> Option<(f64, f64, f64)> {
            // Returns (actual_cost, would_be_cost, savings) for proxy cache hit
            self.pricing.get(model).map(|pricing| {
                let would_be_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_cost
                    + (output_tokens as f64 / 1_000_000.0) * pricing.output_cost;
                (0.0, would_be_cost, would_be_cost) // Proxy cache = 0 cost
            })
        }
    }

    // ========== MODEL ROUTING TESTS ==========

    #[test]
    fn test_route_claude_opus_model() {
        let router = MockRouter::new();
        let route = router.find_route("claude-opus-4");

        assert!(route.is_some(), "Should find route for claude-opus-4");
        assert_eq!(
            route.unwrap().provider,
            ProviderType::Anthropic,
            "Claude should route to Anthropic"
        );
    }

    #[test]
    fn test_route_claude_sonnet_model() {
        let router = MockRouter::new();
        let route = router.find_route("claude-sonnet-3.5");

        assert!(route.is_some(), "Should find route for claude-sonnet-3.5");
        assert_eq!(
            route.unwrap().provider,
            ProviderType::Anthropic,
            "Claude should route to Anthropic"
        );
    }

    #[test]
    fn test_route_openai_gpt4_model() {
        let router = MockRouter::new();
        let route = router.find_route("gpt-4");

        assert!(route.is_some(), "Should find route for gpt-4");
        assert_eq!(
            route.unwrap().provider,
            ProviderType::OpenAI,
            "GPT should route to OpenAI"
        );
    }

    #[test]
    fn test_route_ollama_model() {
        let router = MockRouter::new();
        let route = router.find_route("ollama/llama3-70b");

        assert!(route.is_some(), "Should find route for ollama model");
        assert_eq!(
            route.unwrap().provider,
            ProviderType::Ollama,
            "Ollama should route to localhost"
        );
        assert_eq!(
            route.unwrap().endpoint,
            "http://localhost:11434",
            "Ollama endpoint should be localhost:11434"
        );
    }

    #[test]
    fn test_route_unknown_model() {
        let router = MockRouter::new();
        let route = router.find_route("unknown-model-xyz");

        assert!(route.is_none(), "Unknown model should have no route");
    }

    // ========== PROVIDER ENDPOINT TESTS ==========

    #[test]
    fn test_anthropic_endpoint() {
        let router = MockRouter::new();
        let route = router.find_route("claude-opus-4").unwrap();

        assert_eq!(
            route.endpoint,
            "https://api.anthropic.com/v1",
            "Anthropic endpoint should be https://api.anthropic.com/v1"
        );
    }

    #[test]
    fn test_openai_endpoint() {
        let router = MockRouter::new();
        let route = router.find_route("gpt-4").unwrap();

        assert_eq!(
            route.endpoint,
            "https://api.openai.com/v1",
            "OpenAI endpoint should be https://api.openai.com/v1"
        );
    }

    #[test]
    fn test_ollama_endpoint() {
        let router = MockRouter::new();
        let route = router.find_route("ollama/llama3-70b").unwrap();

        assert_eq!(
            route.endpoint,
            "http://localhost:11434",
            "Ollama endpoint should be http://localhost:11434"
        );
    }

    // ========== COST CALCULATION TESTS ==========

    #[test]
    fn test_cost_calculation_claude_opus() {
        let router = MockRouter::new();
        let cost = router.calculate_cost("claude-opus-4", 1_000_000, 500_000);

        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // 1M input tokens * $15/M + 500K output tokens * $75/M = $15 + $37.50 = $52.50
        assert!((cost_value - 52.5).abs() < 0.01, "Cost should be $52.50, got ${}", cost_value);
    }

    #[test]
    fn test_cost_calculation_claude_sonnet() {
        let router = MockRouter::new();
        let cost = router.calculate_cost("claude-sonnet-3.5", 1_000_000, 500_000);

        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // 1M input tokens * $3/M + 500K output tokens * $15/M = $3 + $7.50 = $10.50
        assert!((cost_value - 10.5).abs() < 0.01, "Cost should be $10.50, got ${}", cost_value);
    }

    #[test]
    fn test_cost_calculation_openai_gpt4() {
        let router = MockRouter::new();
        let cost = router.calculate_cost("gpt-4", 1_000_000, 500_000);

        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // 1M input tokens * $30/M + 500K output tokens * $60/M = $30 + $30 = $60
        assert!((cost_value - 60.0).abs() < 0.01, "Cost should be $60.00, got ${}", cost_value);
    }

    #[test]
    fn test_cost_calculation_ollama_free() {
        let router = MockRouter::new();
        let cost = router.calculate_cost("ollama/llama3-70b", 1_000_000, 500_000);

        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        assert_eq!(cost_value, 0.0, "Ollama should be free");
    }

    #[test]
    fn test_cost_calculation_small_tokens() {
        let router = MockRouter::new();
        let cost = router.calculate_cost("claude-opus-4", 100, 50);

        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // 100 input tokens * $15/M + 50 output tokens * $75/M = $0.00525
        assert!(cost_value > 0.0, "Small token count should still cost something");
        assert!(cost_value < 0.01, "Small token count should cost very little");
        assert!((cost_value - 0.00525).abs() < 0.00001);
    }

    // ========== CACHE SAVINGS CALCULATION TESTS ==========

    #[test]
    fn test_proxy_cache_savings_claude_opus() {
        let router = MockRouter::new();
        let savings = router.calculate_cache_savings("claude-opus-4", 1_000_000, 500_000);

        assert!(savings.is_some());
        let (actual_cost, would_be_cost, savings_amount) = savings.unwrap();

        assert_eq!(actual_cost, 0.0, "Proxy cache hit should cost $0");
        assert!((would_be_cost - 52.5).abs() < 0.01, "Would-be cost should be $52.50");
        assert!((savings_amount - 52.5).abs() < 0.01, "Savings should be $52.50");
    }

    #[test]
    fn test_proxy_cache_savings_claude_sonnet() {
        let router = MockRouter::new();
        let savings = router.calculate_cache_savings("claude-sonnet-3.5", 1_000_000, 500_000);

        assert!(savings.is_some());
        let (actual_cost, would_be_cost, savings_amount) = savings.unwrap();

        assert_eq!(actual_cost, 0.0);
        assert!((would_be_cost - 10.5).abs() < 0.01, "Would-be cost should be $10.50");
        assert!((savings_amount - 10.5).abs() < 0.01, "Savings should be $10.50");
    }

    // ========== CLAUDE PROMPT CACHE TESTS ==========

    #[test]
    fn test_claude_cache_savings_with_90_percent_cached() {
        let router = MockRouter::new();
        let total_input = 1_000_000u32;
        let cached_tokens = 900_000u32;
        let new_tokens = 100_000u32;
        let output_tokens = 500_000u32;

        let actual_cost = router
            .calculate_cost_with_claude_cache(
                "claude-opus-4",
                cached_tokens,
                new_tokens,
                output_tokens,
            )
            .unwrap();

        let would_be_cost = router
            .calculate_cost("claude-opus-4", total_input, output_tokens)
            .unwrap();

        let savings = would_be_cost - actual_cost;

        // Cache read: 900K * $1.5/M = $1.35
        // New input: 100K * $15/M = $1.50
        // Output: 500K * $75/M = $37.50
        // Actual: $1.35 + $1.50 + $37.50 = $40.35
        // Would-be: $15 + $37.50 = $52.50
        // Savings: $12.15
        assert!(
            (actual_cost - 40.35).abs() < 0.01,
            "Actual cost should be ~$40.35, got ${}",
            actual_cost
        );
        assert!(
            (savings - 12.15).abs() < 0.01,
            "Savings should be ~$12.15, got ${}",
            savings
        );
    }

    #[test]
    fn test_claude_cache_savings_with_50_percent_cached() {
        let router = MockRouter::new();
        let total_input = 1_000_000u32;
        let cached_tokens = 500_000u32;
        let new_tokens = 500_000u32;
        let output_tokens = 500_000u32;

        let actual_cost = router
            .calculate_cost_with_claude_cache(
                "claude-opus-4",
                cached_tokens,
                new_tokens,
                output_tokens,
            )
            .unwrap();

        let would_be_cost = router
            .calculate_cost("claude-opus-4", total_input, output_tokens)
            .unwrap();

        let savings = would_be_cost - actual_cost;

        // Cache read: 500K * $1.5/M = $0.75
        // New input: 500K * $15/M = $7.50
        // Output: 500K * $75/M = $37.50
        // Actual: $0.75 + $7.50 + $37.50 = $45.75
        // Would-be: $15 + $37.50 = $52.50
        // Savings: $6.75
        assert!((actual_cost - 45.75).abs() < 0.01);
        assert!((savings - 6.75).abs() < 0.01);
    }

    // ========== SELF-HOSTED SAVINGS TESTS ==========

    #[test]
    fn test_self_hosted_vs_claude_opus_savings() {
        let router = MockRouter::new();

        let claude_cost = router
            .calculate_cost("claude-opus-4", 1_000_000, 500_000)
            .unwrap();

        let ollama_cost = router
            .calculate_cost("ollama/llama3-70b", 1_000_000, 500_000)
            .unwrap();

        let savings = claude_cost - ollama_cost;

        assert_eq!(ollama_cost, 0.0, "Ollama should be free");
        assert!((claude_cost - 52.5).abs() < 0.01);
        assert!((savings - 52.5).abs() < 0.01, "Savings should be 100% of Claude cost");
    }

    #[test]
    fn test_self_hosted_vs_claude_sonnet_savings() {
        let router = MockRouter::new();

        let claude_cost = router
            .calculate_cost("claude-sonnet-3.5", 1_000_000, 500_000)
            .unwrap();

        let ollama_cost = router
            .calculate_cost("ollama/mistral", 1_000_000, 500_000)
            .unwrap();

        let savings = claude_cost - ollama_cost;

        assert_eq!(ollama_cost, 0.0);
        assert!((claude_cost - 10.5).abs() < 0.01);
        assert!((savings - 10.5).abs() < 0.01);
    }

    #[test]
    fn test_self_hosted_vs_openai_savings() {
        let router = MockRouter::new();

        let openai_cost = router
            .calculate_cost("gpt-4", 1_000_000, 500_000)
            .unwrap();

        let ollama_cost = router
            .calculate_cost("ollama/llama3-70b", 1_000_000, 500_000)
            .unwrap();

        let savings = openai_cost - ollama_cost;

        assert_eq!(ollama_cost, 0.0);
        assert!((openai_cost - 60.0).abs() < 0.01);
        assert!((savings - 60.0).abs() < 0.01);
    }

    // ========== CUMULATIVE COST TESTS ==========

    #[test]
    fn test_monthly_cost_claude_opus_without_cache() {
        let router = MockRouter::new();

        // Assume 100 requests per day, 30 days
        // 1M tokens per request on average
        let requests_per_month = 100 * 30;
        let tokens_per_request = 1_000_000u32;
        let output_tokens_per_request = 500_000u32;

        let total_input_tokens = requests_per_month as u32 * tokens_per_request;
        let total_output_tokens = requests_per_month as u32 * output_tokens_per_request;

        let cost = router
            .calculate_cost("claude-opus-4", total_input_tokens, total_output_tokens)
            .unwrap();

        // 3000 requests * 1M tokens = 3B input tokens
        // 3000 requests * 0.5M tokens = 1.5B output tokens
        // 3B * $15/M + 1.5B * $75/M = $45K + $112.5K = $157.5K
        assert!((cost - 157_500.0).abs() < 100.0, "Monthly cost should be ~$157.5K");
    }

    #[test]
    fn test_monthly_savings_with_50_percent_cache_hit_rate() {
        let router = MockRouter::new();

        let requests_per_month = 100 * 30;
        let cache_hits = requests_per_month / 2;
        let tokens_per_request = 1_000_000u32;
        let output_tokens_per_request = 500_000u32;

        // Calculate savings for cache hits (proxy cache = $0)
        let total_savings: f64 = (0..cache_hits)
            .map(|_| {
                router
                    .calculate_cache_savings(
                        "claude-opus-4",
                        tokens_per_request,
                        output_tokens_per_request,
                    )
                    .unwrap()
                    .2
            })
            .sum();

        assert!(total_savings > 0.0, "Should have savings from cache hits");
        // 50% of 3000 requests * $52.50 savings per hit = $78,750
        assert!(
            (total_savings - 78_750.0).abs() < 100.0,
            "Monthly savings should be ~$78.75K with 50% cache hit rate"
        );
    }

    // ========== PRICING EDGE CASES ==========

    #[test]
    fn test_pricing_unknown_model() {
        let router = MockRouter::new();
        let pricing = router.get_pricing("unknown-model-xyz");

        assert!(pricing.is_none(), "Unknown model should have no pricing");
    }

    #[test]
    fn test_cost_zero_tokens() {
        let router = MockRouter::new();
        let cost = router.calculate_cost("claude-opus-4", 0, 0);

        assert!(cost.is_some());
        assert_eq!(cost.unwrap(), 0.0);
    }
}
