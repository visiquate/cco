# CCO Cost Savings Guide

Complete breakdown of how CCO saves money and how to maximize your savings.

## The Three Ways CCO Saves Money

### 1. Moka Cache (100% Savings on Cached Requests)

When CCO sees the same request twice, the second one is served from memory cache instantly, costing $0.00.

#### How it works

```
Request: "Explain machine learning"
├─ First time: Hits Claude API → $0.02 charged → Response cached
└─ Second time: Served from cache → $0.00 (100% free!)
```

#### Real Numbers: 70% Cache Hit Rate

```
Daily usage:
  100 API requests per day
  Cache hit rate: 70%

Cost calculation:
  30 cache hits    × $0.00 = $0.00
  70 cache misses  × $0.02 = $1.40
  Daily cost:              $1.40

Monthly:
  $1.40 × 30 days = $42/month

Without caching:
  100 requests × $0.02 = $2.00/day = $60/month

Savings: $18/month (30% reduction)
```

#### What Gets Cached?

CCO caches based on:
- **Model** (claude-opus-4)
- **Prompt** (exact message text)
- **Parameters** (temperature, max_tokens)

If any changes, it's a cache miss. This is intentional for accuracy.

#### Typical Cache Hit Rates

| Use Case | Hit Rate | Notes |
|----------|----------|-------|
| Development/Testing | 60-80% | Lots of repeated prompts |
| Research | 40-60% | More varied queries |
| Production | 30-50% | Less repetition |
| Brainstorming | 10-30% | Mostly unique requests |

#### Improving Cache Hit Rate

1. **Reuse templates**: Same base prompt with different data
2. **Standardize formats**: Consistent system prompts
3. **Batch operations**: Group similar requests together
4. **Cache warming**: Pre-load common prompts at startup

Example template approach:

```python
# Bad: Each prompt is unique
responses = []
for item in items:
    responses.append(claude.call(f"Summarize: {item}"))  # Low cache hit rate

# Good: Reuse template (high cache hit rate)
template = "Summarize this item:\n{item}"
for item in items:
    responses.append(claude.call(template.format(item=item)))
```

### 2. Claude Prompt Cache (90% Savings on Long Contexts)

Claude supports native caching at the API level. First 1024 tokens of context are cached by Claude servers.

#### How it works

```
Request: Long document to analyze
├─ First time:
│  ├─ Upload 10,000 token document
│  ├─ Claude charges for all 10,000 tokens
│  └─ Claude caches the document
└─ Second time:
   ├─ Same 10,000 token document
   ├─ Claude reads from cache (90% discount)
   └─ Charged only ~1,000 tokens (90% savings!)
```

#### Pricing Comparison

```
Model: claude-opus-4
Input token cost: $15 per 1M tokens
Cache read cost: $1.50 per 1M tokens (90% discount)

Example: 10,000 token cached document

Without cache:
  10,000 tokens × $15/1M = $0.15

With cache (first request):
  10,000 tokens × $15/1M + cache_write = $0.20 (slight premium)

With cache (subsequent requests):
  10,000 tokens × $1.50/1M = $0.015 (90% cheaper!)

Savings per cached read: $0.135 (90% reduction)
```

#### When to Use Prompt Cache

- Large system prompts (repeated per request)
- Long documents analyzed multiple times
- Reference materials reused across requests
- Codebases loaded for each analysis

Example: Code review system

```python
# Without cache: Each review hits full $1.00 cost
CODE = """(10,000+ lines of your codebase)"""
for pr in pull_requests:
    response = claude.call(f"Review code:\n{CODE}\n{pr['code']}")

# With cache: First review $1.10, subsequent $0.15
# 100 reviews: $1.10 + (99 × $0.15) = $15.95
# Without cache: 100 × $1.00 = $100.00
# Savings: $84.05 (84% reduction!)
```

#### Claude Cache Pricing by Model

| Model | Input | Cache Read | Cache Write | Savings |
|-------|-------|-----------|-------------|---------|
| claude-opus-4 | $15.00 | $1.50 | $18.75 | 90% |
| claude-sonnet-3.5 | $3.00 | $0.30 | $3.75 | 90% |
| claude-haiku | $0.25 | $0.05 | $0.30 | 80% |

### 3. Self-Hosted Models (100% API Cost Savings)

Run open-source models locally (Llama, Mistral, etc.). CCO tracks savings compared to commercial models.

#### How it works

```
Request: "Summarize this article"
├─ Option 1: Send to claude-opus-4
│  └─ Cost: $0.02
└─ Option 2: Route to ollama/llama3-70b
   └─ Cost: $0.00 (runs on your hardware)
   └─ CCO reports: "Saved $0.02 vs claude-opus-4"
```

#### Cost Breakdown: Self-Hosted Models

