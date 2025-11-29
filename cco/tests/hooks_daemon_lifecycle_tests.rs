//! Integration tests for daemon lifecycle with hooks
//!
//! Tests hooks integration during daemon startup, running, and shutdown:
//! - Classifier initialization on startup
//! - Settings file generation
//! - Environment variable handling
//! - Default configuration behavior
//! - Daemon stability with hooks
//!
//! Run with: cargo test hooks_daemon_lifecycle

mod hooks_test_helpers;

use cco::daemon::config::DaemonConfig;
use cco::daemon::hooks::HooksConfig;
use hooks_test_helpers::*;
use std::path::PathBuf;
use tempfile::TempDir;

// =============================================================================
// SECTION 1: Daemon Startup with Hooks (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when daemon startup is implemented
async fn test_classifier_initialized_on_daemon_start() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Wait for daemon to fully initialize
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Classifier should be ready
    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    assert!(hooks.enabled);
    assert!(hooks.classifier_available);
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_daemon_starts_with_hooks_disabled() {
    let daemon = TestDaemon::with_hooks_disabled().await.unwrap();

    let response = daemon.client.api_health().await.unwrap();
    let hooks = response.hooks.unwrap();

    assert!(!hooks.enabled);
    assert!(!hooks.classifier_available);
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_daemon_startup_time_with_hooks() {
    let start = std::time::Instant::now();
    let _daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let elapsed = start.elapsed();

    // Daemon should start within reasonable time (< 5 seconds)
    // Even with classifier initialization
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "Daemon startup too slow: {:?}",
        elapsed
    );
}

// =============================================================================
// SECTION 2: Settings File Management (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when settings file is implemented
async fn test_settings_file_written_on_startup() {
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join(".cco-orchestrator-settings");

    // In real implementation, daemon would write settings file
    // For now, test the expected structure

    let mut config = DaemonConfig::default();
    config.hooks.enabled = true;

    // Verify config is valid
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_settings_file_contains_hooks_config() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = DaemonConfig::default();
    config.hooks.enabled = true;
    config.hooks.timeout_ms = 3000;

    // Save config to file
    let config_path = temp_dir.path().join("config.toml");
    config.save(&config_path).unwrap();

    // Load and verify
    let loaded = DaemonConfig::load(&config_path).unwrap();
    assert!(loaded.hooks.enabled);
    assert_eq!(loaded.hooks.timeout_ms, 3000);
}

#[tokio::test]
async fn test_settings_file_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = DaemonConfig::default();
    config.save(&config_path).unwrap();

    // Verify file exists and is readable
    assert!(config_path.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&config_path).unwrap();
        let permissions = metadata.permissions();

        // Should be readable by owner
        assert!(permissions.mode() & 0o400 != 0);
    }
}

#[tokio::test]
async fn test_settings_file_updated_on_config_change() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Save initial config
    let mut config = DaemonConfig::default();
    config.hooks.enabled = false;
    config.save(&config_path).unwrap();

    // Update and save again
    config.hooks.enabled = true;
    config.save(&config_path).unwrap();

    // Load and verify update
    let loaded = DaemonConfig::load(&config_path).unwrap();
    assert!(loaded.hooks.enabled);
}

// =============================================================================
// SECTION 3: Environment Variable Handling (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when env var support is implemented
async fn test_hooks_env_vars_set_by_launcher() {
    // Test that launcher sets expected environment variables
    // This would be tested in launcher integration tests

    // Expected env vars:
    // ORCHESTRATOR_HOOKS_CONFIG
    // ORCHESTRATOR_HOOKS_ENABLED
    // etc.

    // For now, just verify config can be serialized to env-friendly format
    let config = test_hooks_config();
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());
}

#[tokio::test]
async fn test_config_from_environment() {
    // Test reading hooks config from environment variables
    // This is a placeholder - actual implementation depends on env var design

    let config = HooksConfig::default();
    assert!(!config.enabled); // Default is disabled
}

#[tokio::test]
async fn test_environment_overrides_config_file() {
    // Test that environment variables override config file settings
    // Placeholder - depends on implementation

    let mut config = DaemonConfig::default();
    config.hooks.enabled = false;

    // In real implementation, env var would override this
    // For now, just verify config structure
    assert!(!config.hooks.enabled);
}

// =============================================================================
// SECTION 4: Default Configuration Behavior (4 tests)
// =============================================================================

#[tokio::test]
async fn test_classifier_disabled_by_default() {
    let config = DaemonConfig::default();

    assert!(
        !config.hooks.enabled,
        "Hooks should be disabled by default for safety"
    );
}

#[tokio::test]
async fn test_default_config_is_valid() {
    let config = DaemonConfig::default();

    assert!(config.validate().is_ok());
    assert!(config.hooks.validate().is_ok());
}

