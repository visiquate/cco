# Terminal User Guide

## Getting Started

### Accessing the Terminal

1. **Start the CCO Server**
   ```bash
   cargo run --release
   ```
   Output will show:
   ```
   → WebSocket Terminal: ws://127.0.0.1:8080/terminal
   ```

2. **Open in Browser**
   - Web Interface: `http://127.0.0.1:8080`
   - Click "Terminal" tab or navigate to terminal section
   - Terminal emulator loads with xterm.js

3. **First Connection**
   - Browser prompts to upgrade to WebSocket
   - Server spawns shell session
   - Terminal ready for input

### Terminal Appearance

The terminal displays:
- **Command Prompt**: Shows shell prompt (e.g., `bash-5.1$`)
- **Black Background**: Standard terminal color scheme
- **White Text**: Default foreground color
- **Cursor**: Blinking block indicating input position
- **Scrollback**: Previous commands and output visible above

## Basic Usage

### Typing Commands

Type commands naturally:
```bash
$ ls -la
```

Commands execute immediately after pressing Enter.

### Common Commands

**File Listing**
```bash
ls           # List current directory
ls -la       # List with details
cd /path     # Change directory
pwd          # Print working directory
```

**File Operations**
```bash
cp file.txt file-copy.txt    # Copy file
mv file.txt newname.txt      # Rename/move
rm file.txt                  # Delete file
cat file.txt                 # Display file contents
```

**Text Editing**
```bash
nano file.txt    # Simple text editor
vim file.txt     # Advanced text editor
```

**System Information**
```bash
whoami           # Current user
date             # Current date/time
echo $PATH       # Environment variables
uname -a         # System information
```

**Process Management**
```bash
ps aux           # List processes
kill PID         # Terminate process by ID
fg               # Bring background job to foreground
```

## Keyboard Shortcuts

### Navigation & Editing

| Shortcut | Action |
|----------|--------|
| `Up Arrow` | Previous command in history |
| `Down Arrow` | Next command in history |
| `Left Arrow` | Move cursor left |
| `Right Arrow` | Move cursor right |
| `Ctrl+A` | Jump to line start |
| `Ctrl+E` | Jump to line end |
| `Ctrl+Left` | Jump word backward |
| `Ctrl+Right` | Jump word forward |

### Deletion & Correction

| Shortcut | Action |
|----------|--------|
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `Ctrl+W` | Delete word before cursor |
| `Ctrl+U` | Delete entire line |
| `Ctrl+K` | Delete from cursor to end |

### Shell Control

| Shortcut | Action |
|----------|--------|
| `Ctrl+C` | Interrupt/stop running program |
| `Ctrl+D` | Send EOF (end of input) / exit shell |
| `Ctrl+Z` | Suspend running job (background) |
| `Ctrl+L` | Clear screen |
| `Ctrl+R` | Reverse history search |

### Terminal Control

| Shortcut | Action |
|----------|--------|
| `Ctrl+S` | Pause output (Ctrl+Q to resume) |
| `Tab` | Autocomplete command/filename |

## Copy & Paste Functionality

### Copying from Terminal

**Using Mouse**:
1. Click and drag to select text
2. Selected text highlights
3. Press `Ctrl+C` to copy
4. Paste elsewhere with `Ctrl+V`

**Using Keyboard**:
1. Position cursor at text start
2. Hold `Shift` and use arrow keys to select
3. Press `Ctrl+C` to copy

### Pasting into Terminal

**From Clipboard**:
1. Copy text from another application
2. Click in terminal to focus
3. Press `Ctrl+V` to paste
4. Text appears at cursor position
5. Press Enter to execute

**Example**:
```bash
# Copy this from browser: echo "Hello, Terminal!"
# Paste in terminal with Ctrl+V
Hello, Terminal!
```

### Multi-line Pasting

Paste multiple lines at once:
```bash
# Paste these commands:
mkdir test
cd test
touch file.txt
ls -la
```

Each command executes in sequence.

## Supported Shell Commands

### Shell Builtins

Commands built into bash/sh:

```bash
cd           # Change directory
echo         # Print text
export       # Set environment variable
alias        # Create command alias
source       # Execute shell script
jobs         # List background jobs
fg/bg        # Foreground/background control
```

