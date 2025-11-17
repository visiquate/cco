# PTY-Based Terminal Architecture for Claude Orchestra

## Executive Summary

This document outlines the comprehensive architecture for implementing a full PTY-based terminal with actual shell process spawning in the Claude Orchestra CCO dashboard. The implementation replaces the current command simulator with a real terminal that spawns bash/zsh processes and provides complete shell functionality.

## 1. System Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser (Frontend)                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    xterm.js                         │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │ Terminal Emulator                           │  │   │
│  │  │ - ANSI/VT100 Support                       │  │   │
│  │  │ - 256 Color Support                        │  │   │
│  │  │ - Resize Events                            │  │   │
│  │  │ - Input Capture                            │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
│                            ↕                                 │
│         WebSocket (Binary Protocol - ArrayBuffer)           │
└──────────────────────────┬──────────────────────────────────┘
                          ↕
┌──────────────────────────┴──────────────────────────────────┐
│                    Rust Backend (CCO Server)                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              WebSocket Handler                       │   │
│  │         (server.rs - terminal_handler)              │   │
│  └───────────────────┬─────────────────────────────────┘   │
│                      ↓                                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Terminal Manager                        │   │
│  │            (new module: terminal.rs)                │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │ Session Management                           │  │   │
│  │  │ - One PTY per WebSocket connection          │  │   │
│  │  │ - Session lifecycle management              │  │   │
│  │  │ - Resource cleanup on disconnect            │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  │  ┌──────────────────────────────────────────────┐  │   │
│  │  │ PTY Process Control                          │  │   │
│  │  │ - Shell spawning (bash/zsh)                 │  │   │
│  │  │ - Environment setup                         │  │   │
│  │  │ - Signal handling (SIGWINCH, SIGTERM)      │  │   │
│  │  └──────────────────────────────────────────────┘  │   │
│  └───────────────────┬─────────────────────────────────┘   │
│                      ↓                                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              portable-pty Library                    │   │
│  │  - Cross-platform PTY support                      │   │
│  │  - Process spawning                                │   │
│  │  - I/O multiplexing                                │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

## 2. PTY Management Design

### 2.1 Library Selection
**Chosen Library**: `portable-pty v0.8`
- **Rationale**: Cross-platform support (Linux, macOS, Windows)
- **Features**: Native PTY handling, process spawning, resize support
- **Already in Dependencies**: Line 47 of Cargo.toml

### 2.2 Session Management Architecture

```rust
// terminal.rs - Core structures
pub struct TerminalSession {
    id: Uuid,
    pty_master: Box<dyn portable_pty::MasterPty>,
    child_process: Box<dyn portable_pty::Child>,
    reader: Box<dyn std::io::Read + Send>,
    writer: Box<dyn std::io::Write + Send>,
    created_at: Instant,
    last_activity: Instant,
    dimensions: PtySize,
}

pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<Uuid, TerminalSession>>>,
    max_sessions: usize,
    session_timeout: Duration,
    shell_config: ShellConfig,
}
```

### 2.3 Process Lifecycle

1. **Session Creation**:
   - WebSocket connection established
   - Generate unique session ID
   - Spawn PTY with configured shell
   - Initialize I/O threads

2. **Active Session**:
   - Bidirectional data flow
   - Resize handling
   - Activity monitoring
   - Resource tracking

3. **Session Termination**:
   - WebSocket disconnect
   - SIGTERM to child process
   - Resource cleanup
   - Session removal from manager

### 2.4 Shell Detection and Configuration

```rust
pub struct ShellConfig {
    shell_path: PathBuf,      // Auto-detected or configured
    environment: HashMap<String, String>,
    working_directory: PathBuf,
    initial_dimensions: PtySize,
}

impl ShellConfig {
    pub fn auto_detect() -> Self {
        // Priority order:
        // 1. $SHELL environment variable
        // 2. /etc/passwd lookup
        // 3. Fallbacks: /bin/bash, /bin/zsh, /bin/sh
    }
}
```

