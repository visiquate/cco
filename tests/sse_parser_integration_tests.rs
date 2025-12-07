//! Integration tests for SSE parser with LiteLLM streaming
//!
//! These tests verify that the SSE parser correctly handles real-world
//! streaming responses from LiteLLM/Anthropic.

use bytes::Bytes;
use cco::sse::parser::{collect_sse_events, extract_text_from_stream, SseParser};
use futures::stream;
use futures::StreamExt;

/// Test parsing a complete Anthropic-style SSE stream
#[tokio::test]
async fn test_parse_anthropic_stream() {
    let sse_data = r#"event: message_start
data: {"type":"message_start","message":{"id":"msg_123","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4-5-20250929"}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" world!"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":5}}

event: message_stop
data: {"type":"message_stop"}
"#;

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    // Verify event count
    assert_eq!(events.len(), 7, "Should parse 7 events");

    // Verify event types
    assert_eq!(events[0].event, Some("message_start".to_string()));
    assert_eq!(events[1].event, Some("content_block_start".to_string()));
    assert_eq!(events[2].event, Some("content_block_delta".to_string()));
    assert_eq!(events[3].event, Some("content_block_delta".to_string()));
    assert_eq!(events[4].event, Some("content_block_stop".to_string()));
    assert_eq!(events[5].event, Some("message_delta".to_string()));
    assert_eq!(events[6].event, Some("message_stop".to_string()));

    // Verify data is valid JSON
    for event in &events {
        assert!(
            serde_json::from_str::<serde_json::Value>(&event.data).is_ok(),
            "Event data should be valid JSON"
        );
    }
}

/// Test extracting text from streaming response
#[tokio::test]
async fn test_extract_text_from_anthropic_stream() {
    let sse_data = r#"event: message_start
data: {"type":"message_start"}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"text":" "}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"text":"world!"}}

event: message_stop
data: {"type":"message_stop"}
"#;

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let text = extract_text_from_stream(byte_stream).await.unwrap();

    assert_eq!(text, "Hello world!", "Should extract complete text");
}

/// Test handling [DONE] termination signal
#[tokio::test]
async fn test_done_signal_stops_stream() {
    let sse_data = r#"event: content_block_delta
data: {"type":"content_block_delta","delta":{"text":"First"}}

data: [DONE]

event: content_block_delta
data: {"type":"content_block_delta","delta":{"text":"Never seen"}}
"#;

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    // Should stop at [DONE], not parse further events
    assert_eq!(events.len(), 1, "Should stop at [DONE]");
    assert_eq!(
        events[0].event,
        Some("content_block_delta".to_string()),
        "Should only get first event"
    );
}

/// Test parsing chunked SSE stream (multiple bytes chunks)
#[tokio::test]
async fn test_chunked_stream_parsing() {
    let chunks = vec![
        Bytes::from("event: message_start\n"),
        Bytes::from("data: {\"type\":\"message_start\"}\n\n"),
        Bytes::from("event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hi\"}}\n\n"),
    ];

    let byte_stream = Box::pin(stream::iter(chunks.into_iter().map(Ok)));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, Some("message_start".to_string()));
    assert_eq!(events[1].event, Some("content_block_delta".to_string()));
}

/// Test handling multi-line data fields
#[tokio::test]
async fn test_multiline_data_field() {
    let sse_data = "event: test\ndata: first line\ndata: second line\ndata: third line\n\n";

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, Some("test".to_string()));
    assert_eq!(
        events[0].data, "first line\nsecond line\nthird line",
        "Should concatenate multi-line data"
    );
}

/// Test handling SSE comments (lines starting with ':')
#[tokio::test]
async fn test_sse_comments_are_ignored() {
    let sse_data = r#": This is a comment
event: test
: Another comment
data: {"type":"test"}

: Final comment
"#;

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 1, "Comments should be filtered out");
    assert_eq!(events[0].event, Some("test".to_string()));
}

/// Test handling CRLF line endings (Windows style)
#[tokio::test]
async fn test_crlf_line_endings() {
    let sse_data = "event: test\r\ndata: {\"type\":\"test\"}\r\n\r\n";

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, Some("test".to_string()));
    assert_eq!(events[0].data, r#"{"type":"test"}"#);
}

/// Test parsing real-world thinking mode response
#[tokio::test]
async fn test_thinking_mode_response() {
    let sse_data = r#"event: message_start
data: {"type":"message_start"}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"Let me think..."}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: content_block_start
data: {"type":"content_block_start","index":1,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":1,"delta":{"type":"text_delta","text":"The answer is"}}

event: message_stop
data: {"type":"message_stop"}
"#;

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 7, "Should parse all thinking mode events");

    // Verify thinking block
    let thinking_delta = &events[2];
    let json: serde_json::Value = serde_json::from_str(&thinking_delta.data).unwrap();
    assert_eq!(
        json["delta"]["type"].as_str(),
        Some("thinking_delta"),
        "Should have thinking delta"
    );
}

/// Test handling id and retry fields
#[tokio::test]
async fn test_id_and_retry_fields() {
    let sse_data = "id: 12345\nevent: test\ndata: data\nretry: 5000\n\n";

    let bytes = Bytes::from(sse_data);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, Some("12345".to_string()));
    assert_eq!(events[0].retry, Some(5000));
}

/// Test error handling with invalid UTF-8
#[tokio::test]
async fn test_invalid_utf8_handling() {
    // Create invalid UTF-8 bytes
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let bytes = Bytes::from(invalid_utf8);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let mut stream = SseParser::parse_stream(byte_stream);

    // Should not panic, but also should not yield events
    let mut count = 0;
    while let Some(_) = stream.next().await {
        count += 1;
    }

    // Invalid UTF-8 is skipped, so no events
    assert_eq!(count, 0, "Invalid UTF-8 should be skipped");
}

/// Test stream error propagation
#[tokio::test]
async fn test_stream_error_propagation() {
    // Create a stream that yields an error by using a mock error
    // We can't easily create a reqwest::Error directly, so we'll use a URL error
    let error_result: Result<Bytes, reqwest::Error> = Err(reqwest::get(
        "http://invalid-url-that-does-not-exist-12345.test",
    )
    .await
    .unwrap_err());

    let error_stream = stream::once(async move { error_result });
    let byte_stream = Box::pin(error_stream);
    let mut event_stream = SseParser::parse_stream(byte_stream);

    // Should yield the error
    match event_stream.next().await {
        Some(Err(e)) => {
            assert!(e.to_string().contains("Stream error"));
        }
        _ => panic!("Expected error to be propagated"),
    }
}

/// Test large streaming response (performance test)
#[tokio::test]
async fn test_large_stream_performance() {
    // Create a large SSE stream (1000 events)
    let mut large_sse = String::new();
    for i in 0..1000 {
        large_sse.push_str(&format!(
            "event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n\n",
            i
        ));
    }

    let bytes = Bytes::from(large_sse);
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 1000, "Should parse all 1000 events");
}

/// Test empty stream handling
#[tokio::test]
async fn test_empty_stream() {
    let bytes = Bytes::new();
    let byte_stream = Box::pin(stream::once(async move { Ok(bytes) }));

    let events = collect_sse_events(byte_stream).await.unwrap();

    assert_eq!(events.len(), 0, "Empty stream should yield no events");
}
