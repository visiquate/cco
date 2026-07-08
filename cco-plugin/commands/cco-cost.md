# CCO Cost Dashboard

## Trigger
`/cco-cost`

Display a comprehensive cost dashboard with spending summary, trends, and cost breakdown by agent tier.

## Purpose
This command shows detailed cost analysis for Claude Code API usage:
- Total spending and period summary (today, week, month)
- Cost trends and burn rate
- Agent-level cost breakdown by tier
- Cache hit rates and savings
- Comparison to budget thresholds

## Usage

```
You: /cco-cost

💰 Cost Dashboard
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Period: Today

Current Spend: $2.47
Week Total: $12.34
Month Total: $45.23
Budget Status: ✅ Within limits

Cost Breakdown by Tier:
  🟦 Opus (Chief Architect)     : $0.47 (19%)
  🟨 Sonnet (Managers)          : $1.23 (50%)
  🟩 Haiku (Specialists)        : $0.77 (31%)

Cache Performance:
  Hit Rate: 42.3%
  Savings: $1.89
  Efficiency: ⬆ 8% vs yesterday
```

## When to Use

Use `/cco-cost` when you want to:
- Check current spending and trends
- See cost breakdown by agent tier
- Review cache effectiveness
- Monitor budget consumption
- Plan resource allocation

## Implementation

To use this command, ask Claude to:
1. Run `cco cost dashboard` to get the cost dashboard
2. Present the summary with spending by period and agent tier
3. Show cache hit rates and savings (if available in the output)

## Notes

- Cost data aggregates all Claude Code API calls
- Daily and weekly budgets can be configured in `~/.cco/config.toml`
- Cache hits significantly reduce costs (5% input cost vs normal 25%)
- Trends show comparison to previous periods for context
