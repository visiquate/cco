# Configuration Guide

Complete guide to configuring CCO and understanding configuration options.

## Configuration Methods

There are three ways to configure CCO:

1. **Command-line arguments** - Applied when starting daemon
2. **Environment variables** - Applied globally
3. **Configuration files** - Persistent configuration

---

## Command-Line Arguments

### Daemon Startup Options

```bash
cco daemon start [OPTIONS]
```

**Listening & Network:**
- `--port PORT`: Listen port (default: 3000)
- `--host HOST`: Bind address (default: 127.0.0.1)

**Cache Configuration:**
- `--cache-size MB`: Cache size in MB (default: 500)
- `--cache-ttl SECS`: Cache TTL in seconds (default: 3600)

**Database:**
- `--db-path PATH`: SQLite database file path

**Logging:**
- `--log-level LEVEL`: Log level: debug, info, warn, error (default: info)

**Performance:**
- `--workers N`: Worker threads (default: CPU count)

### Examples

**Development:**
```bash
cco daemon start --log-level debug
```

**Production:**
```bash
cco daemon start --host 0.0.0.0 --port 3000 --workers 16
```

**Large Cache:**
```bash
cco daemon start --cache-size 2000 --cache-ttl 7200
```

**Small Machine:**
```bash
cco daemon start --cache-size 100 --workers 2
```

---

## Environment Variables

### API Keys

**Required for Anthropic Claude:**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Optional for OpenAI:**
```bash
export OPENAI_API_KEY="sk-..."
```

**Optional for Groq:**
```bash
export GROQ_API_KEY="..."
```

**Optional for local services:**
```bash
export LOCAL_API_KEY="..."
```

### CCO Configuration

```bash
# Listen configuration
export CCO_PORT=3000              # Port to listen on
export CCO_HOST=127.0.0.1         # Bind address
export CCO_BIND_ADDRESS=0.0.0.0   # Alias for CCO_HOST

# Cache configuration
export CCO_CACHE_SIZE=500         # MB
export CCO_CACHE_TTL=3600         # seconds

# Database configuration
export CCO_DB_PATH=/var/lib/cco.db

# Logging
export CCO_LOG_LEVEL=info         # debug, info, warn, error

# Performance
export CCO_WORKERS=8              # Thread count

# Feature flags
export CCO_ENABLE_CACHE=true
export CCO_ENABLE_ANALYTICS=true
export CCO_ENABLE_DASHBOARD=true
```

### Orchestration

These are automatically set by `cco` command but can be overridden:

```bash
export ORCHESTRATOR_ENABLED=true
export ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings
export ORCHESTRATOR_API_URL=http://localhost:3000
```

### Setting in Shell Profile

To persist environment variables:

**bash (~/.bashrc):**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export CCO_LOG_LEVEL=info
```

**zsh (~/.zshrc):**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export CCO_LOG_LEVEL=info
```

**fish (~/.config/fish/config.fish):**
```fish
set -x ANTHROPIC_API_KEY "sk-ant-..."
set -x CCO_LOG_LEVEL info
```

---

## Configuration Files

CCO looks for configuration in this order:

1. `./config/` (current directory)
2. `~/.config/cco/` (user home)
3. `/etc/cco/` (system-wide, Linux only)

### model-routing.json

Defines which models go to which providers.

**Location:** `config/model-routing.json`

**Example:**
```json
{
  "routes": [
    {
      "pattern": "^claude-",
      "provider": "anthropic",
      "endpoint": "https://api.anthropic.com/v1",
      "api_key_env": "ANTHROPIC_API_KEY",
      "timeout_ms": 60000,
      "max_retries": 3
    },
    {
      "pattern": "^gpt-",
      "provider": "openai",
      "endpoint": "https://api.openai.com/v1",
      "api_key_env": "OPENAI_API_KEY",
      "timeout_ms": 60000,
      "max_retries": 3
    },
    {
      "pattern": "^ollama/",
      "provider": "ollama",
      "endpoint": "http://localhost:11434/api",
      "api_key_env": null,
      "timeout_ms": 120000,
      "max_retries": 2
    },
    {
      "pattern": "^local/",
      "provider": "openai",
      "endpoint": "http://localhost:8001/v1",
      "api_key_env": "LOCAL_API_KEY",
      "timeout_ms": 120000,
      "max_retries": 2
    }
  ],
  "fallback_chain": {
    "claude-opus-4": ["claude-sonnet-3.5", "gpt-4"],
    "claude-sonnet-3.5": ["claude-haiku", "gpt-4-turbo"],
    "gpt-4": ["gpt-4-turbo", "claude-sonnet-3.5"]
  }
}
```

