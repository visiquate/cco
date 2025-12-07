//! LLM Router CLI commands
//!
//! Provides CLI wrapper around the daemon's LLM routing functionality.
//! Routes different types of tasks to appropriate LLM endpoints.

use anyhow::{Context, Result};
use cco::daemon::lifecycle::read_daemon_port;
use serde_json::json;
use std::path::PathBuf;

use super::token_cache;
use crate::LlmRouterAction;

/// Run LLM router command
pub async fn run(action: LlmRouterAction) -> Result<()> {
    match action {
        LlmRouterAction::Stats { format } => stats(format).await,
        LlmRouterAction::Route {
            agent_type,
            task_type,
            format,
        } => route(agent_type, task_type, format).await,
        LlmRouterAction::Call { prompt, format } => call(prompt, format).await,
    }
}

/// Show routing configuration and statistics
async fn stats(format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/api/llm/stats", port))
        .bearer_auth(token)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to get routing stats: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Show routing decision for specific agent and task type
async fn route(agent_type: String, task_type: Option<String>, format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "agent_type": agent_type,
        "task_type": task_type,
    });

    let response = client
        .post(format!("http://localhost:{}/api/llm/route", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to get routing decision: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Call custom LLM endpoint with a prompt
async fn call(prompt: String, format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "prompt": prompt,
    });

    let response = client
        .post(format!("http://localhost:{}/api/llm/call", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to call LLM endpoint: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Auto-detect project ID from git repository or current directory
fn auto_detect_project_id() -> Result<String> {
    // Try git repository name first
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let path = String::from_utf8(output.stdout)?;
            let path = path.trim();
            if !path.is_empty() {
                return Ok(PathBuf::from(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string());
            }
        }
    }

    // Fallback to current directory name
    Ok(std::env::current_dir()?
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string())
}

/// Get or generate authentication token
async fn get_or_generate_token(project_id: &str) -> Result<String> {
    // Check for cached token
    let token_cache_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".cco")
        .join("llm-router-token.json");

    token_cache::tighten_permissions_if_exists(&token_cache_path)
        .context("Failed to secure cached LLM router token permissions")?;

    // Try to load cached token
    if let Ok(cache_content) = std::fs::read_to_string(&token_cache_path) {
        if let Ok(cache) = serde_json::from_str::<serde_json::Value>(&cache_content) {
            if let (Some(token), Some(expires_at)) =
                (cache["token"].as_str(), cache["expires_at"].as_str())
            {
                // Check if token is still valid (with 1 hour buffer)
                if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                    let now = chrono::Utc::now();
                    let buffer = chrono::Duration::hours(1);
                    if expires.signed_duration_since(now) > buffer {
                        return Ok(token.to_string());
                    }
                }
            }
        }
    }

    // Generate new token
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;

    let client = reqwest::Client::new();
    let payload = json!({
        "project_id": project_id,
    });

    let response = client
        .post(format!("http://localhost:{}/api/token/generate", port))
        .json(&payload)
        .send()
        .await
        .context("Failed to generate token")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to generate token: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    let token = data["token"]
        .as_str()
        .context("Token not found in response")?
        .to_string();

    // Cache the token (expires in 24 hours, cache for 23)
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(23);
    let cache = json!({
        "token": token,
        "expires_at": expires_at.to_rfc3339(),
        "project_id": project_id,
    });

    token_cache::write_secure_cache(&token_cache_path, &serde_json::to_string_pretty(&cache)?)
        .context("Failed to persist LLM router token cache securely")?;

    Ok(token)
}

/// Format output based on format type
fn format_output(data: &serde_json::Value, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        "human" => {
            format_human_readable(data)?;
        }
        _ => anyhow::bail!("Unknown format: {}", format),
    }
    Ok(())
}

/// Format output in human-readable format
fn format_human_readable(data: &serde_json::Value) -> Result<()> {
    // Handle stats response
    if let Some(endpoints) = data["endpoints"].as_array() {
        println!("🔀 LLM Routing Configuration\n");
        println!("Endpoints:");
        for endpoint in endpoints {
            if let (Some(name), Some(enabled), Some(url)) = (
                endpoint["name"].as_str(),
                endpoint["enabled"].as_bool(),
                endpoint["url"].as_str(),
            ) {
                let status = if enabled { "✅" } else { "❌" };
                println!("  {} {} - {}", status, name, url);
            }
        }
        println!();

        if let Some(architecture_tasks) = data["architectureTasks"].as_str() {
            println!("Architecture Tasks: {}", architecture_tasks);
        }
        if let Some(coding_tasks) = data["codingTasks"].as_str() {
            println!("Coding Tasks: {}", coding_tasks);
        }

        return Ok(());
    }

    // Handle route decision response
    if let Some(endpoint) = data["endpoint"].as_str() {
        println!("🎯 Routing Decision\n");
        println!("Endpoint: {}", endpoint);

        if let Some(url) = data["url"].as_str() {
            println!("URL: {}", url);
        }

        if let Some(use_claude) = data["useClaudeCode"].as_bool() {
            println!("Use Claude Code: {}", use_claude);
        }

        if let Some(reason) = data["reason"].as_str() {
            println!("Reason: {}", reason);
        }

        return Ok(());
    }

    // Handle call response
    if let Some(text) = data["text"].as_str() {
        println!("📝 LLM Response\n");
        println!("{}", text);

        if let Some(model) = data["model"].as_str() {
            println!("\nModel: {}", model);
        }

        return Ok(());
    }

    // Handle success/error responses
    if let Some(success) = data["success"].as_bool() {
        if success {
            println!("✅ Success");
            if let Some(message) = data["message"].as_str() {
                println!("   {}", message);
            }
        } else {
            println!("❌ Failed");
            if let Some(error) = data["error"].as_str() {
                println!("   Error: {}", error);
            }
        }
        return Ok(());
    }

    // Fallback to JSON output
    println!("{}", serde_json::to_string_pretty(data)?);

    Ok(())
}
