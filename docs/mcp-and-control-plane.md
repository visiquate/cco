# MCP Server and Control Plane

## Overview

CCO exposes a **Model Context Protocol (MCP) server** that allows Claude-based tools and clients (Claude Code, Claude Desktop, Cursor) to interact with the CCO daemon and agent orchestration system over stdio JSON-RPC. The server implements **23 tools** organized into knowledge, task management, cost analysis, and control-plane categories.

The **control plane** (Phase B3) is a subset of MCP tools that let a parent Claude agent observe and direct CCO's own 117-agent registry and task DAG — enabling meta-orchestration where Claude can spawn tracked work, query agent status, and retrieve performance metrics.

---

## Architecture

### Handshake and Security

1. **Stdio transport**: The MCP server runs as a subprocess listening on stdin/stdout in JSON-RPC format.
2. **Initialize gate**: All tool calls (except `initialize` and `notifications/initialized`) are rejected with a `-32002` error until the client sends an `initialize` request.
3. **Unix socket backend**: Tool handlers communicate with the CCO daemon over a Unix domain socket at `~/.cco/daemon.sock` (0700 permissions, owner-only access).
4. **No additional auth**: The socket permissions enforce local-process-only access; no API key or token is required at the MCP layer.

### Control Flow

```
Claude (Client)
    |
MCP Server (stdio JSON-RPC)
    |
Tool Handler (e.g., cost_summary, control_spawn_agent)
    |
Unix Socket -> Daemon (HTTP API over socket)
    |
Task Store, Agent Registry, Metrics DB, etc.
```

---

## Tools (23 Total)

### Knowledge and Graph (6 tools)

- **`knowledge_search`** — Full-text search over stored documents.
- **`knowledge_store`** — Index a new document in the knowledge base.
- **`chunk_search`** — Search for relevant chunks by embedding similarity.
- **`knowledge_recall`** — Retrieve documents by ID or metadata tags.
- **`knowledge_lifecycle_status`** — Check indexing status and storage metrics.
- **`knowledge_graph_query`** — Query relationships in the knowledge graph.

### Context and Agents (2 tools)

- **`get_context`** — Fetch agent execution context and environment.
- **`list_agents`** — Enumerate all 117 registered agents with their metadata.

### Session Management (2 tools)

- **`session_start`** — Initialize a new session with Claude.
- **`pre_compaction`** — Prepare the session for context compression.

### Task DAG (3 tools)

- **`task_create`** — Spawn a new task node in the DAG with dependencies.
- **`task_status`** — Query the status of a task by ID.
- **`task_list`** — List all tasks, optionally filtered by project and status.

### Analytics and Monitoring (3 tools)

- **`agent_performance_stats`** — Retrieve success rate, duration, and token metrics for an agent.
- **`codex_status`** — Check code index status and search capability.
- **`code_search`** — Full-text search over indexed source code.

### Cost Intelligence (3 tools)

- **`cost_summary`** — Aggregated spend totals, per-tier breakdown (opus/sonnet/haiku), cache hit-rate, and token counts for a time period (`today`, `week`, `month`, `all`, or `7d`).
- **`budget_status`** — Check today's and this week's spend against configured daily/weekly budgets.
- **`recommend_config`** — Suggest a Claude model tier and effort level for a natural-language task description based on the embedded agent registry.

### Control Plane (4 tools) — Phase B3

- **`control_list_agents`** — List all 117 registered agents with live task-DAG counts per agent and status breakdown.
- **`control_spawn_agent`** — Create a queued task-DAG node for a named agent. Returns a task ID for polling.
- **`control_agent_status`** — Fetch the full task-DAG node for a task ID (status, dependencies, result/error payloads).
- **`control_agent_output`** — Get recent task-DAG rows and historical performance metrics for a named agent.

---

## Control Plane Deep Dive

### What It Does

The control plane exposes CCO's own orchestration objects — the 117-agent registry and the task DAG — so that a parent Claude instance can:

- **List agents** with live task counts and capabilities.
- **Spawn tracked work** as a task-DAG node assigned to a named agent.
- **Poll task status** and retrieve result/error payloads.
- **Query agent performance** (success rate, avg duration, trust score).

### What It Does NOT Do (Current Limitation)

**CCO has no daemon-side agent execution engine.** The daemon task-DAG scheduler transitions tasks to `in_progress` when dependencies are satisfied, but it does **not spawn a subprocess**. Real work is always performed by an orchestrator Claude instance via its Task tool.

Therefore:

- `control_spawn_agent` creates a *queued work request* (task node) that the orchestrator must discover and dispatch.
- `send_to_agent` and `interrupt_agent` are deliberately omitted — there is no IPC channel to a running agent subprocess.
- The orchestrator uses `task_list` (filtered by `project_id`) to discover ready/pending tasks created by `control_spawn_agent`, then dispatches them using the standard Task tool.

### Control Plane Endpoints (Daemon Side)

| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| GET | `/api/control/agents` | `list_agents` | Registry + live task-DAG counts |
| POST | `/api/control/spawn` | `spawn_agent` | Create a queued task node |
| GET | `/api/control/status/:id` | `agent_status` | Full task-DAG node for a task ID |
| GET | `/api/control/output/:agent` | `agent_output` | Task-DAG rows + performance metrics for an agent |

All routes are served over the Unix domain socket and require no additional authentication.

---

## Registering the MCP Server

### Starting the Server

