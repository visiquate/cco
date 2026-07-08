# CCO RTK Status (Rust Token Killer)

## Trigger
`/cco-rtk`

Check RTK (Rust Token Killer) status and cost savings from Bash command output compression.

## Purpose
This command shows:
- RTK enabled/disabled status (whether RTK binary is on PATH)
- Token savings from Bash command output compression
- How RTK integrates with CCO's PreToolUse hook
- Token reduction impact on cost reporting

## Usage

```
You: /cco-rtk

🔄 RTK Status (Rust Token Killer)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RTK Status: AVAILABLE (not installed)

Purpose:
  RTK (Rust Token Killer) compresses Bash command output to reduce LLM
  token usage. CCO automatically uses RTK via PreToolUse hook when the
  rtk binary is on PATH and rtk_enabled != false in config.

Token Savings:
  Large command outputs compressed: ~80%
  Savings per command: varies (0.5-5K tokens typical)
  Monthly savings (est): $5-15 depending on Bash usage

Integration:
  Setting: rtk_enabled in ~/.cco/config.toml (default: true)
  Activation: Automatic via PreToolUse hook if rtk binary is on PATH
  Effect: Compresses stderr/stdout before sending to Claude

How RTK Works:
  ✅ Detects large Bash outputs (>500 tokens)
  ✅ Applies compression: truncate, deduplicate, summarize
  ✅ Preserves output quality for parsing
  ✅ Reports compression stats in cco cost output

Disable RTK (if needed):

  Edit ~/.cco/config.toml:
    [hooks]
    rtk_enabled = false

  Then restart daemon:
    launchctl kickstart -kp gui/$(id -u)/com.anthropic.cco.daemon

Installation:
  RTK is not bundled with CCO. Install from:
    https://github.com/anthropics/rtk
  Then add rtk binary to PATH for automatic integration.

Performance:
  • RTK enabled:  Bash output -80%, cost savings visible in cco cost
  • RTK disabled: All Bash output sent uncompressed to Claude
```

## When to Use

Use `/cco-rtk` when you want to:
- Check if RTK is installed and enabled
- Understand the token compression savings
- Troubleshoot unexpectedly large token counts
- Enable/disable RTK for specific workflows
- Monitor output compression effectiveness

## Implementation

To use this command, ask Claude to:
1. Run `cco doctor` and filter output for RTK status
2. Show RTK installed/enabled status (is rtk on PATH?)
3. Display token savings from output compression (if available)
4. Explain how RTK integrates via PreToolUse hook
5. Provide instructions for installing, enabling/disabling RTK

## Notes

- RTK is **enabled by default** in config (but requires rtk binary on PATH to activate)
- Integrates via CCO's PreToolUse hook when available
- Automatically compresses Bash command outputs before sending to Claude
- Does not affect prompt caching (separate feature)
- Disable only if you see unexpected behavior with command output parsing
- See `cco cost` for detailed token savings metrics including RTK compression
- Install RTK separately: https://github.com/anthropics/rtk
