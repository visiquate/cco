//! Compile-time embedded agent definitions
//!
//! This module contains agent definitions that are embedded into the binary
//! at compile time from the cco/config/agents/ directory.
//!
//! The embedding is performed by build.rs, which reads markdown files and
//! generates Rust code at compile time. The generated code includes:
//! - `create_embedded_agents()` - HashMap of all Agent definitions
//! - `EMBEDDED_AGENTS_COUNT` - Total count of embedded agents
//! - `EMBEDDED_AGENT_NAMES` - Array of agent names
//! - `AGENT_MODELS` - Lookup table of agent to model mappings
//! - `BUILD_STATS` - Build-time statistics

// Include the auto-generated agents code from build.rs
// This file is located in OUT_DIR and generated at compile time
include!(concat!(env!("OUT_DIR"), "/agents.rs"));

/// Initialize agents from embedded data
///
/// Creates a HashMap of Agent configurations from the compile-time embedded data.
/// This is the primary function for accessing all embedded agents at runtime.
pub fn initialize_embedded_agents() -> HashMap<String, Agent> {
    create_embedded_agents()
}

/// Get list of all embedded agent names
pub fn embedded_agent_names() -> &'static [&'static str] {
    EMBEDDED_AGENT_NAMES
}

/// Get the model assigned to an agent by name
pub fn agent_model(agent_name: &str) -> Option<&'static str> {
    AGENT_MODELS
        .iter()
        .find(|(name, _)| name == &agent_name)
        .map(|(_, model)| *model)
}

/// Get build-time statistics about embedded agents
pub fn build_stats() -> &'static str {
    BUILD_STATS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_agents_not_empty() {
        let count = EMBEDDED_AGENTS_COUNT;
        assert!(
            count > 0,
            "No embedded agents found - check build.rs configuration"
        );
    }

    #[test]
    fn test_embedded_agent_names_not_empty() {
        assert!(
            !embedded_agent_names().is_empty(),
            "Embedded agent names list is empty"
        );
    }

    #[test]
    fn test_initialize_embedded_agents_creates_hashmap() {
        let agents = initialize_embedded_agents();
        assert_eq!(
            agents.len(),
            EMBEDDED_AGENTS_COUNT,
            "Embedded agents count mismatch"
        );
    }

    #[test]
    fn test_agent_model_lookup_works() {
        let agents = initialize_embedded_agents();
        for agent_name in embedded_agent_names() {
            let model = agent_model(agent_name);
            assert!(
                model.is_some(),
                "Could not find model for agent: {}",
                agent_name
            );

            // Verify agent exists in HashMap
            assert!(
                agents.contains_key(*agent_name),
                "Agent not found in HashMap: {}",
                agent_name
            );
        }
    }

    #[test]
    fn test_valid_model_names() {
        let valid_models = ["opus", "sonnet", "haiku"];

        for agent_name in embedded_agent_names() {
            let model = agent_model(agent_name).expect("Model should exist");
            assert!(
                valid_models.contains(&model),
                "Invalid model '{}' for agent '{}', must be opus/sonnet/haiku",
                model,
                agent_name
            );
        }
    }

    #[test]
    fn test_build_stats_available() {
        // Verify BUILD_STATS is available
        let stats = build_stats();
        assert!(!stats.is_empty(), "BUILD_STATS is empty");
        assert!(
            stats.contains("Embedded Agents:"),
            "BUILD_STATS missing 'Embedded Agents' text"
        );
    }
}
