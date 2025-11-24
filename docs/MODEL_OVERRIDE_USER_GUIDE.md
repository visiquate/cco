# CCO Model Override - User Guide

## What is Model Override?

Model Override is a feature that automatically rewrites LLM model requests at the proxy layer. When you request a specific model (like Claude Sonnet), the CCO proxy can transparently redirect that request to a different, more cost-effective model (like Claude Haiku) without any changes to your code.

This happens transparently—Claude Code doesn't know the override happened. You get the same interface and functionality, but with significant cost savings.

## Why Use Model Override?

Model Override delivers substantial cost reductions while maintaining quality:

**Cost Savings Example:**
- **Before**: Using Claude Sonnet costs $3.00 per 1M input tokens
- **After**: Redirected to Claude Haiku costs $0.80 per 1M input tokens
- **Savings**: 73% cost reduction ($2.20 per 1M tokens)

**Real-World Impact:**
For a typical development workflow with 5 agents running 50 times per month:
- Current cost (Sonnet): ~$36/month
- With overrides (Haiku): ~$9.50/month
- Monthly savings: $26.50 (73% reduction)
- Annual savings: $318

## How It Works

Model overrides operate transparently at the HTTP proxy layer:

```
Your Claude Code Request
         ↓
    CCO Proxy
         ↓
    Check: Is this model in override rules?
         ↓
    Yes → Rewrite model parameter
    No  → Pass through unchanged
         ↓
    Send to Anthropic API (with rewritten or original model)
         ↓
    Receive response
         ↓
    Return to Claude Code
```

**Key Point:** Your code never changes. The override happens at the proxy, before the request reaches Anthropic's API.

## Getting Started

### Prerequisites

- CCO proxy installed and built (`cco` directory)
- Claude Code configured to use CCO
- ANTHROPIC_API_KEY set in environment

### Step 1: Verify CCO is Installed

```bash
cd /Users/brent/git/cc-orchestra/cco
ls -la config/model-overrides.toml
```

You should see the configuration file. If not, see [QUICK_START.md](./QUICK_START.md).

### Step 2: Start CCO with Model Overrides

```bash
# Build CCO (if not already built)
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# Start CCO on default port (3000)
./target/release/cco run --port 3000

# Or specify a custom port
./target/release/cco run --port 8000
```

You should see output like:
```
CCO started on http://localhost:3000
Dashboard: http://localhost:3000
Model overrides: enabled
Override rules loaded: 3
```

### Step 3: Configure Claude Code to Use CCO

```bash
# Set the API endpoint to point to CCO instead of Anthropic directly
export ANTHROPIC_API_BASE_URL=http://localhost:3000/v1
export ANTHROPIC_API_KEY=your-actual-api-key-here

# Verify it's working
curl http://localhost:3000/health
```

Response should be:
```json
{
  "status": "ok",
  "overrides_enabled": true,
  "override_rules": 3
}
```

### Step 4: Verify Model Overrides Are Active

When you use Claude Code, check that overrides are being applied:

```bash
# Check override statistics
curl http://localhost:3000/api/overrides/stats

# Response example:
{
  "total_overrides": 47,
  "overrides_by_model": {
    "claude-sonnet-4.5-20250929": {
      "overridden_to": "claude-haiku-4-5-20251001",
      "count": 47
    }
  }
}
```

You can also monitor the CCO dashboard at `http://localhost:3000` to see real-time override activity.

## Configuration

### Default Configuration

Model overrides are configured in `cco/config/model-overrides.toml`:

```toml
[overrides]
# Enable or disable model overrides globally
enabled = true

# Model rewrite rules
# Format: ["original_model", "replacement_model"]
rules = [
    # Sonnet → Haiku rewrites (75% cost savings)
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
    ["claude-sonnet-4", "claude-haiku-4-5-20251001"],
    ["claude-sonnet-3.5", "claude-haiku-4-5-20251001"],
]

[analytics]
# Log all model overrides to console
log_overrides = true

# Track override statistics
track_statistics = true

# Report format: "json", "text", or "silent"
report_format = "json"
```

### Adding Custom Override Rules

To override additional models, add to the `rules` array in `model-overrides.toml`:

```toml
rules = [
    # Original rules
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # Add your custom rules below:
    ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],  # Opus → Sonnet
    ["gpt-4-turbo", "gpt-3.5-turbo"],                             # OpenAI override
]
```

### Disabling Overrides

To temporarily disable all model overrides:

```toml
[overrides]
enabled = false  # Set to false to disable overrides
```

Then restart CCO:
```bash
# Kill existing process
pkill cco

# Start without overrides
./target/release/cco run --port 3000
```

### Disabling Specific Rules

To disable a specific override rule without disabling all overrides, comment it out:

```toml
rules = [
    # This rule is active
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],

    # This rule is disabled (commented out)
    # ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],
]
```

Then restart CCO for changes to take effect.

## Monitoring Overrides

### Dashboard

The easiest way to monitor overrides is through the CCO dashboard:

1. Open `http://localhost:3000` in your browser
2. Look for the "Model Overrides" section
3. You'll see:
   - Number of overrides applied
   - Which models have been overridden
   - Cost savings breakdown

### API Endpoint

Query the `/api/overrides/stats` endpoint:

```bash
# Get current override statistics
curl http://localhost:3000/api/overrides/stats

# Response:
{
  "total_overrides": 47,
  "overrides_by_model": {
    "claude-sonnet-4.5-20250929": {
      "overridden_to": "claude-haiku-4-5-20251001",
      "count": 47,
      "percentage": 100
    }
  }
}
```

### Console Logs

When `log_overrides = true`, you'll see messages like:

```
🔄 Model override: claude-sonnet-4.5-20250929 → claude-haiku-4-5-20251001
📝 Processing chat request for model: claude-haiku-4-5-20251001
📊 Override count: 47 (cost saved: $18.50)
```

## Performance Impact

Model overrides have minimal performance overhead:

| Metric | Impact |
|--------|--------|
| Override lookup | < 1ms (negligible) |
| Request latency | No additional latency |
| Cache behavior | Unchanged (cache uses overridden model) |
| Memory usage | Negligible (small config in memory) |

**Key Point:** Haiku is actually faster than Sonnet for most tasks, so you may see improved response times even with cost savings.

## Understanding Cache Keys with Overrides

When model overrides are enabled, the cache key includes the **overridden model**, not the original:

```
Request 1: model=sonnet → overridden to haiku → cached as "haiku"
Request 2: model=sonnet → overridden to haiku → cache hit! (0 cost)
```

This means:
- ✅ Cache works correctly with overrides enabled
- ✅ Cached responses use the overridden model price
- ✅ Cost tracking is accurate

## Cost Savings Calculator

Use this to estimate your monthly savings:

### Current Setup (Without Overrides)

1. **Determine your monthly token usage:**
   - Count total requests per month
   - Estimate average tokens per request (usually 50k-300k)
   - Total tokens = requests × avg tokens

2. **Calculate cost:**
   ```
   Cost = (input_tokens × $3.00/1M) + (output_tokens × $15.00/1M)
   ```

### With Overrides

1. **Same requests, different pricing:**
   ```
   Cost = (input_tokens × $0.80/1M) + (output_tokens × $4.00/1M)
   ```

2. **Calculate savings:**
   ```
   Savings = Current Cost - Override Cost
   Percentage = (Savings / Current Cost) × 100
   ```

### Example

**Scenario:** 5 agents, 50 runs/month, 238k tokens/run

**Current (Sonnet):**
- Tokens: 238k × 50 = 11.9M/month
- Cost: (8.4M × $3/1M) + (3.5M × $15/1M) = $77.10/month

**With Overrides (Haiku):**
- Same tokens, different pricing
- Cost: (8.4M × $0.80/1M) + (3.5M × $4/1M) = $20.20/month

**Savings:**
- Monthly: $56.90 (74% reduction)
- Annual: $683

See [COST_ANALYSIS.md](./COST_ANALYSIS.md) for more detailed calculations.

## FAQ

### Q: Will this affect output quality?

A: No, not significantly. Claude Haiku is highly capable and delivers excellent results. The main difference is that Haiku is optimized for cost and speed rather than complex reasoning. For the Claude Orchestra use cases (documentation, basic coding, code review), Haiku performs excellently.

### Q: What if I want to use Sonnet for specific tasks?

A: You have two options:

1. **Remove the override rule** - Edit `model-overrides.toml` and comment out the Sonnet rule
2. **Disable overrides temporarily** - Set `enabled = false` in the config

Either way, restart CCO for changes to take effect.

### Q: Can I override multiple models?

A: Yes! Add as many rules as you need to the `rules` array:

```toml
rules = [
    ["claude-sonnet-4.5-20250929", "claude-haiku-4-5-20251001"],
    ["claude-opus-4-1-20250805", "claude-sonnet-4.5-20250929"],
    ["gpt-4-turbo", "gpt-3.5-turbo"],
]
```

### Q: What happens to cached responses?

A: Cache keys use the **overridden model**, so responses are properly cached by the actual model used. This means:
- Responses cached while using overrides continue to work correctly
- Switching overrides on/off will use/miss the cache appropriately

### Q: How do I know when an override happens?

A: Check the logs in one of three ways:

1. **Console output** - Watch the CCO terminal for "🔄 Model override" messages
2. **Dashboard** - Visit `http://localhost:3000` to see override statistics
3. **API** - Query `http://localhost:3000/api/overrides/stats`

