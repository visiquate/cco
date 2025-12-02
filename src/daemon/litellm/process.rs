//! LiteLLM process management
//!
//! Handles starting, stopping, and health checking the LiteLLM subprocess.

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// LiteLLM subprocess configuration
#[derive(Debug, Clone)]
pub struct LiteLLMConfig {
    /// Port for LiteLLM to listen on
    pub port: u16,
    /// Path to LiteLLM config file
    pub config_path: PathBuf,
    /// Path to PEX file (or None to use embedded)
    pub pex_path: Option<PathBuf>,
    /// Health check timeout
    pub health_timeout: Duration,
    /// Health check retry interval
    pub health_retry_interval: Duration,
    /// Maximum health check retries
    pub max_health_retries: u32,
}

impl Default for LiteLLMConfig {
    fn default() -> Self {
        Self {
            port: 4000,
            config_path: PathBuf::from("config/litellm_config.yaml"),
            pex_path: None,
            health_timeout: Duration::from_secs(30),
            health_retry_interval: Duration::from_millis(500),
            max_health_retries: 60, // 30 seconds total
        }
    }
}

/// LiteLLM subprocess manager
pub struct LiteLLMProcess {
    config: LiteLLMConfig,
    child: Option<Child>,
    pex_path: PathBuf,
}

impl LiteLLMProcess {
    /// Create a new LiteLLM process manager
    pub fn new(config: LiteLLMConfig) -> Result<Self> {
        let pex_path = match &config.pex_path {
            Some(path) => path.clone(),
            None => ensure_litellm_pex()?,
        };

        Ok(Self {
            config,
            child: None,
            pex_path,
        })
    }

    /// Create with default config
    pub fn with_defaults() -> Result<Self> {
        Self::new(LiteLLMConfig::default())
    }

    /// Start the LiteLLM subprocess
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running() {
            warn!("LiteLLM is already running");
            return Ok(());
        }

        info!(
            port = self.config.port,
            config = %self.config.config_path.display(),
            pex = %self.pex_path.display(),
            "Starting LiteLLM subprocess"
        );

        // Verify PEX exists
        if !self.pex_path.exists() {
            return Err(anyhow!(
                "LiteLLM PEX not found at {}",
                self.pex_path.display()
            ));
        }

        // Verify config exists
        if !self.config.config_path.exists() {
            return Err(anyhow!(
                "LiteLLM config not found at {}",
                self.config.config_path.display()
            ));
        }

        // Build command
        let mut cmd = Command::new(&self.pex_path);
        cmd.args([
            "--config",
            self.config.config_path.to_str().unwrap(),
            "--port",
            &self.config.port.to_string(),
            "--detailed_debug", // Enable debug logging
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        // Spawn process
        let child = cmd
            .spawn()
            .context("Failed to spawn LiteLLM subprocess")?;

        let pid = child.id();
        self.child = Some(child);

        info!(pid = pid, "LiteLLM subprocess spawned, waiting for health check");

        // Wait for health check
        self.wait_healthy().await?;

        info!(pid = pid, port = self.config.port, "LiteLLM is ready");
        Ok(())
    }

    /// Stop the LiteLLM subprocess
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let pid = child.id();
            info!(pid = pid, "Stopping LiteLLM subprocess");

            // Try graceful shutdown first (SIGTERM on Unix)
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                if let Err(e) = kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                    warn!(pid = pid, error = %e, "Failed to send SIGTERM, trying SIGKILL");
                }

                // Wait briefly for graceful shutdown
                sleep(Duration::from_millis(500)).await;
            }

            // Force kill if still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    info!(pid = pid, status = ?status, "LiteLLM exited");
                }
                Ok(None) => {
                    warn!(pid = pid, "LiteLLM still running, sending SIGKILL");
                    child.kill().ok();
                    child.wait().ok();
                }
                Err(e) => {
                    error!(pid = pid, error = %e, "Failed to check LiteLLM status");
                    child.kill().ok();
                }
            }
        }
        Ok(())
    }

    /// Check if the subprocess is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.child = None;
                    false
                }
                Ok(None) => true,  // Still running
                Err(_) => false,   // Error checking
            }
        } else {
            false
        }
    }

    /// Perform a health check against the LiteLLM endpoint
    pub async fn health_check(&self) -> bool {
        let url = format!("http://127.0.0.1:{}/health", self.config.port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok();

        if let Some(client) = client {
            match client.get(&url).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Wait for LiteLLM to become healthy
    async fn wait_healthy(&self) -> Result<()> {
        for attempt in 1..=self.config.max_health_retries {
            if self.health_check().await {
                debug!(attempt = attempt, "LiteLLM health check passed");
                return Ok(());
            }

            if attempt < self.config.max_health_retries {
                debug!(
                    attempt = attempt,
                    max = self.config.max_health_retries,
                    "LiteLLM not ready yet, retrying..."
                );
                sleep(self.config.health_retry_interval).await;
            }
        }

        Err(anyhow!(
            "LiteLLM failed to become healthy after {} attempts",
            self.config.max_health_retries
        ))
    }

    /// Get the LiteLLM endpoint URL
    pub fn endpoint_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.config.port)
    }

    /// Restart the subprocess
    pub async fn restart(&mut self) -> Result<()> {
        self.stop().await?;
        sleep(Duration::from_millis(500)).await;
        self.start().await
    }
}

impl Drop for LiteLLMProcess {
    fn drop(&mut self) {
        // Best-effort cleanup on drop
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

/// Ensure the LiteLLM PEX file exists
///
/// If embedded in the binary, extracts to ~/.cco/litellm.pex
/// Otherwise, looks for it in the project directory
pub fn ensure_litellm_pex() -> Result<PathBuf> {
    let daemon_dir = crate::daemon::get_daemon_dir()?;
    let pex_path = daemon_dir.join("litellm.pex");

    // Check if already extracted
    if pex_path.exists() {
        debug!(path = %pex_path.display(), "Using existing LiteLLM PEX");
        return Ok(pex_path);
    }

    // Try to find PEX in project directory (development mode)
    let project_pex = PathBuf::from("dist/litellm.pex");
    if project_pex.exists() {
        info!(path = %project_pex.display(), "Using project LiteLLM PEX");
        return Ok(project_pex);
    }

    // Check for embedded PEX bytes (production mode)
    // This will be populated by build.rs when the PEX is embedded
    #[cfg(feature = "embedded_litellm")]
    {
        const LITELLM_PEX_BYTES: &[u8] = include_bytes!("../../../dist/litellm.pex");

        info!(path = %pex_path.display(), "Extracting embedded LiteLLM PEX");
        std::fs::write(&pex_path, LITELLM_PEX_BYTES)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&pex_path, std::fs::Permissions::from_mode(0o755))?;
        }

        return Ok(pex_path);
    }

    Err(anyhow!(
        "LiteLLM PEX not found. Run scripts/build-litellm-pex.sh to create it."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LiteLLMConfig::default();
        assert_eq!(config.port, 4000);
        assert_eq!(config.max_health_retries, 60);
    }

    #[test]
    fn test_endpoint_url() {
        let config = LiteLLMConfig {
            port: 8080,
            ..Default::default()
        };
        let process = LiteLLMProcess {
            config,
            child: None,
            pex_path: PathBuf::from("/tmp/test.pex"),
        };
        assert_eq!(process.endpoint_url(), "http://127.0.0.1:8080");
    }
}
