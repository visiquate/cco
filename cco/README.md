# Claude Code Orchestrator Proxy (CCO)

**Version:** 2025.11.1

**Unified LLM API proxy with automatic cost savings, multi-model routing, and real-time analytics.**

CCO sits between Claude Code and multiple LLM providers, transparently handling caching, routing, and cost tracking. Use Claude's most powerful models while automatically saving 50-90% on API costs through intelligent caching and self-hosted model routing.

## What is CCO?

CCO is a production-ready API proxy daemon that:

- **Saves money**: Caches API responses to eliminate duplicate requests (100% cost savings on hits)
- **Routes requests**: Sends requests to Claude API, OpenAI, Ollama, or local LLMs automatically
- **Tracks costs**: Real-time analytics dashboard showing costs, savings, and model usage
- **Stays transparent**: Works exactly like the Claude API—no code changes needed
- **Runs anywhere**: Cross-platform (macOS, Windows, Linux), single binary, zero configuration

## Quick Start (2 minutes)

### 1. Install CCO

```bash
# Install using official installation script
curl -fsSL https://cco.visiquate.com/install.sh | sh

# Or build from source
cargo build --release
sudo ln -sf $(pwd)/target/release/cco /usr/local/bin/cco
```

### 2. Authenticate (Required)

```bash
# Login to CCO (first time only)
cco login

# Opens browser for secure OIDC authentication
# Enter code shown in terminal
# Complete authentication in browser

# You're logged in! Tokens stored securely in ~/.config/cco/tokens.json
```

**Authentication Features**:
- Secure OIDC device flow (no password in CLI)
- Automatic token refresh (transparent)
- Access control via groups
- Session lasts 30 days

### 3. Configure API Keys

```bash
# Set your Anthropic API key (required)
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional: Set OpenAI key if using OpenAI models
export OPENAI_API_KEY="sk-..."

# Optional: Use local Ollama (no key needed)
# Install from https://ollama.ai
ollama serve
```

### 4. Launch Claude Code with Orchestration

```bash
# Navigate to your project
cd ~/my-project

# Launch Claude Code (daemon auto-starts if needed)
cco

# In another terminal, monitor activity
cco tui
```

That's it! CCO automatically:
- Verifies your authentication status
- Starts the daemon if not running
- Creates encrypted settings in OS temp directory
- Sets orchestration environment variables
- Launches Claude Code in your current directory

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

```
Using ollama/llama3-70b would cost $2.50 with Claude Sonnet
→ Saved: $2.50 per request
```

### 4. Real-Time Analytics Dashboard

View live metrics in your browser:

- Cost per project and model
- Cache hit rate and savings
- Token usage trends
- Request latency
- Model performance comparisons

Access at: `http://localhost:3000` (after starting CCO)

The dashboard automatically refreshes every 5 seconds to show updated metrics and trends.

### 5. Fallback Chains

Automatically try alternative models if your primary choice fails:

```
User requests: "claude-opus-4"
↓ (if fails) Try: "claude-sonnet-3.5"
↓ (if fails) Try: "gpt-4"
```

## Architecture Overview

```
┌─────────────┐
│ Claude Code │
└──────┬──────┘
       │ (API requests)
       ▼
┌──────────────────┐
│   CCO Daemon     │
├──────────────────┤
│ Cache Layer      │ ← Moka in-memory cache
│ Router           │ ← Pattern matching to providers
│ Analytics DB     │ ← SQLite cost tracking
│ Web Dashboard    │ ← Real-time metrics UI
└──────┬───────────┘
       │
   ┌───┴────┬────────┬─────────┐
   ▼        ▼        ▼         ▼
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│Claude│ │OpenAI│ │Ollama│ │Local │
│ API  │ │ API  │ │ LLM  │ │ LLM  │
└──────┘ └──────┘ └──────┘ └──────┘
```

## Usage

### Basic Commands

```bash
# Authentication
cco login              # Login via OIDC device flow
cco logout             # Logout and clear tokens

# Launch Claude Code with orchestration
cco                    # Launches in current directory

# Launch TUI monitoring dashboard
cco tui                # Real-time metrics and agent activity

# Manage daemon
cco daemon start       # Start daemon (usually auto-starts)
cco daemon stop        # Stop daemon
cco daemon restart     # Restart daemon
cco daemon status      # Check daemon status
cco daemon logs        # View daemon logs

# Version and updates
cco version            # Show version
cco update             # Check/install updates (requires authentication)

# Pass arguments to Claude Code
cco --help             # Shows Claude Code help
cco analyze main.rs    # Runs Claude Code analysis
```

