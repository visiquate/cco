//! TUI dashboard launcher module
//!
//! This module provides the functionality to launch the TUI (Terminal User Interface)
//! monitoring dashboard. It checks if the daemon is running and prompts to start it
//! if needed.

use anyhow::Result;
use std::io::{self, Write};

/// Launch TUI monitoring dashboard
///
/// This function is called when a user runs `cco tui`. It ensures the daemon
/// is running (prompting to start if needed) and then launches the TUI dashboard.
///
/// # Flow
/// 1. Check if daemon is running
/// 2. If not running, prompt user to start it
/// 3. Launch TUI dashboard
///
/// # Returns
/// * `Ok(())` - TUI exited normally
/// * `Err` - Failed to start TUI or daemon
pub async fn launch_tui() -> Result<()> {
    use cco::daemon::{load_config, DaemonManager};

    let config = load_config().unwrap_or_default();
    let manager = DaemonManager::new(config);

    // Check if daemon is running
    match manager.get_status().await {
        Ok(_status) => {
            println!("✅ Daemon is running");
        }
        Err(_) => {
            eprintln!("⚠️  Daemon is not running");
            eprintln!("   The TUI dashboard requires the daemon to be running.");
            eprintln!();

            // Prompt user if they want to start daemon
            print!("Start daemon now? [Y/n] ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let should_start = input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y");

            if should_start {
                println!("⚙️  Starting daemon...");
                manager.start().await?;

                // Wait for daemon to be ready
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                // Verify daemon started
                match manager.get_status().await {
                    Ok(_) => {
                        println!("✅ Daemon started successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Daemon failed to start: {}", e);
                        eprintln!("   Try manually: cco daemon start");
                        eprintln!("   Check logs: cco daemon logs");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("Daemon not started. Exiting.");
                return Ok(());
            }
        }
    }

    println!("🎯 Launching TUI dashboard...");
    println!("   Press 'q' to quit, 'h' for help");
    println!();

    // Give user a moment to read the instructions
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Launch TUI
    match cco::TuiApp::new().await {
        Ok(mut app) => app.run().await,
        Err(e) => {
            eprintln!("❌ Failed to start TUI: {}", e);
            eprintln!();
            eprintln!("   Troubleshooting:");
            eprintln!("   1. Check if daemon is running: cco daemon status");
            eprintln!("   2. Try restarting daemon: cco daemon restart");

            // Try to discover daemon port for helpful error message
            if let Ok(port) = cco::daemon::read_daemon_port() {
                eprintln!("   3. Use web dashboard instead: http://localhost:{}", port);
            } else {
                eprintln!("   3. Start the daemon first: cco daemon start");
            }

            eprintln!();
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Full integration tests for TUI are in tests/tui_integration_tests.rs
    // These are just unit tests for helper functions

    #[test]
    fn test_module_compiles() {
        // This test ensures the module compiles correctly
        // Real tests require daemon to be running
        assert!(true);
    }
}
