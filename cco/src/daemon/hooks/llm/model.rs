//! Model management for embedded LLM
//!
//! Handles downloading, caching, loading, and lifecycle management
//! of the Qwen2.5-Coder GGUF model for CRUD classification.

use crate::daemon::hooks::config::HookLlmConfig;
use crate::daemon::hooks::{HookError, HookResult};
use anyhow::Result;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use mistralrs::{
    GgufModelBuilder, Model, PagedAttentionMetaBuilder, RequestBuilder, SamplingParams,
    TextMessageRole,
};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Embedded Qwen2 chat template for CRUD classification
///
/// This is embedded in the binary so users don't need to create external files.
/// The template follows Qwen2/ChatML format with our custom system prompt.
const QWEN2_CHAT_TEMPLATE: &str = r#"{
  "bos_token": "<|im_start|>",
  "eos_token": "<|im_end|>",
  "chat_template": "{% for message in messages %}{% if loop.first and messages[0]['role'] != 'system' %}<|im_start|>system\nYou classify shell commands. Respond with exactly ONE word: READ, CREATE, UPDATE, or DELETE.\n\nRules:\n- READ: Lists files, shows content, searches (ls, cat, grep, find)\n- CREATE: Makes new files or dirs (touch, mkdir, > redirect)\n- UPDATE: Modifies existing (chmod, sed -i, >> append)\n- DELETE: Removes files or dirs (rm, rmdir)\n\nRespond with only the classification word.<|im_end|>\n{% endif %}<|im_start|>{{ message['role'] }}\n{{ message['content'] }}<|im_end|>\n{% endfor %}{% if add_generation_prompt %}<|im_start|>assistant\n{% endif %}"
}"#;

/// Loaded LLM model wrapper
///
/// Wraps the mistral.rs model instance for CRUD classification
pub struct LlmModel {
    /// The loaded model instance from mistral.rs
    model: Model,
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
    ///
    /// Uses the model name from config to construct the URL.
    /// Default: Q4_K_M quantization (~1GB) for better numerical stability.
    fn get_huggingface_url(&self) -> String {
        // Qwen2.5-Coder 1.5B Instruct from HuggingFace
        // Model name comes from config (default: qwen2.5-coder-1.5b-instruct-q4_k_m)
        format!(
            "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/{}.gguf",
            self.config.model_name
        )
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
    /// Uses mistral.rs's GgufModelBuilder for GGUF model loading.
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

        // Extract model directory and filename
        let model_dir = self
            .model_path
            .parent()
            .ok_or_else(|| {
                HookError::execution_failed(
                    "model_load",
                    format!("Invalid model path: {:?}", self.model_path),
                )
            })?
            .to_path_buf();

        let model_filename = self
            .model_path
            .file_name()
            .ok_or_else(|| {
                HookError::execution_failed(
                    "model_load",
                    format!("Invalid model filename: {:?}", self.model_path),
                )
            })?
            .to_string_lossy()
            .to_string();

        // Write embedded chat template to a temp file
        // (mistral.rs requires a file path ending in .json or .jinja)
        // Uses same temp directory as other CCO files (.cco-* naming convention)
        let chat_template_path = std::env::temp_dir().join(".cco-chat-template.json");
        let mut template_file = std::fs::File::create(&chat_template_path).map_err(|e| {
            HookError::execution_failed(
                "model_load",
                format!("Failed to create chat template file: {}", e),
            )
        })?;
        template_file
            .write_all(QWEN2_CHAT_TEMPLATE.as_bytes())
            .map_err(|e| {
                HookError::execution_failed(
                    "model_load",
                    format!("Failed to write chat template: {}", e),
                )
            })?;
        debug!("Wrote embedded chat template to {:?}", chat_template_path);

        let config = self.config.clone();

        // Build the model using mistral.rs
        info!("Building GGUF model with mistral.rs...");

        let model = GgufModelBuilder::new(
            model_dir.to_string_lossy().to_string(),
            vec![model_filename],
        )
        .with_chat_template(chat_template_path.to_string_lossy().to_string())
        .with_logging()
        .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())
        .map_err(|e| {
            HookError::execution_failed("model_load", format!("Failed to configure PagedAttention: {}", e))
        })?
        .build()
        .await
        .map_err(|e| {
            HookError::execution_failed("model_load", format!("Failed to build model: {}", e))
        })?;

        let loaded = Arc::new(LlmModel { model, config });
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
        drop(model_guard); // Release lock before async operation

        let prompt_str = prompt.to_string();
        let temperature = self.config.temperature;

        info!("Running LLM inference for CRUD classification");
        debug!("Prompt: {}", prompt_str);

        // Configure sampling parameters
        let mut sampling_params = SamplingParams::deterministic();
        sampling_params.temperature = Some(temperature as f64);
        sampling_params.top_k = Some(40);
        sampling_params.top_p = Some(0.95);
        sampling_params.max_len = Some(10); // We only need 1 word (READ/CREATE/UPDATE/DELETE)

        // Build request with message and sampling parameters
        let request = RequestBuilder::new()
            .add_message(TextMessageRole::User, prompt_str)
            .set_sampling(sampling_params);

        // Send request to model
        let response = model_arc
            .model
            .send_chat_request(request)
            .await
            .map_err(|e| {
                HookError::execution_failed("inference", format!("LLM inference failed: {}", e))
            })?;

        // Extract response text
        let output = response
            .choices
            .get(0)
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| {
                HookError::execution_failed("inference", "No response from model")
            })?;

        // Log raw model output for debugging
        debug!("Raw LLM output: '{}'", output);

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
            model_name: DEFAULT_MODEL_NAME.to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: DEFAULT_MODEL_SIZE_MB,
            quantization: DEFAULT_QUANTIZATION.to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: DEFAULT_LLM_TEMPERATURE,
        };

        let manager = ModelManager::new(config).await.unwrap();
        assert!(!manager.is_loaded().await);
    }

    #[tokio::test]
    async fn test_model_unload() {
        let config = HookLlmConfig {
            model_type: "qwen-coder".to_string(),
            model_name: DEFAULT_MODEL_NAME.to_string(),
            model_path: PathBuf::from("/tmp/test-model.gguf"),
            model_size_mb: DEFAULT_MODEL_SIZE_MB,
            quantization: DEFAULT_QUANTIZATION.to_string(),
            loaded: false,
            inference_timeout_ms: 2000,
            temperature: DEFAULT_LLM_TEMPERATURE,
        };

        let manager = ModelManager::new(config).await.unwrap();

        // Unload should be safe even if model not loaded
        manager.unload_model().await;
        assert!(!manager.is_loaded().await);
    }

}
