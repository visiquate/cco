//! SSE (Server-Sent Events) parser for LiteLLM streaming responses
//!
//! This module provides utilities for parsing SSE streams from LiteLLM and Anthropic.
//! It handles the low-level SSE format parsing, including:
//! - Event type parsing (event: ...)
//! - Data line parsing (data: ...)
//! - Multi-line data support
//! - [DONE] termination signals
//! - Incremental parsing from byte chunks
//!
//! ## SSE Format
//!
//! ```text
//! event: message_start
//! data: {"type":"message_start","message":{...}}
//!
//! event: content_block_delta
//! data: {"type":"content_block_delta","delta":{"text":"Hello"}}
//!
//! event: message_stop
//! data: [DONE]
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use cco::sse::parser::SseParser;
//! use futures::StreamExt;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Get a byte stream from reqwest
//! let response = reqwest::get("http://example.com/stream").await?;
//! let byte_stream = response.bytes_stream();
//!
//! // Parse SSE events
//! let mut event_stream = SseParser::parse_stream(Box::pin(byte_stream));
//!
//! while let Some(event_result) = event_stream.next().await {
//!     let event = event_result?;
//!     if event.is_done() {
//!         break;
//!     }
//!     println!("Event: {:?}", event.event);
//!     println!("Data: {}", event.data);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, trace, warn};

/// Type alias for a byte stream from reqwest
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// SSE event parsed from stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    /// Event type (e.g., "message_start", "content_block_delta", "message_stop")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,

    /// Event data (JSON string or plain text)
    pub data: String,

    /// Event ID (rarely used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Retry interval in milliseconds (rarely used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<u64>,
}

impl SseEvent {
    /// Check if this is a termination event (data: [DONE])
    pub fn is_done(&self) -> bool {
        self.data.trim() == "[DONE]"
    }

    /// Parse event data as JSON
    pub fn parse_json<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(&self.data).context("Failed to parse SSE event data as JSON")
    }
}

/// SSE parser for consuming Server-Sent Events from a byte stream
///
/// This parser handles incremental parsing of SSE events, properly handling:
/// - Multi-line data fields
/// - Incomplete events across chunk boundaries
/// - Comments (lines starting with ':')
/// - All SSE field types (event, data, id, retry)
pub struct SseParser {
    /// Buffer for incomplete lines
    buffer: String,
    /// Current event being assembled
    current_event: Option<SseEvent>,
}

