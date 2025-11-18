//! Temporary file management for orchestrator settings
//!
//! Creates and manages temporary files in the system temp directory for:
//! - Orchestrator settings (JSON)
//! - Agent definitions (sealed/encrypted)
//! - Orchestration rules (sealed/encrypted)
//! - Coordination hooks (sealed/encrypted)
//!
//! Files are created on daemon start and cleaned up on daemon stop.

use anyhow::Result;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Temp file manager for orchestrator resources
pub struct TempFileManager {
    settings_path: PathBuf,
    agents_path: PathBuf,
    rules_path: PathBuf,
    hooks_path: PathBuf,
}

impl TempFileManager {
    /// Create a new temp file manager
    pub fn new() -> Self {
        let temp_dir = env::temp_dir();
        Self {
            settings_path: temp_dir.join(".cco-orchestrator-settings"),
            agents_path: temp_dir.join(".cco-agents-sealed"),
            rules_path: temp_dir.join(".cco-rules-sealed"),
            hooks_path: temp_dir.join(".cco-hooks-sealed"),
        }
    }

    /// Create all temporary files with content
    pub async fn create_files(&self) -> Result<()> {
        // Generate settings JSON
        let settings = self.generate_settings()?;
        fs::write(&self.settings_path, settings)?;

        // Generate encrypted agents (placeholder for now)
        let agents = self.generate_agents()?;
        fs::write(&self.agents_path, agents)?;

        // Generate encrypted rules (placeholder for now)
        let rules = self.generate_rules()?;
        fs::write(&self.rules_path, rules)?;

        // Generate encrypted hooks (placeholder for now)
        let hooks = self.generate_hooks()?;
        fs::write(&self.hooks_path, hooks)?;

        // Set Unix permissions (read for all)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in [
                &self.settings_path,
                &self.agents_path,
                &self.rules_path,
                &self.hooks_path,
            ] {
                fs::set_permissions(path, fs::Permissions::from_mode(0o644))?;
            }
        }

        tracing::info!(
            "Orchestrator temp files created at: {}",
            self.settings_path.display()
        );

        Ok(())
    }

    /// Clean up all temporary files
    pub async fn cleanup_files(&self) -> Result<()> {
        for path in [
            &self.settings_path,
            &self.agents_path,
            &self.rules_path,
            &self.hooks_path,
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

    /// Get the agents file path
    pub fn agents_path(&self) -> &PathBuf {
        &self.agents_path
    }

    /// Get the rules file path
    pub fn rules_path(&self) -> &PathBuf {
        &self.rules_path
    }

    /// Get the hooks file path
    pub fn hooks_path(&self) -> &PathBuf {
        &self.hooks_path
    }

    /// Generate orchestrator settings JSON
    fn generate_settings(&self) -> Result<Vec<u8>> {
        let settings = json!({
            "version": env!("CARGO_PKG_VERSION"),
            "orchestration": {
                "enabled": true,
                "api_url": "http://localhost:3000"
            },
            "agents": {
                "sealed_file": self.agents_path.to_string_lossy()
            },
            "rules": {
                "sealed_file": self.rules_path.to_string_lossy()
            },
            "hooks": {
                "sealed_file": self.hooks_path.to_string_lossy()
            },
            "temp_dir": env::temp_dir().to_string_lossy()
        });

        Ok(serde_json::to_vec_pretty(&settings)?)
    }

    /// Generate agent definitions (placeholder - will be encrypted in Phase 2)
    fn generate_agents(&self) -> Result<Vec<u8>> {
        // Phase 1: Return plaintext JSON stub
        // Phase 2+: Use encryption pipeline with SBF v1 format
        let agents = json!({
            "version": "2025.11.17",
            "format": "plaintext",
            "agents": [
                {
                    "name": "Chief Architect",
                    "type": "system-architect",
                    "model": "opus-4.1",
                    "role": "Strategic decision-making and coordination"
                },
                {
                    "name": "Python Specialist",
                    "type": "python-specialist",
                    "model": "haiku-4.5",
                    "role": "Python/FastAPI/Django development"
                }
            ],
            "note": "Phase 1: Plaintext stub - will be encrypted in Phase 2"
        });

        Ok(serde_json::to_vec_pretty(&agents)?)
    }

    /// Generate orchestration rules (placeholder - will be encrypted in Phase 2)
    fn generate_rules(&self) -> Result<Vec<u8>> {
        // Phase 1: Return plaintext JSON stub
        // Phase 2+: Use encryption pipeline
        let rules = json!({
            "version": "2025.11.17",
            "format": "plaintext",
            "rules": {
                "always_read_files_fully": true,
                "spawn_all_agents_in_parallel": true,
                "use_knowledge_manager": true,
                "tdd_first": true
            },
            "note": "Phase 1: Stub rules - will be encrypted in Phase 2"
        });

        Ok(serde_json::to_vec_pretty(&rules)?)
    }

    /// Generate coordination hooks (placeholder - will be encrypted in Phase 2)
    fn generate_hooks(&self) -> Result<Vec<u8>> {
        // Phase 1: Return plaintext JSON stub
        // Phase 2+: Use encryption pipeline
        let hooks = json!({
            "version": "2025.11.17",
            "format": "plaintext",
            "pre_compaction": {
                "enabled": true,
                "script": "knowledge-manager.js export"
            },
            "post_compaction": {
                "enabled": true,
                "script": "knowledge-manager.js restore"
            },
            "note": "Phase 1: Stub hooks - will be encrypted in Phase 2"
        });

        Ok(serde_json::to_vec_pretty(&hooks)?)
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
        assert!(manager.settings_path().to_string_lossy().contains(".cco-orchestrator-settings"));
        assert!(manager.agents_path().to_string_lossy().contains(".cco-agents-sealed"));
    }

    #[tokio::test]
    async fn test_create_and_cleanup_files() {
        let manager = TempFileManager::new();

        // Create files
        manager.create_files().await.unwrap();

        // Verify they exist
        assert!(manager.settings_path().exists());
        assert!(manager.agents_path().exists());
        assert!(manager.rules_path().exists());
        assert!(manager.hooks_path().exists());

        // Verify settings content is valid JSON
        let settings_content = fs::read_to_string(manager.settings_path()).unwrap();
        let _: serde_json::Value = serde_json::from_str(&settings_content).unwrap();

        // Cleanup
        manager.cleanup_files().await.unwrap();

        // Verify they're gone
        assert!(!manager.settings_path().exists());
        assert!(!manager.agents_path().exists());
        assert!(!manager.rules_path().exists());
        assert!(!manager.hooks_path().exists());
    }

    #[tokio::test]
    async fn test_settings_json_structure() {
        let manager = TempFileManager::new();
        let settings = manager.generate_settings().unwrap();
        let json: serde_json::Value = serde_json::from_slice(&settings).unwrap();

        assert!(json.get("version").is_some());
        assert!(json.get("orchestration").is_some());
        assert!(json["orchestration"]["enabled"].as_bool().unwrap());
        assert!(json.get("agents").is_some());
        assert!(json.get("rules").is_some());
        assert!(json.get("hooks").is_some());
    }

    #[tokio::test]
    async fn test_agents_json_structure() {
        let manager = TempFileManager::new();
        let agents = manager.generate_agents().unwrap();
        let json: serde_json::Value = serde_json::from_slice(&agents).unwrap();

        assert_eq!(json["format"], "plaintext");
        assert!(json.get("agents").is_some());
        assert!(json["agents"].is_array());
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
}
