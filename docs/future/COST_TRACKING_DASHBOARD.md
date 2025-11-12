# ccproxy Cost Tracking Dashboard

## Overview

The **Cost Tracking Dashboard** provides real-time visibility into your hybrid routing savings. It tracks:
- Requests to Anthropic API (paid)
- Requests to local Ollama models (free)
- Actual costs vs hypothetical all-Anthropic costs
- Cost savings and reduction percentage

## Accessing the Dashboard

### Public URL (Recommended)
```
https://coder.visiquate.com/dashboard/
```

**No authentication required** - The dashboard is read-only and accessible via HTTPS through Cloudflare tunnel.

### Local Access (Mac mini only)
```bash
curl http://127.0.0.1:8082/
```

## Dashboard Features

### Main Metrics

1. **Total Savings Display**
   - Large highlighted number showing cumulative savings
   - Percentage cost reduction vs all-Anthropic pricing
   - Auto-updates every 30 seconds

2. **Request Distribution Card**
   - Anthropic API requests (paid)
   - Local Ollama requests (free)
   - Total request count

3. **Cost Breakdown Card**
   - Actual cost (hybrid routing)
   - Hypothetical cost (if all Anthropic)
   - Total savings amount

4. **Model Usage Breakdown**
   - Individual model usage with badges (PAID/FREE)
   - Request counts per model
   - Cost per model (for paid models only)

### API Endpoint

Get raw JSON data:
```bash
curl https://coder.visiquate.com/dashboard/api/stats

# Response format:
{
  "actual_cost": 0.04,
  "anthropic_requests": 2,
  "hypothetical_cost": 0.12,
  "local_requests": 10,
  "models": {
    "claude-opus-4": {"count": 2, "cost": 0.04, "is_paid": true},
    "claude-3-5-sonnet": {"count": 10, "cost": 0.0, "is_paid": false}
  },
  "savings_percent": 66.7,
  "total_requests": 12,
  "total_saved": 0.08,
  "last_updated": "2025-11-04 18:22:58"
}
```

## How It Works

### Data Source
The dashboard parses `/Users/brent/ccproxy/logs/litellm.log` to extract:
- Model names from API requests
- Request counts
- Usage patterns

### Cost Calculation

**Pricing per 1M tokens (estimates)**:
- `claude-opus-4`: Input $15, Output $75 (avg ~$0.02/request)
- `claude-sonnet-4-5`: Input $3, Output $15 (avg ~$0.004/request)
- Local Ollama models: $0 (completely free)

**Formula**:
```
Actual Cost = (Opus requests × $0.02) + (Sonnet requests × $0.004)
Hypothetical Cost = Total requests × $0.01 (average)
Savings = Hypothetical Cost - Actual Cost
Savings % = 100 × (1 - Actual Cost / Hypothetical Cost)
```

### Auto-Refresh
- Dashboard auto-refreshes every 30 seconds
- No manual refresh needed
- Real-time tracking as requests come in

## Architecture

```
┌─────────────────────────────────────────────────────┐
│     https://coder.visiquate.com/dashboard/          │
│                 (Public Access)                     │
└─────────────────────┬───────────────────────────────┘
                                                      │
                 Cloudflare Tunnel
                                                      │
┌─────────────────────▼───────────────────────────────┐
│                  Traefik                            │
│    - Routes /dashboard → dashboard service          │
│    - StripPrefix middleware                         │
│    - No auth required (read-only)                   │
└─────────────────────┬───────────────────────────────┘
                                                      │
┌─────────────────────▼───────────────────────────────┐
│         Dashboard (Flask + waitress)                │
│         Port: 127.0.0.1:8082                        │
│                                                     │
│    - Parses /Users/brent/ccproxy/logs/litellm.log   │
│    - Calculates costs and savings                   │
│    - Serves HTML dashboard + JSON API               │
└──────────────────────────────────────────────────────┘
```

## Management

### Start Dashboard
```bash
ssh brent@192.168.9.123
cd /Users/brent/ccproxy
./start-dashboard.sh
```

### Stop Dashboard
```bash
ssh brent@192.168.9.123
pkill -f waitress
```

### Check Status
```bash
# Check if dashboard is running
ssh brent@192.168.9.123 'ps aux | grep waitress | grep -v grep'

# Check dashboard logs
ssh brent@192.168.9.123 'tail -50 /Users/brent/ccproxy/logs/dashboard.log'

# Test local endpoint
ssh brent@192.168.9.123 'curl -s http://127.0.0.1:8082/api/stats'
```

### Restart Dashboard
```bash
ssh brent@192.168.9.123 'pkill -f waitress && cd /Users/brent/ccproxy && nohup ./start-dashboard.sh > logs/dashboard.log 2>&1 &'
```

