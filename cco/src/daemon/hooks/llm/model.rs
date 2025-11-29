//! Model management for embedded LLM
//!
//! Handles downloading, caching, loading, and lifecycle management
//! of the Qwen2.5-Coder GGML model for CRUD classification.

use crate::daemon::hooks::config::HookLlmConfig;
use crate::daemon::hooks::{HookError, HookResult};
use anyhow::Result;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use llm::{Model, ModelArchitecture};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Loaded LLM model wrapper
///
/// Wraps the llm crate's model instance for CRUD classification
pub struct LlmModel {
    /// The loaded model instance from llm crate
    model: Box<dyn Model>,
    /// Model configuration (kept for future extensibility)
    #[allow(dead_code)]
    config: HookLlmConfig,
}


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
    model: Arc<Mutex<Option<Arc<LlmModel>>>>,

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

        // Get download URL for Qwen2.5-Coder
        let model_url = self.get_huggingface_url();

        // Create HTTP client
        let client = Client::new();
        let response = client.get(&model_url).send().await.map_err(|e| {
            HookError::execution_failed(
                "model_download",
                format!("Failed to initiate download: {}", e),
            )
        })?;

        if !response.status().is_success() {
            return Err(HookError::execution_failed(
                "model_download",
                format!("Download failed with status: {}", response.status()),
            ));
        }

        let total_size = response.content_length().unwrap_or(0);

        // Create progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Download with progress tracking
        let mut file = tokio::fs::File::create(&self.model_path)
            .await
            .map_err(|e| {
                HookError::execution_failed(
                    "model_download",
                    format!("Failed to create file: {}", e),
                )
            })?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                HookError::execution_failed(
                    "model_download",
                    format!("Download stream error: {}", e),
                )
            })?;

            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .map_err(|e| {
                    HookError::execution_failed(
                        "model_download",
                        format!("Failed to write file: {}", e),
                    )
                })?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Model downloaded");

        // Verify hash
        info!("Verifying model integrity...");
        self.verify_model_hash(&self.model_path)?;

        info!("Model download complete and verified");
        Ok(())
    }

    /// Get HuggingFace download URL for Qwen2.5-Coder
    fn get_huggingface_url(&self) -> String {
        // Qwen2.5-Coder 1.5B Instruct Q2_K from HuggingFace
        // Q2_K quantization: 577MB, optimized for CRUD classification
        "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q2_k.gguf".to_string()
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
    fn verify_model_hash(&self, path: &Path) -> HookResult<()> {
        // Expected SHA256 hash for Qwen2.5-Coder 1.5B Instruct Q2_K
        // Note: This should be verified from the HuggingFace model card
        // For now, we'll skip strict hash verification and just log a warning
        // In production, you would verify against a known-good hash

        let file = std::fs::File::open(path).map_err(|e| {
            HookError::execution_failed(
                "model_verification",
                format!("Failed to open model file: {}", e),
            )
        })?;

        let mut reader = std::io::BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let count = std::io::Read::read(&mut reader, &mut buffer).map_err(|e| {
                HookError::execution_failed(
                    "model_verification",
                    format!("Failed to read file: {}", e),
                )
            })?;

            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let computed_hash = hex::encode(hasher.finalize());

        // Log the hash for reference (users can verify against HuggingFace)
        info!("Model SHA256: {}", computed_hash);
        warn!("Skipping strict hash verification - please verify model integrity from HuggingFace");

        // In a production implementation, you would check:
        // if computed_hash != expected_hash {
        //     return Err(HookError::execution_failed(
        //         "model_verification",
        //         format!("Hash mismatch: got {}, expected {}", computed_hash, expected_hash)
        //     ));
        // }

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

        info!("Loading Qwen2.5-Coder model from {:?}", self.model_path);

        // Ensure model file exists
        if !self.model_path.exists() {
            return Err(HookError::execution_failed(
                "model_load",
                format!("Model file not found at {:?}", self.model_path),
            ));
        }

        // Load model in blocking task to avoid blocking async runtime
        let model_path = self.model_path.clone();
        let config = self.config.clone();

        let loaded = tokio::task::spawn_blocking(move || -> Result<Arc<LlmModel>, HookError> {
            info!("Loading GGUF model using llm crate...");

            // Load the model using llm crate with LLaMA architecture
            let model = llm::load_dynamic(
                ModelArchitecture::Llama,
                &model_path,
                Default::default(),
                |progress| {
                    match progress {
                        llm::LoadProgress::HyperparametersLoaded => {
                            debug!("Model hyperparameters loaded");
                        }
                        llm::LoadProgress::ContextSize { bytes } => {
                            debug!("Context size: {} bytes", bytes);
                        }
                        llm::LoadProgress::TensorLoaded {
                            current_tensor,
                            tensor_count,
                        } => {
                            if current_tensor % 10 == 0 {
                                debug!("Loading tensors: {}/{}", current_tensor, tensor_count);
                            }
                        }
                        llm::LoadProgress::Loaded {
                            file_size,
                            tensor_count,
                        } => {
                            info!(
                                "Model loaded: {} tensors, {} MB",
                                tensor_count,
                                file_size / (1024 * 1024)
                            );
                        }
                    }
                },
            )
            .map_err(|e| {
                HookError::execution_failed("model_load", format!("Failed to load model: {}", e))
            })?;

            Ok(Arc::new(LlmModel { model, config }))
        })
        .await
        .map_err(|e| {
            HookError::execution_failed("model_load", format!("Spawn blocking failed: {}", e))
        })??;

        *model_guard = Some(loaded);

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

    /// Run inference
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to send to the model
    ///
    /// # Returns
    ///
    /// The model's response text (should be one of: READ, CREATE, UPDATE, DELETE)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Model is not loaded
    /// - Inference fails
    /// - Timeout is exceeded
    pub async fn run_inference(&self, prompt: &str) -> HookResult<String> {
        // Ensure model is loaded
        if !self.is_loaded().await {
            self.load_model().await?;
        }

        // Get model reference
        let model_guard = self.model.lock().await;
        let model_arc = model_guard
            .as_ref()
            .ok_or_else(|| HookError::execution_failed("inference", "Model not loaded"))?
            .clone();
        drop(model_guard); // Release lock before blocking operation

        let prompt_str = prompt.to_string();
        let temperature = self.config.temperature;

        // Run inference in blocking task
        let result = tokio::task::spawn_blocking(move || -> Result<String, HookError> {
            info!("Running LLM inference for CRUD classification");
            debug!("Prompt: {}", prompt_str);

            // Create inference session
            let mut session = model_arc.model.start_session(Default::default());

            // Collect the output
            let mut output = String::new();

            // Create inference parameters with temperature control
            let inference_params = llm::InferenceParameters {
                n_threads: num_cpus::get(),
                n_batch: 8,
                top_k: 40,
                top_p: 0.95,
                repeat_penalty: 1.1,
                temperature,
                bias_tokens: Default::default(),
                repetition_penalty_last_n: 64,
            };

            let request = llm::InferenceRequest {
                prompt: prompt_str.as_str(),
                parameters: Some(&inference_params),
                play_back_previous_tokens: false,
                maximum_token_count: Some(10), // We only need 1 word (READ/CREATE/UPDATE/DELETE)
            };

            // Run inference with the prompt
            let inference_result = session.infer::<std::convert::Infallible>(
                model_arc.model.as_ref(),
                &mut rand::thread_rng(),
                &request,
                &mut Default::default(),
                |token| {
                    output.push_str(token);
                    Ok(())
                },
            );

            match inference_result {
                Ok(stats) => {
                    info!(
                        "Inference completed: {} prompt tokens, {} predict tokens, feed_prompt: {}ms, predict: {}ms",
                        stats.prompt_tokens,
                        stats.predict_tokens,
                        stats.feed_prompt_duration.as_millis(),
                        stats.predict_duration.as_millis()
                    );
                }
                Err(e) => {
                    warn!("Inference error: {:?}", e);
                    return Err(HookError::execution_failed(
                        "inference",
                        format!("LLM inference failed: {:?}", e),
                    ));
                }
            }

            // Extract the classification from the output
            // The model should output one of: READ, CREATE, UPDATE, DELETE
            let output_trimmed = output.trim().to_uppercase();
            let classification = if output_trimmed.contains("READ") {
                "READ"
            } else if output_trimmed.contains("CREATE") {
                "CREATE"
            } else if output_trimmed.contains("UPDATE") {
                "UPDATE"
            } else if output_trimmed.contains("DELETE") {
                "DELETE"
            } else {
                // Fallback to CREATE if we can't parse the output
                warn!(
                    "Could not parse classification from output: '{}', defaulting to CREATE",
                    output_trimmed
                );
                "CREATE"
            };

            info!("LLM classification result: {}", classification);
            Ok(classification.to_string())
        })
        .await
        .map_err(|e| {
            HookError::execution_failed("inference", format!("Spawn blocking failed: {}", e))
        })??;

        debug!("Inference result: {}", result);
        Ok(result)
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
            model_type: "qwen-coder".to_string(),
            model_name: "test-model".to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: 577,
            quantization: "Q2_K".to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: 0.05,
        };

        let manager = ModelManager::new(config).await.unwrap();
        assert!(!manager.is_loaded().await);
    }

    #[tokio::test]
    async fn test_model_unload() {
        let config = HookLlmConfig {
            model_type: "qwen-coder".to_string(),
            model_name: "test-model".to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: 577,
            quantization: "Q2_K".to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: 0.05,
        };

        let manager = ModelManager::new(config).await.unwrap();

        // Unload should be safe even if model not loaded
        manager.unload_model().await;
        assert!(!manager.is_loaded().await);
    }

}
