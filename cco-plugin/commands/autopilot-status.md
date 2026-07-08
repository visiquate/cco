# Autopilot Status

## Trigger
`/autopilot-status`

Show detailed status of the current autopilot session without interrupting it.

## Purpose
This command provides a comprehensive progress report while allowing autopilot to continue running. Unlike sending a regular message (which pauses), this shows status and lets work continue uninterrupted.

## Usage

```
You: /autopilot-status

Chief Architect:
🤖 Autopilot Status Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Goal: Build a REST API with JWT authentication
Phase: 2 of 4 - Implementation
Progress: ████████████░░░░░░░░ 65%
Duration: 00:05:30
Status: ✅ Running autonomously

Active Agents (2):
  🎭 Python Specialist
     Task: Implementing API endpoints
     Progress: 80% complete
     Status: Working (last update 15s ago)

  🎭 Test Engineer
     Task: Writing test suite
     Progress: 45% complete
     Status: Working (last update 30s ago)

Completed Phases:
  ✓ Phase 1: Architecture & Design (5 min)
     - Database schema approved
     - API architecture finalized
     - Technology stack chosen

Recent Decisions (last 5):
  • [00:04:15] Chose FastAPI over Flask (async support)
  • [00:04:30] Approved PostgreSQL schema
  • [00:05:00] Selected PyJWT for token handling
  • [00:05:15] Decided on bcrypt for passwords
  • [00:05:20] Approved Python Specialist's endpoint design

Next Milestones:
  ⏰ [~3 min] Complete API endpoints
  ⏰ [~5 min] Finish test suite
  ⏰ [~8 min] Security review
  ⏰ [~10 min] Documentation complete

Cost So Far:
  • Database Architect (Sonnet) - $0.29
  • Python Specialist (Haiku) - $0.08 (ongoing)
  • Test Engineer (Sonnet) - $0.15 (ongoing)
  • My coordination (Opus) - $0.12
  Total: $0.64

Autopilot continues working...
```

## When to Use

Use `/autopilot-status` when you want to:
- Check progress without interrupting work
- See detailed breakdown of active agents
- Review recent decisions
- Get cost information
- See estimated time to completion

## Difference from Regular Messages

| Action | Effect |
|--------|--------|
| Send any message | ⏸️ Pauses autopilot, waits for response |
| `/autopilot-status` | ✅ Shows status, continues working |

## Notes

- This is a non-blocking status check
- Autopilot continues running after showing status
- Updated information retrieved from Knowledge Manager
- Cost calculation is real-time based on token usage
