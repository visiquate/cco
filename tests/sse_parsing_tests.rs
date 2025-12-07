//! Unit tests for SSE (Server-Sent Events) parsing
//!
//! Tests the parsing of SSE event format with data: prefix handling,
//! multi-line events, and various edge cases.

/// Parse a single SSE event from bytes
/// SSE format: "event: <type>\ndata: <json>\n\n"
fn parse_sse_event(data: &str) -> Option<(String, String)> {
    let mut event_type = String::new();
    let mut event_data = String::new();

    for line in data.lines() {
        if line.starts_with("event: ") {
            event_type = line[7..].trim().to_string();
        } else if line.starts_with("data: ") {
            event_data = line[6..].trim().to_string();
        }
    }

    if !event_type.is_empty() && !event_data.is_empty() {
        Some((event_type, event_data))
    } else {
        None
    }
}

/// Extract text from content_block_delta event
fn extract_delta_text(json_data: &str) -> Option<String> {
    // Simple JSON extraction for delta.text
    // In production, use serde_json for robust parsing
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_data) {
        value
            .get("delta")
            .and_then(|d| d.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_sse_event() {
        let sse_data = "event: message_start\ndata: {\"type\":\"message_start\"}\n\n";
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, event_data) = result.unwrap();
        assert_eq!(event_type, "message_start");
        assert!(event_data.contains("message_start"));
    }

    #[test]
    fn test_parse_content_block_delta() {
        let sse_data = r#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

"#;
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, event_data) = result.unwrap();
        assert_eq!(event_type, "content_block_delta");

        let text = extract_delta_text(&event_data);
        assert!(text.is_some());
        assert_eq!(text.unwrap(), "Hello");
    }

    #[test]
    fn test_parse_multiple_events() {
        let events = vec![
            "event: message_start\ndata: {\"type\":\"message_start\"}\n\n",
            "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0}\n\n",
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"text\":\"Hi\"}}\n\n",
        ];

        let parsed: Vec<_> = events.iter().filter_map(|e| parse_sse_event(e)).collect();

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].0, "message_start");
        assert_eq!(parsed[1].0, "content_block_start");
        assert_eq!(parsed[2].0, "content_block_delta");
    }

    #[test]
    fn test_data_prefix_with_spaces() {
        // SSE spec allows spaces after colon
        let sse_data = "event:  message_start  \ndata:  {\"type\":\"test\"}  \n\n";
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, _) = result.unwrap();
        assert_eq!(event_type, "message_start");
    }

    #[test]
    fn test_empty_event() {
        let sse_data = "\n\n";
        let result = parse_sse_event(sse_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_malformed_event_no_data() {
        let sse_data = "event: message_start\n\n";
        let result = parse_sse_event(sse_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_malformed_event_no_type() {
        let sse_data = "data: {\"type\":\"test\"}\n\n";
        let result = parse_sse_event(sse_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_ping_event() {
        let sse_data = "event: ping\ndata: {}\n\n";
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, event_data) = result.unwrap();
        assert_eq!(event_type, "ping");
        assert_eq!(event_data, "{}");
    }

    #[test]
    fn test_message_stop_event() {
        let sse_data = "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, _) = result.unwrap();
        assert_eq!(event_type, "message_stop");
    }

    #[test]
    fn test_error_event() {
        let sse_data = r#"event: error
data: {"type":"error","error":{"type":"rate_limit_error","message":"Rate limit exceeded"}}

"#;
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, event_data) = result.unwrap();
        assert_eq!(event_type, "error");
        assert!(event_data.contains("rate_limit_error"));
    }

    #[test]
    fn test_usage_tracking_in_message_delta() {
        let sse_data = r#"event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":42}}

"#;
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (event_type, event_data) = result.unwrap();
        assert_eq!(event_type, "message_delta");

        // Verify usage data is present
        let value: serde_json::Value = serde_json::from_str(&event_data).unwrap();
        assert_eq!(value["usage"]["output_tokens"], 42);
    }

    #[test]
    fn test_incremental_text_accumulation() {
        // Simulate streaming response with incremental text
        let deltas = vec![
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"The"}}"#,
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" quick"}}"#,
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" brown"}}"#,
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" fox"}}"#,
        ];

        let mut accumulated = String::new();
        for delta_json in deltas {
            if let Some(text) = extract_delta_text(delta_json) {
                accumulated.push_str(&text);
            }
        }

        assert_eq!(accumulated, "The quick brown fox");
    }

    #[test]
    fn test_special_characters_in_text() {
        let sse_data = r#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello\nWorld\t!"}}

"#;
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (_, event_data) = result.unwrap();
        let text = extract_delta_text(&event_data);

        assert!(text.is_some());
        assert_eq!(text.unwrap(), "Hello\nWorld\t!");
    }

    #[test]
    fn test_unicode_in_text() {
        let sse_data = r#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"🚀 Unicode test 日本語"}}

"#;
        let result = parse_sse_event(sse_data);

        assert!(result.is_some());
        let (_, event_data) = result.unwrap();
        let text = extract_delta_text(&event_data);

        assert!(text.is_some());
        assert_eq!(text.unwrap(), "🚀 Unicode test 日本語");
    }
}
