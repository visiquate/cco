//! Credential management CLI commands
//!
//! Provides CLI wrapper around the daemon's credential storage.
//! Note: Daemon API endpoints for credentials need to be implemented.
//! This follows the same pattern as knowledge.rs for future integration.

use anyhow::{Context, Result};
use cco::daemon::lifecycle::read_daemon_port;
use serde_json::json;

use super::token_cache;
use crate::CredentialAction;

/// Run credential management command
pub async fn run(action: CredentialAction) -> Result<()> {
    match action {
        CredentialAction::Store { key, value } => store(key, value).await,
        CredentialAction::Retrieve { key } => retrieve(key).await,
        CredentialAction::Delete { key } => delete(key).await,
        CredentialAction::List => list().await,
        CredentialAction::CheckRotation => check_rotation().await,
    }
}

/// Store a credential
async fn store(key: String, value: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "key": key,
        "value": value,
        "project_id": project_id,
    });

    let response = client
        .post(format!("http://localhost:{}/api/credentials/store", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to store credential: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;

    if data["success"].as_bool().unwrap_or(false) {
        println!("✅ Credential stored: {}", key);
    } else {
        let error = data["error"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("Failed to store credential: {}", error);
    }

    Ok(())
}

/// Retrieve a credential
async fn retrieve(key: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "key": key,
        "project_id": project_id,
    });

    let response = client
        .post(format!(
            "http://localhost:{}/api/credentials/retrieve",
            port
        ))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to retrieve credential: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;

    if data["success"].as_bool().unwrap_or(false) {
        if let Some(value) = data["value"].as_str() {
            println!("{}", value);
        } else {
            anyhow::bail!("Credential value not found in response");
        }
    } else {
        let error = data["error"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("Failed to retrieve credential: {}", error);
    }

    Ok(())
}

/// Delete a credential
async fn delete(key: String) -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "key": key,
        "project_id": project_id,
    });

    let response = client
        .delete(format!("http://localhost:{}/api/credentials/delete", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to delete credential: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;

    if data["success"].as_bool().unwrap_or(false) {
        println!("✅ Credential deleted: {}", key);
    } else {
        let error = data["error"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("Failed to delete credential: {}", error);
    }

    Ok(())
}

/// List all credentials
async fn list() -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "project_id": project_id,
    });

    let response = client
        .post(format!("http://localhost:{}/api/credentials/list", port))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to list credentials: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;

    if data["success"].as_bool().unwrap_or(false) {
        if let Some(credentials) = data["credentials"].as_array() {
            if credentials.is_empty() {
                println!("No credentials stored");
            } else {
                println!("📋 Stored credentials:");
                for cred in credentials {
                    if let Some(key) = cred["key"].as_str() {
                        let credential_type = cred["type"].as_str().unwrap_or("unknown");
                        let last_rotated = cred["last_rotated"].as_str().unwrap_or("never");
                        println!(
                            "   {} (type: {}, rotated: {})",
                            key, credential_type, last_rotated
                        );
                    }
                }
            }
        } else {
            anyhow::bail!("Invalid credentials list in response");
        }
    } else {
        let error = data["error"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("Failed to list credentials: {}", error);
    }

    Ok(())
}

/// Check credential rotation status
async fn check_rotation() -> Result<()> {
    let port = read_daemon_port().context("Daemon not running. Start it with: cco daemon start")?;
    let project_id = auto_detect_project_id()?;
    let token = get_or_generate_token(&project_id).await?;

    let client = reqwest::Client::new();
    let payload = json!({
        "project_id": project_id,
    });

    let response = client
        .post(format!(
            "http://localhost:{}/api/credentials/check-rotation",
            port
        ))
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to check rotation: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;

    if data["success"].as_bool().unwrap_or(false) {
        if let Some(status) = data["status"].as_array() {
            if status.is_empty() {
                println!("✅ All credentials are up to date");
            } else {
                println!("⚠️  Credentials needing rotation:");
                for item in status {
                    if let Some(key) = item["key"].as_str() {
                        let days_old = item["days_old"].as_i64().unwrap_or(0);
                        let recommended =
                            item["recommended_action"].as_str().unwrap_or("rotate soon");
                        println!("   {} - {} days old ({})", key, days_old, recommended);
                    }
                }
            }
        }
    } else {
        let error = data["error"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("Failed to check rotation: {}", error);
    }

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
                return Ok(std::path::PathBuf::from(path)
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
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".cco")
        .join("credentials-token.json");

    token_cache::tighten_permissions_if_exists(&token_cache_path)
        .context("Failed to secure cached credential token permissions")?;

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
        .context("Failed to persist credential token cache securely")?;

    Ok(token)
}
