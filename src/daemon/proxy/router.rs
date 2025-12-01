//! Agent routing logic for model selection
//!
//! Determines which model backend (Anthropic or Azure) should handle
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
}
