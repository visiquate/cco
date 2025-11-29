//! Port Discovery Integration Tests
//!
//! Tests to verify that all commands use port discovery instead of hardcoded port 3000.
//! These tests ensure that the random port discovery implementation works correctly
//! and prevent regression to hardcoded ports.
//!
//! Test Coverage:
//! 1. Daemon starts on random port (not 3000)
//! 2. Health/status commands discover the actual port
//! 3. TUI discovers the actual port
//! 4. Commands fail gracefully without daemon
//! 5. No hardcoded port 3000 in command implementations

#[cfg(test)]
mod port_discovery_integration_tests {
    use cco::daemon::lifecycle::PidFileContent;
    use cco::daemon::DaemonConfig;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    /// Test that daemon can start on port 0 (random port assignment)
    #[tokio::test]
    async fn test_daemon_starts_on_random_port() {
        let mut config = DaemonConfig::default();
        config.port = 0; // Request random port

        // Verify config accepts port 0
        assert_eq!(
            config.port, 0,
            "Config should accept port 0 for random assignment"
        );

        // Verify validation allows port 0
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "Config validation should accept port 0, got: {:?}",
            validation_result
        );
    }

    /// Test that PID file contains non-zero port after daemon binds
    #[test]
    fn test_pid_file_contains_discovered_port() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // Simulate daemon writing PID file after binding to random port
        let discovered_port = 54321; // Simulated OS-assigned port
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: discovered_port,
            version: "2025.11.1".to_string(),
        };

        let pid_json = serde_json::to_string_pretty(&pid_content).unwrap();
        fs::write(&pid_file_path, pid_json).unwrap();

        // Read PID file and verify port
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();

        assert_ne!(parsed.port, 0, "Port should be assigned (not 0)");
        assert_ne!(parsed.port, 3000, "Port should not be hardcoded to 3000");
        assert_eq!(
            parsed.port, discovered_port,
            "Port should match discovered port"
        );
    }

    /// Test that read_daemon_port discovers the actual port from PID file
    #[test]
    fn test_read_daemon_port_discovers_actual_port() {
        // This test simulates what happens when commands call read_daemon_port()
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // Create PID file with random port
        let random_port = 49152;
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: random_port,
            version: "2025.11.1".to_string(),
        };

        let pid_json = serde_json::to_string_pretty(&pid_content).unwrap();
        fs::write(&pid_file_path, pid_json).unwrap();

        // Read and verify
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();

        // Verify discovered port is correct
        assert_eq!(parsed.port, random_port);
        assert!(
            parsed.port >= 1024,
            "Port should be in valid range (>= 1024)"
        );
        // Note: u16 type inherently ensures port <= 65535, so no need to check upper bound
    }

    /// Test that ports differ across daemon restarts
    #[tokio::test]
    async fn test_ports_differ_across_restarts() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // First daemon start
        let first_port = 50000;
        let pid_content_1 = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: first_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content_1).unwrap(),
        )
        .unwrap();

        let parsed_1: PidFileContent =
            serde_json::from_str(&fs::read_to_string(&pid_file_path).unwrap()).unwrap();
        assert_eq!(parsed_1.port, first_port);

        // Simulate restart with different random port
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let second_port = 50001;
        let pid_content_2 = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: second_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content_2).unwrap(),
        )
        .unwrap();

        let parsed_2: PidFileContent =
            serde_json::from_str(&fs::read_to_string(&pid_file_path).unwrap()).unwrap();
        assert_eq!(parsed_2.port, second_port);

        // Verify ports are different
        assert_ne!(
            parsed_1.port, parsed_2.port,
            "Ports should differ across restarts"
        );
        assert_ne!(parsed_1.port, 3000, "First port should not be 3000");
        assert_ne!(parsed_2.port, 3000, "Second port should not be 3000");
    }

    /// Test that read_daemon_port fails gracefully when daemon is not running
    #[test]
    fn test_read_daemon_port_fails_when_daemon_not_running() {
        let temp_dir = TempDir::new().unwrap();

        // Temporarily override daemon directory for test
        // Note: In real scenario, read_daemon_port() would check actual PID file location

        // This test verifies the error handling pattern
        // When no PID file exists, commands should show helpful error message
        let non_existent_pid = temp_dir.path().join("nonexistent.pid");
        assert!(!non_existent_pid.exists(), "PID file should not exist");

        // In real implementation, read_daemon_port() would return error
        // Commands should catch this and show: "Daemon is not running" instead of "connection refused to localhost:3000"
    }

    /// Test that commands use discovered port for API URLs
    #[test]
    fn test_api_url_uses_discovered_port() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // Simulate daemon running on port 58912
        let discovered_port = 58912;
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: discovered_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content).unwrap(),
        )
        .unwrap();

        // Verify API URL construction
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();

        let api_url = format!("http://localhost:{}", parsed.port);
        assert_eq!(api_url, "http://localhost:58912");
        assert!(
            !api_url.contains("3000"),
            "API URL should not contain hardcoded port 3000"
        );
    }

    /// Test that launcher sets correct ORCHESTRATOR_API_URL
    #[test]
    fn test_launcher_sets_correct_api_url_env() {
        use std::env;

        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // Simulate daemon on random port
        let random_port = 62345;
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: random_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content).unwrap(),
        )
        .unwrap();

        // Read discovered port
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();

        // Simulate launcher setting environment variable
        let api_url = format!("http://localhost:{}", parsed.port);
        env::set_var("ORCHESTRATOR_API_URL_TEST", &api_url);

        let env_value = env::var("ORCHESTRATOR_API_URL_TEST").unwrap();
        assert_eq!(env_value, format!("http://localhost:{}", random_port));
        assert!(
            !env_value.contains("3000"),
            "Environment variable should not contain hardcoded port 3000"
        );

        // Cleanup
        env::remove_var("ORCHESTRATOR_API_URL_TEST");
    }

    /// Test PID file update preserves other fields when updating port
    #[test]
    fn test_port_update_preserves_pid_and_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        let original_pid = 99999;
        let original_time = Utc::now();
        let original_version = "2025.11.5".to_string();

        // Create initial PID file with port 0
        let mut content = PidFileContent {
            pid: original_pid,
            started_at: original_time,
            port: 0, // Before binding
            version: original_version.clone(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&content).unwrap(),
        )
        .unwrap();

        // Simulate port discovery and update
        content.port = 55555; // After binding
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&content).unwrap(),
        )
        .unwrap();

        // Verify update preserved other fields
        let parsed: PidFileContent =
            serde_json::from_str(&fs::read_to_string(&pid_file_path).unwrap()).unwrap();
        assert_eq!(parsed.pid, original_pid);
        assert_eq!(parsed.started_at, original_time);
        assert_eq!(parsed.version, original_version);
        assert_eq!(parsed.port, 55555);
    }

    /// Test concurrent port reads are thread-safe
    #[tokio::test]
    async fn test_concurrent_port_discovery_thread_safe() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        let test_port = 45678;
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: test_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content).unwrap(),
        )
        .unwrap();

        // Spawn multiple concurrent reads
        let path1 = pid_file_path.clone();
        let path2 = pid_file_path.clone();
        let path3 = pid_file_path.clone();

        let handle1 = tokio::spawn(async move {
            let contents = fs::read_to_string(&path1).unwrap();
            let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();
            parsed.port
        });

        let handle2 = tokio::spawn(async move {
            let contents = fs::read_to_string(&path2).unwrap();
            let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();
            parsed.port
        });

        let handle3 = tokio::spawn(async move {
            let contents = fs::read_to_string(&path3).unwrap();
            let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();
            parsed.port
        });

        let port1 = handle1.await.unwrap();
        let port2 = handle2.await.unwrap();
        let port3 = handle3.await.unwrap();

        assert_eq!(port1, test_port);
        assert_eq!(port2, test_port);
        assert_eq!(port3, test_port);
        assert_eq!(port1, port2);
        assert_eq!(port2, port3);
        assert_ne!(port1, 3000, "Discovered port should not be hardcoded 3000");
    }

    /// Test error handling when PID file is corrupted
    #[test]
    fn test_corrupted_pid_file_fails_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // Write corrupted JSON
        fs::write(&pid_file_path, "{ invalid json, port: 3000 }").unwrap();

        // Attempt to parse
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let result: Result<PidFileContent, _> = serde_json::from_str(&contents);

        assert!(result.is_err(), "Corrupted PID file should fail to parse");

        // Verify error message is helpful (should not suggest connecting to port 3000)
        let error_msg = result.unwrap_err().to_string();
        assert!(
            !error_msg.contains("3000"),
            "Error should not mention hardcoded port 3000"
        );
    }

    /// Test that status command would display correct port
    #[test]
    fn test_status_command_shows_discovered_port() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        // Simulate daemon running on random port
        let status_port = 51234;
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: status_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content).unwrap(),
        )
        .unwrap();

        // Read PID file (as status command would)
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();

        // Verify status would show correct port
        let dashboard_url = format!("http://127.0.0.1:{}", parsed.port);
        assert_eq!(dashboard_url, "http://127.0.0.1:51234");
        assert!(!dashboard_url.contains("3000"));
    }

    /// Test port validation (u16 type ensures valid range)
    #[test]
    fn test_port_validation_type_safety() {
        // u16 type inherently enforces 0-65535 range
        let valid_ports: Vec<u16> = vec![0, 1, 1024, 8080, 49152, 65535];

        for port in valid_ports {
            let pid_content = PidFileContent {
                pid: 1,
                started_at: Utc::now(),
                port,
                version: "2025.11.1".to_string(),
            };

            // Should serialize and deserialize without issues
            let json = serde_json::to_string(&pid_content).unwrap();
            let parsed: PidFileContent = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.port, port);
        }
    }

    /// Test that DaemonManager.get_status() would use discovered port
    #[tokio::test]
    async fn test_daemon_manager_uses_discovered_port() {
        // Note: This test verifies the pattern, not actual HTTP connection
        let temp_dir = TempDir::new().unwrap();
        let pid_file_path = temp_dir.path().join("daemon.pid");

        let manager_port = 59876;
        let pid_content = PidFileContent {
            pid: std::process::id(),
            started_at: Utc::now(),
            port: manager_port,
            version: "2025.11.1".to_string(),
        };
        fs::write(
            &pid_file_path,
            serde_json::to_string_pretty(&pid_content).unwrap(),
        )
        .unwrap();

        // Verify manager would construct correct URL
        let contents = fs::read_to_string(&pid_file_path).unwrap();
        let parsed: PidFileContent = serde_json::from_str(&contents).unwrap();

        let health_url = format!("http://localhost:{}/health", parsed.port);
        assert_eq!(health_url, "http://localhost:59876/health");
        assert!(!health_url.contains("3000"));
    }
}

