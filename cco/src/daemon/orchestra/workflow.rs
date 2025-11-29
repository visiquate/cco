//! Workflow Generation
//!
//! Generates complete 3-phase workflows for orchestra coordination.

use serde::{Deserialize, Serialize};

use super::instructions::AgentInstructions;

/// Complete workflow for an orchestra project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub phase1_agent_spawn: Phase1,
    pub phase2_execution: Phase2,
    pub phase3_integration: Phase3,
    pub knowledge_management: KnowledgeConfig,
}

/// Phase 1: Agent spawn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase1 {
    pub description: String,
    pub note: String,
    pub agents: AgentInstructions,
}

/// Phase 2: Execution flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase2 {
    pub description: String,
    pub flow: Vec<String>,
}

/// Phase 3: Integration and final review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase3 {
    pub description: String,
    pub steps: Vec<String>,
}

/// Knowledge management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    pub description: String,
    pub enabled: bool,
    pub operations: Vec<String>,
}

impl Workflow {
    /// Create a new workflow from agent instructions
    pub fn new(agents: AgentInstructions, knowledge_enabled: bool) -> Self {
        Self {
            phase1_agent_spawn: Phase1 {
                description: "Spawn all agents in parallel using Claude Code's Task tool"
                    .to_string(),
                note: "ALL agents must be spawned in a SINGLE message".to_string(),
                agents,
            },
            phase2_execution: Phase2 {
                description: "Agents execute their tasks with Knowledge Manager coordination"
                    .to_string(),
                flow: vec![
                    "1. Architect analyzes requirement and creates architecture".to_string(),
                    "2. Architect stores decisions in Knowledge Manager".to_string(),
                    "3. Coding agents retrieve architecture from Knowledge Manager and implement"
                        .to_string(),
                    "4. QA agent monitors for completed features and tests".to_string(),
                    "5. Security agent reviews code for vulnerabilities".to_string(),
                    "6. Documentation agent creates docs for all components".to_string(),
                    "7. Credential manager tracks and secures all credentials".to_string(),
                    "8. All agents store status updates in Knowledge Manager".to_string(),
                ],
            },
            phase3_integration: Phase3 {
                description: "Integration and final review".to_string(),
                steps: vec![
                    "QA agent runs full integration test suite".to_string(),
                    "Security agent performs final security audit".to_string(),
                    "Documentation agent finalizes all docs".to_string(),
                    "Architect reviews all outputs and approves".to_string(),
                    "Credential manager documents all credential requirements".to_string(),
                ],
            },
            knowledge_management: KnowledgeConfig {
                description: "Knowledge capture and retention".to_string(),
                enabled: knowledge_enabled,
                operations: vec![
                    "Automatic knowledge capture during implementation".to_string(),
                    "Pre-compaction knowledge storage".to_string(),
                    "Post-compaction context retrieval".to_string(),
                    "Per-repository knowledge isolation".to_string(),
                    "Semantic search for relevant context".to_string(),
                ],
            },
        }
    }

    /// Generate a human-readable summary of the workflow
    pub fn summary(&self) -> String {
        format!(
            "Orchestra Workflow:\n\
             - Total Agents: {}\n\
             - Architect: {}\n\
             - Coding Agents: {}\n\
             - Integration Agents: {}\n\
             - Support Agents: {}\n\
             - Knowledge Manager: {}\n\
             \n\
             Phase 1: {}\n\
             Phase 2: {} steps\n\
             Phase 3: {} steps",
            self.phase1_agent_spawn.agents.total_count(),
            self.phase1_agent_spawn.agents.architect.name,
            self.phase1_agent_spawn.agents.coding_agents.len(),
            self.phase1_agent_spawn.agents.integration_agents.len(),
            self.phase1_agent_spawn.agents.support_agents.len(),
            if self.knowledge_management.enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.phase1_agent_spawn.description,
            self.phase2_execution.flow.len(),
            self.phase3_integration.steps.len()
        )
    }

    /// Get all agent names in spawn order
    pub fn agent_spawn_order(&self) -> Vec<String> {
        let mut names = vec![self.phase1_agent_spawn.agents.architect.name.clone()];
        names.extend(
            self.phase1_agent_spawn
                .agents
                .coding_agents
                .iter()
                .map(|a| a.name.clone()),
        );
        names.extend(
            self.phase1_agent_spawn
                .agents
                .integration_agents
                .iter()
                .map(|a| a.name.clone()),
        );
        names.extend(
            self.phase1_agent_spawn
                .agents
                .support_agents
                .iter()
                .map(|a| a.name.clone()),
        );
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::orchestra::instructions::AgentPrompt;

    fn create_test_agent(name: &str) -> AgentPrompt {
        AgentPrompt {
            name: name.to_string(),
            agent_type: "test".to_string(),
            model: "haiku".to_string(),
            prompt: "test".to_string(),
            description: "test".to_string(),
            routing: None,
        }
    }

    #[test]
    fn test_workflow_creation() {
        let instructions = AgentInstructions {
            architect: create_test_agent("Chief Architect"),
            coding_agents: vec![create_test_agent("Python Expert")],
            integration_agents: vec![create_test_agent("API Explorer")],
            support_agents: vec![create_test_agent("QA Engineer")],
        };

        let workflow = Workflow::new(instructions, true);

        assert_eq!(workflow.phase1_agent_spawn.agents.total_count(), 4);
        assert!(workflow.knowledge_management.enabled);
        assert_eq!(workflow.phase2_execution.flow.len(), 8);
        assert_eq!(workflow.phase3_integration.steps.len(), 5);
    }

    #[test]
    fn test_agent_spawn_order() {
        let instructions = AgentInstructions {
            architect: create_test_agent("Chief Architect"),
            coding_agents: vec![
                create_test_agent("Python Expert"),
                create_test_agent("Go Expert"),
            ],
            integration_agents: vec![],
            support_agents: vec![create_test_agent("QA Engineer")],
        };

        let workflow = Workflow::new(instructions, false);
        let order = workflow.agent_spawn_order();

        assert_eq!(order.len(), 4);
        assert_eq!(order[0], "Chief Architect");
        assert_eq!(order[1], "Python Expert");
        assert_eq!(order[2], "Go Expert");
        assert_eq!(order[3], "QA Engineer");
    }

    #[test]
    fn test_workflow_summary() {
        let instructions = AgentInstructions {
            architect: create_test_agent("Chief Architect"),
            coding_agents: vec![create_test_agent("Python Expert")],
            integration_agents: vec![],
            support_agents: vec![],
        };

        let workflow = Workflow::new(instructions, true);
        let summary = workflow.summary();

        assert!(summary.contains("Total Agents: 2"));
        assert!(summary.contains("Chief Architect"));
        assert!(summary.contains("Knowledge Manager: enabled"));
    }
}
