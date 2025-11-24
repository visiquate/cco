# Terminal Troubleshooting Guide

## Common Issues and Solutions

### Issue 1: Terminal Doesn't Appear

**Symptoms**:
- Blank page when navigating to web interface
- No terminal visible in browser
- "Terminal" tab missing or grayed out

**Root Cause**:
- Server not running
- WebSocket endpoint not accessible
- JavaScript error preventing terminal load
- Browser compatibility issue

**Solutions**:

1. **Verify Server is Running**
   ```bash
   # Check if server is active
   curl -I http://127.0.0.1:8080

   # Expected output:
   # HTTP/1.1 200 OK
   ```

2. **Check Server Logs**
   ```bash
   # View recent log output
   tail -50 ~/.cache/cco/logs/cco.log

   # Look for errors like:
   # ERROR: Failed to bind port
   # ERROR: WebSocket handler crashed
   ```

3. **Verify Port Availability**
   ```bash
   # Check if port 8080 in use
   lsof -i :8080

   # If port in use, restart server on different port:
   PORT=8081 cargo run --release
   ```

4. **Check Browser Console**
   - Press `F12` to open Developer Tools
   - Click "Console" tab
   - Look for JavaScript errors like:
     ```
     Uncaught TypeError: Cannot read property 'Terminal' of undefined
     WebSocket connection failed
     ```

5. **Try Different Browser**
   - Chrome, Firefox, Safari, Edge all supported
   - Ensure browser has WebSocket support
   - Try incognito/private mode to exclude extensions

6. **Check Firewall**
   ```bash
   # On macOS, allow through firewall:
   # System Preferences → Security & Privacy → Firewall

   # On Linux, open port:
   sudo ufw allow 8080
   ```

**Debug Steps**:
```bash
# Enable debug logging
export RUST_LOG=debug

# Restart server
cargo run --release

# Monitor logs
tail -f ~/.cache/cco/logs/cco.log
```

---

### Issue 2: Can't Type Commands

**Symptoms**:
- Keyboard input doesn't appear in terminal
- Typed characters not visible
- Terminal not responding to keyboard

**Root Cause**:
- Terminal not focused
- JavaScript event handling issue
- WebSocket not connected
- PTY stdin blocked

**Solutions**:

1. **Focus Terminal Window**
   - Click in terminal area to focus
   - Try clicking on the prompt

2. **Verify WebSocket Connection**
   - Open Developer Tools (F12)
   - Go to Network tab
   - Look for WebSocket connection to `/terminal`
   - Status should be "101 Switching Protocols"
   - Should show "Connected" message

3. **Try Simple Input**
   ```bash
   # If terminal is focused but frozen, try:
   # Ctrl+C to interrupt
   # Then type: echo hello
   ```

4. **Check for Hidden Errors**
   - Developer Tools → Console tab
   - Look for errors like:
     ```
     WebSocket closed abnormally
     Failed to send message
     ```

5. **Reload Terminal**
   ```bash
   # Reload page in browser
   Ctrl+R or Cmd+R
   ```

6. **Restart Session**
   - Close browser tab
   - Open new tab
   - Navigate to http://127.0.0.1:8080 again

**Debug Steps**:
```bash
# Check PTY is properly connected
sudo lsof -c cco | grep pts

# Monitor WebSocket traffic
tcpdump -i lo -A 'tcp port 8080'

# Check for PTY read/write errors
export RUST_LOG=cco::terminal=debug
```

---

### Issue 3: Commands Execute But No Output

**Symptoms**:
- Type command and press Enter
- Command appears to run but no output shown
- Terminal blank after command

**Root Cause**:
- Output reading not polling correctly
- PTY reader closed or broken
- Output buffer full or overflow
- Shell output to stderr, not stdout

**Solutions**:

1. **Check Shell is Alive**
   ```bash
   # In terminal, try pressing Ctrl+C
   # You should see "^C" appear

   # Try: echo $?
   # Should show exit code (0 for success)
   ```

2. **Force Output Display**
   ```bash
   # Try explicit command with clear output
   echo "hello world"

   # Or with newline
   printf "test\n"
   ```

3. **Check stderr**
   ```bash
   # Some commands write to stderr
   ls /nonexistent 2>&1  # Redirect stderr to stdout

   # If output appears, command uses stderr
   ```

4. **Clear Screen**
   ```bash
   Ctrl+L  # Clear terminal display
   ```

5. **Check Terminal Scrollback**
   - Output might be above visible area
   - Scroll up to check if output visible
   - Try `clear` command to reset view

6. **Test with Simpler Command**
   ```bash
   # Instead of complex pipeline:
   ls -la | grep .txt | wc -l

   # Try:
   ls
   date
   whoami
   ```

7. **Check Output Buffer**
   - Polling interval might be too long
   - Server debug logs for read/write operations

**Debug Steps**:
```bash
# Enable very verbose logging
export RUST_LOG=cco::server=trace

# Monitor I/O operations
tail -f ~/.cache/cco/logs/cco.log | grep -i "bytes"

# Check PTY directly (Unix only)
sudo cat /dev/pts/N  # where N is PTY number
```

---

