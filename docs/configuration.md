# CCO Configuration Guide

## Configuration File Locations

CCO looks for configuration files in the following order:

1. `./cco.toml` (current directory)
2. `~/.config/cco/config.toml` (user config)
3. `/etc/cco/config.toml` (system config)

## Configuration Format

CCO uses TOML format for configuration files.

### Basic Configuration

```toml
# ~/.config/cco/config.toml

# API Configuration
[api]
key = "sk-ant-..."              # Anthropic API key (required)
endpoint = "https://api.anthropic.com/v1"
timeout = 30                    # Request timeout in seconds

# Model Configuration
[models]
default = "sonnet"              # Default model: opus/sonnet/haiku
opus = "claude-opus-4.1"        # Opus model version
sonnet = "claude-sonnet-4.5"    # Sonnet model version
haiku = "claude-haiku-4.5"      # Haiku model version

# Agent Configuration
[agents]
max_concurrent = 10             # Max concurrent agents
default_timeout = 600           # Agent timeout in seconds
enable_memory = true            # Enable knowledge manager

# Daemon Configuration
[daemon]
enabled = true                  # Enable background daemon
port = 9736                     # Daemon listening port
log_level = "info"              # Logging level

# Update Configuration
[updates]
auto_check = true               # Automatically check for updates
auto_install = false            # Automatically install updates
check_interval = 86400          # Check every 24 hours (seconds)
```

## Configuration Options Reference

### `[api]` - API Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `key` | string | *required* | Anthropic API key |
| `endpoint` | string | `https://api.anthropic.com/v1` | API endpoint URL |
| `timeout` | integer | `30` | Request timeout in seconds |
| `max_retries` | integer | `3` | Maximum retry attempts |
| `retry_delay` | integer | `1000` | Delay between retries (ms) |

### `[models]` - Model Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `default` | string | `"sonnet"` | Default model tier |
| `opus` | string | `"claude-opus-4.1"` | Opus model ID |
| `sonnet` | string | `"claude-sonnet-4.5"` | Sonnet model ID |
| `haiku` | string | `"claude-haiku-4.5"` | Haiku model ID |
| `fallback` | boolean | `true` | Enable automatic fallback |

### `[agents]` - Agent Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_concurrent` | integer | `10` | Maximum concurrent agents |
| `default_timeout` | integer | `600` | Default agent timeout (seconds) |
| `enable_memory` | boolean | `true` | Enable knowledge manager |
| `memory_threshold` | float | `0.8` | Memory persistence threshold |
| `coordination_mode` | string | `"hierarchical"` | Agent coordination mode |

### `[daemon]` - Daemon Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable background daemon |
| `port` | integer | `9736` | Daemon listening port |
| `log_level` | string | `"info"` | Logging level |
| `pid_file` | string | `~/.local/run/cco.pid` | PID file location |
| `log_file` | string | `~/.local/log/cco.log` | Log file location |

### `[updates]` - Update Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `auto_check` | boolean | `true` | Automatically check for updates |
| `auto_install` | boolean | `false` | Automatically install updates |
| `check_interval` | integer | `86400` | Check interval (seconds) |
| `update_channel` | string | `"stable"` | Update channel: stable/beta |

### `[knowledge]` - Knowledge Manager

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable knowledge manager |
| `database_path` | string | `~/.local/share/cco/knowledge` | Database location |
| `embedding_dim` | integer | `384` | Embedding dimensions |
| `max_age_days` | integer | `90` | Knowledge retention period |
| `auto_cleanup` | boolean | `false` | Automatic cleanup |

### `[credentials]` - Credentials Manager

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `storage_path` | string | `~/.local/share/cco/credentials` | Credentials storage |
| `encryption` | string | `"aes-256-cbc"` | Encryption algorithm |
| `rotation_days` | integer | `90` | Credential rotation period |

### `[tui]` - Terminal UI Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `theme` | string | `"dark"` | Color theme: dark/light |
| `refresh_ms` | integer | `1000` | Refresh rate (milliseconds) |
| `show_metrics` | boolean | `true` | Display metrics panel |
| `show_logs` | boolean | `true` | Display logs panel |

## Environment Variables

Environment variables override configuration file settings:

| Variable | Description | Example |
|----------|-------------|---------|
| `CCO_API_KEY` | Anthropic API key | `sk-ant-...` |
| `CCO_CONFIG_PATH` | Config file path | `/custom/path/config.toml` |
| `CCO_LOG_LEVEL` | Logging level | `debug` |
| `CCO_DAEMON_PORT` | Daemon port | `9999` |
| `CCO_NO_UPDATE_CHECK` | Disable update checks | `true` |

## Configuration Management Commands

### View Configuration

```bash
# Show entire configuration
cco config show

# Show specific section
cco config show api

# Get specific value
cco config get api.key
```

### Set Configuration

```bash
# Set API key
cco config set api.key "sk-ant-..."

# Set default model
cco config set models.default sonnet

# Enable auto-updates
cco config set updates.auto_install true
```

### Validate Configuration

```bash
# Validate current configuration
cco debug validate

# Test API connection
cco config test api

# Verify agent system
cco agent status
```

## Example Configurations

### Minimal Configuration

```toml
[api]
key = "sk-ant-..."

[models]
default = "sonnet"
```

### Development Configuration

```toml
[api]
key = "sk-ant-..."
timeout = 60

[models]
default = "haiku"  # Faster, cheaper for dev

[daemon]
log_level = "debug"

[agents]
max_concurrent = 5
enable_memory = true

[updates]
auto_check = false
```

### Production Configuration

```toml
[api]
key = "sk-ant-..."
timeout = 30
max_retries = 5

[models]
default = "sonnet"
fallback = true

[daemon]
enabled = true
log_level = "info"

[agents]
max_concurrent = 20
default_timeout = 900
enable_memory = true

[knowledge]
enabled = true
max_age_days = 180
auto_cleanup = true

[updates]
auto_check = true
auto_install = true
update_channel = "stable"

[tui]
theme = "dark"
refresh_ms = 500
show_metrics = true
```

## Security Best Practices

### API Key Storage

Never commit API keys to version control:

```bash
# Add to .gitignore
echo "cco.toml" >> .gitignore
echo ".env" >> .gitignore

# Use environment variable instead
export CCO_API_KEY="sk-ant-..."
```

### File Permissions

Restrict config file permissions:

```bash
chmod 600 ~/.config/cco/config.toml
```

### Credentials Encryption

CCO automatically encrypts stored credentials using AES-256-CBC.

## Troubleshooting Configuration

### Configuration Not Found

```bash
# Check search paths
cco debug info | grep -A5 "Config"

# Initialize configuration
cco init --global
```

### Invalid Configuration

```bash
# Validate syntax
cco config validate

# Show errors
cco config show --verbose
```

### API Key Issues

```bash
# Test API connection
cco config test api

# Set key via command
cco config set api.key "sk-ant-..."

# Or via environment
export CCO_API_KEY="sk-ant-..."
```

## Next Steps

- [Commands Reference](./commands.md)
- [Feature Overview](./features.md)
- [Troubleshooting Guide](./troubleshooting.md)
