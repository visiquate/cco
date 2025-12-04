//! SSE (Server-Sent Events) utilities
//!
//! This module provides:
//! - SSE client for streaming analytics from CCO proxy (`client`)
//! - SSE parser for parsing LiteLLM/Anthropic streaming responses (`parser`)

pub mod client;
pub mod parser;

pub use client::SseClient;
pub use parser::{SseEvent, SseParser};
