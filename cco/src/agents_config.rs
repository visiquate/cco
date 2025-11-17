//! Agent configuration loading and management
//!
//! Loads agent definitions from ~/.claude/agents/ directory and provides
//! HTTP endpoints to query agent configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// Agent configuration with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    /// Agent identifier (e.g., "chief-architect", "python-specialist")
    pub name: String,
    /// Assigned LLM model (e.g., "opus", "sonnet", "haiku")
    pub model: String,
    /// Description of agent's purpose and capabilities
    pub description: String,
    /// List of tools available to this agent
    pub tools: Vec<String>,
    /// Agent type/role (e.g., "tdd-coding-agent", "python-specialist") - optional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Agent role description (e.g., "Test-driven development specialist") - optional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Container for all loaded agent configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    /// HashMap of agent name -> Agent configuration
    pub agents: HashMap<String, Agent>,
}

impl AgentsConfig {
    /// Create an empty agents config
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Get a specific agent by name
    pub fn get(&self, name: &str) -> Option<&Agent> {
        self.agents.get(name)
    }

    /// Get all agents as a vector
    pub fn all(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    /// Get the count of loaded agents
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Check if there are any agents loaded
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple YAML frontmatter parser for agent definitions
///
/// Parses key: value pairs in YAML format.
/// Supports comma-separated strings for list values.
#[derive(Debug, Clone)]
struct FrontmatterData {
    name: Option<String>,
    model: Option<String>,
    description: Option<String>,
    tools: Option<String>, // Will be parsed as comma-separated
    r#type: Option<String>,
    role: Option<String>,
}

/// Parse YAML frontmatter from a markdown file
///
/// Extracts the YAML block between --- markers at the start of the file.
/// Returns the parsed YAML data or None if no frontmatter found.
fn parse_frontmatter(content: &str) -> Option<FrontmatterData> {
    // Check if file starts with ---
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing --- marker
    let rest = &content[3..]; // Skip opening ---
    let closing_pos = rest.find("---")?;

    // Extract the YAML content between the markers
    let yaml_content = &rest[..closing_pos];

    // Simple line-by-line YAML parser
    let mut data = FrontmatterData {
        name: None,
        model: None,
        description: None,
        tools: None,
        r#type: None,
        role: None,
    };

    for line in yaml_content.lines() {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key: value pairs
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();

            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };

            match key {
                "name" => data.name = Some(value.to_string()),
                "model" => data.model = Some(value.to_string()),
                "description" => data.description = Some(value.to_string()),
                "tools" => data.tools = Some(value.to_string()),
                "type" => data.r#type = Some(value.to_string()),
                "role" => data.role = Some(value.to_string()),
                _ => {}
            }
        }
    }

    Some(data)
}

/// Load a single agent definition from a markdown file
///
/// Parses the YAML frontmatter and extracts:
/// - name: Agent identifier
/// - model: Assigned LLM model
/// - description: Agent description
/// - tools: Comma-separated list of tools
fn load_agent_from_file(path: &PathBuf) -> Option<Agent> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read agent file {:?}: {}", path, e);
            return None;
        }
    };

    let frontmatter = parse_frontmatter(&content)?;

    // Extract required fields
    let name = frontmatter.name?;
    let model = frontmatter.model?;
    let description = frontmatter.description?;

    // Parse tools - comma-separated string
    let tools = if let Some(tools_str) = frontmatter.tools {
        tools_str
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    Some(Agent {
        name,
        model,
        description,
        tools,
        r#type: frontmatter.r#type,
        role: frontmatter.role,
    })
}

