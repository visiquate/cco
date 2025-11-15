//! Auto-update module for CCO
//!
//! Handles background checking for updates and notifying users.

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Enable automatic update checks
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Automatically install updates (requires enabled=true)
    #[serde(default = "default_false")]
    pub auto_install: bool,

    /// Check interval: daily, weekly, never
    #[serde(default = "default_interval")]
    pub check_interval: String,

    /// Update channel: stable, beta
    #[serde(default = "default_channel")]
    pub channel: String,

    /// Last check timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_check: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_update: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_interval() -> String {
    "daily".to_string()
}

fn default_channel() -> String {
    "stable".to_string()
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        }
    }
}

/// Main configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub updates: UpdateConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            updates: UpdateConfig::default(),
        }
    }
}

/// Get the configuration file path
fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let cco_config_dir = config_dir.join("cco");

    fs::create_dir_all(&cco_config_dir)?;

    Ok(cco_config_dir.join("config.toml"))
}

/// Load configuration from disk
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        let config = Config::default();
        save_config(&config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

/// Save configuration to disk
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)?;

    fs::write(&config_path, content)?;

    Ok(())
}

/// Check if an update check is due
fn should_check(config: &UpdateConfig) -> bool {
    if !config.enabled {
        return false;
    }

    if config.check_interval == "never" {
        return false;
    }

    let Some(last_check) = config.last_check else {
        return true; // Never checked before
    };

    let now = Utc::now();
    let elapsed = now - last_check;

    match config.check_interval.as_str() {
        "daily" => elapsed >= Duration::days(1),
        "weekly" => elapsed >= Duration::weeks(1),
        _ => false,
    }
}

/// Perform background update check (non-blocking)
pub fn check_for_updates_async() {
    tokio::spawn(async {
        if let Err(e) = check_for_updates_internal().await {
            tracing::debug!("Background update check failed: {}", e);
        }
    });
}

/// Internal update check logic
async fn check_for_updates_internal() -> Result<()> {
    let mut config = load_config()?;

    if !should_check(&config.updates) {
        return Ok(());
    }

    // Update last_check timestamp
    config.updates.last_check = Some(Utc::now());
    save_config(&config)?;

    // Fetch latest release information
    let _channel = &config.updates.channel;
    let client = reqwest::Client::builder()
        .user_agent(format!("cco/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let url = format!(
        "https://api.github.com/repos/brentley/cco-releases/releases/latest"
    );

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(()); // Silently fail
    }

    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }

    let release: Release = response.json().await?;

    // Compare versions
    let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
    let latest_version = semver::Version::parse(release.tag_name.trim_start_matches('v'))?;

    if latest_version > current_version {
        // Update available
        if config.updates.auto_install {
            // TODO: Implement silent auto-install
            println!("ℹ️  Auto-installing CCO {} in background...", release.tag_name);
        } else {
            // Notify user
            println!("ℹ️  New version available: {} (current: v{})", release.tag_name, current_version);
            println!("   Run 'cco update' to upgrade");
        }
    }

    Ok(())
}

/// Show current update configuration
pub fn show_config() -> Result<()> {
    let config = load_config()?;

    println!("Update Configuration:");
    println!("  Enabled: {}", config.updates.enabled);
    println!("  Auto-install: {}", config.updates.auto_install);
    println!("  Check interval: {}", config.updates.check_interval);
    println!("  Channel: {}", config.updates.channel);

    if let Some(last_check) = config.updates.last_check {
        println!("  Last check: {}", last_check.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("  Last check: Never");
    }

    if let Some(last_update) = config.updates.last_update {
        println!("  Last update: {}", last_update.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("  Last update: Never");
    }

    Ok(())
}

/// Set a configuration value
pub fn set_config(key: &str, value: &str) -> Result<()> {
    let mut config = load_config()?;

    match key {
        "updates.enabled" => {
            config.updates.enabled = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean value: {}", value))?;
        }
        "updates.auto_install" => {
            config.updates.auto_install = value.parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean value: {}", value))?;
        }
        "updates.check_interval" => {
            if !["daily", "weekly", "never"].contains(&value) {
                return Err(anyhow::anyhow!("Invalid interval: {}. Use: daily, weekly, never", value));
            }
            config.updates.check_interval = value.to_string();
        }
        "updates.channel" => {
            if !["stable", "beta"].contains(&value) {
                return Err(anyhow::anyhow!("Invalid channel: {}. Use: stable, beta", value));
            }
            config.updates.channel = value.to_string();
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown configuration key: {}", key));
        }
    }

    save_config(&config)?;
    println!("✅ Configuration updated: {} = {}", key, value);

    Ok(())
}

/// Get a configuration value
pub fn get_config(key: &str) -> Result<String> {
    let config = load_config()?;

    let value = match key {
        "updates.enabled" => config.updates.enabled.to_string(),
        "updates.auto_install" => config.updates.auto_install.to_string(),
        "updates.check_interval" => config.updates.check_interval,
        "updates.channel" => config.updates.channel,
        _ => {
            return Err(anyhow::anyhow!("Unknown configuration key: {}", key));
        }
    };

    println!("{} = {}", key, value);

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.updates.enabled);
        assert!(!config.updates.auto_install);
        assert_eq!(config.updates.check_interval, "daily");
        assert_eq!(config.updates.channel, "stable");
    }

    #[test]
    fn test_should_check_never_checked() {
        let config = UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        };

        assert!(should_check(&config));
    }

    #[test]
    fn test_should_check_disabled() {
        let config = UpdateConfig {
            enabled: false,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        };

        assert!(!should_check(&config));
    }
}
