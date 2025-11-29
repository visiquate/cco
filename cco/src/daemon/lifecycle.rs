//! Daemon lifecycle management
//!
//! Handles daemon start, stop, restart, and status operations with proper
//! PID file management and process signal handling.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::{Command, Stdio};
use std::sync::Arc;
use sysinfo::{Pid, System};
use tracing::info;

use super::config::DaemonConfig;
use super::hooks::{HookExecutor, HookRegistry};

/// Daemon status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub pid: u32,
    pub is_running: bool,
    pub started_at: DateTime<Utc>,
    pub port: u16,
    pub version: String,
}

/// PID file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidFileContent {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub port: u16,
    pub version: String,
}

/// Read the daemon port from the PID file
///
/// This function allows clients to discover which port the daemon is running on.
/// Returns an error if the daemon is not running or the PID file is invalid.
pub fn read_daemon_port() -> Result<u16> {
    let pid_file = super::get_daemon_pid_file()?;

    if !pid_file.exists() {
        bail!("Daemon is not running (no PID file found)");
    }

    let contents = fs::read_to_string(&pid_file).context("Failed to read PID file")?;

    let pid_content: PidFileContent =
        serde_json::from_str(&contents).context("Failed to parse PID file")?;

    Ok(pid_content.port)
}

/// Update the daemon PID file with the actual bound port
///
/// This is called by the daemon process after binding to a socket to record
/// the actual port (especially important when port 0 is used for random assignment).
pub fn update_daemon_port(actual_port: u16) -> Result<()> {
    let pid_file = super::get_daemon_pid_file()?;

    if !pid_file.exists() {
        bail!("PID file not found - cannot update port");
    }

    // Read existing PID file
    let contents = fs::read_to_string(&pid_file).context("Failed to read PID file")?;

    let mut pid_content: PidFileContent =
        serde_json::from_str(&contents).context("Failed to parse PID file")?;

    // Update port with actual bound port
    pid_content.port = actual_port;

    // Write back to PID file
    let pid_json = serde_json::to_string_pretty(&pid_content)?;
    fs::write(&pid_file, pid_json).context("Failed to update PID file")?;

    info!("Updated PID file with actual port: {}", actual_port);

    Ok(())
}

/// Daemon manager for lifecycle operations
pub struct DaemonManager {
    pub config: DaemonConfig,
    /// Hook registry for managing lifecycle hooks
    pub hooks_registry: Arc<HookRegistry>,
    /// Hook executor for executing registered hooks
    pub hooks_executor: HookExecutor,
}

impl DaemonManager {
    /// Create a new daemon manager with configuration
    ///
    /// Initializes the hooks system based on configuration.
    pub fn new(config: DaemonConfig) -> Self {
        // Initialize hooks registry
        let hooks_registry = Arc::new(HookRegistry::new());

        // Create hooks executor with configuration
        let hooks_executor = if config.hooks.is_enabled() {
            info!(
                timeout_ms = config.hooks.timeout_ms,
                max_retries = config.hooks.max_retries,
                "Initializing hooks system"
            );
            HookExecutor::with_config(
                hooks_registry.clone(),
                config.hooks.timeout(),
                config.hooks.max_retries,
            )
        } else {
            info!("Hooks system disabled");
            HookExecutor::new(hooks_registry.clone())
        };

        Self {
            config,
            hooks_registry,
            hooks_executor,
        }
    }

    /// Start the daemon process
    pub async fn start(&self) -> Result<()> {
        // Check if already running
        if let Ok(status) = self.get_status().await {
            if status.is_running {
                bail!(
                    "Daemon is already running on port {} (PID {})",
                    status.port,
                    status.pid
                );
            }
        }

        // Create log file if it doesn't exist
        let log_file = super::get_daemon_log_file()?;
        if !log_file.exists() {
            fs::write(&log_file, "")?;
        }

        // Create temp files for orchestrator settings with daemon config
        let temp_manager = super::TempFileManager::new();

        // Write orchestrator settings with full daemon config (includes hooks)
        temp_manager
            .write_orchestrator_settings(&self.config)
            .await
            .context("Failed to write orchestrator settings")?;

        // Generate and write system prompt with XOR deobfuscation
        let system_prompt = temp_manager.generate_system_prompt()?;
        let prompt_path = temp_manager.system_prompt_path();
        fs::write(&prompt_path, system_prompt)?;

        // Generate and write agents JSON (119 agents from orchestra-config.json)
        let agents_json = temp_manager.generate_agents_json()?;
        let agents_json_path = temp_manager.agents_json_path();
        fs::write(agents_json_path, agents_json)?;

        // Set Unix permissions for temp files
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // System prompt gets restrictive permissions (owner read/write only)
            fs::set_permissions(&prompt_path, fs::Permissions::from_mode(0o600))?;
            // Agents JSON gets read permissions for Claude Code
            fs::set_permissions(agents_json_path, fs::Permissions::from_mode(0o644))?;
        }

