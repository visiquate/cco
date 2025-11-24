//! Daemon configuration management
//!
//! Handles loading, saving, and validation of daemon configuration from TOML files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::hooks::HooksConfig;

/// Daemon configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Port to listen on (default: 3000)
    pub port: u16,

    /// Host to bind to (default: 127.0.0.1)
    pub host: String,

    /// Log level (debug, info, warn, error)
    pub log_level: String,

    /// Log file rotation size in bytes (default: 10MB)
    pub log_rotation_size: u64,

    /// Maximum number of rotated log files to keep (default: 5)
    pub log_max_files: u32,

    /// Database URL for persistence
    pub database_url: String,

    /// Cache size in bytes (default: 1GB)
    pub cache_size: u64,

    /// Cache TTL in seconds (default: 3600)
    pub cache_ttl: u64,

    /// Auto-start on system boot
    pub auto_start: bool,

    /// Enable health checks
    pub health_checks: bool,

    /// Health check interval in seconds
    pub health_check_interval: u64,

    /// Hooks system configuration
    #[serde(default)]
    pub hooks: HooksConfig,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            log_level: "info".to_string(),
            log_rotation_size: 10 * 1024 * 1024, // 10MB
            log_max_files: 5,
            database_url: "sqlite://analytics.db".to_string(),
            cache_size: 1024 * 1024 * 1024, // 1GB
            cache_ttl: 3600, // 1 hour
            auto_start: true,
            health_checks: true,
            health_check_interval: 30,
            hooks: HooksConfig::default(),
        }
    }
}

impl DaemonConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            anyhow::bail!("Invalid port: {} (must be 1-65535)", self.port);
        }

        if self.log_max_files == 0 {
            anyhow::bail!("log_max_files must be at least 1");
        }

        if self.cache_size == 0 {
            anyhow::bail!("cache_size must be greater than 0");
        }

        if self.cache_ttl == 0 {
            anyhow::bail!("cache_ttl must be greater than 0");
        }

        match self.log_level.as_str() {
            "debug" | "info" | "warn" | "error" => {}
            level => anyhow::bail!("Invalid log level: {} (must be debug, info, warn, or error)", level),
        }

        // Validate hooks configuration
        self.hooks.validate()
            .map_err(|e| anyhow::anyhow!("Hooks configuration error: {}", e))?;

        Ok(())
    }

    /// Load configuration from file, with defaults for missing values
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            // If file doesn't exist, return defaults
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(path)
            .context("Failed to read configuration file")?;

        let config: Self = toml::from_str(&contents)
            .context("Failed to parse configuration file")?;

        config.validate()?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        self.validate()?;

        // Create directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create configuration directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        std::fs::write(path, contents)
            .context("Failed to write configuration file")?;

        Ok(())
    }

    /// Set a configuration value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "port" => self.port = value.parse().context("Failed to parse port")?,
            "host" => self.host = value.to_string(),
            "log_level" => self.log_level = value.to_string(),
            "log_rotation_size" => {
                self.log_rotation_size = value.parse().context("Failed to parse log_rotation_size")?;
            }
            "log_max_files" => {
                self.log_max_files = value.parse().context("Failed to parse log_max_files")?;
            }
            "database_url" => self.database_url = value.to_string(),
            "cache_size" => {
                self.cache_size = value.parse().context("Failed to parse cache_size")?;
            }
            "cache_ttl" => {
                self.cache_ttl = value.parse().context("Failed to parse cache_ttl")?;
            }
            "auto_start" => {
                self.auto_start = value.parse().context("Failed to parse auto_start")?;
            }
            "health_checks" => {
                self.health_checks = value.parse().context("Failed to parse health_checks")?;
            }
            "health_check_interval" => {
                self.health_check_interval = value.parse().context("Failed to parse health_check_interval")?;
            }
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }

        self.validate()?;
        Ok(())
    }

    /// Get a configuration value by key
    pub fn get(&self, key: &str) -> Result<String> {
        match key {
            "port" => Ok(self.port.to_string()),
            "host" => Ok(self.host.clone()),
            "log_level" => Ok(self.log_level.clone()),
            "log_rotation_size" => Ok(self.log_rotation_size.to_string()),
            "log_max_files" => Ok(self.log_max_files.to_string()),
            "database_url" => Ok(self.database_url.clone()),
            "cache_size" => Ok(self.cache_size.to_string()),
            "cache_ttl" => Ok(self.cache_ttl.to_string()),
            "auto_start" => Ok(self.auto_start.to_string()),
            "health_checks" => Ok(self.health_checks.to_string()),
            "health_check_interval" => Ok(self.health_check_interval.to_string()),
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
    }
}

/// Load configuration from the default location
pub fn load_config() -> Result<DaemonConfig> {
    let config_path = super::get_daemon_config_file()?;
    DaemonConfig::load(&config_path)
}

/// Save configuration to the default location
pub fn save_config(config: &DaemonConfig) -> Result<()> {
    let config_path = super::get_daemon_config_file()?;
    config.save(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = DaemonConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.log_level, "info");
        assert!(config.auto_start);
    }

    #[test]
    fn test_config_validation() {
        let mut config = DaemonConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 3000;
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());

        config.log_level = "info".to_string();
        config.log_max_files = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = DaemonConfig::default();
        config.port = 8080;
        config.log_level = "debug".to_string();
        config.auto_start = false;

        config.save(&config_path).unwrap();
        assert!(config_path.exists());

        let loaded = DaemonConfig::load(&config_path).unwrap();
        assert_eq!(loaded.port, 8080);
        assert_eq!(loaded.log_level, "debug");
        assert!(!loaded.auto_start);
    }

    #[test]
    fn test_config_set_values() {
        let mut config = DaemonConfig::default();

        config.set("port", "8080").unwrap();
        assert_eq!(config.port, 8080);

        config.set("host", "0.0.0.0").unwrap();
        assert_eq!(config.host, "0.0.0.0");

        config.set("log_level", "debug").unwrap();
        assert_eq!(config.log_level, "debug");

        assert!(config.set("invalid_key", "value").is_err());
    }

    #[test]
    fn test_config_get_values() {
        let config = DaemonConfig::default();

        assert_eq!(config.get("port").unwrap(), "3000");
        assert_eq!(config.get("host").unwrap(), "127.0.0.1");
        assert_eq!(config.get("log_level").unwrap(), "info");

        assert!(config.get("invalid_key").is_err());
    }

    #[test]
    fn test_config_missing_file_returns_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let config = DaemonConfig::load(&config_path).unwrap();
        assert_eq!(config.port, 3000);
    }
}
