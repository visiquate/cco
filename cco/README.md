# Claude Code Orchestrator Proxy (CCO)

**Version:** 202511-1

**Unified LLM API proxy with automatic cost savings, multi-model routing, and real-time analytics.**

CCO sits between Claude Code and multiple LLM providers, transparently handling caching, routing, and cost tracking. Use Claude's most powerful models while automatically saving 50-90% on API costs through intelligent caching and self-hosted model routing.

## Version Format

CCO uses date-based versioning: `YYYY.MM.N`

- `YYYY`: Four-digit year (2025, 2026, etc.)
- `MM`: Two-digit month (01-12)
- `N`: Release number within that month (resets to 1 at the start of each month)

**Examples:**
- `2025.11.1`: First release in November 2025
- `2025.11.2`: Second release in November 2025
- `2025.11.3`: Third release in November 2025
- `2025.12.1`: First release in December 2025
- `2026.1.1`: First release in January 2026

This format provides clarity on when a version was released, with the release number incrementing for each release within the same month and resetting to 1 with each new month.

## What is CCO?

CCO is a production-ready proxy server that:

- **Saves money**: Caches API responses to eliminate duplicate requests (100% cost savings on hits)
- **Routes requests**: Sends requests to Claude API, OpenAI, Ollama, or local LLMs automatically
- **Tracks costs**: Real-time analytics dashboard showing costs, savings, and model usage
- **Stays transparent**: Works exactly like the Claude API—no code changes needed
- **Runs anywhere**: Single binary, Docker support, zero configuration overhead

## Quick Start (2 minutes)

### 1. Install CCO

```bash
# Download the latest release
curl -L https://github.com/example/cco/releases/download/latest/cco-proxy -o cco-proxy
chmod +x cco-proxy

# Or build from source
cargo build --release
./target/release/cco-proxy
```

### 2. Configure API Keys

```bash
# Set your Anthropic API key (required)
export ANTHROPIC_API_KEY="sk-ant-..."

# Optional: Set OpenAI key if using OpenAI models
export OPENAI_API_KEY="sk-..."

# Optional: Use local Ollama (no key needed)
# Install from https://ollama.ai
ollama serve
```

### 3. Start the Proxy

```bash
# Start on port 8000 (default)
./cco-proxy

# Or specify a port
./cco-proxy --port 9000
```

### 4. Point Claude Code to CCO

In your Claude Code configuration or environment:

```bash
# Replace the default Claude API endpoint
export ANTHROPIC_API_KEY="sk-ant-..."  # Your real API key
export LLM_ENDPOINT="http://localhost:8000"  # Point to CCO instead of Claude API
```

That's it! All requests now flow through CCO and benefit from caching and routing.

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

View live metrics:

- Cost per project and model
- Cache hit rate and savings
- Token usage trends
- Request latency
- Model performance comparisons

Access at: `http://localhost:8000` (after starting CCO)

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
    base_url="http://localhost:8000"  # ← Point to CCO
)
response = client.messages.create(
    model="claude-opus-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

## Web UI Dashboard

The CCO dashboard provides real-time analytics and cost tracking in your browser.

### Accessing the Dashboard

**Automatic (Auto-Open):**
```bash
# Start CCO with auto-open enabled (default)
./cco-proxy

# Dashboard automatically opens in your default browser
# If not, manually navigate to: http://localhost:3000
```

**Manual:**
```bash
# Start CCO on custom port
./cco-proxy --port 8888

# Manually open browser to dashboard
open http://localhost:8888
```

### Dashboard Features

The dashboard has three main sections:

**Tab 1: Current Project**
- Real-time cost, tokens, and API call metrics
- Cache hit rate percentage and savings
- Response time trends
- Recent activity log

**Tab 2: Machine-Wide Analytics**
- Total costs and savings across all projects
- Cost breakdown by project and model
- Model usage distribution with charts
- Active projects list with last activity
- Model performance comparisons

**Tab 3: Terminal**
- Live command interface for management
- View logs and debug information
- Manage cache (view stats, clear if needed)
- Export analytics data
- System health monitoring

### Dashboard Auto-Refresh

The dashboard automatically refreshes every 5 seconds to show:
- Updated metrics and trends
- New API calls and cache hits
- Cost calculations
- Project activity

No manual refresh needed—all data is live as requests flow through CCO.

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
./cco-proxy --port 8000
```

### Docker

```bash
docker run -p 8000:8000 \
  -e ANTHROPIC_API_KEY="sk-ant-..." \
  cco-proxy:latest
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco-proxy
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cco-proxy
  template:
    metadata:
      labels:
        app: cco-proxy
    spec:
      containers:
      - name: cco-proxy
        image: cco-proxy:latest
        ports:
        - containerPort: 8000
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: api-keys
              key: anthropic
```

## Dashboard Not Opening

**Dashboard not auto-opening on startup?**
```bash
# Try manually with custom host/port
./cco-proxy --host 0.0.0.0 --port 3000

# Open browser manually
open http://localhost:3000
```

**Common Issues:**
- Firewall blocking local connections: Use `127.0.0.1` instead of `0.0.0.0`
- Port already in use: Try `--port 9000` or higher
- Browser not responding: Check browser console for JavaScript errors

## Troubleshooting

**Port already in use?**
```bash
./cco-proxy --port 9000
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
curl http://localhost:3000/health
```

See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for detailed solutions.

## Next Steps

1. **[USAGE.md](./USAGE.md)** - Complete command reference and configuration
2. **[COST_SAVINGS.md](./COST_SAVINGS.md)** - Understand savings calculations
3. **[MULTI_MODEL.md](./MULTI_MODEL.md)** - Add more providers (OpenAI, Ollama, etc.)
4. **[TROUBLESHOOTING.md](./TROUBLESHOOTING.md)** - Common issues and solutions

## Support

- Issues: [GitHub Issues](https://github.com/example/cco/issues)
- Documentation: This folder
- Discussions: [GitHub Discussions](https://github.com/example/cco/discussions)

## License

Apache 2.0 - See LICENSE file
