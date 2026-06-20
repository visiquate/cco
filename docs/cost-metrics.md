# Cost and Metrics Analysis

CCO tracks Claude API usage and costs by ingesting conversation history from Claude Code and analyzing token consumption. This guide covers cost tracking, pricing, budgeting, and cache efficiency reporting.

## How CCO Tracks Cost

### Data Source

CCO ingests Claude Code conversation history from `~/.claude/projects/*.jsonl` and stores it in a local DuckDB database at `~/.cco/claude_history.duckdb`. Each message in the conversation history is parsed to extract:

- **Input tokens**: text sent to Claude
- **Output tokens**: response text from Claude
- **Cache write tokens**: data written to the ephemeral cache (5-minute and 1-hour variants)
- **Cache read tokens**: data read from the cache on subsequent calls

### Token Fields

Each API call in the database tracks four cache-related token types:

| Field | Description | Cost Factor |
|-------|-------------|-------------|
| `input_tokens` | Regular input (context + prompt) | 1x input price |
| `output_tokens` | Model output | output price |
| `cache_write_5m_tokens` | Tokens written to 5-minute ephemeral cache | 1.25x input price |
| `cache_write_1h_tokens` | Tokens written to 1-hour ephemeral cache | 2x input price |
| `cache_read_tokens` | Tokens read from cache (hit) | 0.1x input price |

The database stores both granular data (per-model, per-project) and aggregate metrics for efficient querying.

## Pricing Model and Tiers

CCO supports five model tiers with distinct pricing (as of February 2026):

| Model | Input $/MTok | Output $/MTok | Tier Use |
|-------|--------------|---------------|----------|
| **Claude Fable 5 / Mythos 5** | $10.00 | $50.00 | Premium / experimental (highest cost) |
| **Claude Opus 4.5** | $5.00 | $25.00 | Complex reasoning, architecture, code review |
| **Claude Sonnet 4** | $3.00 | $15.00 | General-purpose work, managers, reviewers |
| **Claude Haiku 4.5** | $1.00 | $5.00 | Language specialists, utilities, quick tasks |
| **Claude Haiku 3.5** | $0.80 | $4.00 | Legacy; generally replaced by Haiku 4.5 |

### Cache Pricing Convention

Prompt caching applies a consistent discount structure across all models:

- **5-minute ephemeral cache write**: 1.25x input price (write-once overhead)
- **1-hour ephemeral cache write**: 2x input price (longer retention cost)
- **Cache read/hit**: 0.1x input price (90% discount vs. regular input)

Example: A Sonnet call with 1M input tokens normally costs $3.00. If those tokens are cached and hit on the next call, the read costs $0.30 (0.1x the $3.00 input rate), saving $2.70.

### Refreshable Pricing

Pricing is not hard-coded. The system loads pricing from multiple sources in priority order:

1. **User override** (`~/.cco/pricing.json`) — runtime file for ad-hoc price updates
2. **Embedded overrides** (`pricing_overrides.json` bundled in the binary)
3. **Hard-coded constants** (fallback; should never be used in practice)

This allows operators to update pricing without rebuilding CCO. To override prices:

Create `~/.cco/pricing.json`:

```json
{
  "opus-4-5": {
    "input": 5.0,
    "output": 25.0,
    "cache_write_5m": 6.25,
    "cache_write_1h": 10.0,
    "cache_read": 0.50
  },
  "sonnet": {
    "input": 3.0,
    "output": 15.0,
    "cache_write_5m": 3.75,
    "cache_write_1h": 6.0,
    "cache_read": 0.30
  },
  "haiku-4-5": {
    "input": 1.0,
    "output": 5.0,
    "cache_write_5m": 1.25,
    "cache_write_1h": 2.0,
    "cache_read": 0.10
  },
  "haiku-3-5": {
    "input": 0.80,
    "output": 4.0,
    "cache_write_5m": 1.0,
    "cache_write_1h": 1.60,
    "cache_read": 0.08
  },
  "fable": {
    "input": 10.0,
    "output": 50.0,
    "cache_write_5m": 12.5,
    "cache_write_1h": 20.0,
    "cache_read": 1.0
  }
}
```

All five keys are required. Restart the daemon after editing.

## Commands and Dashboards

### Cost Dashboard

Display an overview of spending, trends, and per-model breakdown.

```bash
cco cost dashboard [--period PERIOD] [--format FORMAT]
```

**Options:**

- `--period PERIOD` — `today` (default), `week`, `month`, or `all`
- `--format FORMAT` — `human` (default) or `json`

