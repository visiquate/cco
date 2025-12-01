//! Auto-update module for CCO
//!
//! Provides automatic update checking and installation capabilities.
//!
//! # Architecture
//!
//! - `github.rs`: GitHub API integration for fetching release information
//! - `updater.rs`: Core update orchestration and binary replacement
//! - `mod.rs`: Configuration and high-level API
//!
//! # Usage
//!
//! ```rust
//! // Check for updates manually
//! let manager = AutoUpdateManager::new(config);
//! if let Some(release) = manager.check_for_updates().await? {
//!     println!("Update available: {}", release.version);
//! }
//!
//! // Perform update
//! manager.perform_update(true).await?;
//! ```

pub mod github; // Legacy - kept for backwards compatibility
pub mod releases_api;
pub mod updater;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::version::DateVersion;

/// Get the log directory for updates
fn get_log_dir() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let log_dir = home.join(".cco").join("logs");
    fs::create_dir_all(&log_dir)?;
    Ok(log_dir)
}

/// Get the updates log file path
fn get_updates_log_file() -> Result<PathBuf> {
    let log_dir = get_log_dir()?;
    Ok(log_dir.join("updates.log"))
}

/// Log an update event to the updates log file
fn log_update_event(message: &str) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_message = format!("[{}] {}\n", timestamp, message);

    if let Ok(log_file) = get_updates_log_file() {
        // Rotate log if it's too large (>10MB)
        if let Ok(metadata) = fs::metadata(&log_file) {
            if metadata.len() > 10 * 1024 * 1024 {
                let _ = rotate_log_file(&log_file);
            }
        }

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
        {
            let _ = file.write_all(log_message.as_bytes());
        }
    }

    // Also log to tracing
    tracing::info!("{}", message);
}

