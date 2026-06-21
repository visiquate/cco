# CCO Feature Catalog

Every CCO capability in two sentences, with a link to its deep dive.

---

## Multi-Agent Orchestration

CCO ships 117 compiled-in agent definitions across three model tiers (haiku, sonnet,
opus), covering language specialists, managers, reviewers, security, DevOps, and the
Chief Architect. A soft delegation-nudge hook reminds the orchestrator to push
implementation work to a lower-cost tier rather than doing it inline, and model tiers
are fully remappable via config presets or per-row overrides.

-> Deep dive: [Agent Roster and Delegation (AGENTS.md)](../AGENTS.md)

---

## Cortex: Local-First Self-Improving Memory

Cortex is enabled by default and runs the full self-improving loop automatically on your
machine: capture, consolidation, and recall all use a local store at
`~/.cco/cortex/<repo-id>/`. Share deliberately via `cco cortex export` to write
memories into the repo's `.cco/cortex/memories/` for git tracking and team access.

-> Deep dive: [Cortex Deep Dive](cortex.md)

---

## Cost and Cache Intelligence

CCO parses every `~/.claude/projects/*.jsonl` transcript into a local DuckDB store,
providing per-session, per-agent, and per-period cost breakdowns with no external
service required. The cache efficiency report shows hit-rates, savings, and silent-
buster threads, and `cco cost gate` provides a CI-friendly budget guard that exits 1
when spend exceeds a configurable threshold.

-> Deep dive: [Cost Metrics and DuckDB Store](cost-metrics.md)  
-> See also: [Cost Analysis Guide](COST_ANALYSIS.md)

---

## MCP Server and Control Plane

`cco mcp serve` starts an MCP server (stdio transport) that exposes task-DAG
management, knowledge-graph queries, code-index search, cost tools, and a session
control plane to any MCP client. The daemon communicates over a Unix domain socket at
`~/.cco/daemon.sock`, and the task DAG, typed knowledge graph, code index, and event
bus are all accessible via dedicated `cco tasks`, `cco graph`, `cco index`, and
`cco events` subcommands.

-> Deep dive: [MCP Server and Control Plane](mcp-and-control-plane.md)  
-> Quickstart: [MCP Quick Start](../MCP_QUICK_START.md)

---

## Tool Integrations

CCO integrates with RTK (Rust Token Killer) for Bash-output compression, lean-ctx for
semantic context compression, Headroom for API-traffic caching, Ponytail for anti-bloat
prompt rules, Codex CLI (with OTEL telemetry wired to CCO's receiver), and Claude Code
Remote Control for connecting claude.ai/code and the mobile app to a local session.
`cco optimize` detects which tools are installed and enables them in one step; `cco
integrate` registers CCO's MCP server in editor surfaces.

-> Deep dive: [Tool Integrations](integrations.md)

---

## TUI Cockpit

`cco tui` is a full-terminal monitoring dashboard with four pages: Live (in-flight
subagents, fleet roll-up, filtered events), Spend (cost and sparklines), Cache
(efficiency metrics), and Delegation (nudge analysis). It provides a directory-
independent global view with a project scope picker, a live SSE event stream, and an
automatic polling fallback when SSE is unavailable.

-> Deep dive: [TUI Guide](tui.md)

---

## Auto-Update

`cco update` checks for and installs new releases from GitHub, with a `--check` flag to
report without installing and a `--channel` flag to select stable or beta. The daemon
can be configured to notify or auto-update on a schedule, and the full update lifecycle
is documented including signature verification and rollback.

-> Deep dive: [Auto-Update User Guide](AUTO_UPDATE_USER_GUIDE.md)

---

## Hooks

CCO registers Claude Code lifecycle hooks (SessionStart, PreCompact, PostToolUse,
SubagentStart, SubagentStop, Stop, PermissionRequest) to power cost tracking,
delegation nudges, Cortex capture, and budget-gate warnings. The `cco hook` subcommands
let you list, add, remove, and test hooks, and the guide documents the full hook API
and how to write custom hooks.

-> Deep dive: [Hooks User Guide](HOOKS_USER_GUIDE.md)

---

## Secure Credentials

`cco credentials` stores and retrieves secrets via the OS keyring (Keychain on macOS)
with AES-256-GCM encryption, keeping API keys and tokens out of config files and
environment variables. The migration guide covers moving from legacy credential storage
to the new secure store and verifying rotation status.

-> Deep dive: [Credential Migration Guide](CREDENTIAL_MIGRATION.md)

---

## Cross-Repository Usage

CCO supports working across multiple repositories by resolving project boundaries at the
daemon level, allowing cost tracking and knowledge-graph queries to span worktrees and
monorepos. The cross-repo guide covers setup, project-ID resolution, and the boundaries
that Cortex and the knowledge graph respect.

-> Deep dive: [Cross-Repository Usage](CROSS_REPO_USAGE.md)

---

## Autonomous Workflow and Extended Operation

When `agent_execution_enabled = true`, the daemon's task-DAG scheduler spawns `claude
-p` subprocesses autonomously to execute ready tasks, with configurable concurrency,
timeout, and permission-mode controls. The autonomous workflow guide covers safe
configuration, the `bypassPermissions` opt-in, and patterns for long-running multi-
agent pipelines.

-> Deep dive: [Autonomous Workflow Guide](AUTONOMOUS_WORKFLOW_GUIDE.md)

---

## Telemetry and Privacy

Aggregate telemetry (token counts and cost totals - no prompt or response text) is on
by default on official builds and can be disabled via `telemetry_enabled = false` or
`CCO_TELEMETRY_DISABLED=1`. Transcript upload is off by default and requires explicit
opt-in; MDM-managed config allows fleet-wide policy enforcement.

-> Deep dive: [Telemetry and Privacy](telemetry.md)

---

## Daemon and CLI

The CCO daemon runs as a background service (launchd on macOS), listens on a Unix
domain socket at `~/.cco/daemon.sock`, and manages the DuckDB store, task scheduler,
orchestration API, and all hook dispatch. The `cco daemon` subcommands cover start,
stop, restart, status, logs, and service installation.

-> Deep dive: [Daemon and CLI Commands](../DAEMON_CLI_COMMANDS.md)

---

## Agent Selection Guidance

The agent selection guide explains which of CCO's 117 agents to use for a given task,
covering the haiku, sonnet, and opus tier specializations, how the Chief Architect
delegates, and how to write task descriptions that route to the right agent. It includes
decision trees for common scenarios such as code review, security audit, and
documentation.

-> Deep dive: [Agent Selection Guide](AGENT_SELECTION_GUIDE.md)
