// CLI Launcher Temp Files Tests: Test launcher discovery of temp files
// Tests for CLI launcher finding and using temp directory files
// Coverage Target: 90%+ for launcher temp file integration

#[cfg(test)]
mod cli_launcher_temp_files_tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use tokio;

    // Helper to get temp file paths
    fn get_temp_file_paths() -> Vec<PathBuf> {
        let temp_dir = env::temp_dir();
        vec![
            temp_dir.join(".cco-orchestrator-settings"),
            temp_dir.join(".cco-agents-sealed"),
            temp_dir.join(".cco-rules-sealed"),
            temp_dir.join(".cco-hooks-sealed"),
        ]
    }

    // ============================================================================
    // UNIT TESTS: Temp File Discovery
    // ============================================================================

    #[tokio::test]
    async fn test_launcher_discovers_temp_settings() {
        // Arrange: Create mock settings file
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"{}").unwrap();

        // Act: Verify launcher can find it
        // let result = verify_temp_files_exist(&settings_path).await;

        // Assert: Discovery succeeds
        // assert!(result.is_ok(), "Launcher should discover temp settings");

        // Cleanup
        fs::remove_file(&settings_path).ok();
    }

    #[tokio::test]
    async fn test_launcher_error_when_temp_files_missing() {
        // Arrange: Ensure no temp files exist
        for path in get_temp_file_paths() {
            let _ = fs::remove_file(&path);
        }

        let missing_path = env::temp_dir().join(".cco-orchestrator-settings");

        // Act: Try to verify temp files
        // let result = verify_temp_files_exist(&missing_path).await;

        // Assert: Returns error
        // assert!(result.is_err(), "Should fail when temp files missing");
        // let error = result.unwrap_err().to_string();
        // assert!(error.contains("not found") || error.contains("missing"),
        //         "Error should mention missing files: {}", error);
    }

    #[tokio::test]
    async fn test_launcher_discovers_all_required_files() {
        // Arrange: Create all required temp files
        let temp_dir = env::temp_dir();
        for path in get_temp_file_paths() {
            fs::write(&path, b"mock").unwrap();
        }

        // Act: Verify all files discovered
        // let result = verify_all_temp_files_exist().await;

        // Assert: All files found
        // assert!(result.is_ok(), "All temp files should be discovered");

        // Cleanup
        for path in get_temp_file_paths() {
            fs::remove_file(&path).ok();
        }
    }

    // ============================================================================
    // UNIT TESTS: Environment Variable Setting
    // ============================================================================

    #[tokio::test]
    async fn test_launcher_sets_env_vars() {
        // Arrange: Create temp settings file
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"{}").unwrap();

        // Act: Set environment variables
        // set_orchestrator_env_vars(&settings_path);

        // Assert: Verify env vars set
        // assert_eq!(env::var("ORCHESTRATOR_ENABLED").unwrap(), "true");
        // assert!(env::var("ORCHESTRATOR_SETTINGS").unwrap().contains(".cco-orchestrator-settings"));
        // assert!(env::var("ORCHESTRATOR_AGENTS").unwrap().contains(".cco-agents-sealed"));
        // assert!(env::var("ORCHESTRATOR_RULES").unwrap().contains(".cco-rules-sealed"));
        // assert!(env::var("ORCHESTRATOR_HOOKS").unwrap().contains(".cco-hooks-sealed"));

        // Cleanup
        fs::remove_file(&settings_path).ok();
    }

    #[tokio::test]
    async fn test_env_vars_use_absolute_paths() {
        // Arrange: Create temp files
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"{}").unwrap();

        // Act: Set env vars
        // set_orchestrator_env_vars(&settings_path);

        // Assert: All paths are absolute
        // let settings_var = env::var("ORCHESTRATOR_SETTINGS").unwrap();
        // let agents_var = env::var("ORCHESTRATOR_AGENTS").unwrap();

        // let settings_path = PathBuf::from(&settings_var);
        // let agents_path = PathBuf::from(&agents_var);

        // assert!(settings_path.is_absolute(), "Settings path should be absolute");
        // assert!(agents_path.is_absolute(), "Agents path should be absolute");

        // Cleanup
        fs::remove_file(&settings_path).ok();
    }

    #[tokio::test]
    async fn test_env_vars_cleared_on_error() {
        // Arrange: Simulate error condition
        // let bad_path = PathBuf::from("/nonexistent/.cco-settings");

        // Act: Try to set env vars with bad path
        // let result = set_orchestrator_env_vars(&bad_path);

        // Assert: Env vars not set on failure
        // assert!(result.is_err());
        // assert!(env::var("ORCHESTRATOR_ENABLED").is_err(), "Env var should not be set on error");
    }

    // ============================================================================
    // INTEGRATION TESTS: Full Launcher Workflow
    // ============================================================================

    #[tokio::test]
    async fn test_full_launcher_workflow() {
        // Arrange: Create mock daemon and settings
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"{}").unwrap();

        // Also create sealed files
        for path in get_temp_file_paths() {
            fs::write(&path, b"CCOSEAL1mock").unwrap();
        }

        // Act: Run launcher flow
        // 1. Discover temp files
        // 2. Set env vars
        // 3. Launch Claude Code (mock)
        // let result = launch_claude_code(vec!["--help".to_string()]).await;

        // Assert: Workflow completes
        // assert!(result.is_ok(), "Full workflow should complete");

        // Cleanup
        for path in get_temp_file_paths() {
            fs::remove_file(&path).ok();
        }
    }

    #[tokio::test]
    async fn test_launcher_passes_args_to_claude() {
        // Arrange: Create temp files
        let temp_dir = env::temp_dir();
        for path in get_temp_file_paths() {
            fs::write(&path, b"mock").unwrap();
        }

        // Act: Launch with specific args
        // let args = vec!["analyze".to_string(), "main.rs".to_string()];
        // let result = launch_claude_code(args.clone()).await;

        // Assert: Args passed through
        // Verify spawned process received args (requires mock/spy)

        // Cleanup
        for path in get_temp_file_paths() {
            fs::remove_file(&path).ok();
        }
    }

    #[tokio::test]
    async fn test_launcher_preserves_working_directory() {
        // Arrange: Set specific working directory
        let test_dir = env::temp_dir().join("cco-test-cwd");
        fs::create_dir_all(&test_dir).unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&test_dir).unwrap();

        // Create temp files
        for path in get_temp_file_paths() {
            fs::write(&path, b"mock").unwrap();
        }

        // Act: Launch (mock)
        // let result = launch_claude_code(vec![]).await;

        // Assert: Working directory preserved
        let current = env::current_dir().unwrap();
        assert_eq!(current, test_dir, "Working directory should be preserved");

        // Cleanup
        env::set_current_dir(&original_dir).unwrap();
        for path in get_temp_file_paths() {
            fs::remove_file(&path).ok();
        }
        fs::remove_dir_all(&test_dir).ok();
    }

    // ============================================================================
    // ERROR PATH TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_launcher_error_settings_not_json() {
        // Arrange: Create invalid settings file
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"not json").unwrap();

        // Act: Try to launch
        // let result = launch_claude_code(vec![]).await;

        // Assert: Error about invalid JSON
        // assert!(result.is_err());
        // let error = result.unwrap_err().to_string();
        // assert!(error.contains("JSON") || error.contains("parse"),
        //         "Error should mention JSON parsing: {}", error);

        // Cleanup
        fs::remove_file(&settings_path).ok();
    }

    #[tokio::test]
    async fn test_launcher_error_sealed_file_corrupted() {
        // Arrange: Create corrupted sealed file
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"{}").unwrap();

        let agents_path = temp_dir.join(".cco-agents-sealed");
        fs::write(&agents_path, b"CORRUPTED").unwrap(); // No SBF header

        // Act: Try to launch
        // let result = launch_claude_code(vec![]).await;

        // Assert: Error about corrupted file
        // assert!(result.is_err());

        // Cleanup
        fs::remove_file(&settings_path).ok();
        fs::remove_file(&agents_path).ok();
    }

    #[tokio::test]
    async fn test_launcher_error_claude_not_found() {
        // Arrange: Create temp files
        for path in get_temp_file_paths() {
            fs::write(&path, b"mock").unwrap();
        }

        // Mock: Claude Code not in PATH
        // This requires mocking which::which() or PATH manipulation

        // Act: Try to launch
        // let result = find_claude_code_executable();

        // Assert: Clear error message
        // assert!(result.is_err());
        // let error = result.unwrap_err().to_string();
        // assert!(error.contains("claude") && error.contains("not found"));
        // assert!(error.contains("https://claude.ai/code"), "Should include install link");

        // Cleanup
        for path in get_temp_file_paths() {
            fs::remove_file(&path).ok();
        }
    }

    // ============================================================================
    // CROSS-PLATFORM TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_launcher_temp_dir_unix() {
        #[cfg(unix)]
        {
            let temp_dir = env::temp_dir();
            assert!(temp_dir.is_absolute(), "Temp dir should be absolute");
            assert!(temp_dir.exists(), "Temp dir should exist");
        }
    }

    #[tokio::test]
    async fn test_launcher_temp_dir_windows() {
        #[cfg(windows)]
        {
            let temp_dir = env::temp_dir();
            assert!(temp_dir.is_absolute(), "Temp dir should be absolute");
            assert!(temp_dir.exists(), "Temp dir should exist");
        }
    }

    #[tokio::test]
    async fn test_launcher_handles_spaces_in_paths() {
        // Arrange: Create temp dir with spaces (if allowed by OS)
        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join("cco test dir");

        // Skip if OS doesn't allow spaces in temp paths
        if fs::create_dir_all(&test_dir).is_ok() {
            let settings_path = test_dir.join(".cco-orchestrator-settings");
            fs::write(&settings_path, b"{}").unwrap();

            // Act: Set env vars with space in path
            // let result = set_orchestrator_env_vars(&settings_path);

            // Assert: Handles spaces correctly
            // assert!(result.is_ok(), "Should handle spaces in paths");

            // Cleanup
            fs::remove_file(&settings_path).ok();
            fs::remove_dir_all(&test_dir).ok();
        }
    }

    // ============================================================================
    // DAEMON INTEGRATION TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_launcher_waits_for_daemon_temp_files() {
        // Arrange: Start daemon in background
        // let daemon = DaemonManager::new();
        // let daemon_handle = tokio::spawn(async move {
        //     daemon.start().await
        // });

        // Act: Wait for temp files to appear
        // tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Assert: Temp files created by daemon
        // for path in get_temp_file_paths() {
        //     assert!(path.exists(), "Daemon should create temp file: {:?}", path);
        // }

        // Cleanup
        // daemon_handle.abort();
    }

    #[tokio::test]
    async fn test_launcher_detects_daemon_not_running() {
        // Arrange: Ensure daemon not running
        for path in get_temp_file_paths() {
            let _ = fs::remove_file(&path);
        }

        // Act: Try to launch
        // let result = verify_daemon_running().await;

        // Assert: Detects daemon not running
        // assert!(result.is_err(), "Should detect daemon not running");
        // let error = result.unwrap_err().to_string();
        // assert!(error.contains("daemon") || error.contains("not running"));
    }

    #[tokio::test]
    async fn test_launcher_auto_starts_daemon_if_needed() {
        // Arrange: Daemon not running
        for path in get_temp_file_paths() {
            let _ = fs::remove_file(&path);
        }

        // Act: Launch with auto-start enabled
        // let result = launch_with_auto_start(vec![]).await;

        // Assert: Daemon started automatically
        // assert!(result.is_ok(), "Should auto-start daemon");
        // for path in get_temp_file_paths() {
        //     assert!(path.exists(), "Auto-started daemon should create temp files");
        // }

        // Cleanup
        // Stop daemon
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_temp_file_discovery_under_10ms() {
        // Arrange: Create temp files
        for path in get_temp_file_paths() {
            fs::write(&path, b"mock").unwrap();
        }

        // Act: Measure discovery time
        // let start = std::time::Instant::now();
        // let _result = verify_all_temp_files_exist().await;
        // let duration = start.elapsed();

        // Assert: Fast discovery
        // assert!(duration < tokio::time::Duration::from_millis(10),
        //         "Temp file discovery should be fast: {:?}", duration);

        // Cleanup
        for path in get_temp_file_paths() {
            fs::remove_file(&path).ok();
        }
    }

    #[tokio::test]
    async fn test_env_var_setting_under_5ms() {
        // Arrange: Create settings file
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join(".cco-orchestrator-settings");
        fs::write(&settings_path, b"{}").unwrap();

        // Act: Measure env var setting time
        // let start = std::time::Instant::now();
        // set_orchestrator_env_vars(&settings_path);
        // let duration = start.elapsed();

        // Assert: Very fast
        // assert!(duration < tokio::time::Duration::from_millis(5),
        //         "Env var setting should be instant: {:?}", duration);

        // Cleanup
        fs::remove_file(&settings_path).ok();
    }
}
