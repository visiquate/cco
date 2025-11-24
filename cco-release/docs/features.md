# Features Guide

Complete guide to CCO's features and how to use them.

## Table of Contents

1. [Transparent Caching](#transparent-caching)
2. [Multi-Model Routing](#multi-model-routing)
3. [Real-Time Analytics Dashboard](#real-time-analytics-dashboard)
4. [Cost Tracking](#cost-tracking)
5. [Fallback Chains](#fallback-chains)
6. [Self-Hosted Models](#self-hosted-models)
7. [API Compatibility](#api-compatibility)

---

## Transparent Caching

CCO caches all API responses in-memory. Identical requests return instantly with zero cost.

### How It Works

```
First request:  User input → Claude API → Response → Cached
Second request: User input → Cache hit → Response → Instant (free!)
```

### Cache Key

The cache key is based on:
- Model name
- Complete request parameters
- System prompt
- Temperature, max_tokens, etc.

**Note:** Any difference in parameters means a cache miss.

### Performance Impact

- **Cache hit**: <5ms (instant)
- **Cache miss**: +50-100ms (routing + proxy overhead)
- **Throughput**: Handles 1000+ requests/second

### Typical Savings

- **Development**: 50-70% cost savings (many repeated requests)
- **Batch processing**: 70-90% savings (similar tasks repeated)
- **Production**: 20-50% savings (varied workloads)

### Configuration

```bash
# Increase cache size for more savings
cco daemon start --cache-size 2000  # 2GB (default: 500MB)

# Adjust TTL (time entries live in cache)
cco daemon start --cache-ttl 7200   # 2 hours (default: 1 hour)
```

### Monitoring

```bash
# View cache statistics
curl http://localhost:3000/api/cache/stats

# Response:
{
  "hits": 1234,         # Successful cache hits
  "misses": 456,        # Cache misses (hit API)
  "hitRate": 0.73,      # 73% of requests were cache hits
  "size": 256,          # Current cache size in MB
  "maxSize": 500,       # Maximum cache size in MB
  "entries": 89,        # Number of cached items
  "savedCost": 145.67   # Dollar savings from cache
}
```

### Clear Cache

```bash
# Emergency cache clear
curl -X POST http://localhost:3000/api/cache/clear

# Normal flow: cache expires automatically based on TTL
```

---

## Multi-Model Routing

Automatically route requests to different providers based on model name.

### How It Works

```
User requests model → CCO checks pattern → Routes to provider → Response

Example:
"claude-opus-4" → matches ^claude- → Anthropic API → Cached response
"gpt-4"         → matches ^gpt-    → OpenAI API → Cached response
```

### Default Routes

```json
{
  "routes": [
    { "pattern": "^claude-", "provider": "anthropic" },
    { "pattern": "^gpt-", "provider": "openai" },
    { "pattern": "^ollama/", "provider": "ollama" },
    { "pattern": "^local/", "provider": "openai" }
  ]
}
```

### Adding Custom Routes

Edit `config/model-routing.json`:

```json
{
  "routes": [
    {
      "pattern": "^my-custom-",
      "provider": "openai",
      "endpoint": "http://my-server:8000/v1",
      "api_key_env": "MY_API_KEY",
      "timeout_ms": 120000,
      "max_retries": 3
    }
  ]
}
```

### Using Multiple Models

```python
import anthropic

client = anthropic.Anthropic(
    api_key="sk-ant-...",
    base_url="http://localhost:3000"
)

# All routed automatically
response1 = client.messages.create(
    model="claude-opus-4",
    messages=[{"role": "user", "content": "Hello"}]
)

response2 = client.messages.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello"}]
)

response3 = client.messages.create(
    model="ollama/llama3-70b",
    messages=[{"role": "user", "content": "Hello"}]
)
```

---

## Real-Time Analytics Dashboard

CCO provides a web dashboard showing real-time metrics and costs.

### Accessing Dashboard

```bash
# Dashboard auto-opens in browser when daemon starts
cco daemon start

# Manual access
open http://localhost:3000
```

### Current Project Tab

Shows metrics for the current project:

- **Cost**: Total cost for this project
- **Cost Trend**: 24-hour change (+ or -)
- **Tokens**: Total tokens used
- **Token Trend**: 24-hour change
- **API Calls**: Number of requests
- **Call Trend**: 24-hour change
- **Avg Response Time**: Average request latency
- **Time Trend**: 24-hour change
- **Cache Hit Rate**: Percentage of requests from cache
- **Savings**: Dollar amount saved by caching
- **Recent Activity**: Table of last API calls

### Machine-Wide Analytics Tab

Shows metrics across all projects on this machine:

- **Total Cost**: Sum of all project costs
- **Active Projects**: Number of projects with activity
- **Total Calls**: Total API requests across all projects
- **Total Tokens**: Total tokens used across all projects
- **Project List**: Table of all projects with costs and activity
- **Model Distribution**: Chart showing which models are used most
- **Cost Trend**: Chart showing cost over time

### Dashboard Refresh

Dashboard updates automatically every 5 seconds with live data:

- No manual refresh needed
- All charts update in real-time
- Connection status indicator shows if data is live

---

## Cost Tracking

CCO tracks all API usage and calculates costs in real-time.

### Supported Models

Pricing configured for all major models:

- **Anthropic**: Claude Opus, Sonnet, Haiku
- **OpenAI**: GPT-4, GPT-4 Turbo, GPT-3.5 Turbo
- **Groq**: Mixtral, Llama
- **Local**: Ollama (free)

### Cost Calculation

```
Cost = (input_tokens / 1000000) × input_price +
       (output_tokens / 1000000) × output_price
```

**Example:**
```
Prompt: 100 tokens
Response: 50 tokens
Model: Claude Opus

Input cost: (100 / 1M) × $15.00 = $0.0015
Output cost: (50 / 1M) × $75.00 = $0.00375
Total: $0.00525
```

### Cache-Write Pricing

Some models support cache-write tokens (cheaper writes, cheaper reads):

```
Cache-write tokens: (1000 / 1M) × $3.75 = $0.00375
Cache-read tokens: (500 / 1M) × $0.30 = $0.00015
```

### View Costs

```bash
# Project costs
curl http://localhost:3000/api/project/stats | jq '.cost'

# Machine-wide costs
curl http://localhost:3000/api/machine/stats | jq '.totalCost'

# Cost by model
curl http://localhost:3000/api/machine/stats | jq '.models'

# Cost by project
curl http://localhost:3000/api/machine/stats | jq '.projects'
```

### Export Analytics

```bash
# Export last 7 days as JSON
curl "http://localhost:3000/api/export/analytics?days=7" > analytics.json

# Export as CSV
curl "http://localhost:3000/api/export/csv?days=30" > analytics.csv

# Export specific project
curl "http://localhost:3000/api/export/project/my-project?format=json" > project.json
```

### Real-World Example

**Team of 10 developers, 1 month:**

Without CCO:
- 10 people × 500 requests/day × $0.005 = $25/day
- 25 × 30 days = **$750/month**

With CCO (70% cache hit rate):
- 10 people × 500 × 30% × $0.005 = $7.50/day
- 7.50 × 30 days = **$225/month**
- **Savings: $525/month (70% reduction)**

---

## Fallback Chains

Automatically try alternative models if the primary choice fails.

### How It Works

```
User requests: "claude-opus-4"
    ↓ (succeeds → done)
    ↓ (fails → try next)
Try: "claude-sonnet-3.5"
    ↓ (succeeds → done)
    ↓ (fails → try next)
Try: "gpt-4"
    ↓ (succeeds → done)
    ↓ (fails → error to user)
```

### Configure Fallback Chains

Edit `config/model-routing.json`:

```json
{
  "fallback_chain": {
    "claude-opus-4": ["claude-sonnet-3.5", "gpt-4"],
    "claude-sonnet-3.5": ["claude-haiku", "gpt-4-turbo"],
    "gpt-4": ["gpt-4-turbo", "claude-sonnet-3.5"],
    "ollama/llama3-70b": ["ollama/mistral-7b", "claude-haiku"]
  }
}
```

### Use Cases

1. **Redundancy**: If Claude API is down, use OpenAI automatically
2. **Cost optimization**: Try cheaper model first, fall back to expensive
3. **Resource constraints**: Use local model if available, fall back to API
4. **Capability matching**: Try powerful model first, degrade if needed

### No Code Changes Needed

User code stays the same:

```python
client.messages.create(
    model="claude-opus-4",  # Try primary
    messages=[...]          # Falls back automatically if it fails
)
```

---

## Self-Hosted Models

Run local LLMs via Ollama. All compute happens locally—no API costs.

### Installation

```bash
# Download and install Ollama
# https://ollama.ai

# Install a model
ollama pull llama3-70b
ollama pull mistral-7b

# Start Ollama
ollama serve
```

### Using Local Models

Configure in `config/model-routing.json`:

```json
{
  "routes": [
    {
      "pattern": "^ollama/",
      "provider": "ollama",
      "endpoint": "http://localhost:11434/api",
      "timeout_ms": 120000
    }
  ]
}
```

Use in code:

```python
client.messages.create(
    model="ollama/llama3-70b",
    messages=[{"role": "user", "content": "Hello"}]
)
```

### Cost Savings

CCO tracks savings by comparing to equivalent commercial model:

```
Using ollama/llama3-70b:
  Would cost $2.50 with Claude Sonnet
  → Saved: $2.50 per request
```

**Team savings with local LLMs:**

10 people, 500 requests/day, 50% using local:
- 10 × 500 × 0.50 × $0.005 = $12.50/day
- Total: **$375/month with local models**
- Savings vs pure Claude: **$375/month**

### Performance

- **Response time**: 5-30 seconds (depends on model, hardware)
- **Cost**: $0.00 (compute on your machine)
- **Privacy**: All data stays local

---

## API Compatibility

CCO implements the full Anthropic Messages API. Drop-in replacement.

### No Code Changes Needed

```python
# Before (using Claude API directly)
import anthropic
client = anthropic.Anthropic(api_key="sk-ant-...")

# After (using CCO)
client = anthropic.Anthropic(
    api_key="sk-ant-...",
    base_url="http://localhost:3000"  # ← Only change
)

# Everything else is identical
response = client.messages.create(
    model="claude-opus-4",
    messages=[{"role": "user", "content": "Hello"}]
)
```

### Supported Operations

- ✅ `messages.create()` - Full support
- ✅ `messages.stream()` - Streaming responses
- ✅ `models.list()` - List available models
- ✅ `batches.create()` - Batch processing
- ✅ Vision/images - Full support
- ✅ Tool use - Full support
- ✅ Caching API - Full support

### Streaming

```python
with client.messages.stream(
    model="claude-opus-4",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Write a poem"}]
) as stream:
    for text in stream.text_stream:
        print(text, end="", flush=True)
```

### Tool Use

```python
response = client.messages.create(
    model="claude-opus-4",
    tools=[
        {
            "name": "calculator",
            "description": "Perform math",
            "input_schema": {...}
        }
    ],
    messages=[{"role": "user", "content": "What's 2+2?"}]
)
```

---

## Performance Optimization

### Tips for Maximum Savings

1. **Standardize prompts**: Use templates for similar requests
2. **Batch similar work**: Process similar items together (cache hits)
3. **Use caching models**: Opus/Sonnet support prompt caching
4. **Monitor hit rate**: Target >70% cache hit rate
5. **Size cache appropriately**: Larger cache = more hits

### Example: Batch Processing

```python
# Bad: Different prompt each time (no cache hits)
for item in items:
    response = client.messages.create(
        model="claude-opus-4",
        messages=[{"role": "user", "content": f"Process: {item}"}]
    )

# Good: Template + varied data (high cache on template)
template = "Process the following:\n{item}"
for item in items:
    prompt = template.format(item=item)
    response = client.messages.create(
        model="claude-opus-4",
        messages=[{"role": "user", "content": prompt}]
    )
```

### Monitoring Effectiveness

```bash
# Monitor hit rate
watch -n 2 'curl -s http://localhost:3000/api/cache/stats | jq .hitRate'

# View top cached prompts
curl http://localhost:3000/api/cache/top-prompts?limit=10

# Check savings over time
curl "http://localhost:3000/api/export/analytics?days=7" | jq '.summary.cacheSavings'
```
