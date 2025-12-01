//! Agent Instructions Generation
//!
//! Data structures for agent spawn instructions and prompts.

use serde::{Deserialize, Serialize};

use crate::daemon::llm_router::RoutingDecision;

/// Complete set of agent instructions for all agent types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstructions {
    pub architect: AgentPrompt,
    pub coding_agents: Vec<AgentPrompt>,
    pub integration_agents: Vec<AgentPrompt>,
    pub support_agents: Vec<AgentPrompt>,
}

/// Individual agent prompt with routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPrompt {
    pub name: String,
    pub agent_type: String,
    pub model: String,
    pub prompt: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingDecision>,
}

impl AgentInstructions {
    /// Get all agents as a flat list
    pub fn all_agents(&self) -> Vec<&AgentPrompt> {
        let mut agents = vec![&self.architect];
        agents.extend(self.coding_agents.iter());
        agents.extend(self.integration_agents.iter());
        agents.extend(self.support_agents.iter());
        agents
    }

    /// Get total agent count
    pub fn total_count(&self) -> usize {
        1 + self.coding_agents.len()
            + self.integration_agents.len()
            + self.support_agents.len()
    }

    /// Get agents that use custom endpoints
    pub fn custom_endpoint_agents(&self) -> Vec<&AgentPrompt> {
        self.all_agents()
            .into_iter()
            .filter(|a| {
                a.routing
                    .as_ref()
                    .map(|r| !r.use_claude_code)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get agents that use Claude Code
    pub fn claude_code_agents(&self) -> Vec<&AgentPrompt> {
        self.all_agents()
            .into_iter()
            .filter(|a| {
                a.routing
                    .as_ref()
                    .map(|r| r.use_claude_code)
                    .unwrap_or(true)
            })
            .collect()
    }
}

impl AgentPrompt {
    /// Check if this agent uses a custom endpoint
    pub fn uses_custom_endpoint(&self) -> bool {
        self.routing
            .as_ref()
            .map(|r| !r.use_claude_code)
            .unwrap_or(false)
    }

    /// Get the endpoint URL if using custom endpoint
    pub fn custom_endpoint_url(&self) -> Option<&str> {
        self.routing
            .as_ref()
            .and_then(|r| r.url.as_ref().map(|s| s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_prompt(name: &str, use_claude: bool) -> AgentPrompt {
        AgentPrompt {
            name: name.to_string(),
            agent_type: "test".to_string(),
            model: "haiku".to_string(),
            prompt: "test prompt".to_string(),
            description: "test".to_string(),
            routing: Some(RoutingDecision {
                endpoint: if use_claude { "claude" } else { "custom" }.to_string(),
                use_claude_code: use_claude,
                url: if use_claude {
                    None
                } else {
                    Some("http://example.com".to_string())
                },
                reason: "test".to_string(),
            }),
        }
    }

    #[test]
    fn test_agent_instructions_counts() {
        let instructions = AgentInstructions {
            architect: create_test_prompt("architect", true),
            coding_agents: vec![
                create_test_prompt("coder1", false),
                create_test_prompt("coder2", true),
            ],
            integration_agents: vec![create_test_prompt("integration1", true)],
            support_agents: vec![
                create_test_prompt("support1", true),
                create_test_prompt("support2", false),
            ],
        };

        assert_eq!(instructions.total_count(), 6);
        assert_eq!(instructions.all_agents().len(), 6);
        assert_eq!(instructions.custom_endpoint_agents().len(), 2);
        assert_eq!(instructions.claude_code_agents().len(), 4);
    }

    #[test]
    fn test_agent_prompt_custom_endpoint() {
        let claude_agent = create_test_prompt("claude", true);
        let custom_agent = create_test_prompt("custom", false);

        assert!(!claude_agent.uses_custom_endpoint());
        assert!(custom_agent.uses_custom_endpoint());

        assert_eq!(claude_agent.custom_endpoint_url(), None);
        assert_eq!(
            custom_agent.custom_endpoint_url(),
            Some("http://example.com")
        );
    }
}
