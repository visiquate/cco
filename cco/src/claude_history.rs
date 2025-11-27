//! Claude conversation history parser for extracting metrics from JSONL files
//!
//! This module parses Claude conversation history files located in
//! ~/.claude/projects/{project-name}/ and extracts token usage and cost metrics.

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tracing::{debug, error, info, warn};

/// Per-model breakdown of usage and costs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_write_cost: f64,
    pub cache_read_cost: f64,
    pub total_cost: f64,
    pub message_count: u64,
}

/// Per-project breakdown of usage and costs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectBreakdown {
    pub name: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost: f64,
    pub message_count: u64,
    pub conversation_count: u64,
    pub models: HashMap<String, ModelBreakdown>,
}

impl Default for ProjectBreakdown {
    fn default() -> Self {
        Self {
            name: String::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_creation_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 0.0,
            message_count: 0,
            conversation_count: 0,
            models: HashMap::new(),
        }
    }
}

impl Default for ModelBreakdown {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            input_cost: 0.0,
            output_cost: 0.0,
            cache_write_cost: 0.0,
            cache_read_cost: 0.0,
            total_cost: 0.0,
            message_count: 0,
        }
    }
}

/// Aggregated metrics from Claude conversation history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClaudeMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost: f64,
    pub messages_count: u64,
    pub conversations_count: u64,
    pub model_breakdown: HashMap<String, ModelBreakdown>,
    pub project_breakdown: HashMap<String, ProjectBreakdown>,
    pub last_updated: DateTime<Utc>,
}

impl Default for ClaudeMetrics {
    fn default() -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_creation_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 0.0,
            messages_count: 0,
            conversations_count: 0,
            model_breakdown: HashMap::new(),
            project_breakdown: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Internal structure for parsing Claude message usage data
#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    #[serde(rename = "type")]
    message_type: String,
    message: Option<MessageContent>,
    timestamp: Option<String>,  // ISO 8601 timestamp (e.g., "2025-11-11T22:35:42.543Z")
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    model: Option<String>,
    usage: Option<UsageData>,
}

