# CCO Terminal UI (TUI) Cockpit

The CCO TUI (`cco tui`) is an interactive terminal dashboard for monitoring and managing your Claude Code sessions and agents. It runs independently of any active session and provides global visibility across all projects.

## Quick Start

```bash
# Start the TUI
cco tui

# Key controls
Tab              Cycle to next page
1-4              Jump directly to a page (Live=1, Spend=2, Cache=3, Delegation=4)
t                Change time range
p                Open project scope picker
↑/↓              Navigate selections
r                Refresh data
?                Show keybindings help overlay
q / Esc          Quit
```

## Pages

The TUI displays four main pages. Navigate between them with `Tab` or jump directly with number keys `1`–`4`.

### 1. Live

Real-time view of orchestration in flight.

**Displays:**
- **Running Now**: In-flight subagents with model, task description, elapsed time, and stuck indicator
- **Active Sessions**: Fleet roll-up showing concurrent session counts by project
- **Event Feed**: Filtered event stream (no session_started/ended noise) for task completions, errors, and control events
- **Task Counts**: Honest scoped counts — pending, running, completed, failed, and blocked tasks for the launch directory

**Live updates:**
The TUI streams events from the daemon via Server-Sent Events (SSE) at `/api/events/stream`. When the SSE connection is unavailable (network issue, daemon overloaded), the TUI falls back to polling every 1000ms (configurable via `tui_refresh_ms` in config). The status bar shows a spinning indicator when live.

### 2. Spend

Overview of your spending and efficiency metrics.

**Displays:**
- Total spend and sparkline trend (last 7 days)
- Cache hit rate (percentage of cache reads vs. writes)
- Breakdown by model tier: Opus, Sonnet, Haiku (with token counts and percentages)
- Token totals (input, output, cache-write, cache-read)
- Daily budget burn-down gauge (if `daily_budget_usd` is configured)
- Total conversations and message count

**Key controls:**
- `t` — Change the time range (Hour → Today → Week → Month → All Time)
- `p` — Filter by project (global by default; see **Project Scope Picker**)

### 3. Cache

Cache efficiency and prompt-caching analysis.

**Displays:**
- Overall cache statistics: Total cache-write tokens, cache-read tokens, hit rate with active range and scope labels
- Per-model breakdown: Cache metrics for each model (Opus, Sonnet, Haiku)
- Hit rate gauge: Visual indicator of cache efficiency (read / (read + write))
- Estimated savings: Dollar value saved by cache hits (calculated from your pricing tier)

**Context:**
Prompt caching stores expensive context (system prompts, large documents) on Anthropic's servers. First use writes the cache (costs 1.25x the input tokens); subsequent reads use the cache (0.1x cost). High hit rates indicate effective caching.

### 4. Delegation

Multi-agent delegation and orchestration metrics.

**Displays:**
- Real parent/child linkage: Actual task DAG relationships showing which agents spawned which subagents; parent row now displays the real orchestrator task (falls back to heuristic when linkage unavailable)
- Delegation nudge activity: How often the nudge suggestion has been triggered
- Task delegation breakdown: Work pushed to each agent tier (Opus, Sonnet, Haiku)
- Subagent metrics: Number of subagents spawned, total tokens, cost per tier
- Cost by source: Breakdown of costs by source (JSONL transcripts, push API, OTLP metrics)
- Effort distribution: Tokens and cost by model tier and orchestration stage
- DAG status: Pending, active, and completed task graphs

This page reflects the **delegation-nudge system** — a PostToolUse hook that suggests you use lower-cost agents (Sonnet, Haiku) for implementation rather than doing it inline with Opus.

## Status Bar

The persistent status bar at the bottom of the TUI displays:

```
cco spend: $12.34 cache: 87% scope: all range: Last Hour ⟳
```

- **spend**: Total spend in the selected time range (yellow text)
- **cache**: Overall cache hit rate as percentage (green text)
- **scope**: Project filter ("all" for global, or specific project name)
- **range**: Time range for metrics (Last Hour / Today / This Week / etc.)
- **⟳**: Live indicator (green when SSE stream is connected)

## Project Scope Picker

By default, the TUI shows metrics for all projects combined. Press `p` to open the scope picker:

```
Select Project Scope
  [1] All projects
  [2] myproject
  [3] anotherproject
```

Choose a project to filter all metrics (spend, cache, agents, events) to that project only. The status bar updates to show the active scope. Press `q` to close the picker without changing scope.

## Configuration

TUI behavior is controlled by settings in `~/.cco/config.toml`:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `tui_refresh_ms` | u64 | 1000 | Polling interval (ms) when SSE stream is unavailable; clamped to minimum 250ms |
| `daily_budget_usd` | float | — | When set, TUI displays a budget burn-down gauge on the Spend page |