/// Load agents from a filesystem directory (used for development mode)
///
/// This is called when CCO_AGENTS_DIR environment variable is set.
fn load_agents_from_dir(dir_path: &str) -> AgentsConfig {
    let mut config = AgentsConfig::new();

    // Get agents directory path
    let agents_dir = PathBuf::from(dir_path);

    // Check if directory exists
    if !agents_dir.exists() {
        warn!("Agents directory not found: {:?}", agents_dir);
        return config;
    }

    // Read directory entries
    let entries = match fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to read agents directory: {}", e);
            return config;
        }
    };

    // Load each .md file
    let mut loaded_count = 0;
    let mut error_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process .md files
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        if let Some(agent) = load_agent_from_file(&path) {
            config.agents.insert(agent.name.clone(), agent);
            loaded_count += 1;
        } else {
            error_count += 1;
            warn!("Failed to load agent from: {:?}", path);
        }
    }

    info!(
        "✓ Loaded {} agents from {:?} ({} errors)",
        loaded_count, agents_dir, error_count
    );

    config
}

/// Load all agent definitions from embedded or filesystem sources
///
/// This function supports two modes:
/// 1. **Production mode** (default): Loads agents from embedded binary data
///    compiled at build time. This eliminates filesystem I/O and enables
///    offline operation.
/// 2. **Development mode**: If CCO_AGENTS_DIR environment variable is set,
///    loads agents from the specified filesystem directory instead. This
///    allows iterating on agent definitions without rebuilding.
///
/// The function logs which mode is being used and the number of agents loaded.
///
/// # Panics
/// - If no agents are loaded (either from embedded or filesystem)
/// - If embedded data is malformed (shouldn't happen if build succeeded)
pub fn load_agents_from_embedded() -> AgentsConfig {
    use crate::embedded_agents;

    // Check development mode override
    if let Ok(dev_dir) = std::env::var("CCO_AGENTS_DIR") {
        info!("Development mode: Loading agents from CCO_AGENTS_DIR: {}", dev_dir);
        return load_agents_from_dir(&dev_dir);
    }

    // Production: Load from embedded data generated by build.rs
    info!("Loading embedded agent definitions (compiled at build time)");

    // Initialize agents from embedded data
    let config = AgentsConfig {
        agents: embedded_agents::initialize_embedded_agents(),
    };

    let loaded_count = config.agents.len();

    if loaded_count == 0 {
        panic!("No agents loaded from embedded data - build may have failed!");
    }

    info!("✓ Loaded {} embedded agents from compiled binary", loaded_count);

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test-agent
model: sonnet
description: A test agent
tools: Read, Write, Edit
---

# Content here
"#;

        let result = parse_frontmatter(content);
        assert!(result.is_some());

        let data = result.unwrap();
        assert_eq!(data.name, Some("test-agent".to_string()));
        assert_eq!(data.model, Some("sonnet".to_string()));
        assert_eq!(data.description, Some("A test agent".to_string()));
        assert_eq!(data.tools, Some("Read, Write, Edit".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_no_opening_marker() {
        let content = "name: test\nmodel: sonnet\n---\nContent";
        let result = parse_frontmatter(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_agents_config_operations() {
        let mut config = AgentsConfig::new();

        let agent = Agent {
            name: "test".to_string(),
            model: "sonnet".to_string(),
            description: "Test agent".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string()],
            r#type: None,
            role: None,
        };

        config.agents.insert("test".to_string(), agent.clone());

        assert_eq!(config.len(), 1);
        assert!(!config.is_empty());
        assert_eq!(config.get("test"), Some(&agent));
    }


    #[test]
    fn test_load_agents_from_embedded_loads_agents() {
        // This test will only work if the embedded agents are available
        // It should not panic and should load some agents
        let config = load_agents_from_embedded();
        assert!(!config.is_empty(), "Should have loaded at least some agents");
        assert!(
            config.len() >= 100,
            "Should have loaded 100+ agents, got {}",
            config.len()
        );
    }

    #[test]
    fn test_load_agents_from_embedded_agents_have_required_fields() {
        let config = load_agents_from_embedded();
        for agent in config.all() {
            assert!(!agent.name.is_empty(), "Agent must have a name");
            assert!(!agent.model.is_empty(), "Agent must have a model");
            assert!(!agent.description.is_empty(), "Agent must have a description");
            // Tools can be empty, but if present should be valid strings
            for tool in &agent.tools {
                assert!(!tool.is_empty(), "Tool names must not be empty");
            }
        }
    }
}
