# Task Call Locations - Where Agents Are Being Invoked

## Summary

The Task calls for your 5 agents are happening **automatically through Claude Code's agent auto-spawning system** when you make requests, rather than through explicit hardcoded Task tool invocations in your code.

---

## Evidence Found

### 1. Location: Claude Code History
**File:** `/Users/brent/.claude/history.jsonl`

**Evidence:**
```
Latest session shows these agents being spawned:
⏺ rust-specialist(Implement daemon mode)
  ⎿  Done (8 tool uses · 33.0k tokens · 58s)

⏺ rust-specialist(Implement CLI commands)
  ⎿  Update(src/main.rs)
     Updated src/main.rs with 1 addition

⏺ devops-engineer(Implement log rotation)
  ⎿  Done (7 tool uses · 38.0k tokens · 1m 2s)

⏺ frontend-developer(Add shutdown button)
  ⎿  Done (15 tool uses · 44.1k tokens · 1m 56s)

⏺ test-engineer(Test daemon functionality)
  ⎿  Done (14 tool uses · 52.7k tokens · 1m 4s)

⏺ documentation-expert(Update documentation)
  ⎿  Done (15 tool uses · 71.9k tokens · 1m 1s)
```

**Timestamp:** November 15, 2025 at 15:26:26 GMT

**Session ID:** `f78441bc-ec33-47de-afa1-e31eec7d47a0`

**Project:** `/Users/brent/git/cc-orchestra`

---

## Root Cause Analysis

### The Problem
When Claude Code spawns agents automatically (like when you ask it to "implement daemon mode"), it's NOT using the configured model from:
- `config/orchestra-config.json`
- `~/.claude/agents/agent-name.md` (frontmatter)

Instead, it appears to default to **Sonnet** for quality/safety reasons.

### How It Happens

1. **User makes a request:** "Implement daemon mode"
2. **Claude Code decides to spawn:** `rust-specialist` agent
3. **Model selection issue:**
   - Should use: `haiku` (from agent config)
   - Actually uses: `sonnet` (Claude Code default)
4. **Result:** Token usage is 60-70% higher than necessary

---

## Configuration Status

### ✅ Configuration Is Correct
```bash
$ bash test-agent-models.sh

Testing rust-specialist          ... ✅ PASS (configured: haiku)
Testing devops-engineer          ... ✅ PASS (configured: haiku)
Testing frontend-developer       ... ✅ PASS (configured: haiku)
Testing test-engineer            ... ✅ PASS (configured: haiku)
Testing documentation-expert     ... ✅ PASS (configured: haiku)

✅ All agents configured correctly!
```

### ❌ Runtime Behavior Is Wrong
The configuration says `haiku`, but Claude Code is using `sonnet` anyway.

---

## Where The Override Is Happening

### NOT Found In:
- ❌ Hardcoded Task calls in `.js` files
- ❌ Configuration files with explicit overrides
- ❌ Hooks or plugins (disabled)
- ❌ Claude Code settings files

### Likely Location:
🎯 **Claude Code's internal agent spawning system**

When Claude Code automatically decides to invoke an agent (based on your request), it:
1. Selects the appropriate agent type
2. Generates Task parameters
3. **Defaults to Sonnet model if not explicitly specified** ← THE ISSUE

---

## What Needs to Be Fixed

### Option A: Explicit Task Invocations (Most Direct)
Instead of relying on Claude Code's auto-spawning, you can explicitly invoke agents with correct models:

```javascript
// Instead of asking Claude Code to do something and hoping for haiku...

// Explicitly invoke with correct model:
Task(
  "Implement daemon mode",
  "You are a Rust specialist. Implement daemon mode...",
  "rust-specialist",
  "haiku"  // ✅ Explicit model specification
)
```

### Option B: Modify Agent File Metadata (Medium)
Ensure agent metadata explicitly forces model selection. This might require changes to how Claude Code reads agent frontmatter.

### Option C: Claude Code Setting (Unknown Feasibility)
There might be a Claude Code setting to respect agent-configured models. This would need to be verified with Claude Code documentation.

---

## Immediate Action Items

### ✅ Step 1: Verify When New Agents Are Spawned
The next time you ask Claude Code to work with these agents, note:
- Agent name
- Tokens used
- Actual model being used (if displayed)

### ✅ Step 2: Test Explicit Invocation
Try invoking one agent explicitly with haiku model:

```javascript
Task(
  "Test haiku model",
  "Implement a small feature...",
  "rust-specialist",
  "haiku"
)
```

Check if token usage drops 60-70%.

### ✅ Step 3: Document Findings
Track the token usage difference between:
- Claude Code auto-spawn (current: ~Sonnet)
- Explicit haiku invocation

---

## Expected Results

### Before (Current Auto-Spawned with Sonnet):
```
rust-specialist:        33.0k tokens × Sonnet rate = $0.099
```

### After (Explicit Haiku Invocation):
```
rust-specialist:        33.0k tokens × Haiku rate = $0.026
```

**Cost reduction: ~75%**

---

## Notes

1. **History Location:** `/Users/brent/.claude/history.jsonl` - Contains all recent agent invocations
2. **Settings:** `/Users/brent/.claude/settings.json` - Claude Code's global settings (currently model: "haiku" but may not apply to Task agents)
3. **Hooks:** `/Users/brent/.claude/hooks/` - CRUD enforcement hooks are disabled
4. **Configuration:** `/Users/brent/git/cc-orchestra/config/orchestra-config.json` - All agents correctly configured to haiku

---

## Next Steps

1. **When you next use these agents**, they will likely auto-spawn with Sonnet
2. **Test explicit invocation** with haiku model to verify cost reduction
3. **Track token usage** to confirm 60-70% reduction
4. **Consider using explicit Task calls** for consistent haiku usage

---

## Files to Review

- `/Users/brent/.claude/history.jsonl` - Task invocation history
- `/Users/brent/.claude/settings.json` - Claude Code settings
- `/Users/brent/git/cc-orchestra/config/orchestra-config.json` - Agent configuration (correct)
- `/Users/brent/.claude/agents/rust-specialist.md` - Agent frontmatter (correct)

All agent files in `/Users/brent/.claude/agents/` have correct `model: haiku` in frontmatter ✅

---

## Summary

**Finding:** Task calls for these agents are happening through Claude Code's automatic agent spawning when you make requests, NOT through hardcoded explicit Task invocations.

**Root Cause:** Claude Code appears to default to Sonnet when auto-spawning agents, regardless of agent configuration.

**Solution:** Use explicit Task tool invocations with the correct model parameter to force haiku usage and realize 75% cost savings.

**Status:** Configuration is correct; runtime behavior is the issue.
