//! Model management for embedded LLM
//!
//! Handles downloading, caching, loading, and lifecycle management
//! of the TinyLLaMA GGML model for CRUD classification.

use crate::daemon::hooks::config::HookLlmConfig;
use crate::daemon::hooks::{HookError, HookResult};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Model manager handles lifecycle of the embedded LLM
///
/// Responsibilities:
/// - Download model from HuggingFace on first run
/// - Verify model integrity (SHA256)
/// - Lazy load model into memory
/// - Unload model on memory pressure
/// - Provide thread-safe access to loaded model
pub struct ModelManager {
    /// Configuration for the LLM model
    config: HookLlmConfig,

    /// Loaded model (lazy-loaded, wrapped in Arc for sharing)
    /// NOTE: Actual LLM model type will be added when llm crate is integrated
    model: Arc<Mutex<Option<()>>>, // Placeholder until llm crate is added

    /// Model file path
    model_path: PathBuf,
}

impl ModelManager {
    /// Create a new model manager
    ///
    /// # Arguments
    ///
    /// * `config` - LLM configuration including model path and parameters
    ///
    /// # Errors
    ///
    /// Returns error if model path cannot be resolved or created
    pub async fn new(config: HookLlmConfig) -> Result<Self> {
        let model_path = expand_model_path(&config.model_path)?;

        info!(
            "Initializing model manager for {} at {:?}",
            config.model_name, model_path
        );

        Ok(Self {
            config,
            model: Arc::new(Mutex::new(None)),
            model_path,
        })
    }

    /// Ensure model is available locally
    ///
    /// Downloads the model from HuggingFace if it doesn't exist.
    /// This is called on daemon startup to prepare for classification.
    ///
    /// # Errors
    ///
    /// Returns error if download fails or model verification fails
    pub async fn ensure_model_available(&self) -> HookResult<()> {
        if !self.model_path_exists() {
            self.download_model().await?;
        } else {
            debug!("Model already exists at {:?}", self.model_path);
        }
        Ok(())
    }

    /// Check if model file exists on disk
    fn model_path_exists(&self) -> bool {
        self.model_path.exists()
    }

    /// Download model from HuggingFace
    ///
    /// # Implementation Notes
    ///
    /// - Uses streaming download with progress bar (indicatif)
    /// - Verifies SHA256 hash after download
    /// - Creates parent directories if needed
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Network request fails
    /// - File write fails
    /// - Hash verification fails
    async fn download_model(&self) -> HookResult<()> {
        info!("Downloading model: {}", self.config.model_name);
        println!(
            "📥 Downloading CRUD classifier model (first-time setup, ~{}MB)...",
            self.config.model_size_mb
        );

        // Ensure parent directory exists
        if let Some(parent) = self.model_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                HookError::execution_failed(
                    "model_download",
                    format!("Failed to create model directory: {}", e),
                )
            })?;
        }

        // TODO: Implement actual download with indicatif progress bar
        // For now, return an error indicating the feature needs llm crate integration
        Err(HookError::execution_failed(
            "model_download",
            "Model download requires llm crate integration (Phase 1A implementation pending)",
        ))
    }

    /// Verify model file hash
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the model file
    ///
    /// # Errors
    ///
    /// Returns error if hash doesn't match expected value
    fn verify_model_hash(_path: &Path) -> HookResult<()> {
        // TODO: Implement SHA256 verification using sha2 crate
        // Expected hash should be stored in config
        debug!("Model hash verification (pending implementation)");
        Ok(())
    }

    /// Load model into memory
    ///
    /// Lazy loads the model on first inference request.
    /// Uses tokio::spawn_blocking for the blocking I/O operation.
    ///
    /// # Errors
    ///
    /// Returns error if model file cannot be read or parsed
    pub async fn load_model(&self) -> HookResult<()> {
        let mut model_guard = self.model.lock().await;

        if model_guard.is_some() {
            debug!("Model already loaded");
            return Ok(());
        }

        info!("Loading model from {:?}", self.model_path);

        // TODO: Implement actual model loading with llm crate
        // This will use spawn_blocking to avoid blocking the async runtime
        // let model_path = self.model_path.clone();
        // let loaded_model = tokio::task::spawn_blocking(move || {
        //     llm::load_model(&model_path)
        // })
        // .await
        // .map_err(|e| HookError::execution_failed("model_load", e))?;

        // For now, placeholder
        *model_guard = Some(());

        info!("Model loaded successfully");
        Ok(())
    }

    /// Unload model from memory
    ///
    /// Called when memory pressure is detected or during shutdown.
    /// The model will be lazy-loaded again on next inference request.
    pub async fn unload_model(&self) {
        let mut model_guard = self.model.lock().await;
        if model_guard.is_some() {
            info!("Unloading model from memory");
            *model_guard = None;
        }
    }

    /// Check if model is currently loaded
    pub async fn is_loaded(&self) -> bool {
        self.model.lock().await.is_some()
    }

    /// Get model configuration
    pub fn config(&self) -> &HookLlmConfig {
        &self.config
    }

    /// Get model file path
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// Run inference (placeholder for now)
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to send to the model
    ///
    /// # Returns
    ///
    /// The model's response text
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Model is not loaded
    /// - Inference fails
    /// - Timeout is exceeded
    pub async fn run_inference(&self, _prompt: &str) -> HookResult<String> {
        // Ensure model is loaded
        if !self.is_loaded().await {
            self.load_model().await?;
        }

        // TODO: Implement actual inference with llm crate
        // This will use spawn_blocking to avoid blocking the async runtime
        // Apply timeout from config.inference_timeout_ms

        Err(HookError::execution_failed(
            "inference",
            "Model inference requires llm crate integration (Phase 1A implementation pending)",
        ))
    }
}

/// Expand tilde and environment variables in model path
fn expand_model_path(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();

    // Expand tilde
    let expanded = if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            home.join(&path_str[2..])
        } else {
            path.to_path_buf()
        }
    } else if path_str.starts_with('~') {
        // Handle ~user format (not commonly used, just use home dir)
        if let Some(home) = dirs::home_dir() {
            home.join(&path_str[1..])
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_expand_model_path_tilde() {
        let path = PathBuf::from("~/.cco/models/test.gguf");
        let expanded = expand_model_path(&path).unwrap();

        // Should expand to home directory
        assert!(!expanded.to_string_lossy().contains('~'));
        assert!(expanded.to_string_lossy().contains(".cco/models/test.gguf"));
    }

    #[test]
    fn test_expand_model_path_absolute() {
        let path = PathBuf::from("/absolute/path/model.gguf");
        let expanded = expand_model_path(&path).unwrap();

        assert_eq!(expanded, path);
    }

    #[tokio::test]
    async fn test_model_manager_creation() {
        let config = HookLlmConfig {
            model_type: "tinyllama".to_string(),
            model_name: "test-model".to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: 600,
            quantization: "Q4_K_M".to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: 0.1,
        };

        let manager = ModelManager::new(config).await.unwrap();
        assert!(!manager.is_loaded().await);
    }

    #[tokio::test]
    async fn test_model_unload() {
        let config = HookLlmConfig {
            model_type: "tinyllama".to_string(),
            model_name: "test-model".to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: 600,
            quantization: "Q4_K_M".to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: 0.1,
        };

        let manager = ModelManager::new(config).await.unwrap();

        // Unload should be safe even if model not loaded
        manager.unload_model().await;
        assert!(!manager.is_loaded().await);
    }
}