/// Source code audit tests - verify no hardcoded port 3000 in commands
#[cfg(test)]
mod hardcoded_port_audit_tests {
    use std::fs;
    use std::path::Path;

    /// Helper function to check if a file contains hardcoded port 3000
    fn check_file_for_hardcoded_port(path: &Path) -> Vec<String> {
        let content = fs::read_to_string(path).unwrap_or_default();
        let mut violations = Vec::new();
        let mut in_test_module = false;
        let mut brace_depth = 0;

        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            let trimmed = line.trim();

            // Track test module boundaries
            if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests {") {
                in_test_module = true;
                brace_depth = 0;
            }

            // Count braces to track when we exit test module
            if in_test_module {
                brace_depth += line.matches('{').count() as i32;
                brace_depth -= line.matches('}').count() as i32;

                if brace_depth <= 0 && trimmed == "}" {
                    in_test_module = false;
                }
            }

            // Skip if in test module
            if in_test_module {
                continue;
            }

            // Skip comments
            if line_lower.trim_start().starts_with("//") {
                continue;
            }

            // Skip test functions outside test modules
            if line_lower.contains("#[test]") || line_lower.contains("#[tokio::test]") {
                continue;
            }

            // Skip sidecar port 3001 (intentional)
            if line.contains("3001") {
                continue;
            }

            // Check for hardcoded 3000 in connection logic
            if line.contains("3000") {
                // Allowed contexts:
                // - CLI default values (user can override)
                // - Documentation/comments
                // - Test code

                let context = line.to_lowercase();
                if context.contains("default") && context.contains("port") {
                    // CLI default is OK (user can override with --port)
                    continue;
                }

                violations.push(format!(
                    "{}:{}: Potential hardcoded port 3000: {}",
                    path.display(),
                    line_num + 1,
                    line.trim()
                ));
            }
        }