### Workflows

#### Development with Monitoring

```bash
# Terminal 1: Development
cd ~/my-awesome-project
cco                    # Launches Claude Code with orchestration

# Terminal 2: Monitoring
cco tui                # Watch agent activity in real-time
```

#### Multiple Projects

```bash
# Each project gets same daemon + VFS
cd ~/project-1
cco                    # Launches Claude Code for project 1

# In another terminal
cd ~/project-2
cco                    # Launches Claude Code for project 2
# Both share the same daemon for routing/orchestration
```

### Environment Variables (Auto-Set)

When you run `cco`, these variables are automatically set for Claude Code:

```bash
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings
ORCHESTRATOR_API_URL=http://localhost:3000
```

**Note**: `$TMPDIR` is the OS temp directory:
- macOS: `/var/folders/xx/xxx/T/`
- Windows: `C:\Users\[user]\AppData\Local\Temp\`
- Linux: `/tmp/`

## Real-World Example: Save $500/month

**Scenario**: Team of 10 using Claude for development tasks.

Without CCO:
- 10 people × 500 requests/day × $0.005 per request = **$25/day** = **$750/month**

With CCO (70% cache hit rate):
- Same requests but 70% are cached (free)
- 10 people × 500 × 0.30 × $0.005 = **$7.50/day** = **$225/month**
- **Savings: $525/month** (70% reduction)

With self-hosted models (100% free for ollama/llama):
- 10 people × 500 × 0.50 (half to Llama) = **$3.75/day** = **$112/month**
- **Savings: $638/month** (85% reduction)

## CLI Commands

### Server Management

```bash
# Start the daemon
cco run

# Start with custom configuration
cco run --port 8000 --host 0.0.0.0

# Run with debug logging
cco run --debug

# Check server health
cco health
```

### Installation & Updates

```bash
# Install to ~/.local/bin
cco install

# Check for updates
cco update --check

# Install updates
cco update
```

### Analytics & Monitoring

```bash
# View current status
cco status

# View detailed logs
cco logs

# Graceful shutdown
cco shutdown
```

## Configuration Files

### `model-routing.json`

Defines which models go to which providers:

```json
{
  "routes": [
    {
      "pattern": "^claude-",
      "provider": "anthropic",
      "endpoint": "https://api.anthropic.com/v1",
      "timeout_ms": 60000
    }
  ]
}
```

### `model-pricing.json`

Pricing and cost comparison settings:

```json
{
  "pricing": {
    "claude-opus-4": {
      "input": 15.0,
      "output": 75.0
    },
    "ollama/llama3-70b": {
      "input": 0.0,
      "output": 0.0,
      "savings_comparison": "claude-opus-4"
    }
  }
}
```

## API Compatibility

CCO implements the full Anthropic Messages API. No code changes needed.

### Before (using Claude API directly)

```python
import anthropic