## Files

### Dashboard Application
- **Location**: `/Users/brent/ccproxy/dashboard.py`
- **Type**: Flask web application
- **Server**: waitress WSGI server
- **Port**: 8082 (localhost only)

### Startup Script
- **Location**: `/Users/brent/ccproxy/start-dashboard.sh`
- **Purpose**: Starts dashboard with waitress

### Traefik Configuration
- **Location**: `/Users/brent/git/local-ai-models/traefik-dynamic.yml`
- **Routes**: `/dashboard` → dashboard service (port 8082)
- **Middleware**: StripPrefix to remove `/dashboard` prefix

### Logs
- **Location**: `/Users/brent/ccproxy/logs/dashboard.log`
- **Purpose**: Dashboard startup/error logs

## Customization

### Update Pricing Estimates

Edit `/Users/brent/ccproxy/dashboard.py`:

```python
PRICING = {
    'claude-opus-4': {'input': 15.00, 'output': 75.00},
    'claude-sonnet-4-5': {'input': 3.00, 'output': 15.00},
    # Update these values as Anthropic pricing changes
}
```

Then restart the dashboard.

### Change Refresh Interval

Edit the JavaScript in `dashboard.py`:

```javascript
// Auto-refresh every 30 seconds
setTimeout(() => location.reload(), 30000);  // Change 30000 to desired milliseconds
```

## Troubleshooting

### Dashboard Not Loading

1. **Check if dashboard service is running:**
   ```bash
   ssh brent@192.168.9.123 'ps aux | grep waitress'
   ```

2. **Check dashboard logs:**
   ```bash
   ssh brent@192.168.9.123 'cat /Users/brent/ccproxy/logs/dashboard.log'
   ```

3. **Restart dashboard:**
   ```bash
   ssh brent@192.168.9.123 'pkill -f waitress && cd /Users/brent/ccproxy && nohup ./start-dashboard.sh > logs/dashboard.log 2>&1 &'
   ```

### Showing $0.00 Savings

This is normal if:
- No requests have been made yet
- All requests are going to local Ollama models
- ccproxy logs are empty

**Test hybrid routing** by making a request with `claude-opus-4` model to generate some paid API usage.

### 404 Not Found

If you see 404 errors:

1. **Verify Traefik routing:**
   ```bash
   ssh brent@192.168.9.123 'cat /Users/brent/git/local-ai-models/traefik-dynamic.yml | grep -A 5 dashboard'
   ```

2. **Reload Traefik:**
   ```bash
   ssh brent@192.168.9.123 'pkill -HUP traefik'
   ```

### 503 Service Unavailable

If you see 503 errors:

1. **Check dashboard is listening on port 8082:**
   ```bash
   ssh brent@192.168.9.123 'netstat -an | grep 8082'
   ```

2. **Test local endpoint:**
   ```bash
   ssh brent@192.168.9.123 'curl -s http://127.0.0.1:8082/api/stats'
   ```

## Example Usage

### View Dashboard in Browser

Open in your browser:
```
https://coder.visiquate.com/dashboard/
```

You should see:
- Main dashboard with dark theme (VisiQuate brand colors)
- Savings metric prominently displayed
- Request distribution and cost breakdown
- Model usage table

### Monitor Savings via Command Line

```bash
# Get current stats
curl -s https://coder.visiquate.com/dashboard/api/stats | jq

# Watch savings in real-time (refresh every 10 seconds)
watch -n 10 'curl -s https://coder.visiquate.com/dashboard/api/stats | jq ".total_saved, .savings_percent"'
```

### Integration with Scripts

```bash
#!/bin/bash
# Example: Send savings to Slack

STATS=$(curl -s https://coder.visiquate.com/dashboard/api/stats)
SAVINGS=$(echo "$STATS" | jq -r '.total_saved')
PERCENT=$(echo "$STATS" | jq -r '.savings_percent')

echo "Current savings: \$$SAVINGS ($PERCENT% reduction)"
```

## Future Enhancements

Potential improvements:
- Add charts/graphs for cost trends over time
- Store historical data in SQLite database
- Add date range filtering
- Export data to CSV
- Slack/email notifications when savings exceed thresholds
- Detailed token usage tracking (input vs output)
- Cost projections and budgeting

## Support

For issues or questions:
- Check logs: `/Users/brent/ccproxy/logs/dashboard.log`
- Verify ccproxy logs: `/Users/brent/ccproxy/logs/litellm.log`
- Review deployment: [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md)

## Related Documentation

- [Hybrid Routing Setup](HYBRID_ROUTING_SETUP.md)
- [Orchestra Roster (TDD)](ORCHESTRA_ROSTER_TDD.md)
- [Deployment Status](../DEPLOYMENT_STATUS.md)
