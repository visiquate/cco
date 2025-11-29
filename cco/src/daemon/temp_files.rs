//! Temporary file management for orchestrator settings
//!
//! Creates and manages temporary files in the system temp directory for:
//! - Orchestrator settings (JSON)
//! - Agent definitions (JSON via --agents flag)
//! - System prompt (XOR obfuscated)
//!
//! Files are created on daemon start and cleaned up on daemon stop.
//!
//! Note: The sealed file system (agents/rules/hooks sealed files) has been removed
//! as it was dead code - files were written but never read. The --agents flag
//! provides the actual agent data used by the system.

use anyhow::Result;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

use super::config::DaemonConfig;

/// Temp file manager for orchestrator resources
pub struct TempFileManager {
    settings_path: PathBuf,
    system_prompt_path: PathBuf,
    agents_json_path: PathBuf,
}

impl TempFileManager {
    /// Create a new temp file manager
    pub fn new() -> Self {
        let temp_dir = env::temp_dir();
        Self {
            settings_path: temp_dir.join(".cco-orchestrator-settings"),
            system_prompt_path: temp_dir.join(".cco-system-prompt"),
            agents_json_path: temp_dir.join(".cco-agents-json"),
        }
    }

    /// Get the system prompt file path (internal use only)
    ///
    /// Note: This method is public for daemon lifecycle management but should
    /// not be considered part of the stable public API. The system prompt path
    /// is intentionally not exposed to end users for security.
    ///
    /// # Internal Use Only
    /// This method is used by the daemon lifecycle and CCO wrapper to manage
    /// the system prompt file. End users should not need to access this directly.
    pub fn system_prompt_path(&self) -> PathBuf {
        self.system_prompt_path.clone()
    }

    /// Get the agents JSON file path
    pub fn agents_json_path(&self) -> &PathBuf {
        &self.agents_json_path
    }

    /// Create all temporary files with content
    ///
    /// This method creates temp files with default configuration.
    /// For daemon-managed settings, use `write_orchestrator_settings()` instead.
    pub async fn create_files(&self) -> Result<()> {
        // Generate settings JSON with defaults
        let settings = self.generate_settings()?;
        fs::write(&self.settings_path, settings)?;

        // Set Unix permissions (read for all)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.settings_path, fs::Permissions::from_mode(0o644))?;
        }

        tracing::info!(
            "Orchestrator temp files created at: {}",
            self.settings_path.display()
        );

