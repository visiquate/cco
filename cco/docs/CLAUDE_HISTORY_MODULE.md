# Claude History Module

The `claude_history` module provides functionality to parse Claude conversation history JSONL files and extract comprehensive metrics including token usage and costs.

## Overview

Claude Code stores conversation history in `~/.claude/projects/{project-name}/` as JSONL (JSON Lines) files. Each conversation is stored in a separate file with a UUID filename.

This module:
- Parses all JSONL files in a project directory
- Extracts token usage from assistant messages
- Calculates accurate costs based on current Claude API pricing
- Aggregates metrics by model
- Handles cache tokens (creation and reads) correctly

## Usage

### Basic Example

```rust
use cco::claude_history::load_claude_project_metrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let project_path = "/Users/brent/.claude/projects/-Users-brent-git-myproject";
    let metrics = load_claude_project_metrics(project_path).await?;

    println!("Total cost: ${:.2}", metrics.total_cost);
    println!("Total messages: {}", metrics.messages_count);
    println!("Conversations: {}", metrics.conversations_count);

    Ok(())
}
```

### Running the Example

```bash
cargo run --example claude_history_example
```

## Data Structures

### ClaudeMetrics

Main aggregated metrics structure:

```rust
pub struct ClaudeMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost: f64,
    pub messages_count: u64,
    pub conversations_count: u64,
    pub model_breakdown: HashMap<String, ModelBreakdown>,
    pub last_updated: DateTime<Utc>,
}
```

### ModelBreakdown

Per-model metrics:

```rust
pub struct ModelBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost: f64,
    pub message_count: u64,
}
```

## Model Pricing (November 2025)

The module uses the following pricing rates:

| Model | Input (per M tokens) | Output (per M tokens) |
|-------|---------------------|----------------------|
| claude-opus-4 | $15.00 | $45.00 |
| claude-sonnet-4-5 | $3.00 | $15.00 |
| claude-sonnet-3.5 | $3.00 | $15.00 |
| claude-haiku-4-5 | $0.80 | $4.00 |
| claude-3-5-haiku | $0.80 | $4.00 |

### Cache Token Pricing

- **Cache Creation**: Input price × 1.25 (25% more expensive)
- **Cache Reads**: Input price × 0.1 (90% cheaper)

## JSONL File Format

Each JSONL file contains one JSON object per line. The module processes lines with `type: "assistant"`:

```json
{
  "type": "assistant",
  "message": {
    "model": "claude-sonnet-4-5-20250929",
    "usage": {
      "input_tokens": 1000,
      "output_tokens": 500,
      "cache_creation_input_tokens": 5000,
      "cache_read_input_tokens": 10000
    }
  }
}
```

Other line types (like `"type": "summary"` or `"type": "file-history-snapshot"`) are safely ignored.

## Model Name Normalization

The module normalizes model names by removing date suffixes:

- `claude-sonnet-4-5-20250929` → `claude-sonnet-4-5`
- `claude-opus-4-20250514` → `claude-opus-4`
- `claude-3-5-haiku-20250403` → `claude-3-5-haiku`

## Error Handling

The module is designed to be fault-tolerant:

- **Malformed JSON**: Lines that fail to parse are skipped silently
- **Missing fields**: Messages without usage data are ignored
- **Nonexistent directories**: Returns empty metrics (not an error)
- **Unreadable files**: Logged as warnings, processing continues
- **Unknown models**: Uses default Sonnet pricing with a warning

## Testing

The module includes comprehensive unit tests:

```bash
# Run all claude_history tests
cargo test --lib claude_history

# Run specific test
cargo test --lib claude_history::tests::test_parse_sample_jsonl

# Run with output
cargo test --lib -- --nocapture claude_history
```

### Test Coverage

- ✅ Model name normalization
- ✅ Model pricing lookup
- ✅ Cost calculation accuracy
- ✅ JSONL parsing (valid and invalid)
- ✅ Empty project directories
- ✅ Nonexistent directories
- ✅ Model aggregation
- ✅ Graceful error handling
- ✅ Cache token handling

## Integration with CCO Dashboard

The module is designed to integrate with the CCO dashboard's metrics endpoint:

```rust
// In server.rs
use cco::claude_history::load_claude_project_metrics;

async fn metrics_handler() -> Json<Value> {
    let project_path = dirs::home_dir()
        .unwrap()
        .join(".claude/projects/-Users-brent-git-cc-orchestra");

    let metrics = load_claude_project_metrics(project_path.to_str().unwrap())
        .await
        .unwrap_or_default();

    Json(json!({
        "total_cost": metrics.total_cost,
        "total_messages": metrics.messages_count,
        "model_breakdown": metrics.model_breakdown,
    }))
}
```

## Performance Characteristics

- **Streaming**: Reads JSONL files line-by-line to minimize memory usage
- **Async I/O**: Uses Tokio's async file I/O for better performance
- **No buffering**: Processes messages incrementally
- **Graceful degradation**: Continues processing on individual file errors

## Example Output

Running the example on the cc-orchestra project:

```
=== Claude Project Metrics ===

Total Conversations: 240
Total Messages:      9285

Token Usage:
  Input tokens:              2796874
  Output tokens:             7203184
  Cache creation:           71567399
  Cache reads:             600845856

Total Cost: $426.7709

=== Breakdown by Model ===

claude-sonnet-4-5:
  Messages:           5216
  Input tokens:        2250806
  Output tokens:       4494627
  Cache creation:     48607929
  Cache reads:       362152729
  Cost:           $   365.0974

claude-haiku-4-5:
  Messages:           3784
  Input tokens:         544973
  Output tokens:       2423979
  Cache creation:     21900278
  Cache reads:       228229758
  Cost:           $    50.2906
```

## Future Enhancements

- [ ] Add time-based filtering (metrics for last 7 days, 30 days, etc.)
- [ ] Support for conversation-level metrics (not just project-wide)
- [ ] Export metrics to JSON/CSV
- [ ] Integration with Knowledge Manager for cost tracking
- [ ] Real-time monitoring of new conversations
- [ ] Cost alerts and budgeting features

## Related Files

- **Module**: `/Users/brent/git/cc-orchestra/cco/src/claude_history.rs`
- **Example**: `/Users/brent/git/cc-orchestra/cco/examples/claude_history_example.rs`
- **Tests**: Inline in module (13 tests)
- **Integration**: Ready for `/Users/brent/git/cc-orchestra/cco/src/server.rs`
