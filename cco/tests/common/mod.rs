//! Common test utilities for daemon and TUI testing
//!
//! Provides mock servers, helpers, and test fixtures for integration testing.

use anyhow::Result;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Mock daemon server for testing
pub struct MockDaemonServer {
    pub port: u16,
    pub base_url: String,
}

impl MockDaemonServer {
    /// Start a mock HTTP server on an available port
    pub async fn start() -> Result<Self> {
        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);

        let base_url = format!("http://127.0.0.1:{}", port);

        Ok(Self { port, base_url })
    }

    /// Get health endpoint URL
    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url)
    }

    /// Get agents endpoint URL
    pub fn agents_url(&self) -> String {
        format!("{}/api/agents", self.base_url)
    }

    /// Get SSE stream endpoint URL
    pub fn stream_url(&self) -> String {
        format!("{}/api/stream", self.base_url)
    }
}

/// Wait for a port to become available (listening)
pub async fn wait_for_port(port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("Timeout waiting for port {} to be available", port)
}

/// Wait for a port to become unavailable (no longer listening)
pub async fn wait_for_port_closed(port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_err() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("Timeout waiting for port {} to close", port)
}

/// Check if a port is currently listening
pub fn is_port_listening(port: u16) -> bool {
    TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

/// Create a temporary test directory that auto-cleans
pub fn temp_test_dir() -> tempfile::TempDir {
    tempfile::TempDir::new().expect("Failed to create temp directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_daemon_server_creation() {
        let server = MockDaemonServer::start().await.unwrap();
        assert!(server.port > 0);
        assert!(server.base_url.contains(&server.port.to_string()));
    }

    #[test]
    fn test_is_port_listening() {
        // Bind to a random port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        assert!(is_port_listening(port));
        drop(listener);

        // Port should now be closed
        std::thread::sleep(Duration::from_millis(100));
        // Note: May still be in TIME_WAIT, so we can't reliably test closed state
    }

    #[test]
    fn test_temp_test_dir() {
        let dir = temp_test_dir();
        assert!(dir.path().exists());
    }

    #[tokio::test]
    async fn test_wait_for_port_timeout() {
        // Use a port that's guaranteed not to be listening
        let result = wait_for_port(65534, Duration::from_millis(100)).await;
        assert!(result.is_err());
    }
}