### External Programs

Any program installed on system:

```bash
ls, grep, sed, awk       # Unix utilities
python, node, ruby       # Programming languages
git, npm, cargo          # Development tools
vim, nano, less          # Text editors
curl, wget, ssh          # Network utilities
```

### Pipes & Redirection

Combine commands:

```bash
ls | grep file           # Pipe output to filter
cat file.txt > out.txt   # Redirect to file
echo "text" >> file.txt  # Append to file
grep pattern < file.txt  # Read from file
```

### Command History

Access previous commands:

```bash
history              # Show command history
!10                  # Run command #10
!!                   # Run previous command
!text                # Run last command starting with text
```

## Advanced Features

### Environment Variables

Access system and user variables:

```bash
echo $HOME           # User home directory
echo $USER           # Current username
echo $PATH           # Command search path
echo $TERM           # Terminal type (xterm-256color)
```

### Creating Scripts

Create shell scripts:

```bash
# Create script
cat > myscript.sh << 'EOF'
#!/bin/bash
echo "Hello from script"
ls -la
EOF

# Make executable
chmod +x myscript.sh

# Run script
./myscript.sh
```

### File Permissions

Manage access permissions:

```bash
chmod 755 script.sh      # Make executable
chmod 644 file.txt       # Make read-only
chown user:group file    # Change ownership
```

### Background Jobs

Run commands in background:

```bash
long-running-command &   # Run in background
jobs                     # List background jobs
fg %1                    # Bring job 1 to foreground
kill %1                  # Kill job 1
```

### Process Monitoring

Monitor running processes:

```bash
ps aux                   # List all processes
top                      # Interactive process monitor (exit with q)
watch command            # Repeat command every 2 seconds
```

## Terminal Control & Configuration

### Terminal Size

Terminal automatically adjusts to browser window:

1. **Resize Browser Window**
   - Drag corner to resize
   - Terminal adapts automatically

2. **Zoom Terminal** (if supported)
   - Browser zoom affects text size
   - Terminal reflows text

3. **Check Terminal Size**
   ```bash
   stty -a              # Show terminal settings
   echo $COLUMNS        # Column count
   echo $LINES          # Line count
   ```

### Clearing Screen

```bash
clear                # Clear terminal screen
Ctrl+L               # Same as clear
reset                # Full terminal reset
```

### Environment Setup

Initial environment configured:

```bash
TERM=xterm-256color  # Color support
LANG=en_US.UTF-8     # UTF-8 encoding
HOME=/home/user      # Home directory
USER=username        # Current user
PATH=...             # Command search path
```

Modify in session:

```bash
export MY_VAR="value"    # Set variable
export TERM=dumb         # Change terminal type
```

## Known Limitations

### Terminal Size

- **Default**: 80 columns × 24 rows
- **Minimum**: Limited by browser window
- **Maximum**: Limited by display device
- **Current Implementation**: Resize logged but not fully applied (future enhancement)

### Character Support

- **UTF-8**: Full support
- **ANSI Colors**: Full 256-color support
- **Control Codes**: Most common codes supported
- **Mouse Events**: Not supported (current limitation)

### Performance

- **Network Latency**: Affects responsiveness (5-50ms typical)
- **Shell Processing**: Variable depending on command
- **Output Rate**: Can handle moderate to high output rates

### File Operations

- **File Transfer**: No built-in upload/download
- **Workaround**: Use `curl`, `wget`, or `scp` for transfers
- **Future**: Native file transfer planned

### Session Management

- **Session Duration**: No explicit timeout
- **Idle Timeout**: Server keeps-alive every 30 seconds
- **Reconnection**: New session required if disconnected
- **Persistence**: Commands don't persist between sessions

### Multiple Terminals

- **Multiple Tabs**: One terminal per connection
- **Workaround**: Use `tmux` or `screen` for multiple sessions within one terminal

## Troubleshooting Basics

### Terminal Doesn't Appear

**Symptom**: Blank page or no terminal visible

**Solutions**:
1. Check server is running
   ```bash
   curl http://127.0.0.1:8080
   ```
2. Check browser console for errors
   - Press F12 to open Developer Tools
   - Look for JavaScript errors
3. Try different browser
4. Check firewall isn't blocking WebSocket

