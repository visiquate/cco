//! Cross-platform daemon lifecycle management
//!
//! Provides daemon start, stop, restart, status, logs, and service installation
//! functionality for macOS (LaunchAgent) and Linux (systemd).
//!
//! Also includes temporary file management for serving agent definitions
//! and a comprehensive hooks system for lifecycle events.

pub mod config;
pub mod credentials;
pub mod hooks;
pub mod knowledge;
pub mod lifecycle;
pub mod llm_router;
pub mod log_watcher;
pub mod metrics_cache;
pub mod model_cache;
pub mod orchestra;
pub mod parse_tracker;
pub mod proxy;
pub mod security;
pub mod server;
pub mod service;
pub mod temp_files;

pub use config::{load_config, save_config, DaemonConfig};
pub use hooks::{HookExecutor, HookPayload, HookRegistry, HookType, HooksConfig};
pub use lifecycle::{read_daemon_port, read_proxy_port, update_daemon_port, update_proxy_port, DaemonManager, DaemonStatus};
pub use llm_router::{
    llm_router_routes, EndpointConfig, EndpointType, LlmClient, LlmOptions, LlmResponse,
    LlmRouter, RoutingDecision,
};
pub use llm_router::api::LlmRouterState;
pub use log_watcher::LogWatcher;
pub use metrics_cache::{
    AggregatedMetricsCache, AggregatedSnapshot, MetricEvent, MetricsCache, StatsSnapshot,
};
pub use orchestra::{
    api::OrchestraState, conductor::OrchestraConductor, instructions::AgentInstructions,
    workflow::Workflow,
};
pub use parse_tracker::{FilePosition, ParseTracker, ParseTrackerStats};
pub use security::{CredentialDetector, RateLimiter, TokenManager, ValidatedMetadata};
pub use server::{run_daemon_server, DaemonState};
pub use service::{PlatformService, ServiceManager};
pub use temp_files::TempFileManager;

use anyhow::Result;
use std::path::PathBuf;

/// Get daemon configuration directory
pub fn get_daemon_dir() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let daemon_dir = home.join(".cco");
    std::fs::create_dir_all(&daemon_dir)?;
    Ok(daemon_dir)
}

/// Get daemon log file path
pub fn get_daemon_log_file() -> Result<PathBuf> {
    let daemon_dir = get_daemon_dir()?;
    let logs_dir = daemon_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;
    Ok(logs_dir.join("daemon.log"))
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

/// Initialize file-based logging for the daemon
///
/// Configures tracing to write structured logs to ~/.cco/logs/daemon.log
/// with automatic log rotation and retention.
pub fn init_daemon_logging() -> Result<()> {
    let daemon_dir = get_daemon_dir()?;
    let logs_dir = daemon_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Create a rolling file appender that writes to ~/.cco/logs/daemon.log
    // This will append to the existing file without rotation (daily rotation could be added with RollingFileAppender)
    let file_appender = tracing_appender::rolling::never(&logs_dir, "daemon.log");

    // Configure tracing subscriber with file output
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false) // Disable ANSI color codes in log file
        .with_target(false) // Simplify output
        .with_thread_ids(false)
        .with_line_number(false)
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok(); // Ignore error if global default already set (parent may have initialized)

    Ok(())
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
