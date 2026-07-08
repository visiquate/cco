# CCO Status

## Trigger
`/cco-status`

Check the current daemon status and system health.

## Purpose
This command provides a quick snapshot of the CCO daemon state without interrupting work:
- Daemon process status (PID, version, uptime)
- Unix socket availability (~/.cco/daemon.sock)
- Running/stopped state
- Quick health check

## Usage

```
You: /cco-status

✅ Daemon Status:
   PID: 81692
   Running: true
   Version: 2026.6.3+74816d9bc
   Started at: 2026-06-15 01:53:07.083680 UTC
   Socket: ~/.cco/daemon.sock
```

## When to Use

Use `/cco-status` when you want to:
- Verify the daemon is running
- Check the daemon version and uptime
- Confirm the Unix socket is accessible
- Perform a quick health check

## Implementation

To use this command, ask Claude to:
1. Run `cco daemon status` to get the current daemon status
2. Summarize the output showing PID, version, uptime, and socket path

## Notes

- The daemon runs in the background as a macOS launchd service
- Communication happens via Unix socket at ~/.cco/daemon.sock (owner only, 0700 permissions)
- If the daemon is not running, it will need to be started