client = anthropic.Anthropic(api_key="sk-ant-...")
response = client.messages.create(
    model="claude-opus-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

### After (using CCO)

```python
import anthropic

# Change endpoint only, code stays the same
client = anthropic.Anthropic(
    api_key="sk-ant-...",
    base_url="http://localhost:3000"  # ← Point to CCO
)
response = client.messages.create(
    model="claude-opus-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

## Web Dashboard

The CCO dashboard provides real-time analytics and cost tracking in your browser.

### Accessing the Dashboard

```bash
# Start CCO (dashboard auto-opens in browser)
cco run

# Manually navigate to:
open http://localhost:3000
```

### Dashboard Features

**Current Project Tab**
- Real-time cost, tokens, and API call metrics
- Cache hit rate percentage and savings
- Response time trends
- Recent activity log

**Machine-Wide Analytics Tab**
- Total costs and savings across all projects
- Cost breakdown by project and model
- Model usage distribution with charts
- Active projects list with last activity
- Model performance comparisons

The dashboard automatically refreshes every 5 seconds with live data.

## Performance Impact

CCO adds minimal overhead:

- **Cache hits**: <5ms (instant)
- **Cache misses**: +50-100ms (routing + proxy overhead)
- **Memory usage**: ~50-200MB depending on cache size
- **Throughput**: Handles 1000+ requests/second

## Security & Privacy

- **API keys**: Never logged or persisted (except in-memory cache)
- **SSL/TLS**: Use HTTPS in production
- **Rate limiting**: Configurable per model and project
- **Cost controls**: Set per-project budget limits

## Deployment Options

### Local Development

```bash
cco run --port 3000
```

### Docker

```bash
docker run -p 3000:3000 \
  -e ANTHROPIC_API_KEY="sk-ant-..." \
  cco:latest
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cco
  template:
    metadata:
      labels:
        app: cco
    spec:
      containers:
      - name: cco
        image: cco:latest
        ports:
        - containerPort: 3000
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: api-keys
              key: anthropic
```

## Troubleshooting

**Port already in use?**
```bash
cco run --port 9000
```

**Cache not working?**
```bash
# Check cache stats via dashboard or API
curl http://localhost:3000/api/cache/stats

# Clear cache if needed
curl -X POST http://localhost:3000/api/cache/clear
```

**API key errors?**
```bash
# Verify key is set
echo $ANTHROPIC_API_KEY

# Test connection
cco health
```

**Dashboard not opening?**
```bash
# Manually open browser
open http://localhost:3000

# Try different port
cco run --port 8000
```

See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for detailed solutions.

## Version Format

CCO uses date-based versioning: `YYYY.MM.N`

- `YYYY`: Four-digit year (2025, 2026, etc.)
- `MM`: Month (1-12)
- `N`: Release number within that month (resets to 1 at the start of each month)

**Examples:**
- `2025.11.1`: First release in November 2025
- `2025.11.2`: Second release in November 2025
- `2025.12.1`: First release in December 2025

This format provides clarity on when a version was released, with simple incrementing per month.

## Roadmap

### Near-Term
- [ ] TUI dashboard for non-browser environments
- [ ] Advanced cost prediction and budgeting
- [ ] Multi-user authentication and project isolation
- [ ] Enhanced caching strategies (semantic similarity)

### Long-Term
- [ ] Distributed caching across multiple machines
- [ ] Custom model fine-tuning integration
- [ ] Enterprise SSO and RBAC
- [ ] Cloud-hosted CCO service

## Documentation

### User Documentation
1. **[USAGE.md](./USAGE.md)** - Complete command reference and configuration
2. **[USER_GUIDE_AUTHENTICATION.md](../docs/USER_GUIDE_AUTHENTICATION.md)** - Authentication guide for end users
3. **[COST_SAVINGS.md](./COST_SAVINGS.md)** - Understand savings calculations
4. **[TROUBLESHOOTING.md](./TROUBLESHOOTING.md)** - Common issues and solutions
5. **[BUILDING.md](./BUILDING.md)** - Build from source instructions

### Administrator Documentation
1. **[DEPLOYMENT_PRIVATE_DISTRIBUTION.md](../docs/DEPLOYMENT_PRIVATE_DISTRIBUTION.md)** - Server deployment guide
2. **[ADMIN_GUIDE_ACCESS_CONTROL.md](../docs/ADMIN_GUIDE_ACCESS_CONTROL.md)** - User and access management
3. **[MIGRATION_FROM_GITHUB_RELEASES.md](../docs/MIGRATION_FROM_GITHUB_RELEASES.md)** - Migration guide
4. **[ARCHITECTURE_PRIVATE_DISTRIBUTION.md](../docs/ARCHITECTURE_PRIVATE_DISTRIBUTION.md)** - Technical architecture

### Developer Documentation
1. **[AUTH_MODULE_SUMMARY.md](./AUTH_MODULE_SUMMARY.md)** - Authentication module overview
2. **[AUTH_AND_RELEASES_API_IMPLEMENTATION.md](./AUTH_AND_RELEASES_API_IMPLEMENTATION.md)** - API implementation details
3. **[TESTING_AUTH_FLOW.md](./TESTING_AUTH_FLOW.md)** - Testing procedures

## Support

- Issues: [GitHub Issues](https://github.com/example/cco/issues)
- Documentation: This repository
- Discussions: [GitHub Discussions](https://github.com/example/cco/discussions)

## License

Apache 2.0 - See LICENSE file
