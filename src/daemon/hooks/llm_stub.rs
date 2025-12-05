//! Stub LLM classifier for non-macOS platforms
//!
//! On Linux and other platforms, the embedded LLM (mistral.rs with Metal)
//! is not available. This stub provides the same interface but always
//! returns a fallback classification.

use crate::daemon::hooks::config::HookLlmConfig;
use crate::daemon::hooks::{ClassificationResult, CrudClassification, HookResult};
use tracing::info;

/// Stub CRUD classifier for non-macOS platforms
///
/// Always returns CREATE classification (safest fallback requiring confirmation)
/// since the Metal-based LLM inference is only available on macOS.
pub struct CrudClassifier {
    /// Fallback classification (CREATE is safest - requires confirmation)
    fallback_classification: CrudClassification,
}

impl CrudClassifier {
    /// Create a new stub CRUD classifier
    pub async fn new(_config: HookLlmConfig) -> HookResult<Self> {
        info!("LLM classifier not available on this platform (requires macOS with Metal)");
        info!("All commands will be classified as CREATE (requiring confirmation)");

        Ok(Self {
            fallback_classification: CrudClassification::Create,
        })
    }

    /// No-op on non-macOS platforms
    pub async fn ensure_model_available(&self) -> HookResult<()> {
        // Model not available on non-macOS platforms
        Ok(())
    }

    /// No-op on non-macOS platforms
    pub async fn preload_model(&self) -> HookResult<()> {
        // Model not available on non-macOS platforms
        Ok(())
    }

    /// Return fallback classification (CREATE)
    ///
    /// On non-macOS platforms, LLM inference is not available, so we return
    /// the safest fallback classification which requires user confirmation.
    pub async fn classify(&self, command: &str) -> ClassificationResult {
        info!(
            "Stub classifier returning CREATE for command: {} (LLM not available)",
            command
        );

        ClassificationResult::with_reasoning(
            self.fallback_classification,
            0.5, // Low confidence since we're not actually classifying
            "LLM classifier not available on this platform (requires macOS with Metal). Using CREATE fallback.".to_string(),
        )
    }

    /// No-op on non-macOS platforms
    pub async fn unload_model(&self) {
        // No model to unload
    }

    /// Always returns false on non-macOS platforms
    pub async fn is_model_loaded(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::hooks::config::HookLlmConfig;
    use std::path::PathBuf;

    fn test_config() -> HookLlmConfig {
        HookLlmConfig {
            model_type: "qwen-coder".to_string(),
            model_name: "test-model".to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: 577,
            quantization: "Q2_K".to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: 0.05,
        }
    }

    #[tokio::test]
    async fn test_stub_classifier_creation() {
        let classifier = CrudClassifier::new(test_config()).await;
        assert!(classifier.is_ok());
    }

    #[tokio::test]
    async fn test_stub_classifier_returns_create() {
        let classifier = CrudClassifier::new(test_config()).await.unwrap();
        let result = classifier.classify("ls -la").await;

        // Should always return CREATE
        assert_eq!(result.classification, CrudClassification::Create);
        assert!(result.reasoning.is_some());
        assert!(result.reasoning.unwrap().contains("not available"));
    }

    #[tokio::test]
    async fn test_stub_model_not_loaded() {
        let classifier = CrudClassifier::new(test_config()).await.unwrap();
        assert!(!classifier.is_model_loaded().await);
    }
}
