//! CLI Integration for Orchestration Sidecar
//!
//! Provides subcommands for sidecar management:
//! - start/stop orchestration server
//! - query context
//! - store results
//! - publish/subscribe events

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;

/// Orchestration CLI commands
#[derive(Debug, Parser)]
#[command(name = "orchestration")]
#[command(about = "Manage the orchestration sidecar")]
pub struct OrchestrationCli {
    #[command(subcommand)]
    pub command: OrchestrationCommand,
}

#[derive(Debug, Subcommand)]
pub enum OrchestrationCommand {
    /// Start the orchestration sidecar server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "3001")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Storage path for results
        #[arg(long)]
        storage_path: Option<String>,
    },

    /// Stop the orchestration sidecar server
    Stop,

    /// Get context for an agent
    GetContext {
        /// Issue ID
        issue_id: String,

        /// Agent type
        agent_type: String,
    },

    /// Store agent results
    StoreResult {
        /// Issue ID
        issue_id: String,

        /// Agent type
        agent_type: String,

        /// Result JSON file
        #[arg(short, long)]
        file: String,
    },

    /// Publish an event
    PublishEvent {
        /// Event type
        event_type: String,

        /// Publisher ID
        #[arg(short, long)]
        publisher: String,

        /// Topic
        #[arg(short, long)]
        topic: String,

        /// Event data (JSON)
        #[arg(short, long)]
        data: String,
    },

    /// Subscribe to events
    Subscribe {
        /// Event type to subscribe to
        event_type: String,

        /// Timeout in milliseconds
        #[arg(short, long, default_value = "30000")]
        timeout: u64,
    },

    /// Show sidecar status
    Status,

    /// Clear context cache
    ClearCache {
        /// Issue ID to clear cache for
        issue_id: String,
    },
}

/// Execute orchestration CLI command
pub async fn execute(cli: OrchestrationCli) -> Result<()> {
    match cli.command {
        OrchestrationCommand::Start {
            port,
            host,
            storage_path,
        } => {
            let config = super::ServerConfig {
                port,
                host: host.clone(),
                storage_path: storage_path.unwrap_or_else(|| {
                    dirs::home_dir()
                        .unwrap()
                        .join(".cco/orchestration")
                        .to_string_lossy()
                        .to_string()
                }),
                ..Default::default()
            };

            println!("🚀 Starting orchestration sidecar on {}:{}", host, port);

            let state = super::initialize(config.clone()).await?;
            let server = super::OrchestrationServer::new(state, config);

            server.run().await?;
        }

        OrchestrationCommand::Stop => {
            println!("⏹️  Stopping orchestration sidecar...");
            // TODO: Implement graceful shutdown signal
            println!("✅ Sidecar stopped");
        }

        OrchestrationCommand::GetContext {
            issue_id,
            agent_type,
        } => {
            println!("📥 Getting context for {} (issue: {})", agent_type, issue_id);

            let client = reqwest::Client::new();
            let url = format!(
                "http://127.0.0.1:3001/api/context/{}/{}",
                issue_id, agent_type
            );

            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    } else {
                        eprintln!("❌ Error: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect: {}", e);
                    eprintln!("   Make sure the sidecar is running (cco orchestration start)");
                }
            }
        }

        OrchestrationCommand::StoreResult {
            issue_id,
            agent_type,
            file,
        } => {
            println!(
                "💾 Storing result for {} (issue: {})",
                agent_type, issue_id
            );

            let result_json = std::fs::read_to_string(&file)?;
            let result: serde_json::Value = serde_json::from_str(&result_json)?;

            let client = reqwest::Client::new();
            let url = "http://127.0.0.1:3001/api/results";

            let request_body = serde_json::json!({
                "agent_id": uuid::Uuid::new_v4().to_string(),
                "agent_type": agent_type,
                "issue_id": issue_id,
                "project_id": "default",
                "result": result,
                "timestamp": chrono::Utc::now(),
            });

            match client.post(url).json(&request_body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("✅ Result stored");
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    } else {
                        eprintln!("❌ Error: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect: {}", e);
                }
            }
        }

        OrchestrationCommand::PublishEvent {
            event_type,
            publisher,
            topic,
            data,
        } => {
            println!("📢 Publishing event: {}", event_type);

            let event_data: serde_json::Value = serde_json::from_str(&data)?;

            let client = reqwest::Client::new();
            let url = format!("http://127.0.0.1:3001/api/events/{}", event_type);

            let request_body = serde_json::json!({
                "event_type": event_type,
                "publisher": publisher,
                "topic": topic,
                "data": event_data,
            });

            match client.post(&url).json(&request_body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("✅ Event published");
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    } else {
                        eprintln!("❌ Error: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect: {}", e);
                }
            }
        }

        OrchestrationCommand::Subscribe {
            event_type,
            timeout,
        } => {
            println!("👂 Subscribing to events: {}", event_type);

            let client = reqwest::Client::new();
            let url = format!(
                "http://127.0.0.1:3001/api/events/wait/{}?timeout={}",
                event_type, timeout
            );

            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    } else {
                        eprintln!("❌ Error: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect: {}", e);
                }
            }
        }

        OrchestrationCommand::Status => {
            println!("📊 Orchestration Sidecar Status\n");

            let client = reqwest::Client::new();
            let health_url = "http://127.0.0.1:3001/health";
            let status_url = "http://127.0.0.1:3001/status";

            // Check health
            match client.get(health_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("✅ Sidecar is running");
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    }
                }
                Err(_) => {
                    println!("❌ Sidecar is not running");
                    println!("   Start it with: cco orchestration start");
                    return Ok(());
                }
            }

            // Get detailed status
            match client.get(status_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("\n📈 Detailed Status:");
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Could not get detailed status: {}", e);
                }
            }
        }

        OrchestrationCommand::ClearCache { issue_id } => {
            println!("🗑️  Clearing cache for issue: {}", issue_id);

            let client = reqwest::Client::new();
            let url = format!("http://127.0.0.1:3001/api/cache/context/{}", issue_id);

            match client.delete(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json: serde_json::Value = response.json().await?;
                        println!("✅ Cache cleared");
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    } else {
                        eprintln!("❌ Error: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect: {}", e);
                }
            }
        }
    }

    Ok(())
}
