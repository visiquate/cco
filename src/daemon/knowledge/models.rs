//! Data models for the Knowledge Store
//!
//! Defines the core data structures used for storing and retrieving knowledge items,
//! matching the schema from the JavaScript knowledge-manager.js implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Knowledge type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeType {
    Decision,
    Architecture,
    Implementation,
    Configuration,
    Credential,
    Issue,
    General,
    System,
}

impl std::fmt::Display for KnowledgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeType::Decision => write!(f, "decision"),
            KnowledgeType::Architecture => write!(f, "architecture"),
            KnowledgeType::Implementation => write!(f, "implementation"),
            KnowledgeType::Configuration => write!(f, "configuration"),
            KnowledgeType::Credential => write!(f, "credential"),
            KnowledgeType::Issue => write!(f, "issue"),
            KnowledgeType::General => write!(f, "general"),
            KnowledgeType::System => write!(f, "system"),
        }
    }
}

impl From<&str> for KnowledgeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "decision" => KnowledgeType::Decision,
            "architecture" => KnowledgeType::Architecture,
            "implementation" => KnowledgeType::Implementation,
            "configuration" => KnowledgeType::Configuration,
            "credential" => KnowledgeType::Credential,
            "issue" => KnowledgeType::Issue,
            "system" => KnowledgeType::System,
            _ => KnowledgeType::General,
        }
    }
}

/// Knowledge item stored in LanceDB
///
/// Schema matches the JavaScript implementation exactly:
/// - id: "type-timestamp-random"
/// - vector: 384 dimensions, [-1, 1] range (SHA256-based embedding)
/// - text: Knowledge content
/// - type: KnowledgeType variant
/// - project_id: Repository identifier
/// - session_id: Agent session
/// - agent: Agent name
/// - timestamp: ISO8601 creation time
/// - metadata: JSON-serialized metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub vector: Vec<f32>,
    pub text: String,
    #[serde(rename = "type")]
    pub knowledge_type: String,
    pub project_id: String,
    pub session_id: String,
    pub agent: String,
    pub timestamp: String, // ISO8601 formatted
    pub metadata: String,  // JSON string
}

/// Request to store knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreKnowledgeRequest {
    pub text: String,
    #[serde(rename = "type", default)]
    pub knowledge_type: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Response from storing knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreKnowledgeResponse {
    pub id: String,
    pub stored: bool,
}

/// Request to search knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(rename = "type", default)]
    pub knowledge_type: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
}

fn default_limit() -> usize {
    10
}

fn default_threshold() -> f64 {
    0.5
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    #[serde(rename = "type")]
    pub knowledge_type: String,
    pub project_id: String,
    pub session_id: String,
    pub agent: String,
    pub timestamp: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub score: f32, // Similarity score (distance)
}

/// Session start request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartRequest {
    pub session_id: String,
    #[serde(default)]
    pub context: Option<serde_json::Value>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Session start response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartResponse {
    pub success: bool,
    pub recent_knowledge: Vec<KnowledgeItem>,
    pub summary: String,
}

/// Pre-compaction context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCompactionRequest {
    pub conversation: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Pre-compaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCompactionResponse {
    pub success: bool,
    pub count: usize,
    pub ids: Vec<String>,
}

/// Post-compaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCompactionRequest {
    pub current_task: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Context summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub total_items: usize,
    pub by_type: HashMap<String, usize>,
    pub by_agent: HashMap<String, usize>,
    pub top_decisions: Vec<String>,
    pub recent_activity: Vec<RecentActivityItem>,
}

/// Recent activity item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivityItem {
    #[serde(rename = "type")]
    pub knowledge_type: String,
    pub agent: String,
    pub timestamp: String,
    pub preview: String,
}

/// Post-compaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCompactionResponse {
    pub search_results: Vec<SearchResult>,
    pub recent_knowledge: Vec<SearchResult>,
    pub summary: ContextSummary,
}

/// Statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub repository: String,
    pub total_records: usize,
    pub by_type: HashMap<String, usize>,
    pub by_agent: HashMap<String, usize>,
    pub by_project: HashMap<String, usize>,
    pub oldest_record: Option<String>,
    pub newest_record: Option<String>,
}

/// Cleanup request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRequest {
    #[serde(default = "default_cleanup_days")]
    pub older_than_days: i64,
    #[serde(default)]
    pub project_id: Option<String>,
}

fn default_cleanup_days() -> i64 {
    90
}

/// Cleanup response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResponse {
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_type_conversion() {
        assert_eq!(KnowledgeType::from("decision"), KnowledgeType::Decision);
        assert_eq!(
            KnowledgeType::from("ARCHITECTURE"),
            KnowledgeType::Architecture
        );
        assert_eq!(KnowledgeType::from("unknown"), KnowledgeType::General);
    }

    #[test]
    fn test_knowledge_type_display() {
        assert_eq!(KnowledgeType::Decision.to_string(), "decision");
        assert_eq!(KnowledgeType::Architecture.to_string(), "architecture");
    }

    #[test]
    fn test_search_request_defaults() {
        let json = r#"{"query": "test"}"#;
        let req: SearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.limit, 10);
        assert_eq!(req.threshold, 0.5);
    }

    #[test]
    fn test_cleanup_request_defaults() {
        let json = r#"{}"#;
        let req: CleanupRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.older_than_days, 90);
    }
}