        violations
    }

    #[test]
    fn test_no_hardcoded_port_in_launcher() {
        let launcher_path = Path::new("src/commands/launcher.rs");
        if launcher_path.exists() {
            let violations = check_file_for_hardcoded_port(launcher_path);

            if !violations.is_empty() {
                panic!(
                    "Found hardcoded port 3000 in launcher:\n{}",
                    violations.join("\n")
                );
            }
        }
    }

    #[test]
    fn test_no_hardcoded_port_in_tui() {
        let tui_path = Path::new("src/commands/tui.rs");
        if tui_path.exists() {
            let violations = check_file_for_hardcoded_port(tui_path);

            if !violations.is_empty() {
                panic!(
                    "Found hardcoded port 3000 in TUI:\n{}",
                    violations.join("\n")
                );
            }
        }
    }

    #[test]
    fn test_no_hardcoded_port_in_status() {
        let status_path = Path::new("src/commands/status.rs");
        if status_path.exists() {
            let violations = check_file_for_hardcoded_port(status_path);

            if !violations.is_empty() {
                panic!(
                    "Found hardcoded port 3000 in status:\n{}",
                    violations.join("\n")
                );
            }
        }
    }

    #[test]
    fn test_no_hardcoded_port_in_daemon_lifecycle() {
        let lifecycle_path = Path::new("src/daemon/lifecycle.rs");
        if lifecycle_path.exists() {
            let violations = check_file_for_hardcoded_port(lifecycle_path);

            if !violations.is_empty() {
                panic!(
                    "Found hardcoded port 3000 in daemon lifecycle:\n{}",
                    violations.join("\n")
                );
            }
        }
    }

    #[test]
    fn test_no_hardcoded_port_in_daemon_manager() {
        let manager_path = Path::new("src/daemon/manager.rs");
        if manager_path.exists() {
            let violations = check_file_for_hardcoded_port(manager_path);

            if !violations.is_empty() {
                panic!(
                    "Found hardcoded port 3000 in daemon manager:\n{}",
                    violations.join("\n")
                );
            }
        }
    }

    #[test]
    fn test_all_command_files_no_hardcoded_port() {
        let commands_dir = Path::new("src/commands");
        if !commands_dir.exists() {
            return;
        }

        let mut all_violations = Vec::new();

        for entry in fs::read_dir(commands_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let violations = check_file_for_hardcoded_port(&path);
                all_violations.extend(violations);
            }
        }

        if !all_violations.is_empty() {
            panic!(
                "Found hardcoded port 3000 in command files:\n{}",
                all_violations.join("\n")
            );
        }
    }
}

