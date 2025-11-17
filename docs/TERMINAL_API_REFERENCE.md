# Terminal API Reference

## Overview

The Terminal System provides a WebSocket-based API for real-time shell interaction. This document specifies the protocol, message formats, and usage examples.

## WebSocket Endpoint

**Base URL**: `ws://localhost:8080/terminal` or `wss://localhost:8080/terminal` (secure)

**HTTP Upgrade**:
```http
GET /terminal HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: [random base64]
Sec-WebSocket-Version: 13
```

**Response**:
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: [response key]
```

## Message Types

### 1. Input Message (Binary)

**Direction**: Client → Server

**Purpose**: Send keyboard input and commands to shell

**Format**:
```
Message Type: Binary
Content: Raw UTF-8 bytes
Example: [0x6C, 0x73, 0x20, 0x2D, 0x6C, 0x61, 0x0A]  // "ls -la\n"
```

**Constraints**:
- Must be valid UTF-8
- May contain control characters
- No NUL bytes (0x00) allowed currently
- Maximum size: 4096 bytes per message

**Examples**:

Command execution:
```
ls -la\n
[l][s][ ][-][l][a][\n]
```

Multiple commands (bash):
```
cd /tmp && ls\n
```

Interactive input:
```
y\n  (for yes/no prompts)
password\n  (for password prompts)
```

**JavaScript Client**:
```javascript
const ws = new WebSocket('ws://localhost:8080/terminal');

ws.onopen = () => {
    // Send command
    const cmd = 'ls -la\n';
    const encoded = new TextEncoder().encode(cmd);
    ws.send(encoded);
};
```

**Python Client**:
```python
import websocket
import json

ws = websocket.create_connection('ws://localhost:8080/terminal')

# Send command
cmd = 'echo hello\n'.encode('utf-8')
ws.send_binary(cmd)

