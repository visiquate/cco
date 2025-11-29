//! Integration tests for Credential Management CLI
//!
//! Tests all aspects of the `cco credentials` CLI commands including:
//! - `cco credentials store/retrieve/delete/list`
//! - Migration from old JavaScript format
//! - Error handling (daemon not running)
//! - Token authentication
//! - Project ID auto-detection
//! - Output formatting
//!
//! Run with: cargo test credentials_cli

use anyhow::Result;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// =============================================================================
// Test Helpers
// =============================================================================

/// Helper to run `cco credentials` CLI commands
struct CliTestHarness {
    cco_binary: PathBuf,
    temp_dir: TempDir,
    daemon_port: Option<u16>,
}

impl CliTestHarness {
    /// Create a new CLI test harness
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;

        // Find the cco binary (would be in target/debug or target/release)
        let cco_binary = if cfg!(debug_assertions) {
            std::env::current_dir()?.join("target/debug/cco")
        } else {
            std::env::current_dir()?.join("target/release/cco")
        };

        Ok(Self {
            cco_binary,
            temp_dir,
            daemon_port: None,
        })
    }

    /// Run a CLI command and capture output
    fn run_command(&self, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(&self.cco_binary)
            .args(args)
            .current_dir(self.temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        Ok(CommandOutput {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Start a test daemon for CLI integration
    async fn start_daemon(&mut self) -> Result<u16> {
        // Find available port
        let port = find_available_port();

        // Start daemon in background
        Command::new(&self.cco_binary)
            .args(&["daemon", "start", "--port", &port.to_string()])
            .current_dir(self.temp_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Wait for daemon to be ready
        sleep(Duration::from_secs(2)).await;

        self.daemon_port = Some(port);
        Ok(port)
    }

    /// Stop the test daemon
    fn stop_daemon(&self) -> Result<()> {
        if let Some(port) = self.daemon_port {
            Command::new(&self.cco_binary)
                .args(&["shutdown", "--port", &port.to_string()])
                .current_dir(self.temp_dir.path())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        }
        Ok(())
    }

    /// Check if daemon is running
    fn is_daemon_running(&self) -> bool {
        self.run_command(&["status"]).is_ok()
    }
}

impl Drop for CliTestHarness {
    fn drop(&mut self) {
        let _ = self.stop_daemon();
    }
}

struct CommandOutput {
    success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    listener.local_addr().unwrap().port()
}

// Note: These tests are currently ignored because they require:
// 1. The cco binary to be built (cargo build)
// 2. The daemon to support credential endpoints
// 3. The CLI commands to be fully wired up
//
// Remove #[ignore] once Phase 2 is fully complete

// =============================================================================
// SECTION 1: Basic CLI Command Tests (6 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_store_credential() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    let output = harness
        .run_command(&["credentials", "store", "test_key", "test_value"])
        .unwrap();

    assert!(output.success, "Store command should succeed");
    assert!(
        output.stdout.contains("Credential stored"),
        "Output should confirm storage"
    );

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_store_and_retrieve() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store credential
    let store_output = harness
        .run_command(&["credentials", "store", "api_key", "sk_12345"])
        .unwrap();
    assert!(store_output.success);

    // Retrieve credential
    let retrieve_output = harness
        .run_command(&["credentials", "retrieve", "api_key"])
        .unwrap();

    assert!(retrieve_output.success, "Retrieve command should succeed");
    assert_eq!(
        retrieve_output.stdout.trim(),
        "sk_12345",
        "Retrieved value should match stored value"
    );

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_list_credentials() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store multiple credentials
    harness
        .run_command(&["credentials", "store", "key1", "value1"])
        .unwrap();
    harness
        .run_command(&["credentials", "store", "key2", "value2"])
        .unwrap();
    harness
        .run_command(&["credentials", "store", "key3", "value3"])
        .unwrap();

    // List credentials
    let output = harness.run_command(&["credentials", "list"]).unwrap();

    assert!(output.success, "List command should succeed");
    assert!(output.stdout.contains("key1"), "Should list key1");
    assert!(output.stdout.contains("key2"), "Should list key2");
    assert!(output.stdout.contains("key3"), "Should list key3");

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_delete_credential() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store credential
    harness
        .run_command(&["credentials", "store", "delete_me", "secret"])
        .unwrap();

    // Delete it
    let delete_output = harness
        .run_command(&["credentials", "delete", "delete_me"])
        .unwrap();

    assert!(delete_output.success, "Delete command should succeed");
    assert!(
        delete_output.stdout.contains("deleted"),
        "Output should confirm deletion"
    );

    // Try to retrieve - should fail
    let retrieve_output = harness
        .run_command(&["credentials", "retrieve", "delete_me"])
        .unwrap();

    assert!(!retrieve_output.success, "Retrieve should fail after delete");

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_check_rotation() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store some credentials
    harness
        .run_command(&["credentials", "store", "old_key", "secret"])
        .unwrap();

    // Check rotation
    let output = harness
        .run_command(&["credentials", "check-rotation"])
        .unwrap();

    assert!(output.success, "Check rotation command should succeed");
    // Output format depends on implementation
    // Should either show credentials needing rotation or "all up to date"

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_empty_list() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // List when empty
    let output = harness.run_command(&["credentials", "list"]).unwrap();

    assert!(output.success, "List command should succeed even when empty");
    assert!(
        output.stdout.contains("No credentials") || output.stdout.is_empty(),
        "Should indicate empty list"
    );

    harness.stop_daemon().unwrap();
}

// =============================================================================
// SECTION 2: Error Handling Tests (5 tests)
// =============================================================================

#[test]
#[ignore] // Remove after daemon integration complete
fn test_cli_daemon_not_running() {
    let harness = CliTestHarness::new().unwrap();

    // Try to store without daemon running
    let output = harness
        .run_command(&["credentials", "store", "key", "value"])
        .unwrap();

    assert!(!output.success, "Command should fail without daemon");
    assert!(
        output.stderr.contains("not running") || output.stderr.contains("Failed to connect"),
        "Error should indicate daemon not running"
    );
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_retrieve_nonexistent() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    let output = harness
        .run_command(&["credentials", "retrieve", "does_not_exist"])
        .unwrap();

    assert!(!output.success, "Retrieve should fail for nonexistent key");
    assert!(
        output.stderr.contains("not found") || output.stderr.contains("Failed"),
        "Error should indicate key not found"
    );

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_delete_nonexistent() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    let output = harness
        .run_command(&["credentials", "delete", "does_not_exist"])
        .unwrap();

    assert!(!output.success, "Delete should fail for nonexistent key");

    harness.stop_daemon().unwrap();
}

#[test]
#[ignore] // Remove after daemon integration complete
fn test_cli_invalid_command() {
    let harness = CliTestHarness::new().unwrap();

    // Try invalid subcommand
    let output = harness
        .run_command(&["credentials", "invalid_command"])
        .unwrap();

    assert!(!output.success, "Invalid command should fail");
    // Should show help or error message
}

#[test]
#[ignore] // Remove after daemon integration complete
fn test_cli_missing_arguments() {
    let harness = CliTestHarness::new().unwrap();

    // Try store without value
    let output = harness
        .run_command(&["credentials", "store", "key_only"])
        .unwrap();

    assert!(!output.success, "Command with missing args should fail");
    // Should show help or indicate missing arguments
}

// =============================================================================
// SECTION 3: Authentication and Token Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_automatic_token_generation() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // First command should auto-generate token
    let output1 = harness
        .run_command(&["credentials", "store", "test1", "value1"])
        .unwrap();

    assert!(output1.success, "First command should succeed");

    // Second command should reuse token
    let output2 = harness
        .run_command(&["credentials", "store", "test2", "value2"])
        .unwrap();

    assert!(output2.success, "Second command should succeed");

    // Both should use same token (no re-authentication prompt)
    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_token_persistence() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store credential
    harness
        .run_command(&["credentials", "store", "key1", "value1"])
        .unwrap();

    // Stop daemon
    harness.stop_daemon().unwrap();

    // Restart daemon
    harness.start_daemon().await.unwrap();

    // Token should still work (if cached)
    let output = harness
        .run_command(&["credentials", "retrieve", "key1"])
        .unwrap();

    assert!(output.success, "Command should work after daemon restart");

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_project_id_detection() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Initialize a git repo in temp dir
    Command::new("git")
        .args(&["init"])
        .current_dir(harness.temp_dir.path())
        .output()
        .unwrap();

    // Store credential (should auto-detect project ID from git)
    let output = harness
        .run_command(&["credentials", "store", "git_key", "value"])
        .unwrap();

    assert!(output.success, "Should work with git project detection");

    harness.stop_daemon().unwrap();
}

// =============================================================================
// SECTION 4: Migration Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after migration feature implemented
async fn test_migrate_from_javascript_format() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Create old JavaScript-style credentials file
    let old_creds_path = harness.temp_dir.path().join("credentials.json");
    let old_creds_content = r#"{
        "credentials": {
            "old_api_key": {
                "value": "sk_old_12345",
                "type": "api-token",
                "created": "2024-01-01T00:00:00Z"
            },
            "old_db_password": {
                "value": "postgres_secret",
                "type": "database",
                "created": "2024-01-01T00:00:00Z"
            }
        }
    }"#;
    std::fs::write(&old_creds_path, old_creds_content).unwrap();

    // Run migration command
    let migrate_output = harness
        .run_command(&[
            "credentials",
            "migrate",
            "--from",
            old_creds_path.to_str().unwrap(),
        ])
        .unwrap();

    assert!(migrate_output.success, "Migration should succeed");
    assert!(
        migrate_output.stdout.contains("Migrated 2 credentials")
            || migrate_output.stdout.contains("migrated"),
        "Should report migration count"
    );

    // Verify migrated credentials are accessible
    let retrieve1 = harness
        .run_command(&["credentials", "retrieve", "old_api_key"])
        .unwrap();

    assert!(retrieve1.success);
    assert_eq!(retrieve1.stdout.trim(), "sk_old_12345");

    let retrieve2 = harness
        .run_command(&["credentials", "retrieve", "old_db_password"])
        .unwrap();

    assert!(retrieve2.success);
    assert_eq!(retrieve2.stdout.trim(), "postgres_secret");

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after migration feature implemented
async fn test_migrate_from_nonexistent_file() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Try to migrate from non-existent file
    let output = harness
        .run_command(&["credentials", "migrate", "--from", "/nonexistent/file.json"])
        .unwrap();

    assert!(!output.success, "Migration should fail for missing file");
    assert!(
        output.stderr.contains("not found") || output.stderr.contains("No such file"),
        "Should indicate file not found"
    );

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after migration feature implemented
async fn test_migrate_invalid_json() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Create invalid JSON file
    let invalid_path = harness.temp_dir.path().join("invalid.json");
    std::fs::write(&invalid_path, "{ invalid json }").unwrap();

    // Try to migrate
    let output = harness
        .run_command(&[
            "credentials",
            "migrate",
            "--from",
            invalid_path.to_str().unwrap(),
        ])
        .unwrap();

    assert!(!output.success, "Migration should fail for invalid JSON");
    assert!(
        output.stderr.contains("parse") || output.stderr.contains("invalid"),
        "Should indicate parse error"
    );

    harness.stop_daemon().unwrap();
}

