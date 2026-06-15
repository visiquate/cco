# CCO — Claude Orchestra

CCO is a Rust CLI and background daemon that wraps and extends Claude Code. It adds persistent cost and token tracking backed by DuckDB, a roster of 117 specialized agents across three model tiers with automatic work-delegation nudges, an MCP server exposing task DAGs, a knowledge graph, code search, and cost tools to any MCP client, and conveniences such as the RTK token-compression shim, a configurable statusline, and Claude Code Remote Control. A terminal TUI shows cost, cache efficiency, and delegation metrics in real time.

## Contents

- [Features](#features)
- [Install](#install)
- [Quickstart](#quickstart)
- [Configuration](#configuration)
- [Telemetry and Privacy](#telemetry-and-privacy)
- [Documentation](#documentation)
- [License](#license)

---

## Features

### Cost Intelligence

CCO tracks every Claude Code API call without touching Anthropic's servers. All data stays on your machine in a local DuckDB file.

- Parses every `~/.claude/projects/*.jsonl` transcript into `~/.cco/claude_history.duckdb`; no external service required
- `cco cost dashboard` — totals and trends by period (today / week / month / all)
- `cco cost session` — per-session breakdown with token and dollar detail
- `cco cost agents` — spending by model tier (opus / sonnet / haiku)
- `cco cost cache` — cache hit-rate, savings estimate, and detection of silent-buster threads (cache writes that are never read)
- `cco cost gate` — CI-friendly budget guard; exits 1 when spend exceeds a threshold; reads `daily_budget_usd` or `weekly_budget_usd` from config if no `--max-usd` flag is provided
- Optional soft budget-gate hook that fires a non-blocking reminder when today's spend approaches your daily limit
- Refreshable pricing loaded at runtime from `~/.cco/pricing.json` (no rebuild needed); supports all current tiers including Claude Fable 5 / Mythos 5
- RTK token savings reported inline: tokens compressed by RTK are converted to implied USD using your actual blended input rate

### Multi-Agent Orchestration and Delegation

- 117 compiled-in agent definitions across three model tiers:
  - **haiku** — 81 agents: language specialists, documentation, utilities, research
  - **sonnet** — 35 agents: managers, reviewers, security, QA, DevOps, architects
  - **opus** — 1 agent: Chief Architect
- Agent definitions live in source at `src/agents/*.md` (YAML frontmatter + prompt body) and are compiled into the binary at build time — no external files required at runtime
- The **delegation nudge** is a soft PostToolUse hook that reminds the orchestrator to push implementation work to a lower-cost tier rather than doing it inline; non-blocking and rate-limited so it does not interrupt your session

### MCP Server and Control Plane

- `cco mcp serve` — starts an MCP server (stdio JSON-RPC transport) registerable in Claude Code, Claude Desktop, or Cursor
- 23 tools across five categories: knowledge and graph search, session management, task DAG, analytics, and cost intelligence
- **Cost tools**: `cost_summary`, `budget_status`, `recommend_config` — query metrics without the daemon being online
- **Control plane** (Phase B3): `control_list_agents`, `control_spawn_agent`, `control_agent_status`, `control_agent_output` — lets a parent Claude instance observe and direct CCO's agent registry and task DAG
- Task DAG: `cco tasks` — create, list, and check status of structured work items
- Knowledge graph: `cco graph` — search, traverse, and add relationship nodes
- Code index: `cco index` — scan workspace, search symbols, show stats
- Event bus: `cco events` — list, count, and prune orchestration events
- Daemon communicates over a Unix domain socket at `~/.cco/daemon.sock` (no TCP port required; 0700 permissions, owner-only)

### Integrations

- **RTK (Rust Token Killer)** — optional shim that compresses Bash output 60–90% before it reaches Claude, reducing token consumption; bundled by default and controlled by `rtk_enabled` in config; RTK's own telemetry is disabled automatically
- **Statusline** — CCO injects a `statusLine` into Claude Code settings showing live model, effort level, directory, git branch, context tokens, cost, and rate limits; customizable via `statusline_command` or disabled via `statusline_enabled = false`
- **Claude Code Remote Control** — enabled by default (OAuth sessions only; skipped automatically when `ANTHROPIC_API_KEY` is set); connects claude.ai/code and the Claude mobile app to the local session; session named from the current working directory by default

### TUI

- `cco tui` — terminal dashboard with cost totals, cache-efficiency view, and delegation-nudge metrics
- Refreshes continuously from the local DuckDB store; no network required
- Three panels: Cost, Cache, Delegation; navigate with arrow keys or Tab

---

## Install

### Shell Script (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/install.sh | bash
```

This is the canonical install path on macOS and Linux. The binary lands in `~/.local/bin/cco`, and `cco update` replaces it in place.

### Manual Download

Download the latest signed release archive for your platform from [GitHub Releases](https://github.com/visiquate/cco/releases/latest), then extract and install:

```bash
# macOS Apple Silicon
curl -fsSL https://github.com/visiquate/cco/releases/latest/download/cco-aarch64-apple-darwin.tar.gz | tar xz
mkdir -p ~/.local/bin && mv cco ~/.local/bin/cco

# Linux x86_64
curl -fsSL https://github.com/visiquate/cco/releases/latest/download/cco-x86_64-unknown-linux-gnu.tar.gz | tar xz
mkdir -p ~/.local/bin && mv cco ~/.local/bin/cco
```

For macOS Intel or Linux ARM64, build from source (see below).

### Build from Source

Requires Rust 1.70+ and Claude Code.

```bash
git clone https://github.com/visiquate/cco.git
cd cco
cargo build --release
# or
make install    # builds release and copies to ~/.local/bin
```

### Canonical Installation Paths

All installers target the same directories so `cco update` can safely replace the binary in place:

- **macOS / Linux:** `~/.local/bin/cco`
- **Windows:** `%ProgramFiles%\CCO\cco.exe`

If `which cco` returns `/usr/local/bin/cco`, delete that legacy copy and reinstall so the canonical location takes precedence:

```bash
sudo rm -f /usr/local/bin/cco
curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/install.sh | bash
which cco
```

### Verify the Installation

After placing the binary on your PATH, run once to verify and repair any issues:

```bash
cco doctor --fix
```

`cco doctor --fix` checks that Claude Code is present, the daemon socket is reachable, hooks are registered, and RTK is configured; it repairs common issues automatically.

---

## Quickstart

```bash
# Start the background daemon (manages the DuckDB store and Unix socket)
cco daemon start

# Launch Claude Code through CCO (applies hooks, statusline, Remote Control)
cco

# Open the TUI cost/cache dashboard in a second terminal
cco tui

# Summarize all-time costs
cco cost dashboard

# Show cache efficiency for the current week
cco cost cache --period week

# Check whether today's spend is under a threshold (exits 1 if over)
cco cost gate --max-usd 10.00

# Run a full health check and auto-fix common issues
cco doctor --fix
```

---

## Configuration

CCO reads `~/.cco/config.toml`. All keys are optional; omitting a key uses the documented default.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `rtk_enabled` | bool | `true` | Enable RTK Bash-output compression when `rtk` is on PATH |
| `daily_budget_usd` | float | — | Daily spend ceiling; triggers soft budget-gate warnings |
| `weekly_budget_usd` | float | — | Weekly spend ceiling; used by `cco cost gate --period week` |
| `telemetry_enabled` | bool | `true` | Upload aggregate token/cost metrics (no prompt text) |
| `telemetry_upload_transcripts` | bool | `false` | Upload full prompt/response transcripts (explicit opt-in) |
| `statusline_enabled` | bool | `true` | Inject CCO statusline into Claude Code settings |
| `statusline_command` | string | — | Custom command for the statusline; overrides the default script |
| `remote_control_enabled` | bool | `true` | Enable Claude Code Remote Control (OAuth sessions only) |
| `session_name` | string | — | Override the session label shown in the TUI and control plane |

Example `~/.cco/config.toml`:

```toml
daily_budget_usd = 5.00
weekly_budget_usd = 25.00
rtk_enabled = true
telemetry_enabled = true
telemetry_upload_transcripts = false
statusline_enabled = true
remote_control_enabled = true
```

Environment variable overrides:

| Variable | Effect |
|----------|--------|
| `CCO_TELEMETRY_DISABLED=1` | Disable aggregate telemetry regardless of config |
| `CCO_TELEMETRY_TRANSCRIPTS=1` | Enable transcript upload regardless of config |
| `CCO_REMOTE_CONTROL=0` | Disable Remote Control regardless of config |
| `CCO_DAILY_BUDGET_USD` | Set daily budget ceiling without editing config |
| `CCO_CONFIG_PATH` | Override config file path |

---

## Telemetry and Privacy

Aggregate telemetry (token counts and cost totals — no prompt or response text) is on by default in official builds. To opt out, set `telemetry_enabled = false` in `~/.cco/config.toml` or export `CCO_TELEMETRY_DISABLED=1`.

Transcript upload is off by default and must be explicitly opted in. Administrators can enforce org-wide telemetry policy via an MDM-managed config file that users cannot override.

See [docs/telemetry.md](docs/telemetry.md) for the full data schema, retention policy, MDM lockdown instructions, and opt-out details.

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/cost-metrics.md](docs/cost-metrics.md) | DuckDB store, token parsing, dual-parser design, cache efficiency, budget gate, and RTK savings |
| [docs/telemetry.md](docs/telemetry.md) | Data collected, retention, opt-out, and MDM lockdown |
| [docs/mcp-and-control-plane.md](docs/mcp-and-control-plane.md) | MCP server tools, task DAG, knowledge graph, and control-plane API |
| [docs/integrations.md](docs/integrations.md) | RTK setup, statusline customization, Remote Control, editor surfaces, and `cco doctor` |

---

## License

MIT — see [LICENSE](LICENSE).