        tracing::info!("Generated agents JSON at: {}", agents_json_path.display());

        // Get the binary path (the cco binary itself)
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

        // Start the daemon with 'daemon run' command (runs HTTP server in foreground)
        let child = Command::new(&exe_path)
            .arg("daemon")
            .arg("run")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn daemon process")?;

        let pid = child.id();

        // Write initial PID file with requested port (will be updated by daemon process with actual port)
        let pid_content = PidFileContent {
            pid,
            started_at: Utc::now(),
            port: self.config.port,
            version: crate::version::DateVersion::current().to_string(),
        };

        let pid_file = super::get_daemon_pid_file()?;
        let pid_json = serde_json::to_string_pretty(&pid_content)?;
        fs::write(&pid_file, pid_json).context("Failed to write PID file")?;

        println!("✅ Daemon started successfully (PID: {})", pid);
        if self.config.port == 0 {
            println!("   Port: OS will assign random port (checking...)");
        } else {
            println!("   Requested Port: {}", self.config.port);
        }
        println!("   Log file: {}", log_file.display());
        println!("   Settings: {}", temp_manager.settings_path().display());

        // CRITICAL: Wait for daemon to fully initialize and update PID file with actual port
        // Daemon needs time to:
        // - Initialize Tokio runtime (~100-500ms)
        // - Load configuration and analytics (~500-2000ms)
        // - Bind to socket and register routes (~150-700ms)
        // - Update PID file with actual port (~50-100ms)
        // Total typical startup: 1-3 seconds, add buffer for slower systems
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Read the actual port from the updated PID file
        if let Ok(actual_port) = read_daemon_port() {
            println!("   Actual Port: {}", actual_port);
            println!("   Dashboard: http://{}:{}", self.config.host, actual_port);
        } else {
            println!(
                "   Dashboard: http://{}:{}",
                self.config.host, self.config.port
            );
        }

