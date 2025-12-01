//! CRUD classifier using embedded LLM
//!
//! Provides high-level command classification API with timeout enforcement
//! and graceful degradation.

use crate::daemon::hooks::config::HookLlmConfig;
use crate::daemon::hooks::{ClassificationResult, CrudClassification, HookError, HookResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, error, info};

use super::model::ModelManager;
use super::prompt::{build_crud_prompt, parse_classification};

/// CRUD classifier using embedded Qwen2.5-Coder
///
/// Provides command classification with:
/// - Timeout enforcement (default 2 seconds)
/// - Graceful fallback on errors
/// - Memory-efficient lazy loading
/// - Thread-safe concurrent access
pub struct CrudClassifier {
    /// Model manager for the embedded LLM
    model_manager: Arc<Mutex<ModelManager>>,

    /// Inference timeout
    timeout: Duration,

    /// Fallback classification on error (CREATE is safest - requires confirmation)
    fallback_classification: CrudClassification,
}

impl CrudClassifier {
    /// Create a new CRUD classifier
    ///
    /// # Arguments
    ///
    /// * `config` - LLM configuration
    ///
    /// # Errors
    ///
    /// Returns error if model manager cannot be initialized
    pub async fn new(config: HookLlmConfig) -> HookResult<Self> {
        let timeout = Duration::from_millis(config.inference_timeout_ms);

        let model_manager = ModelManager::new(config)
            .await
            .map_err(|e| HookError::execution_failed("classifier_init", e.to_string()))?;

        Ok(Self {
            model_manager: Arc::new(Mutex::new(model_manager)),
            timeout,
            fallback_classification: CrudClassification::Create, // Safest fallback
        })
    }

    /// Ensure the model is downloaded and ready
    ///
    /// Should be called during daemon startup to avoid delays on first classification.
    ///
    /// # Errors
    ///
    /// Returns error if model download fails
    pub async fn ensure_model_available(&self) -> HookResult<()> {
        let manager = self.model_manager.lock().await;
        manager.ensure_model_available().await
    }

    /// Eagerly load the model into memory
    ///
    /// Called during daemon startup to pre-load the model.
    /// This eliminates 2s+ timeout on first classification request.
    ///
    /// # Errors
    ///
    /// Returns error if model cannot be loaded (e.g., file not found, corrupted)
    pub async fn preload_model(&self) -> HookResult<()> {
        let manager = self.model_manager.lock().await;
        manager.load_model().await
    }

    /// Classify a command with CRUD classification
    ///
    /// # Arguments
    ///
    /// * `command` - The shell command to classify
    ///
    /// # Returns
    ///
    /// A `ClassificationResult` with the CRUD type and confidence score
    ///
    /// # Behavior
    ///
    /// - Applies timeout from configuration
    /// - Returns fallback classification (CREATE) on error or timeout
    /// - Logs errors but does not propagate them (graceful degradation)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use cco::daemon::hooks::llm::CrudClassifier;
    /// use cco::daemon::hooks::config::HookLlmConfig;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let classifier = CrudClassifier::new(HookLlmConfig::default()).await?;
    /// let result = classifier.classify("ls -la").await;
    /// println!("Classification: {}", result.classification);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn classify(&self, command: &str) -> ClassificationResult {
        debug!("Classifying command: {}", command);

        match self.classify_internal(command).await {
            Ok(result) => {
                info!(
                    "Command classified as {} with confidence {:.2}",
                    result.classification, result.confidence
                );
                result
            }
            Err(e) => {
                error!("Classification failed: {}, using fallback", e);
                ClassificationResult::with_reasoning(
                    self.fallback_classification,
                    0.5, // Low confidence for fallback
                    format!("Fallback due to error: {}", e),
                )
            }
        }
    }

    /// Internal classification with proper error handling
    async fn classify_internal(&self, command: &str) -> HookResult<ClassificationResult> {
        // Build prompt
        let prompt = build_crud_prompt(command);

        // Run inference with timeout
        let response = timeout(self.timeout, self.run_inference(&prompt))
            .await
            .map_err(|_| HookError::timeout("crud_classifier", self.timeout))??;

        // Parse response
        let classification = parse_classification(&response)?;

        // Determine confidence based on response quality
        let confidence = self.calculate_confidence(&response, &classification);

        Ok(ClassificationResult::with_reasoning(
            classification,
            confidence,
            format!("LLM response: {}", response.trim()),
        ))
    }

    /// Run inference through the model manager
    async fn run_inference(&self, prompt: &str) -> HookResult<String> {
        let manager = self.model_manager.lock().await;
        manager.run_inference(prompt).await
    }

    /// Calculate confidence score based on response quality
    ///
    /// Higher confidence if:
    /// - Response is concise (just the classification word)
    /// - Response is capitalized correctly
    /// - Response doesn't contain hedging language
    fn calculate_confidence(&self, response: &str, classification: &CrudClassification) -> f32 {
        let response = response.trim();
        let classification_str = classification.to_string();

        // Base confidence
        let mut confidence: f32 = 0.8;

        // Exact match increases confidence
        if response.eq_ignore_ascii_case(&classification_str) {
            confidence += 0.15;
        }

        // Short response increases confidence (model is certain)
        if response.len() <= classification_str.len() + 5 {
            confidence += 0.05;
        }

        // Hedging language decreases confidence
        let hedging_words = ["maybe", "might", "could", "possibly", "probably"];
        if hedging_words
            .iter()
            .any(|word| response.to_lowercase().contains(word))
        {
            confidence -= 0.2;
        }

        // Clamp to valid range
        confidence.clamp(0.0, 1.0)
    }

    /// Unload model from memory
    ///
    /// Can be called during memory pressure or shutdown.
    /// Model will be lazy-loaded again on next classification.
    pub async fn unload_model(&self) {
        let manager = self.model_manager.lock().await;
        manager.unload_model().await;
    }

    /// Check if model is currently loaded
    pub async fn is_model_loaded(&self) -> bool {
        let manager = self.model_manager.lock().await;
        manager.is_loaded().await
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
    async fn test_classifier_creation() {
        let classifier = CrudClassifier::new(test_config()).await;
        assert!(classifier.is_ok());
    }

    #[tokio::test]
    async fn test_classifier_fallback() {
        let classifier = CrudClassifier::new(test_config()).await.unwrap();

        // Classification will fail (no actual model), should return fallback
        let result = classifier.classify("ls -la").await;

        // Should get fallback classification (CREATE)
        assert_eq!(result.classification, CrudClassification::Create);
        assert!(result.reasoning.is_some());
        assert!(result.reasoning.unwrap().contains("Fallback"));
    }

    #[test]
    fn test_calculate_confidence() {
        let classifier = CrudClassifier {
            model_manager: Arc::new(Mutex::new(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(ModelManager::new(test_config()))
                    .unwrap(),
            )),
            timeout: Duration::from_secs(2),
            fallback_classification: CrudClassification::Create,
        };

        // Exact match
        let confidence = classifier.calculate_confidence("READ", &CrudClassification::Read);
        assert!(confidence >= 0.9);

        // With extra text
        let confidence =
            classifier.calculate_confidence("READ - explanation", &CrudClassification::Read);
        assert!(confidence < 0.9);

        // With hedging
        let confidence = classifier.calculate_confidence("maybe READ", &CrudClassification::Read);
        assert!(confidence < 0.8);
    }

    #[tokio::test]
    async fn test_model_unload() {
        let classifier = CrudClassifier::new(test_config()).await.unwrap();

        // Unload should be safe
        classifier.unload_model().await;
        assert!(!classifier.is_model_loaded().await);
    }
}
