//! Knowledge management CLI commands
//!
//! Provides CLI wrapper around the daemon's LanceDB knowledge store.
//! Replaces the JavaScript knowledge-manager.js with native Rust implementation.

use anyhow::{Context, Result};
use cco::daemon::lifecycle::read_daemon_port;
use serde_json::json;
use std::path::PathBuf;

use crate::KnowledgeAction;

/// Run knowledge management command
pub async fn run(action: KnowledgeAction) -> Result<()> {
    match action {
        KnowledgeAction::Store {
            text,
            r#type,
            agent,
            format,
        } => store(text, r#type, agent, format).await,
        KnowledgeAction::Search {
            query,
            limit,
            format,
        } => search(query, limit, format).await,
        KnowledgeAction::Stats { format } => stats(format).await,
        KnowledgeAction::PreCompaction {
            conversation,
            format,
        } => pre_compaction(conversation, format).await,
        KnowledgeAction::PostCompaction {
            task,
            limit,
            format,
        } => post_compaction(task, limit, format).await,
    }
}

/// Store a knowledge item
async fn store(
    text: String,
    knowledge_type: Option<String>,
    agent: Option<String>,
    format: String,
) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "text": text,
        "knowledge_type": knowledge_type,
        "project_id": project_id,
        "agent": agent,
    });

    let response = client
        .post(format!("http://localhost:{}/api/knowledge/store", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to store knowledge: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Search knowledge base
async fn search(query: String, limit: usize, format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "query": query,
        "limit": limit,
        "project_id": project_id,
    });

    let response = client
        .post(format!("http://localhost:{}/api/knowledge/search", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to search knowledge: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Show knowledge statistics
async fn stats(format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let token = get_or_generate_token(&auto_detect_project_id()?).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/api/knowledge/stats", port))
        .bearer_auth(token)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to get stats: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Pre-compaction knowledge capture
async fn pre_compaction(conversation: String, format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    // Check if conversation is a file path
    let conversation_text = if std::path::Path::new(&conversation).exists() {
        std::fs::read_to_string(&conversation).context(format!(
            "Failed to read conversation file: {}",
            conversation
        ))?
    } else {
        conversation
    };

    let client = reqwest::Client::new();
    let payload = json!({
        "conversation": conversation_text,
        "project_id": project_id,
    });

    let response = client
        .post(format!(
            "http://localhost:{}/api/knowledge/pre-compaction",
            port
        ))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to run pre-compaction: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;
    format_output(&data, &format)?;

    Ok(())
}

/// Post-compaction knowledge retrieval
async fn post_compaction(task: String, limit: usize, format: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "current_task": task,
        "project_id": project_id,
        "limit": limit,
    });

    let response = client
        .post(format!(
            "http://localhost:{}/api/knowledge/post-compaction",
            port
        ))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to run post-compaction: {}", error_text);
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
        .join("knowledge-token.json");

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

    // Ensure .cco directory exists
    if let Some(parent) = token_cache_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    std::fs::write(&token_cache_path, serde_json::to_string_pretty(&cache)?).ok();

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
    // Handle different response types
    if let Some(success) = data["success"].as_bool() {
        if success {
            println!("✅ Success");

            // Store response
            if let Some(id) = data["id"].as_str() {
                println!("   ID: {}", id);
            }

            // Search results
            if let Some(results) = data["results"].as_array() {
                println!("   Found {} results:", results.len());
                for (i, result) in results.iter().enumerate() {
                    if let Some(text) = result["text"].as_str() {
                        println!("   {}. {}", i + 1, text);
                        if let Some(knowledge_type) = result["type"].as_str() {
                            println!("      Type: {}", knowledge_type);
                        }
                    }
                }
            }

            // Pre-compaction response
            if let Some(count) = data["count"].as_u64() {
                println!("   Captured {} knowledge items", count);
            }

            // Post-compaction response
            if let Some(summary) = data["summary"].as_str() {
                println!("   {}", summary);
            }
        } else {
            println!("❌ Failed");
            if let Some(error) = data["error"].as_str() {
                println!("   Error: {}", error);
            }
        }
    } else if let Some(stats) = data.as_object() {
        // Stats response
        println!("📊 Knowledge Base Statistics:");
        for (key, value) in stats {
            println!("   {}: {}", key, value);
        }
    } else {
        // Generic JSON output
        println!("{}", serde_json::to_string_pretty(data)?);
    }

    Ok(())
}
