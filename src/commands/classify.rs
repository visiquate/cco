//! CRUD Classifier CLI commands
//!
//! Provides CLI wrapper around the daemon's CRUD classifier functionality.
//! Allows testing and feedback collection for classifier improvements.

use anyhow::{Context, Result};
use cco::daemon::lifecycle::read_daemon_port;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

/// Corrections storage structure
#[derive(Debug, Serialize, Deserialize)]
struct CorrectionsStore {
    corrections: Vec<CorrectionEntry>,
    version: String,
    last_updated: String,
}

/// A single correction entry
#[derive(Debug, Serialize, Deserialize)]
struct CorrectionEntry {
    command: String,
    predicted: String,
    expected: String,
    confidence: f32,
    timestamp: String,
}

impl CorrectionsStore {
    fn new() -> Self {
        Self {
            corrections: Vec::new(),
            version: "1.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn add(&mut self, entry: CorrectionEntry) {
        self.corrections.push(entry);
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    fn clear(&mut self) {
        self.corrections.clear();
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
}

/// Get corrections file path
fn get_corrections_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".cco")
        .join("classifier-corrections.json")
}

/// Load corrections from file
fn load_corrections() -> Result<CorrectionsStore> {
    let path = get_corrections_path();

    if !path.exists() {
        return Ok(CorrectionsStore::new());
    }

    let content = std::fs::read_to_string(&path)
        .context("Failed to read corrections file")?;

    let store = serde_json::from_str(&content)
        .context("Failed to parse corrections file")?;

    Ok(store)
}

/// Save corrections to file
fn save_corrections(store: &CorrectionsStore) -> Result<()> {
    let path = get_corrections_path();

    // Ensure .cco directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create .cco directory")?;
    }

    let content = serde_json::to_string_pretty(store)
        .context("Failed to serialize corrections")?;

    std::fs::write(&path, content)
        .context("Failed to write corrections file")?;

    Ok(())
}

/// Parse expected classification from string
fn parse_expected(expected: &str) -> Result<String> {
    let normalized = expected.to_uppercase();

    match normalized.as_str() {
        "R" | "READ" => Ok("Read".to_string()),
        "C" | "CREATE" => Ok("Create".to_string()),
        "U" | "UPDATE" => Ok("Update".to_string()),
        "D" | "DELETE" => Ok("Delete".to_string()),
        _ => anyhow::bail!("Invalid expected classification: '{}'. Use: read/create/update/delete (or r/c/u/d)", expected),
    }
}

/// Run classify command
pub async fn run(
    command: Option<String>,
    expected: Option<String>,
    format: String,
    list_corrections: bool,
    clear_corrections: bool,
    export_corrections: Option<String>,
    last: bool,
) -> Result<()> {
    // Handle list corrections
    if list_corrections {
        return list_corrections_cmd();
    }

    // Handle clear corrections
    if clear_corrections {
        return clear_corrections_cmd();
    }

    // Handle export corrections
    if let Some(path) = export_corrections {
        return export_corrections_cmd(&path);
    }

    // Handle --last flag
    if last {
        return reclassify_last(expected.as_deref(), &format).await;
    }

    // Require command for classification
    let command = command.context("Command is required for classification")?;

    // Classify the command
    classify_command(&command, expected.as_deref(), &format).await
}

