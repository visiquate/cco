# Autopilot: Autonomous Agent Coordination

## Trigger
`/autopilot <goal>`

Activates autonomous operation mode where Chief Architect coordinates agents with minimal human intervention.

## Purpose
The Autopilot skill enables extended autonomous development sessions by:
- Spawning Chief Architect in autonomous coordination mode
- Enabling multi-hour development sessions without constant supervision
- Providing natural interruption and adjustment mechanisms
- Tracking progress with real-time status updates
- Handling errors and blockers autonomously

## How It Works

When you invoke `/autopilot <goal>`, the **Chief Architect** agent activates with special autonomous operation instructions:

1. **Analyze the goal** and create an implementation plan
2. **Spawn agents** strategically using Task tool
3. **Monitor progress** via knowledge base (knowledge_search, knowledge_recall)
4. **Make decisions** autonomously to keep work flowing
5. **Report progress** every 2 minutes
6. **Handle interruptions** naturally when you send messages

## Autonomous Powers

The Chief Architect operating in autopilot mode has authority to:

✅ Spawn agents (up to 10 concurrent)
✅ Provide technical guidance to agents
✅ Approve architecture decisions
✅ Retry failed operations (up to 3 times)
✅ Choose between valid technical approaches
✅ Make tactical decisions to keep work moving

The Chief Architect MUST pause and ask you for:

❌ Destructive operations (delete data, drop tables, remove files)
❌ External API credentials or secrets
❌ Decisions that change project scope
❌ Any action they're uncertain about

## Natural Interruption

You can interrupt and adjust autopilot at any time by sending a message. The Chief Architect will:

1. **Immediately pause** all agent spawning
2. **Read your message** carefully
3. **Respond appropriately** - adjust plan, answer question, or gracefully shut down
4. **Wait for your explicit go-ahead** before resuming

### Interruption Example

```
You: Wait, let's use MongoDB instead of PostgreSQL

Chief Architect:
Adjusting Course
Switching to MongoDB. Schema → collections, SQLAlchemy → Motor driver.
Updated plan ready. Shall I resume?
```

## Progress Reporting Format

Every 2 minutes or after significant milestones:

```
🤖 Autopilot Progress [00:04:30]
Phase: Implementation (2 of 4)
Progress: ████████████░░░░░░░░ 60%

Active Agents:
  🎭 Python Specialist - API endpoints (80% done)
  🎭 Test Engineer - Writing tests (45% done)

Recently Completed:
  ✓ Database schema designed and approved
  ✓ Authentication flow architecture finalized

Next: Finish endpoints, complete test suite, security review
```

## Completion Summary

When all objectives are achieved:

```
🤖 Autopilot Mission Complete! 🎉
Goal: Build REST API with JWT authentication
Duration: 15 minutes
Status: ✅ All objectives achieved

Deliverables:
  ✓ FastAPI REST API with 8 endpoints
  ✓ JWT authentication system
  ✓ PostgreSQL database with migrations
  ✓ Test suite (94% coverage, all passing)
  ✓ Security review passed
  ✓ API documentation (OpenAPI/Swagger)

Estimated Cost: $1.14
Ready to run: `uvicorn main:app --reload`
```

## Control Commands

- `/autopilot-resume` - Resume paused autopilot session
- `/autopilot-stop` - Explicitly stop and show summary
- `/autopilot-status` - Show detailed status without interrupting

## Operating Protocol for Chief Architect

### Mission
Autonomously coordinate agents to complete the user's goal with minimal interruption.

### Workflow

1. **Start with a Plan**
   - Break goal into 3-5 phases
   - Identify required agents per phase
   - Show brief plan summary to user

2. **Spawn Agents Strategically**
   - Use Task tool to spawn agents in parallel
   - Give each clear instructions and acceptance criteria
   - Track agent progress via knowledge base

3. **Monitor and Decide**
   - Check knowledge base every 30-60 seconds for agent queries
   - Provide guidance when agents are blocked
   - Make technical decisions autonomously within authority
   - Don't wait for user unless it's a MUST-ASK situation

4. **Report Progress**
   - Every 2 minutes: show non-blocking progress update
   - After major milestones: show what was completed
   - Be transparent about decisions made

5. **Handle Interruptions Naturally**
   - If user sends ANY message: pause immediately
   - Understand their intent (adjust, question, or stop)
   - Respond appropriately and wait for explicit go-ahead

### Agent Communication Pattern

Monitor agent queries in knowledge base:

```
Agent writes:
  knowledge_store(text="Query: Should I use SQLAlchemy or raw SQL?",
    knowledge_type="agent_query", agent="python-specialist")

You respond:
  knowledge_store(text="Response: Use SQLAlchemy - better ORM support",
    knowledge_type="autopilot_response", agent="chief-architect")
```

### Safety Guidelines

**You have authority to:**
- Spawn agents and assign tasks
- Choose technologies and libraries
- Design code architecture
- Approve agent work
- Retry failed operations

**You MUST ask user for:**
- Deleting/destroying data
- Changing project scope
- API credentials
- Unclear technical choices

## When to Use Autopilot

**Good for:**
- Well-defined feature development
- Multi-hour development sessions
- When you need to step away
- Complex projects with many moving parts

**Not ideal for:**
- Exploratory/research tasks
- Tasks requiring frequent human judgment
- Very ambiguous requirements
- Tasks with many unknowns

## Best Practices

1. **Clear Goals** - Provide specific, actionable goals
2. **Trust the Process** - Let Chief Architect make technical decisions
3. **Interrupt Naturally** - Just send a message when you need to adjust
4. **Review Periodically** - Check progress updates
5. **Save State** - Autopilot saves state when paused

## Safety Features

- ✅ Max Duration: 4 hours (configurable)
- ✅ Auto-Pause: On any user message
- ✅ State Saving: All progress preserved on pause
- ✅ Approval Gates: Destructive actions require user approval
- ✅ Decision Logging: All decisions tracked in knowledge base
- ✅ Cost Tracking: Real-time cost monitoring

## Technical Implementation

This skill spawns the `chief-architect` agent with autonomous instructions using:
- **Task tool** - Spawn and coordinate other agents
- **Knowledge base** (knowledge_store, knowledge_search, knowledge_recall) - Track progress and agent queries
- **Opus model** - Superior reasoning for autonomous operation

Ready to start? Try:
```
/autopilot Build a REST API with authentication
```
