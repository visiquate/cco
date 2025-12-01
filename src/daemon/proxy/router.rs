//! Agent routing logic for model selection
//!
//! Determines which model backend (Anthropic, Azure, or DeepSeek) should handle
//! requests based on agent type classification.

use tracing::debug;

/// List of agent types that should be routed to Azure GPT-5.1-codex-mini
pub const AZURE_ROUTED_AGENTS: &[&str] = &[
    "code-reviewer",
    "architect-review",
    "flutter-go-reviewer",
    "test-engineer",
    "test-automator",
    "qa-engineer",
    "mcp-testing-engineer",
    "security-auditor",
];

/// List of agent types that should be routed to DeepSeek V3.1
///
/// DeepSeek is excellent for coding tasks, research, and code review
pub const DEEPSEEK_ROUTED_AGENTS: &[&str] = &[
    "python-specialist",
    "rust-specialist",
    "go-specialist",
    "technical-researcher",
    "api-explorer",
    "data-scientist",
    "ml-engineer",
];

/// Determine if a request should be routed to Azure based on agent type
///
/// Examines the agent type and checks if it's in the list of agents
/// that should use Azure GPT-5.1-codex-mini instead of Claude Opus.
///
/// # Arguments
/// * `agent_type` - The agent type from the Task tool call
///
/// # Returns
/// `true` if the agent should use Azure, `false` for pass-through to Claude
pub fn should_route_to_azure(agent_type: &str) -> bool {
    let normalized = agent_type.to_lowercase();
    let normalized = normalized.trim();
    let should_route = AZURE_ROUTED_AGENTS
        .iter()
        .any(|&agent| agent.eq_ignore_ascii_case(normalized));

    debug!(
        agent_type = normalized,
        route_to_azure = should_route,
        "Evaluated agent routing decision"
    );

    should_route
}

/// Determine if a request should be routed to DeepSeek V3.1 based on agent type
///
/// Examines the agent type and checks if it's in the list of agents
/// that should use DeepSeek V3.1 for coding, research, and analysis tasks.
///
/// # Arguments
/// * `agent_type` - The agent type from the Task tool call
///
/// # Returns
/// `true` if the agent should use DeepSeek, `false` otherwise
pub fn should_route_to_deepseek(agent_type: &str) -> bool {
    let normalized = agent_type.to_lowercase();
    let normalized = normalized.trim();
    let should_route = DEEPSEEK_ROUTED_AGENTS
        .iter()
        .any(|&agent| agent.eq_ignore_ascii_case(normalized));

    debug!(
        agent_type = normalized,
        route_to_deepseek = should_route,
        "Evaluated DeepSeek routing decision"
    );

    should_route
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_reviewer_routes_to_azure() {
        assert!(should_route_to_azure("code-reviewer"));
    }

    #[test]
    fn test_architect_review_routes_to_azure() {
        assert!(should_route_to_azure("architect-review"));
    }

    #[test]
    fn test_test_engineer_routes_to_azure() {
        assert!(should_route_to_azure("test-engineer"));
    }

    #[test]
    fn test_security_auditor_routes_to_azure() {
        assert!(should_route_to_azure("security-auditor"));
    }

    #[test]
    fn test_case_insensitive_matching() {
        assert!(should_route_to_azure("CODE-REVIEWER"));
        assert!(should_route_to_azure("Code-Reviewer"));
        assert!(should_route_to_azure("ARCHITECT-REVIEW"));
    }

    #[test]
    fn test_whitespace_trimming() {
        assert!(should_route_to_azure("  code-reviewer  "));
    }

    #[test]
    fn test_unknown_agent_passes_through() {
        assert!(!should_route_to_azure("unknown-agent"));
    }

    #[test]
    fn test_chief_architect_passes_through() {
        assert!(!should_route_to_azure("chief-architect"));
    }

    #[test]
    fn test_partial_match_does_not_route() {
        assert!(!should_route_to_azure("code"));
        assert!(!should_route_to_azure("reviewer"));
    }

    #[test]
    fn test_all_configured_agents_route() {
        for agent in AZURE_ROUTED_AGENTS {
            assert!(should_route_to_azure(agent), "Agent {} should route to Azure", agent);
        }
    }

    #[test]
    fn test_python_specialist_routes_to_deepseek() {
        assert!(should_route_to_deepseek("python-specialist"));
    }

    #[test]
    fn test_rust_specialist_routes_to_deepseek() {
        assert!(should_route_to_deepseek("rust-specialist"));
    }

    #[test]
    fn test_go_specialist_routes_to_deepseek() {
        assert!(should_route_to_deepseek("go-specialist"));
    }

    #[test]
    fn test_technical_researcher_routes_to_deepseek() {
        assert!(should_route_to_deepseek("technical-researcher"));
    }

    #[test]
    fn test_api_explorer_routes_to_deepseek() {
        assert!(should_route_to_deepseek("api-explorer"));
    }

    #[test]
    fn test_deepseek_case_insensitive_matching() {
        assert!(should_route_to_deepseek("PYTHON-SPECIALIST"));
        assert!(should_route_to_deepseek("Python-Specialist"));
        assert!(should_route_to_deepseek("RUST-SPECIALIST"));
    }

    #[test]
    fn test_deepseek_whitespace_trimming() {
        assert!(should_route_to_deepseek("  python-specialist  "));
    }

    #[test]
    fn test_unknown_agent_doesnt_route_to_deepseek() {
        assert!(!should_route_to_deepseek("unknown-agent"));
        assert!(!should_route_to_deepseek("chief-architect"));
    }

    #[test]
    fn test_all_deepseek_configured_agents_route() {
        for agent in DEEPSEEK_ROUTED_AGENTS {
            assert!(should_route_to_deepseek(agent), "Agent {} should route to DeepSeek", agent);
        }
    }

    #[test]
    fn test_deepseek_and_azure_are_mutually_exclusive() {
        // Ensure no agent is in both lists
        for agent in DEEPSEEK_ROUTED_AGENTS {
            assert!(!AZURE_ROUTED_AGENTS.contains(agent),
                "Agent {} should not be in both DeepSeek and Azure lists", agent);
        }
    }
}