## 3. WebSocket Binary Protocol

### 3.1 Message Format

```typescript
// Binary message structure
enum MessageType {
    DATA = 0x01,        // Terminal output data
    RESIZE = 0x02,      // Terminal resize event
    CONTROL = 0x03,     // Control commands
    HEARTBEAT = 0x04,   // Keep-alive
    ERROR = 0x05,       // Error messages
}

// Message Layout (binary):
// [Type: 1 byte][Length: 4 bytes][Payload: N bytes]
```

### 3.2 Data Flow Protocol

**Input Flow (Browser → Server)**:
```
1. User types in xterm.js
2. xterm.onData captures input
3. Encode as binary message:
   [0x01][length][UTF-8 bytes]
4. Send via WebSocket.send(ArrayBuffer)
5. Server receives binary frame
6. Extract payload, write to PTY stdin
```

**Output Flow (Server → Browser)**:
```
1. PTY process writes to stdout/stderr
2. Server reads from PTY master
3. Encode as binary message:
   [0x01][length][output bytes]
4. Send via WebSocket binary frame
5. Browser receives ArrayBuffer
6. Decode and write to xterm.js
```

### 3.3 Resize Protocol

```typescript
// Frontend resize event
terminal.onResize(({cols, rows}) => {
    const buffer = new ArrayBuffer(9);
    const view = new DataView(buffer);
    view.setUint8(0, MessageType.RESIZE);
    view.setUint32(1, 4, true); // length
    view.setUint16(5, cols, true);
    view.setUint16(7, rows, true);
    ws.send(buffer);
});
```

```rust
// Backend resize handling
match message_type {
    MessageType::Resize => {
        let cols = u16::from_le_bytes([payload[0], payload[1]]);
        let rows = u16::from_le_bytes([payload[2], payload[3]]);
        session.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
    }
}
```

## 4. Frontend Architecture

### 4.1 xterm.js Configuration

```javascript
// Enhanced terminal initialization
const terminal = new Terminal({
    rows: 30,
    cols: 120,
    theme: {
        background: '#0f172a',
        foreground: '#e2e8f0',
        cursor: '#3b82f6',
        selection: 'rgba(59, 130, 246, 0.3)',
    },
    fontFamily: 'JetBrains Mono, Monaco, Consolas, monospace',
    fontSize: 14,
    lineHeight: 1.2,
    cursorBlink: true,
    cursorStyle: 'block',
    scrollback: 10000,
    tabStopWidth: 8,
    bellStyle: 'both',
    macOptionIsMeta: true,
    rightClickSelectsWord: true,
    rendererType: 'canvas',  // Better performance
});
```

### 4.2 Binary WebSocket Handler