### Issue 4: Connection Drops

**Symptoms**:
- "Connection closed" message appears
- Terminal suddenly goes blank
- Unexpected disconnect without action

**Root Cause**:
- Network connection lost
- Server crashed or restarted
- WebSocket keep-alive timeout
- Process killed by system

**Solutions**:

1. **Check Network Connection**
   ```bash
   # Verify network is up
   ping 127.0.0.1

   # For remote connection
   ping server-ip
   ```

2. **Check Server is Running**
   ```bash
   # Verify process
   ps aux | grep cco

   # If not running, restart:
   cargo run --release
   ```

3. **Check System Resources**
   ```bash
   # Monitor memory
   free -h

   # Monitor disk space
   df -h

   # Check for OOM killer
   dmesg | tail -20  # Look for "Killed process"
   ```

4. **Review Server Logs**
   ```bash
   # Check for crash or error
   tail -100 ~/.cache/cco/logs/cco.log

   # Look for patterns:
   # thread panicked
   # SIGTERM
   # ERROR: connection reset
   ```

5. **Reconnect**
   - Reload browser page (F5)
   - New WebSocket connection created
   - New shell session spawned

**Debug Steps**:
```bash
# Monitor server process
watch -n 1 'ps aux | grep cco'

# Monitor system resources
htop

# Monitor network connections
netstat -an | grep -E ':(8080|ESTABLISHED)'

# Enable server debug logging
export RUST_LOG=cco=debug,tokio=info
```

---

### Issue 5: Terminal Unresponsive or Hangs

**Symptoms**:
- Terminal stops responding to input
- Cursor visible but commands not executing
- Takes very long to show output

**Root Cause**:
- Shell waiting for input (reading stdin)
- Background process hung
- System resources exhausted
- Network latency very high

**Solutions**:

1. **Interrupt Current Process**
   ```bash
   Ctrl+C  # Send interrupt signal
   # You should see "^C" and get prompt back
   ```

2. **Try Simple Command**
   ```bash
   # Test if shell responsive
   echo hello

   # If works, shell is alive
   ```

3. **Send EOF**
   ```bash
   Ctrl+D  # Send EOF signal
   # May exit current program or shell
   ```

4. **Check System Resources**
   ```bash
   # High memory usage
   free -h

   # High disk usage
   df -h

   # High CPU usage
   top
   ```

5. **Kill Hung Process**
   ```bash
   # Get process list
   ps aux

   # Kill specific PID from another terminal
   kill -9 PID
   ```

6. **Restart Terminal Session**
   - Reload page in browser
   - New shell spawned
   - Previous hanging process discarded

**Debug Steps**:
```bash
# Check what shell is doing
strace -p $(pgrep bash)  # Trace system calls

# Monitor terminal size
watch 'stty -a < /dev/pts/N'

# Check for I/O blocking
lsof -p $(pgrep bash) | grep pts
```

---

### Issue 6: Garbled Output or Wrong Characters

**Symptoms**:
- Output shows gibberish or wrong characters
- ANSI color codes visible as text
- UTF-8 characters displayed incorrectly

**Root Cause**:
- Character encoding mismatch
- Terminal type not set correctly
- ANSI codes not interpreted
- Client/server encoding mismatch

**Solutions**:

1. **Set UTF-8 Encoding**
   ```bash
   # In terminal, run:
   export LANG=en_US.UTF-8
   export LC_ALL=en_US.UTF-8

   # Verify:
   echo $LANG
   ```

2. **Check Terminal Type**
   ```bash
   # View current terminal type
   echo $TERM

   # Should be: xterm-256color

   # Set if different
   export TERM=xterm-256color
   ```

3. **Test Character Display**
   ```bash
   # Test UTF-8
   echo "Hello 世界 🌍"

   # Test colors (ANSI codes)
   echo -e "\033[31mRed\033[0m"
   ```

4. **Reload Terminal**
   - Reload page in browser (F5)
   - Forces fresh connection
   - Resets terminal state

5. **Clear Terminal**
   ```bash
   reset  # Full reset
   clear  # Clear display
   ```

**Debug Steps**:
```bash
# Check locale settings
locale

# Verify UTF-8 support
file /bin/bash  # Shows encoding support

# Monitor character transmission
xxd  # Display hex dump of input
```

---

### Issue 7: Slow or High Latency Response

**Symptoms**:
- Delay between typing and output
- Sluggish terminal response
- Commands take long to show results

**Root Cause**:
- High network latency (remote server)
- Server CPU/memory exhausted
- Background tasks competing for resources
- Polling interval too long

**Solutions**:

1. **Measure Network Latency**
   ```bash
   # Check ping time
   ping -c 5 server-ip

   # Expected: < 50ms for local, < 100ms for remote
   ```

2. **Check Server Resources**
   ```bash
   # CPU usage
   top -b -n 1

   # Memory usage
   free -h

   # Disk I/O
   iostat -x 1 3
   ```

3. **Check Active Sessions**
   ```bash
   # Count WebSocket connections
   netstat -an | grep ESTABLISHED | wc -l

   # List PTY processes
   ps aux | grep pty
   ```