### Q: Can overrides fail?

A: No. Model overrides are a simple string replacement that happens before the request is sent. If the override fails, the request is not sent at all (fail-safe behavior).

### Q: Do overrides work with Claude Code?

A: Yes! Claude Code uses the Anthropic API client, which connects through the CCO proxy. As long as Claude Code is configured to use `ANTHROPIC_API_BASE_URL=http://localhost:3000/v1`, overrides will apply automatically.

### Q: What if I need to temporarily disable overrides?

A: Edit `model-overrides.toml` and set `enabled = false`, then restart CCO:

```toml
[overrides]
enabled = false
```

### Q: Can I use overrides with other CCO features?

A: Yes! Overrides work alongside:
- Caching (Moka)
- Analytics
- Multi-model routing
- Fallback chains
- All other CCO features

## Troubleshooting

### Overrides Not Being Applied

**Problem:** You're still being charged Sonnet prices, or the override count stays at 0.

**Solutions:**

1. **Verify overrides are enabled in config:**
   ```toml
   [overrides]
   enabled = true  # Make sure this is true
   ```

2. **Check model names match exactly:**
   ```bash
   # Model names are case-sensitive
   # These do NOT match:
   # "claude-sonnet-4.5" ≠ "Claude-Sonnet-4.5"
   ```

3. **Restart CCO after configuration changes:**
   ```bash
   pkill cco
   ./target/release/cco run --port 3000
   ```

4. **Verify Claude Code is using CCO:**
   ```bash
   echo $ANTHROPIC_API_BASE_URL
   # Should output: http://localhost:3000/v1
   ```

5. **Check the health endpoint:**
   ```bash
   curl http://localhost:3000/health
   # Should include: "overrides_enabled": true
   ```

### Claude Code Can't Reach CCO

**Problem:** "Connection refused" or "timeout" errors when using Claude Code.

**Solutions:**

1. **Verify CCO is running:**
   ```bash
   curl http://localhost:3000/health
   ```

2. **Check the port is correct:**
   ```bash
   # If you started CCO on port 8000:
   export ANTHROPIC_API_BASE_URL=http://localhost:8000/v1
   ```

3. **Check firewall allows localhost:**
   ```bash
   # Try connecting to the server
   curl -v http://localhost:3000/health
   ```

4. **Make sure CCO started successfully:**
   ```bash
   # Look for "CCO started on http://localhost:3000" in the console
   ```

### Wrong Models Being Overridden

**Problem:** The wrong model is being overridden, or the override rules aren't working as expected.

**Solutions:**

1. **Check the exact model names:**
   ```bash
   # Print the config to verify model names
   grep -A 20 "^rules" /Users/brent/git/cc-orchestra/cco/config/model-overrides.toml
   ```

2. **Verify the override rules are in correct format:**
   ```toml
   # Correct format:
   rules = [
       ["original-model", "replacement-model"],
   ]

   # NOT:
   rules = [
       "original-model" => "replacement-model",
   ]
   ```

3. **Check for typos in model names** - They are case-sensitive

4. **Verify the replacement model exists:**
   - Make sure `claude-haiku-4-5-20251001` is a valid model
   - Check https://docs.anthropic.com for current model names

5. **Restart CCO to load the new rules:**
   ```bash
   pkill cco
   ./target/release/cco run --port 3000
   ```

### Monitoring and Debugging

**Check current overrides:**
```bash
# Get statistics
curl http://localhost:3000/api/overrides/stats | jq

# Get health status
curl http://localhost:3000/health | jq
```

**Enable verbose logging:**

Edit `model-overrides.toml`:
```toml
[analytics]
log_overrides = true
report_format = "json"
```

Then restart CCO and watch the console for detailed override messages.

## Next Steps

1. **[Cost Analysis](./COST_ANALYSIS.md)** - Detailed cost breakdowns and ROI calculations
2. **[Configuration Reference](./MODEL_OVERRIDE_CONFIG_REFERENCE.md)** - Complete configuration options
3. **[Operator Guide](./MODEL_OVERRIDE_OPERATOR_GUIDE.md)** - Deployment and operations
4. **[CCO README](./README.md)** - Complete CCO documentation

## Getting Help

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section above
2. Review the dashboard at `http://localhost:3000` for insights
3. Check CCO logs: Look at console output when CCO is running
4. See [MODEL_OVERRIDE_OPERATOR_GUIDE.md](./MODEL_OVERRIDE_OPERATOR_GUIDE.md) for operational procedures

---

**Ready to save money?** Start with [Step 1: Verify CCO is Installed](#step-1-verify-cco-is-installed) and follow the setup guide above.