/// Reclassify the last classified command
async fn reclassify_last(expected: Option<&str>, format: &str) -> Result<()> {
    let port = read_daemon_port()
        .context("Daemon not running. Start it with: cco daemon start")?;

    // Fetch last classified command from daemon
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/api/classify/last", port))
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to fetch last classified command: {}", error_text);
    }

    #[derive(Deserialize)]
    struct LastClassifiedResponse {
        command: String,
        classification: String,
    }

    let last: LastClassifiedResponse = response.json().await?;

    println!("Reclassifying last command: {}", last.command);
    println!("Previous classification: {}", last.classification);
    println!();

    // If expected is provided, check if it matches and store correction
    if let Some(expected_str) = expected {
        let expected_normalized = parse_expected(expected_str)?;

        if last.classification != expected_normalized {
            // Store correction
            let mut store = load_corrections()?;
            store.add(CorrectionEntry {
                command: last.command.clone(),
                predicted: last.classification.clone(),
                expected: expected_normalized.clone(),
                confidence: 0.0, // We don't have confidence from the last classification
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
            save_corrections(&store)?;

            println!("✅ Reclassified '{}' from {} to {} - Correction saved",
                     last.command, last.classification, expected_normalized);
        } else {
            println!("✅ Classification matches expected: {}", last.classification);
        }
    } else {
        // Just re-run classification without expected value
        classify_command(&last.command, None, format).await?;
    }

    Ok(())
}

/// Classify a command
async fn classify_command(
    command: &str,
    expected: Option<&str>,
    format: &str,
) -> Result<()> {
    let port = read_daemon_port()
        .context("Daemon not running. Start it with: cco daemon start")?;

    let client = reqwest::Client::new();
    let payload = json!({
        "command": command,
    });

    let response = client
        .post(format!("http://localhost:{}/api/classify", port))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to daemon")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Classification failed: {}", error_text);
    }

    let data: serde_json::Value = response.json().await?;

    // Extract classification result
    let predicted = data["classification"]
        .as_str()
        .context("Missing classification in response")?
        .to_string();

    let confidence = data["confidence"]
        .as_f64()
        .context("Missing confidence in response")? as f32;

    // If expected is provided, store correction if mismatch
    if let Some(expected_str) = expected {
        let expected_normalized = parse_expected(expected_str)?;

        if predicted != expected_normalized {
            // Store correction
            let mut store = load_corrections()?;
            store.add(CorrectionEntry {
                command: command.to_string(),
                predicted: predicted.clone(),
                expected: expected_normalized.clone(),
                confidence,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
            save_corrections(&store)?;

            println!("⚠️  Mismatch detected! Correction stored.");
            println!("   Predicted: {}", predicted);
            println!("   Expected:  {}", expected_normalized);
        } else {
            println!("✅ Classification matches expected: {}", predicted);
        }
    }

    // Format and display output
    format_output(&data, format)?;

    Ok(())
}

/// List all stored corrections
fn list_corrections_cmd() -> Result<()> {
    let store = load_corrections()?;

    if store.corrections.is_empty() {
        println!("No corrections stored yet.");
        return Ok(());
    }

    println!("📋 Stored Corrections ({} total)\n", store.corrections.len());

    for (i, entry) in store.corrections.iter().enumerate() {
        println!("{}. Command: {}", i + 1, entry.command);
        println!("   Predicted: {} (confidence: {:.2})", entry.predicted, entry.confidence);
        println!("   Expected:  {}", entry.expected);
        println!("   Timestamp: {}", entry.timestamp);
        println!();
    }

    println!("Last updated: {}", store.last_updated);

    Ok(())
}

/// Clear all stored corrections
fn clear_corrections_cmd() -> Result<()> {
    let mut store = load_corrections()?;
    let count = store.corrections.len();

    store.clear();
    save_corrections(&store)?;

    println!("✅ Cleared {} corrections", count);

    Ok(())
}

/// Export corrections to a file
fn export_corrections_cmd(path: &str) -> Result<()> {
    let store = load_corrections()?;

    if store.corrections.is_empty() {
        println!("No corrections to export.");
        return Ok(());
    }

    let content = serde_json::to_string_pretty(&store)
        .context("Failed to serialize corrections")?;

    std::fs::write(path, content)
        .context("Failed to write export file")?;

    println!("✅ Exported {} corrections to {}", store.corrections.len(), path);

    Ok(())
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
        _ => anyhow::bail!("Unknown format: {}. Use 'json' or 'human'", format),
    }
    Ok(())
}

/// Format output in human-readable format
fn format_human_readable(data: &serde_json::Value) -> Result<()> {
    let classification = data["classification"]
        .as_str()
        .context("Missing classification")?;

    let confidence = data["confidence"]
        .as_f64()
        .context("Missing confidence")?;

    let reasoning = data["reasoning"]
        .as_str()
        .unwrap_or("No reasoning provided");

    println!("🔍 CRUD Classification Result\n");
    println!("Classification: {}", classification);
    println!("Confidence:     {:.1}%", confidence * 100.0);
    println!("Reasoning:      {}", reasoning);

    Ok(())
}