        Ok(())
    }

    /// Write orchestrator settings from daemon config
    ///
    /// This is the preferred method for daemon startup as it includes
    /// the full hooks configuration from the daemon config.
    pub async fn write_orchestrator_settings(&self, daemon_config: &DaemonConfig) -> Result<()> {
        let settings = json!({
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339(),

            "daemon": {
                "host": daemon_config.host,
                "port": daemon_config.port,
                "version": env!("CARGO_PKG_VERSION"),
            },

            "orchestrator": {
                "enabled": true,
                "api_url": format!("http://{}:{}", daemon_config.host, daemon_config.port),
            },

            "hooks": {
                "PreToolUse": [],
                "PostToolUse": [],
                "Notification": [],
                "UserPromptSubmit": [],
                "SessionStart": [
                    json!({
                        "type": "http",
                        "url": format!("http://{}:{}/api/knowledge/session-start", daemon_config.host, daemon_config.port),
                        "method": "POST",
                        "timeout_ms": 5000,
                    })
                ],
                "SessionEnd": [],
                "Stop": [],
                "SubagentStart": [],
                "SubagentStop": [],
                "PreCompact": [
                    json!({
                        "type": "http",
                        "url": format!("http://{}:{}/api/knowledge/pre-compaction", daemon_config.host, daemon_config.port),
                        "method": "POST",
                        "timeout_ms": 10000,
                    })
                ],
                "PermissionRequest": []
            },

            "hooks_config": {
                "enabled": daemon_config.hooks.enabled,
                "timeout_ms": daemon_config.hooks.timeout_ms,
                "max_retries": daemon_config.hooks.max_retries,
                "llm": {
                    "model_type": daemon_config.hooks.llm.model_type,
                    "model_name": daemon_config.hooks.llm.model_name,
                    "model_path": daemon_config.hooks.llm.model_path.to_string_lossy(),
                    "model_size_mb": daemon_config.hooks.llm.model_size_mb,
                    "quantization": daemon_config.hooks.llm.quantization,
                    "loaded": daemon_config.hooks.llm.loaded,
                    "inference_timeout_ms": daemon_config.hooks.llm.inference_timeout_ms,
                    "temperature": daemon_config.hooks.llm.temperature,
                },
                "permissions": {
                    "allow_command_modification": daemon_config.hooks.permissions.allow_command_modification,
                    "allow_execution_blocking": daemon_config.hooks.permissions.allow_execution_blocking,
                    "allow_external_calls": daemon_config.hooks.permissions.allow_external_calls,
                    "allow_env_access": daemon_config.hooks.permissions.allow_env_access,
                    "allow_file_read": daemon_config.hooks.permissions.allow_file_read,
                    "allow_file_write": daemon_config.hooks.permissions.allow_file_write,
                }
            },

            "temp_dir": env::temp_dir().to_string_lossy()
        });

        let settings_json = serde_json::to_string_pretty(&settings)?;
        fs::write(&self.settings_path, settings_json)?;

        // Set Unix permissions (read for all)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.settings_path, fs::Permissions::from_mode(0o644))?;
        }

        tracing::info!(
            "Wrote orchestrator settings to: {}",
            self.settings_path.display()
        );

        Ok(())
    }

    /// Generate Claude-formatted agents JSON from orchestra config
    ///
    /// Parses orchestra-config.json and converts all 119 agents to Claude Code's format.
    /// Uses embedded prompts from JSON (falls back to role description if prompts not yet added).
    ///
    /// # Returns
    /// JSON bytes ready to be written to VFS or passed to Claude via --agents flag
    pub fn generate_agents_json(&self) -> Result<Vec<u8>> {
        use crate::orchestra::OrchestraConfig;

        // Load orchestra config from parent repo
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";
        let config = OrchestraConfig::load(config_path)?;

        // Convert to Claude format
        let claude_agents = config.to_claude_format()?;

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&claude_agents)?;

        tracing::info!(
            "Generated agents JSON: {} agents, {} bytes",
            claude_agents.len(),
            json.len()
        );

        Ok(json.into_bytes())
    }

    /// Clean up all temporary files
    pub async fn cleanup_files(&self) -> Result<()> {
        for path in [
            &self.settings_path,
            &self.system_prompt_path,
            &self.agents_json_path,
        ] {
            if path.exists() {
                fs::remove_file(path).ok();
            }
        }

        tracing::info!("Orchestrator temp files cleaned up");
        Ok(())
    }

    /// Get the settings file path
    pub fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }

    /// Get the knowledge database base directory
    ///
    /// Returns: ~/.cco/knowledge/
    pub fn knowledge_db_base(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join(".cco").join("knowledge")
    }

    /// Get the knowledge database path for a specific repository
    ///
    /// Returns: ~/.cco/knowledge/{repo_name}/
    pub fn knowledge_db_path(&self, repo_name: &str) -> PathBuf {
        self.knowledge_db_base().join(repo_name)
    }

    /// Clean up knowledge database for a specific repository
    pub async fn cleanup_knowledge_db(&self, repo_name: &str) -> Result<()> {
        let db_path = self.knowledge_db_path(repo_name);
        if db_path.exists() {
            tokio::fs::remove_dir_all(db_path).await?;
            tracing::info!("Cleaned up knowledge database for: {}", repo_name);
        }
        Ok(())
    }

    /// Generate system prompt with XOR deobfuscation
    ///
    /// Deobfuscates the embedded binary using XOR key 0xA7 and returns
    /// a JSON structure containing the prompt content.
    pub fn generate_system_prompt(&self) -> Result<Vec<u8>> {
        // Deobfuscate embedded binary
        const OBFUSCATED: &[u8] = include_bytes!("../../config/orchestrator-prompt.bin");
        const XOR_KEY: u8 = 0xA7;

        let content: Vec<u8> = OBFUSCATED.iter().map(|b| b ^ XOR_KEY).collect();

        let content_str = String::from_utf8(content)?;

        let data = json!({
            "version": env!("CARGO_PKG_VERSION"),
            "format": "sealed",
            "content": content_str,
        });
        Ok(serde_json::to_vec_pretty(&data)?)
    }

    /// Generate orchestrator settings JSON
    pub fn generate_settings(&self) -> Result<Vec<u8>> {
        // Get home directory for model path
        let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let model_path = home_dir.join(".cco/models/qwen2.5-coder-1.5b-instruct-q2_k.gguf");

        // Auto-discover daemon port (fallback to 3000 if not running)
        let daemon_port = super::read_daemon_port().unwrap_or(3000);
        let api_url = format!("http://localhost:{}", daemon_port);

        let settings = json!({
            "version": env!("CARGO_PKG_VERSION"),
            "orchestration": {
                "enabled": true,
                "api_url": api_url
            },
            "hooks": {
                "PreToolUse": [],
                "PostToolUse": [],
                "Notification": [],
                "UserPromptSubmit": [],
                "SessionStart": [],
                "SessionEnd": [],
                "Stop": [],
                "SubagentStart": [],
                "SubagentStop": [],
                "PreCompact": [],
                "PermissionRequest": []
            },

            "hooks_config": {
                "enabled": true,
                "timeout_ms": 5000,
                "max_retries": 2,
                "llm": {
                    "model_type": "qwen-coder",
                    "model_name": "qwen2.5-coder-1.5b-instruct-q2_k",
                    "model_path": model_path.to_string_lossy(),
                    "model_size_mb": 577,
                    "quantization": "Q2_K",
                    "loaded": false,
                    "inference_timeout_ms": 2000,
                    "temperature": 0.05
                },
                "permissions": {
                    "allow_command_modification": false,
                    "allow_execution_blocking": false,
                    "allow_external_calls": false,
                    "allow_env_access": false,
                    "allow_file_read": false,
                    "allow_file_write": false
                }
            },
            "temp_dir": env::temp_dir().to_string_lossy()
        });

        Ok(serde_json::to_vec_pretty(&settings)?)
    }
}

