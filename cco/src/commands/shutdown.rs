//! Shutdown command - stop running CCO instances

use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

use super::status::{get_all_instances, get_pid_file, is_process_running};

/// Shutdown a specific instance by port
pub async fn shutdown_by_port(port: u16) -> Result<()> {
    let pid_file = get_pid_file(port)?;

    if !pid_file.exists() {
        bail!("No CCO instance running on port {}", port);
    }

    let pid_info = super::status::read_pid_file(&pid_file)?;

    if !is_process_running(pid_info.pid) {
        println!("Instance on port {} (PID {}) is not running", port, pid_info.pid);
        super::status::cleanup_stale_pid_file(&pid_file)?;
        return Ok(());
    }

    println!("Shutting down CCO instance on port {} (PID {})...", port, pid_info.pid);

    // Send SIGTERM signal
    if let Err(e) = send_sigterm(pid_info.pid) {
        eprintln!("Failed to send SIGTERM: {}", e);
        bail!("Failed to shutdown instance");
    }

    // Wait for graceful shutdown (up to 10 seconds)
    let mut attempts = 0;
    while attempts < 20 {
        sleep(Duration::from_millis(500)).await;

        if !is_process_running(pid_info.pid) {
            println!("✅ Instance shut down gracefully");

            // Remove PID file
            let _ = std::fs::remove_file(&pid_file);
            return Ok(());
        }

        attempts += 1;
    }

    // If still running after 10 seconds, try SIGKILL
    println!("⚠️  Process did not shut down gracefully, sending SIGKILL...");

    if let Err(e) = send_sigkill(pid_info.pid) {
        eprintln!("Failed to send SIGKILL: {}", e);
        bail!("Failed to force shutdown instance");
    }

    sleep(Duration::from_millis(500)).await;

    if !is_process_running(pid_info.pid) {
        println!("✅ Instance force shut down");
        let _ = std::fs::remove_file(&pid_file);
        Ok(())
    } else {
        bail!("Failed to shutdown instance (PID {})", pid_info.pid)
    }
}

/// Shutdown all instances
pub async fn shutdown_all() -> Result<()> {
    let instances = get_all_instances()?;

    if instances.is_empty() {
        println!("No CCO instances running");
        return Ok(());
    }

    let running_instances: Vec<_> = instances.into_iter()
        .filter(|i| i.is_running)
        .collect();

    if running_instances.is_empty() {
        println!("No CCO instances running");
        return Ok(());
    }

    println!("Found {} running instance(s)", running_instances.len());

    for instance in running_instances {
        shutdown_by_port(instance.port).await?;
    }

    println!("\n✅ All instances shut down");

    Ok(())
}

/// Interactive shutdown - show menu to select instance
pub async fn shutdown_interactive() -> Result<()> {
    let instances = get_all_instances()?;

    if instances.is_empty() {
        println!("No CCO instances running");
        return Ok(());
    }

    let running_instances: Vec<_> = instances.into_iter()
        .filter(|i| i.is_running)
        .collect();

    if running_instances.is_empty() {
        println!("No CCO instances running");
        return Ok(());
    }

    // Display instances
    println!("\nRunning CCO instances:");
    println!();

    for (idx, instance) in running_instances.iter().enumerate() {
        println!("  {}. Port {} (PID {}) - {}",
                 idx + 1,
                 instance.port,
                 instance.pid,
                 instance.dashboard_url);
    }

    println!();
    println!("  a. Shutdown all");
    println!("  q. Quit");
    println!();

    // Get user input
    print!("Select instance to shutdown: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match input {
        "q" | "Q" => {
            println!("Cancelled");
            return Ok(());
        }
        "a" | "A" => {
            return shutdown_all().await;
        }
        _ => {
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= running_instances.len() {
                    let instance = &running_instances[idx - 1];
                    return shutdown_by_port(instance.port).await;
                }
            }

            bail!("Invalid selection");
        }
    }
}

/// Send SIGTERM to process
#[cfg(unix)]
fn send_sigterm(pid: u32) -> Result<()> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid as NixPid;

    let nix_pid = NixPid::from_raw(pid as i32);
    signal::kill(nix_pid, Signal::SIGTERM)
        .context("Failed to send SIGTERM")?;

    Ok(())
}

/// Send SIGKILL to process
#[cfg(unix)]
fn send_sigkill(pid: u32) -> Result<()> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid as NixPid;

    let nix_pid = NixPid::from_raw(pid as i32);
    signal::kill(nix_pid, Signal::SIGKILL)
        .context("Failed to send SIGKILL")?;

    Ok(())
}

/// Windows implementation (placeholder)
#[cfg(not(unix))]
fn send_sigterm(pid: u32) -> Result<()> {
    // On Windows, we would use TerminateProcess or similar
    let mut system = System::new();
    system.refresh_processes();

    if let Some(process) = system.process(Pid::from_u32(pid)) {
        if process.kill() {
            Ok(())
        } else {
            bail!("Failed to terminate process")
        }
    } else {
        bail!("Process not found")
    }
}

/// Windows implementation (placeholder)
#[cfg(not(unix))]
fn send_sigkill(pid: u32) -> Result<()> {
    send_sigterm(pid) // On Windows, use same method
}

/// Main entry point for shutdown command
pub async fn run(port: Option<u16>, all: bool) -> Result<()> {
    if all {
        shutdown_all().await
    } else if let Some(port) = port {
        shutdown_by_port(port).await
    } else {
        shutdown_interactive().await
    }
}
