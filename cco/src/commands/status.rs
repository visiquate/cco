//! Status command - show running CCO instances

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use sysinfo::{Pid, System};

/// PID file metadata structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PidInfo {
    pub pid: u32,
    pub port: u16,
    pub started_at: DateTime<Utc>,
    pub version: String,
}

/// Running instance information
#[derive(Debug)]
pub struct InstanceInfo {
    pub pid: u32,
    pub port: u16,
    pub uptime: String,
    pub dashboard_url: String,
    pub version: String,
    pub is_running: bool,
}

/// Get the PID directory path
pub fn get_pid_dir() -> Result<PathBuf> {
    let data_dir = data_local_dir().context("Failed to get local data directory")?;

    let pid_dir = data_dir.join("cco").join("pids");

    // Create directory if it doesn't exist
    fs::create_dir_all(&pid_dir).context("Failed to create PID directory")?;

    Ok(pid_dir)
}

/// Get the PID file path for a specific port
pub fn get_pid_file(port: u16) -> Result<PathBuf> {
    let pid_dir = get_pid_dir()?;
    Ok(pid_dir.join(format!("cco-{}.pid", port)))
}

/// Read a PID file
pub fn read_pid_file(path: &PathBuf) -> Result<PidInfo> {
    let contents = fs::read_to_string(path).context("Failed to read PID file")?;

    let pid_info: PidInfo = serde_json::from_str(&contents).context("Failed to parse PID file")?;

    Ok(pid_info)
}

/// Check if a process is running
pub fn is_process_running(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes();

    system.process(Pid::from_u32(pid)).is_some()
}

/// Clean up stale PID file
pub fn cleanup_stale_pid_file(path: &PathBuf) -> Result<()> {
    fs::remove_file(path).context("Failed to remove stale PID file")?;
    Ok(())
}

/// Get all running instances
pub fn get_all_instances() -> Result<Vec<InstanceInfo>> {
    let pid_dir = get_pid_dir()?;
    let mut instances = Vec::new();

    // Read all PID files
    let entries = fs::read_dir(&pid_dir).context("Failed to read PID directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Only process .pid files
        if path.extension().and_then(|s| s.to_str()) != Some("pid") {
            continue;
        }

        match read_pid_file(&path) {
            Ok(pid_info) => {
                let is_running = is_process_running(pid_info.pid);

                // Clean up stale PID files
                if !is_running {
                    let _ = cleanup_stale_pid_file(&path);
                }

                // Calculate uptime
                let uptime = if is_running {
                    let now = Utc::now();
                    let duration = now.signed_duration_since(pid_info.started_at);

                    let days = duration.num_days();
                    let hours = duration.num_hours() % 24;
                    let minutes = duration.num_minutes() % 60;

                    if days > 0 {
                        format!("{}d {}h {}m", days, hours, minutes)
                    } else if hours > 0 {
                        format!("{}h {}m", hours, minutes)
                    } else {
                        format!("{}m", minutes)
                    }
                } else {
                    "stopped".to_string()
                };

                instances.push(InstanceInfo {
                    pid: pid_info.pid,
                    port: pid_info.port,
                    uptime,
                    dashboard_url: format!("http://127.0.0.1:{}", pid_info.port),
                    version: pid_info.version,
                    is_running,
                });
            }
            Err(_) => {
                // If we can't parse the PID file, try to remove it
                let _ = fs::remove_file(&path);
            }
        }
    }

    // Sort by port
    instances.sort_by_key(|i| i.port);

    Ok(instances)
}

/// Display status of all instances
pub async fn run() -> Result<()> {
    let instances = get_all_instances()?;

    if instances.is_empty() {
        println!("No CCO instances running");
        return Ok(());
    }

    // Display header
    println!(
        "\n{:<8} {:<8} {:<12} {:<30} {:<15} {:<10}",
        "PID", "PORT", "UPTIME", "DASHBOARD", "VERSION", "STATUS"
    );
    println!("{}", "-".repeat(95));

    // Display instances
    for instance in instances {
        let status = if instance.is_running {
            "running"
        } else {
            "stopped"
        };

        println!(
            "{:<8} {:<8} {:<12} {:<30} {:<15} {:<10}",
            instance.pid,
            instance.port,
            instance.uptime,
            instance.dashboard_url,
            instance.version,
            status
        );
    }

    println!();

    Ok(())
}
