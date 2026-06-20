# Daemon CLI Commands for CCO

This document describes the new CLI commands for interacting with the CCO daemon without needing to construct HTTP calls manually.

## Overview

All daemon commands are available through the `cco` binary and communicate with the running daemon via HTTP on the automatically-discovered port. The daemon must be running for these commands to work.

Start the daemon with:
```bash
cco daemon start
```

## Commands

### Agents Management (`cco agents`)

List and query agent information from the daemon.

#### List all agents
```bash
cco agents list
```

Output (human-readable):
```
📋 Agents (117 total)

Opus  (1)
  - chief-architect: Strategic decision-making and orchestration

Sonnet  (35)
  - python-specialist: Python language expert
  - rust-specialist: Rust systems programming expert
  - ...

Haiku  (81)
  - swift-specialist: Swift mobile development
  - ...
```

#### List agents filtered by model
```bash
cco agents list --model sonnet
cco agents list --model haiku
cco agents list --model opus
```

#### Show detailed agent information
```bash
cco agents info chief-architect
```

Output (human-readable):
```
🤖 Agent: chief-architect

  Model: opus-4-20250805
  Description: Strategic decision-making and orchestration
  Mode: subagent
  Temperature: 0.1
  Tools:
    ✓ read
    ✓ write
    ✗ bash
    ✗ edit
  Context: Leadership coordinator
```

#### JSON output
```bash
cco agents list --format json
cco agents info chief-architect --format json
```

### Context Management (`cco context`)

Get context information for specific agents.

#### Get agent context
```bash
cco context get --agent python-specialist
```

Output (human-readable):
```
📦 Context for Agent: python-specialist

System Prompt:
  You are an expert Python developer...

Task Description:
  Implement the requested Python feature

Previous Context:
  1. Established Python best practices
  2. Set up type hints and testing framework
  3. ...

Parameters:
  version: 3.11
  framework: FastAPI
  test_framework: pytest
```

#### JSON output
```bash
cco context get --agent python-specialist --format json
```

## Implementation Details

### Shared Client Module (`src/commands/client.rs`)

All daemon commands use a shared HTTP client that:

1. **Discovers the daemon port**: Automatically reads from the PID file at `~/.cco/daemon.pid`
2. **Handles errors gracefully**: Provides helpful error messages if the daemon isn't running
3. **Formats output**: Supports both JSON and human-readable formats
4. **Manages connections**: Creates HTTP clients with appropriate timeouts and connection pooling

Key functions:
- `get_daemon_url()` - Returns the daemon's HTTP URL
- `daemon_get(path)` - Makes GET requests to the daemon
- `daemon_post(path, body)` - Makes POST requests to the daemon
- `format_output(data, format)` - Formats responses for display

### Command Modules

Each command has its own module in `src/commands/`:

- **agents.rs** - Agent listing and information queries
- **context.rs** - Agent context retrieval

### Main.rs Integration

New command handlers in `main.rs`:

```rust
Commands::Agents { action } => {
    tracing_subscriber::fmt::init();
    commands::agents::run(action).await
}

Commands::Context { action } => {
    tracing_subscriber::fmt::init();
    commands::context::run(action).await
}
```

## Error Handling

All commands provide helpful error messages:

```bash
$ cco agents list
❌ Daemon not running. Start it with: cco daemon start
```

Connection timeouts and other errors are clearly reported with suggestions for resolution.

## Response Formats

### Human-Readable Format (default)

Optimized for human consumption with:
- Emojis for visual scanning
- Organized sections and indentation
- Summary statistics
- Helpful context information

### JSON Format

Machine-readable output suitable for:
- Parsing in scripts
- Integration with other tools
- Programmatic processing

Use `--format json` flag with any command.

## Examples

### Get agent list filtered by model and export to file
```bash
cco agents list --model sonnet --format json > sonnet-agents.json
```

### Get context for all agents in a project
```bash
for agent in chief-architect python-specialist rust-specialist; do
  echo "=== Context for $agent ==="
  cco context get --agent $agent --format json | jq .
done
```

### Monitor agent availability
```bash
#!/bin/bash
while true; do
  clear
  echo "Agent Status:"
  cco agents list --format json | jq '.agents | length'
  sleep 30
done
```

## Integration with Scripts

Commands return appropriate exit codes:
- 0: Success
- 1: Error (daemon not running, invalid arguments, API error)

Example shell script:
```bash
#!/bin/bash

# Check daemon is running
cco agents list > /dev/null 2>&1 || {
  echo "Starting daemon..."
  cco daemon start
  sleep 2
}

# Get agent info
agent_info=$(cco agents list --format json)
echo "Loaded $( echo $agent_info | jq '.agents | length') agents"
```

## Future Extensions

The infrastructure supports additional daemon operations:

1. **More agent operations**:
   - `cco agents stats` - Agent statistics
   - `cco agents search --skill "python"` - Find agents by skill
   - `cco agents validate` - Validate agent configuration

2. **More context operations**:
   - `cco context list` - List all available contexts
   - `cco context create` - Create new context for agents
   - `cco context delete` - Remove context

The client module's design makes adding these new commands straightforward.