impl Default for TempFileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_temp_file_manager_creation() {
        let manager = TempFileManager::new();
        assert!(manager
            .settings_path()
            .to_string_lossy()
            .contains(".cco-orchestrator-settings"));
    }

    #[tokio::test]
    async fn test_create_and_cleanup_files() {
        let manager = TempFileManager::new();

        // Create files
        manager.create_files().await.unwrap();

        // Verify they exist
        assert!(manager.settings_path().exists());

        // Verify settings content is valid JSON
        let settings_content = fs::read_to_string(manager.settings_path()).unwrap();
        let _: serde_json::Value = serde_json::from_str(&settings_content).unwrap();

        // Cleanup
        manager.cleanup_files().await.unwrap();

        // Verify they're gone
        assert!(!manager.settings_path().exists());
    }

    #[tokio::test]
    async fn test_settings_json_structure() {
        let manager = TempFileManager::new();
        let settings = manager.generate_settings().unwrap();
        let json: serde_json::Value = serde_json::from_slice(&settings).unwrap();

        assert!(json.get("version").is_some());
        assert!(json.get("orchestration").is_some());
        assert!(json["orchestration"]["enabled"].as_bool().unwrap());
        assert!(json.get("hooks").is_some());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let manager = TempFileManager::new();
        manager.create_files().await.unwrap();

        let metadata = fs::metadata(manager.settings_path()).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o644);

        manager.cleanup_files().await.unwrap();
    }

    #[tokio::test]
    async fn test_orchestrator_settings_includes_hooks() {
        let config = DaemonConfig::default();
        let temp_manager = TempFileManager::new();

        // Write settings with daemon config
        temp_manager
            .write_orchestrator_settings(&config)
            .await
            .unwrap();

        // Read settings file
        let settings_content = fs::read_to_string(temp_manager.settings_path()).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&settings_content).unwrap();

        // Verify hooks section exists (new format with hook types)
        assert!(settings.get("hooks").is_some(), "hooks section missing");
        let hooks = settings["hooks"].as_object().unwrap();

        // Verify all hook types are present
        assert!(hooks.contains_key("PreToolUse"), "hooks.PreToolUse missing");
        assert!(
            hooks.contains_key("PostToolUse"),
            "hooks.PostToolUse missing"
        );
        assert!(
            hooks.contains_key("Notification"),
            "hooks.Notification missing"
        );
        assert!(
            hooks.contains_key("UserPromptSubmit"),
            "hooks.UserPromptSubmit missing"
        );
        assert!(
            hooks.contains_key("SessionStart"),
            "hooks.SessionStart missing"
        );
        assert!(hooks.contains_key("SessionEnd"), "hooks.SessionEnd missing");
        assert!(hooks.contains_key("Stop"), "hooks.Stop missing");
        assert!(
            hooks.contains_key("SubagentStart"),
            "hooks.SubagentStart missing"
        );
        assert!(
            hooks.contains_key("SubagentStop"),
            "hooks.SubagentStop missing"
        );
        assert!(hooks.contains_key("PreCompact"), "hooks.PreCompact missing");
        assert!(
            hooks.contains_key("PermissionRequest"),
            "hooks.PermissionRequest missing"
        );

        // Verify hooks_config section exists
        assert!(
            settings.get("hooks_config").is_some(),
            "hooks_config section missing"
        );
        let hooks_config = settings["hooks_config"].as_object().unwrap();

        // Verify hooks configuration fields
        assert!(
            hooks_config.contains_key("enabled"),
            "hooks_config.enabled missing"
        );
        assert!(
            hooks_config.contains_key("timeout_ms"),
            "hooks_config.timeout_ms missing"
        );
        assert!(
            hooks_config.contains_key("max_retries"),
            "hooks_config.max_retries missing"
        );
        assert!(hooks_config.contains_key("llm"), "hooks_config.llm missing");
        assert!(
            hooks_config.contains_key("permissions"),
            "hooks_config.permissions missing"
        );

        // Verify LLM configuration
        let llm = hooks_config["llm"].as_object().unwrap();
        assert!(llm.contains_key("model_type"), "llm.model_type missing");
        assert!(llm.contains_key("model_name"), "llm.model_name missing");
        assert!(llm.contains_key("model_path"), "llm.model_path missing");
        assert!(
            llm.contains_key("model_size_mb"),
            "llm.model_size_mb missing"
        );
        assert!(llm.contains_key("quantization"), "llm.quantization missing");
        assert!(llm.contains_key("loaded"), "llm.loaded missing");
        assert!(
            llm.contains_key("inference_timeout_ms"),
            "llm.inference_timeout_ms missing"
        );
        assert!(llm.contains_key("temperature"), "llm.temperature missing");

        // Verify permissions configuration
        let perms = hooks_config["permissions"].as_object().unwrap();
        assert!(
            perms.contains_key("allow_command_modification"),
            "permissions.allow_command_modification missing"
        );
        assert!(
            perms.contains_key("allow_execution_blocking"),
            "permissions.allow_execution_blocking missing"
        );
        assert!(
            perms.contains_key("allow_external_calls"),
            "permissions.allow_external_calls missing"
        );
        assert!(
            perms.contains_key("allow_env_access"),
            "permissions.allow_env_access missing"
        );
        assert!(
            perms.contains_key("allow_file_read"),
            "permissions.allow_file_read missing"
        );
        assert!(
            perms.contains_key("allow_file_write"),
            "permissions.allow_file_write missing"
        );

        // Verify daemon section
        assert!(settings.get("daemon").is_some(), "daemon section missing");
        assert!(
            settings["daemon"].get("host").is_some(),
            "daemon.host missing"
        );
        assert!(
            settings["daemon"].get("port").is_some(),
            "daemon.port missing"
        );
        assert!(
            settings["daemon"].get("version").is_some(),
            "daemon.version missing"
        );

        // Verify orchestrator section
        assert!(
            settings.get("orchestrator").is_some(),
            "orchestrator section missing"
        );
        assert!(
            settings["orchestrator"].get("enabled").is_some(),
            "orchestrator.enabled missing"
        );
        assert!(
            settings["orchestrator"].get("api_url").is_some(),
            "orchestrator.api_url missing"
        );

        // Cleanup
        temp_manager.cleanup_files().await.unwrap();
    }

    #[tokio::test]
    async fn test_orchestrator_settings_with_custom_hooks_config() {
        let mut config = DaemonConfig::default();

        // Customize hooks configuration
        config.hooks.enabled = true;
        config.hooks.timeout_ms = 7500;
        config.hooks.max_retries = 5;
        config.hooks.llm.temperature = 0.2;
        config.hooks.permissions.allow_file_read = true;

        let temp_manager = TempFileManager::new();
        temp_manager
            .write_orchestrator_settings(&config)
            .await
            .unwrap();

        // Read and verify
        let settings_content = fs::read_to_string(temp_manager.settings_path()).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&settings_content).unwrap();

        // Verify customized values in hooks_config (not hooks)
        assert_eq!(settings["hooks_config"]["enabled"].as_bool().unwrap(), true);
        assert_eq!(
            settings["hooks_config"]["timeout_ms"].as_u64().unwrap(),
            7500
        );
        assert_eq!(settings["hooks_config"]["max_retries"].as_u64().unwrap(), 5);
        // Use approximate comparison for floating point
        let temp = settings["hooks_config"]["llm"]["temperature"]
            .as_f64()
            .unwrap();
        assert!(
            (temp - 0.2).abs() < 0.01,
            "Temperature should be approximately 0.2, got {}",
            temp
        );
        assert_eq!(
            settings["hooks_config"]["permissions"]["allow_file_read"]
                .as_bool()
                .unwrap(),
            true
        );

        // Cleanup
        temp_manager.cleanup_files().await.unwrap();
    }
}
