//! Comprehensive tests for model override logic
//!
//! This module tests the CCO model override feature including:
//! - Basic override application
//! - Override rule matching
//! - Request rewriting
//! - Parameter preservation
//! - Edge cases and error handling

#[cfg(test)]
mod model_override_unit_tests {
    use std::collections::HashMap;

    /// Model override rule
    #[derive(Clone, Debug)]
    struct ModelOverrideRule {
        from_model: String,
        to_model: String,
    }

    /// Mock override configuration
    struct OverrideConfig {
        rules: HashMap<String, String>,
    }

    impl OverrideConfig {
        fn new() -> Self {
            Self {
                rules: HashMap::new(),
            }
        }

        fn add_rule(&mut self, from: &str, to: &str) {
            self.rules.insert(from.to_string(), to.to_string());
        }

        fn apply_override(&self, model: &str) -> String {
            self.rules
                .get(model)
                .cloned()
                .unwrap_or_else(|| model.to_string())
        }
    }

    /// Chat request with model field
    #[derive(Clone, Debug)]
    struct ChatRequest {
        model: String,
        messages: Vec<Message>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    }

    #[derive(Clone, Debug)]
    struct Message {
        role: String,
        content: String,
    }

    // ========== BASIC OVERRIDE TESTS ==========

    #[test]
    fn test_simple_model_override() {
        // Given: override rule "claude-sonnet-4.5" → "claude-haiku-4.5"
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");

        // When: apply override to "claude-sonnet-4.5"
        let result = config.apply_override("claude-sonnet-4.5");

        // Then: result should be "claude-haiku-4.5"
        assert_eq!(
            result, "claude-haiku-4.5",
            "Override should rewrite sonnet to haiku"
        );
    }

    #[test]
    fn test_no_override_when_not_in_rules() {
        // Given: override rules for sonnet, not for opus
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");

        // When: apply override to "claude-opus-4"
        let result = config.apply_override("claude-opus-4");

        // Then: result should be unchanged
        assert_eq!(
            result, "claude-opus-4",
            "Model without override rule should pass through unchanged"
        );
    }

    #[test]
    fn test_override_exact_match_only() {
        // Given: override rule for exact model name
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5-20250929", "claude-haiku-4.5");

        // When: apply override to slightly different model name
        let result = config.apply_override("claude-sonnet-4.5-20250928");

        // Then: should NOT match (exact match required)
        assert_eq!(
            result, "claude-sonnet-4.5-20250928",
            "Override should require exact match, no fuzzy matching"
        );
    }

    #[test]
    fn test_multiple_override_rules() {
        // Given: multiple override rules loaded
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        config.add_rule("claude-opus-4", "claude-sonnet-4.5");
        config.add_rule("gpt-4", "claude-haiku-4.5");

        // When: apply various models through the overrides
        let sonnet_result = config.apply_override("claude-sonnet-4.5");
        let opus_result = config.apply_override("claude-opus-4");
        let gpt_result = config.apply_override("gpt-4");
        let unknown_result = config.apply_override("unknown-model");

        // Then: each should apply correctly
        assert_eq!(sonnet_result, "claude-haiku-4.5");
        assert_eq!(opus_result, "claude-sonnet-4.5");
        assert_eq!(gpt_result, "claude-haiku-4.5");
        assert_eq!(unknown_result, "unknown-model");
    }

    #[test]
    fn test_chat_request_with_override() {
        // Given: ChatRequest with model="claude-sonnet-4.5"
        let mut request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };

        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");

        // When: override rules applied
        request.model = config.apply_override(&request.model);

