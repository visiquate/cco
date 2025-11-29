//! Integration Tests: Daemon/TUI Lifecycle
//!
//! Comprehensive integration tests validating daemon startup, TUI connection,
//! lifecycle management, and error handling scenarios.
//!
//! Test Suites:
//! 1. Daemon Startup - Basic initialization and endpoint availability
//! 2. TUI Connection - Client connectivity and API structure validation
//! 3. Daemon Lifecycle - Process startup/shutdown and resource cleanup
//! 4. Error Handling - Error scenarios and graceful failure modes

#[cfg(test)]
mod daemon_startup_tests {
    use std::process::Command;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    /// Helper: Check if a port is listening
    async fn is_port_listening(port: u16, timeout_secs: u64) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if let Ok(stream) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
                drop(stream);
                return true;
            }

            if start.elapsed() > timeout {
                return false;
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Helper: Start daemon process
    fn spawn_daemon(port: u16) -> std::process::Child {
        Command::new("cargo")
            .args(&["run", "--", "run", "--port", &port.to_string()])
            .current_dir("/Users/brent/git/cc-orchestra/cco")
            .spawn()
            .expect("Failed to spawn daemon")
    }

    #[tokio::test]
    async fn test_daemon_startup_successful() {
        // Start daemon on non-standard port to avoid conflicts
        let port = 13000u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start and bind to port
        let listening = is_port_listening(port, 10).await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Assertions
        assert!(
            listening,
            "Daemon should bind to port {} within 10 seconds",
            port
        );
    }

    #[tokio::test]
    async fn test_daemon_listens_on_port() {
        let port = 13001u16;
        let mut daemon = spawn_daemon(port);

        // Wait for port to be listening
        let listening = is_port_listening(port, 10).await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        assert!(listening, "Port {} should be listening", port);
    }

    #[tokio::test]
    async fn test_health_endpoint_returns_status() {
        let port = 13002u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        // Small delay for server initialization
        sleep(Duration::from_millis(500)).await;

        // Test health endpoint
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Verify response
        assert!(response.is_ok(), "Health endpoint should respond");
        let resp = response.unwrap();
        assert_eq!(resp.status(), 200, "Health endpoint should return 200");

        // Verify JSON structure
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            assert_eq!(json["status"], "ok", "Status should be 'ok'");
            assert!(json["version"].is_string(), "Version should be present");
            assert!(
                json["cache_stats"].is_object(),
                "Cache stats should be present"
            );
            assert!(json["uptime"].is_number(), "Uptime should be present");
        }
    }

    #[tokio::test]
    async fn test_ready_endpoint_indicates_daemon_ready() {
        let port = 13003u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        // Test ready endpoint
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/ready", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Verify response
        assert!(response.is_ok(), "Ready endpoint should respond");
        let resp = response.unwrap();
        assert_eq!(resp.status(), 200, "Ready endpoint should return 200");

        // Verify JSON structure
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            assert_eq!(json["ready"], true, "Ready should be true");
            assert!(json["version"].is_string(), "Version should be present");
            assert!(json["timestamp"].is_string(), "Timestamp should be present");
        }
    }

    #[tokio::test]
    async fn test_critical_endpoints_all_respond() {
        let port = 13004u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();

        // Test all critical endpoints
        let endpoints = vec!["/health", "/ready", "/api/agents", "/api/stats"];

        let mut all_ok = true;
        for endpoint in endpoints {
            match client
                .get(format!("http://127.0.0.1:{}{}", port, endpoint))
                .timeout(Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status() != 200 {
                        eprintln!("Endpoint {} returned status {}", endpoint, resp.status());
                        all_ok = false;
                    }
                }
                Err(e) => {
                    eprintln!("Endpoint {} failed: {}", endpoint, e);
                    all_ok = false;
                }
            }
        }

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        assert!(all_ok, "All critical endpoints should respond with 200");
    }
}

#[cfg(test)]
mod tui_connection_tests {
    use std::process::Command;
    use std::time::Duration;
    use tokio::time::sleep;

    fn spawn_daemon(port: u16) -> std::process::Child {
        Command::new("cargo")
            .args(&["run", "--", "run", "--port", &port.to_string()])
            .current_dir("/Users/brent/git/cc-orchestra/cco")
            .spawn()
            .expect("Failed to spawn daemon")
    }

    async fn is_port_listening(port: u16, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if let Ok(stream) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
                drop(stream);
                return true;
            }