#[derive(Debug, Deserialize)]
pub struct UsageData {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

/// Model pricing structure matching Python live-monitor.py
///
/// Cache tokens pricing model:
/// - cache_creation_input_tokens: 25% more expensive than regular input (storage write cost)
/// - cache_read_input_tokens: 10% of regular input cost (90% discount - cached content doesn't consume full tokens)
///
/// Returns: (input_price, output_price, cache_write_price, cache_read_price) per million tokens
pub fn get_model_pricing(model_name: &str) -> (f64, f64, f64, f64) {
    // Normalize model name by extracting the base model
    let normalized = normalize_model_name(model_name);

    match normalized.as_str() {
        // Synthetic/Error messages from Claude Code infrastructure
        // These have 0 token usage and represent errors or system responses, not actual API calls
        "<synthetic>" => {
            // These are internal error/synthetic messages with 0 tokens - no charge
            // They appear when agents encounter API errors or infrastructure responses
            (0.0, 0.0, 0.0, 0.0)
        }
        // Sonnet 4.5 variants
        "claude-sonnet-4-5" | "claude-3-5-sonnet" => {
            let input = 3.0;
            let output = 15.0;
            let cache_write = 3.75;  // 25% premium over input
            let cache_read = 0.30;   // 90% discount (10% of input)
            (input, output, cache_write, cache_read)
        }
        // Haiku 4.5 variants
        "claude-haiku-4-5" | "claude-3-5-haiku" => {
            let input = 1.0;
            let output = 5.0;
            let cache_write = 1.25;  // 25% premium over input
            let cache_read = 0.10;   // 90% discount (10% of input)
            (input, output, cache_write, cache_read)
        }
        // Opus 4 variants
        "claude-opus-4" | "claude-opus-4-1" => {
            let input = 15.0;
            let output = 75.0;
            let cache_write = 18.75; // 25% premium over input
            let cache_read = 1.50;   // 90% discount (10% of input)
            (input, output, cache_write, cache_read)
        }
        _ => {
            // Log unknown models at debug level
            // Note: Most "unknown" models are actually <synthetic> infrastructure placeholders
            // which already have 0 token usage and won't affect pricing calculations
            tracing::debug!(
                "Model pricing defaulting to Sonnet 4.5 for unknown model: {}. \
                 If this is not a <synthetic> infrastructure event, verify model name.",
                model_name
            );
            (3.0, 15.0, 3.75, 0.30) // Default to Sonnet pricing
        }
    }
}

/// Normalize model name for pricing lookup
/// Examples:
///   "claude-sonnet-4-5-20250929" → "claude-sonnet-4-5"
///   "claude-3-5-sonnet-20241022" → "claude-3-5-sonnet"
///   "claude-opus-4-20250514" → "claude-opus-4"
///   "claude-opus-4-1" → "claude-opus-4-1"
///   "claude-haiku-4-5-20251001" → "claude-haiku-4-5"
///   "claude-3-5-haiku-20241022" → "claude-3-5-haiku"
pub fn normalize_model_name(model_name: &str) -> String {
    // Remove date suffix if present (format: -YYYYMMDD)
    let parts: Vec<&str> = model_name.split('-').collect();

    // Common patterns:
    // claude-sonnet-4-5-20250929 → claude-sonnet-4-5
    // claude-3-5-sonnet-20241022 → claude-3-5-sonnet
    // claude-opus-4-20250514 → claude-opus-4
    // claude-opus-4-1 → claude-opus-4-1 (no date, keep as-is)
    // claude-haiku-4-5-20251001 → claude-haiku-4-5
    // claude-3-5-haiku-20241022 → claude-3-5-haiku

    if parts.len() >= 3 {
        // Check if last part looks like a date (8 digits)
        if let Some(last) = parts.last() {
            if last.len() == 8 && last.chars().all(|c| c.is_ascii_digit()) {
                // Remove the date part
                return parts[..parts.len() - 1].join("-");
            }
        }
    }

    model_name.to_string()
}

/// Calculate cost for a given token count
pub fn calculate_cost(tokens: u64, cost_per_million: f64) -> f64 {
    (tokens as f64 / 1_000_000.0) * cost_per_million
}

/// Parse a single JSONL line and extract message data
fn parse_jsonl_line(line: &str) -> Option<ClaudeMessage> {
    match serde_json::from_str::<ClaudeMessage>(line) {
        Ok(msg) => Some(msg),
        Err(e) => {
            debug!("Failed to parse JSONL line: {}", e);
            None
        }
    }
}

/// Parse file with exponential backoff retry logic
///
/// Retries up to `max_retries` times with exponential backoff on transient errors
async fn parse_jsonl_file_with_retry(
    path: &Path,
    max_retries: u32,
) -> Result<Vec<(String, UsageData, Option<String>)>> {
    let mut retries = 0;

    loop {
        match parse_jsonl_file(path).await {
            Ok(messages) => {
                if retries > 0 {
                    debug!("Parse succeeded after {} retries: {:?}", retries, path);
                }
                return Ok(messages);
            }
            Err(e) if retries < max_retries => {
                let delay = Duration::from_secs(2u64.pow(retries));
                warn!(
                    "Parse failed (attempt {}), retrying in {:?}: {} - {:?}",
                    retries + 1,
                    delay,
                    e,
                    path
                );
                tokio::time::sleep(delay).await;
                retries += 1;
            }
            Err(e) => {
                error!("Parse failed after {} retries: {} - {:?}", retries, e, path);
                return Err(e);
            }
        }
    }
}

/// Parse a single JSONL file and extract assistant messages with usage data and timestamps
async fn parse_jsonl_file(path: &Path) -> Result<Vec<(String, UsageData, Option<String>)>> {
    let file = fs::File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut messages = Vec::new();

    while let Some(line) = lines.next_line().await? {
        if let Some(msg) = parse_jsonl_line(&line) {
            // Only process assistant messages with usage data
            if msg.message_type == "assistant" {
                if let Some(content) = msg.message {
                    if let (Some(model), Some(usage)) = (content.model, content.usage) {
                        messages.push((model, usage, msg.timestamp));
                    }
                }
            }
        }
    }

    Ok(messages)
}

/// Parse a JSONL file incrementally from a specific byte offset
///
/// This function enables incremental parsing by seeking to the last known position
/// and only parsing new lines. This dramatically reduces parsing time when files
/// are frequently re-parsed but rarely change.
///
/// # Arguments
/// * `path` - Path to the JSONL file
/// * `byte_offset` - Byte position to start parsing from (0 = start of file)
///
/// # Returns
/// Tuple of (messages, new_byte_offset, new_line_count)
/// - messages: Newly parsed messages since byte_offset
/// - new_byte_offset: Updated byte position after parsing
/// - new_line_count: Total number of lines processed in this call
///
/// # Performance
/// - Full file parse: ~100ms for large files
/// - Incremental parse (no changes): <10ms
/// - Incremental parse (few new lines): <20ms
///
/// # Example
/// ```no_run
/// use cco::claude_history::parse_jsonl_file_from_offset;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let path = Path::new("/path/to/file.jsonl");
///     let (messages, new_offset, line_count) = parse_jsonl_file_from_offset(path, 0).await?;
///     println!("Parsed {} messages from {} lines", messages.len(), line_count);
///     Ok(())
/// }
/// ```
pub async fn parse_jsonl_file_from_offset(
    path: &Path,
    byte_offset: u64,
) -> Result<(Vec<(String, UsageData, Option<String>)>, u64, usize)> {
    let file = fs::File::open(path).await?;
    let mut reader = BufReader::new(file);

    // Seek to the last known position
    if byte_offset > 0 {
        reader.seek(tokio::io::SeekFrom::Start(byte_offset)).await?;
    }

    let mut lines = reader.lines();
    let mut messages = Vec::new();
    let mut current_offset = byte_offset;
    let mut line_count = 0;

    while let Some(line) = lines.next_line().await? {
        let line_bytes = line.len() as u64 + 1; // +1 for newline
        current_offset += line_bytes;
        line_count += 1;

        if let Some(msg) = parse_jsonl_line(&line) {
            // Only process assistant messages with usage data
            if msg.message_type == "assistant" {
                if let Some(content) = msg.message {
                    if let (Some(model), Some(usage)) = (content.model, content.usage) {
                        messages.push((model, usage, msg.timestamp));
                    }
                }
            }
        }
    }

    Ok((messages, current_offset, line_count))
}

/// Load Claude metrics from all projects in ~/.claude/projects/
///
/// Scans all project directories and aggregates metrics by project and model.
///
/// # Returns
/// ClaudeMetrics struct with aggregated usage, cost data, and breakdowns by project and model
///
/// # Example
/// ```no_run
/// use cco::claude_history::load_claude_metrics_from_home_dir;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let metrics = load_claude_metrics_from_home_dir().await?;
///     println!("Total cost: ${:.2}", metrics.total_cost);
///     println!("Total API calls: {}", metrics.messages_count);
///     Ok(())
/// }
/// ```
pub async fn load_claude_metrics_from_home_dir() -> Result<ClaudeMetrics> {
    // Get home directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let projects_dir = format!("{}/.claude/projects", home);
    let projects_path = Path::new(&projects_dir);

    if !projects_path.exists() {
        tracing::debug!("Claude projects directory does not exist: {}", projects_dir);
        return Ok(ClaudeMetrics::default());
    }

    let mut metrics = ClaudeMetrics::default();

    // Read all project directories
    let mut entries = fs::read_dir(projects_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Only process directories
        if !path.is_dir() {
            continue;
        }

        // Get project name from directory name
        let project_name = entry.file_name().to_string_lossy().to_string();

        // Parse project metrics
        match load_claude_project_metrics(path.to_str().unwrap()).await {
            Ok(project_metrics) => {
                // Aggregate totals
                metrics.total_input_tokens += project_metrics.total_input_tokens;
                metrics.total_output_tokens += project_metrics.total_output_tokens;
                metrics.total_cache_creation_tokens += project_metrics.total_cache_creation_tokens;
                metrics.total_cache_read_tokens += project_metrics.total_cache_read_tokens;
                metrics.total_cost += project_metrics.total_cost;
                metrics.messages_count += project_metrics.messages_count;
                metrics.conversations_count += project_metrics.conversations_count;

                // Aggregate model breakdown
                for (model, breakdown) in &project_metrics.model_breakdown {
                    let entry = metrics.model_breakdown
                        .entry(model.clone())
                        .or_insert_with(ModelBreakdown::default);

                    entry.input_tokens += breakdown.input_tokens;
                    entry.output_tokens += breakdown.output_tokens;
                    entry.cache_creation_tokens += breakdown.cache_creation_tokens;
                    entry.cache_read_tokens += breakdown.cache_read_tokens;
                    entry.input_cost += breakdown.input_cost;
                    entry.output_cost += breakdown.output_cost;
                    entry.cache_write_cost += breakdown.cache_write_cost;
                    entry.cache_read_cost += breakdown.cache_read_cost;
                    entry.total_cost += breakdown.total_cost;
                    entry.message_count += breakdown.message_count;
                }

                // Add project breakdown (if there are any messages)
                if project_metrics.messages_count > 0 {
                    metrics.project_breakdown.insert(
                        project_name.clone(),
                        ProjectBreakdown {
                            name: project_name,
                            total_input_tokens: project_metrics.total_input_tokens,
                            total_output_tokens: project_metrics.total_output_tokens,
                            total_cache_creation_tokens: project_metrics.total_cache_creation_tokens,
                            total_cache_read_tokens: project_metrics.total_cache_read_tokens,
                            total_cost: project_metrics.total_cost,
                            message_count: project_metrics.messages_count,
                            conversation_count: project_metrics.conversations_count,
                            models: project_metrics.model_breakdown.clone(),
                        },
                    );
                }
            }
            Err(e) => {
                tracing::debug!("Failed to load project metrics for {}: {}", project_name, e);
                // Continue processing other projects
            }
        }
    }

    metrics.last_updated = Utc::now();
    Ok(metrics)
}

/// Load Claude metrics from home directory metrics.json file (legacy format)
///
/// This loads the older Claude Code metrics format from ~/.claude/metrics.json
///
/// # Returns
/// ClaudeMetrics struct with aggregated usage and cost data
///
/// # Example
/// ```no_run
/// use cco::claude_history::load_claude_metrics_from_metrics_json;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let metrics = load_claude_metrics_from_metrics_json().await?;
///     println!("Total cost: ${:.2}", metrics.total_cost);
///     println!("Total API calls: {}", metrics.messages_count);
///     Ok(())
/// }
/// ```
pub async fn load_claude_metrics_from_metrics_json() -> Result<ClaudeMetrics> {
    // Get home directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let metrics_path = format!("{}/.claude/metrics.json", home);
    let metrics_file = Path::new(&metrics_path);

    if !metrics_file.exists() {
        tracing::debug!("Claude metrics file does not exist: {}", metrics_path);
        return Ok(ClaudeMetrics::default());
    }

    // Read and parse the JSON array
    let content = fs::read_to_string(metrics_file).await?;

    #[derive(Debug, Deserialize)]
    struct MetricsEntry {
        model: String,
        input_tokens: u64,
        output_tokens: u64,
        #[serde(default)]
        #[allow(dead_code)]
        cache_hit: bool,
        actual_cost: f64,
        #[serde(default)]
        #[allow(dead_code)]
        would_be_cost: f64,
        #[serde(default)]
        #[allow(dead_code)]
        savings: f64,
    }

    let entries: Vec<MetricsEntry> = serde_json::from_str(&content)?;

    let mut metrics = ClaudeMetrics::default();

    for entry in entries {
        let normalized_model = normalize_model_name(&entry.model);
        let (input_price, output_price, _cache_write_price, _cache_read_price) =
            get_model_pricing(&normalized_model);

        // Aggregate totals
        metrics.total_input_tokens += entry.input_tokens;
        metrics.total_output_tokens += entry.output_tokens;
        metrics.total_cost += entry.actual_cost;
        metrics.messages_count += 1;

        // Update per-model breakdown
        let breakdown = metrics.model_breakdown
            .entry(normalized_model.clone())
            .or_insert_with(ModelBreakdown::default);

        breakdown.input_tokens += entry.input_tokens;
        breakdown.output_tokens += entry.output_tokens;
        breakdown.total_cost += entry.actual_cost;
        breakdown.message_count += 1;

        // Calculate individual cost components for the breakdown
        // Note: The actual_cost from metrics.json is the authoritative total
        // We distribute it proportionally based on token pricing
        let total_cost_estimate =
            calculate_cost(entry.input_tokens, input_price) +
            calculate_cost(entry.output_tokens, output_price);

        if total_cost_estimate > 0.0 {
            let input_ratio = calculate_cost(entry.input_tokens, input_price) / total_cost_estimate;
            let output_ratio = calculate_cost(entry.output_tokens, output_price) / total_cost_estimate;

            breakdown.input_cost += entry.actual_cost * input_ratio;
            breakdown.output_cost += entry.actual_cost * output_ratio;
        }
    }

    metrics.conversations_count = 1; // Single metrics file represents current session
    metrics.last_updated = Utc::now();

    Ok(metrics)
}

/// Load Claude project metrics from a project directory
///
/// # Arguments
/// * `project_path` - Path to the Claude project directory (e.g., ~/.claude/projects/my-project)
///
/// # Returns
/// ClaudeMetrics struct with aggregated usage and cost data
///
/// # Example
/// ```no_run
/// use cco::claude_history::load_claude_project_metrics;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let metrics = load_claude_project_metrics(
///         "/Users/brent/.claude/projects/cc-orchestra"
///     ).await?;
///
///     println!("Total cost: ${:.2}", metrics.total_cost);
///     println!("Total messages: {}", metrics.messages_count);
///     Ok(())
/// }
/// ```
pub async fn load_claude_project_metrics(project_path: &str) -> Result<ClaudeMetrics> {
    let project_dir = Path::new(project_path);

    if !project_dir.exists() {
        tracing::warn!("Project directory does not exist: {}", project_path);
        return Ok(ClaudeMetrics::default());
    }

    let mut metrics = ClaudeMetrics::default();
    let mut conversation_count = 0;

    // Read all JSONL files in the directory
    let mut entries = fs::read_dir(project_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Only process .jsonl files
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            conversation_count += 1;

            // Use retry logic with exponential backoff (max 3 retries)
            match parse_jsonl_file_with_retry(&path, 3).await {
                Ok(messages) => {
                    for (model, usage, _timestamp) in messages {
                        let normalized_model = normalize_model_name(&model);
                        let (input_price, output_price, cache_write_price, cache_read_price) =
                            get_model_pricing(&normalized_model);

                        let input_tokens = usage.input_tokens.unwrap_or(0);
                        let output_tokens = usage.output_tokens.unwrap_or(0);
                        let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0);
                        let cache_read = usage.cache_read_input_tokens.unwrap_or(0);

                        // Calculate costs using exact pricing from Python monitor
                        let input_cost = calculate_cost(input_tokens, input_price);
                        let output_cost = calculate_cost(output_tokens, output_price);
                        let cache_write_cost = calculate_cost(cache_creation, cache_write_price);
                        let cache_read_cost = calculate_cost(cache_read, cache_read_price);

                        let total_message_cost = input_cost + output_cost + cache_write_cost + cache_read_cost;

                        // Update global totals
                        metrics.total_input_tokens += input_tokens;
                        metrics.total_output_tokens += output_tokens;
                        metrics.total_cache_creation_tokens += cache_creation;
                        metrics.total_cache_read_tokens += cache_read;
                        metrics.total_cost += total_message_cost;
                        metrics.messages_count += 1;

                        // Update per-model breakdown
                        let breakdown = metrics.model_breakdown
                            .entry(normalized_model.clone())
                            .or_insert_with(ModelBreakdown::default);

                        breakdown.input_tokens += input_tokens;
                        breakdown.output_tokens += output_tokens;
                        breakdown.cache_creation_tokens += cache_creation;
                        breakdown.cache_read_tokens += cache_read;
                        breakdown.input_cost += input_cost;
                        breakdown.output_cost += output_cost;
                        breakdown.cache_write_cost += cache_write_cost;
                        breakdown.cache_read_cost += cache_read_cost;
                        breakdown.total_cost += total_message_cost;
                        breakdown.message_count += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse file {:?}: {}", path, e);
                    // Continue processing other files
                }
            }
        }
    }

