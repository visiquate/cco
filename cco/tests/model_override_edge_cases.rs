//! Edge case and boundary condition tests for model overrides
//!
//! This module tests unusual scenarios and error conditions:
//! - Boundary values
//! - Invalid inputs
//! - Race conditions
//! - Memory/performance limits
//! - Error recovery

#[cfg(test)]
mod model_override_edge_cases {
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Override configuration
    #[derive(Clone)]
    struct OverrideConfig {
        rules: HashMap<String, String>,
        max_rules: usize,
    }

    impl OverrideConfig {
        fn new() -> Self {
            Self {
                rules: HashMap::new(),
                max_rules: 10000, // Reasonable limit
            }
        }

        fn add_rule(&mut self, from: &str, to: &str) -> Result<(), String> {
            if self.rules.len() >= self.max_rules {
                return Err(format!(
                    "Maximum number of override rules ({}) exceeded",
                    self.max_rules
                ));
            }
            self.rules.insert(from.to_string(), to.to_string());
            Ok(())
        }

        fn apply_override(&self, model: &str) -> String {
            self.rules
                .get(model)
                .cloned()
                .unwrap_or_else(|| model.to_string())
        }
    }

    // ========== BOUNDARY VALUE TESTS ==========

    #[test]
    fn test_empty_model_name() {
        // Given: override config
        let config = OverrideConfig::new();

        // When: apply to empty string
        let result = config.apply_override("");

        // Then: should return empty string
        assert_eq!(result, "");
    }

    #[test]
    fn test_single_character_model_name() {
        // Given: single char model name
        let mut config = OverrideConfig::new();
        config.add_rule("a", "b").unwrap();

        // When: apply override
        let result = config.apply_override("a");

        // Then: should work
        assert_eq!(result, "b");
    }

    #[test]
    fn test_very_long_model_name() {
        // Given: extremely long model name (10KB)
        let long_name = "a".repeat(10_000);
        let mut config = OverrideConfig::new();
        config.add_rule(&long_name, "target").unwrap();

        // When: apply override
        let result = config.apply_override(&long_name);

        // Then: should work correctly
        assert_eq!(result, "target");
    }

    #[test]
    fn test_unicode_in_model_names() {
        // Given: model names with unicode characters
        let mut config = OverrideConfig::new();
        config.add_rule("模型-中文", "model-english").unwrap();
        config.add_rule("модель-русский", "model-russian").unwrap();
        config.add_rule("モデル-日本語", "model-japanese").unwrap();
        config.add_rule("مودیل-عربی", "model-arabic").unwrap();

        // When: apply overrides
        let result1 = config.apply_override("模型-中文");
        let result2 = config.apply_override("модель-русский");
        let result3 = config.apply_override("モデル-日本語");
        let result4 = config.apply_override("مودیل-عربی");

        // Then: should work with unicode
        assert_eq!(result1, "model-english");
        assert_eq!(result2, "model-russian");
        assert_eq!(result3, "model-japanese");
        assert_eq!(result4, "model-arabic");
    }

    #[test]
    fn test_special_characters_in_model_names() {
        // Given: model names with special characters
        let mut config = OverrideConfig::new();
        config.add_rule("model@v1.0", "model@v2.0").unwrap();
        config.add_rule("model#tag", "model#newtag").unwrap();
        config.add_rule("model$price", "model$free").unwrap();
        config.add_rule("model%percent", "model%new").unwrap();
        config.add_rule("model&and", "model&or").unwrap();
        config.add_rule("model*star", "model*nova").unwrap();
        config.add_rule("model(paren)", "model[bracket]").unwrap();
        config.add_rule("model+plus", "model-minus").unwrap();
        config.add_rule("model=equals", "model!=notequals").unwrap();

        // When: apply overrides
        assert_eq!(config.apply_override("model@v1.0"), "model@v2.0");
        assert_eq!(config.apply_override("model#tag"), "model#newtag");
        assert_eq!(config.apply_override("model$price"), "model$free");
        assert_eq!(config.apply_override("model%percent"), "model%new");
        assert_eq!(config.apply_override("model&and"), "model&or");
        assert_eq!(config.apply_override("model*star"), "model*nova");
        assert_eq!(config.apply_override("model(paren)"), "model[bracket]");
        assert_eq!(config.apply_override("model+plus"), "model-minus");
        assert_eq!(config.apply_override("model=equals"), "model!=notequals");
    }