// =============================================================================
// SECTION 5: Output Formatting Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_output_format_store() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    let output = harness
        .run_command(&["credentials", "store", "format_test", "value"])
        .unwrap();

    assert!(output.success);
    assert!(
        output.stdout.contains("✅") || output.stdout.contains("stored"),
        "Should have success indicator"
    );
    assert!(
        output.stdout.contains("format_test"),
        "Should include credential key"
    );

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_output_format_list() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store some credentials
    harness
        .run_command(&["credentials", "store", "key1", "value1"])
        .unwrap();
    harness
        .run_command(&["credentials", "store", "key2", "value2"])
        .unwrap();

    let output = harness.run_command(&["credentials", "list"]).unwrap();

    assert!(output.success);
    // Should have some formatting (emoji or header)
    assert!(
        output.stdout.contains("📋") || output.stdout.contains("Credentials"),
        "Should have formatted output"
    );

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_retrieve_output_only_value() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store credential
    harness
        .run_command(&["credentials", "store", "clean_key", "clean_value"])
        .unwrap();

    // Retrieve should output ONLY the value (for scripting)
    let output = harness
        .run_command(&["credentials", "retrieve", "clean_key"])
        .unwrap();

    assert!(output.success);
    assert_eq!(
        output.stdout.trim(),
        "clean_value",
        "Retrieve should output only the value for scripting"
    );
    assert!(
        !output.stdout.contains("✅") && !output.stdout.contains("retrieved"),
        "Should not have extra formatting in retrieve output"
    );

    harness.stop_daemon().unwrap();
}