4. **Optimize Polling**
   - Polling interval is 10ms (reasonable)
   - Could increase to 20ms if CPU-bound
   - Could decrease to 5ms if latency-critical

5. **Test Local vs Remote**
   ```bash
   # Local test (should be < 50ms)
   time echo hello

   # If very slow, local issue
   # If acceptable, network is problem
   ```

6. **Close Other Connections**
   - Close other browser tabs
   - Close other SSH connections
   - Reduce network congestion

**Debug Steps**:
```bash
# Monitor polling rate
export RUST_LOG=cco::server=debug
tail -f logs | grep -c "bytes" | watch

# Benchmark I/O performance
time dd if=/dev/zero bs=1024 count=100000 | wc -c

# Check network statistics
ss -s
netstat -s
```

---

## General Debugging

### Enable Debug Logging

**Set Environment Variable**:
```bash
export RUST_LOG=cco::terminal=debug
# or
export RUST_LOG=cco=debug
# or
export RUST_LOG=trace
```

**Log Levels** (most to least verbose):
- `trace`: Very detailed, function entry/exit
- `debug`: Important information
- `info`: Major events only
- `warn`: Warnings only
- `error`: Errors only

**Example Output**:
```
[2025-11-15T10:30:45Z DEBUG cco::server] Received 47 bytes from client
[2025-11-15T10:30:45Z DEBUG cco::terminal] Read 256 bytes from shell
[2025-11-15T10:30:45Z DEBUG cco::server] Sending 256 bytes to client
```

### Check Browser Console

**Steps**:
1. Open Developer Tools: `F12` or `Ctrl+Shift+I`
2. Go to "Console" tab
3. Look for red error messages
4. Look for yellow warnings

**Common Errors**:
```javascript
// WebSocket connection failed
Uncaught Error: WebSocket is closed before the connection is established.

// Terminal library not loaded
Uncaught ReferenceError: Terminal is not defined

// CORS issue
Access-Control-Allow-Origin header

// Protocol error
WebSocket protocol error
```

### Monitor Network Traffic

**Using Browser DevTools**:
1. Open Developer Tools: `F12`
2. Go to "Network" tab
3. Filter by "WS" (WebSocket)
4. Look for `/terminal` connection
5. Check status (should be "101 Switching Protocols")
6. Monitor Messages tab for traffic

**Using System Tools**:
```bash
# Monitor port traffic
tcpdump -i lo -n 'tcp port 8080' -w capture.pcap

# View captured traffic
tcpdump -r capture.pcap -A

# Monitor WebSocket frames
wireshark  # GUI network analyzer
```

### Check Process Status

**List Processes**:
```bash
# Find cco process
ps aux | grep cco

# List all PTY processes
ps aux | grep pts

# Monitor process in real-time
top -p $(pgrep cco)
```

**Check File Descriptors**:
```bash
# Count open files
lsof -p $(pgrep cco) | wc -l

# List all open file descriptors
lsof -p $(pgrep cco)

# Check PTY files
lsof -p $(pgrep cco) | grep pts
```

### Review Server Logs

**Log Locations**:
- `~/.cache/cco/logs/cco.log` (default)
- Set `RUST_LOG` environment variable to control verbosity

**View Logs**:
```bash
# View last 50 lines
tail -50 ~/.cache/cco/logs/cco.log

# Follow logs in real-time
tail -f ~/.cache/cco/logs/cco.log

# Search for errors
grep ERROR ~/.cache/cco/logs/cco.log

# Filter by session
grep "session_id" ~/.cache/cco/logs/cco.log
```

### Create Debug Report

When reporting issues, collect:

```bash
# 1. System information
uname -a
free -h
df -h

# 2. Server version
curl http://127.0.0.1:8080/version

# 3. Recent logs
tail -100 ~/.cache/cco/logs/cco.log > debug_logs.txt

# 4. Process information
ps aux | grep cco > process_info.txt

# 5. Network status
netstat -an | grep 8080 > network_info.txt

# 6. Browser console errors
# Manually copy from Developer Tools Console

# Bundle for reporting
tar czf debug_report.tar.gz debug_logs.txt process_info.txt network_info.txt
```

---

## Reporting Issues

**When reporting issues, include**:
1. **Symptom description**: What you observed
2. **Steps to reproduce**: Exact commands/actions
3. **Expected behavior**: What should happen
4. **Actual behavior**: What happened instead
5. **Environment**: OS, browser, server version
6. **Logs**: Debug output from terminal
7. **Errors**: Console errors, crash dumps

**Example Issue Report**:
```
Title: Terminal hangs when running long command

Symptom: Terminal becomes unresponsive when running `find / -type f`

Steps to reproduce:
1. Open terminal
2. Run: find / -type f
3. Wait 5 seconds
4. Try typing Ctrl+C
5. No response

Expected: Command interrupted, prompt returns
Actual: Terminal frozen, no response to input

Environment:
- macOS 12.6.1
- Chrome 108
- CCO version 2025.11.2

Logs attached: debug_logs.txt
Console errors: None
```

---

**Last Updated**: November 2025
**Version**: 1.0
**Status**: Comprehensive Coverage
