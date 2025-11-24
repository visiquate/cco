# CCO Quick Reference

One-page reference for common CCO commands and operations.

## Installation

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex

# Verify
cco --version
```

## Basic Usage

```bash
# Start proxy (default port 8000)
cco proxy

# Start on custom port
cco proxy --port 9000

# Start with debug logging
cco proxy --log-level debug

# Start in background
cco proxy --daemon
```

## API Keys

```bash
# Set API key (required)
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional: OpenAI
export OPENAI_API_KEY="sk-..."

# Point Claude Code to CCO
export LLM_ENDPOINT="http://localhost:8000"
```

## Statistics

```bash
# Current project stats
cco stats

# All projects
cco stats --all

# By model
cco stats --by-model

# Export to file
cco stats --export costs.json
```

## Cache Management

```bash
# View cache stats
cco cache stats

# Clear cache
cco cache clear

# Prune old entries
cco cache prune --older-than 1h

# List cached entries
cco cache list --limit 20
```

## Configuration

```bash
# Initialize config
cco config init

# View configuration
cco config show

# Set value
cco config set proxy.port 9000
cco config set cache.max_capacity 2000

# Get value
cco config get proxy.port

# Edit in editor
cco config edit

# Validate config
cco config validate
```

## Updates

```bash
# Check for updates
cco update --check

# Install update
cco update --install

# Enable auto-updates
cco config set updates.auto_install true

# Rollback
cco update --rollback
```

## Troubleshooting

```bash
# Health check
cco health

# View logs
tail -f ~/.config/cco/cco.log

# Test configuration
cco proxy --test-config

# Reset configuration
cco config reset
```

## Common Configuration Settings

```toml
# ~/.config/cco/config.toml

[proxy]
host = "127.0.0.1"
port = 8000
workers = 4

[cache]
enabled = true
max_capacity = 1000
ttl_seconds = 3600

[updates]
enabled = true
auto_install = false
check_interval = "daily"
channel = "stable"
```

## Environment Variables

```bash
# Proxy settings
export CCO_HOST="127.0.0.1"
export CCO_PORT="8000"
export CCO_LOG_LEVEL="info"

# Cache settings
export CCO_CACHE_ENABLED="true"
export CCO_CACHE_SIZE="1000"

# API keys
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
```

## File Locations

```
~/.local/bin/cco                    # Binary
~/.config/cco/config.toml           # Configuration
~/.config/cco/analytics.db          # Analytics database
~/.cache/cco/                       # Cache directory
~/.config/cco/cco.log              # Log file
```

## API Endpoints

```bash
# Health check
curl http://localhost:8000/health

# Statistics
curl http://localhost:8000/api/stats/current

# Cache stats
curl http://localhost:8000/api/cache/stats

# Clear cache
curl -X POST http://localhost:8000/api/cache/clear
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Command not found | Add `~/.local/bin` to PATH |
| Port in use | Use `--port 9000` or kill process on port 8000 |
| Permission denied | `chmod +x ~/.local/bin/cco` |
| Cache not working | `cco config set cache.enabled true` |
| API key errors | Verify `echo $ANTHROPIC_API_KEY` |
| High memory | Reduce cache: `cco config set cache.max_capacity 500` |

## Update Channels

| Channel | Description | Use Case |
|---------|-------------|----------|
| `stable` | Production releases | Recommended for all users |
| `beta` | Pre-release testing | Early adopters, testing |
| `nightly` | Daily development builds | Developers only |

## Platform-Specific

### macOS

```bash
# Remove quarantine
xattr -d com.apple.quarantine ~/.local/bin/cco

# Check if running
ps aux | grep cco
```

### Linux

```bash
# Check if running
ps aux | grep cco

# System-wide install (optional)
sudo mv cco /usr/local/bin/
```

### Windows

```powershell
# Check if running
Get-Process | Where-Object {$_.Name -eq "cco"}

# Add to PATH
$env:PATH += ";$env:USERPROFILE\.local\bin"
```

## Uninstallation

```bash
# Remove binary
rm ~/.local/bin/cco

# Remove configuration (optional)
rm -rf ~/.config/cco

# Remove cache (optional)
rm -rf ~/.cache/cco
```

## Links

- **Documentation**: https://github.com/brentley/cco-releases
- **Releases**: https://github.com/brentley/cco-releases/releases
- **Issues**: https://github.com/brentley/cco-releases/issues
- **Support**: support@visiquate.com

---

For detailed documentation, see:
- [INSTALLATION.md](docs/INSTALLATION.md) - Installation guide
- [CONFIGURATION.md](docs/CONFIGURATION.md) - Configuration reference
- [USAGE.md](docs/USAGE.md) - Usage guide
- [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) - Troubleshooting guide
- [UPDATING.md](docs/UPDATING.md) - Update guide