```javascript
class TerminalConnection {
    constructor(terminal) {
        this.terminal = terminal;
        this.ws = null;
        this.encoder = new TextEncoder();
        this.decoder = new TextDecoder();
        this.reconnectAttempts = 0;
        this.maxReconnects = 5;
    }

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        this.ws = new WebSocket(`${protocol}//${window.location.host}/terminal`);
        this.ws.binaryType = 'arraybuffer';

        this.ws.onopen = () => this.handleOpen();
        this.ws.onmessage = (event) => this.handleMessage(event);
        this.ws.onclose = () => this.handleClose();
        this.ws.onerror = (error) => this.handleError(error);
    }

    sendData(data) {
        if (this.ws.readyState !== WebSocket.OPEN) return;

        const encoded = this.encoder.encode(data);
        const buffer = new ArrayBuffer(5 + encoded.length);
        const view = new DataView(buffer);

        view.setUint8(0, 0x01); // DATA type
        view.setUint32(1, encoded.length, true);
        new Uint8Array(buffer, 5).set(encoded);

        this.ws.send(buffer);
    }

    handleMessage(event) {
        const buffer = event.data;
        const view = new DataView(buffer);
        const type = view.getUint8(0);
        const length = view.getUint32(1, true);
        const payload = new Uint8Array(buffer, 5, length);

        switch(type) {
            case 0x01: // DATA
                const text = this.decoder.decode(payload);
                this.terminal.write(text);
                break;
            case 0x05: // ERROR
                const error = this.decoder.decode(payload);
                console.error('Terminal error:', error);
                this.terminal.write(`\r\n\x1b[31mError: ${error}\x1b[0m\r\n`);
                break;
        }
    }
}
```

### 4.3 Enhanced Features

```javascript
// Copy/Paste support
terminal.attachCustomKeyEventHandler((event) => {
    // Ctrl+Shift+C for copy
    if (event.ctrlKey && event.shiftKey && event.code === 'KeyC') {
        const selection = terminal.getSelection();
        if (selection) {
            navigator.clipboard.writeText(selection);
            return false;
        }
    }
    // Ctrl+Shift+V for paste
    if (event.ctrlKey && event.shiftKey && event.code === 'KeyV') {
        navigator.clipboard.readText().then(text => {
            connection.sendData(text);
        });
        return false;
    }
    return true;
});

// Search functionality
const searchAddon = new SearchAddon();
terminal.loadAddon(searchAddon);

// Web links
const webLinksAddon = new WebLinksAddon();
terminal.loadAddon(webLinksAddon);
```

## 5. Backend Implementation

### 5.1 Terminal Module Structure

```rust
// terminal.rs
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Terminal {
    manager: Arc<TerminalManager>,
}

impl Terminal {
    pub async fn new(config: TerminalConfig) -> Result<Self> {
        let manager = Arc::new(TerminalManager::new(config)?);
        Ok(Self { manager })
    }

    pub async fn create_session(&self) -> Result<TerminalSession> {
        self.manager.create_session().await
    }

