//! macOS LaunchAgent service management
//!
//! Creates and manages LaunchAgent plist files in ~/Library/LaunchAgents/

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::ServiceManager;

const SERVICE_NAME: &str = "com.anthropic.cco.daemon";
const PLIST_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>"#;

const PLIST_FOOTER: &str = r#"</dict>
</plist>"#;

/// macOS LaunchAgent service manager
pub struct MacOSService {
    pub plist_path: PathBuf,
    pub service_name: String,
}

impl MacOSService {
    /// Create new macOS service manager
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not determine home directory")?;

        let launch_agents_dir = home.join("Library/LaunchAgents");

        Ok(Self {
            plist_path: launch_agents_dir.join(format!("{}.plist", SERVICE_NAME)),
            service_name: SERVICE_NAME.to_string(),
        })
    }

    /// Generate plist content for LaunchAgent
    pub fn generate_plist(&self) -> Result<String> {
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

        let exe_path_str = exe_path.to_string_lossy();

        let log_file = super::super::get_daemon_log_file()?;
        let log_file_str = log_file.to_string_lossy();

        let plist_body = format!(
            r#"
	<key>Label</key>
	<string>{}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{}</string>
		<string>daemon</string>
		<string>run</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>KeepAlive</key>
	<dict>
		<key>SuccessfulExit</key>
		<false/>
	</dict>
	<key>StandardOutPath</key>
	<string>{}</string>
	<key>StandardErrorPath</key>
	<string>{}</string>
	<key>WorkingDirectory</key>
	<string>{}</string>
	<key>EnvironmentVariables</key>
	<dict>
		<key>PATH</key>
		<string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
	</dict>"#,
            self.service_name,
            exe_path_str,
            log_file_str,
            log_file_str,
            dirs::home_dir().unwrap_or_default().to_string_lossy()
        );

        Ok(format!("{}{}\n{}", PLIST_HEADER, plist_body, PLIST_FOOTER))
    }

    /// Load the LaunchAgent
    fn load_agent(&self) -> Result<()> {
        Command::new("launchctl")
            .arg("load")
            .arg(&self.plist_path)
            .output()
            .context("Failed to load LaunchAgent")?;

        Ok(())
    }

    /// Unload the LaunchAgent
    fn unload_agent(&self) -> Result<()> {
        Command::new("launchctl")
            .arg("unload")
            .arg(&self.plist_path)
            .output()
            .context("Failed to unload LaunchAgent")?;

        Ok(())
    }
}

impl ServiceManager for MacOSService {
    fn install(&self) -> Result<()> {
        // Check if already installed
        if self.is_installed()? {
            anyhow::bail!("Service is already installed");
        }

        // Create LaunchAgents directory if needed
        let parent_dir = self
            .plist_path
            .parent()
            .context("Failed to get parent directory")?;

        fs::create_dir_all(parent_dir).context("Failed to create LaunchAgents directory")?;

        // Generate and write plist file
        let plist_content = self.generate_plist()?;
        fs::write(&self.plist_path, plist_content).context("Failed to write plist file")?;

        // Load the agent
        self.load_agent()?;

        println!("✅ Service installed successfully");
        println!("   Service: {}", self.service_name);
        println!("   Location: {}", self.plist_path.display());
        println!("   The daemon will start automatically on next login");

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        if !self.is_installed()? {
            anyhow::bail!("Service is not installed");
        }

        // Unload the agent first
        let _ = self.unload_agent();

        // Remove the plist file
        fs::remove_file(&self.plist_path).context("Failed to remove plist file")?;

        println!("✅ Service uninstalled successfully");
        println!("   Service: {}", self.service_name);

        Ok(())
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(self.plist_path.exists())
    }

    fn enable(&self) -> Result<()> {
        if !self.is_installed()? {
            anyhow::bail!("Service is not installed");
        }

        self.load_agent()?;
        println!("✅ Service enabled");

        Ok(())
    }

    fn disable(&self) -> Result<()> {
        if !self.is_installed()? {
            anyhow::bail!("Service is not installed");
        }

        self.unload_agent()?;
        println!("✅ Service disabled");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_service_creation() {
        let result = MacOSService::new();
        assert!(result.is_ok());

        let service = result.unwrap();
        assert!(service
            .plist_path
            .to_string_lossy()
            .contains("Library/LaunchAgents"));
        assert!(service
            .plist_path
            .to_string_lossy()
            .contains("com.anthropic.cco.daemon.plist"));
    }

    #[test]
    fn test_plist_generation() {
        let service = MacOSService::new().unwrap();
        let plist = service.generate_plist().unwrap();

        assert!(plist.contains("<?xml version"));
        assert!(plist.contains("com.anthropic.cco.daemon"));
        assert!(plist.contains("daemon"));
        assert!(plist.contains("run"));
        assert!(plist.contains("KeepAlive"));
        assert!(plist.contains("RunAtLoad"));
    }

    #[test]
    fn test_service_name_constant() {
        assert_eq!(SERVICE_NAME, "com.anthropic.cco.daemon");
    }
}
