# CCO Doctor

## Trigger
`/cco-doctor`

Run a comprehensive health check and diagnostics for the CCO installation.

## Purpose
This command performs system diagnostics to identify and report on:
- Installation completeness
- Configuration validity
- Daemon availability
- Credentials setup
- Plugin directory structure
- API connectivity

## Usage

```
You: /cco-doctor

🏥 CCO Health Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
System Diagnostics:

✅ Installation
   Binary found at: ~/.local/bin/cco
   Version: 2026.6.3+74816d9bc
   Permissions: Executable

✅ Configuration
   Config file: ~/.cco/config.toml
   Valid TOML syntax
   Model settings: Valid
   Budget thresholds: $100/week, $20/day

✅ Daemon
   Running: Yes (PID 81692)
   Socket: ~/.cco/daemon.sock (accessible)
   Version match: Yes

⚠️ Credentials
   API key: FOUND (valid)
   Credentials method: OS Keyring
   
✅ Plugin
   Plugin dir: ~/.claude/plugins/cco/
   Commands: 43 registered
   Skills: 18 registered

✅ Connectivity
   API endpoint: Reachable
   Daemon socket: Responding
   Plugin registration: OK

Summary: All systems operational ✅
```

## When to Use

Use `/cco-doctor` when you want to:
- Troubleshoot installation issues
- Verify all components are working
- Check configuration validity
- Diagnose connectivity problems
- Get a system health report

## Auto-fix

Use `/cco-doctor --fix` to automatically remediate common issues:
- Create missing directories
- Validate and repair config files
- Restart the daemon
- Re-register plugins

## Implementation

To use this command, ask Claude to:
1. Run `cco doctor` to get the health report (human-readable format)
2. Summarize findings: installation, config, daemon, credentials, plugin status
3. Note any warnings or issues
4. Optionally mention `cco doctor --fix` can auto-remediate if issues are found

## Notes

- `cco doctor --format json` outputs machine-readable JSON for scripting
- The `--fix` flag auto-repairs most common configuration issues
- Run this after installation or when troubleshooting
- Daemon restarts are safe and preserve running sessions
