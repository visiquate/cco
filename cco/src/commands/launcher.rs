//! Claude Code launcher module with orchestration support
//!
//! This module provides the functionality to launch Claude Code with full
//! orchestration support including daemon auto-start, temp file verification,
//! and environment variable injection.

use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Launch Claude Code with orchestration support
///
/// This is the main entry point when a user runs `cco` without any subcommand.
/// It ensures the daemon is running, temp settings exist, environment variables are set,
/// and then launches Claude Code in the current working directory.
///
/// # Flow
/// 1. Ensure daemon is running (auto-start if needed)
/// 2. Get temp settings file path
/// 3. Verify temp files exist (daemon should have created them)
/// 4. Set ORCHESTRATOR_* environment variables
/// 5. Find Claude Code executable in PATH
/// 6. Launch Claude Code with --settings flag and all arguments passed through
///
/// # Arguments
/// * `args` - Arguments to pass through to Claude Code
///
/// # Examples
/// ```no_run
/// // Launch Claude Code with no arguments
/// launch_claude_code(vec![]).await?;
///
/// // Launch Claude Code with --help argument
/// launch_claude_code(vec!["--help".to_string()]).await?;
/// ```
pub async fn launch_claude_code(args: Vec<String>) -> Result<()> {
    // Step 1: Ensure daemon is running
    ensure_daemon_running().await?;

    // Step 2: Get temp settings file path
    let settings_path = get_orchestrator_settings_path()?;

    // Step 3: Verify temp files exist (daemon should have created them)
    verify_temp_files_exist(&settings_path).await?;

    // Step 4: Set orchestrator environment variables
    set_orchestrator_env_vars(&settings_path);

    // Step 5: Find and launch Claude Code
    launch_claude_code_process(&settings_path, args).await?;

    Ok(())
}

/// Ensure daemon is running, start if needed
///
/// Checks if the daemon is running by attempting to get its status.
/// If the daemon is not running, it will auto-start and wait up to 3 seconds
/// for the daemon to become healthy.
///
/// # Returns
/// * `Ok(())` - Daemon is running and healthy
/// * `Err` - Failed to start daemon or daemon did not become healthy
async fn ensure_daemon_running() -> Result<()> {
    use cco::daemon::{load_config, DaemonManager};

    let config = load_config().unwrap_or_default();
    let manager = DaemonManager::new(config);

    // Check if daemon is running
    match manager.get_status().await {
        Ok(_status) => {
            // Daemon is running
            println!("✅ Daemon is running");
            Ok(())
        }
        Err(_) => {
            // Daemon not running, start it
            println!("⚙️  Starting daemon...");
            manager.start().await.context("Failed to start daemon")?;

            // Wait for daemon to be ready (max 3 seconds)
            for attempt in 1..=10 {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                if manager.get_status().await.is_ok() {
                    println!("✅ Daemon started");
                    return Ok(());
                }

                if attempt == 10 {
                    anyhow::bail!(
                        "Daemon failed to start within 3 seconds.\n\
                         Try manually starting: cco daemon start\n\
                         Or check logs: cco daemon logs"
                    );
                }
            }

            Ok(())
        }
    }
}

/// Get path to orchestrator settings file in temp directory
///
/// Returns the standard path where the daemon should create the settings file.
///
/// # Returns
/// Path to settings file (e.g., `/tmp/.cco-orchestrator-settings` on Unix)
fn get_orchestrator_settings_path() -> Result<PathBuf> {
    let temp_dir = env::temp_dir();
    let settings_path = temp_dir.join(".cco-orchestrator-settings");
    Ok(settings_path)
}

/// Verify temp files exist (daemon should have created them)
///
/// Checks that the daemon successfully created the orchestrator settings file.
/// This ensures the daemon is fully initialized before launching Claude Code.
///
/// # Arguments
/// * `settings_path` - Path to the settings file
///
/// # Returns
/// * `Ok(())` - Settings file exists and is readable
/// * `Err` - Settings file not found or not readable
async fn verify_temp_files_exist(settings_path: &PathBuf) -> Result<()> {
    // Check that daemon created the settings file
    if !settings_path.exists() {
        anyhow::bail!(
            "Orchestrator settings not found at: {}\n\
             This usually means the daemon failed to start.\n\
             Try: cco daemon restart",
            settings_path.display()
        );
    }

    // Verify it's readable
    let metadata = settings_path.metadata().context("Cannot read orchestrator settings")?;

    if !metadata.is_file() {
        anyhow::bail!(
            "Orchestrator settings path is not a file: {}",
            settings_path.display()
        );
    }

    println!("✅ Orchestrator settings found");
    Ok(())
}