```
Monthly usage: 100,000 API calls

Option A: Pure Claude
  100,000 × $0.005 = $500/month

Option B: 50% Claude + 50% Self-Hosted Llama
  50,000 × $0.005 = $250
  50,000 × $0.00 = $0
  Total: $250/month
  Savings: $250/month (50% reduction)

Option C: 100% Self-Hosted Llama
  100,000 × $0.00 = $0/month
  Infrastructure cost: ~$30/month (GPU)
  Total: $30/month
  Savings: $470/month (94% reduction!)
```

#### Self-Hosted Model Comparison

| Model | Quality | Speed | Cost | When to Use |
|-------|---------|-------|------|------------|
| Llama 3 8B | Good | Fast | Free | Text generation, summarization |
| Llama 3 70B | Excellent | Medium | Free | Complex reasoning, code |
| Mistral 7B | Good | Very fast | Free | Fast responses |
| Mixtral 8x7B | Great | Medium | Free | Balanced tasks |

#### Hardware Requirements

```
Model                Memory    GPU      Cost/month
Llama 3 8B          16GB      6GB VRAM $10 (AWS g4)
Llama 3 70B         64GB      40GB VRAM $50 (AWS a100)
Mixtral 8x7B        48GB      24GB VRAM $30 (AWS a100)

Running on your machine:
  RTX 4090           24GB VRAM  $1500 (one-time)
  M1/M2 Mac          8-16GB RAM ~$0 (already own)
```

#### Setup Example: Ollama + CCO

```bash
# 1. Install Ollama (macOS/Linux/Windows)
curl https://ollama.ai/install.sh | sh

# 2. Download a model
ollama pull llama3:70b

# 3. Start Ollama
ollama serve

# 4. Configure CCO to use it
# Add to config/model-routing.json:
{
  "pattern": "^custom/",
  "provider": "ollama",
  "endpoint": "http://localhost:11434"
}

# 5. Use it in code
client.messages.create(
    model="custom/llama3:70b",  # Routes to Ollama
    messages=[...]
)
```

## Real-World Savings Examples

### Example 1: Small Team (5 people)

**Before CCO:**
- 5 people × 200 requests/day × $0.005 = $5/day = $150/month

**After CCO (70% cache hit):**
- 5 people × 200 requests × 30% × $0.005 = $1.50/day = $45/month
- Savings: $105/month (70% reduction)

**With self-hosted:**
- 5 people × 200 requests × 50% × $0.005 = $2.50/day = $75/month
- Savings: $75/month (50% reduction)

### Example 2: Large Team (50 people)

**Before CCO:**
- 50 people × 500 requests/day × $0.005 = $125/day = $3,750/month

**After CCO (70% cache hit):**
- 50 people × 500 requests × 30% × $0.005 = $37.50/day = $1,125/month
- Savings: $2,625/month (70% reduction)

**With self-hosted + cache:**
- 50 people × 500 requests × 15% × $0.005 = $18.75/day = $562.50/month
- Infrastructure: $100/month
- Total: $662.50/month
- Savings: $3,087.50/month (82% reduction!)

### Example 3: Production API Server

**Before CCO:**
- 1M requests/month × $0.005 = $5,000/month

**After CCO (50% cache hit):**
- 500K cached + 500K API × $0.005 = $2,500/month
- Savings: $2,500/month (50% reduction)

**With prompt caching (10,000 token docs):**
- First request: $0.15
- Next 99 requests: 99 × $0.015 = $1.49
- Per 100 requests: $1.64 vs $0.50 (normal)
- Savings: 68% reduction per cached prompt

## Savings Dashboard

CCO's analytics dashboard shows real-time savings:

```
┌─────────────────────────────────────────┐
│        CCO Savings Overview              │
├─────────────────────────────────────────┤
│ Total API Calls:      1,234             │
│ Cache Hits:             864 (70%)       │
│ Cache Misses:           370 (30%)       │
│                                          │
│ Actual Cost:           $1.85            │
│ Would-Be Cost:         $6.17            │
│ Savings:               $4.32 (70%)      │
│                                          │
│ Savings This Month:    $125.45          │
│ All-Time Savings:      $1,234.56        │
└─────────────────────────────────────────┘
```

### Understanding the Numbers

**Actual Cost**: What you're really paying (cache misses + cache overhead)

**Would-Be Cost**: What it would cost without caching

**Savings**: The difference (direct money saved)

## Optimization Strategies

### Strategy 1: Maximize Cache Hit Rate

Measure current rate:
```bash
curl http://localhost:8000/api/cache/stats
# Returns: {"hitRate": 0.65}  # 65% hit rate
```

Optimize by:
1. Standardizing prompts
2. Reusing templates
3. Caching system messages
4. Batching similar requests

Expected improvement: 65% → 75%+

### Strategy 2: Use Prompt Cache for Documents

Identify opportunities:
- Long system prompts used repeatedly
- Reference documents analyzed multiple times
- Code analyzed for multiple purposes