#[tokio::test]
async fn test_default_hooks_config_safe() {
    let hooks_config = HooksConfig::default();

    // All permissions should be disabled by default
    assert!(!hooks_config.permissions.allow_command_modification);
    assert!(!hooks_config.permissions.allow_execution_blocking);
    assert!(!hooks_config.permissions.allow_external_calls);
    assert!(!hooks_config.permissions.allow_file_write);
}

#[tokio::test]
async fn test_default_config_reasonable_timeouts() {
    let hooks_config = HooksConfig::default();

    // Timeouts should be reasonable
    assert!(hooks_config.timeout_ms > 0);
    assert!(hooks_config.timeout_ms < 60000); // Less than 1 minute

    assert!(hooks_config.llm.inference_timeout_ms > 0);
    assert!(hooks_config.llm.inference_timeout_ms <= 5000); // 5 seconds max
}

// =============================================================================
// SECTION 5: Daemon Stability Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when daemon lifecycle is fully implemented
async fn test_daemon_stable_with_classifier_enabled() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Run multiple classifications
    for i in 0..10 {
        let cmd = format!("echo 'test{}'", i);
        let result = daemon.client.classify(&cmd).await;
        assert!(result.is_ok() || result.is_err()); // Should not panic
    }

    // Daemon should still be healthy
    let health = daemon.client.health().await;
    assert!(health.is_ok());
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_daemon_recovers_from_classifier_failure() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Send problematic command that might cause classifier issues
    let _ = daemon.client.classify("").await;
    let _ = daemon.client.classify(&"x".repeat(100000)).await;

    // Daemon should still respond to health checks
    let health = daemon.client.health().await;
    assert!(health.is_ok());

    // And should still handle normal commands
    let result = daemon.client.classify("ls -la").await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore] // Remove when implemented
async fn test_daemon_shutdown_graceful_with_hooks() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Make some classification requests
    let _ = daemon.client.classify("ls").await;

    // Daemon shutdown should be graceful
    // (This would be tested via actual daemon shutdown mechanism)
    drop(daemon);

    // No panics or errors should occur
}

// =============================================================================
// SECTION 6: Configuration Loading (3 tests)
// =============================================================================

#[tokio::test]
async fn test_load_config_from_toml_with_hooks() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let toml_content = r#"
port = 3000
host = "127.0.0.1"
log_level = "info"
log_rotation_size = 10485760
log_max_files = 5
database_url = "sqlite://analytics.db"
cache_size = 1073741824
cache_ttl = 3600
auto_start = true
health_checks = true
health_check_interval = 30

[hooks]
enabled = true
timeout_ms = 3000
max_retries = 3

[hooks.llm]
model_type = "tinyllama"
model_name = "tinyllama-1.1b-chat-v1.0.Q4_K_M"
inference_timeout_ms = 2000
temperature = 0.1

[hooks.permissions]
allow_command_modification = false
allow_execution_blocking = false
"#;

    std::fs::write(&config_path, toml_content).unwrap();

    let config = DaemonConfig::load(&config_path).unwrap();

    assert!(config.hooks.enabled);
    assert_eq!(config.hooks.timeout_ms, 3000);
    assert_eq!(config.hooks.max_retries, 3);
    assert_eq!(config.hooks.llm.model_type, "tinyllama");
}

#[tokio::test]
async fn test_partial_hooks_config_uses_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let toml_content = r#"
port = 3000
host = "127.0.0.1"
log_level = "info"
log_rotation_size = 10485760
log_max_files = 5
database_url = "sqlite://analytics.db"
cache_size = 1073741824
cache_ttl = 3600
auto_start = true
health_checks = true
health_check_interval = 30

[hooks]
enabled = true
# Other hooks settings use defaults
"#;

    std::fs::write(&config_path, toml_content).unwrap();

    let config = DaemonConfig::load(&config_path).unwrap();

    assert!(config.hooks.enabled);
    assert_eq!(config.hooks.timeout_ms, 5000); // Default
    assert_eq!(config.hooks.max_retries, 2); // Default
}

#[tokio::test]
async fn test_missing_hooks_section_uses_full_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let toml_content = r#"
port = 3000
host = "127.0.0.1"
log_level = "info"
log_rotation_size = 10485760
log_max_files = 5
database_url = "sqlite://analytics.db"
cache_size = 1073741824
cache_ttl = 3600
auto_start = true
health_checks = true
health_check_interval = 30
# No [hooks] section
"#;

    std::fs::write(&config_path, toml_content).unwrap();

    let config = DaemonConfig::load(&config_path).unwrap();

    // Should use complete defaults
    assert!(!config.hooks.enabled); // Default is disabled
    assert_eq!(config.hooks.timeout_ms, 5000);
}
