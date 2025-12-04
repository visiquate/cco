//! Orchestra configuration parser
//!
//! Parses orchestra-config.json and converts it to Claude Code's agent format.
//! This module provides the single source of truth for all 119 agents.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Orchestra configuration with all agent categories
#[derive(Debug, Deserialize)]
pub struct OrchestraConfig {
    pub architect: Agent,
    #[serde(rename = "codingAgents")]
    pub coding_agents: Vec<Agent>,
    #[serde(rename = "integrationAgents")]
    pub integration_agents: Vec<Agent>,
    #[serde(rename = "developmentAgents")]
    pub development_agents: Vec<Agent>,
    #[serde(rename = "dataAgents")]
    pub data_agents: Vec<Agent>,
    #[serde(rename = "infrastructureAgents")]
    pub infrastructure_agents: Vec<Agent>,
    #[serde(rename = "securityAgents")]
    pub security_agents: Vec<Agent>,
    #[serde(rename = "aiMlAgents")]
    pub ai_ml_agents: Vec<Agent>,
    #[serde(rename = "mcpAgents")]
    pub mcp_agents: Vec<Agent>,
    #[serde(rename = "documentationAgents")]
    pub documentation_agents: Vec<Agent>,
    #[serde(rename = "researchAgents")]
    pub research_agents: Vec<Agent>,
    #[serde(rename = "supportAgents")]
    pub support_agents: Vec<Agent>,
    #[serde(rename = "businessAgents")]
    pub business_agents: Vec<Agent>,
}

/// Agent definition from orchestra-config.json
#[derive(Debug, Deserialize, Clone)]
pub struct Agent {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub model: String,
    /// Role description (optional - falls back to name if missing)
    #[serde(default)]
    pub role: Option<String>,
    /// Full agent prompt (required - embedded in orchestra-config.json)
    pub prompt: String,
    /// Optional reference to markdown file (not used at runtime)
    #[serde(rename = "agentFile")]
    pub agent_file: Option<String>,
}

/// Claude Code agent format
#[derive(Debug, Serialize)]
pub struct ClaudeAgent {
    pub description: String,
    pub prompt: String,
    pub model: String,
}

impl OrchestraConfig {
    /// Load orchestra configuration from embedded config (compile-time)
    pub fn load_embedded() -> Result<Self> {
        let content = crate::embedded_config::embedded_orchestra_config_str();
        let config: OrchestraConfig = serde_json::from_str(content)
            .context("Failed to parse embedded orchestra-config.json")?;
        Ok(config)
    }

    /// Load orchestra configuration from JSON file
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read orchestra config: {}", path))?;

        let config: OrchestraConfig =
            serde_json::from_str(&content).context("Failed to parse orchestra-config.json")?;

        Ok(config)
    }

    /// Get all agents flattened into a single vector
    pub fn all_agents(&self) -> Vec<&Agent> {
        let mut agents = vec![&self.architect];
        agents.extend(&self.coding_agents);
        agents.extend(&self.integration_agents);
        agents.extend(&self.development_agents);
        agents.extend(&self.data_agents);
        agents.extend(&self.infrastructure_agents);
        agents.extend(&self.security_agents);
        agents.extend(&self.ai_ml_agents);
        agents.extend(&self.mcp_agents);
        agents.extend(&self.documentation_agents);
        agents.extend(&self.research_agents);
        agents.extend(&self.support_agents);
        agents.extend(&self.business_agents);
        agents
    }

    /// Convert to Claude Code agent format
    ///
    /// Creates a HashMap mapping agent types to Claude agent definitions.
    /// Uses embedded prompts directly from JSON (no fallback - prompts are required).
    pub fn to_claude_format(&self) -> Result<HashMap<String, ClaudeAgent>> {
        let mut claude_agents = HashMap::new();

        for agent in self.all_agents() {
            // Get role description (fallback to name if missing)
            let role = agent
                .role
                .as_ref()
                .map(|r| r.clone())
                .unwrap_or_else(|| agent.name.clone());

            claude_agents.insert(
                agent.agent_type.clone(),
                ClaudeAgent {
                    description: role,
                    prompt: agent.prompt.clone(),
                    model: agent.model.clone(),
                },
            );
        }

        Ok(claude_agents)
    }

    /// Count total number of agents
    pub fn agent_count(&self) -> usize {
        self.all_agents().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_orchestra_config() {
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";

        // Skip test if config file doesn't exist (CI environment)
        if !std::path::Path::new(config_path).exists() {
            return;
        }

        let config = OrchestraConfig::load(config_path).unwrap();

        // Verify we have the chief architect
        assert_eq!(config.architect.agent_type, "chief-architect");

        // Verify total agent count (1 architect + 116 specialized = 117 total)
        assert_eq!(config.agent_count(), 117, "Expected 117 total agents");
    }

    #[test]
    fn test_convert_to_claude_format() {
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";

        // Skip test if config file doesn't exist (CI environment)
        if !std::path::Path::new(config_path).exists() {
            return;
        }

        let config = OrchestraConfig::load(config_path).unwrap();
        let claude_agents = config.to_claude_format().unwrap();

        // Verify we have 117 agents in Claude format
        assert_eq!(claude_agents.len(), 117);

        // Verify chief architect exists
        assert!(claude_agents.contains_key("chief-architect"));
        let architect = claude_agents.get("chief-architect").unwrap();
        assert!(!architect.prompt.is_empty());
        assert_eq!(architect.model, "opus");

        // Verify a few other key agents
        assert!(claude_agents.contains_key("python-specialist"));
        assert!(claude_agents.contains_key("tdd-coding-agent"));
    }

    #[test]
    fn test_all_agents_have_required_fields() {
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";

        // Skip test if config file doesn't exist (CI environment)
        if !std::path::Path::new(config_path).exists() {
            return;
        }

        let config = OrchestraConfig::load(config_path).unwrap();

        for agent in config.all_agents() {
            // Verify required fields
            assert!(!agent.name.is_empty(), "Agent missing name");
            assert!(!agent.agent_type.is_empty(), "Agent missing type");
            assert!(!agent.model.is_empty(), "Agent missing model");
            // Note: role is optional and falls back to name if missing
        }
    }

    #[test]
    fn test_agent_types_are_unique() {
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";

        // Skip test if config file doesn't exist (CI environment)
        if !std::path::Path::new(config_path).exists() {
            return;
        }

        let config = OrchestraConfig::load(config_path).unwrap();
        let claude_agents = config.to_claude_format().unwrap();

        // If all agent types are unique, HashMap size should equal agent count
        assert_eq!(claude_agents.len(), config.agent_count());
    }

    #[test]
    fn test_all_agents_have_prompts() {
        let config_path = "/Users/brent/git/cc-orchestra/config/orchestra-config.json";

        // Skip test if config file doesn't exist (CI environment)
        if !std::path::Path::new(config_path).exists() {
            return;
        }

        let config = OrchestraConfig::load(config_path).unwrap();

        for agent in config.all_agents() {
            assert!(
                !agent.prompt.is_empty(),
                "Agent '{}' (type: {}) has empty prompt",
                agent.name,
                agent.agent_type
            );

            // Verify prompt has reasonable content (at least 50 characters)
            assert!(
                agent.prompt.len() >= 50,
                "Agent '{}' (type: {}) has suspiciously short prompt: {} chars",
                agent.name,
                agent.agent_type,
                agent.prompt.len()
            );
        }
    }
}