/// Set ORCHESTRATOR_* environment variables
///
/// Sets environment variables that tell Claude Code where to find orchestration files.
/// These variables point to temp directory locations where the daemon created the files.
///
/// # Environment Variables Set
/// * `ORCHESTRATOR_ENABLED` - Flag to enable orchestration (true)
/// * `ORCHESTRATOR_SETTINGS` - Path to settings file in temp directory
/// * `ORCHESTRATOR_AGENTS` - Path to sealed agents file in temp directory
/// * `ORCHESTRATOR_RULES` - Path to sealed orchestrator rules in temp directory
/// * `ORCHESTRATOR_HOOKS` - Path to sealed hooks config in temp directory
/// * `ORCHESTRATOR_API_URL` - Daemon API endpoint (http://localhost:3000)
/// * `ORCHESTRATOR_HOOKS_CONFIG` - JSON hooks configuration (from settings file)
///
/// # Arguments
/// * `settings_path` - Path to the settings file
fn set_orchestrator_env_vars(settings_path: &PathBuf) {
    let temp_dir = env::temp_dir();

    env::set_var("ORCHESTRATOR_ENABLED", "true");
    env::set_var("ORCHESTRATOR_SETTINGS", settings_path.to_string_lossy().to_string());
    env::set_var("ORCHESTRATOR_AGENTS", temp_dir.join(".cco-agents-sealed").to_string_lossy().to_string());
    env::set_var("ORCHESTRATOR_RULES", temp_dir.join(".cco-rules-sealed").to_string_lossy().to_string());
    env::set_var("ORCHESTRATOR_HOOKS", temp_dir.join(".cco-hooks-sealed").to_string_lossy().to_string());
    env::set_var("ORCHESTRATOR_API_URL", "http://localhost:3000");

    // Read settings file and extract hooks configuration
    if let Ok(settings_content) = std::fs::read_to_string(settings_path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&settings_content) {
            // Pass entire hooks config as JSON for Claude Code to parse
            if let Some(hooks) = settings.get("hooks") {
                if let Ok(hooks_json) = serde_json::to_string(hooks) {
                    env::set_var("ORCHESTRATOR_HOOKS_CONFIG", hooks_json);
                }

                // Set specific permission flags for quick access
                if let Some(perms) = hooks.get("permissions") {
                    if let Some(auto_allow_read) = perms.get("allow_file_read").and_then(|v| v.as_bool()) {
                        env::set_var("ORCHESTRATOR_AUTO_ALLOW_READ", auto_allow_read.to_string());
                    }
                    if let Some(require_cud) = perms.get("allow_command_modification").and_then(|v| v.as_bool()) {
                        env::set_var("ORCHESTRATOR_REQUIRE_CUD_CONFIRMATION", (!require_cud).to_string());
                    }
                }

                // Set hooks enabled flag
                if let Some(enabled) = hooks.get("enabled").and_then(|v| v.as_bool()) {
                    env::set_var("ORCHESTRATOR_HOOKS_ENABLED", enabled.to_string());
                }
            }
        }
    }

    println!("✅ Orchestration environment configured");
}

/// Find Claude Code executable in PATH
///
/// Searches for the Claude Code executable in the system PATH.
/// Tries multiple common names and locations.
///
/// # Returns
/// * `Ok(PathBuf)` - Path to Claude Code executable
/// * `Err` - Claude Code not found in PATH
fn find_claude_code_executable() -> Result<PathBuf> {
    // Try common names in order of preference
    let candidates = vec![
        "claude",      // Most common alias
        "claude-code", // Full name
        "claude-ai",   // Alternative name
    ];

    for candidate in candidates {
        if let Ok(path) = which::which(candidate) {
            return Ok(path);
        }
    }

    // Not found in PATH
    anyhow::bail!(
        "Claude Code executable not found in PATH\n\
         Please install Claude Code first:\n\
         https://claude.ai/code\n\
         \n\
         After installation, ensure 'claude' is in your PATH."
    )
}

