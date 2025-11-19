# CCO Commands Reference

Complete reference for all CCO commands.

## Global Options

These options apply to all commands:

```bash
cco [OPTIONS] <COMMAND>

Options:
  -v, --verbose          Increase logging verbosity
  -q, --quiet            Decrease logging verbosity
  --config <FILE>        Use alternative config file
  --help                 Display help information
  --version              Display version information
```

## Core Commands

### `cco init`

Initialize CCO configuration in the current directory or globally.

```bash
cco init [OPTIONS]

Options:
  --global               Initialize global configuration
  --api-key <KEY>        Set API key during initialization
  --model <MODEL>        Set default model (opus/sonnet/haiku)
```

**Examples:**

```bash
# Initialize in current project
cco init

# Initialize globally with API key
cco init --global --api-key sk-ant-...

# Initialize with custom model
cco init --model sonnet
```

### `cco config`

Manage CCO configuration.

```bash
cco config <SUBCOMMAND>

Subcommands:
  show                   Display current configuration
  set <KEY> <VALUE>      Set configuration value
  get <KEY>              Get configuration value
  unset <KEY>            Remove configuration value
  list                   List all configuration keys
```

**Examples:**

```bash
# Show current config
cco config show

# Set API key
cco config set api_key sk-ant-...

# Get default model
cco config get default_model

# List all config keys
cco config list
```

### `cco agent`

Manage and interact with agents.

```bash
cco agent <SUBCOMMAND>

Subcommands:
  list                   List all available agents
  info <AGENT>           Show detailed agent information
  spawn <AGENT>          Spawn a specific agent
  status                 Show agent status and health
```

**Examples:**

```bash
# List all agents
cco agent list

# Get info about Python Specialist
cco agent info python-specialist

# Spawn DevOps Engineer
cco agent spawn devops-engineer

# Check agent system status
cco agent status
```

### `cco daemon`

Control the CCO background daemon.

```bash
cco daemon <SUBCOMMAND>

Subcommands:
  start                  Start the daemon
  stop                   Stop the daemon
  restart                Restart the daemon
  status                 Check daemon status
  logs                   View daemon logs
```

**Examples:**

```bash
# Start daemon
cco daemon start

# Check if running
cco daemon status

# View recent logs
cco daemon logs --tail 50

# Stop daemon
cco daemon stop
```

### `cco tui`

Launch the Terminal User Interface for interactive monitoring.

```bash
cco tui [OPTIONS]

Options:
  --refresh <MS>         Set refresh rate in milliseconds (default: 1000)
  --theme <THEME>        Set color theme (dark/light)
```

**Examples:**

```bash
# Launch TUI
cco tui

# Launch with faster refresh
cco tui --refresh 500

# Launch with light theme
cco tui --theme light
```

### `cco metrics`

View and manage metrics data.

```bash
cco metrics <SUBCOMMAND>

Subcommands:
  show                   Display current metrics
  export <FILE>          Export metrics to file
  clear                  Clear metrics data
  report                 Generate metrics report
```

**Examples:**

```bash
# Show current metrics
cco metrics show

# Export to JSON
cco metrics export metrics.json

# Generate detailed report
cco metrics report
```

## Workflow Commands

### `cco orchestrate`

Orchestrate a development task with multi-agent coordination.

```bash
cco orchestrate [OPTIONS] "<TASK>"

Options:
  --agents <AGENTS>      Comma-separated list of agents to use
  --model <MODEL>        Override default model
  --parallel             Enable parallel execution
  --no-memory            Disable knowledge manager
```

**Examples:**

```bash
# Simple orchestration
cco orchestrate "Build a REST API with authentication"

# With specific agents
cco orchestrate --agents "python-specialist,security-auditor" "Add JWT auth"

# Parallel execution
cco orchestrate --parallel "Create mobile app and backend"
```

## Utility Commands

### `cco knowledge`

Manage the knowledge manager database.

```bash
cco knowledge <SUBCOMMAND>

Subcommands:
  search <QUERY>         Search knowledge base
  store <TEXT>           Store knowledge
  list                   List all knowledge entries
  stats                  Show knowledge base statistics
  prune                  Remove old knowledge entries
```

**Examples:**

```bash
# Search knowledge
cco knowledge search "authentication"

# Store decision
cco knowledge store "Decision: Use JWT for auth"

# View statistics
cco knowledge stats
```

### `cco credentials`

Manage secure credentials.

```bash
cco credentials <SUBCOMMAND>

Subcommands:
  store <KEY> <VALUE>    Store a credential
  retrieve <KEY>         Retrieve a credential
  list                   List all credential keys
  delete <KEY>           Delete a credential
  rotate <KEY>           Rotate a credential
```

**Examples:**

```bash
# Store API key
cco credentials store github_token ghp_...

# Retrieve credential
cco credentials retrieve github_token

# List all keys
cco credentials list

# Delete credential
cco credentials delete old_key
```

### `cco update`

Manage CCO updates.

```bash
cco update [OPTIONS]

Options:
  --check                Check for updates without installing
  --auto                 Enable automatic updates
  --no-auto              Disable automatic updates
```

**Examples:**

```bash
# Check for updates
cco update --check

# Enable automatic updates
cco update --auto

# Update now
cco update
```

## Advanced Commands

### `cco hook`

Manage lifecycle hooks.

```bash
cco hook <SUBCOMMAND>

Subcommands:
  list                   List all registered hooks
  add <HOOK> <SCRIPT>    Add a hook
  remove <HOOK>          Remove a hook
  test <HOOK>            Test hook execution
```

**Examples:**

```bash
# List hooks
cco hook list

# Add pre-compaction hook
cco hook add pre-compaction ./scripts/save-state.sh

# Test hook
cco hook test pre-compaction
```

### `cco debug`

Debugging and diagnostics tools.

```bash
cco debug <SUBCOMMAND>

Subcommands:
  info                   System diagnostic information
  logs                   View debug logs
  trace <COMMAND>        Trace command execution
  validate               Validate installation
```

**Examples:**

```bash
# Get diagnostic info
cco debug info

# View debug logs
cco debug logs --level debug

# Trace a command
cco debug trace orchestrate "test task"

# Validate installation
cco debug validate
```

## Environment Variables

CCO respects these environment variables:

- `CCO_CONFIG_PATH` - Override config file location
- `CCO_LOG_LEVEL` - Set logging level (trace/debug/info/warn/error)
- `CCO_API_KEY` - Set API key (overrides config)
- `CCO_NO_COLOR` - Disable colored output
- `CCO_CACHE_DIR` - Override cache directory

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - API error
- `4` - Agent error
- `5` - Daemon error

## Getting Help

```bash
# General help
cco --help

# Command-specific help
cco <command> --help

# Subcommand help
cco <command> <subcommand> --help
```

## Next Steps

- [Configuration Guide](./configuration.md)
- [Feature Overview](./features.md)
- [Troubleshooting](./troubleshooting.md)