Implementation:
```python
# Request with prompt caching enabled
response = client.messages.create(
    model="claude-opus-4",
    system=[{
        "type": "text",
        "text": LONG_DOCUMENT,
        "cache_control": {"type": "ephemeral"}  # Enable caching
    }],
    messages=[{"role": "user", "content": "Analyze this"}]
)
```

### Strategy 3: Smart Model Routing

For non-critical tasks, route to cheaper/free models:

```json
{
  "routes": [
    {
      "pattern": "^fast/.*",
      "provider": "ollama",
      "endpoint": "http://localhost:11434"
    },
    {
      "pattern": "^smart/.*",
      "provider": "anthropic",
      "endpoint": "https://api.anthropic.com/v1"
    }
  ]
}
```

```python
# Fast task (use Ollama)
response = client.messages.create(
    model="fast/llama3",
    messages=[...]  # Cost: $0
)

# Complex task (use Claude)
response = client.messages.create(
    model="smart/claude-opus",
    messages=[...]  # Cost: $0.02
)
```

### Strategy 4: Tiered Service Quality

```python
def analyze(data, quality="auto"):
    if quality == "fast":
        model = "custom/llama3"  # Free
    elif quality == "balanced":
        model = "claude-haiku"   # Cheapest
    elif quality == "best":
        model = "claude-opus-4"  # Best
    else:  # auto
        # Use fast for simple tasks
        if len(data) < 1000:
            model = "custom/llama3"
        else:
            model = "claude-opus-4"

    return client.messages.create(
        model=model,
        messages=[{"role": "user", "content": data}]
    )
```

## Cost Comparison: Before & After CCO

```
Monthly usage: 100,000 API calls

┌─────────────────┬──────────────┬──────────────┬──────────────┐
│ Strategy        │ Requests     │ Cost         │ vs Direct    │
├─────────────────┼──────────────┼──────────────┼──────────────┤
│ Direct Claude   │ 100,000      │ $500/month   │ Baseline     │
│ + Cache (70%)   │ 100,000      │ $150/month   │ 70% savings  │
│ + Prompt Cache  │ 100,000      │ $50/month    │ 90% savings  │
│ + Self-Hosted   │ 100,000      │ $50/month    │ 90% savings  │
│ All Combined    │ 100,000      │ $15/month    │ 97% savings  │
└─────────────────┴──────────────┴──────────────┴──────────────┘
```

## ROI Calculation

### When CCO Pays for Itself

```
Scenario: Deploy CCO on AWS

One-time setup:
  Server instance: $0 (t2.medium included in tier)
  Configuration: 1 hour × $0 (included)
  Total: $0

Monthly costs:
  AWS t2.medium: $30/month
  Monitoring: $10/month
  Backups: $5/month
  Total: $45/month

Savings from caching: $350/month (70% of $500)

ROI: $350 - $45 = $305 net savings/month
Payback period: Immediate (first month positive)
```

### When to Deploy CCO

| Situation | ROI | Recommendation |
|-----------|-----|-----------------|
| <$100/month API | Low | Optional (still beneficial) |
| $100-500/month | High | Highly recommended |
| $500-2000/month | Very High | Essential |
| >$2000/month | Exceptional | Must have |

## Cost Monitoring

### Track Daily Costs

```bash
# Daily cost trend
curl http://localhost:8000/api/analytics/daily?days=30 | jq '.daily_costs'

# Weekly summary
curl http://localhost:8000/api/analytics/weekly | jq '.this_week'

# Monthly projection
curl http://localhost:8000/api/analytics/projection
```

### Budget Alerts

```bash
# Set budget limit
curl -X POST http://localhost:8000/api/budget/set \
  -d '{"limit": 500, "period": "month"}'

# Alerts when approaching limit
# Receives email at 75% and 95% of budget
```

### Cost Attribution by Project

```bash
# See which projects cost most
curl http://localhost:8000/api/analytics/by-project | head -20

# Project: frontend
#   Requests: 45,234
#   Cost: $234.56
#   Cache Hits: 30,000 (66%)

# Project: backend
#   Requests: 23,456
#   Cost: $123.45
#   Cache Hits: 15,000 (64%)
```

## Next Steps

1. **Deploy CCO** - See README.md for setup
2. **Monitor savings** - Check dashboard at http://localhost:8000
3. **Optimize** - Use strategies in this guide
4. **Scale** - Add self-hosted models for more savings

## FAQ

**Q: Will caching affect response quality?**
A: No, cached responses are identical to fresh ones.

**Q: What if I need up-to-date information?**
A: Cache only applies to identical requests. Different prompts always get fresh responses.

**Q: Can I use Claude's native cache with CCO?**
A: Yes, CCO passes through Claude's cache headers automatically.

**Q: How long are responses cached?**
A: Default 1 hour, configurable via `--cache-ttl` flag.

**Q: Does cache increase latency?**
A: Cache hits are typically <5ms (99% faster than API).

**Q: What about privacy?**
A: Cache is stored locally, never sent to third parties.

See [README.md](./README.md) for more details.