// =============================================================================
// SECTION 6: Integration with Other Commands (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_status_shows_credentials() {
    let mut harness = CliTestHarness::new().unwrap();
    harness.start_daemon().await.unwrap();

    // Store some credentials
    harness
        .run_command(&["credentials", "store", "key1", "value1"])
        .unwrap();

    // Check status
    let output = harness.run_command(&["status"]).unwrap();

    assert!(output.success);
    // Status might show credential count or endpoint availability
    // Exact format depends on implementation

    harness.stop_daemon().unwrap();
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_cli_shutdown_preserves_credentials() {
    let mut harness = CliTestHarness::new().unwrap();
    let port1 = harness.start_daemon().await.unwrap();

    // Store credential
    harness
        .run_command(&["credentials", "store", "persist_key", "persist_value"])
        .unwrap();

    // Shutdown daemon
    harness.stop_daemon().unwrap();
    sleep(Duration::from_secs(1)).await;

    // Restart daemon (potentially on different port)
    harness.daemon_port = None;
    let port2 = harness.start_daemon().await.unwrap();

    println!("First daemon port: {}, Second: {}", port1, port2);

    // Credential should still exist
    let output = harness
        .run_command(&["credentials", "retrieve", "persist_key"])
        .unwrap();

    assert!(output.success, "Credentials should persist across restarts");
    assert_eq!(
        output.stdout.trim(),
        "persist_value",
        "Value should match original"
    );

    harness.stop_daemon().unwrap();
}