**Example output:**

```
╔════════════════════════════════════════════════════════════════╗
║              CLAUDE CODE COST DASHBOARD (week)              ║
╚════════════════════════════════════════════════════════════════╝

📊 Total Spend (week): $42.67
   Conversations: 18  | Messages: 523
   Daily Average: $6.10
   Cache Savings: $0.0825

💼 Breakdown by Model Tier:
   [████████████░░░░░░] claude-opus-4-5          $28.45 (66.7%)
   [███████░░░░░░░░░░░] claude-sonnet-4         $10.22 (23.9%)
   [████░░░░░░░░░░░░░░] claude-haiku-4-5         $4.00 ( 9.4%)

📁 Top 3 Projects:
   • cc-orchestra - $31.50
   • my-ai-tool - $8.30
   • research-spike - $2.87

📈 Token Usage:
   Input:  2,145,000 tokens ($6.82)
   Output:   523,000 tokens ($3.15)
   Cache Write: 145,000 tokens ($0.25)
   Cache Read:  89,000 tokens ($0.01)
```

### Session Costs

Show detailed cost breakdown by project or for recent sessions.

```bash
cco cost session [--id PROJECT_ID | --recent N] [--format FORMAT]
```

**Example:**

```bash
cco cost session --recent 5
```

Shows the top 5 projects by spend with message counts and conversation counts.

To see details for a specific project:

```bash
cco cost session --id cc-orchestra
```

### Per-Model Breakdown

Show cost distribution across all models used in a period.

```bash
cco cost agents [--period PERIOD] [--format FORMAT]
```

**Example:**

```bash
╔═══════════════════════════════════════════════════════════════════════════════╗
║           COST BREAKDOWN BY MODEL/AGENT (week)                          ║
╚═══════════════════════════════════════════════════════════════════════════════╝

Model                   Input Cost   Output Cost   Cache Costs  Total Cost % Total
─────────────────────────────────────────────────────────────────────────────────
claude-opus-4-5         $18.5000     $8.1500       $0.5000      $27.1500   63.5%
claude-sonnet-4         $7.2000      $2.4000       $0.1200      $9.7200    22.8%
claude-haiku-4-5        $3.1000      $0.5000       $0.0200      $3.6200    8.5%
claude-haiku-3-5        $0.8000      $0.1500       $0.0100      $0.9600    2.2%
─────────────────────────────────────────────────────────────────────────────────
TOTAL                   $29.6000     $11.2000      $0.6500      $41.4500   100.0%
```

### Export Cost Data

Export metrics to CSV or JSON for external analysis.

```bash
cco cost export [--output FILE] [--format csv|json] [--period PERIOD]
```

**Example:**

```bash
cco cost export --output costs.csv --format csv --period month
```

CSV columns: `date`, `project`, `model`, `input_cost`, `output_cost`, `cache_write_cost`, `cache_read_cost`, `total_cost`.

### Cache Efficiency Report

Analyze cache usage patterns: hit rates, savings, and detection of wasted cache writes ("silent busters").

```bash
cco cost cache [--period PERIOD] [--format FORMAT]
```

**Key metrics:**

- **Hit rate**: percentage of cache reads relative to total cache writes
- **Estimated savings**: difference between normal input cost vs. cache-read cost
- **Silent-buster threads**: conversations that wrote cache but never read from it (likely a cache-control placement issue)

**Example output:**

```
+======================================================================+
| CACHE EFFICIENCY REPORT (WEEK) |
+======================================================================+

  Total write tokens : 145,000 (5m: 12,000, 1h: 133,000)
  Total read tokens  : 89,000
  Hit rate           : 61.4%
  Estimated savings  : $0.0785
  Silent-buster threads: 2 (threads with writes but zero reads)

  Model                      Write-5m      Write-1h      Read         Hit%    Saved $
  ────────────────────────────────────────────────────────────────────────────────
  claude-opus-4-5              10,000       100,000    65,000       64.6%    $0.0325
  claude-sonnet-4               2,000        28,000    22,000       78.6%    $0.0198
  claude-haiku-4-5                 500         5,000     2,000       40.0%    $0.0062
  ────────────────────────────────────────────────────────────────────────────────
  TOTAL                        12,500       133,000    89,000       61.4%    $0.0785

  [!] 2 thread(s) wrote cache entries that were never read.
      Check that cache_control blocks are positioned consistently
      and that the same prefix is reused across turns in each thread.
```

