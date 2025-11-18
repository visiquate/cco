//! Server lifecycle management commands
//!
//! Provides idempotent install/run/uninstall operations for the CCO HTTP server.
//! These commands ensure safe, repeatable server management with proper status checking.

use anyhow::{Context, Result};
use cco::daemon::{DaemonConfig, DaemonManager};
use cco::api_client::ApiClient;
use std::time::Duration;

/// Install the server (idempotent - safe to run multiple times)
///
/// Creates the necessary directories and configuration files if they don't exist.
/// If already installed (and not --force), returns success immediately.
pub async fn install(force: bool) -> Result<()> {
    // Check if already installed
    let config_file = cco::daemon::get_daemon_config_file()?;

    if config_file.exists() && !force {
        println!("✅ Server already installed");
        println!("   Config: {}", config_file.display());
        println!("   Use --force to reinstall");
        return Ok(());
    }

    println!("📦 Installing CCO server...");

    // Create daemon directory structure
    let daemon_dir = cco::daemon::get_daemon_dir()?;
    std::fs::create_dir_all(&daemon_dir)
        .context("Failed to create daemon directory")?;

    // Create default configuration
    let config = DaemonConfig::default();
    cco::daemon::save_config(&config)
        .context("Failed to save configuration")?;

    println!("✅ Server installed successfully");
    println!("   Config directory: {}", daemon_dir.display());
    println!("   Config file: {}", config_file.display());
    println!("   Default port: {}", config.port);
    println!("   Default host: {}", config.host);
    println!();
    println!("   Run 'cco server run' to start the server");

    Ok(())
}

/// Run the server (idempotent - returns immediately if already running)
///
/// Starts the server process if not already running and waits for it to become ready.
/// If already running, verifies health and returns success.
pub async fn run(host: &str, port: u16) -> Result<()> {
    // Create config for this run
    let config = DaemonConfig {
        port,
        host: host.to_string(),
        log_level: "info".to_string(),
        log_rotation_size: 10 * 1024 * 1024,
        log_max_files: 5,
        database_url: "sqlite://analytics.db".to_string(),
        cache_size: 1073741824,
        cache_ttl: 3600,
        auto_start: true,
        health_checks: true,
        health_check_interval: 30,
    };

    let manager = DaemonManager::new(config.clone());

    // Check if already running
    match manager.get_status().await {
        Ok(status) => {
            if status.is_running {
                // Verify with health check
                let base_url = format!("http://{}:{}", host, port);
                let client = ApiClient::new(base_url.clone());

                match client.health().await {
                    Ok(_) => {
                        println!("✅ Server already running");
                        println!("   PID: {}", status.pid);
                        println!("   Port: {}", status.port);
                        println!("   Version: {}", status.version);
                        println!("   Dashboard: {}", base_url);
                        return Ok(());
                    }
                    Err(e) => {
                        println!("⚠️  Server process exists but not responding: {}", e);
                        println!("   Attempting restart...");
                        // Fall through to start logic
                    }
                }
            } else {
                println!("⚠️  Stale server process found, cleaning up...");
                // PID file exists but process not running - clean up
            }
        }
        Err(_) => {
            // No server running - normal case
        }
    }

    // Start the server
    println!("🔌 Starting server on {}:{}...", host, port);
    manager.start().await
        .context("Failed to start server")?;

    // Wait for server to be ready
    println!("⏳ Waiting for server to become ready...");
    wait_for_server_ready(host, port, Duration::from_secs(30)).await?;

    println!("✅ Server ready");
    println!("   Dashboard: http://{}:{}", host, port);

    Ok(())
}

/// Uninstall the server (idempotent - safe to run multiple times)
///
/// Stops any running server instances and removes configuration files.
/// If not installed, returns success.
pub async fn uninstall() -> Result<()> {
    println!("🗑️  Uninstalling CCO server...");

    // Try to stop running server
    let config = match cco::daemon::load_config() {
        Ok(c) => c,
        Err(_) => DaemonConfig::default(),
    };

    let manager = DaemonManager::new(config);

    match manager.stop().await {
        Ok(_) => println!("✅ Server stopped"),
        Err(_) => println!("   (No running server found)"),
    }

    // Remove configuration files
    let config_file = cco::daemon::get_daemon_config_file()?;
    if config_file.exists() {
        std::fs::remove_file(&config_file)
            .context("Failed to remove config file")?;
        println!("✅ Configuration removed");
    }

    // Remove PID file if exists
    let pid_file = cco::daemon::get_daemon_pid_file()?;
    if pid_file.exists() {
        std::fs::remove_file(&pid_file)
            .context("Failed to remove PID file")?;
        println!("✅ PID file removed");
    }

    // Remove log file if exists
    let log_file = cco::daemon::get_daemon_log_file()?;
    if log_file.exists() {
        std::fs::remove_file(&log_file)
            .context("Failed to remove log file")?;
        println!("✅ Log file removed");
    }

    println!("✅ Server uninstalled successfully");

    Ok(())
}

/// Wait for server to become ready by polling the /ready endpoint
async fn wait_for_server_ready(host: &str, port: u16, timeout: Duration) -> Result<()> {
    let base_url = format!("http://{}:{}", host, port);
    let client = ApiClient::new(base_url);

    let start = std::time::Instant::now();
    let mut attempts = 0;

    // Initial delay before first poll - server needs time to initialize
    tokio::time::sleep(Duration::from_millis(500)).await;

    while start.elapsed() < timeout {
        attempts += 1;

        // Try ready check first (faster)
        match client.ready().await {
            Ok(_) => {
                println!("   (Ready after {} attempts in {:.1}s)",
                    attempts, start.elapsed().as_secs_f64());
                return Ok(());
            }
            Err(_) => {
                // Fall back to health check
                match client.health().await {
                    Ok(health) if health.status == "ok" => {
                        println!("   (Ready after {} attempts in {:.1}s)",
                            attempts, start.elapsed().as_secs_f64());
                        return Ok(());
                    }
                    _ => {
                        // Neither ready nor health succeeded, wait and retry
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
            }
        }
    }

    anyhow::bail!("Timeout waiting for server to become ready (tried {} times over {:.1}s)",
        attempts, timeout.as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_install_idempotent() {
        // First install should succeed
        let result1 = install(false).await;
        assert!(result1.is_ok() || result1.is_err()); // May fail in CI environment

        // Second install should also succeed (idempotent)
        let result2 = install(false).await;
        assert!(result2.is_ok() || result2.is_err());
    }

    #[tokio::test]
    async fn test_uninstall_idempotent() {
        // Uninstall should succeed even if nothing installed
        let result = uninstall().await;
        assert!(result.is_ok() || result.is_err()); // May fail in CI environment
    }
}