# Receive output
data = ws.recv()
print(data.decode('utf-8'))
```

---

### 2. Output Message (Binary)

**Direction**: Server → Client

**Purpose**: Send shell output to client for display

**Format**:
```
Message Type: Binary
Content: Raw UTF-8 bytes from shell
Example: [0x62, 0x61, 0x73, 0x68, 0x2D, 0x35, 0x2E, ...]  // "bash-5..."
```

**Contents**:
- Shell output (stdout)
- Shell errors (stderr) - mixed with stdout
- ANSI escape codes for colors and styling
- Control characters

**Characteristics**:
- Sent every 10ms when data available
- May contain partial output (not aligned to line boundaries)
- May contain multiple ANSI sequences
- Preserves exact byte sequence from shell

**Examples**:

Simple output:
```
$ ls
file.txt
dir1
dir2
```

With colors (ANSI codes):
```
$ ls --color
<esc>[0m<esc>[01;31mfile.txt<esc>[0m
<esc>[01;34mdir1<esc>[0m
<esc>[01;34mdir2<esc>[0m
```

Error output:
```
$ ls /nonexistent
ls: cannot access '/nonexistent': No such file or directory
```

**JavaScript Client**:
```javascript
const ws = new WebSocket('ws://localhost:8080/terminal');

ws.binaryType = 'arraybuffer';  // Important!

ws.onmessage = (event) => {
    const output = new TextDecoder().decode(event.data);
    console.log('Shell output:', output);

    // Display in terminal (xterm.js)
    terminal.write(output);
};
```

**Python Client**:
```python
import websocket

ws = websocket.create_connection('ws://localhost:8080/terminal')

while True:
    data = ws.recv()
    if isinstance(data, bytes):
        output = data.decode('utf-8')
        print(output, end='', flush=True)
```

---

### 3. Resize Message (Text)

**Direction**: Client → Server

**Purpose**: Notify server of terminal size change

**Format**:
```
Message Type: Text
Format: \x1b[RESIZE;COLS;ROWS
Example: "\x1b[RESIZE;120;40"
```

**Components**:
- Prefix: `\x1b[RESIZE;` (literal escape sequence)
- COLS: Terminal width in columns (u16)
- ROWS: Terminal height in rows (u16)
- Optional trailing newline (ignored)

**Parsing Algorithm**:
```
1. Check if message starts with "\x1b[RESIZE;"
2. Extract remainder after prefix
3. Split by semicolon: [cols_str, rows_str]
4. Parse cols_str as u16
5. Parse rows_str as u16 (trim whitespace)
6. Call session.set_terminal_size(cols, rows)
```

**Examples**:

80×24 (standard):
```
\x1b[RESIZE;80;24
```

Full HD terminal (120 columns, 40 rows):
```
\x1b[RESIZE;120;40
```

With trailing newline:
```
\x1b[RESIZE;100;30\n
```

**JavaScript Client**:
```javascript
function sendResize(cols, rows) {
    const resizeMsg = `\x1b[RESIZE;${cols};${rows}`;
    ws.send(resizeMsg);
}

// Usage
sendResize(120, 40);

// Listen for window resize
window.addEventListener('resize', () => {
    const cols = Math.floor(window.innerWidth / 8);  // Approximate
    const rows = Math.floor(window.innerHeight / 16); // Approximate
    sendResize(cols, rows);
});
```

**Python Client**:
```python
def send_resize(ws, cols, rows):
    resize_msg = f'\x1b[RESIZE;{cols};{rows}'
    ws.send(resize_msg)

# Usage
send_resize(ws, 120, 40)
```

---

### 4. Keep-Alive Message (Binary or Text)

**Direction**: Server → Client

**Purpose**: Maintain connection, detect stale sessions

**Format**:
```
Message Type: Binary (empty) or Text (empty)
Content: Empty (zero bytes)
Interval: 30 seconds
```

**Characteristics**:
- Sent automatically by server every 30 seconds
- Indicates session still alive
- Should not be displayed to user
- Client should ignore (no response needed)

**Server Implementation**:
```rust
let mut keep_alive = interval(Duration::from_secs(30));

tokio::select! {
    _ = keep_alive.tick() => {
        match session.is_running() {
            Ok(true) => {
                // Send empty keep-alive
                let msg = Message::binary(vec![]);
                sender.send(msg).await?;
            }
            _ => break,  // Session dead
        }
    }
}
```

**JavaScript Client** (xterm.js):
```javascript
ws.onmessage = (event) => {
    if (event.data.byteLength === 0) {
        // Keep-alive message, ignore
        return;
    }

    // Normal output message
    const output = new TextDecoder().decode(event.data);
    terminal.write(output);
};
```

---

## Connection Lifecycle

### 1. Establishment Phase

```
Client                          Server
  │                               │
  ├─ HTTP GET /terminal ────────→ │
  │                               │
  │ ← ─ ─ 101 Switching ─ ─ ─ ─ ┤
  │                               │
  ├─ Upgrade to WebSocket ──────→ │
  │                               │
  │ ← ─ ─ Shell prompt ─ ─ ─ ─ ┤
  │                               │
```

**Steps**:
1. Client opens WebSocket connection
2. Server accepts connection
3. Server spawns shell process
4. Server reads initial shell output
5. Server sends output to client
6. Connection ready for input

**Timeline**:
- Connection upgrade: < 10ms
- Shell spawn: 10-50ms
- Initial output: 10-100ms
- Total: 20-160ms typical

### 2. Active Phase

```
Client                          Server
  │                               │
  ├─ Input (binary) ─────────────→ │
  │                               │
  │ ← ─ ─ Output (binary) ─ ─ ─ ┤
  │                               │
  ├─ Resize (text) ──────────────→ │
  │                               │
  │ ← ─ ─ Keep-alive ─ ─ ─ ─ ─ ┤ (every 30s)
  │                               │
```

**Concurrent Operations**:
- Client sends input at any time
- Server sends output asynchronously
- Server sends keep-alive every 30 seconds
- No guaranteed ordering between different message types

**Polling Strategy**:
- Output polled every 10ms
- Keep-alive sent every 30 seconds
- Both independent operations

### 3. Termination Phase

**Client-Initiated**:
```
Client                          Server
  │                               │
  ├─ Close Frame ─────────────────→ │
  │                               │
  │ ← ─ ─ Close ACK ─ ─ ─ ─ ─ ┤
  │                               │
  │                          Kill shell
  │                          Close PTY
```

**Server-Initiated**:
```
Client                          Server
  │                               │
  │ ← ─ ─ Close Frame ─ ─ ─ ─ ┤ (shell exited)
  │                               │
  ├─ Close ACK ──────────────────→ │
  │                               │
```

**Server Cleanup**:
```rust
// Abort background task
sender_handle.abort();

// Close shell
session.close_session()?;

// Log closure
info!("Terminal session closed");
```

---

## Error Codes

The WebSocket protocol has standard codes:

| Code | Meaning | Cause | Recovery |
|------|---------|-------|----------|
| 1000 | Normal Closure | Client closed connection | Normal |
| 1001 | Going Away | Server shutdown | Reconnect |
| 1002 | Protocol Error | Invalid message format | Reload |
| 1003 | Unsupported Data | Invalid message type | Reload |
| 1006 | Abnormal Closure | Network error | Reconnect |
| 1008 | Policy Violation | Message too large | Reconnect |
| 1009 | Message Too Big | Input > 4KB | Break into smaller messages |
| 1011 | Internal Error | Server error | Report issue |

**Handling Example** (JavaScript):
```javascript
ws.onclose = (event) => {
    if (event.code === 1000) {
        console.log('Terminal closed normally');
    } else if (event.code === 1006) {
        console.warn('Connection lost, reconnecting...');
        // Attempt reconnect with exponential backoff
    } else {
        console.error('Error code:', event.code);
    }
};

ws.onerror = (event) => {
    console.error('WebSocket error:', event);
};
```

---

## Rate Limits

**Current Implementation**:
- No built-in rate limits
- Limited by TCP flow control
- Limited by browser implementation

**Recommendations**:
- Input: One message per keystroke (typical)
- Output: One message per 10ms polling interval
- Keep-alive: One message per 30 seconds

**Future Safeguards**:
- Maximum message size: 1MB
- Maximum messages per second: 1000
- Maximum connections per IP: 100
- Message queue size: 1000 messages

---

## Examples

### Example 1: Basic Connection

**HTML + JavaScript**:
```html
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/xterm@5.0.0/css/xterm.css" />
</head>
<body>
    <div id="terminal"></div>

    <script src="https://cdn.jsdelivr.net/npm/xterm@5.0.0/lib/xterm.js"></script>
    <script>
        const terminal = new Terminal();
        terminal.open(document.getElementById('terminal'));

        const ws = new WebSocket('ws://localhost:8080/terminal');
        ws.binaryType = 'arraybuffer';

        ws.onopen = () => {
            terminal.write('Connected to terminal\r\n');
        };

        ws.onmessage = (event) => {
            const output = new TextDecoder().decode(event.data);
            terminal.write(output);
        };

        ws.onerror = (error) => {
            terminal.write('Connection error: ' + error);
        };

        terminal.onData(data => {
            const encoded = new TextEncoder().encode(data);
            ws.send(encoded);
        });

        terminal.onResize(({ cols, rows }) => {
            const resizeMsg = `\x1b[RESIZE;${cols};${rows}`;
            ws.send(resizeMsg);
        });
    </script>
</body>
</html>
```

### Example 2: Python WebSocket Client

```python
#!/usr/bin/env python3
import websocket
import threading
import sys

def on_message(ws, message):
    try:
        output = message.decode('utf-8') if isinstance(message, bytes) else message
        print(output, end='', flush=True)
    except Exception as e:
        print(f"Error decoding message: {e}", file=sys.stderr)

def on_error(ws, error):
    print(f"Error: {error}", file=sys.stderr)

def on_close(ws, close_status_code, close_msg):
    print("Connection closed")

def on_open(ws):
    print("Connected to terminal")

    def run():
        while True:
            try:
                user_input = input()
                cmd = (user_input + '\n').encode('utf-8')
                ws.send_binary(cmd)
            except EOFError:
                ws.close()
                break

    thread = threading.Thread(target=run)
    thread.daemon = True
    thread.start()

if __name__ == "__main__":
    websocket.enableTrace(False)

    ws = websocket.WebSocketApp(
        "ws://localhost:8080/terminal",
        on_open=on_open,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close
    )

    ws.run_forever()
```

### Example 3: Node.js WebSocket Client

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8080/terminal');

ws.on('open', () => {
    console.log('Connected to terminal');

    // Send command
    ws.send(Buffer.from('ls -la\n', 'utf-8'));
});

ws.on('message', (data) => {
    if (Buffer.isBuffer(data) && data.length > 0) {
        console.log(data.toString('utf-8'));
    }
});

ws.on('error', (error) => {
    console.error('Error:', error);
});

ws.on('close', () => {
    console.log('Connection closed');
    process.exit(0);
});

// Handle keyboard input
process.stdin.setRawMode(true);
process.stdin.on('data', (chunk) => {
    ws.send(chunk);
});
```

### Example 4: cURL with WebSocket

```bash
# Note: Standard cURL doesn't support WebSocket
# Use websocat instead:

# Install websocat
cargo install websocat

# Connect to terminal
websocat ws://localhost:8080/terminal

# Type commands
ls -la
echo hello

# Ctrl+C to exit
```

---

## Specifications

### Message Size Limits

- **Input**: 4096 bytes maximum per message
- **Output**: 4096 bytes maximum per message (multiple messages for larger output)
- **Resize**: ~30 bytes typical

### Timing

- **Output Polling**: Every 10 milliseconds
- **Keep-Alive**: Every 30 seconds
- **Process Timeout**: 5 seconds (kill to termination)
- **Typical Latency**: 15-50ms per round-trip

### Character Encoding

- **Required**: UTF-8 only
- **Line Endings**: Both LF (`\n`) and CRLF (`\r\n`) supported
- **Control Characters**: Most ANSI escape codes supported
- **Invalid UTF-8**: Connection may close

### Concurrency

- **Multiple Connections**: Supported (separate shell per connection)
- **Session Isolation**: Completely independent sessions
- **No Shared State**: Each session owns its PTY and shell process
- **Connection Limit**: Determined by system file descriptor limit

---

## Security Considerations

### Authentication

- Terminal endpoint requires authentication at HTTP layer
- No built-in API key or token validation
- Implement at reverse proxy or application level

### Input Validation

- No filtering of shell input
- User responsible for safe commands
- No command whitelisting

### Output Security

- Raw output transmitted (may contain sensitive data)
- Use WSS (WebSocket Secure) in production
- Consider network encryption

### Resource Limits

- 4KB buffers per session
- No bandwidth throttling
- Implement rate limiting at reverse proxy

---

**Last Updated**: November 2025
**Version**: 1.0
**Status**: Complete API Specification
