# CCO Usage Guide

Complete usage guide and command reference for CCO (Claude Code Orchestra).

## Table of Contents

- [Quick Start](#quick-start)
- [Command Overview](#command-overview)
- [Starting the Proxy](#starting-the-proxy)
- [Using with Claude Code](#using-with-claude-code)
- [Cost Analytics](#cost-analytics)
- [Cache Management](#cache-management)
- [Update Management](#update-management)
- [Configuration Management](#configuration-management)
- [Advanced Usage](#advanced-usage)
- [Best Practices](#best-practices)

## Quick Start

```bash
# 1. Set your API key
export ANTHROPIC_API_KEY="sk-ant-your-key-here"

# 2. Start the proxy
cco proxy --port 8000

# 3. View the dashboard
open http://localhost:8000

# 4. Point your application to CCO
export LLM_ENDPOINT="http://localhost:8000"
```

## Command Overview

CCO provides several commands for different operations:

```bash
cco --help              # Show help
cco --version           # Show version

cco proxy               # Start the proxy server
cco stats               # View cost analytics
cco cache               # Manage cache
cco update              # Manage updates
cco config              # Configure settings
cco health              # Check system health
```

## Starting the Proxy

### Basic Usage

```bash
# Start on default port (8000)
cco proxy

# Start on custom port
cco proxy --port 9000

# Start with debug logging
cco proxy --log-level debug

# Start in background (daemon mode)
cco proxy --daemon
```

### Advanced Options

```bash
# Custom configuration file
cco proxy --config /path/to/config.toml

# Bind to all interfaces (for network access)
cco proxy --host 0.0.0.0 --port 8000

# Custom cache size
cco proxy --cache-size 2000

# Disable caching
cco proxy --no-cache

# Disable analytics
cco proxy --no-analytics

# Enable TLS/HTTPS
cco proxy --tls-cert cert.pem --tls-key key.pem

# Combine options
cco proxy \
  --port 9000 \
  --host 0.0.0.0 \
  --cache-size 2000 \
  --log-level info \
  --daemon
```

### Stopping the Proxy

```bash
# If running in foreground
# Press Ctrl+C

# If running as daemon
pkill cco

# Graceful shutdown
cco proxy --stop

# Check if running
cco health
```

## Using with Claude Code

### Basic Setup

```bash
# 1. Start CCO proxy
cco proxy --port 8000 &

# 2. Configure environment
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
export LLM_ENDPOINT="http://localhost:8000"

# 3. Use Claude Code normally
# All requests now flow through CCO
```

### Using with Python SDK

```python
import anthropic

# Point to CCO instead of Claude API
client = anthropic.Anthropic(
    api_key="sk-ant-your-key-here",
    base_url="http://localhost:8000"  # CCO proxy
)

response = client.messages.create(
    model="claude-sonnet-3.5",
    messages=[{"role": "user", "content": "Hello!"}],
    max_tokens=100
)
```

### Using with curl

```bash
# Same API as Claude, just different endpoint
curl -X POST http://localhost:8000/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-3.5",
    "messages": [
      {"role": "user", "content": "What is machine learning?"}
    ],
    "max_tokens": 1000
  }'
```

## Cost Analytics

### Viewing Statistics

```bash
# Current project stats
cco stats

# Machine-wide stats (all projects)
cco stats --all

# Specific time range
cco stats --from 2025-11-01 --to 2025-11-15

# By model
cco stats --by-model

# By project
cco stats --by-project

# Show only costs (no tokens)
cco stats --costs-only
```

### Example Output

```
CCO Cost Analytics
─────────────────────────────────────────────
Current Project: my-app

Total Requests:    1,234
Cache Hit Rate:    68.5% (845 hits)
Total Cost:        $45.23
Cache Savings:     $98.45 (68.5%)
Net Cost:          $45.23

By Model:
  claude-opus-4:       $32.10 (450 requests)
  claude-sonnet-3.5:   $13.13 (784 requests)

Token Usage:
  Input Tokens:    2,345,678
  Output Tokens:   456,789
  Cached Tokens:   1,603,089 (68.3%)
```

### Exporting Data

```bash
# Export to JSON
cco stats --export costs.json

# Export to CSV
cco stats --export costs.csv --format csv

# Export specific date range
cco stats --from 2025-11-01 --to 2025-11-15 --export november.json

# Export by model
cco stats --by-model --export by-model.csv
```

### Cost Tracking API

```bash
# Get real-time costs via API
curl http://localhost:8000/api/stats/current

# Get historical data
curl http://localhost:8000/api/stats/history?days=30

# Get by project
curl http://localhost:8000/api/stats/project/my-app
```

## Cache Management

### Cache Operations

```bash
# View cache statistics
cco cache stats

# Clear entire cache
cco cache clear

# Clear cache for specific model
cco cache clear --model claude-opus-4

# Clear old entries (older than 1 hour)
cco cache prune --older-than 1h

# View cache contents (limited)
cco cache list --limit 20

# Export cache statistics
cco cache stats --export cache-stats.json
```

### Cache Statistics

```bash
cco cache stats
```

Output:
```
CCO Cache Statistics
─────────────────────────────────────────────
Status:           Enabled
Strategy:         LRU (Least Recently Used)
Capacity:         1000 entries
Current Size:     743 entries (74.3% full)
Hit Rate:         68.5%
Total Hits:       845
Total Misses:     389
Evictions:        156

Memory Usage:     87.3 MB
Average Entry:    120 KB

Top Cached Models:
  claude-sonnet-3.5:   423 entries (57%)
  claude-opus-4:       320 entries (43%)
```

### Cache API

```bash
# Cache stats via API
curl http://localhost:8000/api/cache/stats

# Clear cache via API
curl -X POST http://localhost:8000/api/cache/clear

# Prune cache via API
curl -X POST http://localhost:8000/api/cache/prune
```

## Update Management

### Checking for Updates

```bash
# Check if update available
cco update --check

# Check specific channel
cco update --check --channel beta

# Verbose output
cco update --check --verbose
```

### Installing Updates

```bash
# Install latest version (interactive)
cco update --install

# Install without confirmation
cco update --install --yes

# Install specific version
cco update --install --version 0.3.0

# Install from beta channel
cco update --install --channel beta
```

### Update Configuration

```bash
# Enable automatic updates
cco config set updates.auto_install true

# Change update channel
cco config set updates.channel beta

# Change check interval
cco config set updates.check_interval weekly

# View update configuration
cco config get updates
```

### Rollback

```bash
# View available backups
cco update --list-backups

# Rollback to previous version
cco update --rollback

# Rollback to specific version
cco update --rollback --version 0.2.0
```

## Configuration Management

### Viewing Configuration

```bash
# Show all configuration
cco config show

# Show effective configuration (after merging all sources)
cco config show --effective

# Get specific value
cco config get proxy.port
cco config get cache.max_capacity
```

### Setting Configuration

```bash
# Set individual values
cco config set proxy.port 9000
cco config set cache.max_capacity 2000
cco config set updates.auto_install true

# Set nested values
cco config set security.rate_limit_per_minute 100
```

### Editing Configuration

```bash
# Open configuration in editor
cco config edit

# Use specific editor
EDITOR=vim cco config edit
```

### Initializing Configuration

```bash
# Create default configuration
cco config init

# Create configuration with prompts
cco config init --interactive

# Reset to defaults (deletes existing)
cco config reset
```

### Configuration Validation

```bash
# Validate configuration syntax
cco config validate

# Test configuration without starting
cco proxy --test-config
```

## Advanced Usage

### Multi-Provider Routing

```bash
# Start proxy with multi-provider routing
cco proxy --config multi-provider.toml
```

Configuration (`multi-provider.toml`):
```toml
[[routes]]
pattern = "^claude-"
provider = "anthropic"
endpoint = "https://api.anthropic.com/v1"

[[routes]]
pattern = "^gpt-"
provider = "openai"
endpoint = "https://api.openai.com/v1"

[[routes]]
pattern = "^ollama/"
provider = "ollama"
endpoint = "http://localhost:11434"
```

### Local Models with Ollama

```bash
# 1. Install and start Ollama
# Visit: https://ollama.ai

# 2. Pull a model
ollama pull llama3

# 3. Configure CCO to route to Ollama
cco config set routes.ollama.endpoint "http://localhost:11434"

# 4. Use Ollama models through CCO
curl -X POST http://localhost:8000/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ollama/llama3",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Docker Deployment

```bash
# Build Docker image
docker build -t cco:latest .

# Run container
docker run -d \
  --name cco-proxy \
  -p 8000:8000 \
  -e ANTHROPIC_API_KEY="sk-ant-..." \
  -v ~/.config/cco:/root/.config/cco \
  cco:latest

# View logs
docker logs -f cco-proxy

# Stop container
docker stop cco-proxy
```

### Production Deployment

```bash
# Start with production configuration
cco proxy \
  --config /etc/cco/production.toml \
  --host 0.0.0.0 \
  --port 443 \
  --tls-cert /etc/cco/cert.pem \
  --tls-key /etc/cco/key.pem \
  --log-level warn \
  --daemon

# Monitor with systemd
sudo systemctl start cco
sudo systemctl enable cco
sudo systemctl status cco
```

### API Integration

CCO exposes several API endpoints:

```bash
# Health check
curl http://localhost:8000/health

# Statistics
curl http://localhost:8000/api/stats/current
curl http://localhost:8000/api/stats/history?days=7

# Cache management
curl http://localhost:8000/api/cache/stats
curl -X POST http://localhost:8000/api/cache/clear

# Configuration
curl http://localhost:8000/api/config
```

### Logging and Debugging

```bash
# Enable debug logging
cco proxy --log-level debug

# Log to file
cco proxy --log-file /var/log/cco/proxy.log

# JSON logging (for parsing)
cco proxy --log-format json

# Trace specific requests
cco proxy --trace-requests --log-level trace
```

## Best Practices

### 1. Cache Configuration

```bash
# For development (small cache, short TTL)
cco config set cache.max_capacity 500
cco config set cache.ttl_seconds 1800  # 30 minutes

# For production (large cache, longer TTL)
cco config set cache.max_capacity 5000
cco config set cache.ttl_seconds 7200  # 2 hours
```

### 2. Security

```bash
# Never commit API keys
# Use environment variables
export ANTHROPIC_API_KEY="sk-ant-..."

# For production, use secrets management
# Kubernetes: kubectl create secret
# Docker: docker secret create
# AWS: Systems Manager Parameter Store
```

### 3. Monitoring

```bash
# Set up periodic stats collection
crontab -e
# Add: */15 * * * * cco stats --export /var/log/cco/stats-$(date +\%Y\%m\%d-\%H\%M).json

# Monitor cache hit rate
watch -n 5 'cco cache stats | grep "Hit Rate"'

# Monitor costs
watch -n 60 'cco stats --costs-only'
```

### 4. Cost Optimization

```bash
# Enable aggressive caching
cco config set cache.strategy lfu
cco config set cache.max_capacity 10000
cco config set cache.ttl_seconds 14400  # 4 hours

# Use local models for simple tasks
# Route simple queries to Ollama, complex ones to Claude
```

### 5. Update Management

```bash
# For production: Disable auto-updates
cco config set updates.auto_install false
cco config set updates.channel stable

# For development: Enable auto-updates
cco config set updates.auto_install true
cco config set updates.channel beta
```

## Common Workflows

### Development Workflow

```bash
# 1. Start CCO in development mode
cco proxy --port 8000 --log-level debug

# 2. Point Claude Code to CCO
export LLM_ENDPOINT="http://localhost:8000"

# 3. Develop as normal
# All requests are cached automatically

# 4. View costs periodically
cco stats

# 5. Clear cache when needed
cco cache clear
```

### Team Workflow

```bash
# 1. Deploy shared CCO instance
cco proxy --host 0.0.0.0 --port 8000 --config /shared/cco.toml

# 2. Team members point to shared instance
export LLM_ENDPOINT="http://cco-server:8000"

# 3. Monitor shared cache and costs
cco stats --all

# 4. Periodic cache pruning
cco cache prune --older-than 24h
```

### CI/CD Workflow

```bash
# In CI pipeline
# Start CCO for testing
cco proxy --port 8000 --daemon

# Run tests (using CCO)
export LLM_ENDPOINT="http://localhost:8000"
pytest tests/

# View test costs
cco stats --export ci-costs.json

# Stop CCO
pkill cco
```

## Troubleshooting Commands

```bash
# Check if CCO is running
cco health

# View current configuration
cco config show --effective

# Test configuration
cco proxy --test-config

# Verify API connectivity
curl http://localhost:8000/health

# Check cache status
cco cache stats

# View logs
tail -f ~/.config/cco/cco.log

# Reset to defaults
cco config reset
```

## Next Steps

- [CONFIGURATION.md](CONFIGURATION.md) - Detailed configuration options
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues and solutions
- [API.md](API.md) - Complete API reference

## Support

For help with usage:

- Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- Search [GitHub Issues](https://github.com/brentley/cco-releases/issues)
- Email: support@visiquate.com

---

Last updated: 2025-11-15
