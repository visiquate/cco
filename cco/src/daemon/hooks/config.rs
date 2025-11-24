//! Configuration structures for the hooks system
//!
//! Defines configuration options for hook behavior, LLM integration,
//! and permission controls. Designed to integrate with daemon config.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::error::{HookError, HookResult};

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

    /// LLM configuration for Phase 2
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
/// Phase 1A: Uses embedded TinyLLaMA (1.1B) for CRUD classification.
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

    /// Quantization level (e.g., "Q4_K_M")
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
/// Phase 1: Empty (hooks registered programmatically)
/// Phase 2: Will support configuration-based hook registration
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

// LLM defaults for embedded TinyLLaMA
fn default_model_type() -> String {
    "tinyllama".to_string()
}

fn default_model_name() -> String {
    "tinyllama-1.1b-chat-v1.0.Q4_K_M".to_string()
}

fn default_model_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf")
}

fn default_model_size_mb() -> u32 {
    600
}

fn default_quantization() -> String {
    "Q4_K_M".to_string()
}

fn default_inference_timeout_ms() -> u64 {
    2000 // 2 seconds
}

fn default_llm_temperature() -> f32 {
    0.1 // Low temperature for consistent classification
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
            allow_file_read: default_false(),
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
            return Err(HookError::invalid_config("timeout_ms must be greater than 0"));
        }

        if self.max_retries > 10 {
            return Err(HookError::invalid_config("max_retries must be <= 10"));
        }

        // Validate LLM config
        if self.llm.temperature < 0.0 || self.llm.temperature > 1.0 {
            return Err(HookError::invalid_config(
                "llm.temperature must be between 0.0 and 1.0"
            ));
        }

        if self.llm.inference_timeout_ms == 0 {
            return Err(HookError::invalid_config("llm.inference_timeout_ms must be greater than 0"));
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
        assert_eq!(config.llm.model_type, "tinyllama");
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
        assert!(!config.is_enabled());

        config.enabled = true;
        assert!(config.is_enabled());
    }

    #[test]
    fn test_permissions_default() {
        let perms = HooksPermissions::default();
        assert!(!perms.allow_command_modification);
        assert!(!perms.allow_execution_blocking);
        assert!(!perms.allow_external_calls);
        assert!(!perms.allow_env_access);
        assert!(!perms.allow_file_read);
        assert!(!perms.allow_file_write);
    }

    #[test]
    fn test_llm_config_default() {
        let llm = HookLlmConfig::default();
        assert_eq!(llm.model_type, "tinyllama");
        assert_eq!(llm.model_name, "tinyllama-1.1b-chat-v1.0.Q4_K_M");
        assert_eq!(llm.quantization, "Q4_K_M");
        assert_eq!(llm.model_size_mb, 600);
        assert_eq!(llm.inference_timeout_ms, 2000);
        assert_eq!(llm.temperature, 0.1);
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
        assert_eq!(config.llm.model_type, "tinyllama"); // Default
    }
}