    pub async fn handle_websocket(&self, ws: WebSocket) -> Result<()> {
        let session = self.create_session().await?;
        let (tx, rx) = mpsc::channel(256);

        // Spawn PTY reader task
        let reader_handle = tokio::spawn(async move {
            Self::pty_reader_task(session.clone(), tx).await
        });

        // Handle WebSocket messages
        Self::websocket_handler(ws, session, rx).await?;

        reader_handle.abort();
        Ok(())
    }
}
```

### 5.2 PTY Process Spawning

```rust
impl TerminalManager {
    pub async fn create_session(&self) -> Result<TerminalSession> {
        let pty_system = NativePtySystem::default();

        let pair = pty_system.openpty(PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new(&self.shell_config.shell_path);
        cmd.env_clear();
        for (key, value) in &self.shell_config.environment {
            cmd.env(key, value);
        }
        cmd.cwd(&self.shell_config.working_directory);

        let child = pair.slave.spawn_command(cmd)?;

        let session = TerminalSession {
            id: Uuid::new_v4(),
            pty_master: pair.master,
            child_process: child,
            reader: pair.master.try_clone_reader()?,
            writer: pair.master.take_writer()?,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            dimensions: self.shell_config.initial_dimensions,
        };

        self.sessions.lock().await.insert(session.id, session.clone());
        Ok(session)
    }
}
```

### 5.3 I/O Multiplexing

```rust
async fn pty_reader_task(
    session: Arc<TerminalSession>,
    tx: mpsc::Sender<Vec<u8>>
) -> Result<()> {
    let mut buffer = [0u8; 4096];
    let mut reader = session.reader.lock().await;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                let data = buffer[..n].to_vec();
                if tx.send(data).await.is_err() {
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(e) => {
                eprintln!("PTY read error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
```

## 6. Security Architecture

### 6.1 Security Constraints

**Access Control**:
- Authentication required for terminal access
- Session tokens with expiration
- Rate limiting on WebSocket connections
- Maximum concurrent sessions per user

**Shell Restrictions**:
```rust
pub struct SecurityConfig {
    // User restrictions
    allowed_users: Vec<String>,
    max_sessions_per_user: usize,

    // Shell restrictions
    restricted_mode: bool,
    allowed_commands: Option<Vec<String>>,
    forbidden_paths: Vec<PathBuf>,

    // Resource limits
    max_cpu_percent: f32,
    max_memory_mb: usize,
    max_process_count: usize,
    session_timeout_minutes: u32,

    // Network restrictions
    disable_network_commands: bool,
    allowed_ports: Vec<u16>,
}
```

### 6.2 Input Sanitization

```rust
impl TerminalSession {
    fn sanitize_input(&self, input: &[u8]) -> Vec<u8> {
        // Filter control sequences that could be malicious
        input.iter().filter_map(|&byte| {
            match byte {
                // Allow printable ASCII and common control chars
                0x08..=0x0D | 0x20..=0x7E => Some(byte),
                // Allow UTF-8 continuation bytes
                0x80..=0xBF => Some(byte),
                // Allow UTF-8 start bytes
                0xC0..=0xFD => Some(byte),
                // Filter potentially dangerous control codes
                _ => None,
            }
        }).collect()
    }
}
```

### 6.3 Resource Management

```rust
pub struct ResourceMonitor {
    sessions: Arc<Mutex<HashMap<Uuid, ResourceUsage>>>,
    limits: ResourceLimits,
}

impl ResourceMonitor {
    pub async fn check_limits(&self, session_id: Uuid) -> Result<()> {
        let usage = self.get_usage(session_id).await?;

        if usage.memory_mb > self.limits.max_memory_mb {
            return Err(anyhow!("Memory limit exceeded"));
        }

        if usage.cpu_percent > self.limits.max_cpu_percent {
            return Err(anyhow!("CPU limit exceeded"));
        }

        if usage.process_count > self.limits.max_process_count {
            return Err(anyhow!("Process limit exceeded"));
        }

        Ok(())
    }

    pub async fn enforce_limits(&self) {
        // Run periodically to kill sessions exceeding limits
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            for (id, usage) in self.sessions.lock().await.iter() {
                if self.exceeds_limits(usage) {
                    self.terminate_session(*id).await;
                }
            }
        }
    }
}
```

### 6.4 Signal Handling Safety

```rust
impl TerminalSession {
    pub async fn handle_signal(&mut self, signal: Signal) -> Result<()> {
        match signal {
            Signal::SIGWINCH(cols, rows) => {
                // Safe resize
                self.pty_master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })?;
            }
            Signal::SIGTERM | Signal::SIGKILL => {
                // Graceful shutdown
                self.child_process.kill()?;
                self.cleanup().await?;
            }
            Signal::SIGTSTP => {
                // Suspend (Ctrl+Z)
                self.child_process.signal(nix::sys::signal::Signal::SIGTSTP)?;
            }
            _ => {
                // Ignore other signals for security
            }
        }
        Ok(())
    }
}
```

## 7. Integration Points

### 7.1 Server.rs Modifications

```rust
// server.rs - Updated terminal handler
async fn terminal_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>
) -> Response {
    // Check authentication
    ws.on_upgrade(move |socket| handle_terminal_socket(socket, state))
}

async fn handle_terminal_socket(
    socket: WebSocket,
    state: Arc<ServerState>
) {
    // Create terminal instance
    let terminal = match Terminal::new(state.terminal_config.clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create terminal: {}", e);
            return;
        }
    };