/// Rotate log file (keep last 30 days)
fn rotate_log_file(log_file: &Path) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let rotated_name = format!(
        "{}.{}",
        log_file.file_name().unwrap().to_string_lossy(),
        timestamp
    );
    let rotated_path = log_file.with_file_name(rotated_name);

    // Move current log to rotated file
    fs::rename(log_file, &rotated_path)?;

    // Clean up old logs (keep last 30 days)
    if let Ok(log_dir) = get_log_dir() {
        let cutoff = Utc::now() - Duration::days(30);
        if let Ok(entries) = fs::read_dir(log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("updates.log.") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                let modified_datetime: DateTime<Utc> = modified.into();
                                if modified_datetime < cutoff {
                                    let _ = fs::remove_file(&path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get the path to the updates log file (exported for external use)
pub fn get_log_file_path() -> Result<PathBuf> {
    get_updates_log_file()
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Enable automatic update checks
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Automatically install updates (requires enabled=true)
    #[serde(default = "default_true")]
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
            auto_install: true, // Changed: Auto-install enabled by default
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: None,
            last_update: None,
        }
    }
}

impl UpdateConfig {
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        // CCO_AUTO_UPDATE=false - Disable auto-updates
        if let Ok(val) = std::env::var("CCO_AUTO_UPDATE") {
            if let Ok(enabled) = val.parse::<bool>() {
                self.enabled = enabled;
                self.auto_install = enabled;
            }
        }

        // CCO_AUTO_UPDATE_CHANNEL=beta - Override channel
        if let Ok(channel) = std::env::var("CCO_AUTO_UPDATE_CHANNEL") {
            if ["stable", "beta"].contains(&channel.as_str()) {
                self.channel = channel;
            }
        }

        // CCO_AUTO_UPDATE_INTERVAL=weekly - Override check interval
        if let Ok(interval) = std::env::var("CCO_AUTO_UPDATE_INTERVAL") {
            if ["daily", "weekly", "never"].contains(&interval.as_str()) {
                self.check_interval = interval;
            }
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

/// Auto-update manager
pub struct AutoUpdateManager {
    config_path: PathBuf,
    config: Config,
}

impl AutoUpdateManager {
    /// Create a new auto-update manager
    pub fn new() -> Result<Self> {
        let config_path = get_config_path()?;
        let mut config = load_config_from_path(&config_path)?;

        // Apply environment variable overrides
        config.updates.apply_env_overrides();

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let config_path = get_config_path()?;

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Check if an update check is due
    pub fn should_check(&self) -> bool {
        should_check(&self.config.updates)
    }

    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Update configuration
    pub fn update_config<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        f(&mut self.config);
        save_config_to_path(&self.config_path, &self.config)
    }

    /// Check for updates (returns release info if update available)
    pub async fn check_for_updates(&mut self) -> Result<Option<releases_api::ReleaseInfo>> {
        if !self.config.updates.enabled {
            log_update_event("Update checks are disabled");
            return Ok(None);
        }

        log_update_event(&format!(
            "Checking for updates (channel: {})",
            self.config.updates.channel
        ));

        // Update last check timestamp
        self.update_config(|config| {
            config.updates.last_check = Some(Utc::now());
        })?;

        // Fetch latest release from authenticated API
        let release = match releases_api::fetch_latest_release(&self.config.updates.channel).await {
            Ok(r) => r,
            Err(e) => {
                // Check if it's an authentication error
                let err_msg = format!("{}", e);
                if err_msg.contains("Not authenticated")
                    || err_msg.contains("Please run 'cco login'")
                {
                    log_update_event("Update check requires authentication");
                    println!("\n⚠️  Update check requires authentication.");
                    println!("   Please run 'cco login' to access updates.");
                    return Ok(None);
                }
                log_update_event(&format!("Failed to check for updates: {}", e));
                return Err(e);
            }
        };

        // Compare with current version
        let current = DateVersion::parse(DateVersion::current())?;
        let latest = DateVersion::parse(&release.version)?;

        if latest > current {
            log_update_event(&format!("Update available: {} -> {}", current, latest));
            Ok(Some(release))
        } else {
            log_update_event(&format!("No updates available (current: {})", current));
            Ok(None)
        }
    }

    /// Download and verify a new binary
    pub async fn download_binary(&self, release: &releases_api::ReleaseInfo) -> Result<PathBuf> {
        updater::download_and_verify(release).await
    }

    /// Replace current binary with new one (atomic operation)
    pub async fn replace_binary(&mut self, new_binary_path: &Path) -> Result<()> {
        let result = updater::replace_binary(new_binary_path).await;

        if result.is_ok() {
            // Update last_update timestamp
            self.update_config(|config| {
                config.updates.last_update = Some(Utc::now());
            })?;
        }

        result
    }

    /// Perform complete update (check → download → verify → replace)
    pub async fn perform_update(&mut self, auto_confirm: bool) -> Result<()> {
        log_update_event("Starting update process");

        // Check for updates
        let release = match self.check_for_updates().await? {
            Some(r) => r,
            None => {
                log_update_event("No updates available");
                return Ok(());
            }
        };

        log_update_event(&format!(
            "Update available: {} -> {}",
            DateVersion::current(),
            release.version
        ));

        // Show release notes if available
        if !release.release_notes.is_empty() && !auto_confirm {
            println!("\nWhat's new in {}:", release.version);
            for (i, line) in release.release_notes.lines().take(10).enumerate() {
                if i == 0 && line.starts_with('#') {
                    continue; // Skip title
                }
                println!("  {}", line);
            }
            if release.release_notes.lines().count() > 10 {
                println!("  ... (see full release notes in GitHub)");
            }
        }

        // Confirm if not auto-confirming
        if !auto_confirm {
            print!("\nUpdate now? [Y/n]: ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if !input.is_empty() && input != "y" && input != "yes" {
                log_update_event("Update cancelled by user");
                println!("Update cancelled");
                return Ok(());
            }
        }

        // Download and verify
        log_update_event(&format!("Downloading CCO {}...", release.version));
        println!("→ Downloading CCO {}...", release.version);
        let binary_path = match self.download_binary(&release).await {
            Ok(path) => {
                log_update_event("Download completed successfully");
                path
            }
            Err(e) => {
                log_update_event(&format!("Download failed: {}", e));
                return Err(e).context("Failed to download update");
            }
        };

        // Replace binary
        log_update_event("Installing update...");
        println!("→ Installing update...");
        match self.replace_binary(&binary_path).await {
            Ok(_) => {
                log_update_event(&format!("Successfully updated to {}", release.version));
                println!("✅ Successfully updated to {}", release.version);
                println!("\nRestart CCO to use the new version.");
                Ok(())
            }
            Err(e) => {
                log_update_event(&format!("Installation failed: {}", e));
                Err(e).context("Failed to install update")
            }
        }
    }

    /// Verify current binary works
    pub fn verify_binary() -> Result<bool> {
        updater::verify_current_binary()
    }
}

impl Default for AutoUpdateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create AutoUpdateManager")
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

/// Load configuration from a specific path
fn load_config_from_path(config_path: &Path) -> Result<Config> {
    if !config_path.exists() {
        let config = Config::default();
        save_config_to_path(config_path, &config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

/// Save configuration to a specific path
fn save_config_to_path(config_path: &Path, config: &Config) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

/// Load configuration from disk (legacy API)
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    load_config_from_path(&config_path)
}

/// Save configuration to disk (legacy API)
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    save_config_to_path(&config_path, config)
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

/// Perform synchronous update check (blocks startup)
/// This ensures we check for updates BEFORE launching the application
pub async fn check_for_updates_blocking() {
    // Silently check and install updates if enabled
    // This respects all config settings (enabled, auto_install, check_interval)
    if let Err(e) = check_for_updates_internal().await {
        tracing::debug!("Update check failed: {}", e);
        // Don't block startup on update check failure
    }
}

/// Internal update check logic
async fn check_for_updates_internal() -> Result<()> {
    let mut manager = AutoUpdateManager::new()?;

    if !manager.should_check() {
        return Ok(());
    }

    // Check for updates
    match manager.check_for_updates().await? {
        Some(release) => {
            if manager.config.updates.auto_install {
                // Auto-install in background
                log_update_event(&format!(
                    "Auto-installing CCO {} in background...",
                    release.version
                ));
                println!(
                    "ℹ️  Auto-installing CCO {} in background...",
                    release.version
                );

                match manager.perform_update(true).await {
                    Ok(_) => {
                        log_update_event("Auto-update completed successfully");
                        println!("✅ Auto-update completed. Restart CCO to use the new version.");
                    }
                    Err(e) => {
                        log_update_event(&format!("Auto-update failed: {}", e));
                        tracing::error!("Auto-update failed: {}", e);
                        println!(
                            "⚠️  Auto-update failed: {}. Run 'cco update' to try again.",
                            e
                        );
                    }
                }
            } else {
                // Notify user
                log_update_event(&format!(
                    "New version available: {} (current: {}). User confirmation required.",
                    release.version,
                    DateVersion::current()
                ));
                println!(
                    "ℹ️  New version available: {} (current: {})",
                    release.version,
                    DateVersion::current()
                );
                println!("   Run 'cco update' to upgrade");
            }
        }
        None => {
            // No updates available - already logged in check_for_updates
        }
    }

    Ok(())
}

/// Show current update configuration
pub fn show_config() -> Result<()> {
    let mut config = load_config()?;

    println!("\nUpdate Configuration:");
    println!("  Enabled: {}", config.updates.enabled);
    println!("  Auto-install: {}", config.updates.auto_install);
    println!("  Check interval: {}", config.updates.check_interval);
    println!("  Channel: {}", config.updates.channel);

    if let Some(last_check) = config.updates.last_check {
        println!(
            "  Last check: {}",
            last_check.format("%Y-%m-%d %H:%M:%S UTC")
        );
    } else {
        println!("  Last check: Never");
    }

    if let Some(last_update) = config.updates.last_update {
        println!(
            "  Last update: {}",
            last_update.format("%Y-%m-%d %H:%M:%S UTC")
        );
    } else {
        println!("  Last update: Never");
    }

    // Show environment variable overrides if any
    let has_env_overrides = std::env::var("CCO_AUTO_UPDATE").is_ok()
        || std::env::var("CCO_AUTO_UPDATE_CHANNEL").is_ok()
        || std::env::var("CCO_AUTO_UPDATE_INTERVAL").is_ok();

    if has_env_overrides {
        println!("\nEnvironment Variable Overrides:");
        if let Ok(val) = std::env::var("CCO_AUTO_UPDATE") {
            println!("  CCO_AUTO_UPDATE: {}", val);
        }
        if let Ok(val) = std::env::var("CCO_AUTO_UPDATE_CHANNEL") {
            println!("  CCO_AUTO_UPDATE_CHANNEL: {}", val);
        }
        if let Ok(val) = std::env::var("CCO_AUTO_UPDATE_INTERVAL") {
            println!("  CCO_AUTO_UPDATE_INTERVAL: {}", val);
        }

        // Show effective config after overrides
        config.updates.apply_env_overrides();
        println!("\nEffective Configuration (with overrides):");
        println!("  Enabled: {}", config.updates.enabled);
        println!("  Auto-install: {}", config.updates.auto_install);
        println!("  Check interval: {}", config.updates.check_interval);
        println!("  Channel: {}", config.updates.channel);
    }

    // Show log file location
    if let Ok(log_file) = get_updates_log_file() {
        println!("\nUpdate Log:");
        println!("  Location: {}", log_file.display());
        if log_file.exists() {
            if let Ok(metadata) = fs::metadata(&log_file) {
                let size_kb = metadata.len() / 1024;
                println!("  Size: {} KB", size_kb);
            }
        } else {
            println!("  (No log file yet)");
        }
    }

    Ok(())
}

/// Set a configuration value
pub fn set_config(key: &str, value: &str) -> Result<()> {
    let mut config = load_config()?;

    match key {
        "updates.enabled" => {
            config.updates.enabled = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean value: {}", value))?;
        }
        "updates.auto_install" => {
            config.updates.auto_install = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid boolean value: {}", value))?;
        }
        "updates.check_interval" => {
            if !["daily", "weekly", "never"].contains(&value) {
                return Err(anyhow::anyhow!(
                    "Invalid interval: {}. Use: daily, weekly, never",
                    value
                ));
            }
            config.updates.check_interval = value.to_string();
        }
        "updates.channel" => {
            if !["stable", "beta"].contains(&value) {
                return Err(anyhow::anyhow!(
                    "Invalid channel: {}. Use: stable, beta",
                    value
                ));
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
        assert!(config.updates.auto_install); // Changed: auto_install is now true by default
        assert_eq!(config.updates.check_interval, "daily");
        assert_eq!(config.updates.channel, "stable");
    }

    #[test]
    fn test_env_override_enabled() {
        std::env::set_var("CCO_AUTO_UPDATE", "false");
        let mut config = UpdateConfig::default();
        config.apply_env_overrides();
        assert!(!config.enabled);
        assert!(!config.auto_install);
        std::env::remove_var("CCO_AUTO_UPDATE");
    }

    #[test]
    fn test_env_override_channel() {
        std::env::set_var("CCO_AUTO_UPDATE_CHANNEL", "beta");
        let mut config = UpdateConfig::default();
        config.apply_env_overrides();
        assert_eq!(config.channel, "beta");
        std::env::remove_var("CCO_AUTO_UPDATE_CHANNEL");
    }

    #[test]
    fn test_env_override_interval() {
        std::env::set_var("CCO_AUTO_UPDATE_INTERVAL", "weekly");
        let mut config = UpdateConfig::default();
        config.apply_env_overrides();
        assert_eq!(config.check_interval, "weekly");
        std::env::remove_var("CCO_AUTO_UPDATE_INTERVAL");
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

    #[test]
    fn test_should_check_recently_checked() {
        let config = UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: Some(Utc::now()),
            last_update: None,
        };

        assert!(!should_check(&config));
    }

    #[test]
    fn test_should_check_old_check() {
        let config = UpdateConfig {
            enabled: true,
            auto_install: false,
            check_interval: "daily".to_string(),
            channel: "stable".to_string(),
            last_check: Some(Utc::now() - Duration::days(2)),
            last_update: None,
        };

        assert!(should_check(&config));
    }
}