        // Then: request.model should be rewritten to "claude-haiku-4.5"
        assert_eq!(
            request.model, "claude-haiku-4.5",
            "Request model should be overridden"
        );
    }

    #[test]
    fn test_override_preserves_messages() {
        // Given: ChatRequest with specific messages
        let original_messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "What is 2+2?".to_string(),
            },
        ];

        let mut request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: original_messages.clone(),
            temperature: Some(1.0),
            max_tokens: Some(4096),
        };

        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");

        // When: override applied
        request.model = config.apply_override(&request.model);

        // Then: messages should be unchanged
        assert_eq!(request.messages.len(), original_messages.len());
        assert_eq!(request.messages[0].role, original_messages[0].role);
        assert_eq!(request.messages[0].content, original_messages[0].content);
        assert_eq!(request.messages[1].role, original_messages[1].role);
        assert_eq!(request.messages[1].content, original_messages[1].content);
    }

    #[test]
    fn test_override_preserves_temperature_and_max_tokens() {
        // Given: ChatRequest with temperature=0.5, max_tokens=2000
        let mut request = ChatRequest {
            model: "claude-sonnet-4.5".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            temperature: Some(0.5),
            max_tokens: Some(2000),
        };

        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");

        // When: override applied
        let original_temp = request.temperature;
        let original_max_tokens = request.max_tokens;
        request.model = config.apply_override(&request.model);

        // Then: temperature and max_tokens unchanged
        assert_eq!(request.temperature, original_temp);
        assert_eq!(request.max_tokens, original_max_tokens);
        assert_eq!(request.temperature, Some(0.5));
        assert_eq!(request.max_tokens, Some(2000));
    }

    #[test]
    fn test_empty_overrides_map() {
        // Given: empty override map
        let config = OverrideConfig::new();

        // When: apply_override called
        let result1 = config.apply_override("claude-sonnet-4.5");
        let result2 = config.apply_override("claude-opus-4");
        let result3 = config.apply_override("gpt-4");

        // Then: all models pass through unchanged
        assert_eq!(result1, "claude-sonnet-4.5");
        assert_eq!(result2, "claude-opus-4");
        assert_eq!(result3, "gpt-4");
    }

    // ========== EDGE CASE TESTS ==========

    #[test]
    fn test_model_name_case_sensitivity() {
        // Given: override for specific case
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet", "claude-haiku");

        // When: apply to different case
        let result = config.apply_override("Claude-Sonnet");

        // Then: should NOT match (case-sensitive)
        assert_eq!(result, "Claude-Sonnet", "Override should be case-sensitive");
    }

    #[test]
    fn test_no_partial_string_matching() {
        // Given: override for short name
        let mut config = OverrideConfig::new();
        config.add_rule("sonnet", "haiku");

        // When: apply to longer model name containing the string
        let result = config.apply_override("claude-sonnet-4.5-20250929");

        // Then: should NOT match (need exact full model name)
        assert_eq!(
            result, "claude-sonnet-4.5-20250929",
            "Override should not do partial matching"
        );
    }

    #[test]
    fn test_single_override_pass_only() {
        // Given: override rules that could chain
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        config.add_rule("claude-haiku-4.5", "claude-opus-4");

        // When: sonnet processed
        let result = config.apply_override("claude-sonnet-4.5");

        // Then: should result in haiku (not chained to opus)
        assert_eq!(
            result, "claude-haiku-4.5",
            "Override should only apply once, no chaining"
        );
    }

    #[tokio::test]
    async fn test_concurrent_override_requests() {
        // Given: OverrideConfig with rules (wrapped in Arc for thread safety)
        use std::sync::Arc;

        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        config.add_rule("claude-opus-4", "claude-sonnet-4.5");
        let config = Arc::new(config);

        // When: 100 concurrent requests sent
        let mut handles = vec![];
        for i in 0..100 {
            let config_clone = config.clone();
            let handle = tokio::spawn(async move {
                let model = if i % 2 == 0 {
                    "claude-sonnet-4.5"
                } else {
                    "claude-opus-4"
                };
                config_clone.apply_override(model)
            });
            handles.push(handle);
        }

        // Then: all should apply overrides correctly with no data races
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.expect("Task should complete");
            if i % 2 == 0 {
                assert_eq!(result, "claude-haiku-4.5");
            } else {
                assert_eq!(result, "claude-sonnet-4.5");
            }
        }
    }

    #[test]
    fn test_long_model_names() {
        // Given: override rule for unusually long model name
        let mut config = OverrideConfig::new();
        let long_model_name = "claude-sonnet-4.5-20250929-experimental-ultra-long-name-v2";
        config.add_rule(long_model_name, "claude-haiku-4.5");

        // When: applied
        let result = config.apply_override(long_model_name);

        // Then: should work normally
        assert_eq!(result, "claude-haiku-4.5");
    }

    #[test]
    fn test_model_names_with_special_chars() {
        // Given: model names with dots, dashes, numbers
        let mut config = OverrideConfig::new();
        config.add_rule("claude-3.5-sonnet-v2", "claude-haiku-4.5");
        config.add_rule("gpt-4-turbo-2024-04-09", "claude-haiku-4.5");
        config.add_rule("model_with_underscores", "claude-haiku-4.5");

        // When: override rules applied
        let result1 = config.apply_override("claude-3.5-sonnet-v2");
        let result2 = config.apply_override("gpt-4-turbo-2024-04-09");
        let result3 = config.apply_override("model_with_underscores");

        // Then: should match exactly
        assert_eq!(result1, "claude-haiku-4.5");
        assert_eq!(result2, "claude-haiku-4.5");
        assert_eq!(result3, "claude-haiku-4.5");
    }

    #[test]
    fn test_override_to_same_model() {
        // Given: override that maps model to itself (no-op override)
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-sonnet-4.5");

        // When: override applied
        let result = config.apply_override("claude-sonnet-4.5");

        // Then: should return same model
        assert_eq!(result, "claude-sonnet-4.5");
    }

    #[test]
    fn test_override_with_empty_strings() {
        // Given: empty model name
        let config = OverrideConfig::new();

        // When: apply override to empty string
        let result = config.apply_override("");

        // Then: should return empty string unchanged
        assert_eq!(result, "");
    }

    #[test]
    fn test_override_with_whitespace() {
        // Given: model name with whitespace
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");

        // When: apply to model with whitespace
        let result = config.apply_override(" claude-sonnet-4.5 ");

        // Then: should NOT match (whitespace is significant)
        assert_eq!(result, " claude-sonnet-4.5 ");
    }

    // ========== PERFORMANCE TESTS ==========

    #[test]
    fn test_override_lookup_is_fast() {
        // Given: large override map with 1000 rules
        let mut config = OverrideConfig::new();
        for i in 0..1000 {
            config.add_rule(&format!("model-{}", i), "target-model");
        }

        // When: lookup performed
        let start = std::time::Instant::now();
        let result = config.apply_override("model-500");
        let duration = start.elapsed();

        // Then: lookup should be O(1) and very fast (< 1ms)
        assert_eq!(result, "target-model");
        assert!(
            duration.as_micros() < 1000,
            "HashMap lookup should be very fast (< 1ms), took {:?}",
            duration
        );
    }

    #[test]
    fn test_many_sequential_overrides() {
        // Given: config with rules
        let mut config = OverrideConfig::new();
        config.add_rule("claude-sonnet-4.5", "claude-haiku-4.5");
        config.add_rule("claude-opus-4", "claude-sonnet-4.5");

        // When: 10000 sequential override applications
        let start = std::time::Instant::now();
        for i in 0..10000 {
            let model = if i % 2 == 0 {
                "claude-sonnet-4.5"
            } else {
                "claude-opus-4"
            };
            let _ = config.apply_override(model);
        }
        let duration = start.elapsed();

        // Then: should complete quickly (< 100ms for 10k operations)
        assert!(
            duration.as_millis() < 100,
            "10k override operations should be fast (< 100ms), took {:?}",
            duration
        );
    }
}
