# Autopilot Mode - Quick Reference

## What is Autopilot?

Autopilot mode enables the Chief Architect to coordinate agents autonomously for extended periods (up to 4 hours) with minimal human intervention. Perfect for well-defined development tasks where you want to step away and let the orchestra work.

## Quick Start

```
/autopilot Build a REST API with JWT authentication
```

The Chief Architect will:
1. ✅ Analyze requirements and create a plan
2. ✅ Spawn agents strategically
3. ✅ Make technical decisions autonomously
4. ✅ Report progress every 2 minutes
5. ✅ Handle errors and blockers
6. ✅ Complete the goal or pause for help

## Natural Interruption

Just send any message to pause:

```
Chief Architect: [working autonomously...]

You: Wait, use PostgreSQL instead

Chief Architect: [pauses immediately, adjusts plan, asks to resume]
```

## Control Commands

- `/autopilot` - Start autonomous session
- `/autopilot-resume` - Resume after pause
- `/autopilot-stop` - Stop and show summary
- `/autopilot-status` - Check status without interrupting

## What Chief Architect Can Do Autonomously

✅ Spawn and coordinate agents
✅ Make technical decisions (libraries, patterns, approaches)
✅ Approve architecture from agents
✅ Retry failed operations
✅ Unblock agents with guidance

## What Requires Your Approval

❌ Destructive operations (delete, drop, remove)
❌ External API credentials
❌ Scope changes
❌ Uncertain decisions

## Example Session

```bash
You: /autopilot Build a FastAPI REST API with JWT auth

Chief Architect:
🤖 Autopilot Mode Activated
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Goal: Build a FastAPI REST API with JWT auth

Plan (4 phases):
1. Architecture & Design (5 min)
2. Database & Models (10 min)
3. API Implementation (15 min)
4. Testing & Security (10 min)

Estimated: ~40 minutes, 6 agents

[00:00:05] 🎭 Spawning Database Architect...
[00:02:00] 🤖 Progress: Phase 1 complete (100%)
[00:02:05] 🎭 Spawning Python Specialist...
[00:05:00] 🧠 Decision: Using FastAPI (async support)
[00:08:00] 🤖 Progress: Phase 2 at 60%

You: How's the database schema looking?

Chief Architect:
🤖 Autopilot Paused
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Here's the schema:
[shows schema...]

Ready to resume?

You: Yes

Chief Architect:
🤖 Autopilot Resumed
[00:09:15] Continuing work...
[00:15:00] 🤖 Mission Complete! 🎉
```

## Best Practices

1. **Clear Goals** - Be specific about what you want
2. **Trust the Process** - Let Chief Architect make technical calls
3. **Check Progress** - Use `/autopilot-status` for updates
4. **Interrupt Naturally** - Just send a message when needed
5. **Review Results** - Check the work after completion

## Cost Efficiency

Autopilot uses the Chief Architect (Opus) for coordination, which makes strategic decisions infrequently. Typical costs:

- **Simple feature** (30 min): ~$1.50
- **Medium project** (90 min): ~$3.00
- **Complex system** (4 hours): ~$8.00

Much cheaper than constant manual coordination!

## When to Use

**Great for:**
- ✅ Well-defined features
- ✅ Multi-hour sessions
- ✅ When you need to step away
- ✅ Complex multi-agent projects

**Not ideal for:**
- ❌ Exploratory work
- ❌ Ambiguous requirements
- ❌ Frequent human judgment needed

## Safety Features

- Max 4-hour runtime (configurable)
- Auto-pause on user messages
- State saved at every pause
- Approval gates for risky operations
- Complete decision audit trail

---

**Try it now:**
```
/autopilot Build a REST API with authentication
```
