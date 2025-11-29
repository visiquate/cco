//! LLM Router module
//!
//! Routes agent tasks to appropriate LLM endpoints based on task type:
//! - Architecture tasks → Claude (always)
//! - Coding tasks → custom LLM if configured, else Claude
//!
//! Supports Ollama and OpenAI-compatible endpoints with bearer token authentication.

pub mod api;
pub mod client;
pub mod endpoints;
pub mod router;

pub use api::{llm_router_routes, LlmRouterState};
pub use client::{LlmClient, LlmOptions, LlmResponse};
pub use endpoints::{EndpointConfig, EndpointType};
pub use router::{LlmRouter, RoutingDecision};
