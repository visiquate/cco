//! Orchestra Conductor Module
//!
//! Provides orchestration functionality for the 119-agent Claude Orchestra system.
//! Generates agent spawn instructions, workflows, and coordinates multi-agent development.

pub mod api;
pub mod conductor;
pub mod instructions;
pub mod workflow;

pub use conductor::{OrchestraConductor, OrchestraConfig};
pub use instructions::{AgentInstructions, AgentPrompt};
pub use workflow::{KnowledgeConfig, Phase1, Phase2, Phase3, Workflow};
