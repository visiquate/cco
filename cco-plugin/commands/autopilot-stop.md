# Autopilot Stop

## Trigger
`/autopilot-stop`

Explicitly stop the current autopilot session and show a comprehensive summary.

## Purpose
This command gracefully terminates autonomous operation and provides:
- Session summary with completion status
- List of completed and in-progress tasks
- Cost breakdown by agent
- Saved state information for potential resume

## Usage

```
You: /autopilot-stop

Chief Architect:
🤖 Autopilot Stopped
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Session Summary:
  Goal: Build a REST API with JWT authentication
  Duration: 8 minutes
  Status: Paused at Implementation phase (60% complete)

Completed:
  ✓ Architecture design (5 min)
  ✓ Database schema (3 min)
  ✓ 60% of API endpoints

In Progress (saved state):
  ⚙️ Python Specialist - finishing endpoints
  ⚙️ Test Engineer - 45% through test suite

Cost Summary:
  • Database Architect (Sonnet) - $0.29
  • Python Specialist (Haiku) - $0.12
  • My coordination (Opus) - $0.18
  Total: $0.59

All work is saved in Knowledge Manager.
Resume anytime with: /autopilot-resume
```

## When to Use

Use `/autopilot-stop` when you want to:
- End the session and review what's been built
- Take a longer break before continuing
- Explicitly terminate rather than just sending a message
- Get a comprehensive summary of progress and costs

## Alternative: Natural Stopping

Note: You don't need this command to stop autopilot. Simply sending any message will pause it naturally. Use `/autopilot-stop` only when you want the explicit termination message.

## Notes

- All progress is saved automatically
- Agent states are preserved in Knowledge Manager
- You can resume later with `/autopilot-resume`
- Stopping does not lose any work