### Budget Gate (CI Integration)

Check if spending has exceeded a budget threshold. Useful for CI pipelines to fail fast if costs spike.

```bash
cco cost gate [--max-usd AMOUNT] [--period PERIOD] [--format FORMAT]
```

**Threshold resolution** (in order of priority):

1. `--max-usd` argument
2. Config file: `daily_budget_usd` (for "today") or `weekly_budget_usd` (for "week")
3. `CCO_DAILY_BUDGET_USD` environment variable
4. No threshold (always returns "OK")

**Exit behavior:**

- Human mode: exits with code 1 if exceeded, 0 if OK
- JSON mode: always exits with 0 but indicates status in the JSON output

**Example:**

```bash
cco cost gate --period week --max-usd 50.0
```

Output if under budget:

```
Period       Spend        Threshold    Status
──────────────────────────────────────────────
week         $42.67       $50.00       OK
```

Output if over budget:

```
Period       Spend        Threshold    Status
──────────────────────────────────────────────
week         $65.32       $50.00       EXCEEDED

# Exit code: 1
```

In JSON mode:

```bash
cco cost gate --period week --max-usd 50.0 --format json
```

```json
{
  "cost": 65.32,
  "threshold": 50.0,
  "exceeded": true,
  "period": "week"
}
```

## Budget Configuration

Set daily or weekly spending limits in `~/.cco/config.toml`:

```toml
# Daily budget — checked by `cco cost gate` and the daemon's budget-gate hook
daily_budget_usd = 10.0

# Weekly budget — used when period == "week"
weekly_budget_usd = 50.0
```

### Budget Gate Hook

When a daily budget is set, the daemon emits a soft (non-blocking) reminder via the budget-gate PostToolUse hook when today's spend approaches the limit. This gives you a chance to review costs without interrupting your work.

## RTK (Rust Token Killer) Savings

If `rtk` is installed and on PATH, the cost dashboard automatically includes RTK savings — how many tokens RTK's aggressive bash output trimming has saved your project.

**Example:**

```
   RTK Savings (lifetime):
   Tokens saved: 2,345,678 across 456 commands (78.3% avg reduction)
   Implied savings: ~$2.34 (@ $1.00/MTok blended input)
```

The "blended input" rate is the weighted average of all models used in the period. If no usage data is available, it falls back to the Sonnet rate ($3.00/MTok) and is marked as an estimate.

RTK savings are lifetime (not period-scoped) because the trimming happens at the shell level and is not tied to Claude Code's internal periods.

## TUI Views

The interactive TUI (`cco tui`) includes dedicated cost and cache analysis panels:

- **Cost panel**: shows daily-average spend, model breakdown, and top projects
- **Cache panel**: displays hit rate, savings, and silent-buster detection
- **Delegation panel**: (if applicable) tracks delegation nudge metrics and agent coverage

Navigate with arrow keys or `Tab`; press `q` to quit.

## Integration with Telemetry

Cost and metrics data is stored locally in `~/.cco/claude_history.duckdb`. By default, CCO sends only aggregate token counts to the VisiQuate ingest endpoint (no prompt/response text). See [Telemetry](telemetry.md) for privacy controls and how to opt in to transcript uploads.

## Troubleshooting

### No cost data available

**Cause**: No Claude Code conversations yet, or the DuckDB file hasn't been created.

**Fix**: Run a conversation in Claude Code first. The daemon will ingest and index the data automatically.

### Pricing doesn't match expectations

**Check**:

1. Is the correct model being used? Run `cco cost agents` to verify.
2. Are cache tokens being counted? Run `cco cost cache` to see cache hit rates.
3. Did you override pricing? Check `~/.cco/pricing.json`.

**Rebuild from scratch**: Delete `~/.cco/claude_history.duckdb` and restart the daemon to re-ingest all conversations.

### Cache hit rate is 0%

**Cause**: No calls are reusing the same `cache_control` blocks, or the blocks are positioned inconsistently.

**Fix**: Ensure that your system prompts or static preambles are wrapped in `cache_control` and reused across consecutive calls. See the "Silent-buster threads" section of the cache report.

### RTK savings not showing

**Cause**: `rtk` is not installed or not on PATH.

**Fix**: Install RTK (`cargo install rtk`) or skip it (RTK integration is optional).

## See Also

- [Telemetry](telemetry.md) — privacy controls, aggregate metrics vs. transcript uploads
- [Configuration](../CLAUDE.md) — how to set `daily_budget_usd` and `weekly_budget_usd`
