# CCO Cache Status

## Trigger
`/cco-cache`

Show cache hit-rate and cost savings from prompt caching.

## Purpose
This command displays detailed cache performance metrics:
- Overall cache hit rate percentage
- Total tokens saved by caching
- Cost reduction from cache usage
- Cache-miss analysis and optimization suggestions
- Cumulative savings over time

## Usage

```
You: /cco-cache

📊 Cache Performance Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Session Cache (1-hour ephemeral):

Hit Rate: 42.3%
Hits: 128 / 303 requests
Tokens Saved: 18,245
Cost Savings: $0.45 (vs $0.87 without cache)
Efficiency: 51% reduction

Cache Breakdown by Agent Type:
  Opus   : 38% hit rate (0.52 avg savings/call)
  Sonnet : 44% hit rate (0.34 avg savings/call)
  Haiku  : 41% hit rate (0.15 avg savings/call)

Cache Strategy:
  ✅ Agent prompts cached across sessions
  ✅ Agent definitions cached (5m blocks)
  ✅ Knowledge graph cached (1h ephemeral)
  ⚠️ Large files not cached (RTK disabled)
```

## When to Use

Use `/cco-cache` when you want to:
- Monitor cache effectiveness
- See how much caching is saving
- Understand cache hit patterns
- Optimize for better cache utilization
- Verify RTK (Rust Token Killer) status

## Implementation

To use this command, ask Claude to:
1. Run `cco cost cache` to get cache performance metrics
2. Display hit rate, savings, and breakdown by agent
3. Note RTK status (whether enabled/disabled)

## Notes

- Cache is 1-hour ephemeral per session (automatic with Claude Code SDK)
- Cache-write cost is 2x input cost (1h blocks)
- Cache-read cost is 10% of normal input cost
- RTK (Rust Token Killer) can be enabled to compress Bash outputs and reduce token usage
- Higher hit rates indicate better prompt consistency across sessions
