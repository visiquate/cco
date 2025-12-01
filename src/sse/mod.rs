//! SSE (Server-Sent Events) client module for streaming analytics from CCO proxy
//!
//! This module provides an SSE client that connects to the CCO proxy's `/api/stream` endpoint
//! to receive real-time API call events and record them in the MetricsEngine.

pub mod client;

pub use client::SseClient;