        Ok(())
    }

    /// Stop the daemon process
    pub async fn stop(&self) -> Result<()> {
        let status = self.get_status().await?;

        if !status.is_running {
            println!("⚠️  Daemon is not running");
            // Clean up stale PID file
            let pid_file = super::get_daemon_pid_file()?;
            if pid_file.exists() {
                let _ = fs::remove_file(&pid_file);
            }
            // Clean up temp files even if daemon isn't running
            let temp_manager = super::TempFileManager::new();
            let _ = temp_manager.cleanup_files().await;
            return Ok(());
        }

        println!("Shutting down daemon (PID {})...", status.pid);

        // Send SIGTERM signal
        self.send_signal(status.pid, Signal::Term)
            .context("Failed to send SIGTERM")?;

        // Wait for graceful shutdown (up to 10 seconds)
        for _ in 0..20 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            if !self.is_process_running(status.pid) {
                println!("✅ Daemon shut down gracefully");
                self.cleanup_pid_file()?;

                // Clean up temp files
                let temp_manager = super::TempFileManager::new();
                temp_manager
                    .cleanup_files()
                    .await
                    .context("Failed to cleanup orchestrator temp files")?;

                return Ok(());
            }
        }

        // If still running, try SIGKILL
        println!("⚠️  Process did not shut down gracefully, sending SIGKILL...");

        self.send_signal(status.pid, Signal::Kill)
            .context("Failed to send SIGKILL")?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        if !self.is_process_running(status.pid) {
            println!("✅ Daemon force shut down");
            self.cleanup_pid_file()?;

            // Clean up temp files
            let temp_manager = super::TempFileManager::new();
            temp_manager
                .cleanup_files()
                .await
                .context("Failed to cleanup orchestrator temp files")?;

            Ok(())
        } else {
            bail!("Failed to shutdown daemon (PID {})", status.pid)
        }
    }

    /// Restart the daemon
    pub async fn restart(&self) -> Result<()> {
        println!("Restarting daemon...");

        // Try to stop first, ignore errors if not running
        let _ = self.stop().await;

        // Wait a bit before starting
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        self.start().await?;
        println!("✅ Daemon restarted successfully");

        Ok(())
    }

    /// Get daemon status
    pub async fn get_status(&self) -> Result<DaemonStatus> {
        let pid_file = super::get_daemon_pid_file()?;

        if !pid_file.exists() {
            bail!("Daemon is not running (no PID file found)");
        }

        let contents = fs::read_to_string(&pid_file).context("Failed to read PID file")?;

        let pid_content: PidFileContent =
            serde_json::from_str(&contents).context("Failed to parse PID file")?;

        let is_running = self.is_process_running(pid_content.pid);

        if !is_running {
            let _ = fs::remove_file(&pid_file);
        }

        Ok(DaemonStatus {
            pid: pid_content.pid,
            is_running,
            started_at: pid_content.started_at,
            port: pid_content.port,
            version: pid_content.version,
        })
    }

    /// Check if process is running
    fn is_process_running(&self, pid: u32) -> bool {
        let mut system = System::new();
        system.refresh_processes();
        system.process(Pid::from_u32(pid)).is_some()
    }

    /// Clean up PID file
    fn cleanup_pid_file(&self) -> Result<()> {
        let pid_file = super::get_daemon_pid_file()?;
        if pid_file.exists() {
            fs::remove_file(&pid_file)?;
        }
        Ok(())
    }

    /// Send signal to process
    fn send_signal(&self, pid: u32, signal: Signal) -> Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal as NixSignal};
            use nix::unistd::Pid as NixPid;

            let nix_signal = match signal {
                Signal::Term => NixSignal::SIGTERM,
                Signal::Kill => NixSignal::SIGKILL,
            };

            let nix_pid = NixPid::from_raw(pid as i32);
            signal::kill(nix_pid, nix_signal).context("Failed to send signal")?;
        }

        #[cfg(not(unix))]
        {
            let mut system = System::new();
            system.refresh_processes();

            if let Some(process) = system.process(Pid::from_u32(pid)) {
                if !process.kill() {
                    bail!("Failed to terminate process");
                }
            } else {
                bail!("Process not found");
            }
        }

        Ok(())
    }
}

/// Signal types
#[derive(Debug, Clone, Copy)]
enum Signal {
    Term,
    Kill,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_manager_creation() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config);
        assert_eq!(manager.config.port, 0); // Default is now random OS-assigned port
    }

    #[test]
    fn test_pid_file_content_serialization() {
        let content = PidFileContent {
            pid: 1234,
            started_at: Utc::now(),
            port: 3000,
            version: "2025.11.1".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: PidFileContent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.pid, 1234);
        assert_eq!(deserialized.port, 3000);
    }

    #[test]
    fn test_is_process_running() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config);

        // Current process should be running
        let current_pid = std::process::id();
        assert!(manager.is_process_running(current_pid));

        // Invalid PID should not be running
        assert!(!manager.is_process_running(999999));
    }

    #[tokio::test]
    async fn test_get_status_no_pid_file() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config);

        let result = manager.get_status().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_pid_file() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config);

        // Should not error if file doesn't exist
        assert!(manager.cleanup_pid_file().is_ok());
    }

    #[tokio::test]
    async fn test_daemon_startup_creates_temp_files() {
        use std::env;

        let temp_manager = super::super::TempFileManager::new();

        // Ensure cleanup before test
        let _ = temp_manager.cleanup_files().await;

        // Create files
        temp_manager.create_files().await.unwrap();

        // Verify settings file exists
        let temp_dir = env::temp_dir();
        assert!(temp_dir.join(".cco-orchestrator-settings").exists());

        // Cleanup
        temp_manager.cleanup_files().await.unwrap();

        // Verify cleanup
        assert!(!temp_dir.join(".cco-orchestrator-settings").exists());
    }
}
