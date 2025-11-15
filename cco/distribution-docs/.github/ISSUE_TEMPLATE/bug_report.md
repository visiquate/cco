---
name: Bug Report
about: Report a bug or unexpected behavior
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of the bug.

## Steps to Reproduce

1. Start CCO with `cco proxy ...`
2. Execute command `...`
3. Observe error `...`

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Environment

**CCO Version:**
```
cco --version
```

**Operating System:**
- [ ] macOS (Intel)
- [ ] macOS (Apple Silicon)
- [ ] Linux (x86_64)
- [ ] Linux (ARM64)
- [ ] Windows

**OS Version:**
```
# macOS/Linux
uname -a

# Windows
systeminfo | findstr /B /C:"OS Name" /C:"OS Version"
```

**Configuration:**
```bash
# Paste output of (with sensitive data redacted):
cco config show --effective
```

## Logs

```
# Paste relevant logs (last 50 lines):
tail -50 ~/.config/cco/cco.log
```

## Additional Context

Any other information that might help diagnose the issue:
- Recent changes to configuration
- Network environment (proxy, firewall, etc.)
- Related error messages
- Screenshots if applicable

## Checklist

- [ ] I have searched existing issues
- [ ] I have read the troubleshooting guide
- [ ] I am using the latest version
- [ ] I have included all required information above