**Configuration Fields:**

- `pattern`: Regex pattern to match model names (e.g., `^claude-` matches all Claude models)
- `provider`: Provider type (anthropic, openai, ollama, etc.)
- `endpoint`: API endpoint URL
- `api_key_env`: Environment variable containing API key (null if not needed)
- `timeout_ms`: Request timeout in milliseconds
- `max_retries`: Automatic retry attempts on failure
- `fallback_chain`: Alternative models to try if primary fails

### model-pricing.json

Defines pricing for cost tracking.

**Location:** `config/model-pricing.json`

**Example:**
```json
{
  "pricing": {
    "claude-opus-4": {
      "input": 15.0,
      "output": 75.0,
      "cache_read": 1.5,
      "cache_write": 18.75
    },
    "claude-sonnet-3.5": {
      "input": 3.0,
      "output": 15.0,
      "cache_read": 0.3,
      "cache_write": 3.75
    },
    "claude-haiku": {
      "input": 0.25,
      "output": 1.25,
      "cache_read": 0.025,
      "cache_write": 0.30
    },
    "gpt-4": {
      "input": 30.0,
      "output": 60.0
    },
    "gpt-4-turbo": {
      "input": 10.0,
      "output": 30.0
    },
    "gpt-3.5-turbo": {
      "input": 0.5,
      "output": 1.5
    },
    "ollama/llama3-70b": {
      "input": 0.0,
      "output": 0.0,
      "savings_comparison": "claude-sonnet-3.5"
    },
    "ollama/mistral-7b": {
      "input": 0.0,
      "output": 0.0,
      "savings_comparison": "claude-haiku"
    }
  }
}
```

**Price Format:**

- `input`: Cost per 1M input tokens (in cents)
- `output`: Cost per 1M output tokens (in cents)
- `cache_read`: Cost per 1M cache read tokens (optional)
- `cache_write`: Cost per 1M cache write tokens (optional)
- `savings_comparison`: Compare free models to this paid model for display

---

## Common Configurations

### Development Machine

Smaller cache, all features enabled:

```bash
cco daemon start \
  --port 3000 \
  --cache-size 500 \
  --cache-ttl 3600 \
  --log-level info
```

### Team Server

Larger cache, accessible to team:

```bash
cco daemon start \
  --host 0.0.0.0 \
  --port 3000 \
  --cache-size 2000 \
  --cache-ttl 7200 \
  --workers 16 \
  --log-level warn
```

### Production

Optimized for throughput and reliability:

```bash
cco daemon start \
  --host 0.0.0.0 \
  --port 3000 \
  --cache-size 3000 \
  --cache-ttl 7200 \
  --workers 32 \
  --log-level warn \
  --db-path /var/lib/cco/analytics.db
```

### Low-Resource Machine

Minimal cache and workers:

```bash
cco daemon start \
  --cache-size 100 \
  --cache-ttl 1800 \
  --workers 2 \
  --log-level error
```

### High-Traffic API Service

Maximum performance:

```bash
cco daemon start \
  --cache-size 5000 \
  --cache-ttl 14400 \
  --workers 64 \
  --log-level error \
  --db-path /var/lib/cco/fast.db
```

---

## Docker Configuration

### Environment File

Create `.env` file:

```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
CCO_PORT=3000
CCO_HOST=0.0.0.0
CCO_CACHE_SIZE=1000
CCO_WORKERS=8
CCO_LOG_LEVEL=info
```

### Docker Run

```bash
docker run -d \
  -p 3000:3000 \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  -e CCO_CACHE_SIZE=1000 \
  -e CCO_HOST=0.0.0.0 \
  -v cco-data:/var/lib/cco \
  --name cco \
  cco:latest
```

