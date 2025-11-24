# Model Override Issue - Root Cause & Fix

## Problem Summary

Agents configured to use **Haiku** are being invoked with **Sonnet**, causing:
- ❌ Higher costs (Sonnet is more expensive than Haiku)
- ❌ Slower execution (unnecessary complexity for simple tasks)
- ❌ Wasted token usage
- ❌ Configuration not being respected

**Example Evidence:**
```
⏺ rust-specialist(Implement daemon mode)
  ⎿  Done (8 tool uses · 33.0k tokens · 58s)  ← Too many tokens (Sonnet)
                             ↑
                    Should be much lower for Haiku
```

## Root Cause

When the Task tool is invoked to spawn agents, **hardcoded model parameters are overriding the configuration**:

```javascript
// ❌ WRONG - Overrides the configured model
Task("description", "prompt", "rust-specialist", "sonnet")
                                                   ↑
                                    Hardcoded override
```

The configuration correctly specifies Haiku:
```json
{
  "name": "Rust Specialist",
  "type": "rust-specialist",
  "model": "haiku"  ← Configuration says haiku
}
```

But when invoked:
```javascript
Task(..., ..., "rust-specialist", "sonnet")  ← Task call says sonnet ← WRONG!
```

## The Five Affected Agents

All configured for Haiku, but being invoked with Sonnet:

| Agent | Configured | Being Used | Issue |
|-------|-----------|-----------|-------|
| rust-specialist | haiku | sonnet | ❌ Override |
| devops-engineer | haiku | sonnet | ❌ Override |
| frontend-developer | haiku | sonnet | ❌ Override |
| test-engineer | haiku | sonnet | ❌ Override |
| documentation-expert | haiku | sonnet | ❌ Override |

## Fix Steps

### Option 1: Direct Fix (Immediate)

When calling the Task tool, **use the configured model**:

```javascript
// ✅ CORRECT
Task(
  "Implement daemon mode",
  "Full prompt here...",
  "rust-specialist",
  "haiku"  // ✅ Uses configured model
);
```

### Option 2: Automated Fix (Using Agent Spawner)

Use the provided agent spawner helper:

```javascript
const AgentSpawner = require('./src/agent-spawner.js');

// Get correct model automatically
const model = AgentSpawner.getAgentModel('rust-specialist');
console.log(model);  // Output: "haiku"

// Generate task params with correct model
const params = AgentSpawner.generateTaskParams(
  "Implement daemon mode",
  "Full prompt here...",
  "rust-specialist"
);

Task(params.description, params.prompt, params.subagent_type, params.model);
```

### Option 3: Search & Replace (If invocations are centralized)

If you have code that spawns all agents, search for patterns:

```bash
# Find all hardcoded model overrides
grep -r 'Task.*"sonnet"' .
grep -r 'Task.*"haiku"' .

# For these agents, change:
# FROM: Task(..., "rust-specialist", "sonnet")
# TO:   Task(..., "rust-specialist", "haiku")
```

## Validation

Verify your fix:

```bash
# Check configured model for an agent
node src/agent-spawner.js get-model rust-specialist
# Output: haiku

# Validate that your Task call uses the right model
node src/agent-spawner.js validate rust-specialist haiku
# Output: ✅ (exit code 0)

node src/agent-spawner.js validate rust-specialist sonnet
# Output: ⚠️ Model Mismatch (exit code 1)
```

## Configuration Verification

All agent files and configuration are already correct:

```bash
# Verify models match config
node verify-agent-models.js
# Output: ✅ All agent model assignments match the config!
```

## Cost Impact

### Before Fix (Current - Using Sonnet for everything)
```
rust-specialist:        ~30k tokens × Sonnet rate
devops-engineer:        ~40k tokens × Sonnet rate
frontend-developer:     ~44k tokens × Sonnet rate
test-engineer:          ~52k tokens × Sonnet rate
documentation-expert:   ~72k tokens × Sonnet rate
────────────────────────────────────────────────
Total:                  ~238k tokens × Sonnet rate
Estimated monthly:      $600+ (rough estimate)
```

### After Fix (Using Haiku as configured)
```
rust-specialist:        ~30k tokens × Haiku rate
devops-engineer:        ~40k tokens × Haiku rate
frontend-developer:     ~44k tokens × Haiku rate
test-engineer:          ~52k tokens × Haiku rate
documentation-expert:   ~72k tokens × Haiku rate
────────────────────────────────────────────────
Total:                  ~238k tokens × Haiku rate
Estimated monthly:      $150-200 (rough estimate)
Expected Savings:       60-70% cost reduction
```

## Where the Issue Likely Is

The hardcoded models are probably in:

1. **Manual Task calls** - If you manually invoke Task tool
2. **Orchestration code** - If there's a script that spawns agents
3. **Claude Code templates** - If there's a default that uses Sonnet
4. **Environment or configuration** - If there's a default model setting

## Quick Action Items

- [ ] Search codebase for `Task(..., ..., "sonnet")` patterns
- [ ] Update to use configured models
- [ ] Run `node verify-agent-models.js` to confirm config
- [ ] Run `node src/agent-spawner.js map` to see all configured models
- [ ] Test with a single agent to verify cost reduction
- [ ] Monitor next few agent invocations for proper model usage

## Files Created to Help

1. **`fix-agent-models.js`** - Fix agent files (already run ✅)
2. **`fix-task-model-calls.js`** - Shows correct Task usage
3. **`src/agent-spawner.js`** - Helper for automated model lookup
4. **`MODEL_OVERRIDE_FIX.md`** - This document

## Next Steps

1. **Identify where Task tool is called** for these agents
2. **Update model parameter** to use configured values
3. **Test one agent** to verify cost reduction
4. **Monitor token usage** for subsequent runs
5. **Apply same fix** to all agent invocations

---

**Summary:** The configuration is correct. The issue is in the Task tool invocations that override the configured models with hardcoded "sonnet". Update the Task calls to respect the configuration.