impl SseParser {
    /// Create a new SSE parser
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            current_event: None,
        }
    }

    /// Parse a byte stream into SSE events
    ///
    /// Returns a stream of parsed SSE events. Filters out empty events and comments.
    /// Events are yielded as they are complete (after blank line separator).
    ///
    /// # Arguments
    ///
    /// * `byte_stream` - A pinned boxed stream of byte chunks from reqwest
    ///
    /// # Returns
    ///
    /// A stream of `Result<SseEvent>` that yields events as they are parsed
    pub fn parse_stream(
        mut byte_stream: ByteStream,
    ) -> Pin<Box<dyn Stream<Item = Result<SseEvent>> + Send>> {
        Box::pin(async_stream::stream! {
            let mut parser = SseParser::new();

            while let Some(chunk_result) = byte_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let text = match std::str::from_utf8(&chunk) {
                            Ok(t) => t,
                            Err(e) => {
                                warn!(error = %e, "Invalid UTF-8 in SSE stream");
                                continue;
                            }
                        };

                        // Parse events from the chunk
                        for event in parser.process_chunk(text) {
                            yield Ok(event);
                        }
                    }
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Stream error: {}", e));
                        break;
                    }
                }
            }

            // Flush any remaining event
            if let Some(event) = parser.flush() {
                yield Ok(event);
            }
        })
    }

    /// Process a chunk of text and extract complete events
    ///
    /// This method handles incremental parsing, maintaining state across chunk boundaries.
    /// Events are only returned when complete (after encountering a blank line).
    pub fn process_chunk(&mut self, text: &str) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // Add new text to buffer
        self.buffer.push_str(text);

        // Process complete lines
        while let Some(line_end) = self.buffer.find('\n') {
            let line = self.buffer[..line_end].to_string();
            self.buffer.drain(..=line_end);

            // Remove trailing \r if present (handles both \n and \r\n)
            let line = line.trim_end_matches('\r');

            trace!(line = %line, "Processing SSE line");

            // Empty line signals end of event
            if line.is_empty() {
                if let Some(event) = self.current_event.take() {
                    // Only yield events with actual data
                    if !event.data.is_empty() {
                        debug!(event = ?event.event, data_len = event.data.len(), "Complete SSE event");
                        events.push(event);
                    }
                }
                continue;
            }

            // Skip comments
            if line.starts_with(':') {
                trace!("Skipping SSE comment");
                continue;
            }

            // Parse field
            if let Some((field, value)) = line.split_once(':') {
                let value = value.trim_start();

                // Ensure we have a current event to populate
                if self.current_event.is_none() {
                    self.current_event = Some(SseEvent {
                        event: None,
                        data: String::new(),
                        id: None,
                        retry: None,
                    });
                }

                if let Some(ref mut event) = self.current_event {
                    match field {
                        "event" => {
                            event.event = Some(value.to_string());
                        }
                        "data" => {
                            // Append data (can be multiple lines)
                            if !event.data.is_empty() {
                                event.data.push('\n');
                            }
                            event.data.push_str(value);
                        }
                        "id" => {
                            event.id = Some(value.to_string());
                        }
                        "retry" => {
                            if let Ok(retry) = value.parse::<u64>() {
                                event.retry = Some(retry);
                            }
                        }
                        _ => {
                            trace!(field = %field, "Unknown SSE field");
                        }
                    }
                }
            }
        }

        events
    }

    /// Flush any remaining incomplete event
    ///
    /// This should be called when the stream ends to extract any incomplete event
    /// that hasn't been terminated with a blank line.
    fn flush(&mut self) -> Option<SseEvent> {
        self.current_event.take().filter(|e| !e.data.is_empty())
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to consume an SSE stream and collect all events
///
/// Useful for testing and debugging. Returns all events until stream completion or [DONE].
///
/// # Arguments
///
/// * `byte_stream` - A byte stream from reqwest
///
/// # Returns
///
/// A vector of all parsed SSE events up to termination
///
/// # Errors
///
/// Returns an error if parsing fails or the stream encounters an error
pub async fn collect_sse_events(byte_stream: ByteStream) -> Result<Vec<SseEvent>> {
    let mut stream = SseParser::parse_stream(byte_stream);
    let mut events = Vec::new();

    while let Some(event_result) = stream.next().await {
        let event = event_result?;

        // Check for termination
        if event.is_done() {
            debug!("Received [DONE] event, stopping collection");
            break;
        }

        events.push(event);
    }

    Ok(events)
}

/// Helper function to consume an SSE stream and extract text content
///
/// Specifically parses Anthropic-format streaming responses and concatenates text deltas.
/// This is useful for getting the complete response text from a streaming API call.
///
/// # Arguments
///
/// * `byte_stream` - A byte stream from reqwest containing Anthropic SSE events
///
/// # Returns
///
/// The concatenated text content from all content_block_delta events
///
/// # Errors
///
/// Returns an error if parsing fails or the stream encounters an error
pub async fn extract_text_from_stream(byte_stream: ByteStream) -> Result<String> {
    let mut stream = SseParser::parse_stream(byte_stream);
    let mut text = String::new();

    while let Some(event_result) = stream.next().await {
        let event = event_result?;

        // Check for termination
        if event.is_done() {
            break;
        }

        // Try to parse as JSON and extract text
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) {
            // Handle content_block_delta events with text deltas
            if json.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                if let Some(delta_text) = json
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                {
                    text.push_str(delta_text);
                }
            }
            // Handle text_delta events (alternative format)
            else if json.get("type").and_then(|t| t.as_str()) == Some("text_delta") {
                if let Some(delta_text) = json.get("text").and_then(|t| t.as_str()) {
                    text.push_str(delta_text);
                }
            }
        }
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[test]
    fn test_sse_event_is_done() {
        let event = SseEvent {
            event: None,
            data: "[DONE]".to_string(),
            id: None,
            retry: None,
        };
        assert!(event.is_done());

        let event = SseEvent {
            event: None,
            data: "  [DONE]  ".to_string(),
            id: None,
            retry: None,
        };
        assert!(event.is_done());

        let event = SseEvent {
            event: None,
            data: "not done".to_string(),
            id: None,
            retry: None,
        };
        assert!(!event.is_done());
    }

    #[test]
    fn test_sse_parser_simple_event() {
        let mut parser = SseParser::new();

        let chunk = "event: message_start\ndata: {\"type\":\"message_start\"}\n\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, Some("message_start".to_string()));
        assert_eq!(events[0].data, "{\"type\":\"message_start\"}");
    }

    #[test]
    fn test_sse_parser_multiline_data() {
        let mut parser = SseParser::new();

        let chunk = "data: line 1\ndata: line 2\n\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line 1\nline 2");
    }

    #[test]
    fn test_sse_parser_comment() {
        let mut parser = SseParser::new();

        let chunk = ": this is a comment\ndata: actual data\n\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "actual data");
    }

    #[test]
    fn test_sse_parser_done_event() {
        let mut parser = SseParser::new();

        let chunk = "data: [DONE]\n\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 1);
        assert!(events[0].is_done());
    }

    #[test]
    fn test_sse_parser_multiple_events() {
        let mut parser = SseParser::new();

        let chunk = "event: start\ndata: first\n\nevent: end\ndata: second\n\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event, Some("start".to_string()));
        assert_eq!(events[0].data, "first");
        assert_eq!(events[1].event, Some("end".to_string()));
        assert_eq!(events[1].data, "second");
    }

    #[test]
    fn test_sse_parser_chunked() {
        let mut parser = SseParser::new();

        // First chunk - incomplete event
        let events = parser.process_chunk("event: test\n");
        assert_eq!(events.len(), 0);

        // Second chunk - complete the event
        let events = parser.process_chunk("data: hello\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, Some("test".to_string()));
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn test_sse_parser_crlf() {
        let mut parser = SseParser::new();

        // Test with \r\n line endings
        let chunk = "event: test\r\ndata: hello\r\n\r\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, Some("test".to_string()));
        assert_eq!(events[0].data, "hello");
    }

    #[tokio::test]
    async fn test_parse_stream() {
        let data = "event: message\ndata: test\n\ndata: [DONE]\n\n";
        let bytes = Bytes::from(data);
        let byte_stream: ByteStream = Box::pin(stream::once(async move { Ok(bytes) }));

        let events = collect_sse_events(byte_stream).await.unwrap();

        assert_eq!(events.len(), 1); // [DONE] stops collection
        assert_eq!(events[0].event, Some("message".to_string()));
        assert_eq!(events[0].data, "test");
    }

    #[tokio::test]
    async fn test_extract_text_from_stream() {
        let data = r#"event: message_start
data: {"type":"message_start"}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"text":" World"}}

data: [DONE]

"#;
        let bytes = Bytes::from(data);
        let byte_stream: ByteStream = Box::pin(stream::once(async move { Ok(bytes) }));

        let text = extract_text_from_stream(byte_stream).await.unwrap();

        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_parse_json() {
        #[derive(Deserialize)]
        struct TestData {
            r#type: String,
        }

        let event = SseEvent {
            event: None,
            data: r#"{"type":"message_start"}"#.to_string(),
            id: None,
            retry: None,
        };

        let parsed: TestData = event.parse_json().unwrap();
        assert_eq!(parsed.r#type, "message_start");
    }

    #[test]
    fn test_flush_incomplete_event() {
        let mut parser = SseParser::new();

        // Process incomplete event (with newline but no blank line to complete it)
        parser.process_chunk("data: incomplete\n");

        // Flush should return the event
        let event = parser.flush();
        assert!(event.is_some());
        assert_eq!(event.unwrap().data, "incomplete");
    }

    #[test]
    fn test_id_and_retry_fields() {
        let mut parser = SseParser::new();

        let chunk = "id: 123\nevent: test\ndata: hello\nretry: 5000\n\n";
        let events = parser.process_chunk(chunk);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, Some("123".to_string()));
        assert_eq!(events[0].retry, Some(5000));
    }
}
