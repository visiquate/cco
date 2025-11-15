//! Logs command - view CCO instance logs

use anyhow::{bail, Context, Result};
use dirs::data_local_dir;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use super::status::get_all_instances;

/// Get the logs directory path
pub fn get_logs_dir() -> Result<PathBuf> {
    let data_dir = data_local_dir()
        .context("Failed to get local data directory")?;

    let logs_dir = data_dir.join("cco").join("logs");

    // Create directory if it doesn't exist
    fs::create_dir_all(&logs_dir)
        .context("Failed to create logs directory")?;

    Ok(logs_dir)
}

/// Get the log file path for a specific port
pub fn get_log_file(port: u16) -> Result<PathBuf> {
    let logs_dir = get_logs_dir()?;
    Ok(logs_dir.join(format!("cco-{}.log", port)))
}

/// Read last N lines from a file
fn read_last_lines(file: &mut File, num_lines: usize) -> Result<Vec<String>> {
    let file_size = file.metadata()?.len();

    if file_size == 0 {
        return Ok(Vec::new());
    }

    // Read file in chunks from the end
    let mut buffer = Vec::new();
    let chunk_size = 8192;
    let mut position = file_size;
    let mut lines = Vec::new();

    while position > 0 && lines.len() < num_lines {
        let read_size = std::cmp::min(chunk_size, position);
        position -= read_size;

        file.seek(SeekFrom::Start(position))?;

        let mut chunk = vec![0u8; read_size as usize];
        file.read_exact(&mut chunk)?;

        buffer.splice(0..0, chunk);

        // Parse lines from buffer
        let text = String::from_utf8_lossy(&buffer);
        lines = text.lines().map(|s| s.to_string()).collect();

        if lines.len() >= num_lines {
            break;
        }
    }

    // Return last N lines
    if lines.len() > num_lines {
        Ok(lines[lines.len() - num_lines..].to_vec())
    } else {
        Ok(lines)
    }
}

/// Read a gzipped log file
fn read_gz_file(path: &PathBuf) -> Result<Vec<String>> {
    let file = File::open(path)
        .context("Failed to open gzipped log file")?;

    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let lines: Result<Vec<String>, _> = reader.lines().collect();
    lines.context("Failed to read gzipped log file")
}

/// Display logs for a specific port
pub async fn show_logs(port: u16, follow: bool, num_lines: usize) -> Result<()> {
    let log_file_path = get_log_file(port)?;

    if !log_file_path.exists() {
        bail!("No log file found for port {}", port);
    }

    if follow {
        // Follow mode - tail -f behavior
        follow_logs(&log_file_path).await
    } else {
        // Static mode - show last N lines
        show_static_logs(&log_file_path, num_lines)
    }
}

/// Show static logs (last N lines)
fn show_static_logs(log_file_path: &PathBuf, num_lines: usize) -> Result<()> {
    // Check for rotated logs (*.log.gz files)
    let logs_dir = log_file_path.parent()
        .context("Failed to get logs directory")?;

    let file_name = log_file_path.file_name()
        .and_then(|s| s.to_str())
        .context("Failed to get log file name")?;

    // Find all related log files (including rotated ones)
    let mut log_files = Vec::new();

    if let Ok(entries) = fs::read_dir(logs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if name.starts_with(file_name) {
                log_files.push(path);
            }
        }
    }

    // Sort by name (oldest first)
    log_files.sort();

    // Collect all lines
    let mut all_lines = Vec::new();

    for log_file in log_files {
        if log_file.extension().and_then(|s| s.to_str()) == Some("gz") {
            // Read gzipped file
            if let Ok(lines) = read_gz_file(&log_file) {
                all_lines.extend(lines);
            }
        } else {
            // Read regular file
            if let Ok(file) = File::open(&log_file) {
                let reader = BufReader::new(file);
                if let Ok(lines) = reader.lines().collect::<Result<Vec<String>, _>>() {
                    all_lines.extend(lines);
                }
            }
        }
    }

    // Show last N lines
    let start_index = if all_lines.len() > num_lines {
        all_lines.len() - num_lines
    } else {
        0
    };

    for line in &all_lines[start_index..] {
        println!("{}", line);
    }

    Ok(())
}

/// Follow logs (tail -f behavior)
async fn follow_logs(log_file_path: &PathBuf) -> Result<()> {
    let mut file = File::open(log_file_path)
        .context("Failed to open log file")?;

    // Seek to end of file
    file.seek(SeekFrom::End(0))?;

    println!("Following logs (Ctrl+C to stop)...\n");

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    loop {
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // No data available, sleep and retry
                sleep(Duration::from_millis(100)).await;
            }
            Ok(_) => {
                // Print the line
                print!("{}", buffer);
                io::stdout().flush()?;
                buffer.clear();
            }
            Err(e) => {
                eprintln!("Error reading log file: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Interactive logs - show menu to select instance
pub async fn logs_interactive(follow: bool, num_lines: usize) -> Result<()> {
    let instances = get_all_instances()?;

    if instances.is_empty() {
        println!("No CCO instances found");
        return Ok(());
    }

    let running_instances: Vec<_> = instances.into_iter()
        .filter(|i| i.is_running)
        .collect();

    if running_instances.is_empty() {
        println!("No CCO instances running");
        return Ok(());
    }

    if running_instances.len() == 1 {
        // Only one instance, show its logs
        let instance = &running_instances[0];
        return show_logs(instance.port, follow, num_lines).await;
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
    println!("  q. Quit");
    println!();

    // Get user input
    print!("Select instance to view logs: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match input {
        "q" | "Q" => {
            println!("Cancelled");
            return Ok(());
        }
        _ => {
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= running_instances.len() {
                    let instance = &running_instances[idx - 1];
                    return show_logs(instance.port, follow, num_lines).await;
                }
            }

            bail!("Invalid selection");
        }
    }
}

/// Main entry point for logs command
pub async fn run(port: Option<u16>, follow: bool, num_lines: usize) -> Result<()> {
    if let Some(port) = port {
        show_logs(port, follow, num_lines).await
    } else {
        logs_interactive(follow, num_lines).await
    }
}
