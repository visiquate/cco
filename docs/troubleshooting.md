# CCO Troubleshooting Guide

## Common Issues

### Installation Issues

#### Command Not Found After Installation

**Symptom:** `bash: cco: command not found`

**Solution:**

1. Check if binary is in PATH:
   ```bash
   echo $PATH | grep -o "/usr/local/bin"
   ```

2. If missing, add to PATH (add to `~/.bashrc` or `~/.zshrc`):
   ```bash
   export PATH="/usr/local/bin:$PATH"
   source ~/.bashrc  # or ~/.zshrc
   ```

3. Verify binary exists:
   ```bash
   ls -lh /usr/local/bin/cco
   ```

#### Permission Denied

**Symptom:** `Permission denied` when running `cco`

**Solution:**

```bash
# Make binary executable
chmod +x /usr/local/bin/cco

# If it's in a user directory
chmod +x ~/.local/bin/cco
```

#### macOS Gatekeeper Warning

**Symptom:** "cco cannot be opened because the developer cannot be verified"

**Solution:**

Option 1 (Recommended):
```bash
# Remove quarantine attribute
sudo xattr -rd com.apple.quarantine /usr/local/bin/cco
```

Option 2:
1. Go to System Preferences > Security & Privacy
2. Click "Allow" next to the CCO warning
3. Run `cco` again

### Configuration Issues

#### API Key Not Working

**Symptom:** `401 Unauthorized` or `Invalid API key`

**Solution:**

1. Verify API key format:
   ```bash
   # Should start with sk-ant-
   cco config get api.key
   ```

2. Set API key correctly:
   ```bash
   cco config set api.key "sk-ant-YOUR-KEY-HERE"
   ```

3. Test API connection:
   ```bash
   cco debug info
   ```

4. Check environment variable override:
   ```bash
   echo $CCO_API_KEY
   # Unset if not needed
   unset CCO_API_KEY
   ```

#### Configuration File Not Found

**Symptom:** `Configuration file not found`

**Solution:**

1. Initialize configuration:
   ```bash
   cco init --global
   ```

2. Check configuration locations:
   ```bash
   cco debug info | grep -A5 "Config"
   ```

3. Create manual configuration:
   ```bash
   mkdir -p ~/.config/cco
   cat > ~/.config/cco/config.toml <<EOF
   [api]
   key = "sk-ant-YOUR-KEY"

   [models]
   default = "sonnet"
   EOF
   ```

#### Invalid Configuration

**Symptom:** `Invalid configuration` or `Parse error`

**Solution:**

1. Validate TOML syntax:
   ```bash
   # Use an online TOML validator or
   cco config validate
   ```

2. Check for common errors:
   - Missing quotes around strings
   - Incorrect section headers
   - Invalid option names

3. Show current config with errors:
   ```bash
   cco config show --verbose
   ```

### Daemon Issues

#### Daemon Won't Start

**Symptom:** `Failed to start daemon` or daemon immediately exits

**Solution:**

1. Check if already running:
   ```bash
   cco daemon status
   ```

2. Check for port conflicts:
   ```bash
   lsof -i :9736  # Default port
   # Kill conflicting process if needed
   ```

3. Check permissions:
   ```bash
   mkdir -p ~/.local/run
   chmod 755 ~/.local/run
   ```

4. View daemon logs:
   ```bash
   cco daemon logs --tail 50
   ```

5. Try different port:
   ```bash
   cco config set daemon.port 9999
   cco daemon start
   ```

#### Daemon Crashes

**Symptom:** Daemon starts but crashes after a short time

**Solution:**

1. Check logs for errors:
   ```bash
   cco daemon logs --level error
   ```

2. Run in foreground mode:
   ```bash
   CCO_LOG_LEVEL=debug cco daemon start --foreground
   ```

3. Check system resources:
   ```bash
   # Ensure sufficient memory and disk space
   free -h
   df -h
   ```

4. Clean up stale PID file:
   ```bash
   rm ~/.local/run/cco.pid
   cco daemon start
   ```

### Agent Issues

#### Agent Spawn Failures

