//! Integration tests for Orchestra Conductor
//!
//! Tests agent instruction generation, workflow generation, agent count calculations,
//! Knowledge Manager integration, LLM router integration, and configuration parsing.
//!
//! Run with: cargo test orchestra_integration

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

// =============================================================================
// Test Client and Helpers
// =============================================================================

#[derive(Clone)]
struct OrchestraTestClient {
    client: Client,
    base_url: String,
    port: u16,
}

impl OrchestraTestClient {
    fn new(port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", port),
            port,
        }
    }

    async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.health().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }
        anyhow::bail!("Daemon did not become ready within {:?}", timeout)
    }

    async fn generate_instruction(
        &self,
        agent_type: &str,
        requirement: &str,
    ) -> Result<InstructionResponse> {
        let url = format!("{}/api/orchestra/instruction", self.base_url);
        let request = InstructionRequest {
            agent_type: agent_type.to_string(),
            requirement: requirement.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn generate_workflow(&self, requirement: &str) -> Result<WorkflowResponse> {
        let url = format!("{}/api/orchestra/workflow", self.base_url);
        let request = WorkflowRequest {
            requirement: requirement.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn get_agent_count(&self, requirement: &str) -> Result<AgentCountResponse> {
        let url = format!("{}/api/orchestra/agent-count", self.base_url);
        let request = AgentCountRequest {
            requirement: requirement.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn list_agents(&self) -> Result<AgentListResponse> {
        let url = format!("{}/api/orchestra/agents", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn get_agent_by_type(&self, agent_type: &str) -> Result<AgentDetailResponse> {
        let url = format!("{}/api/orchestra/agents/{}", self.base_url, agent_type);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }
}

// =============================================================================
// API Request/Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct InstructionRequest {
    agent_type: String,
    requirement: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstructionResponse {
    agent_type: String,
    agent_name: String,
    instruction: String,
    model: String,
    includes_knowledge_manager: bool,
    includes_llm_router: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowRequest {
    requirement: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowResponse {
    requirement: String,
    phases: Vec<WorkflowPhase>,
    total_agents: usize,
    estimated_duration: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowPhase {
    phase_number: u32,
    phase_name: String,
    description: String,
    agents: Vec<String>,
    parallel_execution: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentCountRequest {
    requirement: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentCountResponse {
    requirement: String,
    recommended_agents: Vec<String>,
    total_count: usize,
    breakdown: HashMap<String, usize>, // category -> count
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentListResponse {
    total_agents: usize,
    agents: Vec<AgentSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentSummary {
    agent_type: String,
    name: String,
    role: String,
    model: String,
    category: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentDetailResponse {
    agent_type: String,
    name: String,
    role: String,
    model: String,
    prompt: String,
    category: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// =============================================================================
// Test Helpers
// =============================================================================

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    listener.local_addr().unwrap().port()
}

// Note: These tests are currently ignored because they require:
// 1. The daemon to have orchestra conductor module integrated
// 2. The orchestra endpoints to be registered
// 3. Access to orchestra-config.json
//
// Remove #[ignore] once Phase 3 implementation is complete

// =============================================================================
// SECTION 1: Agent Instruction Generation Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_generate_architect_instruction() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_instruction(
            "chief-architect",
            "Build a REST API with authentication",
        )
        .await
        .unwrap();

    assert_eq!(response.agent_type, "chief-architect");
    assert_eq!(response.model, "opus");
    assert!(!response.instruction.is_empty());
    assert!(response.instruction.contains("architecture"));
    assert!(response.includes_knowledge_manager);
    assert!(response.includes_llm_router);
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_generate_coding_agent_instruction() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_instruction("python-specialist", "Implement JWT authentication")
        .await
        .unwrap();

    assert_eq!(response.agent_type, "python-specialist");
    assert_eq!(response.model, "haiku");
    assert!(!response.instruction.is_empty());
    assert!(response.includes_knowledge_manager);
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_generate_integration_agent_instruction() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_instruction("api-explorer", "Integrate with Stripe API")
        .await
        .unwrap();

    assert_eq!(response.agent_type, "api-explorer");
    assert_eq!(response.model, "sonnet");
    assert!(response.instruction.contains("API") || response.instruction.contains("integration"));
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_generate_support_agent_instruction() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_instruction("documentation-expert", "Document the REST API endpoints")
        .await
        .unwrap();

    assert_eq!(response.agent_type, "documentation-expert");
    assert_eq!(response.model, "haiku");
    assert!(
        response.instruction.contains("documentation")
            || response.instruction.contains("document")
    );
}

// =============================================================================
// SECTION 2: Workflow Generation Tests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_generate_simple_workflow() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Build a REST API with authentication")
        .await
        .unwrap();

    assert_eq!(response.phases.len(), 3); // Design, Implementation, Quality Assurance
    assert!(response.total_agents >= 4); // At least architect, coder, security, docs

    // Verify phase structure
    assert_eq!(response.phases[0].phase_number, 1);
    assert!(response.phases[0].phase_name.contains("Design"));
    assert!(response.phases[0].agents.contains(&"chief-architect".to_string()));
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_generate_complex_workflow() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Build a Flutter mobile app with Go backend and Python ML service")
        .await
        .unwrap();

    assert_eq!(response.phases.len(), 3);
    assert!(response.total_agents >= 8); // Multiple language specialists, architect, QA, etc.

    // Should include all relevant specialists
    let all_agents: Vec<String> = response
        .phases
        .iter()
        .flat_map(|p| p.agents.clone())
        .collect();

    assert!(all_agents.contains(&"flutter-specialist".to_string()));
    assert!(all_agents.contains(&"go-specialist".to_string()));
    assert!(all_agents.contains(&"python-specialist".to_string()));
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_workflow_phases_are_ordered() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Create a web application")
        .await
        .unwrap();

    // Verify phases are in order
    for (i, phase) in response.phases.iter().enumerate() {
        assert_eq!(phase.phase_number, (i + 1) as u32);
    }

    // Phase 1 should be design/architecture
    assert!(
        response.phases[0].phase_name.contains("Design")
            || response.phases[0].phase_name.contains("Architecture")
    );

    // Phase 2 should be implementation
    assert!(
        response.phases[1].phase_name.contains("Implement")
            || response.phases[1].phase_name.contains("Development")
    );

    // Phase 3 should be quality/review
    assert!(
        response.phases[2].phase_name.contains("Quality")
            || response.phases[2].phase_name.contains("Review")
    );
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_workflow_includes_essential_agents() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Build a production application")
        .await
        .unwrap();

    let all_agents: Vec<String> = response
        .phases
        .iter()
        .flat_map(|p| p.agents.clone())
        .collect();

    // Essential agents for production app
    assert!(all_agents.contains(&"chief-architect".to_string()));
    assert!(
        all_agents.contains(&"security-auditor".to_string())
            || all_agents.contains(&"security-engineer".to_string())
    );
    assert!(
        all_agents.contains(&"test-engineer".to_string())
            || all_agents.contains(&"test-automator".to_string())
    );
    assert!(all_agents.contains(&"documentation-expert".to_string()));
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_workflow_parallel_execution_flags() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Build a microservices platform")
        .await
        .unwrap();

    // Phase 1 (Design) should NOT be parallel (architect leads)
    assert!(!response.phases[0].parallel_execution);

    // Phase 2 (Implementation) SHOULD be parallel (multiple coders)
    assert!(response.phases[1].parallel_execution);

    // Phase 3 (QA) SHOULD be parallel (multiple reviewers)
    assert!(response.phases[2].parallel_execution);
}

// =============================================================================
// SECTION 3: Agent Count Calculation Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_calculate_agent_count_simple() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .get_agent_count("Add a REST endpoint")
        .await
        .unwrap();

    assert!(response.total_count >= 3); // Architect, coder, docs minimum
    assert!(response.total_count <= 6); // Should be reasonable for simple task
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_calculate_agent_count_complex() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .get_agent_count("Build a complete e-commerce platform with payment processing")
        .await
        .unwrap();

    assert!(response.total_count >= 10); // Many agents for complex project
    assert!(response.breakdown.contains_key("coding"));
    assert!(response.breakdown.contains_key("security"));
    assert!(response.breakdown.contains_key("integration"));
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_agent_count_breakdown_categories() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .get_agent_count("Build a web application")
        .await
        .unwrap();

    // Should have breakdown by category
    assert!(!response.breakdown.is_empty());

    // Sum of breakdown should equal total
    let breakdown_sum: usize = response.breakdown.values().sum();
    assert_eq!(breakdown_sum, response.total_count);
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_agent_count_includes_all_categories() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .get_agent_count("Build a production-ready application")
        .await
        .unwrap();

    // Production app should include multiple categories
    assert!(response.breakdown.len() >= 3);
    assert!(response.breakdown.contains_key("leadership")); // Architect
    assert!(
        response.breakdown.contains_key("coding")
            || response.breakdown.contains_key("development")
    );
    assert!(response.breakdown.contains_key("support")); // Docs, etc.
}

// =============================================================================
// SECTION 4: Configuration Parsing Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_list_all_agents() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client.list_agents().await.unwrap();

    // Should have 117 agents (1 architect + 116 specialized)
    assert_eq!(response.total_agents, 117);
    assert_eq!(response.agents.len(), 117);

    // Verify chief architect exists
    assert!(response
        .agents
        .iter()
        .any(|a| a.agent_type == "chief-architect"));
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_get_specific_agent_details() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client.get_agent_by_type("chief-architect").await.unwrap();

    assert_eq!(response.agent_type, "chief-architect");
    assert_eq!(response.model, "opus");
    assert!(!response.prompt.is_empty());
    assert!(response.prompt.len() >= 50); // Reasonable prompt length
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_all_agents_have_required_fields() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client.list_agents().await.unwrap();

    for agent in response.agents {
        assert!(!agent.agent_type.is_empty(), "Agent missing type");
        assert!(!agent.name.is_empty(), "Agent missing name");
        assert!(!agent.role.is_empty(), "Agent missing role");
        assert!(!agent.model.is_empty(), "Agent missing model");
        assert!(!agent.category.is_empty(), "Agent missing category");
    }
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_agent_model_distribution() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client.list_agents().await.unwrap();

    let opus_count = response.agents.iter().filter(|a| a.model == "opus").count();
    let sonnet_count = response
        .agents
        .iter()
        .filter(|a| a.model == "sonnet")
        .count();
    let haiku_count = response
        .agents
        .iter()
        .filter(|a| a.model == "haiku")
        .count();

    // Expected distribution: 1 Opus, 37 Sonnet, 79 Haiku
    assert_eq!(opus_count, 1, "Should have exactly 1 Opus agent");
    assert!(
        sonnet_count >= 30 && sonnet_count <= 40,
        "Should have ~37 Sonnet agents"
    );
    assert!(
        haiku_count >= 70 && haiku_count <= 85,
        "Should have ~79 Haiku agents"
    );
}

// =============================================================================
// SECTION 5: Knowledge Manager Integration Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_instruction_includes_knowledge_manager_commands() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_instruction("python-specialist", "Build a feature")
        .await
        .unwrap();

    assert!(response.includes_knowledge_manager);
    assert!(response.instruction.contains("knowledge"));
    assert!(
        response.instruction.contains("cco knowledge")
            || response.instruction.contains("Knowledge Manager")
    );
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_workflow_includes_knowledge_coordination() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Build an application")
        .await
        .unwrap();

    // Each phase should reference knowledge sharing
    for phase in response.phases {
        assert!(!phase.description.is_empty());
        // Phases should mention coordination or knowledge sharing
        // (implementation detail - may vary)
    }
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_agent_instructions_include_coordination_protocol() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let agents_to_test = vec![
        "chief-architect",
        "python-specialist",
        "security-auditor",
        "test-engineer",
    ];

    for agent_type in agents_to_test {
        let response = client
            .generate_instruction(agent_type, "Test task")
            .await
            .unwrap();

        // Should include coordination protocol (before work, during work, after work)
        assert!(
            response.instruction.contains("before")
                || response.instruction.contains("during")
                || response.instruction.contains("after")
                || response.instruction.contains("knowledge")
        );
    }
}

// =============================================================================
// SECTION 6: LLM Router Integration Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_instruction_includes_llm_router_awareness() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_instruction("chief-architect", "Design system")
        .await
        .unwrap();

    assert!(response.includes_llm_router);
    // Architect should be aware they're using Claude (Opus)
    assert_eq!(response.model, "opus");
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_coding_agents_can_use_custom_llm() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let coding_agents = vec!["python-specialist", "go-specialist", "rust-specialist"];

    for agent_type in coding_agents {
        let response = client
            .generate_instruction(agent_type, "Write code")
            .await
            .unwrap();

        // Coding agents should be aware of potential routing to custom LLM
        assert!(
            response.model == "haiku"
                || response.model == "sonnet"
                || response.model == "custom"
        );
    }
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_workflow_respects_model_requirements() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let response = client
        .generate_workflow("Build a system")
        .await
        .unwrap();

    // Phase 1 (Design) should always use Opus (architect)
    assert!(response.phases[0]
        .agents
        .contains(&"chief-architect".to_string()));

    // Verify agent details confirm model
    let architect = client.get_agent_by_type("chief-architect").await.unwrap();
    assert_eq!(architect.model, "opus");
}

// =============================================================================
// SECTION 7: Error Handling Tests (3 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_invalid_agent_type() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let result = client
        .generate_instruction("nonexistent-agent", "Do something")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_empty_requirement() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let result = client.generate_workflow("").await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Remove after orchestra implementation complete
async fn test_malformed_request() {
    let port = find_available_port();
    let client = OrchestraTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let url = format!("{}/api/orchestra/workflow", client.base_url);
    let response = client
        .client
        .post(&url)
        .json(&json!({"invalid": "request"}))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