    metrics.conversations_count = conversation_count;
    metrics.last_updated = Utc::now();

    Ok(metrics)
}

/// Load Claude metrics from home directory with parallel JSONL parsing
///
/// This is an optimized version that processes files in parallel for maximum throughput.
/// Target: 2,339 files in under 60 seconds (>40 files/second).
///
/// # Performance Optimizations
/// - **Parallel file processing**: 50 concurrent workers (configurable)
/// - **DashMap aggregation**: Lock-free concurrent HashMap updates
/// - **Batch file discovery**: Collect all files upfront, process in parallel
/// - **Memory efficient**: Pre-allocated capacity hints, streaming line-by-line
///
/// # Returns
/// ClaudeMetrics struct with aggregated usage, cost data, and breakdowns by project and model
///
/// # Example
/// ```no_run
/// use cco::claude_history::load_claude_metrics_from_home_dir_parallel;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let metrics = load_claude_metrics_from_home_dir_parallel().await?;
///     println!("Total cost: ${:.2}", metrics.total_cost);
///     println!("Total API calls: {}", metrics.messages_count);
///     Ok(())
/// }
/// ```
pub async fn load_claude_metrics_from_home_dir_parallel() -> Result<ClaudeMetrics> {
    const CONCURRENCY_LIMIT: usize = 50; // Optimal balance between CPU/IO

    let start_time = std::time::Instant::now();

    // Get home directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let projects_dir = format!("{}/.claude/projects", home);
    let projects_path = Path::new(&projects_dir);

    if !projects_path.exists() {
        tracing::debug!("Claude projects directory does not exist: {}", projects_dir);
        return Ok(ClaudeMetrics::default());
    }

    // Step 1: Collect all JSONL files from all projects (fast sequential scan)
    info!("Discovering JSONL files in {}...", projects_dir);
    let discovery_start = std::time::Instant::now();

    let mut all_files: Vec<(PathBuf, String)> = Vec::with_capacity(2500); // Expect ~2,354 files

    let mut project_entries = fs::read_dir(projects_path).await?;
    while let Some(project_entry) = project_entries.next_entry().await? {
        let project_path = project_entry.path();

        // Only process directories
        if !project_path.is_dir() {
            continue;
        }

        let project_name = project_entry.file_name().to_string_lossy().to_string();

        // Read all JSONL files in this project
        let mut file_entries = fs::read_dir(&project_path).await?;
        while let Some(file_entry) = file_entries.next_entry().await? {
            let file_path = file_entry.path();

            // Only process .jsonl files
            if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                all_files.push((file_path, project_name.clone()));
            }
        }
    }

    let discovery_elapsed = discovery_start.elapsed();
    info!(
        "Discovered {} JSONL files in {:.2}s",
        all_files.len(),
        discovery_elapsed.as_secs_f64()
    );

    // Step 2: Process files in parallel with semaphore-controlled concurrency
    info!(
        "Parsing {} files with {} concurrent workers...",
        all_files.len(),
        CONCURRENCY_LIMIT
    );
    let parse_start = std::time::Instant::now();

    // Concurrent aggregation structures using DashMap (lock-free)
    let model_breakdown: Arc<DashMap<String, ModelBreakdown>> = Arc::new(DashMap::with_capacity(10));
    let project_breakdown: Arc<DashMap<String, ProjectBreakdown>> = Arc::new(DashMap::with_capacity(30));

    // Global counters (we'll use atomic operations via DashMap pattern)
    let global_stats = Arc::new(DashMap::new());
    global_stats.insert("total_input_tokens", 0u64);
    global_stats.insert("total_output_tokens", 0u64);
    global_stats.insert("total_cache_creation_tokens", 0u64);
    global_stats.insert("total_cache_read_tokens", 0u64);
    global_stats.insert("total_cost", 0u64); // Store as integer (cost * 1_000_000_000 for precision)
    global_stats.insert("messages_count", 0u64);
    global_stats.insert("conversations_count", 0u64);
    global_stats.insert("files_processed", 0u64);
    global_stats.insert("files_failed", 0u64);

    // Semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(CONCURRENCY_LIMIT));

    // Spawn all file parsing tasks
    let mut handles = Vec::with_capacity(all_files.len());

    for (file_path, project_name) in all_files {
        let semaphore = Arc::clone(&semaphore);
        let model_breakdown = Arc::clone(&model_breakdown);
        let project_breakdown = Arc::clone(&project_breakdown);
        let global_stats = Arc::clone(&global_stats);

        let handle = tokio::spawn(async move {
            // Acquire semaphore permit (blocks if at concurrency limit)
            let _permit = semaphore.acquire().await.expect("Semaphore closed");

            // Parse the file
            match parse_jsonl_file(&file_path).await {
                Ok(messages) => {
                    let message_count = messages.len();

                    // Process each message and update concurrent structures
                    for (model, usage, _timestamp) in messages {
                        let normalized_model = normalize_model_name(&model);
                        let (input_price, output_price, cache_write_price, cache_read_price) =
                            get_model_pricing(&normalized_model);

                        let input_tokens = usage.input_tokens.unwrap_or(0);
                        let output_tokens = usage.output_tokens.unwrap_or(0);
                        let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0);
                        let cache_read = usage.cache_read_input_tokens.unwrap_or(0);

                        // Calculate costs
                        let input_cost = calculate_cost(input_tokens, input_price);
                        let output_cost = calculate_cost(output_tokens, output_price);
                        let cache_write_cost = calculate_cost(cache_creation, cache_write_price);
                        let cache_read_cost = calculate_cost(cache_read, cache_read_price);
                        let total_message_cost = input_cost + output_cost + cache_write_cost + cache_read_cost;

                        // Update global stats atomically
                        global_stats
                            .entry("total_input_tokens")
                            .and_modify(|v| *v += input_tokens);
                        global_stats
                            .entry("total_output_tokens")
                            .and_modify(|v| *v += output_tokens);
                        global_stats
                            .entry("total_cache_creation_tokens")
                            .and_modify(|v| *v += cache_creation);
                        global_stats
                            .entry("total_cache_read_tokens")
                            .and_modify(|v| *v += cache_read);
                        global_stats
                            .entry("total_cost")
                            .and_modify(|v| *v += (total_message_cost * 1_000_000_000.0) as u64);
                        global_stats
                            .entry("messages_count")
                            .and_modify(|v| *v += 1);

                        // Update per-model breakdown
                        model_breakdown
                            .entry(normalized_model.clone())
                            .and_modify(|breakdown| {
                                breakdown.input_tokens += input_tokens;
                                breakdown.output_tokens += output_tokens;
                                breakdown.cache_creation_tokens += cache_creation;
                                breakdown.cache_read_tokens += cache_read;
                                breakdown.input_cost += input_cost;
                                breakdown.output_cost += output_cost;
                                breakdown.cache_write_cost += cache_write_cost;
                                breakdown.cache_read_cost += cache_read_cost;
                                breakdown.total_cost += total_message_cost;
                                breakdown.message_count += 1;
                            })
                            .or_insert_with(|| ModelBreakdown {
                                input_tokens,
                                output_tokens,
                                cache_creation_tokens: cache_creation,
                                cache_read_tokens: cache_read,
                                input_cost,
                                output_cost,
                                cache_write_cost,
                                cache_read_cost,
                                total_cost: total_message_cost,
                                message_count: 1,
                            });

                        // Update per-project breakdown
                        project_breakdown
                            .entry(project_name.clone())
                            .and_modify(|breakdown| {
                                breakdown.total_input_tokens += input_tokens;
                                breakdown.total_output_tokens += output_tokens;
                                breakdown.total_cache_creation_tokens += cache_creation;
                                breakdown.total_cache_read_tokens += cache_read;
                                breakdown.total_cost += total_message_cost;
                                breakdown.message_count += 1;

                                // Update project's model breakdown
                                breakdown
                                    .models
                                    .entry(normalized_model.clone())
                                    .and_modify(|model_bd| {
                                        model_bd.input_tokens += input_tokens;
                                        model_bd.output_tokens += output_tokens;
                                        model_bd.cache_creation_tokens += cache_creation;
                                        model_bd.cache_read_tokens += cache_read;
                                        model_bd.input_cost += input_cost;
                                        model_bd.output_cost += output_cost;
                                        model_bd.cache_write_cost += cache_write_cost;
                                        model_bd.cache_read_cost += cache_read_cost;
                                        model_bd.total_cost += total_message_cost;
                                        model_bd.message_count += 1;
                                    })
                                    .or_insert_with(|| ModelBreakdown {
                                        input_tokens,
                                        output_tokens,
                                        cache_creation_tokens: cache_creation,
                                        cache_read_tokens: cache_read,
                                        input_cost,
                                        output_cost,
                                        cache_write_cost,
                                        cache_read_cost,
                                        total_cost: total_message_cost,
                                        message_count: 1,
                                    });
                            })
                            .or_insert_with(|| {
                                let mut models = HashMap::new();
                                models.insert(
                                    normalized_model.clone(),
                                    ModelBreakdown {
                                        input_tokens,
                                        output_tokens,
                                        cache_creation_tokens: cache_creation,
                                        cache_read_tokens: cache_read,
                                        input_cost,
                                        output_cost,
                                        cache_write_cost,
                                        cache_read_cost,
                                        total_cost: total_message_cost,
                                        message_count: 1,
                                    },
                                );

                                ProjectBreakdown {
                                    name: project_name.clone(),
                                    total_input_tokens: input_tokens,
                                    total_output_tokens: output_tokens,
                                    total_cache_creation_tokens: cache_creation,
                                    total_cache_read_tokens: cache_read,
                                    total_cost: total_message_cost,
                                    message_count: 1,
                                    conversation_count: 0, // Will be set after
                                    models,
                                }
                            });
                    }

                    // Increment conversations count (even for files with no messages, to match sequential behavior)
                    project_breakdown
                        .entry(project_name.clone())
                        .and_modify(|breakdown| {
                            breakdown.conversation_count += 1;
                        });
                    global_stats
                        .entry("conversations_count")
                        .and_modify(|v| *v += 1);

                    global_stats
                        .entry("files_processed")
                        .and_modify(|v| *v += 1);

                    Ok::<(), anyhow::Error>(())
                }
                Err(e) => {
                    debug!("Failed to parse file {:?}: {}", file_path, e);
                    global_stats
                        .entry("files_failed")
                        .and_modify(|v| *v += 1);
                    Err(e)
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await; // Ignore individual file errors (logged above)
    }

    let parse_elapsed = parse_start.elapsed();
    info!(
        "Parsed {} files in {:.2}s ({:.1} files/sec)",
        global_stats.get("files_processed").map(|v| *v).unwrap_or(0),
        parse_elapsed.as_secs_f64(),
        global_stats.get("files_processed").map(|v| *v).unwrap_or(0) as f64 / parse_elapsed.as_secs_f64()
    );

    // Step 3: Convert DashMap to regular HashMap for ClaudeMetrics
    let model_breakdown_map: HashMap<String, ModelBreakdown> = model_breakdown
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    let project_breakdown_map: HashMap<String, ProjectBreakdown> = project_breakdown
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    // Build final metrics struct
    let metrics = ClaudeMetrics {
        total_input_tokens: global_stats.get("total_input_tokens").map(|v| *v).unwrap_or(0),
        total_output_tokens: global_stats.get("total_output_tokens").map(|v| *v).unwrap_or(0),
        total_cache_creation_tokens: global_stats
            .get("total_cache_creation_tokens")
            .map(|v| *v)
            .unwrap_or(0),
        total_cache_read_tokens: global_stats
            .get("total_cache_read_tokens")
            .map(|v| *v)
            .unwrap_or(0),
        total_cost: global_stats.get("total_cost").map(|v| *v).unwrap_or(0) as f64 / 1_000_000_000.0,
        messages_count: global_stats.get("messages_count").map(|v| *v).unwrap_or(0),
        conversations_count: global_stats.get("conversations_count").map(|v| *v).unwrap_or(0),
        model_breakdown: model_breakdown_map,
        project_breakdown: project_breakdown_map,
        last_updated: Utc::now(),
    };

    let total_elapsed = start_time.elapsed();
    info!(
        "Total metrics load time: {:.2}s (discovery: {:.2}s, parse: {:.2}s)",
        total_elapsed.as_secs_f64(),
        discovery_elapsed.as_secs_f64(),
        parse_elapsed.as_secs_f64()
    );
    info!(
        "Performance: {:.1} files/sec, {} messages, ${:.2} total cost",
        global_stats.get("files_processed").map(|v| *v).unwrap_or(0) as f64 / total_elapsed.as_secs_f64(),
        metrics.messages_count,
        metrics.total_cost
    );

    Ok(metrics)
}

/// Load Claude project metrics grouped by date for time-series analysis
///
/// Returns a HashMap where key is date (YYYY-MM-DD) and value is metrics for that day
///
/// # Arguments
/// * `project_path` - Path to the Claude project directory
///
/// # Returns
/// HashMap<date, Vec<(model, usage_data, timestamp)>> for each day
///
/// # Example
/// ```no_run
/// use cco::claude_history::load_claude_project_metrics_by_date;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let daily_metrics = load_claude_project_metrics_by_date(
///         "/Users/brent/.claude/projects/cc-orchestra"
///     ).await?;
///
///     for (date, metrics) in daily_metrics {
///         println!("Date: {}, Messages: {}", date, metrics.len());
///     }
///     Ok(())
/// }
/// ```
pub async fn load_claude_project_metrics_by_date(
    project_path: &str,
) -> Result<std::collections::HashMap<String, Vec<(String, UsageData, String)>>> {
    use std::collections::HashMap;

    let project_dir = Path::new(project_path);

    if !project_dir.exists() {
        tracing::warn!("Project directory does not exist: {}", project_path);
        return Ok(HashMap::new());
    }

    let mut metrics_by_date: HashMap<String, Vec<(String, UsageData, String)>> = HashMap::new();

    // Read all JSONL files in the directory
    let mut entries = fs::read_dir(project_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Only process .jsonl files
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // Use retry logic with exponential backoff (max 3 retries)
            match parse_jsonl_file_with_retry(&path, 3).await {
                Ok(messages) => {
                    for (model, usage, timestamp_opt) in messages {
                        // Extract date from timestamp
                        if let Some(timestamp_str) = timestamp_opt {
                            // Parse ISO 8601 timestamp and extract date
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&timestamp_str) {
                                let date = dt.format("%Y-%m-%d").to_string();
                                metrics_by_date
                                    .entry(date)
                                    .or_insert_with(Vec::new)
                                    .push((model, usage, timestamp_str));
                            } else {
                                tracing::debug!(
                                    "Failed to parse timestamp: {}. Skipping message.",
                                    timestamp_str
                                );
                            }
                        } else {
                            tracing::debug!("Message missing timestamp. Skipping.");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse file {:?}: {}", path, e);
                    // Continue processing other files
                }
            }
        }
    }

    Ok(metrics_by_date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(
            normalize_model_name("claude-sonnet-4-5-20250929"),
            "claude-sonnet-4-5"
        );
        assert_eq!(
            normalize_model_name("claude-opus-4-20250514"),
            "claude-opus-4"
        );
        assert_eq!(
            normalize_model_name("claude-3-5-haiku-20250403"),
            "claude-3-5-haiku"
        );
        assert_eq!(
            normalize_model_name("claude-sonnet-3.5"),
            "claude-sonnet-3.5"
        );
    }

    #[test]
    fn test_model_pricing() {
        // Test Opus 4 pricing
        let (input, output, cache_write, cache_read) = get_model_pricing("claude-opus-4");
        assert_eq!(input, 15.0);
        assert_eq!(output, 75.0);
        assert_eq!(cache_write, 18.75); // 25% premium
        assert_eq!(cache_read, 1.50);   // 10% of input

        // Test Sonnet 4.5 pricing
        let (input, output, cache_write, cache_read) = get_model_pricing("claude-sonnet-4-5");
        assert_eq!(input, 3.0);
        assert_eq!(output, 15.0);
        assert_eq!(cache_write, 3.75);  // 25% premium
        assert_eq!(cache_read, 0.30);   // 10% of input

        // Test Haiku 4.5 pricing
        let (input, output, cache_write, cache_read) = get_model_pricing("claude-haiku-4-5");
        assert_eq!(input, 1.0);
        assert_eq!(output, 5.0);
        assert_eq!(cache_write, 1.25);  // 25% premium
        assert_eq!(cache_read, 0.10);   // 10% of input

        // Test variant names (3-5-sonnet format)
        let (input, output, cache_write, cache_read) = get_model_pricing("claude-3-5-sonnet");
        assert_eq!(input, 3.0);
        assert_eq!(output, 15.0);
        assert_eq!(cache_write, 3.75);
        assert_eq!(cache_read, 0.30);

        // Test opus-4-1 variant
        let (input, output, cache_write, cache_read) = get_model_pricing("claude-opus-4-1");
        assert_eq!(input, 15.0);
        assert_eq!(output, 75.0);
        assert_eq!(cache_write, 18.75);
        assert_eq!(cache_read, 1.50);
    }

    #[test]
    fn test_calculate_cost() {
        // 1 million tokens at $15/M = $15
        let cost = calculate_cost(1_000_000, 15.0);
        assert!((cost - 15.0).abs() < 0.001);

        // 500k tokens at $3/M = $1.50
        let cost = calculate_cost(500_000, 3.0);
        assert!((cost - 1.5).abs() < 0.001);

        // 10k tokens at $15/M = $0.15
        let cost = calculate_cost(10_000, 15.0);
        assert!((cost - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_parse_valid_jsonl_line() {
        let line = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":10,"output_tokens":662}}}"#;
        let msg = parse_jsonl_line(line);
        assert!(msg.is_some());

        let msg = msg.unwrap();
        assert_eq!(msg.message_type, "assistant");
        assert!(msg.message.is_some());
    }

    #[test]
    fn test_parse_invalid_jsonl_gracefully() {
        let line = r#"{"invalid json"#;
        let msg = parse_jsonl_line(line);
        assert!(msg.is_none());
    }

    #[test]
    fn test_parse_non_assistant_message() {
        let line = r#"{"type":"summary","summary":"Test summary"}"#;
        let msg = parse_jsonl_line(line);
        assert!(msg.is_some());
        assert_eq!(msg.unwrap().message_type, "summary");
    }

    #[tokio::test]
    async fn test_empty_project_directory() {
        // Create a temporary empty directory
        let temp_dir = std::env::temp_dir().join("cco_test_empty");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let metrics = load_claude_project_metrics(temp_dir.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(metrics.total_cost, 0.0);
        assert_eq!(metrics.messages_count, 0);
        assert_eq!(metrics.conversations_count, 0);

        // Cleanup
        fs::remove_dir_all(&temp_dir).await.ok();
    }

    #[tokio::test]
    async fn test_nonexistent_project_directory() {
        let metrics = load_claude_project_metrics("/nonexistent/path/to/project")
            .await
            .unwrap();

        assert_eq!(metrics.total_cost, 0.0);
        assert_eq!(metrics.messages_count, 0);
    }

    #[tokio::test]
    async fn test_parse_sample_jsonl() {
        // Create a temporary test file
        let temp_dir = std::env::temp_dir().join("cco_test_jsonl");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let test_file = temp_dir.join("test.jsonl");
        let mut file = fs::File::create(&test_file).await.unwrap();

        // Write test data
        let line1 = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1000,"output_tokens":500}}}"#;
        let line2 = r#"{"type":"summary","summary":"Test summary"}"#;
        let line3 = r#"{"type":"assistant","message":{"model":"claude-opus-4-20250514","usage":{"input_tokens":2000,"output_tokens":1000,"cache_creation_input_tokens":5000}}}"#;

        file.write_all(format!("{}\n{}\n{}\n", line1, line2, line3).as_bytes())
            .await
            .unwrap();
        file.flush().await.unwrap();
        drop(file);

        let metrics = load_claude_project_metrics(temp_dir.to_str().unwrap())
            .await
            .unwrap();

        // Should have 2 assistant messages
        assert_eq!(metrics.messages_count, 2);
        assert_eq!(metrics.conversations_count, 1);

        // Verify token counts
        assert_eq!(metrics.total_input_tokens, 3000); // 1000 + 2000
        assert_eq!(metrics.total_output_tokens, 1500); // 500 + 1000
        assert_eq!(metrics.total_cache_creation_tokens, 5000);

        // Verify model breakdown
        assert!(metrics.model_breakdown.contains_key("claude-sonnet-4-5"));
        assert!(metrics.model_breakdown.contains_key("claude-opus-4"));

        let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
        assert_eq!(sonnet.input_tokens, 1000);
        assert_eq!(sonnet.output_tokens, 500);
        assert_eq!(sonnet.message_count, 1);

        let opus = metrics.model_breakdown.get("claude-opus-4").unwrap();
        assert_eq!(opus.input_tokens, 2000);
        assert_eq!(opus.output_tokens, 1000);
        assert_eq!(opus.cache_creation_tokens, 5000);
        assert_eq!(opus.message_count, 1);

        // Verify costs with cache pricing
        // Sonnet: input=(1000 * 3)/1M + output=(500 * 15)/1M = $0.003 + $0.0075 = $0.0105
        let expected_sonnet = (1000.0 * 3.0 + 500.0 * 15.0) / 1_000_000.0;
        assert!((sonnet.total_cost - expected_sonnet).abs() < 0.00001);
        assert!((sonnet.input_cost - 0.003).abs() < 0.00001);
        assert!((sonnet.output_cost - 0.0075).abs() < 0.00001);

        // Opus: input=(2000 * 15)/1M + output=(1000 * 75)/1M + cache_write=(5000 * 18.75)/1M
        //     = $0.03 + $0.075 + $0.09375 = $0.19875
        let expected_opus_input = (2000.0 * 15.0) / 1_000_000.0;
        let expected_opus_output = (1000.0 * 75.0) / 1_000_000.0;
        let expected_opus_cache_write = (5000.0 * 18.75) / 1_000_000.0;
        let expected_opus_total = expected_opus_input + expected_opus_output + expected_opus_cache_write;

        assert!((opus.input_cost - expected_opus_input).abs() < 0.00001);
        assert!((opus.output_cost - expected_opus_output).abs() < 0.00001);
        assert!((opus.cache_write_cost - expected_opus_cache_write).abs() < 0.00001);
        assert!((opus.total_cost - expected_opus_total).abs() < 0.00001);

        // Verify global totals
        assert!(metrics.total_cost > 0.0);

        // Cleanup
        fs::remove_dir_all(&temp_dir).await.ok();
    }

    #[tokio::test]
    async fn test_invalid_json_gracefully_skipped() {
        let temp_dir = std::env::temp_dir().join("cco_test_invalid");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let test_file = temp_dir.join("invalid.jsonl");
        let mut file = fs::File::create(&test_file).await.unwrap();

        // Mix of valid and invalid JSON
        let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}}}
{"invalid json line
{"type":"assistant","message":{"model":"claude-haiku-4-5-20250403","usage":{"input_tokens":200,"output_tokens":100}}}
malformed data here
"#;

        file.write_all(content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        let metrics = load_claude_project_metrics(temp_dir.to_str().unwrap())
            .await
            .unwrap();

        // Should have parsed 2 valid messages, skipped 2 invalid
        assert_eq!(metrics.messages_count, 2);
        assert_eq!(metrics.total_input_tokens, 300); // 100 + 200
        assert_eq!(metrics.total_output_tokens, 150); // 50 + 100

        // Cleanup
        fs::remove_dir_all(&temp_dir).await.ok();
    }

    #[tokio::test]
    async fn test_model_aggregation() {
        let temp_dir = std::env::temp_dir().join("cco_test_aggregation");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let test_file = temp_dir.join("aggregation.jsonl");
        let mut file = fs::File::create(&test_file).await.unwrap();

        // Multiple messages from same model
        let content = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":100,"output_tokens":50}}}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":200,"output_tokens":100}}}
{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":300,"output_tokens":150}}}
"#;

        file.write_all(content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        let metrics = load_claude_project_metrics(temp_dir.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(metrics.messages_count, 3);
        assert_eq!(metrics.model_breakdown.len(), 1);

        let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();
        assert_eq!(sonnet.input_tokens, 600); // 100 + 200 + 300
        assert_eq!(sonnet.output_tokens, 300); // 50 + 100 + 150
        assert_eq!(sonnet.message_count, 3);

        // Verify cost calculation
        // Input: 600 * $3/M = $0.0018
        // Output: 300 * $15/M = $0.0045
        // Total: $0.0063
        let expected_cost = (600.0 * 3.0 + 300.0 * 15.0) / 1_000_000.0;
        assert!((sonnet.total_cost - expected_cost).abs() < 0.00001);

        // Cleanup
        fs::remove_dir_all(&temp_dir).await.ok();
    }

    #[test]
    fn test_default_metrics() {
        let metrics = ClaudeMetrics::default();
        assert_eq!(metrics.total_cost, 0.0);
        assert_eq!(metrics.messages_count, 0);
        assert_eq!(metrics.conversations_count, 0);
        assert!(metrics.model_breakdown.is_empty());
    }

    #[test]
    fn test_default_model_breakdown() {
        let breakdown = ModelBreakdown::default();
        assert_eq!(breakdown.input_tokens, 0);
        assert_eq!(breakdown.output_tokens, 0);
        assert_eq!(breakdown.cache_creation_tokens, 0);
        assert_eq!(breakdown.cache_read_tokens, 0);
        assert_eq!(breakdown.input_cost, 0.0);
        assert_eq!(breakdown.output_cost, 0.0);
        assert_eq!(breakdown.cache_write_cost, 0.0);
        assert_eq!(breakdown.cache_read_cost, 0.0);
        assert_eq!(breakdown.total_cost, 0.0);
        assert_eq!(breakdown.message_count, 0);
    }

    #[test]
    fn test_cache_token_pricing() {
        // Verify cache pricing matches Python monitor formula:
        // - cache_write = input_price * 1.25 (25% premium)
        // - cache_read = input_price * 0.10 (90% discount)

        // Test Sonnet
        let (input, _output, cache_write, cache_read) = get_model_pricing("claude-sonnet-4-5");
        assert!((cache_write - input * 1.25).abs() < 0.0001); // 3.0 * 1.25 = 3.75
        assert!((cache_read - input * 0.10).abs() < 0.0001);  // 3.0 * 0.10 = 0.30

        // Test Haiku
        let (input, _output, cache_write, cache_read) = get_model_pricing("claude-haiku-4-5");
        assert!((cache_write - input * 1.25).abs() < 0.0001); // 1.0 * 1.25 = 1.25
        assert!((cache_read - input * 0.10).abs() < 0.0001);  // 1.0 * 0.10 = 0.10

        // Test Opus
        let (input, _output, cache_write, cache_read) = get_model_pricing("claude-opus-4");
        assert!((cache_write - input * 1.25).abs() < 0.0001); // 15.0 * 1.25 = 18.75
        assert!((cache_read - input * 0.10).abs() < 0.0001);  // 15.0 * 0.10 = 1.50
    }

    #[test]
    fn test_cache_cost_calculations() {
        // Test actual cost calculations with cache tokens
        // Scenario: 10K input, 5K output, 20K cache_write, 50K cache_read (Sonnet)

        let input_tokens = 10_000_u64;
        let output_tokens = 5_000_u64;
        let cache_creation = 20_000_u64;
        let cache_read = 50_000_u64;

        let (input_price, output_price, cache_write_price, cache_read_price) =
            get_model_pricing("claude-sonnet-4-5");

        let input_cost = calculate_cost(input_tokens, input_price);
        let output_cost = calculate_cost(output_tokens, output_price);
        let cache_write_cost = calculate_cost(cache_creation, cache_write_price);
        let cache_read_cost = calculate_cost(cache_read, cache_read_price);

        // Expected costs:
        // input: 10K * $3/M = $0.03
        // output: 5K * $15/M = $0.075
        // cache_write: 20K * $3.75/M = $0.075
        // cache_read: 50K * $0.30/M = $0.015
        // total: $0.195 (not 0.19 - recalculated)

        let expected_input = (10_000.0 * 3.0) / 1_000_000.0;
        let expected_output = (5_000.0 * 15.0) / 1_000_000.0;
        let expected_cache_write = (20_000.0 * 3.75) / 1_000_000.0;
        let expected_cache_read = (50_000.0 * 0.30) / 1_000_000.0;

        assert!((input_cost - expected_input).abs() < 0.00001);
        assert!((output_cost - expected_output).abs() < 0.00001);
        assert!((cache_write_cost - expected_cache_write).abs() < 0.00001);
        assert!((cache_read_cost - expected_cache_read).abs() < 0.00001);

        let total = input_cost + output_cost + cache_write_cost + cache_read_cost;
        let expected_total = expected_input + expected_output + expected_cache_write + expected_cache_read;
        assert!((total - expected_total).abs() < 0.00001);
    }

    #[tokio::test]
    async fn test_cache_tokens_in_jsonl() {
        // Test parsing JSONL with cache tokens (matching Python monitor behavior)
        let temp_dir = std::env::temp_dir().join("cco_test_cache");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let test_file = temp_dir.join("cache_test.jsonl");
        let mut file = fs::File::create(&test_file).await.unwrap();

        // Message with cache_read_input_tokens (cached request - 90% savings)
        let line = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-5-20250929","usage":{"input_tokens":1000,"output_tokens":500,"cache_read_input_tokens":10000}}}"#;
        file.write_all(format!("{}\n", line).as_bytes())
            .await
            .unwrap();
        file.flush().await.unwrap();
        drop(file);

        let metrics = load_claude_project_metrics(temp_dir.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(metrics.total_input_tokens, 1000);
        assert_eq!(metrics.total_cache_read_tokens, 10000);

        let sonnet = metrics.model_breakdown.get("claude-sonnet-4-5").unwrap();

        // Verify cache read cost is 90% cheaper
        // cache_read: 10000 * $0.30/M = $0.003
        // regular input of same tokens would cost: 10000 * $3/M = $0.03
        // savings: $0.027 (90%)
        assert!((sonnet.cache_read_cost - 0.003).abs() < 0.00001);

        // Cleanup
        fs::remove_dir_all(&temp_dir).await.ok();
    }
}