**Symptom:** `Failed to spawn agent` or agents timeout

**Solution:**

1. Check agent system status:
   ```bash
   cco agent status
   ```

2. Verify API connectivity:
   ```bash
   cco debug validate
   ```

3. Reduce concurrent agents:
   ```bash
   cco config set agents.max_concurrent 5
   ```

4. Increase timeout:
   ```bash
   cco config set agents.default_timeout 900
   ```

5. Check available agents:
   ```bash
   cco agent list
   ```

#### Agent Communication Errors

**Symptom:** Agents can't communicate or coordinate

**Solution:**

1. Enable knowledge manager:
   ```bash
   cco config set agents.enable_memory true
   ```

2. Clear knowledge database:
   ```bash
   cco knowledge prune
   ```

3. Restart daemon:
   ```bash
   cco daemon restart
   ```

### TUI Issues

#### TUI Won't Launch

**Symptom:** `Failed to initialize TUI` or crashes on launch

**Solution:**

1. Check terminal compatibility:
   ```bash
   echo $TERM
   # Should be xterm-256color or similar
   ```

2. Set TERM variable:
   ```bash
   export TERM=xterm-256color
   cco tui
   ```

3. Disable colors if needed:
   ```bash
   CCO_NO_COLOR=1 cco tui
   ```

4. Use alternative theme:
   ```bash
   cco tui --theme light
   ```

#### TUI Display Issues

**Symptom:** Garbled output, rendering problems

**Solution:**

1. Clear screen:
   ```bash
   clear
   cco tui
   ```

2. Resize terminal:
   - Ensure terminal is at least 80x24
   - Try fullscreen mode

3. Update terminal emulator:
   - Some older terminals have rendering bugs
   - Try iTerm2 (macOS) or modern terminal

4. Adjust refresh rate:
   ```bash
   cco tui --refresh 2000  # Slower refresh
   ```

### Performance Issues

#### Slow Response Times

**Symptom:** Commands take a long time to execute

**Solution:**

1. Check API status:
   ```bash
   cco debug info
   ```

2. Monitor network:
   ```bash
   ping api.anthropic.com
   ```

3. Reduce concurrent agents:
   ```bash
   cco config set agents.max_concurrent 5
   ```

4. Use faster model:
   ```bash
   cco config set models.default haiku
   ```

5. Clear caches:
   ```bash
   rm -rf ~/.cache/cco
   ```

#### High Memory Usage

**Symptom:** CCO consuming too much memory

**Solution:**

1. Check knowledge database size:
   ```bash
   cco knowledge stats
   ```

2. Prune old knowledge:
   ```bash
   cco knowledge prune --older-than 30d
   ```

3. Reduce agent count:
   ```bash
   cco config set agents.max_concurrent 3
   ```

4. Restart daemon:
   ```bash
   cco daemon restart
   ```

### Knowledge Manager Issues

#### Knowledge Search Not Working

**Symptom:** No results from knowledge search

**Solution:**

1. Check database status:
   ```bash
   cco knowledge stats
   ```

2. Rebuild database:
   ```bash
   cco knowledge rebuild
   ```

3. Verify storage location:
   ```bash
   ls -lh ~/.local/share/cco/knowledge/
   ```

4. Clear and reinitialize:
   ```bash
   rm -rf ~/.local/share/cco/knowledge/
   cco knowledge init
   ```

#### Knowledge Not Persisting

**Symptom:** Knowledge lost between sessions

**Solution:**

1. Verify memory is enabled:
   ```bash
   cco config get agents.enable_memory
   ```

2. Check storage permissions:
   ```bash
   ls -ld ~/.local/share/cco/
   chmod 755 ~/.local/share/cco/
   ```

3. Test knowledge storage:
   ```bash
   cco knowledge store "test entry"
   cco knowledge search "test"
   ```

### Update Issues

#### Update Check Fails

**Symptom:** `Failed to check for updates`

**Solution:**

1. Check network connectivity:
   ```bash
   curl -I https://github.com/brentley/cco/releases/latest
   ```

2. Disable proxy if set:
   ```bash
   unset HTTP_PROXY
   unset HTTPS_PROXY
   ```

