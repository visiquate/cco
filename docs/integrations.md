# CCO Integrations

This guide covers the external tools and services that integrate with Claude Code Orchestra (CCO), and how to configure them.

## Quick Links

- [RTK (Rust Token Killer)](#rtk-rust-token-killer) — Bash output compression
- [Statusline](#statusline) — Session status display
- [Remote Control](#remote-control) — Mobile and web control
- [Health Check (`cco doctor`)](#health-check-cco-doctor) — System diagnosis
- [Editor Surfaces](#editor-surfaces) — VS Code, JetBrains, and more
- [Configuration Reference](#configuration-reference) — All config keys

---

## RTK (Rust Token Killer)

RTK compresses Bash command output 60–90% before it reaches the LLM context. This dramatically reduces token usage on dev-heavy tasks.

### How it Works

When `rtk` is on PATH and `rtk_enabled != false` in your config, CCO registers RTK's `rtk hook claude` hook as a **PreToolUse** hook in the generated Claude Code settings. RTK intercepts bash commands, rewrites their output to strip noise (ANSI colors, repeated lines, verbose headers), and passes the trimmed output upstream to Claude.

CCO also **disables RTK's own anonymous telemetry by default** by setting `RTK_TELEMETRY_DISABLED=1` in the generated Claude Code settings environment. This prevents the RTK subprocess from phoning home.

### Installation

Choose any method:

**Homebrew (macOS and Linux):**
```bash
brew install rtk
```

**Curl installer (macOS and Linux):**
```bash
curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh
```

**Auto-install via `cco doctor --fix`:**
```bash
cco doctor --fix
```

This detects Homebrew and uses it if available, or falls back to the curl installer.

### Configuration

RTK is **enabled by default** when `rtk` is on PATH. To disable it explicitly, edit `~/.cco/config.toml`:

```toml
rtk_enabled = false
```

To verify RTK status:

```bash
cco doctor
```

Look for the `RTK (Token Killer)` check. When RTK is installed and enabled, you'll see:

```
RTK (Token Killer)
   Installed (v1.0.0) — PreToolUse hook active
   • rtk hook claude rewrites dev commands before they reach the LLM.
   • To disable: set rtk_enabled = false in ~/.cco/config.toml
```

### Token Savings

View your RTK savings in the cost dashboard:

```bash
cco cost dashboard
```

Output includes:

```
RTK Savings: $2.45 (2,800 tokens @ $0.87/MTok)
   1,200 commands trimmed  | 61.2% average compression
```

The implied savings are calculated as total tokens saved by RTK multiplied by your blended input-token rate (from your actual Claude Code usage). Falls back to Sonnet input rate ($3.00/MTok) if no usage data exists. See [Cost Metrics](cost-metrics.md) for full details.

---

## Statusline

The statusline is a real-time status bar showing your session's model, effort, directory, git branch, context tokens, cost, and rate limits. It appears in the Claude Code TUI header.

### How it Works

CCO ships a bundled statusline script that generates a status line on each refresh. The daemon registers this as the `statusLine` in the generated Claude Code settings, which invokes it whenever Claude Code renders the header.

The statusline displays:

- **Model** — Current model (opus/sonnet/haiku)
- **Effort** — Reasoning level (none/light/medium)
- **Directory** — Current working directory
- **Branch** — Git branch if in a repo
- **Context** — Cached vs. live context tokens
- **Cost** — Total spend this session
- **Rate limits** — Requests per minute, tokens per minute

### Configuration

The statusline is **enabled by default**. No config needed.

To disable it, edit `~/.cco/config.toml`:

```toml
statusline_enabled = false
```

To override the statusline command entirely:

```toml
statusline_command = "bash /path/to/my/custom/status.sh"
```

The custom command must output a single line of plain text.

---

## Remote Control

Claude Code Remote Control connects your local session to claude.ai/code and the Claude mobile app, allowing you to control your coding session from any device.

### How it Works

When Remote Control is enabled, Claude Code registers the session with Anthropic's control plane using OAuth. The session name is derived from your current directory basename (e.g., "myproject" when in `/path/to/myproject`). You can then connect from claude.ai/code or your phone and send commands to your local session.

### Requirements

- Recent version of Claude Code (2024+)
- OAuth login to Claude (`ANTHROPIC_API_KEY` **not** set)
- Network access to Anthropic control plane

**Important:** Remote Control is **OAuth-only**. If you set `ANTHROPIC_API_KEY`, Remote Control is automatically disabled because API-key authentication is incompatible with the control plane.

### Configuration

Remote Control is **enabled by default**. No config needed.

To disable it, choose one of these methods:

**CLI flag (highest precedence):**
```bash
cco --no-remote-control
```

**Environment variable:**
```bash
export CCO_REMOTE_CONTROL=0
```

**Config file:**
```toml
remote_control_enabled = false
```

**Precedence (highest to lowest):**
1. `cco --remote-control` / `cco --no-remote-control` CLI flags
2. `CCO_REMOTE_CONTROL` environment variable (`0`/`false` disables, `1`/`true` enables)
3. `remote_control_enabled` field in `~/.cco/config.toml`
4. Default: `true` (enabled)

### Session Naming

The session name displayed in the control plane is derived from your current directory by default:

```bash
cd /Users/you/projects/myapp
cco  # Session named "myapp"
```

To set a custom session name, edit `~/.cco/config.toml`:

```toml
session_name = "my-custom-session"
```

### Verify Remote Control Status

```bash
cco doctor
```

To confirm it's working, sign in to https://claude.ai/code and look for your session listed under "Remote Sessions."

---

## Health Check (`cco doctor`)

The `cco doctor` command performs a comprehensive health check of your CCO installation and can auto-remediate common issues.

### Usage

**Run a health check:**
```bash
cco doctor
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
10. **Other Systems** — Token manager, orchestration, agent metrics, code index

### Output Formats

```bash
cco doctor              # Human-readable (default)
cco doctor --format json  # JSON output for scripting
cco doctor --verbose    # Includes debug-level diagnostics
cco doctor --quiet      # Errors only; suitable for CI/CD
```

---

## Editor Surfaces

CCO works across all Claude Code environments: CLI, VS Code, JetBrains IDEs, and Claude Code running inside Cursor's terminal.

### Claude Code CLI

The primary interface. Launch with `cco`:

```bash
cco                        # Direct launch with default settings
cco --no-remote-control    # Disable Remote Control for this session
cco --help                 # Show all options
```

### VS Code Extension

Claude Code integrates with VS Code's built-in AI editor. CCO's daemon settings (hooks, statusline, RTK) apply automatically:

1. Install Claude Code from the VS Code extension marketplace
2. Launch Claude Code (keyboard shortcut or command palette)
3. CCO's settings and integrations load automatically

### JetBrains IDEs

Claude Code plugin for IntelliJ, PyCharm, WebStorm, and others. Same integration as VS Code:

1. Install Claude Code from the JetBrains plugin marketplace
2. Open the Claude Code chat panel
3. CCO's settings apply automatically

### Cursor Integration

Cursor has two AI features:

- **Cursor Agent** (native, no Claude Code) — Cost tracking unavailable
- **Claude Code in terminal** — Full CCO integration

For CCO cost tracking and remote control, run `cco` or `claude` directly in Cursor's integrated terminal. If you use Cursor's native agent, costs won't be tracked by `cco cost` (Cursor doesn't expose token usage reliably). However, `cco cost` still tracks all Claude Code runs inside Cursor's terminal.

### MCP Server Registration

Register the CCO MCP server in your editor's config to expose cost tools, knowledge graph, and task DAG to your Claude session.

**Claude Desktop** (`~/.claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "cco": {
      "command": "cco",
      "args": ["mcp", "serve"]
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
      "args": ["mcp", "serve"]
    }
  }
}
```

See [MCP and Control Plane](mcp-and-control-plane.md) for the full tool list and example payloads.

---

## Configuration Reference

All configuration is stored in `~/.cco/config.toml` (TOML format). This file is optional; absent fields default to sensible values.

### Config Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `rtk_enabled` | bool | `true` | Enable RTK PreToolUse hook when `rtk` is on PATH |
| `statusline_enabled` | bool | `true` | Include statusline in Claude Code settings |
| `statusline_command` | string | — | Override statusline command (default: bundled script) |
| `remote_control_enabled` | bool | `true` | Enable OAuth-based remote control from web/mobile |
| `session_name` | string | — | Custom session name (default: current directory basename) |
| `telemetry_enabled` | bool | `true` | Enable aggregate telemetry (tokens, costs) |
| `telemetry_upload_transcripts` | bool | `false` | Upload full prompts/responses (requires `telemetry_enabled = true`) |
| `daily_budget_usd` | float | — | Daily spending budget (soft warning via PostToolUse hook) |
| `weekly_budget_usd` | float | — | Weekly spending budget |

### Environment Variables

| Variable | Format | Overrides |
|----------|--------|-----------|
| `CCO_CONFIG_PATH` | path | Config file location (default: `~/.cco/config.toml`) |
| `CCO_REMOTE_CONTROL` | `0`, `1`, `true`, `false` | `remote_control_enabled` |
| `CCO_TELEMETRY_DISABLED` | any value set | Disables aggregate telemetry |
| `CCO_TELEMETRY_TRANSCRIPTS` | any value set | Enables transcript upload |
| `CCO_DAILY_BUDGET_USD` | float string | `daily_budget_usd` |

---

## Troubleshooting

### RTK Not Activating

**Symptom:** `cco doctor` shows RTK as "Not installed" even though you have it installed.

**Check 1:** Verify RTK is on PATH:
```bash
which rtk
rtk --version
```

**Check 2:** Verify `rtk_enabled` is not set to false in `~/.cco/config.toml`.

**Check 3:** Restart the daemon:
```bash
cco daemon restart
```

### Statusline Not Showing

**Symptom:** Claude Code header doesn't show the statusline.

1. Run `cco doctor` and check that `statusline_enabled` is true.
2. Check that the daemon is running: `cco daemon status`. If not, `cco daemon start`.
3. Restart Claude Code after enabling.

### Remote Control Not Working

**Symptom:** Session doesn't appear in https://claude.ai/code.

1. Verify `ANTHROPIC_API_KEY` is not set in your shell. Remote Control requires OAuth, not an API key.
2. Sign into Claude Code with your Anthropic account if not already done.
3. Run `cco doctor` to confirm `remote_control_enabled` is true.
4. Ensure your Claude Code installation is recent.

### Cost Dashboard Shows No Data

**Symptom:** `cco cost dashboard` shows "No cost data available."

1. Verify `~/.claude/projects/` contains JSONL files (run a Claude Code session first if not).
2. Verify the daemon is running: `cco daemon status`.
3. Run `cco doctor --fix` to create missing directories and restart the daemon if needed.

---

## See Also

- [MCP and Control Plane](mcp-and-control-plane.md) — MCP server details and daemon control plane
- [Telemetry](telemetry.md) — Aggregate metrics and transcript upload
- [Cost Metrics](cost-metrics.md) — Budget tracking, cache efficiency, and RTK savings