### Docker Compose

```yaml
version: '3'
services:
  cco:
    image: cco:latest
    ports:
      - "3000:3000"
    environment:
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
      OPENAI_API_KEY: ${OPENAI_API_KEY}
      CCO_PORT: 3000
      CCO_HOST: 0.0.0.0
      CCO_CACHE_SIZE: 1000
      CCO_WORKERS: 8
      CCO_LOG_LEVEL: info
    volumes:
      - cco-data:/var/lib/cco
      - ./config:/etc/cco
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  cco-data:
```

---

## Performance Tuning

### Cache Settings

**Development (fast iteration, low memory):**
```bash
--cache-size 200 --cache-ttl 1800  # 200MB, 30 min
```

**Production (high hit rate, consistent):**
```bash
--cache-size 2000 --cache-ttl 7200  # 2GB, 2 hours
```

**Batch processing (large datasets):**
```bash
--cache-size 3000 --cache-ttl 14400  # 3GB, 4 hours
```

**Memory-constrained:**
```bash
--cache-size 50 --cache-ttl 900  # 50MB, 15 min
```

### Worker Threads

**Single-core machine:**
```bash
--workers 1
```

**Multi-core (optimal is CPU cores × 2):**
```bash
# For 8-core CPU:
--workers 16
```

**High throughput:**
```bash
--workers $(nproc)  # Use all available cores
```

### Database Performance

**Default SQLite:**
```bash
--db-path /var/lib/cco/analytics.db
```

**Fast SSD:**
```bash
--db-path /nvme/cco/analytics.db
```

**Network storage (with caution):**
```bash
--db-path /mnt/shared/cco/analytics.db
```

---

## Security Configuration

### API Key Management

**Option 1: Environment variable (recommended)**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
cco daemon start
```

**Option 2: .env file (gitignore it!)**
```bash
cat > .env << EOF
ANTHROPIC_API_KEY=sk-ant-...
EOF

source .env
cco daemon start
```

**Option 3: Docker secrets**
```bash
docker run --secret anthropic_key=sk-ant-... cco:latest
```

### Network Security

**Local development only:**
```bash
cco daemon start --host 127.0.0.1 --port 3000
```

**Team server (use HTTPS proxy):**
```bash
cco daemon start --host 127.0.0.1 --port 3000
# Then use nginx/Caddy as HTTPS reverse proxy
```

**Multiple machines:**
```bash
# Never bind to 0.0.0.0 directly
# Always use HTTPS reverse proxy in front
```

---

## Monitoring Configuration

### Log Levels

```bash
# Maximum detail (development)
--log-level debug

# Informational (default)
--log-level info

# Warnings and errors only (production)
--log-level warn

# Only errors (minimal logging)
--log-level error
```

### Log Output

**To stdout (default):**
```bash
cco daemon start --log-level info
```

**To file:**
```bash
cco daemon start > /var/log/cco/daemon.log 2>&1 &
```

**Rotate logs:**
```bash
# Use logrotate (Linux) or similar
# Create /etc/logrotate.d/cco
/var/log/cco/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
}
```

---

## Troubleshooting Configuration

### Verify Settings

```bash
# Check environment
env | grep CCO_
env | grep ORCHESTRATOR_
env | grep API_KEY

# Check config files
cat ./config/model-routing.json
cat ./config/model-pricing.json

# Check daemon status
cco daemon status
cco daemon logs | head -20
```

### Common Issues

**Port already in use:**
```bash
lsof -i :3000
# Then use different port:
cco daemon start --port 3001
```

**API key not recognized:**
```bash
# Verify key is set
echo $ANTHROPIC_API_KEY | head -c 10

# Test connection
curl -H "x-api-key: $ANTHROPIC_API_KEY" \
  https://api.anthropic.com/v1/models
```

**Cache not working:**
```bash
# Check cache size
cco daemon status | grep -i cache

# Check hit rate
curl http://localhost:3000/api/cache/stats
```

---

## Configuration Reset

Reset to defaults:

```bash
# Stop daemon
cco daemon stop

# Remove cache (clears on next restart)
cco daemon restart

# Reset all config
rm -rf ~/.config/cco

# Start fresh
cco daemon start
```
