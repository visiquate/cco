# Autopilot Resume

## Trigger
`/autopilot-resume`

Resume a paused autopilot session from where it left off.

## Purpose
This command restarts the Chief Architect's autonomous coordination after an interruption. The Chief Architect will:
- Resume from the exact point where it paused
- Restore all agent states from Knowledge Manager
- Continue working toward the original goal
- Pick up any adjustments you made during the pause

## Usage

```
You: /autopilot-resume

Chief Architect:
🤖 Autopilot Resumed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Continuing from where we left off...

Restoring state:
  ✓ Phase 2 of 4: Implementation (60% complete)
  ✓ 2 agents active, ready to resume
  ✓ Goal unchanged: Build REST API with JWT

[00:08:15] 🎭 Python Specialist resuming...
[00:08:20] 🎭 Test Engineer resuming...
```

## When to Use

Use `/autopilot-resume` after:
- Answering the Chief Architect's questions
- Making adjustments to the plan or approach
- Reviewing progress and deciding to continue
- Taking a break and wanting to restart work

## Notes

- All progress is automatically saved when autopilot pauses
- State restoration happens via Knowledge Manager
- Agents continue from their last checkpoint
- No work is lost during interruptions