Example:

```toml
tui_refresh_ms = 500           # Poll every 500ms (fast refresh)
daily_budget_usd = 25.00       # Show budget gauge with $25 daily limit
```

## How It Works

### Daemon Communication

The TUI connects to the daemon via a Unix domain socket at `~/.cco/daemon.sock`. All data (stats, agents, events) is fetched from the daemon; the TUI does not read JSONL files directly.

**Key endpoints:**
- `GET /api/stats?time_range={hour,today,week,month,all}` — Metrics and cost data
- `GET /api/control/agents` — Agent list and task counts
- `GET /api/events/stream` (SSE) — Live event stream
- `GET /api/events` — Event history (fallback when SSE unavailable)
- `GET /api/tasks` — Task DAG status

### Directory Independence

The TUI is **directory-independent** — it shows metrics across all projects regardless of which directory you're in. Use the project scope picker (`p`) to narrow the view to a single project if needed. The daemon reconciles metrics in the background on its own schedule (file watcher), so no per-project bootstrap is needed at TUI startup.

### Responsive Design

The TUI adapts to terminal size:

- **Full layout** (width ≥ 80, height ≥ 10): Multi-column dashboard with all panes visible
- **Narrow layout** (width < 80, height ≥ 10): Single-column stacked view for small terminals
- **Too small** (width < 40 or height < 10): Displays a resize hint

If your terminal is too small, expand it or split your pane.

## Common Workflows

### Monitor Spend in Real Time

```bash
# In one terminal, run Claude Code
cco

# In another terminal, watch the TUI
cco tui

# Switch to the Spend page (press 2)
# Watch the spend and cache metrics update live
```

### Watch Running Agents in Real Time

```bash
cco tui
# Press 1 to go to Live
# See in-flight subagents, active sessions, and event feed
# Task counts show honest scope for the launch directory
```

### Check Cache Efficiency

```bash
cco tui
# Press 3 for Cache
# See hit rate and per-model cache stats
# High hit rate (>50%) indicates effective caching
```

### Review Delegation Metrics

```bash
cco tui
# Press 4 for Delegation
# See parent/child task linkage and delegation nudge activity
# Review cost breakdown by agent tier
```

### Filter by Project

```bash
cco tui
# Press p to open project picker
# Select a project (e.g., "myapp")
# All metrics now show only data for that project
# Status bar updates: "scope: myapp"
```

### Change Time Range

```bash
cco tui
# Press t to cycle: Hour → Today → Week → Month → All Time
# Dashboard updates with metrics for the new range
```

## Troubleshooting

### TUI Shows "Connection Lost"

**Symptom:** Status bar shows "⚠️ Connection lost (X/10 failures)".

**Fix:** Start the daemon:
```bash
cco daemon start
```

After 10 consecutive failures, the TUI exits with an error message. Restart the daemon and try again.

### Data Not Updating

**Symptom:** Metrics are stale or not changing.

**Check 1:** Verify daemon is running:
```bash
cco daemon status
```

**Check 2:** Verify you have recent Claude Code sessions:
```bash
ls -la ~/.claude/projects/
```

If the directory is empty or has no recent files, run a Claude Code session first.

**Check 3:** Increase refresh rate (if using polling fallback):

Edit `~/.cco/config.toml`:
```toml
tui_refresh_ms = 250     # Faster polling
```

Restart the TUI.

### "Daemon socket not found"

**Symptom:** TUI exits immediately with "Unix socket not found".

**Fix:** Install the daemon service:
```bash
cco daemon install
cco daemon start
```

### Terminal Too Small

**Symptom:** TUI shows "Terminal too small" message.

**Fix:** Expand your terminal to at least 40 columns × 10 rows.

```bash
# Example: macOS Terminal
# Menu > View > Window Sizes > 120x30
```

## Keyboard Reference

| Key | Action |
|-----|--------|
| `Tab` | Cycle to next page |
| `1`–`4` | Jump to Live, Spend, Cache, or Delegation page |
| `t` | Cycle time range (Hour → Today → Week → Month → All Time) |
| `p` | Open project scope picker |
| `↑` / `↓` | Navigate list selections (up/down) |
| `r` | Refresh data immediately |
| `?` | Show keybindings help overlay |
| `q` / `Esc` | Quit |

## Related Documentation

- [Integrations Guide](docs/integrations.md) — RTK, Headroom, lean-ctx, and tool integration
- [Cost Metrics](docs/cost-metrics.md) — How metrics are calculated and stored
- [MCP and Control Plane](docs/mcp-and-control-plane.md) — Daemon API and MCP server details
