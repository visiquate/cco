//! Configuration structures for the hooks system
//!
//! Defines configuration options for hook behavior, LLM integration,
//! and permission controls. Designed to integrate with daemon config.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::error::{HookError, HookResult};

// ============================================================================
// LLM CONFIGURATION CONSTANTS
// Define all LLM-related constants here for single point of change
// ============================================================================

/// Default temperature for LLM inference.
/// Lower values (0.1) provide more deterministic, consistent results.
/// Q4_K_M quantization is stable enough for low temperature.
pub const DEFAULT_LLM_TEMPERATURE: f32 = 0.1;

/// Default model type for CRUD classification
pub const DEFAULT_MODEL_TYPE: &str = "qwen-coder";

/// Default model name (Qwen2.5-Coder 1.5B with Q4_K_M quantization)
/// Q4_K_M provides better numerical stability than Q2_K while still being fast
pub const DEFAULT_MODEL_NAME: &str = "qwen2.5-coder-1.5b-instruct-q4_k_m";

/// Default model size in megabytes (Q4_K_M is ~1GB)
pub const DEFAULT_MODEL_SIZE_MB: u32 = 1000;

/// Default quantization level
/// Q4_K_M: 4-bit quantization with medium quality - good balance of speed and accuracy
pub const DEFAULT_QUANTIZATION: &str = "Q4_K_M";

/// Default inference timeout in milliseconds
/// Q4_K_M model needs more time than Q2_K - 5 seconds should be sufficient
pub const DEFAULT_INFERENCE_TIMEOUT_MS: u64 = 5000;

/// Main configuration for the hooks system
///
/// This structure is embedded in the daemon configuration file
/// under the `hooks` section.
///
/// # Example TOML
///
/// ```toml
/// [hooks]
/// enabled = true
/// timeout_ms = 5000
/// max_retries = 2
///
/// [hooks.llm]
/// enabled = false
/// model = "gpt-4"
/// temperature = 0.7
/// max_tokens = 500
///
/// [hooks.permissions]
/// allow_command_modification = false
/// allow_execution_blocking = false
/// allow_external_calls = false
///
/// [hooks.callbacks]
/// pre_command = []
/// post_command = []
/// post_execution = []
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Enable or disable the hooks system
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Timeout for hook execution in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Maximum retry attempts for failed hooks
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// LLM configuration for embedded CRUD classification
    #[serde(default)]
    pub llm: HookLlmConfig,

    /// Permission controls for hooks
    #[serde(default)]
    pub permissions: HooksPermissions,

    /// Hook callback configurations
    #[serde(default)]
    pub callbacks: HooksCallbacks,
}

/// LLM configuration for embedded CRUD classification
///
/// Uses embedded Qwen2.5-Coder (1.5B) for CRUD classification.
/// The model is downloaded on first run and cached locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookLlmConfig {
    /// Model type identifier
    #[serde(default = "default_model_type")]
    pub model_type: String,

    /// Full model name
    #[serde(default = "default_model_name")]
    pub model_name: String,

    /// Local path to the GGML model file
    #[serde(default = "default_model_path")]
    pub model_path: std::path::PathBuf,

    /// Model size in MB
    #[serde(default = "default_model_size_mb")]
    pub model_size_mb: u32,

    /// Quantization level (e.g., "Q2_K")
    #[serde(default = "default_quantization")]
    pub quantization: String,

    /// Whether model is currently loaded in memory
    #[serde(default = "default_false")]
    pub loaded: bool,

    /// Inference timeout in milliseconds
    #[serde(default = "default_inference_timeout_ms")]
    pub inference_timeout_ms: u64,

    /// Temperature for generation (0.0 - 1.0)
    /// Low temperature (0.1) for consistent classification
    #[serde(default = "default_llm_temperature")]
    pub temperature: f32,
}

/// Permission controls for hook capabilities
///
/// Defines what actions hooks are allowed to perform.
/// By default, all permissions are disabled for security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksPermissions {
    /// Allow hooks to modify commands before execution
    #[serde(default = "default_false")]
    pub allow_command_modification: bool,

    /// Allow hooks to block command execution
    #[serde(default = "default_false")]
    pub allow_execution_blocking: bool,

    /// Allow hooks to make external API calls
    #[serde(default = "default_false")]
    pub allow_external_calls: bool,

    /// Allow hooks to access environment variables
    #[serde(default = "default_false")]
    pub allow_env_access: bool,

    /// Allow hooks to read files
    #[serde(default = "default_false")]
    pub allow_file_read: bool,

    /// Allow hooks to write files
    #[serde(default = "default_false")]
    pub allow_file_write: bool,
}

/// Hook callback configurations
///
/// Defines which callbacks are enabled for each hook type.
/// Currently: Hooks registered programmatically
/// Future enhancement: Configuration-based hook registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksCallbacks {
    /// Pre-command hook configurations
    #[serde(default)]
    pub pre_command: Vec<String>,

    /// Post-command hook configurations
    #[serde(default)]
    pub post_command: Vec<String>,

    /// Post-execution hook configurations
    #[serde(default)]
    pub post_execution: Vec<String>,
}

// Default value functions for serde
fn default_enabled() -> bool {
    true // Enabled by default for autonomous CRUD classification
}

fn default_timeout_ms() -> u64 {
    5000 // 5 seconds
}

fn default_max_retries() -> u32 {
    2
}

fn default_false() -> bool {
    false
}

// LLM defaults for embedded Qwen2.5-Coder
fn default_model_type() -> String {
    "qwen-coder".to_string()
}

fn default_model_name() -> String {
    DEFAULT_MODEL_NAME.to_string()
}

fn default_model_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(format!(".cco/models/{}.gguf", DEFAULT_MODEL_NAME))
}

