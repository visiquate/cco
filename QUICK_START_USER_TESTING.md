# CCO Quick Start - User Testing Guide

## TL;DR - Start Testing Now

```bash
# Option 1: Dashboard Mode (default)
cd /Users/brent/git/cc-orchestra
./cco/target/release/cco run
# Opens browser at http://127.0.0.1:3000/

# Option 2: Debug Mode (no browser)
cd /Users/brent/git/cc-orchestra
NO_BROWSER=1 ./cco/target/release/cco run --debug
```

---

## What You're Testing

**Version**: 2025.11.3+d9b27d1  
**Status**: ✅ All systems operational

### Key Features Working
- Real-time cost monitoring ($888.53 total)
- 56 agents loaded (Opus, Sonnet, Haiku)
- Model distribution: 58% Sonnet, 23% Haiku, 19% Opus
- HTTP API on port 3000
- Dashboard UI

---

## Testing Checklist

### 1. Daemon Startup
```bash
./cco/target/release/cco run --debug
```

**Expected**:
- Starts in < 2 seconds
- No fatal errors
- Shows "Starting Claude Code Orchestra 2025.11.3+..."
- Binds to port 3000

### 2. Health Check
```bash
curl http://127.0.0.1:3000/health | jq .
```

**Expected**:
```json
{
  "status": "ok",
  "version": "2025.11.3+d9b27d1"
}
```

### 3. View Statistics
```bash
curl http://127.0.0.1:3000/api/stats | jq '.project'
```

**Expected**:
- Cost: ~$888
- Calls: ~20,391
- Real data (not zeros)

### 4. View Agents
```bash
curl http://127.0.0.1:3000/api/agents | jq length
```

**Expected**: 56 agents

### 5. Dashboard UI
```
http://127.0.0.1:3000/
```

**Expected**:
- Cost chart with 30-day history
- Model distribution pie chart
- Real-time stats display

---

## Quick Commands

```bash
# Start daemon (debug mode)
NO_BROWSER=1 ./cco/target/release/cco run --debug

# Start daemon (TUI mode)
./cco/target/release/cco run

# Check version
./cco/target/release/cco --version

# Stop daemon
pkill -f "cco run"

# Check if running
ps aux | grep "[c]co run"

# Health check
curl http://127.0.0.1:3000/health

# Full stats
curl http://127.0.0.1:3000/api/stats | jq .

# Model distribution
curl http://127.0.0.1:3000/api/stats | jq '.chart_data.model_distribution'
```

---

## What to Test

1. **Startup Performance**
   - Does it start quickly?
   - Any error messages?

2. **Data Accuracy**
   - Do cost numbers match expectations?
   - Are model percentages correct?

3. **Dashboard UI**
   - Do charts render correctly?
   - Is data refreshing?

4. **Stability**
   - Does it stay running?
   - Any crashes or hangs?

5. **TUI Mode**
   - Does interactive mode work?
   - Can you navigate the interface?

---

## Known Minor Issues

1. Some agent config warnings during build (non-critical)
2. Log files not created in debug mode (stdout only)

**Neither affects functionality**

---

## Report Issues

If you find problems, note:
- What command you ran
- Expected vs actual behavior
- Any error messages
- Browser console errors (if using dashboard)

---

## Success Criteria

✅ Daemon starts without fatal errors  
✅ Health endpoint returns "ok"  
✅ Stats show real data ($888+, 20k+ calls)  
✅ Dashboard loads and displays charts  
✅ Daemon runs stable for 5+ minutes  

---

**Ready to test!** Start with debug mode for easiest troubleshooting.