    // Handle the WebSocket with full PTY
    if let Err(e) = terminal.handle_websocket(socket).await {
        eprintln!("Terminal session error: {}", e);
    }
}
```

### 7.2 Dashboard.js Integration

```javascript
// dashboard.js - Enhanced terminal initialization
function initTerminal() {
    const terminalElement = document.getElementById('terminal');
    if (!terminalElement) return;

    // Create terminal with addons
    state.terminal = new Terminal(terminalConfig);
    state.fitAddon = new FitAddon();
    state.searchAddon = new SearchAddon();
    state.webLinksAddon = new WebLinksAddon();

    state.terminal.loadAddon(state.fitAddon);
    state.terminal.loadAddon(state.searchAddon);
    state.terminal.loadAddon(state.webLinksAddon);

    state.terminal.open(terminalElement);
    state.fitAddon.fit();

    // Initialize connection
    state.connection = new TerminalConnection(state.terminal);
    state.connection.connect();

    // Handle terminal input
    state.terminal.onData(data => {
        state.connection.sendData(data);
    });

    // Handle resize
    state.terminal.onResize(({cols, rows}) => {
        state.connection.sendResize(cols, rows);
    });
}
```

### 7.3 Build System Updates

```toml
# Cargo.toml - Already has dependencies
[dependencies]
portable-pty = "0.8"
bytes = "1.5"  # For binary protocol handling

[target.'cfg(unix)'.dependencies]
nix = { version = "0.27", features = ["signal", "process"] }
```

## 8. Implementation Checklist

### Phase 1: Backend Core
- [ ] Create `terminal.rs` module
- [ ] Implement `TerminalManager` struct
- [ ] Implement `TerminalSession` lifecycle
- [ ] Add PTY spawning logic
- [ ] Implement binary protocol encoder/decoder

### Phase 2: WebSocket Integration
- [ ] Update `terminal_handler` in server.rs
- [ ] Implement binary message handling
- [ ] Add resize protocol support
- [ ] Implement heartbeat/keepalive

### Phase 3: Frontend Enhancement
- [ ] Update dashboard.js terminal initialization
- [ ] Implement `TerminalConnection` class
- [ ] Add binary protocol handling
- [ ] Integrate xterm.js addons

### Phase 4: Security Implementation
- [ ] Add authentication checks
- [ ] Implement resource monitoring
- [ ] Add input sanitization
- [ ] Configure security limits

### Phase 5: Testing & Polish
- [ ] Unit tests for terminal module
- [ ] Integration tests for WebSocket flow
- [ ] Security penetration testing
- [ ] Performance optimization
- [ ] Documentation updates

## 9. Deployment Considerations

### 9.1 Environment Variables
```bash
# Terminal configuration
CCO_TERMINAL_SHELL=/bin/bash
CCO_TERMINAL_MAX_SESSIONS=10
CCO_TERMINAL_TIMEOUT_MINUTES=30
CCO_TERMINAL_RESTRICTED_MODE=false
```

### 9.2 Docker Configuration
```dockerfile
# Ensure PTY support in container
RUN apt-get update && apt-get install -y \
    bash \
    zsh \
    coreutils \
    && rm -rf /var/lib/apt/lists/*

# Set proper permissions
RUN chmod 666 /dev/ptmx
```

### 9.3 Monitoring Metrics
- Active terminal sessions
- Session duration
- Data throughput (bytes/sec)
- Resource usage per session
- Error rates and types

## 10. Risk Mitigation

### High-Risk Areas
1. **Shell Injection**: Mitigated by PTY isolation
2. **Resource Exhaustion**: Mitigated by limits and monitoring
3. **Privilege Escalation**: Mitigated by process isolation
4. **Data Leakage**: Mitigated by session isolation

### Monitoring Requirements
- Real-time resource tracking
- Audit logging of all commands
- Anomaly detection for suspicious patterns
- Automatic session termination on violations

## Conclusion

This architecture provides a secure, performant, and fully-featured terminal implementation for Claude Orchestra. The design prioritizes security through multiple layers of protection while maintaining the flexibility and power of a real shell environment. The binary WebSocket protocol ensures efficient data transfer, and the PTY management system provides proper process isolation and resource control.