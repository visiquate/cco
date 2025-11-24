# CCO Configuration Reference

Complete configuration reference for CCO (Claude Code Orchestra).

## Table of Contents

- [Configuration File Location](#configuration-file-location)
- [Configuration Format](#configuration-format)
- [Configuration Options](#configuration-options)
  - [Proxy Settings](#proxy-settings)
  - [Cache Settings](#cache-settings)
  - [Routing Settings](#routing-settings)
  - [Analytics Settings](#analytics-settings)
  - [Update Settings](#update-settings)
  - [Security Settings](#security-settings)
- [Environment Variables](#environment-variables)
- [Command-Line Configuration](#command-line-configuration)
- [Configuration Examples](#configuration-examples)
- [Configuration Precedence](#configuration-precedence)

## Configuration File Location

CCO uses TOML format for configuration files.

**Default location**:
- **macOS/Linux**: `~/.config/cco/config.toml`
- **Windows**: `%USERPROFILE%\.config\cco\config.toml`

**Custom location**:
```bash
cco proxy --config /path/to/config.toml
```

**Initialize configuration**:
```bash
# Create default configuration file
cco config init

# View current configuration
cco config show

# Edit configuration
cco config edit
```

## Configuration Format

Configuration files use TOML syntax:

```toml
# Comments start with #

# Top-level settings
debug = false
log_level = "info"

# Nested sections use [section.subsection]
[proxy]
port = 8000
host = "127.0.0.1"

# Arrays use square brackets
[[routes]]
pattern = "^claude-"
provider = "anthropic"
```

## Configuration Options

### Proxy Settings

Configure the HTTP/HTTPS proxy server.

```toml
[proxy]
# Network binding
host = "127.0.0.1"        # Listen address (0.0.0.0 for all interfaces)
port = 8000                # Listen port
workers = 4                # Worker threads (default: CPU cores)

# Timeouts (in seconds)
request_timeout = 60       # Maximum request duration
connect_timeout = 10       # Connection timeout to upstream APIs
read_timeout = 30          # Read timeout for responses
write_timeout = 30         # Write timeout for requests

# Limits
max_connections = 1000     # Maximum concurrent connections
max_request_size = 10485760 # Maximum request body size (10 MB)
max_response_size = 104857600 # Maximum response size (100 MB)

# TLS/SSL (for HTTPS)
tls_enabled = false
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

# CORS (for web dashboard)
cors_enabled = true
cors_origins = ["http://localhost:3000"]
```

**Key Options**:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `host` | string | `127.0.0.1` | Bind address (`0.0.0.0` for all interfaces) |
| `port` | integer | `8000` | Port to listen on |
| `workers` | integer | CPU cores | Number of worker threads |
| `request_timeout` | integer | `60` | Request timeout in seconds |
| `max_connections` | integer | `1000` | Max concurrent connections |
| `tls_enabled` | boolean | `false` | Enable HTTPS |

### Cache Settings

Configure the in-memory cache (Moka).

```toml
[cache]
# Cache behavior
enabled = true             # Enable/disable caching
max_capacity = 1000        # Maximum number of cached entries
ttl_seconds = 3600         # Time-to-live for cached entries (1 hour)
idle_timeout = 1800        # Evict entries idle for 30 minutes

# Cache strategy
strategy = "lru"           # Cache eviction strategy: lru, lfu, fifo
cache_requests = true      # Cache request bodies
cache_responses = true     # Cache response bodies

# Cache key
include_headers = ["x-api-key"] # Headers to include in cache key
normalize_whitespace = true     # Normalize whitespace in requests

# Persistence (optional)
persist_enabled = false
persist_path = "~/.cache/cco/cache.db"
persist_interval = 300     # Save to disk every 5 minutes
```

**Cache Strategy Options**:
- `lru` (Least Recently Used): Default, good for most workloads
- `lfu` (Least Frequently Used): Better for workloads with hot entries
- `fifo` (First In First Out): Simple, predictable eviction

**Key Options**:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable caching |
| `max_capacity` | integer | `1000` | Max cache entries |
| `ttl_seconds` | integer | `3600` | Cache entry TTL |
| `strategy` | string | `lru` | Eviction strategy |

### Routing Settings

Configure multi-model routing to different providers.

```toml
# Define routes using [[routes]] array
[[routes]]
# Pattern matching (regex)
pattern = "^claude-"       # Match models starting with "claude-"
provider = "anthropic"     # Route to Anthropic
endpoint = "https://api.anthropic.com/v1"
timeout_ms = 60000

[[routes]]
pattern = "^gpt-"
provider = "openai"
endpoint = "https://api.openai.com/v1"
timeout_ms = 60000

[[routes]]
pattern = "^ollama/"
provider = "ollama"
endpoint = "http://localhost:11434"
timeout_ms = 120000

# Fallback chain (try alternatives if primary fails)
[[fallbacks]]
primary = "claude-opus-4"
fallbacks = ["claude-sonnet-3.5", "gpt-4"]
max_retries = 2

# Provider API keys (can also use environment variables)
[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"  # Read from environment
api_version = "2023-06-01"
max_tokens = 4096

[providers.openai]
api_key_env = "OPENAI_API_KEY"
organization = "org-xxxxx"
max_tokens = 4096

[providers.ollama]
# No API key needed for local Ollama
base_url = "http://localhost:11434"
```

**Key Options**:

| Option | Type | Description |
|--------|------|-------------|
| `pattern` | string (regex) | Model name pattern to match |
| `provider` | string | Provider name (anthropic, openai, ollama) |
| `endpoint` | string | API endpoint URL |
| `timeout_ms` | integer | Request timeout in milliseconds |

### Analytics Settings

Configure cost tracking and analytics.

```toml
[analytics]
# Database
enabled = true
db_path = "~/.config/cco/analytics.db"
retention_days = 90        # Keep data for 90 days

# Tracking
track_costs = true         # Track API costs
track_tokens = true        # Track token usage
track_latency = true       # Track request latency
track_cache_hits = true    # Track cache performance

# Aggregation
aggregate_interval = 300   # Aggregate stats every 5 minutes
flush_interval = 60        # Flush to disk every minute

# Export
export_format = "json"     # json, csv, parquet
export_path = "~/.config/cco/exports"
```

**Key Options**:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable analytics |
| `db_path` | string | `~/.config/cco/analytics.db` | SQLite database path |
| `retention_days` | integer | `90` | Data retention period |
| `track_costs` | boolean | `true` | Track API costs |

### Update Settings

Configure automatic update behavior.

```toml
[updates]
# Update checking
enabled = true             # Enable update checks
check_interval = "daily"   # daily, weekly, never
channel = "stable"         # stable, beta, nightly
last_check = "2025-11-15T10:00:00Z" # Last check timestamp (auto-updated)

# Installation
auto_install = false       # Automatically install updates
notify_on_update = true    # Show notification when update available
verify_signatures = true   # Verify binary signatures (future)

# Rollback
keep_backups = true        # Keep backup of previous version
backup_count = 3           # Number of backups to keep

# Version constraints
minimum_version = "0.2.0"  # Don't downgrade below this
skip_versions = []         # Versions to skip (e.g., ["0.3.0"])
```

**Update Channels**:
- `stable`: Production-ready releases (recommended)
- `beta`: Pre-release testing (may have bugs)
- `nightly`: Latest development builds (unstable)

**Key Options**:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable update checks |
| `check_interval` | string | `daily` | Check frequency |
| `channel` | string | `stable` | Update channel |
| `auto_install` | boolean | `false` | Auto-install updates |

### Security Settings

Configure security features.

```toml
[security]
# API key handling
api_keys_in_memory_only = true  # Don't persist API keys
redact_keys_in_logs = true      # Redact keys from logs

# Rate limiting
rate_limit_enabled = true
rate_limit_per_minute = 60
rate_limit_burst = 10

# Access control
allowed_ips = []           # Empty = allow all
blocked_ips = []           # Block specific IPs
require_api_key = true     # Require API key for requests

# Audit logging
audit_log_enabled = false
audit_log_path = "~/.config/cco/audit.log"
audit_log_format = "json"
```

**Key Options**:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `api_keys_in_memory_only` | boolean | `true` | Don't persist API keys |
| `rate_limit_enabled` | boolean | `true` | Enable rate limiting |
| `rate_limit_per_minute` | integer | `60` | Max requests per minute |
| `allowed_ips` | array | `[]` | Whitelist IPs (empty = allow all) |

## Environment Variables

CCO supports configuration via environment variables, which override config file settings.

### Core Settings

```bash
# Proxy settings
export CCO_HOST="127.0.0.1"
export CCO_PORT="8000"
export CCO_LOG_LEVEL="info"          # debug, info, warn, error

# Cache settings
export CCO_CACHE_ENABLED="true"
export CCO_CACHE_SIZE="1000"
export CCO_CACHE_TTL="3600"

# API keys (recommended method)
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."

# Update settings
export CCO_UPDATE_CHANNEL="stable"   # stable, beta, nightly
export CCO_AUTO_UPDATE="false"

# Database
export CCO_DB_PATH="$HOME/.config/cco/analytics.db"
```

### Complete Environment Variable List

| Environment Variable | Config File Equivalent | Default |
|---------------------|------------------------|---------|
| `CCO_HOST` | `proxy.host` | `127.0.0.1` |
| `CCO_PORT` | `proxy.port` | `8000` |
| `CCO_LOG_LEVEL` | `log_level` | `info` |
| `CCO_CACHE_ENABLED` | `cache.enabled` | `true` |
| `CCO_CACHE_SIZE` | `cache.max_capacity` | `1000` |
| `CCO_CACHE_TTL` | `cache.ttl_seconds` | `3600` |
| `CCO_UPDATE_CHANNEL` | `updates.channel` | `stable` |
| `CCO_AUTO_UPDATE` | `updates.auto_install` | `false` |
| `CCO_DB_PATH` | `analytics.db_path` | `~/.config/cco/analytics.db` |
| `ANTHROPIC_API_KEY` | `providers.anthropic.api_key_env` | - |
| `OPENAI_API_KEY` | `providers.openai.api_key_env` | - |

## Command-Line Configuration

Many settings can be overridden via command-line flags:

```bash
# Start proxy with custom settings
cco proxy \
  --port 9000 \
  --host 0.0.0.0 \
  --cache-size 2000 \
  --log-level debug \
  --config /path/to/config.toml

# View effective configuration
cco config show --effective

# Update specific settings
cco config set proxy.port 9000
cco config set cache.max_capacity 2000
cco config set updates.auto_install true

# Get specific setting
cco config get proxy.port
```

**Common Command-Line Flags**:

| Flag | Description | Default |
|------|-------------|---------|
| `--port` | Port to listen on | `8000` |
| `--host` | Host address to bind | `127.0.0.1` |
| `--cache-size` | Cache capacity | `1000` |
| `--log-level` | Log level | `info` |
| `--config` | Config file path | `~/.config/cco/config.toml` |
| `--no-cache` | Disable caching | - |
| `--no-analytics` | Disable analytics | - |

## Configuration Examples

### Minimal Configuration

```toml
# ~/.config/cco/config.toml
[proxy]
port = 8000

[cache]
enabled = true
max_capacity = 500
```

### Development Configuration

```toml
# Development setup with debug logging
log_level = "debug"

[proxy]
host = "127.0.0.1"
port = 8000
workers = 2

[cache]
enabled = true
max_capacity = 500
ttl_seconds = 1800       # 30 minutes

[analytics]
enabled = true
track_costs = true

[updates]
enabled = true
channel = "beta"         # Get beta releases
auto_install = false
notify_on_update = true
```

### Production Configuration

```toml
# Production setup with security and performance
log_level = "warn"

[proxy]
host = "0.0.0.0"         # Listen on all interfaces
port = 443               # HTTPS port
workers = 8              # More workers
tls_enabled = true
tls_cert = "/etc/cco/cert.pem"
tls_key = "/etc/cco/key.pem"
request_timeout = 120
max_connections = 5000

[cache]
enabled = true
max_capacity = 10000     # Larger cache
ttl_seconds = 7200       # 2 hours
persist_enabled = true
persist_path = "/var/cache/cco/cache.db"

[analytics]
enabled = true
db_path = "/var/lib/cco/analytics.db"
retention_days = 365

[security]
rate_limit_enabled = true
rate_limit_per_minute = 1000
allowed_ips = ["10.0.0.0/8", "172.16.0.0/12"]
audit_log_enabled = true
audit_log_path = "/var/log/cco/audit.log"

[updates]
enabled = true
channel = "stable"
auto_install = false     # Manual updates in production
```

### Multi-Provider Routing

```toml
# Route different models to different providers
[[routes]]
pattern = "^claude-opus"
provider = "anthropic"
endpoint = "https://api.anthropic.com/v1"

[[routes]]
pattern = "^claude-sonnet"
provider = "anthropic"
endpoint = "https://api.anthropic.com/v1"

[[routes]]
pattern = "^gpt-4"
provider = "openai"
endpoint = "https://api.openai.com/v1"

[[routes]]
pattern = "^gpt-3.5"
provider = "openai"
endpoint = "https://api.openai.com/v1"

[[routes]]
pattern = "^ollama/"
provider = "ollama"
endpoint = "http://localhost:11434"

# Fallback chains
[[fallbacks]]
primary = "claude-opus-4"
fallbacks = ["claude-sonnet-3.5", "gpt-4", "ollama/llama3-70b"]
max_retries = 3

# Provider configurations
[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"

[providers.openai]
api_key_env = "OPENAI_API_KEY"
organization = "org-xxxxx"

[providers.ollama]
base_url = "http://localhost:11434"
```

### Team/Enterprise Configuration

```toml
# Shared configuration for team use
[proxy]
host = "0.0.0.0"
port = 8000

[cache]
enabled = true
max_capacity = 20000     # Large shared cache
ttl_seconds = 14400      # 4 hours

[analytics]
enabled = true
db_path = "/shared/cco/analytics.db"
track_costs = true
retention_days = 365

# Per-project cost tracking
[analytics.projects]
project_a = { budget_monthly = 100.0 }
project_b = { budget_monthly = 250.0 }
project_c = { budget_monthly = 500.0 }

[security]
rate_limit_enabled = true
rate_limit_per_minute = 5000
audit_log_enabled = true
```

## Configuration Precedence

CCO uses the following precedence order (highest to lowest):

1. **Command-line flags** (highest priority)
   ```bash
   cco proxy --port 9000 --cache-size 2000
   ```

2. **Environment variables**
   ```bash
   export CCO_PORT=9000
   export CCO_CACHE_SIZE=2000
   ```

3. **Configuration file**
   ```toml
   [proxy]
   port = 9000

   [cache]
   max_capacity = 2000
   ```

4. **Built-in defaults** (lowest priority)

**Example**:
```bash
# Config file says port 8000
# Environment variable says port 9000
# Command-line flag says port 7000
# Result: Uses port 7000 (command-line wins)

export CCO_PORT=9000
cco proxy --port 7000 --config config.toml
# Listens on port 7000
```

## Configuration Validation

Validate your configuration:

```bash
# Check configuration syntax
cco config validate

# Show effective configuration (after merging all sources)
cco config show --effective

# Test configuration
cco proxy --test-config
```

## Configuration Backups

CCO automatically backs up configuration on changes:

```bash
# View configuration history
ls ~/.config/cco/backups/
# config.toml.backup-20251115-100000
# config.toml.backup-20251114-153000

# Restore from backup
cp ~/.config/cco/backups/config.toml.backup-20251115-100000 ~/.config/cco/config.toml
```

## Next Steps

- [USAGE.md](USAGE.md) - Usage guide and examples
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Configuration troubleshooting
- [UPDATING.md](UPDATING.md) - Update configuration management

---

Last updated: 2025-11-15