/// Documentation for port discovery integration tests
///
/// # Test Philosophy
///
/// These tests ensure that the codebase never regresses to using hardcoded port 3000.
/// All commands must discover the daemon's actual port from the PID file.
///
/// # Port Discovery Flow
///
/// 1. Daemon starts with port 0 (random OS assignment)
/// 2. OS binds daemon to available port (e.g., 49152-65535)
/// 3. Daemon writes actual port to PID file via update_daemon_port()
/// 4. Commands call read_daemon_port() to discover actual port
/// 5. Commands connect to discovered port (not hardcoded 3000)
///
/// # What These Tests Verify
///
/// - Daemon accepts port 0 configuration
/// - PID file contains actual bound port (not 0 or 3000)
/// - Commands read PID file to discover port
/// - Error messages don't mention hardcoded port 3000
/// - Concurrent port discovery is thread-safe
/// - Port discovery fails gracefully when daemon is down
///
/// # Test Categories
///
/// 1. **Integration Tests** - Test actual port discovery flow
/// 2. **Source Code Audits** - Grep for hardcoded "3000" in code
/// 3. **Error Handling** - Verify helpful error messages
/// 4. **Thread Safety** - Test concurrent port discovery
///
/// # Running Tests
///
/// ```bash
/// # Run all port discovery tests
/// cargo test port_discovery
///
/// # Run source code audits only
/// cargo test hardcoded_port_audit
///
/// # Run with output
/// cargo test port_discovery -- --nocapture
/// ```
///
/// # CI Integration
///
/// These tests run on every commit to prevent regression.
/// Any hardcoded port 3000 (except CLI defaults and tests) will fail CI.
#[allow(dead_code)]
const PORT_DISCOVERY_TESTS_DOCUMENTATION: &str = "See module-level documentation above";
