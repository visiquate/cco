//! CLI Integration for Orchestration
//!
//! Note: Orchestration server functionality has been merged into the daemon.
//! Use `cco orchestration` subcommands which now talk to the daemon's
//! /api/orchestration/* endpoints.
//!
//! This module provides the CLI types for backward compatibility.

use clap::{Parser, Subcommand};

/// Orchestration CLI commands
#[derive(Debug, Parser)]
#[command(name = "orchestration")]
#[command(about = "Manage orchestration system (now integrated into daemon)")]
pub struct OrchestrationCli {
    #[command(subcommand)]
    pub command: OrchestrationCommand,
}

#[derive(Debug, Subcommand)]
pub enum OrchestrationCommand {
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

    /// Show orchestration status
    Status,

    /// Clear context cache
    ClearCache {
        /// Issue ID to clear cache for
        issue_id: String,
    },
}

// Note: The execute() function has been removed.
// CLI execution is now handled in src/main.rs Commands::Orchestration