fn default_model_size_mb() -> u32 {
    DEFAULT_MODEL_SIZE_MB
}

fn default_quantization() -> String {
    DEFAULT_QUANTIZATION.to_string()
}

fn default_inference_timeout_ms() -> u64 {
    DEFAULT_INFERENCE_TIMEOUT_MS
}

fn default_llm_temperature() -> f32 {
    DEFAULT_LLM_TEMPERATURE
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            timeout_ms: default_timeout_ms(),
            max_retries: default_max_retries(),
            llm: HookLlmConfig::default(),
            permissions: HooksPermissions::default(),
            callbacks: HooksCallbacks::default(),
        }
    }
}

impl Default for HookLlmConfig {
    fn default() -> Self {
        Self {
            model_type: default_model_type(),
            model_name: default_model_name(),
            model_path: default_model_path(),
            model_size_mb: default_model_size_mb(),
            quantization: default_quantization(),
            loaded: default_false(),
            inference_timeout_ms: default_inference_timeout_ms(),
            temperature: default_llm_temperature(),
        }
    }
}

impl Default for HooksPermissions {
    fn default() -> Self {
        Self {
            allow_command_modification: default_false(),
            allow_execution_blocking: default_false(),
            allow_external_calls: default_false(),
            allow_env_access: default_false(),
            allow_file_read: true, // Auto-approve READ operations by default
            allow_file_write: default_false(),
        }
    }
}

impl Default for HooksCallbacks {
    fn default() -> Self {
        Self {
            pre_command: Vec::new(),
            post_command: Vec::new(),
            post_execution: Vec::new(),
        }
    }
}

impl HooksConfig {
    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns `HookError::InvalidConfig` if:
    /// - Timeout is 0
    /// - Max retries is unreasonably high (>10)
    /// - LLM temperature is out of range (0.0-1.0)
    pub fn validate(&self) -> HookResult<()> {
        if self.timeout_ms == 0 {
            return Err(HookError::invalid_config(
                "timeout_ms must be greater than 0",
            ));
        }

        if self.max_retries > 10 {
            return Err(HookError::invalid_config("max_retries must be <= 10"));
        }

        // Validate LLM config
        if self.llm.temperature < 0.0 || self.llm.temperature > 1.0 {
            return Err(HookError::invalid_config(
                "llm.temperature must be between 0.0 and 1.0",
            ));
        }

        if self.llm.inference_timeout_ms == 0 {
            return Err(HookError::invalid_config(
                "llm.inference_timeout_ms must be greater than 0",
            ));
        }

        if self.llm.model_name.is_empty() {
            return Err(HookError::invalid_config("llm.model_name cannot be empty"));
        }

        if self.llm.model_type.is_empty() {
            return Err(HookError::invalid_config("llm.model_type cannot be empty"));
        }

        Ok(())
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Check if hooks are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HooksConfig::default();
        assert!(config.enabled);
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.llm.model_type, "qwen-coder");
        assert_eq!(config.llm.inference_timeout_ms, 2000);
    }

    #[test]
    fn test_config_validation() {
        let mut config = HooksConfig::default();
        assert!(config.validate().is_ok());

        // Invalid timeout
        config.timeout_ms = 0;
        assert!(config.validate().is_err());

        // Invalid max retries
        config.timeout_ms = 5000;
        config.max_retries = 100;
        assert!(config.validate().is_err());

        // Invalid LLM temperature
        config.max_retries = 2;
        config.llm.temperature = 1.5;
        assert!(config.validate().is_err());

        // Invalid inference timeout
        config.llm.temperature = 0.1;
        config.llm.inference_timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_conversion() {
        let config = HooksConfig {
            timeout_ms: 3000,
            ..Default::default()
        };

        assert_eq!(config.timeout(), Duration::from_millis(3000));
    }

    #[test]
    fn test_is_enabled() {
        let mut config = HooksConfig::default();
        assert!(config.is_enabled()); // Hooks are enabled by default

        config.enabled = false;
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_permissions_default() {
        let perms = HooksPermissions::default();
        assert!(!perms.allow_command_modification);
        assert!(!perms.allow_execution_blocking);
        assert!(!perms.allow_external_calls);
        assert!(!perms.allow_env_access);
        assert!(perms.allow_file_read); // READ operations auto-approved by default
        assert!(!perms.allow_file_write);
    }

    #[test]
    fn test_llm_config_default() {
        let llm = HookLlmConfig::default();
        assert_eq!(llm.model_type, DEFAULT_MODEL_TYPE);
        assert_eq!(llm.model_name, DEFAULT_MODEL_NAME);
        assert_eq!(llm.quantization, DEFAULT_QUANTIZATION);
        assert_eq!(llm.model_size_mb, DEFAULT_MODEL_SIZE_MB);
        assert_eq!(llm.inference_timeout_ms, DEFAULT_INFERENCE_TIMEOUT_MS);
        assert_eq!(llm.temperature, DEFAULT_LLM_TEMPERATURE);
        assert!(!llm.loaded);
    }

    #[test]
    fn test_toml_serialization() {
        let config = HooksConfig {
            enabled: true,
            timeout_ms: 7500,
            max_retries: 3,
            ..Default::default()
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("enabled = true"));
        assert!(toml.contains("timeout_ms = 7500"));
        assert!(toml.contains("max_retries = 3"));
    }

    #[test]
    fn test_partial_toml_with_defaults() {
        let toml = r#"
            enabled = true
        "#;

        let config: HooksConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.timeout_ms, 5000); // Default
        assert_eq!(config.max_retries, 2); // Default
        assert_eq!(config.llm.model_type, "qwen-coder"); // Default
    }
}