The CCO binary exposes the MCP server via the `cco mcp serve` command:

```bash
cco mcp serve [--server-name NAME] [--server-version VERSION]
```

The server reads JSON-RPC requests from stdin and writes responses to stdout (no buffering).

### Client Configuration

Clients register the MCP server in their configuration files. The server is launched as a subprocess per-session.

#### Claude Desktop (`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "cco": {
      "command": "/path/to/cco",
      "args": ["mcp", "serve"]
    }
  }
}
```

Location: `~/.claude/claude_desktop_config.json` (macOS/Linux) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows).

#### Claude Code (`.claude/mcp.json`)

```json
{
  "mcpServers": {
    "cco": {
      "command": "/path/to/cco",
      "args": ["mcp", "serve"]
    }
  }
}
```

Location: `~/.claude/mcp.json` in each Claude Code workspace.

#### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "cco": {
      "command": "/path/to/cco",
      "args": ["mcp", "serve"]
    }
  }
}
```

Location: `.cursor/mcp.json` in the project root.

---

## Example Payloads

### Cost Summary Tool

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "cost_summary",
    "arguments": {
      "period": "7d"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Cost Summary (7d)\n\nTotal: $12.3456  |  Tokens: 1250000  |  Cache hit-rate: 65.2%\n\nPer-tier breakdown:\n  opus    : $5.1234  (41.5%)  500000 tokens\n  sonnet  : $4.5678  (37.0%)  450000 tokens\n  haiku   : $2.6544  (21.5%)  300000 tokens\n"
      }
    ],
    "data": {
      "period": "week",
      "total_cost_usd": 12.3456,
      "total_tokens": 1250000,
      "cache_hit_rate": 0.652,
      "by_tier": [
        { "tier": "opus",   "cost_usd": 5.1234, "tokens": 500000 },
        { "tier": "sonnet", "cost_usd": 4.5678, "tokens": 450000 },
        { "tier": "haiku",  "cost_usd": 2.6544, "tokens": 300000 }
      ]
    }
  }
}
```

### Control Spawn Agent Tool

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "control_spawn_agent",
    "arguments": {
      "agent": "rust-specialist",
      "prompt": "Implement a safe async ring buffer with lock-free pop/push semantics",
      "model": "sonnet",
      "project_id": "buffer-project",
      "dependencies": []
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "agent": "rust-specialist",
    "model_tier": "sonnet",
    "status": "pending",
    "project_id": "buffer-project",
    "created_at": "2026-06-14T10:30:45.123Z",
    "next_steps": "Poll status: GET /api/control/status/550e8400-e29b-41d4-a716-446655440000. Orchestrator must dispatch via Task tool with agent='rust-specialist' model='sonnet'."
  }
}
```

### Control Agent Status Tool

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "control_agent_status",
    "arguments": {
      "id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**Response (when completed):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "agent": "rust-specialist",
    "started_at": "2026-06-14T10:31:00.000Z",
    "completed_at": "2026-06-14T10:45:30.567Z",
    "result": {
      "code": "pub struct RingBuffer<T> { ... }",
      "tests_passed": 42,
      "compilation_warnings": 0
    }
  }
}
```

---

## Integration with Other Systems

### Task DAG Integration

The control plane and standard task engine share the same task-DAG store. A task spawned with `control_spawn_agent` can be monitored with `task_status` and `task_list`. Conversely, any task in the DAG appears in `control_agent_output` when scoped to the agent.

### Cost Tracking

Cost tools (`cost_summary`, `budget_status`, `recommend_config`) do not depend on the daemon or task DAG; they query the metrics database directly. They remain available even if the task store is offline.

### Agent Registry

The 117-agent registry is compiled into the binary at build time (from `src/agents/*.md`) and is always available — even if the daemon is not running. `control_list_agents` falls back to the embedded registry if the daemon is unreachable, though it then returns an empty `task_counts` object.

---

## Troubleshooting

### "Server not initialized: send 'initialize' before calling methods"

The client tried to call a tool without sending an `initialize` request first. All tool calls require a completed handshake.

### "Daemon unavailable — registry served from embedded binary only"

The MCP server could not reach the daemon on the Unix socket. Possible causes:

- The daemon is not running. Start it with `cco daemon start`.
- The socket file `~/.cco/daemon.sock` was deleted. Restart the daemon.
- Permissions issue. Check that `~/.cco/daemon.sock` is readable by your user (should be 0700, owner-only).

Tool calls will still work for read-only operations (e.g., `list_agents` returns the embedded registry), but writes and live task-DAG data are unavailable.

### "Task store not available"

The daemon is running but the task-DAG store failed to initialize. Check daemon logs: `cco daemon logs --follow`.

---

## Security Notes

1. **Unix socket only**: The daemon listens exclusively on `~/.cco/daemon.sock` (0700), not on a network port. Only local processes with file-system access can communicate.
2. **No credentials in MCP**: The MCP layer does not validate API keys or tokens. The socket permissions are the security boundary.
3. **Audit trail**: All tool calls are logged to the daemon and can be audited via `cco daemon logs`.
4. **Read-only registry**: The agent registry is immutable at runtime (compiled at build time). New agents require a code change and recompilation.

## See Also

- [Cost Metrics and Budget Tracking](cost-metrics.md) — details on cost summary, budget alerts, and per-tier pricing
- [Integrations](integrations.md) — MCP client setup, editor surfaces, and RTK