### Can't Type Commands

**Symptom**: Keyboard input not appearing in terminal

**Solutions**:
1. Click in terminal to ensure focus
2. Try typing in different location
3. Reload page (F5)
4. Check browser console for errors

### Commands Not Executing

**Symptom**: Type command but nothing happens after Enter

**Solutions**:
1. Check for blinking cursor
2. Try simple command: `echo hello`
3. Check shell is responsive: `Ctrl+C`
4. Restart terminal session

### Output Not Displaying

**Symptom**: Commands run but no output shown

**Solutions**:
1. Try explicit output: `echo "test"`
2. Check terminal isn't scrolled up: `Ctrl+L` to clear
3. Some programs write to stderr: `2>&1` to redirect
4. Reload page if terminal hung

### Terminal Hangs or Freezes

**Symptom**: Terminal stops responding to input

**Solutions**:
1. Try `Ctrl+C` to interrupt
2. Try `Ctrl+D` to send EOF
3. Reload page to reconnect
4. Check browser console for JavaScript errors

### Connection Drops

**Symptom**: "Connection lost" message or unexpected disconnect

**Solutions**:
1. Check network connection
2. Restart server
3. Reload page to reconnect
4. Check browser network tab for WebSocket errors

### Slow or Laggy Response

**Symptom**: Keyboard delays or sluggish output

**Solutions**:
1. Check network latency: `ping server-ip`
2. Check server CPU usage: `top`
3. Try simpler commands to test
4. Close other browser tabs
5. Check server logs for errors

### Character Display Issues

**Symptom**: Garbled text or wrong characters shown

**Solutions**:
1. Terminal expects UTF-8 encoding
2. Verify client browser UTF-8 enabled
3. Try: `export LANG=en_US.UTF-8`
4. Restart terminal session

## Best Practices

### Command Organization

1. **Use Clear Directory Structure**
   ```bash
   mkdir -p ~/projects/myproject
   cd ~/projects/myproject
   ```

2. **Create Scripts for Repeated Tasks**
   ```bash
   #!/bin/bash
   # Save as deploy.sh
   ./build.sh
   ./test.sh
   ./upload.sh
   ```

3. **Document Complex Commands in Comments**
   ```bash
   # Delete all .tmp files recursively
   find . -name "*.tmp" -delete
   ```

### Efficiency Tips

1. **Use Command Aliases**
   ```bash
   alias ll='ls -la'
   alias mktest='mkdir test && cd test'
   ```

2. **Leverage History**
   - Press Up Arrow repeatedly
   - Use Ctrl+R for reverse search

3. **Batch Similar Operations**
   ```bash
   for file in *.txt; do
       echo "Processing $file"
       process_file "$file"
   done
   ```

### Safety Practices

1. **Use Confirm Flags for Destructive Commands**
   ```bash
   rm -i file.txt       # Ask before deleting
   mv -i old new        # Ask before overwriting
   ```

2. **Test with Echo First**
   ```bash
   # Test before running: echo the command
   echo rm -f *.tmp
   # If output looks good, run it
   rm -f *.tmp
   ```

3. **Backup Important Files**
   ```bash
   cp important.txt important.txt.bak
   ```

## Additional Resources

- **Shell Scripting**: Learn bash/sh scripting syntax
- **Command Reference**: `man command` for documentation
- **Interactive Learning**: Online shell tutorials
- **Troubleshooting**: Server logs in `/var/log/` or `~/.cache/`

## Getting Help

### In-Terminal Help

```bash
man command          # Manual pages
command --help       # Quick help
help builtin         # Shell builtin help
info topic           # Detailed information
```

### Within Terminal

```bash
# Inspect environment
env | grep TERM
printenv | sort

# Test connectivity
ping localhost
curl http://127.0.0.1:8080

# Check system resources
free -h              # Memory usage
df -h                # Disk usage
top -b -n 1          # Process snapshot
```

### Server Logs

Server logs terminal sessions for debugging:

```bash
# Check logs (location depends on configuration)
tail -f ~/.cache/cco/logs/terminal.log

# Filter for errors
grep "ERROR" ~/.cache/cco/logs/terminal.log
```

---

**Last Updated**: November 2025
**Stability**: Production Ready
**Support**: See server documentation for advanced configuration
