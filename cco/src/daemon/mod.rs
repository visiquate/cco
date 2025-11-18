//! Cross-platform daemon lifecycle management
//!
//! Provides daemon start, stop, restart, status, logs, and service installation
//! functionality for macOS (LaunchAgent) and Linux (systemd).
//!
//! Also includes temporary file management for serving agent definitions.

pub mod config;
pub mod lifecycle;
pub mod service;
pub mod temp_files;

pub use config::{DaemonConfig, load_config, save_config};
pub use lifecycle::{DaemonManager, DaemonStatus};
pub use service::{ServiceManager, PlatformService};
pub use temp_files::TempFileManager;

use anyhow::Result;
use std::path::PathBuf;

/// Get daemon configuration directory
pub fn get_daemon_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let daemon_dir = home.join(".cco");
    std::fs::create_dir_all(&daemon_dir)?;
    Ok(daemon_dir)
}

/// Get daemon log file path
pub fn get_daemon_log_file() -> Result<PathBuf> {
    let daemon_dir = get_daemon_dir()?;
    Ok(daemon_dir.join("daemon.log"))
}

/// Get daemon PID file path
pub fn get_daemon_pid_file() -> Result<PathBuf> {
    let daemon_dir = get_daemon_dir()?;
    Ok(daemon_dir.join("daemon.pid"))
}

/// Get daemon config file path
pub fn get_daemon_config_file() -> Result<PathBuf> {
    let daemon_dir = get_daemon_dir()?;
    Ok(daemon_dir.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_dir_creation() {
        let result = get_daemon_dir();
        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(dir.to_string_lossy().contains(".cco"));
    }

    #[test]
    fn test_get_daemon_log_file() {
        let result = get_daemon_log_file();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("daemon.log"));
    }

    #[test]
    fn test_get_daemon_pid_file() {
        let result = get_daemon_pid_file();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("daemon.pid"));
    }

    #[test]
    fn test_get_daemon_config_file() {
        let result = get_daemon_config_file();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