    #[test]
    fn test_whitespace_variations() {
        // Given: model names with various whitespace
        let mut config = OverrideConfig::new();
        config.add_rule("model with spaces", "target").unwrap();
        config.add_rule("model\twith\ttabs", "target-tabs").unwrap();
        config
            .add_rule("model\nwith\nnewlines", "target-newlines")
            .unwrap();

        // When: apply overrides
        let result1 = config.apply_override("model with spaces");
        let result2 = config.apply_override("model\twith\ttabs");
        let result3 = config.apply_override("model\nwith\nnewlines");

        // Then: should match exactly (whitespace is significant)
        assert_eq!(result1, "target");
        assert_eq!(result2, "target-tabs");
        assert_eq!(result3, "target-newlines");

        // Different whitespace should NOT match
        assert_ne!(config.apply_override("model  with  spaces"), "target");
    }

    #[test]
    fn test_leading_and_trailing_whitespace() {
        // Given: model with leading/trailing whitespace
        let config = OverrideConfig::new();

        // When: apply to model with whitespace
        let result1 = config.apply_override(" model");
        let result2 = config.apply_override("model ");
        let result3 = config.apply_override(" model ");

        // Then: whitespace is preserved (no trimming)
        assert_eq!(result1, " model");
        assert_eq!(result2, "model ");
        assert_eq!(result3, " model ");
    }

    // ========== CIRCULAR REFERENCE PREVENTION ==========

    #[test]
    fn test_no_circular_override_a_to_b_to_a() {
        // Given: circular override rules (should be prevented by single-pass logic)
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "model-b").unwrap();
        config.add_rule("model-b", "model-a").unwrap();

        // When: apply to model-a
        let result = config.apply_override("model-a");

