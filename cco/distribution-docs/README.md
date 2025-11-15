# CCO - Claude Code Orchestra

[![Latest Release](https://img.shields.io/github/v/release/brentley/cco-releases?label=latest)](https://github.com/brentley/cco-releases/releases)
[![Platform Support](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)](https://github.com/brentley/cco-releases)
[![License](https://img.shields.io/badge/license-Apache%202.0-green)](LICENSE)

**Multi-agent development system with automatic cost tracking, multi-model routing, and real-time analytics.**

CCO sits between Claude Code and multiple LLM providers, transparently handling caching, routing, and cost tracking. Use Claude's most powerful models while automatically tracking costs and leveraging intelligent caching for performance optimization.

## Quick Install

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/brentley/cco-releases/main/install.sh | bash

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex
```

After installation:
```bash
# Verify installation
cco --version

# Start the proxy
cco proxy --port 8000

# View dashboard
open http://localhost:8000
```

## What is CCO?

CCO is a production-ready proxy server that:

- **Tracks costs**: Monitors API costs with detailed analytics (100% savings on cache hits)
- **Routes intelligently**: Sends requests to Claude API, OpenAI, Ollama, or local LLMs automatically
- **Tracks everything**: Real-time analytics dashboard showing costs, savings, and model usage
- **Stays transparent**: Works exactly like the Claude API—no code changes needed
- **Runs anywhere**: Single binary, Docker support, zero configuration overhead

### Real-World Savings Example

**Scenario**: Team of 10 using Claude for development tasks.

**Without CCO**: 10 people × 500 requests/day × $0.005 per request = **$750/month**

**With CCO** (70% cache hit rate):
- Same requests but 70% are cached (free)
- Cost: **$225/month**
- **Savings: $525/month** (70% reduction)

**With self-hosted models**: Cost: **$112/month** | **Savings: $638/month** (85% reduction)

## Platform Support

| Platform | Architecture | Status | Binary Name |
|----------|--------------|--------|-------------|
| macOS | Apple Silicon (M1/M2/M3) | ✅ Supported | `darwin-arm64` |
| macOS | Intel (x86_64) | ✅ Supported | `darwin-x86_64` |
| Linux | x86_64 | ✅ Supported | `linux-x86_64` |
| Linux | ARM64 | ✅ Supported | `linux-aarch64` |
| Windows | x86_64 | ✅ Supported | `windows-x86_64` |

## Key Features

### 1. Transparent Caching (Moka)
Every API request is cached in-memory. Identical requests return instantly with zero API cost.

```
First request:  "What is machine learning?" → Claude API → $0.02 → Cached
Second request: "What is machine learning?" → Cache hit → $0.00 (free!)
```

**Typical savings**: 50-70% on development workloads, up to 90% on repetitive tasks.

### 2. Multi-Model Routing
Define rules to automatically route requests to different providers:

```json
{
  "routes": [
    { "pattern": "claude-*", "provider": "anthropic" },
    { "pattern": "gpt-*", "provider": "openai" },
    { "pattern": "ollama/*", "provider": "ollama" }
  ]
}
```

### 3. Self-Hosted Models (Free)
Run Llama, Mistral, or custom models locally via Ollama. CCO tracks savings compared to equivalent commercial models.

### 4. Real-Time Analytics Dashboard
View live metrics:
- Cost per project and model
- Cache hit rate and savings
- Token usage trends
- Request latency
- Model performance comparisons

Access at: `http://localhost:8000` (after starting CCO)

### 5. Automatic Updates
CCO automatically checks for updates and can install them in the background. Configure update behavior:

```bash
# Check for updates
cco update --check

# Install latest version
cco update --install

# Configure auto-update
cco config set updates.auto_install true
```

## Usage Examples

### Basic Usage

```bash
# Start the proxy on default port (8000)
cco proxy

# Start on custom port
cco proxy --port 9000

# Start with specific cache size
cco proxy --cache-size 1000

# Enable debug logging
cco proxy --log-level debug
```

### With Claude Code

Set your environment to point Claude Code to CCO:

```bash
# Set your real Anthropic API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Point to CCO proxy instead of Claude API
export LLM_ENDPOINT="http://localhost:8000"
```

That's it! All Claude Code requests now flow through CCO and benefit from caching and routing.

### Viewing Analytics

```bash
# View current project stats
cco stats

# View machine-wide stats
cco stats --all

# Export cost data
cco stats --export costs.json
```

## Configuration

CCO stores configuration in `~/.config/cco/config.toml`. Key settings:

```toml
[proxy]
port = 8000
host = "0.0.0.0"
cache_size = 1000

[updates]
enabled = true
auto_install = false          # Require explicit opt-in
check_interval = "daily"      # daily, weekly, never
channel = "stable"            # stable, beta, nightly
```

See [CONFIGURATION.md](docs/CONFIGURATION.md) for complete reference.

## Updating CCO

CCO includes automatic update checking:

```bash
# Check for updates (manual)
cco update --check

# Install latest version
cco update --install

# Enable automatic updates
cco config set updates.auto_install true

# Change update channel
cco config set updates.channel beta
```

See [UPDATING.md](docs/UPDATING.md) for detailed update instructions.

## Manual Installation

If you prefer to install manually:

1. Download the appropriate binary for your platform from [Releases](https://github.com/brentley/cco-releases/releases)
2. Extract the archive
3. Move the `cco` binary to `~/.local/bin/` (or any directory in your PATH)
4. Make it executable: `chmod +x ~/.local/bin/cco`
5. Verify: `cco --version`

See [INSTALLATION.md](docs/INSTALLATION.md) for detailed installation instructions.

## Troubleshooting

### Port already in use?
```bash
cco proxy --port 9000
```

### Cache not working?
```bash
# Check cache stats
cco cache stats

# Clear cache if needed
cco cache clear
```

### Binary not found after installation?
```bash
# Add to PATH manually
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

See [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) for detailed solutions.

## Frequently Asked Questions

**Q: Does CCO send my data anywhere?**
A: No. CCO only proxies requests to the providers you configure. All caching is local. No telemetry is collected by default.

**Q: How much does it cost?**
A: CCO is free and open source. You only pay for the underlying API calls (Claude, OpenAI, etc.).

**Q: Can I use it with OpenAI?**
A: Yes! CCO supports multiple providers. See [CONFIGURATION.md](docs/CONFIGURATION.md) for routing configuration.

**Q: Is it secure?**
A: Yes. API keys are never logged or persisted. Use HTTPS in production environments. See [SECURITY.md](SECURITY.md).

**Q: How do I uninstall?**
A: Simply remove the binary: `rm ~/.local/bin/cco` and the config directory: `rm -rf ~/.config/cco`

## Architecture

```
┌─────────────┐
│ Claude Code │
└──────┬──────┘
       │ (API requests)
       ▼
┌──────────────────┐
│   CCO Proxy      │
├──────────────────┤
│ Cache Layer      │ ← Moka in-memory cache
│ Router           │ ← Pattern matching to providers
│ Analytics DB     │ ← SQLite cost tracking
└──────┬───────────┘
       │
   ┌───┴────┬────────┬─────────┐
   ▼        ▼        ▼         ▼
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│Claude│ │OpenAI│ │Ollama│ │Local │
│ API  │ │ API  │ │ LLM  │ │ LLM  │
└──────┘ └──────┘ └──────┘ └──────┘
```

## Documentation

- [Installation Guide](docs/INSTALLATION.md) - Detailed installation instructions
- [Configuration Reference](docs/CONFIGURATION.md) - All configuration options
- [Usage Guide](docs/USAGE.md) - Complete command reference
- [Updating CCO](docs/UPDATING.md) - Update and version management
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and solutions
- [Security](SECURITY.md) - Security information and reporting
- [Changelog](CHANGELOG.md) - Version history

## Security

CCO takes security seriously. If you discover a security vulnerability, please email security@visiquate.com. Do not create a public issue.

See [SECURITY.md](SECURITY.md) for our security policy and supported versions.

## License

CCO is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/brentley/cco-releases/issues)
- **Documentation**: [docs/](docs/)
- **Email**: support@visiquate.com

## About

CCO is developed by [VisiQuate](https://visiquate.com) as part of the Claude Orchestra project.
