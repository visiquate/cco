//! Linux systemd service management
//!
//! Creates and manages systemd user service unit files in ~/.config/systemd/user/

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::ServiceManager;

const SERVICE_NAME: &str = "cco-daemon";
const SERVICE_FILE_NAME: &str = "cco-daemon.service";

/// Linux systemd service manager
pub struct LinuxService {
    pub service_dir: PathBuf,
    pub service_path: PathBuf,
    pub service_name: String,
}

impl LinuxService {
    /// Create new Linux service manager
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;

        let service_dir = home.join(".config/systemd/user");
        let service_path = service_dir.join(SERVICE_FILE_NAME);

        Ok(Self {
            service_dir,
            service_path,
            service_name: SERVICE_NAME.to_string(),
        })
    }

    /// Generate systemd service unit content
    pub fn generate_service_unit(&self) -> Result<String> {
        let exe_path = std::env::current_exe()
            .context("Failed to get current executable path")?;

        let exe_path_str = exe_path.to_string_lossy();

        let log_file = super::super::get_daemon_log_file()?;
        let log_file_str = log_file.to_string_lossy();

        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        let home_str = home.to_string_lossy();

        let service_unit = format!(
            r#"[Unit]
Description=Claude Code Orchestra Daemon
After=network.target

[Service]
Type=simple
ExecStart={} daemon run
Restart=always
RestartSec=10
StandardOutput=append:{}
StandardError=append:{}
WorkingDirectory={}

[Install]
WantedBy=default.target
"#,
            exe_path_str, log_file_str, log_file_str, home_str
        );

        Ok(service_unit)
    }

    /// Reload systemd user daemon configuration
    fn systemctl_daemon_reload(&self) -> Result<()> {
        Command::new("systemctl")
            .arg("--user")
            .arg("daemon-reload")
            .output()
            .context("Failed to reload systemd daemon")?;

        Ok(())
    }

    /// Start the service
    fn systemctl_start(&self) -> Result<()> {
        Command::new("systemctl")
            .arg("--user")
            .arg("start")
            .arg(&self.service_name)
            .output()
            .context("Failed to start service")?;

        Ok(())
    }

    /// Stop the service
    fn systemctl_stop(&self) -> Result<()> {
        Command::new("systemctl")
            .arg("--user")
            .arg("stop")
            .arg(&self.service_name)
            .output()
            .context("Failed to stop service")?;

        Ok(())
    }

    /// Enable the service
    fn systemctl_enable(&self) -> Result<()> {
        Command::new("systemctl")
            .arg("--user")
            .arg("enable")
            .arg(&self.service_name)
            .output()
            .context("Failed to enable service")?;

        Ok(())
    }

    /// Disable the service
    fn systemctl_disable(&self) -> Result<()> {
        Command::new("systemctl")
            .arg("--user")
            .arg("disable")
            .arg(&self.service_name)
            .output()
            .context("Failed to disable service")?;

        Ok(())
    }
}

impl ServiceManager for LinuxService {
    fn install(&self) -> Result<()> {
        // Check if already installed
        if self.is_installed()? {
            anyhow::bail!("Service is already installed");
        }

        // Create systemd user directory if needed
        fs::create_dir_all(&self.service_dir)
            .context("Failed to create systemd user directory")?;

        // Generate and write service unit file
        let service_unit = self.generate_service_unit()?;
        fs::write(&self.service_path, service_unit)
            .context("Failed to write service unit file")?;

        // Reload systemd daemon
        self.systemctl_daemon_reload()?;

        // Enable the service
        self.systemctl_enable()?;

        println!("✅ Service installed successfully");
        println!("   Service: {}", self.service_name);
        println!("   Location: {}", self.service_path.display());
        println!("   The daemon will start automatically on boot");

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        if !self.is_installed()? {
            anyhow::bail!("Service is not installed");
        }

        // Stop and disable the service
        let _ = self.systemctl_stop();
        let _ = self.systemctl_disable();

        // Remove the service unit file
        fs::remove_file(&self.service_path)
            .context("Failed to remove service unit file")?;

        // Reload systemd daemon
        self.systemctl_daemon_reload()?;

        println!("✅ Service uninstalled successfully");
        println!("   Service: {}", self.service_name);

        Ok(())
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(self.service_path.exists())
    }

    fn enable(&self) -> Result<()> {
        if !self.is_installed()? {
            anyhow::bail!("Service is not installed");
        }

        self.systemctl_daemon_reload()?;
        self.systemctl_enable()?;
        self.systemctl_start()?;

        println!("✅ Service enabled");

        Ok(())
    }

    fn disable(&self) -> Result<()> {
        if !self.is_installed()? {
            anyhow::bail!("Service is not installed");
        }

        self.systemctl_stop()?;
        self.systemctl_disable()?;

        println!("✅ Service disabled");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_service_creation() {
        let result = LinuxService::new();
        assert!(result.is_ok());

        let service = result.unwrap();
        assert!(service.service_path.to_string_lossy().contains(".config/systemd/user"));
        assert!(service.service_path.to_string_lossy().contains("cco-daemon.service"));
    }

    #[test]
    fn test_service_unit_generation() {
        let service = LinuxService::new().unwrap();
        let unit = service.generate_service_unit().unwrap();

        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("[Install]"));
        assert!(unit.contains("daemon run"));
        assert!(unit.contains("Restart=always"));
        assert!(unit.contains("Type=simple"));
    }

    #[test]
    fn test_service_name_constant() {
        assert_eq!(SERVICE_NAME, "cco-daemon");
        assert_eq!(SERVICE_FILE_NAME, "cco-daemon.service");
    }
}