        // Then: should only apply once (model-a -> model-b, not -> model-a again)
        assert_eq!(result, "model-b", "Should not follow circular chain");
    }

    #[test]
    fn test_no_chain_override() {
        // Given: chained override rules
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "model-b").unwrap();
        config.add_rule("model-b", "model-c").unwrap();
        config.add_rule("model-c", "model-d").unwrap();

        // When: apply to model-a
        let result = config.apply_override("model-a");

        // Then: should only apply one level (not chain through)
        assert_eq!(result, "model-b", "Should not chain overrides");
    }

    // ========== LARGE SCALE TESTS ==========

    #[test]
    fn test_many_override_rules() {
        // Given: 1000 override rules
        let mut config = OverrideConfig::new();
        for i in 0..1000 {
            config
                .add_rule(&format!("model-{}", i), &format!("target-{}", i))
                .unwrap();
        }

        // When: apply various overrides
        let result1 = config.apply_override("model-0");
        let result500 = config.apply_override("model-500");
        let result999 = config.apply_override("model-999");
        let unknown = config.apply_override("model-1000");

        // Then: all should work correctly
        assert_eq!(result1, "target-0");
        assert_eq!(result500, "target-500");
        assert_eq!(result999, "target-999");
        assert_eq!(unknown, "model-1000");
    }

    #[test]
    fn test_override_rule_limit() {
        // Given: config with max_rules limit
        let mut config = OverrideConfig::new();
        config.max_rules = 10;

        // When: try to add more than limit
        for i in 0..10 {
            assert!(
                config.add_rule(&format!("model-{}", i), "target").is_ok(),
                "Should accept rules up to limit"
            );
        }

        let result = config.add_rule("model-11", "target");

        // Then: should fail after limit
        assert!(result.is_err(), "Should reject rules beyond limit");
        assert!(result.unwrap_err().contains("Maximum number"));
    }

    // ========== CONCURRENT ACCESS TESTS ==========

    #[tokio::test]
    async fn test_high_concurrency_reads() {
        // Given: config with rules
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "target-a").unwrap();
        config.add_rule("model-b", "target-b").unwrap();
        let config = Arc::new(config);

        // When: 1000 concurrent read operations
        let mut handles = vec![];
        for i in 0..1000 {
            let config_clone = config.clone();
            let handle = tokio::spawn(async move {
                let model = if i % 2 == 0 { "model-a" } else { "model-b" };
                config_clone.apply_override(model)
            });
            handles.push(handle);
        }

        // Then: all should complete successfully
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.expect("Task should complete");
            let expected = if i % 2 == 0 { "target-a" } else { "target-b" };
            assert_eq!(result, expected);
        }
    }

    #[tokio::test]
    async fn test_read_during_initialization() {
        // Given: config being initialized
        let config = Arc::new(tokio::sync::RwLock::new(OverrideConfig::new()));

        // Start background task to add rules
        let config_clone = config.clone();
        let init_handle = tokio::spawn(async move {
            for i in 0..100 {
                let mut cfg = config_clone.write().await;
                cfg.add_rule(&format!("model-{}", i), "target").unwrap();
                drop(cfg);
                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            }
        });

        // When: concurrent reads during initialization
        let mut read_handles = vec![];
        for i in 0..100 {
            let config_clone = config.clone();
            let handle = tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_micros(5)).await;
                let cfg = config_clone.read().await;
                cfg.apply_override(&format!("model-{}", i))
            });
            read_handles.push(handle);
        }

        // Then: all operations should complete without deadlock
        init_handle.await.expect("Init should complete");
        for handle in read_handles {
            handle.await.expect("Read should complete");
        }
    }

    // ========== MEMORY AND PERFORMANCE ==========

    #[test]
    fn test_override_lookup_performance() {
        // Given: large config with 10000 rules
        let mut config = OverrideConfig::new();
        config.max_rules = 10000;
        for i in 0..10000 {
            config
                .add_rule(&format!("model-{}", i), &format!("target-{}", i))
                .unwrap();
        }

        // When: perform 10000 lookups
        let start = std::time::Instant::now();
        for i in 0..10000 {
            let _ = config.apply_override(&format!("model-{}", i));
        }
        let duration = start.elapsed();

        // Then: should be very fast (< 50ms for 10k lookups)
        assert!(
            duration.as_millis() < 50,
            "10k lookups should be fast (< 50ms), took {:?}",
            duration
        );
    }

    #[test]
    fn test_memory_usage_reasonable() {
        // Given: config with 1000 rules
        let mut config = OverrideConfig::new();
        for i in 0..1000 {
            config
                .add_rule(&format!("model-{}", i), &format!("target-{}", i))
                .unwrap();
        }

        // Then: should not consume excessive memory
        // Note: This is a smoke test, actual memory profiling would need external tools
        assert_eq!(config.rules.len(), 1000);
    }

    // ========== NULL/NONE HANDLING ==========

    #[test]
    fn test_override_to_empty_string() {
        // Given: override rule that maps to empty string
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "").unwrap();

        // When: apply override
        let result = config.apply_override("model-a");

        // Then: should return empty string
        assert_eq!(result, "");
    }

    #[test]
    fn test_self_referential_override() {
        // Given: model that overrides to itself
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "model-a").unwrap();

        // When: apply override
        let result = config.apply_override("model-a");

        // Then: should return same model
        assert_eq!(result, "model-a");
    }

    // ========== DETERMINISM TESTS ==========

    #[test]
    fn test_override_is_deterministic() {
        // Given: config with rules
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "target").unwrap();

        // When: apply same override multiple times
        let results: Vec<String> = (0..100).map(|_| config.apply_override("model-a")).collect();

        // Then: all results should be identical
        assert!(results.iter().all(|r| r == "target"));
    }

    #[tokio::test]
    async fn test_concurrent_same_model_override() {
        // Given: config with rule
        let mut config = OverrideConfig::new();
        config.add_rule("model-a", "target").unwrap();
        let config = Arc::new(config);

        // When: 100 concurrent requests for same model
        let mut handles = vec![];
        for _ in 0..100 {
            let config_clone = config.clone();
            let handle = tokio::spawn(async move { config_clone.apply_override("model-a") });
            handles.push(handle);
        }

        // Then: all should get same result
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert_eq!(result, "target");
        }
    }

    // ========== ERROR RECOVERY ==========

    #[test]
    fn test_recovery_from_max_rules_error() {
        // Given: config at max capacity
        let mut config = OverrideConfig::new();
        config.max_rules = 5;
        for i in 0..5 {
            config.add_rule(&format!("model-{}", i), "target").unwrap();
        }

        // When: try to add one more (should fail)
        let result = config.add_rule("model-6", "target");
        assert!(result.is_err());

        // Then: existing rules should still work
        assert_eq!(config.apply_override("model-0"), "target");
        assert_eq!(config.apply_override("model-4"), "target");
    }

    #[test]
    fn test_override_with_newlines_in_name() {
        // Given: model name with embedded newlines
        let mut config = OverrideConfig::new();
        config.add_rule("model\nwith\nnewlines", "target").unwrap();

        // When: apply override
        let result = config.apply_override("model\nwith\nnewlines");

        // Then: should match exactly
        assert_eq!(result, "target");
    }

    #[test]
    fn test_override_with_null_bytes() {
        // Given: model name with null bytes
        let model_with_null = "model\0with\0null";
        let mut config = OverrideConfig::new();
        config.add_rule(model_with_null, "target").unwrap();

        // When: apply override
        let result = config.apply_override(model_with_null);

        // Then: should work (Rust strings are not null-terminated)
        assert_eq!(result, "target");
    }
}