3. Manual update check:
   ```bash
   cco update --check --verbose
   ```

4. Disable auto-update:
   ```bash
   cco config set updates.auto_check false
   ```

#### Update Installation Fails

**Symptom:** Update downloads but won't install

**Solution:**

1. Check permissions:
   ```bash
   ls -l /usr/local/bin/cco
   sudo chown $USER /usr/local/bin/cco
   ```

2. Manual update:
   ```bash
   curl -LO https://github.com/brentley/cco/releases/latest/download/cco-$(uname -s)-$(uname -m)
   chmod +x cco-*
   sudo mv cco-* /usr/local/bin/cco
   ```

3. Verify new version:
   ```bash
   cco --version
   ```

## Error Messages

### API Errors

| Error | Meaning | Solution |
|-------|---------|----------|
| `401 Unauthorized` | Invalid API key | Check and reset API key |
| `429 Rate Limited` | Too many requests | Wait and reduce concurrent agents |
| `500 Server Error` | API service issue | Wait and retry, check status page |
| `Timeout` | Request took too long | Increase timeout setting |

### Configuration Errors

| Error | Meaning | Solution |
|-------|---------|----------|
| `Parse error` | Invalid TOML syntax | Check configuration syntax |
| `Missing required field` | Required option missing | Add required configuration |
| `Invalid value` | Wrong value type | Check value format |
| `File not found` | Config file missing | Run `cco init` |

### Agent Errors

| Error | Meaning | Solution |
|-------|---------|----------|
| `Agent not found` | Invalid agent name | Check `cco agent list` |
| `Spawn failed` | Couldn't create agent | Check logs, reduce concurrent agents |
| `Timeout` | Agent didn't respond | Increase timeout |
| `Communication error` | Agent coordination failed | Restart daemon |

## Diagnostic Commands

### System Information

```bash
# Complete system diagnostics
cco debug info

# Validate installation
cco debug validate

# Check API connectivity
cco config test api

# Agent system health
cco agent status
```

### Logging

```bash
# View recent logs
cco daemon logs --tail 100

# Follow logs in real-time
cco daemon logs --follow

# Filter by level
cco daemon logs --level error

# Export logs
cco daemon logs > cco-debug.log
```

### Metrics

```bash
# Current metrics
cco metrics show

# Export metrics
cco metrics export metrics.json

# Generate report
cco metrics report > metrics-report.txt
```

## Getting Help

If you're still experiencing issues:

1. **Check Documentation**
   - [Installation Guide](./installation.md)
   - [Configuration Guide](./configuration.md)
   - [Commands Reference](./commands.md)

2. **Collect Diagnostic Information**
   ```bash
   cco debug info > debug-info.txt
   cco daemon logs --tail 200 > logs.txt
   cco config show > config.txt
   ```

3. **Contact Support**
   - Include diagnostic files
   - Describe the issue clearly
   - Provide steps to reproduce
   - Include CCO version: `cco --version`

## Known Issues

### macOS

- **Gatekeeper warnings**: Expected for unsigned binaries
- **System tray**: May not show icon on some macOS versions
- **Keychain access**: First run may prompt for permission

### Linux

- **systemd integration**: Manual setup required
- **X11 vs Wayland**: TUI rendering differences
- **Terminal emulators**: Some have better support than others

### General

- **Large repositories**: Knowledge manager may be slow on very large codebases
- **Network latency**: Affects agent coordination in high-latency environments
- **Token limits**: Long-running operations may hit rate limits

## Prevention

### Best Practices

1. **Regular Updates**
   ```bash
   cco update --check
   ```

2. **Configuration Backups**
   ```bash
   cp ~/.config/cco/config.toml ~/.config/cco/config.toml.backup
   ```

3. **Knowledge Maintenance**
   ```bash
   cco knowledge prune --older-than 60d
   ```

4. **Log Rotation**
   ```bash
   # Automatic with logrotate or manual
   > ~/.local/log/cco.log
   ```

5. **Metric Monitoring**
   ```bash
   cco metrics show
   ```
