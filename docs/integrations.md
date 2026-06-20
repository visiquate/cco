# CCO Integrations

This guide covers the external tools and services that integrate with Claude Code Orchestra (CCO), and how to configure them.

## Quick Links

- [RTK (Rust Token Killer)](#rtk-rust-token-killer) — Bash output compression
- [Compression Modes](#compression-modes) — RTK vs lean-ctx vs none
- [Headroom Proxy](#headroom-proxy) — API-boundary compression and caching
- [lean-ctx](#lean-ctx) — Semantic context compression via MCP
- [Ponytail](#ponytail) — Anti-bloat prompt rules
- [Statusline](#statusline) — Session status display
- [Remote Control](#remote-control) — Mobile and web control
- [Codex CLI](#codex-cli) — External code review
- [Health Check (`cco doctor`)](#health-check-cco-doctor) — System diagnosis
- [Cost Tracking](#cost-tracking) — Token savings with RTK
- [Editor Surfaces](#editor-surfaces) — VS Code, JetBrains, and more
- [Configuration Reference](#configuration-reference) — All config keys

---

## RTK (Rust Token Killer)

RTK compresses Bash command output 60–90% before it reaches the LLM context. This dramatically reduces token usage on dev-heavy tasks.

### How it Works

When `rtk` is on PATH and `rtk_enabled != false` in your config, CCO registers RTK's `rtk hook claude` hook as a **PreToolUse** hook in the generated Claude Code settings. RTK intercepts bash commands, rewrites their output to strip noise (ANSI colors, repeated lines, verbose headers), and passes the trimmed output upstream to Claude.

### Installation

Choose any method:

**Homebrew (macOS & Linux):**
```bash
brew install rtk
```

**Curl installer (macOS & Linux):**
```bash
curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh
```

**Auto-install via `cco doctor --fix`:**
```bash
cco doctor --fix
```
This will detect Homebrew and use it if available, or fall back to the curl installer.

### Configuration

**Enable RTK (default):**

RTK is **enabled by default** when `rtk` is on PATH. No config needed.

**Disable RTK explicitly:**

Edit `~/.cco/config.toml`:
```toml
rtk_enabled = false
```

Or set the environment variable:
```bash
export CCO_TELEMETRY_DISABLED=1  # Disables RTK telemetry only, not the hook
```

**Verify RTK status:**
```bash
cco doctor
```

Look for the `RTK (Token Killer)` check. When RTK is installed and enabled, you'll see:
```
✅ RTK (Token Killer)
   Installed (v1.0.0) — PreToolUse hook active
   • rtk hook claude rewrites dev commands before they reach the LLM.
   • To disable: set rtk_enabled = false in ~/.cco/config.toml
```

### Telemetry

CCO **disables RTK's anonymous telemetry by default** by setting `RTK_TELEMETRY_DISABLED=1` in the generated Claude Code settings environment. This prevents the RTK subprocess from phoning home.

To enable RTK telemetry, remove this environment variable from your settings:
```bash
# In ~/.cco/config.toml, you would need to override the env section
# (currently not exposed as a config option; contact support if needed)
```

### Token Savings

View your RTK savings with the `cco cost` command:

```bash
cco cost dashboard
```

Output includes:
```
📊 Total Spend (today): $12.34
   RTK Savings: $2.45 (2,800 tokens @ $0.87/MTok)
```

The implied savings are calculated as:
- Total tokens saved by RTK (from `rtk gain`)
- Multiplied by your blended input-token rate (from your actual Claude Code usage)
- Falls back to Sonnet input rate ($3.00/MTok) if no usage data exists

See [Cost Tracking](#cost-tracking) for more details.

---

## Compression Modes

The `compression_mode` setting in `~/.cco/config.toml` selects which context-compression tool(s) CCO applies to session context before sending to Claude. These modes are mutually exclusive in the shell-output layer.

### Mode Overview

| Mode | Tool | Purpose | Layer |
|------|------|---------|-------|
| `rtk` | RTK (Rust Token Killer) | Compress Bash command output 60–90% | Shell output (PreToolUse hook) |
| `lean-ctx` | lean-ctx + LanceDB | Semantic search for general session context | Runtime context (MCP server) |
| `lean-ctx-mcp-only` | lean-ctx (MCP only) | Semantic search for MCP server context, not general session | MCP layer (no general context) |
| `none` | (none) | No compression | — |

**Default:** `rtk` (when mode is absent)

### Configuration

Edit `~/.cco/config.toml`:

```toml
# Use RTK (default) for shell-output compression
compression_mode = "rtk"

# Use lean-ctx for semantic compression of general session context
compression_mode = "lean-ctx"

# Use lean-ctx for MCP-only context (not general session)
compression_mode = "lean-ctx-mcp-only"

# Disable compression entirely
compression_mode = "none"
```

### RTK Mode

**When to use:** You have `rtk` installed and want to compress Bash command output.

**Requires:** `rtk` binary on PATH (install via `brew install rtk`)

**Effect:** CCO wires the RTK PreToolUse hook (`rtk hook claude`) into Claude Code settings. Every Bash command output is rewritten to strip ANSI colors, duplicates, and verbose headers before reaching Claude's context.

**Savings:** 60–90% reduction on dev-command output.

### lean-ctx Mode

**When to use:** You want semantic-search compression for general session context (prompts, responses, intermediate state).

**Requires:** `lean-ctx` binary on PATH and LanceDB (installed via `brew install lean-ctx`)

**Effect:** CCO injects the lean-ctx MCP server and sets `LEAN_CTX_NO_UPDATE_CHECK=1`. The MCP server performs semantic search and compresses context by selecting the most relevant chunks.

**Note:** As of v2026.6.7, integration is detection + doctor reporting. Runtime behavior is staggered for later increments.

### lean-ctx MCP-Only Mode

**When to use:** You want semantic compression only for MCP server tools (task DAG, knowledge graph), not general session context.

**Requires:** `lean-ctx` binary on PATH

**Effect:** Similar to `lean-ctx` but restricts compression to MCP context only. General session context flows uncompressed.

### none Mode

**When to use:** You want no compression (baseline behavior).

**Effect:** No hooks are registered; context flows uncompressed to Claude.

### Verify Compression Mode

Check your active mode:

```bash
cco doctor
```

Output includes:
```
✅ RTK (Token Killer)
   Installed (v1.0.0) — PreToolUse hook active
   compression_mode = rtk
```

---

## Headroom Proxy

Headroom is an optional daemon that compresses API traffic and manages prompt caching at the API boundary. CCO can supervise and route traffic through a Headroom instance.

### How It Works

When `headroom_enabled = true`, the CCO daemon:
1. Spawns a Headroom proxy subprocess (`headroom proxy --port <port>`)
2. Polls the proxy health endpoint every ~30 seconds
3. Injects `ANTHROPIC_BASE_URL=http://127.0.0.1:<port>` into Claude Code sessions when the proxy is healthy
4. Falls back to direct Anthropic API traffic if the proxy becomes unhealthy (never routes through a dead proxy)
5. Cleans up the proxy process on daemon shutdown

### Installation

Install Headroom separately (not included with CCO):

```bash
pip install "headroom-ai[proxy]"
```

Verify installation:

```bash
headroom --version
```

### Configuration

**Enable Headroom in `~/.cco/config.toml`:**

```toml
headroom_enabled = true
headroom_port = 18787           # Default: 18787 (avoid conflicts with 8787)
```

**Note on TTL:** The `headroom_ttl_hours` configuration key is accepted for documentation purposes but is currently unused (the upstream `headroom proxy` CLI does not support a `--ttl` flag).

### Defaults

| Key | Default |
|-----|---------|
| `headroom_enabled` | `false` (opt-in) |
| `headroom_port` | `18787` |

### Health Gating

Headroom is **health-gated** — CCO only injects `ANTHROPIC_BASE_URL` when the proxy responds to health checks. If the proxy crashes or becomes unresponsive, Claude Code routes directly to Anthropic API (never through a dead proxy).

Health check:
```bash
curl http://127.0.0.1:18787/health
```

### Verify Headroom Status

```bash
cco doctor
```

Output:
```
✅ Headroom
   Installed (v1.0.0) — API-boundary compression available
   Note: CCO does not yet manage Headroom proxy supervision (investigation #20).
```

### Related

- **Investigation #20:** Daemon-side Headroom proxy supervision is in active research

---

## lean-ctx

lean-ctx is an optional MCP server that compresses context via semantic search and LanceDB embeddings. It integrates with CCO when `compression_mode` is set to `lean-ctx` or `lean-ctx-mcp-only`.

### How It Works

When lean-ctx is enabled (via `compression_mode`), CCO:
1. Injects the lean-ctx MCP server into Claude Code's MCP configuration
2. Sets `LEAN_CTX_NO_UPDATE_CHECK=1` to disable auto-update checks
3. lean-ctx performs semantic search on context to identify the most relevant chunks
4. Unused or low-relevance context is dropped, reducing token consumption
5. The daemon exports lean-ctx savings metrics (tokens saved, original baseline, event count) to OTLP for inclusion in cost dashboards and reports

### Installation

Install lean-ctx separately (not included with CCO):

```bash
brew install lean-ctx
```

Or from source: https://github.com/dimfeld/lean-ctx

Verify installation:

```bash
lean-ctx --version
```

### Configuration

**Activate lean-ctx via `compression_mode` in `~/.cco/config.toml`:**

```toml
# General context compression
compression_mode = "lean-ctx"

# MCP-only compression
compression_mode = "lean-ctx-mcp-only"
```

No additional keys needed (defaults are sensible).

### Savings Export

When `compression_mode` is `"lean-ctx"` or `"lean-ctx-mcp-only"`, the daemon periodically reads lean-ctx's savings ledger (`~/.local/share/lean-ctx/savings/ledger.jsonl`) and exports aggregated token-savings metrics as OTLP gauges:

- `lean_ctx.tokens_saved` — Total tokens saved per day
- `lean_ctx.tokens_original` — Baseline token count (counterfactual without compression)
- `lean_ctx.commands` — Number of compression events

These metrics flow to the metrics-ingest Worker alongside other telemetry and surface in the cost dashboard.

### Verify lean-ctx Status

```bash
cco doctor
```

Output:
```
✅ lean-ctx
   Installed (v1.0.0) — context compression available
   compression_mode = lean-ctx
   Savings exporter: active
```

---

## Ponytail

Ponytail is an optional Claude Code plugin that injects anti-bloat rules to encourage concise, efficient prompts and responses. It is lazy and opt-in; users must install it manually and CCO wires it when enabled.

### How It Works

Ponytail is a Claude Code plugin that adds rules constraining:
- Response length (avoid unnecessarily verbose explanations)
- Code generation (prefer concise algorithms over over-engineered solutions)
- Comments (encourage meaningful comments, skip the obvious)

When `ponytail_enabled = true`, CCO detects the plugin and configures it with the chosen aggressiveness mode.

### Installation

Install Ponytail via the Claude Code plugin marketplace:

```bash
# In Claude Code, type:
/plugin marketplace add DietrichGebert/ponytail
/plugin install ponytail@ponytail
```

Verify installation:
```bash
cco doctor
```

### Configuration

**Enable Ponytail in `~/.cco/config.toml`:**

```toml
ponytail_enabled = true
ponytail_mode = "full"       # Options: "lite", "full", "ultra"
```

**Modes:**

| Mode | Aggressiveness | Use Case |
|------|---|---|
| `lite` | Gentle suggestions | Light policing of verbosity |
| `full` | Moderate (default) | Balanced conciseness vs. completeness |
| `ultra` | Strict | Aggressive minimalism; best for token-heavy tasks |

**Defaults:**

| Key | Default |
|-----|---------|
| `ponytail_enabled` | `false` (opt-in) |
| `ponytail_mode` | `"full"` (if enabled) |

### Verify Ponytail Status

```bash
cco doctor
```

Output when enabled and installed:
```
✅ Ponytail
   Installed and enabled
   Mode: full
   Lazy senior dev anti-bloat ruleset active
```

Output when enabled but not installed:
```
⚠️  Ponytail
   Not installed (optional — lazy dev anti-bloat ruleset)
   Note: ponytail_enabled = true but plugin not installed (will not load).
```

---

## Codex CLI

Codex CLI is an optional external code reviewer for second-opinion reviews covering correctness, security, and edge cases. CCO provides a launcher that automatically injects OTEL telemetry configuration.

### How It Works

When Codex is installed and authenticated:
1. CCO detects the `codex` binary on PATH
2. `cco codex [args...]` command launches Codex with injected OTEL configuration
3. CCO non-destructively merges `[otel]` config into `~/.codex/config.toml` (reads daemon's OTLP receiver port)
4. All Codex flags and arguments pass through unchanged
5. Codex telemetry flows to CCO's metrics-ingest endpoint for token counting and cost tracking
6. CCO's PostToolUse hook is registered in Codex hooks configuration

### Installation

Install Codex CLI separately (not included with CCO):

```bash
pip install codex-ai
```

Authenticate:

```bash
codex login
```

Verify installation:

```bash
cco doctor
```

### Usage

Launch Codex with CCO integration:

```bash
cco codex [args...]
```

Examples:
```bash
cco codex --help
cco codex review myfile.py
cco codex --model gpt-4o --verbose
```

All arguments pass directly to the Codex binary.

### Telemetry

When Codex is launched via `cco codex`, the daemon provides its OTLP receiver endpoint. Codex exports metrics to CCO, which includes them in the cost dashboard and metrics reports.

### Configuration

No explicit configuration needed. When `codex` is on PATH and authenticated, it is auto-detected.

### Verify Codex Status

```bash
cco doctor
```

Output when installed and authenticated:
```
✅ Codex CLI
   Available (v1.0.0) — launcher ready
```

Output when installed but not authenticated:
```
⚠️  Codex CLI
   Installed but not authenticated
   Run: codex login
```

---

## Statusline

The statusline is a real-time status bar showing your session's model, effort, directory, git branch, context tokens, cost, and rate limits. It appears in the Claude Code TUI header.

### How it Works

CCO ships a plugin at `cco-plugin/statusline.sh` that generates a status line on each refresh. The daemon registers this as the `statusLine` in the generated Claude Code settings, which invokes it whenever Claude Code renders the header.

The statusline displays:
- **Model** — Current model (opus/sonnet/haiku)
- **Effort** — Reasoning level (none/light/medium)
- **Directory** — Current working directory
- **Branch** — Git branch if in a repo
- **Context** — Cached vs. live context tokens
- **Cost** — Total spend this session
- **Rate limits** — Requests per minute, tokens per minute

### Configuration

**Enable statusline (default):**

The statusline is **enabled by default**. No config needed.

**Disable statusline:**

Edit `~/.cco/config.toml`:
```toml
statusline_enabled = false
```

**Custom statusline command:**

To override the statusline command entirely:
```toml
statusline_command = "bash /path/to/my/custom/status.sh"
```

Or use an environment variable:
```bash
export CCO_STATUSLINE_COMMAND="echo 'custom status'"
```

The custom command must output a single line of plain text (no ANSI colors recommended for compatibility).

### Verify Statusline Status

```bash
cco doctor
```

The statusline is configured in the daemon's generated Claude Code settings and will be active when the daemon is running.

---

## Remote Control

Claude Code Remote Control connects your local session to claude.ai/code and the Claude mobile app, allowing you to control your coding session from any device.

### How it Works

When Remote Control is enabled, Claude Code registers the session with Anthropic's control plane using OAuth. The session name is derived from your current directory basename (e.g., "myproject" when in `/path/to/myproject`). You can then connect from claude.ai/code or your phone and send commands to your local session.

### Requirements

- Recent version of Claude Code (2024+)
- OAuth login to Claude (`ANTHROPIC_API_KEY` **not** set)
- Network access to Anthropic control plane

**Important:** Remote Control is **OAuth-only**. If you set `ANTHROPIC_API_KEY`, Remote Control is automatically disabled (silent fail) because API-key authentication is incompatible with the control plane.

### Configuration

**Enable Remote Control (default):**

Remote Control is **enabled by default**. No config needed.

**Disable Remote Control:**

Choose one of these methods:

**CLI flag (highest precedence):**
```bash
cco --no-remote-control
```

**Environment variable:**
```bash
export CCO_REMOTE_CONTROL=0
```

**Config file:**
Edit `~/.cco/config.toml`:
```toml
remote_control_enabled = false
```

**Precedence (highest to lowest):**
1. `cco --remote-control` / `cco --no-remote-control` CLI flags
2. `CCO_REMOTE_CONTROL` environment variable (`0`/`false` disables, `1`/`true` enables)
3. `remote_control_enabled` field in `~/.cco/config.toml`
4. Default: `true` (enabled)

### Session Naming

The session name displayed in the control plane is derived from your current directory:

**Automatic (default):**
The basename of `pwd` is used. For example:
```bash
cd /Users/you/projects/myapp
cco  # Session named "myapp"
```

**Custom session name:**

Edit `~/.cco/config.toml`:
```toml
session_name = "my-custom-session"
```

Or use an environment variable (when supported):
```bash
export CCO_SESSION_NAME="my-session"
```

### Verify Remote Control Status

Check your configuration:
```bash
cco doctor
```

When Remote Control is enabled, you should see it logged in Claude Code's UI when you launch a session. To confirm it's working, sign in to https://claude.ai/code and look for your session listed under "Remote Sessions."

---

## Health Check (`cco doctor`)

The `cco doctor` command performs a comprehensive health check of your CCO installation and can auto-remediate common issues.

### Usage

**Run a health check:**
```bash
cco doctor
```

Output:
```
Claude Code Orchestra - Installation Health Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Binary
   Installed at ~/.local/bin/cco (v2026.2.68)

⚠️  RTK (Token Killer)
   Not installed (optional — saves 60-90% tokens on dev commands)
   • Install: brew install rtk
   • Or: curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/...

✅ Permissions
   File permissions are secure

✅ All checks passed. System is healthy.
```

**Auto-fix common issues:**
```bash
cco doctor --fix
```

The `--fix` flag automatically applies safe remediations:
- Creates missing directories (`~/.cco/`, `~/.cco/logs/`, `~/.cco/knowledge/`)
- Fixes directory permissions (sets `~/.cco/` to `0700`)
- Codesigns and clears quarantine on macOS (launchd requirement)
- Starts the daemon if it's not running
- Installs RTK via Homebrew or curl (optional)

Manual-only items remain reported with instructions.

### What `cco doctor` Checks

1. **Binary** — Executable exists and is the correct version
2. **Directory Structure** — Required subdirectories exist with correct permissions
3. **Permissions** — `~/.cco/` is `0700` (owner-only)
4. **Databases** — Task store, knowledge graph, metrics, event store
5. **Service** — launchd (macOS) or systemd (Linux) registration
6. **Connectivity** — Daemon is running and responsive
7. **Unix Socket** — `~/.cco/daemon.sock` exists
8. **Embeddings** — Semantic embedding model loaded and ready
9. **RTK (Token Killer)** — Optional token compression tool
10. **Codex CLI** — Optional external code reviewer
11. **Other Systems** — Token manager, orchestration, agent metrics, code index

### Output Formats

**Human-readable (default):**
```bash
cco doctor
```

**JSON:**
```bash
cco doctor --format json
```

Returns structured health data suitable for scripting or dashboards.

**Verbose:**
```bash
cco doctor --verbose
```

Includes debug-level diagnostics.

**Quiet:**
```bash
cco doctor --quiet
```

Shows only errors; suitable for CI/CD pipelines.

---

## Cost Tracking

CCO tracks API costs across all Claude Code sessions. The `cco cost` command provides dashboards, breakdowns, and savings analysis.

### Dashboard

View your total spend and breakdown by model:

```bash
cco cost dashboard
```

Output:
```
╔════════════════════════════════════════════════════════════════╗
║              CLAUDE CODE COST DASHBOARD (today)              ║
╚════════════════════════════════════════════════════════════════╝

📊 Total Spend (today): $12.34
   Conversations: 3  | Messages: 24
   Daily Average: $12.34
   Cache Savings: $1.23

💼 Breakdown by Model Tier:
   ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░ opus        $8.50 ( 68.9%)
   ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ sonnet      $3.40 ( 27.6%)
   ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ haiku       $0.44 (  3.6%)

🎯 RTK Savings: $2.45 (2,800 tokens @ $0.87/MTok)
   1,200 commands trimmed  | 61.2% average compression
```

### RTK Savings

When RTK is installed and active, the dashboard includes implied USD savings:

- **Total tokens saved**: Aggregated from `rtk gain --format json`
- **Implied USD**: Tokens × blended input rate from your actual usage
- **Fallback rate**: $3.00/MTok (Sonnet) when no usage data exists
- **Marked as estimate**: When the rate is not based on real metrics

### Breakdown by Session

```bash
cco cost session <session-id>
```

Shows cost for a specific conversation session.

### Breakdown by Agent

```bash
cco cost agents
```

Shows spending by agent type (Opus, Sonnet, Haiku specialists).

### Cache Report

```bash
cco cost cache
```

Shows cache write costs, cache hit savings, and net benefit of prompt caching.

### Spending Gate

Set a budget and get warnings when you exceed it:

```bash
cco cost gate --period today --max-usd 50
```

This command:
- Loads metrics for the selected period (today/week/month)
- Compares total spend to your threshold
- Exits with code 0 if under budget, 1 if over
- Useful for CI/CD pipelines to block expensive runs

Configure default budgets in `~/.cco/config.toml`:
```toml
daily_budget_usd = 50.0
weekly_budget_usd = 200.0
```

### Supported Periods

- `today` — Since midnight (local time)
- `week` — Last 7 days (Monday–Sunday)
- `month` — Calendar month (1st–last day)
- `all` — Entire history

### Export Costs

```bash
cco cost export --output report.csv --format csv --period today
```

Supported formats: `csv`, `json`.

---

## Configuration Reference

All configuration is stored in `~/.cco/config.toml` (TOML format). This file is optional; absent fields default to sensible values.

### Example Config

```toml
# Default editor client
default_client = "claude-code"

# Context compression: rtk | lean-ctx | lean-ctx-mcp-only | none
compression_mode = "rtk"

# RTK (Rust Token Killer) — token compression hook
rtk_enabled = true

# Statusline display in Claude Code header
statusline_enabled = true
statusline_command = "bash ~/.local/bin/cco-plugin/statusline.sh"

# Remote Control (OAuth) session naming
remote_control_enabled = true
session_name = "myproject"

# Telemetry (aggregate token/cost metrics)
telemetry_enabled = true
telemetry_upload_transcripts = false

# Spending budgets
daily_budget_usd = 50.0
weekly_budget_usd = 200.0

# TUI refresh when SSE unavailable
tui_refresh_ms = 1000

# Headroom API-boundary proxy (opt-in)
headroom_enabled = false
headroom_port = 18787
headroom_ttl_hours = 24

# Ponytail anti-bloat rules (opt-in)
ponytail_enabled = false
ponytail_mode = "full"
```

### Config Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_client` | string | none | Preferred editor: `"claude-code"` or `"auto"` |
| `use_happy` | bool | none | Enable Happy wrapper for mobile/web (experimental) |
| `rtk_enabled` | bool | `true` | Enable RTK PreToolUse hook when `rtk` is on PATH |
| `compression_mode` | string | `"rtk"` | Context compression mode: `"rtk"`, `"lean-ctx"`, `"lean-ctx-mcp-only"`, or `"none"` |
| `statusline_enabled` | bool | `true` | Include statusline in Claude Code settings |
| `statusline_command` | string | none | Override statusline command (default: embedded bash script) |
| `remote_control_enabled` | bool | `true` | Enable OAuth-based remote control from web/mobile |
| `session_name` | string | none | Custom session name (default: current directory basename) |
| `telemetry_enabled` | bool | `true` | Enable aggregate telemetry (tokens, costs) |
| `telemetry_upload_transcripts` | bool | `false` | Upload full prompts/responses (requires `telemetry_enabled = true`) |
| `daily_budget_usd` | float | none | Daily spending budget (soft warning via PostToolUse hook) |
| `weekly_budget_usd` | float | none | Weekly spending budget |
| `tui_refresh_ms` | u64 | `1000` | TUI polling interval (ms) when SSE unavailable; clamped to minimum 250ms |
| `headroom_enabled` | bool | `false` | Enable Headroom proxy supervision (opt-in) |
| `headroom_port` | u16 | `18787` | Port for Headroom proxy (only used when `headroom_enabled = true`) |
| `headroom_ttl_hours` | u32 | `24` | Headroom cache TTL in hours (only used when `headroom_enabled = true`) |
| `ponytail_enabled` | bool | `false` | Enable Ponytail anti-bloat plugin (opt-in; must be installed manually) |
| `ponytail_mode` | string | `"full"` | Ponytail aggressiveness: `"lite"`, `"full"`, or `"ultra"` (only used when `ponytail_enabled = true`) |

### Environment Variables

Override config with these env vars (highest precedence after CLI flags):

| Variable | Format | Overrides |
|----------|--------|-----------|
| `CCO_CONFIG_PATH` | path | Config file location (default: `~/.cco/config.toml`) |
| `CCO_REMOTE_CONTROL` | `0`, `1`, `true`, `false` | `remote_control_enabled` |
| `CCO_TELEMETRY_DISABLED` | any value set | Disables aggregate telemetry |
| `CCO_TELEMETRY_TRANSCRIPTS` | any value set | Enables transcript upload |

---

## Editor Surfaces

CCO works across all Claude Code environments: CLI, VS Code, JetBrains IDEs, and Claude Code running inside Cursor's terminal.

### Claude Code CLI

The primary interface. Launch with `cco`:

```bash
cco                   # Direct launch with default settings
cco --no-remote-control  # Disable Remote Control
cco --help           # Show all options
```

### VS Code Extension

Claude Code integrates with VS Code's built-in AI editor. CCO's daemon settings (hooks, statusline, RTK) apply automatically:

1. Install Claude Code from the VS Code extension marketplace
2. Launch Claude Code (keyboard shortcut or command palette)
3. CCO's settings and integrations load automatically

### JetBrains IDEs

Claude Code plugin for IntelliJ, PyCharm, WebStorm, etc. Same integration as VS Code:

1. Install Claude Code from the JetBrains plugin marketplace
2. Open the Claude Code chat panel
3. CCO's settings apply automatically

### Cursor Integration

Cursor has two AI features:
- **Cursor Agent** (native, no Claude Code) — Cost tracking unavailable
- **Claude Code in terminal** — Full CCO integration

For CCO cost tracking and remote control:

```bash
# In Cursor's terminal, use cco or claude directly
cco
claude "write a test for this function"
```

If you use Cursor's native agent, costs won't be tracked by `cco cost` (Cursor doesn't expose token usage reliably). However, `cco cost` still tracks all Claude Code runs inside Cursor's terminal.

### MCP Server Registration

The CCO daemon includes an MCP (Model Context Protocol) server. Register it in:

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "cco": {
      "command": "cco",
      "args": ["mcp"]
    }
  }
}
```

**Cursor** (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "cco": {
      "command": "cco",
      "args": ["mcp"]
    }
  }
}
```

The MCP server provides:
- Code indexing and search
- Knowledge graph queries
- Task management
- Real-time metrics and cost data

---

## Troubleshooting

### RTK Not Activating

**Symptom:** `cco doctor` shows RTK as "Not installed" even though you have it installed.

**Check 1:** Verify RTK is on PATH:
```bash
which rtk
rtk --version
```

If not found, re-install RTK (see [Installation](#installation)).

**Check 2:** Verify RTK hook is enabled:
```bash
cco doctor
```

If you see `rtk_enabled = false` in the output, enable it:
```bash
# Edit ~/.cco/config.toml and remove or set:
rtk_enabled = true
```

**Check 3:** Restart the daemon:
```bash
cco daemon restart
```

### Statusline Not Showing

**Symptom:** Claude Code header doesn't show the statusline.

**Check 1:** Verify statusline is enabled:
```bash
cco doctor
```

If disabled, enable it in `~/.cco/config.toml`:
```toml
statusline_enabled = true
```

**Check 2:** Verify the daemon is running:
```bash
cco daemon status
```

If not running, start it:
```bash
cco daemon start
```

**Check 3:** Restart Claude Code after enabling.

### Remote Control Not Working

**Symptom:** Session doesn't appear in https://claude.ai/code.

**Check 1:** Verify Remote Control is enabled:
```bash
echo $CCO_REMOTE_CONTROL  # Should be empty or 0/1
```

**Check 2:** Verify OAuth is available:

Remote Control requires OAuth login. If you have `ANTHROPIC_API_KEY` set, disable it:
```bash
unset ANTHROPIC_API_KEY
```

**Check 3:** Verify you're signed into Claude:

In Claude Code, sign in with your Anthropic account. OAuth is required.

**Check 4:** Check Claude Code version:

Remote Control requires a recent version of Claude Code. Update if needed:
```bash
claude --version
```

### Cost Dashboard Shows No Data

**Symptom:** `cco cost dashboard` shows "No cost data available."

**Check 1:** Verify Claude Code sessions are being recorded:
```bash
ls -la ~/.claude/projects/
```

If the directory is empty, run a Claude Code session first.

**Check 2:** Check telemetry is enabled (default):
```bash
echo $CCO_TELEMETRY_DISABLED  # Should be empty if enabled
```

**Check 3:** Verify the metrics database exists:
```bash
ls -la ~/.cco/metrics.duckdb
```

If missing, the daemon will create it on next start:
```bash
cco daemon restart
```

---

## Related Documentation

- [MCP and Control Plane](docs/mcp-and-control-plane.md) — MCP server details and daemon control plane
- [Telemetry](docs/telemetry.md) — Aggregate metrics and transcript upload
- [Configuration](docs/configuration.md) — Detailed config file reference

---

## Support

For issues or feature requests:
- GitHub: https://github.com/langstons/cco/issues
- CCO Memory: See local `.claude/projects/cc-orchestra/memory/` for known issues and workarounds