/// Launch Claude Code process with settings file
///
/// Spawns the Claude Code process with:
/// * --settings flag pointing to orchestrator settings
/// * All pass-through arguments from user
/// * Current working directory preserved
/// * All ORCHESTRATOR_* environment variables set
///
/// # Arguments
/// * `settings_path` - Path to orchestrator settings file
/// * `args` - Arguments to pass to Claude Code
///
/// # Returns
/// * `Ok(())` - Claude Code exited successfully
/// * `Err` - Claude Code failed to start or exited with error
async fn launch_claude_code_process(settings_path: &PathBuf, args: Vec<String>) -> Result<()> {
    let claude_code_path = find_claude_code_executable()?;
    let cwd = env::current_dir().context("Failed to get current working directory")?;

    println!("🚀 Launching Claude Code with orchestration support...");
    println!("   Working directory: {}", cwd.display());
    println!("   Settings: {}", settings_path.display());
    println!("   Executable: {}", claude_code_path.display());

    if !args.is_empty() {
        println!("   Arguments: {}", args.join(" "));
    }

    println!();

    // Build command with --settings flag
    let mut cmd = Command::new(&claude_code_path);
    cmd.arg("--settings");
    cmd.arg(settings_path);
    cmd.args(&args);
    cmd.current_dir(&cwd);

    // Execute Claude Code and wait for it to complete
    let status = cmd
        .status()
        .context("Failed to execute Claude Code process")?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        anyhow::bail!("Claude Code exited with status code: {}", code);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;

    #[test]
    fn test_get_orchestrator_settings_path() {
        let settings_path = get_orchestrator_settings_path().unwrap();

        // Should be in temp directory
        assert!(settings_path.starts_with(env::temp_dir()));

        // Should end with settings filename
        assert_eq!(settings_path.file_name().unwrap(), ".cco-orchestrator-settings");
    }

    #[tokio::test]
    async fn test_verify_temp_files_exist_success() {
        // Create mock temp directory and settings file
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join(".cco-orchestrator-settings");

        // Create mock settings file
        fs::write(&settings_path, b"{}").unwrap();

        // Verify settings exist
        let result = verify_temp_files_exist(&settings_path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_temp_files_exist_missing() {
        // Try to verify non-existent file
        let settings_path = PathBuf::from("/nonexistent/.cco-orchestrator-settings");

        let result = verify_temp_files_exist(&settings_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_set_orchestrator_env_vars() {
        let settings_path = PathBuf::from("/tmp/.cco-orchestrator-settings");
        set_orchestrator_env_vars(&settings_path);

        // Verify environment variables are set
        assert_eq!(env::var("ORCHESTRATOR_ENABLED").unwrap(), "true");
        assert_eq!(
            env::var("ORCHESTRATOR_SETTINGS").unwrap(),
            settings_path.to_string_lossy().to_string()
        );
        assert!(env::var("ORCHESTRATOR_AGENTS").is_ok());
        assert!(env::var("ORCHESTRATOR_RULES").is_ok());
        assert!(env::var("ORCHESTRATOR_HOOKS").is_ok());
        assert_eq!(
            env::var("ORCHESTRATOR_API_URL").unwrap(),
            "http://localhost:3000"
        );

        // Clean up environment variables
        env::remove_var("ORCHESTRATOR_ENABLED");
        env::remove_var("ORCHESTRATOR_SETTINGS");
        env::remove_var("ORCHESTRATOR_AGENTS");
        env::remove_var("ORCHESTRATOR_RULES");
        env::remove_var("ORCHESTRATOR_HOOKS");
        env::remove_var("ORCHESTRATOR_API_URL");
    }

    #[test]
    fn test_find_claude_code_executable() {
        // This test will succeed if 'claude' is in PATH, fail otherwise
        // We can't guarantee it's installed in test environment, so just test the logic
        let result = find_claude_code_executable();

        // If claude is installed, should return Ok
        // If not installed, should return descriptive error
        if let Err(e) = result {
            assert!(e.to_string().contains("not found in PATH"));
            assert!(e.to_string().contains("https://claude.ai/code"));
        }
    }
}
