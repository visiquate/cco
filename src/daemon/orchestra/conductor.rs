//! Orchestra Conductor
//!
//! Core orchestration logic for the 119-agent Claude Orchestra system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::instructions::{AgentInstructions, AgentPrompt};
use super::workflow::Workflow;
use crate::daemon::llm_router::{LlmRouter, RoutingDecision};

/// Orchestra configuration loaded from orchestra-config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestraConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architect: ArchitectConfig,
    #[serde(rename = "codingAgents")]
    pub coding_agents: Vec<AgentConfig>,
    #[serde(rename = "integrationAgents", default)]
    pub integration_agents: Vec<AgentConfig>,
    #[serde(rename = "supportAgents")]
    pub support_agents: Vec<AgentConfig>,
    #[serde(rename = "knowledgeManager", default)]
    pub knowledge_manager: Option<KnowledgeManagerConfig>,
    #[serde(rename = "llmRouting", default)]
    pub llm_routing: Option<LlmRoutingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectConfig {
    pub name: String,
    pub model: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub model: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub specialties: Vec<String>,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub responsibilities: Vec<String>,
    pub prompt: String,
    #[serde(default)]
    pub apis: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeManagerConfig {
    pub enabled: bool,
    #[serde(rename = "baseDir")]
    pub base_dir: String,
    #[serde(rename = "embeddingDim")]
    pub embedding_dim: u32,
}

// Re-export from llm_router module
pub use crate::daemon::llm_router::router::LlmRoutingConfig;

// Re-export from llm_router module
pub use crate::daemon::llm_router::EndpointConfig;

/// Main orchestra conductor
pub struct OrchestraConductor {
    pub config: OrchestraConfig,
    llm_router: Option<LlmRouter>,
}

impl OrchestraConductor {
    /// Create a new conductor with loaded configuration
    pub fn new(config: OrchestraConfig) -> Self {
        // Initialize LLM router if routing config exists
        let llm_router = config
            .llm_routing
            .as_ref()
            .map(|routing_config| LlmRouter::new(routing_config.clone()));

        Self { config, llm_router }
    }

    /// Load configuration from file
    pub fn load_config<P: AsRef<Path>>(path: P) -> Result<OrchestraConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: OrchestraConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Load conductor from config file
    pub fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Self::load_config(path)?;
        Ok(Self::new(config))
    }

    /// Get total agent count
    pub fn get_total_agent_count(&self) -> usize {
        1 + self.config.coding_agents.len()
            + self.config.integration_agents.len()
            + self.config.support_agents.len()
    }

    /// Generate agent spawn instructions for all agents
    pub fn generate_agent_spawn_instructions(&self, user_requirement: &str) -> AgentInstructions {
        AgentInstructions {
            architect: self.generate_architect_instructions(user_requirement),
            coding_agents: self
                .config
                .coding_agents
                .iter()
                .map(|agent| self.generate_coding_agent_instructions(agent, user_requirement))
                .collect(),
            integration_agents: self
                .config
                .integration_agents
                .iter()
                .map(|agent| self.generate_integration_agent_instructions(agent, user_requirement))
                .collect(),
            support_agents: self
                .config
                .support_agents
                .iter()
                .map(|agent| self.generate_support_agent_instructions(agent, user_requirement))
                .collect(),
        }
    }

    /// Generate architect instructions
    fn generate_architect_instructions(&self, requirement: &str) -> AgentPrompt {
        let prompt = format!(
            r#"You are the Chief Architect for this project.

USER REQUIREMENT: {}

YOUR RESPONSIBILITIES:
1. Analyze the user requirement and break it down into technical components
2. Make strategic architecture decisions
3. Determine which coding agents are needed for each component
4. Coordinate with support agents (QA, Security, Documentation, Credentials)
5. Store all decisions in shared memory using hooks
6. Guide the coding agents with clear technical specifications

COORDINATION PROTOCOL:
- Search Knowledge Manager for relevant context before starting
- Store architecture decisions: 'node ~/git/cc-orchestra/src/knowledge-manager.js store "Decision: ..." --type decision'
- Share decisions with agents via Knowledge Manager storage
- Review code from all agents before approval
- Ensure security and QA agents review all implementations

OUTPUT:
- Architecture document
- Component breakdown
- Technology stack recommendations
- Task assignments for coding agents
- Security requirements
- Testing requirements"#,
            requirement
        );

        AgentPrompt {
            name: self.config.architect.name.clone(),
            agent_type: self.config.architect.agent_type.clone(),
            model: self.config.architect.model.clone(),
            prompt,
            description: "Architect analyzes requirements and guides team".to_string(),
            routing: None,
        }
    }

    /// Generate coding agent instructions
    fn generate_coding_agent_instructions(
        &self,
        agent: &AgentConfig,
        requirement: &str,
    ) -> AgentPrompt {
        // Determine routing for this agent
        let routing = self.route_task(&agent.agent_type, "implement");

        let languages = agent.languages.join("/");
        let specialties = agent.specialties.join(", ");

        let mut prompt = format!(
            r#"You are a {} specialist.

SPECIALTIES: {}

PROJECT REQUIREMENT: {}

YOUR RESPONSIBILITIES:
1. Check shared memory for architecture decisions from the Chief Architect
2. Implement components assigned to you in {}
3. Follow the architecture and coding standards
4. Write clean, well-documented code
5. Coordinate with other agents via shared memory
6. Notify QA agent when features are ready for testing
7. Address security concerns raised by Security Auditor

COORDINATION PROTOCOL:
- Before coding: 'node ~/git/cc-orchestra/src/knowledge-manager.js search "architect decisions"'
- After coding: 'node ~/git/cc-orchestra/src/knowledge-manager.js store "Edit: [filename] - [changes]" --type edit --agent {}'
- Store your decisions: 'node ~/git/cc-orchestra/src/knowledge-manager.js store "Implementation: ..." --type implementation --agent {}'
- Share completion status in Knowledge Manager

QUALITY STANDARDS:
- Write comprehensive tests
- Include inline documentation
- Follow language-specific best practices
- Ensure security best practices
- Optimize for performance"#,
            languages, specialties, requirement, languages, agent.name, agent.name
        );

        // Add custom endpoint note if not using Claude Code
        if let Some(ref routing_decision) = routing {
            if !routing_decision.use_claude_code {
                prompt.push_str(&format!(
                    r#"

NOTE: This coding task should be executed using the custom LLM endpoint.
The Claude Orchestra orchestrator will handle routing your implementation requests appropriately."#
                ));
            }
        }

        AgentPrompt {
            name: agent.name.clone(),
            agent_type: agent.agent_type.clone(),
            model: agent.model.clone(),
            prompt,
            description: format!("{} implements features", agent.name),
            routing,
        }
    }

    /// Generate integration agent instructions
    fn generate_integration_agent_instructions(
        &self,
        agent: &AgentConfig,
        requirement: &str,
    ) -> AgentPrompt {
        let role_specific = match agent.name.as_str() {
            "API Explorer" => {
                r#"
FOCUS: Explore and understand third-party APIs
- Test API endpoints and authentication
- Document API capabilities and limitations
- Create integration POCs
- Analyze rate limits and quotas
- Generate API client code
- Monitor API changes"#
            }
            "Salesforce API Specialist" => {
                r#"
FOCUS: Salesforce API integration
- Connect to Salesforce via REST/SOAP API
- Write optimized SOQL queries
- Handle OAuth 2.0 authentication
- Implement bulk operations
- Set up streaming API integrations
- Map Salesforce objects to application models
- Handle rate limits and governor limits"#
            }
            "Authentik API Specialist" => {
                r#"
FOCUS: Authentik authentication and API integration
- Configure OAuth2/OIDC flows with Authentik
- Manage users and groups via API
- Set up application providers
- Configure SAML integration
- Implement MFA workflows
- Synchronize user attributes
- Handle Authentik webhooks and events"#
            }
            _ => "",
        };

        let specialties = if !agent.specialties.is_empty() {
            format!(
                "\n\nAPI-SPECIFIC TASKS:\n- {}",
                agent.specialties.join("\n- ")
            )
        } else {
            String::new()
        };

        let apis = if !agent.apis.is_empty() {
            format!("\n\nAPI VERSIONS:\n- {}", agent.apis.join("\n- "))
        } else {
            String::new()
        };

        let responsibilities = format!("\n\nOUTPUT:\n- {}", agent.responsibilities.join("\n- "));

        let prompt = format!(
            r#"You are the {} for this project.

PROJECT REQUIREMENT: {}

YOUR ROLE: {}
{}

COORDINATION PROTOCOL:
- Retrieve architecture decisions from Knowledge Manager
- Coordinate with coding agents via Knowledge Manager
- Share API schemas and client code in Knowledge Manager
- Report integration challenges to Architect via Knowledge Manager
- Coordinate with Security Auditor on API authentication
- Store all findings and decisions in Knowledge Manager
{}{}{}
"#,
            agent.name, requirement, agent.role, role_specific, specialties, apis, responsibilities
        );

        AgentPrompt {
            name: agent.name.clone(),
            agent_type: agent.agent_type.clone(),
            model: agent.model.clone(),
            prompt,
            description: format!("{} performs {}", agent.name, agent.role),
            routing: None,
        }
    }

    /// Generate support agent instructions
    fn generate_support_agent_instructions(
        &self,
        agent: &AgentConfig,
        requirement: &str,
    ) -> AgentPrompt {
        let role_specific = match agent.name.as_str() {
            "Documentation Lead" => {
                r#"
FOCUS: Code-level documentation and API reference
- Inline code comments and docstrings
- API reference documentation with code examples
- Function/method documentation (JSDoc, docstrings, etc.)
- Code snippets and usage examples
- README code sections with examples
- Developer-focused documentation"#
            }
            "Technical Writer" => {
                r#"
FOCUS: Architecture documentation and user guides
- Architecture documentation and system design
- System design diagrams and flowcharts
- User guides and tutorials for end users
- How-to guides and best practices
- Conceptual documentation
- Integration guides and deployment guides
- High-level technical communication"#
            }
            "User Experience Designer" => {
                r#"
FOCUS: User experience design and validation
- Design UI/UX mockups and wireframes
- Analyze user flows and journeys
- Ensure accessibility compliance (WCAG 2.1 AA)
- Perform usability testing and validation
- Review mobile-first design implementation
- Final quality validation before completion
- Can block deployment if UX standards not met
- Coordinate with QA on usability testing"#
            }
            "QA Engineer" => {
                r#"
FOCUS: Integration and end-to-end testing
- Create integration test suites
- Test cross-component interactions
- Performance testing
- CI/CD pipeline integration
- Test coverage reports
- Coordinate with UX Designer on usability tests"#
            }
            "Security Auditor" => {
                r#"
FOCUS: Security analysis and vulnerability detection
- Review all code for security vulnerabilities
- Check for OWASP Top 10 issues
- Audit authentication/authorization
- Review credential handling
- Dependency vulnerability scanning
- Generate security reports"#
            }
            "Credential Manager" => {
                r#"
FOCUS: Secure credential management
- Design credential storage strategy (environment variables, secrets manager, etc.)
- Track all credentials used in the project
- Implement secure retrieval mechanisms
- Document credential rotation procedures
- Never store credentials in code
- Use /tmp/credentials.json for temporary storage during development
- Coordinate with Security Auditor"#
            }
            "DevOps Engineer" => {
                r#"
FOCUS: Infrastructure, builds, and deployments
- Docker and docker-compose configuration
- Kubernetes manifests and deployments
- CI/CD pipeline setup (GitHub Actions, GitLab CI)
- Infrastructure as Code (Terraform, CloudFormation)
- AWS infrastructure setup (ECS, ECR, CloudFormation)
- Monitoring and logging configuration
- Zero-downtime deployment strategies
- Container orchestration and scaling"#
            }
            _ => "",
        };

        let responsibilities = format!("\n\nOUTPUT:\n- {}", agent.responsibilities.join("\n- "));

        let prompt = format!(
            r#"You are the {} for this project.

PROJECT REQUIREMENT: {}

YOUR ROLE: {}
{}

COORDINATION PROTOCOL:
- Monitor Knowledge Manager for updates from coding agents
- Review implementations from your perspective
- Report findings to Knowledge Manager
- Coordinate with Chief Architect on critical issues
- Store all analysis and findings in Knowledge Manager
{}
"#,
            agent.name, requirement, agent.role, role_specific, responsibilities
        );

        AgentPrompt {
            name: agent.name.clone(),
            agent_type: agent.agent_type.clone(),
            model: agent.model.clone(),
            prompt,
            description: format!("{} performs {}", agent.name, agent.role),
            routing: None,
        }
    }

    /// Generate complete workflow for a user request
    pub fn generate_workflow(&self, user_requirement: &str) -> Workflow {
        let agents = self.generate_agent_spawn_instructions(user_requirement);

        Workflow::new(
            agents,
            self.config
                .knowledge_manager
                .as_ref()
                .map(|km| km.enabled)
                .unwrap_or(false),
        )
    }

    /// Route a task to appropriate LLM endpoint
    fn route_task(&self, agent_type: &str, task_type: &str) -> Option<RoutingDecision> {
        self.llm_router
            .as_ref()
            .map(|router| router.route_task(agent_type, Some(task_type)))
    }

    /// Get orchestra statistics
    pub fn get_stats(&self) -> OrchestraStats {
        OrchestraStats {
            total_agents: self.get_total_agent_count(),
            architect_count: 1,
            coding_agents_count: self.config.coding_agents.len(),
            integration_agents_count: self.config.integration_agents.len(),
            support_agents_count: self.config.support_agents.len(),
            knowledge_manager_enabled: self
                .config
                .knowledge_manager
                .as_ref()
                .map(|km| km.enabled)
                .unwrap_or(false),
            llm_routing_enabled: self
                .config
                .llm_routing
                .as_ref()
                .and_then(|lr| lr.endpoints.get("coding"))
                .map(|ep| ep.enabled)
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OrchestraStats {
    pub total_agents: usize,
    pub architect_count: usize,
    pub coding_agents_count: usize,
    pub integration_agents_count: usize,
    pub support_agents_count: usize,
    pub knowledge_manager_enabled: bool,
    pub llm_routing_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conductor_creation() {
        let config = OrchestraConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "test".to_string(),
            architect: ArchitectConfig {
                name: "Chief".to_string(),
                model: "opus".to_string(),
                agent_type: "chief-architect".to_string(),
                role: "leader".to_string(),
                capabilities: vec![],
                prompt: "test".to_string(),
            },
            coding_agents: vec![],
            integration_agents: vec![],
            support_agents: vec![],
            knowledge_manager: None,
            llm_routing: None,
        };

        let conductor = OrchestraConductor::new(config);
        assert_eq!(conductor.get_total_agent_count(), 1);
    }

    #[test]
    fn test_generate_instructions() {
        let config = OrchestraConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "test".to_string(),
            architect: ArchitectConfig {
                name: "Chief Architect".to_string(),
                model: "opus".to_string(),
                agent_type: "chief-architect".to_string(),
                role: "leader".to_string(),
                capabilities: vec![],
                prompt: "test".to_string(),
            },
            coding_agents: vec![],
            integration_agents: vec![],
            support_agents: vec![],
            knowledge_manager: None,
            llm_routing: None,
        };

        let conductor = OrchestraConductor::new(config);
        let instructions = conductor.generate_agent_spawn_instructions("Test requirement");

        assert_eq!(instructions.architect.name, "Chief Architect");
        assert!(instructions.architect.prompt.contains("Test requirement"));
    }
}