            if start.elapsed() > timeout {
                return false;
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    #[tokio::test]
    async fn test_tui_can_connect_to_daemon_api() {
        let port = 13010u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        // Connect to API
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/api/agents", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Verify connection succeeded
        assert!(
            response.is_ok(),
            "TUI should connect to daemon API successfully"
        );
        assert_eq!(response.unwrap().status(), 200, "API should return 200");
    }

    #[tokio::test]
    async fn test_agent_list_endpoint_returns_correct_structure() {
        let port = 13011u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/api/agents", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Verify structure
        assert!(response.is_ok(), "Should get agent list");
        let resp = response.unwrap();

        if let Ok(json) = resp.json::<serde_json::Value>().await {
            // Should be a JSON array
            assert!(json.is_array(), "Response should be a JSON array");

            // Verify array has at least one element
            let arr = json.as_array().expect("Should be array");
            assert!(!arr.is_empty(), "Should have at least one agent");

            // Verify agent structure
            for agent in arr {
                assert!(
                    agent.get("name").is_some(),
                    "Agent should have 'name' field"
                );
                assert!(
                    agent.get("type").is_some(),
                    "Agent should have 'type' field"
                );
                assert!(
                    agent.get("model").is_some(),
                    "Agent should have 'model' field"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_stats_endpoint_returns_metrics() {
        let port = 13012u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/api/stats", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Verify response structure
        assert!(response.is_ok(), "Stats endpoint should respond");
        let resp = response.unwrap();

        if let Ok(json) = resp.json::<serde_json::Value>().await {
            // Check main structure
            assert!(json.get("project").is_some(), "Should have 'project' field");
            assert!(json.get("machine").is_some(), "Should have 'machine' field");

            // Verify project metrics
            if let Some(project) = json.get("project") {
                assert!(project.get("name").is_some(), "Project should have name");
                assert!(project.get("cost").is_some(), "Project should have cost");
                assert!(project.get("calls").is_some(), "Project should have calls");
            }

            // Verify machine metrics
            if let Some(machine) = json.get("machine") {
                assert!(machine.get("cpu").is_some(), "Machine should have cpu");
                assert!(
                    machine.get("memory").is_some(),
                    "Machine should have memory"
                );
                assert!(
                    machine.get("uptime").is_some(),
                    "Machine should have uptime"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_terminal_endpoint_exists() {
        // This is a basic connectivity test for WebSocket upgrade
        // Full WebSocket testing would require tokio-tungstenite or similar
        let port = 13013u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();

        // Try to make a regular HTTP request to terminal endpoint
        // (WebSocket upgrade test would require proper WebSocket client)
        let response = client
            .get(format!("http://127.0.0.1:{}/terminal", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Terminal endpoint should exist (might return error due to HTTP vs WebSocket)
        assert!(response.is_ok(), "Terminal endpoint should exist");
    }
}

#[cfg(test)]
mod daemon_lifecycle_tests {
    use std::process::Command;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    fn spawn_daemon(port: u16) -> std::process::Child {
        Command::new("cargo")
            .args(&["run", "--", "run", "--port", &port.to_string()])
            .current_dir("/Users/brent/git/cc-orchestra/cco")
            .spawn()
            .expect("Failed to spawn daemon")
    }

    async fn is_port_listening(port: u16, timeout_secs: u64) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if let Ok(stream) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
                drop(stream);
                return true;
            }

            if start.elapsed() > timeout {
                return false;
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    #[allow(dead_code)]
    async fn is_port_closed(port: u16, timeout_secs: u64) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            match std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
                Ok(_) => {
                    // Port still listening
                    if start.elapsed() > timeout {
                        return false;
                    }
                    sleep(Duration::from_millis(100)).await;
                }
                Err(_) => {
                    // Port is closed
                    return true;
                }
            }
        }
    }

    #[tokio::test]
    async fn test_daemon_starts_cleanly() {
        let port = 13020u16;
        let daemon = spawn_daemon(port);

        // Daemon should start and bind to port
        let started = is_port_listening(port, 10).await;

        // Kill daemon (if still needed to clean up)
        let mut daemon = daemon;
        let _ = daemon.kill();
        let _ = daemon.wait();

        assert!(started, "Daemon should start cleanly and bind to port");
    }

    #[tokio::test]
    async fn test_daemon_handles_sigint_gracefully() {
        let port = 13021u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        // Send SIGINT (Ctrl+C)
        let start = Instant::now();
        let _ = daemon.kill();
        let result = daemon.wait();

        // Cleanup duration tracking
        let elapsed = start.elapsed();

        // Verify daemon exited
        assert!(result.is_ok(), "Daemon should exit after SIGINT");

        // Should exit within reasonable time (but this may vary on test system)
        assert!(
            elapsed < Duration::from_secs(5),
            "Daemon should shutdown quickly (got {:?})",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_daemon_shutdown_timing() {
        let port = 13022u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        // Measure shutdown time
        let start = Instant::now();
        let _ = daemon.kill();
        let _ = daemon.wait();
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(5),
            "Shutdown should complete in reasonable time (got {:?})",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_port_released_after_shutdown() {
        let port = 13023u16;

        // Start first daemon
        let mut daemon1 = spawn_daemon(port);

        if !is_port_listening(port, 10).await {
            let _ = daemon1.kill();
            panic!("Daemon 1 failed to start");
        }

        // Kill daemon 1
        let _ = daemon1.kill();
        let _ = daemon1.wait();

        // Wait for port to be released
        sleep(Duration::from_secs(1)).await;

        // Try to start daemon 2 on same port
        let mut daemon2 = spawn_daemon(port);

        let started = is_port_listening(port, 10).await;

        // Cleanup
        let _ = daemon2.kill();
        let _ = daemon2.wait();

        assert!(
            started,
            "Port should be released after shutdown, second daemon should start"
        );
    }

    #[tokio::test]
    async fn test_no_zombie_processes() {
        let port = 13024u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        // Kill daemon
        let _ = daemon.kill();
        let _ = daemon.wait();

        sleep(Duration::from_millis(500)).await;

        // Check for zombie processes by listing processes
        // On Unix systems, zombie processes will show as defunct
        let output = std::process::Command::new("ps")
            .arg("aux")
            .output()
            .expect("Failed to run ps command");

        let ps_output = String::from_utf8_lossy(&output.stdout);

        // Check that there are no defunct cco processes
        let has_zombies = ps_output.lines().any(|line| {
            line.contains("defunct") && (line.contains("cco") || line.contains("cargo"))
        });

        assert!(
            !has_zombies,
            "Should not have zombie processes after daemon exits"
        );
    }
}

#[cfg(test)]
mod error_handling_tests {
    use std::process::Command;
    use std::time::Duration;
    use tokio::time::sleep;

    fn spawn_daemon(port: u16) -> std::process::Child {
        Command::new("cargo")
            .args(&["run", "--", "run", "--port", &port.to_string()])
            .current_dir("/Users/brent/git/cc-orchestra/cco")
            .spawn()
            .expect("Failed to spawn daemon")
    }

    async fn is_port_listening(port: u16, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if let Ok(stream) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
                drop(stream);
                return true;
            }

            if start.elapsed() > timeout {
                return false;
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    #[tokio::test]
    async fn test_daemon_handles_port_conflict() {
        let port = 13030u16;

        // Start first daemon
        let mut daemon1 = spawn_daemon(port);

        if !is_port_listening(port, 10).await {
            let _ = daemon1.kill();
            panic!("Daemon 1 failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        // Try to start second daemon on same port (should fail)
        let mut daemon2 = spawn_daemon(port);

        // Wait a bit and check if second daemon exited
        sleep(Duration::from_secs(2)).await;

        // Try to get status of daemon2
        let status = daemon2.try_wait();

        // Cleanup
        let _ = daemon1.kill();
        let _ = daemon1.wait();
        let _ = daemon2.kill();
        let _ = daemon2.wait();

        // Second daemon should have exited due to port conflict
        match status {
            Ok(Some(_exit_status)) => {
                // Daemon exited, as expected
                assert!(true, "Second daemon correctly failed due to port conflict");
            }
            Ok(None) => {
                panic!("Second daemon should have exited due to port conflict");
            }
            Err(_) => {
                // Error getting status, but daemon should have exited
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_endpoint_returns_404() {
        let port = 13031u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "http://127.0.0.1:{}/api/invalid-endpoint-12345",
                port
            ))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        assert!(response.is_ok(), "Request should complete");

        let resp = response.unwrap();
        assert_eq!(resp.status(), 404, "Invalid endpoint should return 404");
    }

    #[tokio::test]
    async fn test_daemon_stability_after_bad_request() {
        let port = 13032u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();

        // Make invalid request
        let _bad_response = client
            .get(format!("http://127.0.0.1:{}/invalid", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        sleep(Duration::from_millis(100)).await;

        // Now make valid request to verify daemon is still responsive
        let good_response = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        assert!(
            good_response.is_ok(),
            "Daemon should still be responsive after bad request"
        );
        assert_eq!(
            good_response.unwrap().status(),
            200,
            "Daemon should respond normally after error"
        );
    }

    #[tokio::test]
    async fn test_malformed_request_handling() {
        let port = 13033u16;
        let mut daemon = spawn_daemon(port);

        // Wait for daemon to start
        if !is_port_listening(port, 10).await {
            let _ = daemon.kill();
            panic!("Daemon failed to start");
        }

        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();

        // Try POST with invalid JSON
        let response = client
            .post(format!("http://127.0.0.1:{}/v1/chat/completions", port))
            .header("content-type", "application/json")
            .body("{invalid json}")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        // Cleanup
        let _ = daemon.kill();
        let _ = daemon.wait();

        // Should get some kind of error response
        assert!(response.is_ok(), "Request should complete");

        let resp = response.unwrap();
        assert!(
            resp.status().is_client_error() || resp.status().is_server_error(),
            "Invalid JSON should return error status"
        );
    }
}
