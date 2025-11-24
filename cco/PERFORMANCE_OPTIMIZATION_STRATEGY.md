# CCO Terminal Performance Optimization Strategy

**Date:** 2025-01-17
**Engineer:** Performance Engineer
**Project:** CCO WASM Terminal Implementation
**Version:** 1.0

---

## Executive Summary

This document outlines the **complete optimization strategy** for CCO's terminal implementation, from immediate quick wins to long-term WASM enhancements.

**Strategic Approach:**
1. **Fix baseline issues first** (12 hours) - Immediate UX improvements
2. **Evaluate WASM viability** (16 hours) - Prototype and measure
3. **Decide on WASM investment** (Based on data) - Go/No-Go decision
4. **Long-term enhancements** (Future) - Session persistence, tabs, etc.

**Key Principle:** **Measure twice, cut once** - Always validate assumptions with data before committing effort.

---

## 1. Three-Phase Optimization Roadmap

### Phase 1: Quick Wins (12 hours) - IMMEDIATE PRIORITY

**Goal:** Fix known issues and implement low-effort, high-impact optimizations.

**Timeline:** Sprint 1 (1-2 weeks)

**Effort:** 12 hours total

**Impact:**
- ✅ 90% CPU reduction during heavy output (WebGL addon)
- ✅ Memory capped at 65MB (scrollback limit)
- ✅ Proper vim/tmux support (PTY resize)
- ✅ Better first-impression UX (prompt delay fix)

#### Task 1.1: Load xterm.js WebGL Addon (2 hours)

**Priority:** CRITICAL

**Current State:**
- Canvas rendering: 60% of CPU during heavy output (7-13% total)
- No GPU acceleration

**Target State:**
- WebGL rendering: 10% of CPU during heavy output (1-2% total)
- **90% CPU reduction**

**Implementation:**

```javascript
// dashboard.js - Add WebGL addon
import { WebglAddon } from 'xterm-addon-webgl';

function initTerminal() {
    // ... existing terminal initialization ...

    state.terminal = new Terminal(terminalConfig);
    state.fitAddon = new FitAddon();
    state.terminal.loadAddon(state.fitAddon);

    // NEW: Load WebGL addon with fallback
    try {
        const webglAddon = new WebglAddon();
        webglAddon.onContextLoss(() => {
            webglAddon.dispose();
            console.warn('WebGL context lost, terminal will use Canvas rendering');
        });
        state.terminal.loadAddon(webglAddon);
        console.log('WebGL rendering enabled');
    } catch (e) {
        console.warn('WebGL not supported, falling back to Canvas:', e);
        // Terminal continues with Canvas rendering (no user impact)
    }

    state.terminal.open(terminalElement);
    // ... rest of initialization ...
}
```

**Testing:**
```bash
# Verify GPU acceleration is working
# 1. Open Chrome DevTools → Rendering → Frame Rendering Stats
# 2. Run heavy output: yes | head -10000
# 3. Verify GPU is active and CPU usage < 5%

# Test fallback
# 1. Disable GPU in chrome://flags
# 2. Verify terminal still works (Canvas fallback)
```

**Acceptance Criteria:**
- ✅ WebGL addon loads successfully on Chrome, Firefox, Safari
- ✅ CPU usage < 5% during heavy output (vs current 8-22%)
- ✅ Graceful fallback to Canvas if WebGL unavailable
- ✅ No visual regressions

**Effort Breakdown:**
- Implementation: 1 hour
- Testing (all browsers): 0.5 hours
- Documentation: 0.5 hours

---

#### Task 1.2: Set Scrollback Limit (1 hour)

**Priority:** HIGH

**Current State:**
- Default scrollback: 10,000 lines
- Memory growth: 1.5 KB/line
- Max memory: 150+ MB after heavy output

**Target State:**
- Scrollback: 1,000 lines max
- Memory capped: 65 MB max
- User can still scroll through recent history

**Implementation:**

```javascript
// dashboard.js - Limit scrollback
const terminal = new Terminal({
    rows: 30,
    cols: 120,
    scrollback: 1000,  // NEW: Limit to 1,000 lines
    theme: darkTheme,
    fontFamily: 'Monaco, Menlo, Ubuntu Mono, Consolas, "Courier New", monospace',
    fontSize: 14,
    cursorBlink: true,
    cursorStyle: 'block',
    allowProposedApi: true,
});
```

**User Communication:**
```javascript
// Optional: Add tooltip explaining limit
<div class="terminal-info">
    <p>Terminal scrollback limited to 1,000 lines for optimal performance.
       Use <code>| tee output.log</code> to save longer output.</p>
</div>
```

**Testing:**
```bash
# Verify scrollback limit works
seq 1 5000 | cat
# Scroll to top - should only see last ~1000 lines

# Verify memory doesn't exceed limit
# 1. Open DevTools → Memory → Take heap snapshot
# 2. Run: seq 1 10000 | cat
# 3. Take another snapshot
# 4. Compare - should be < 20 MB growth
```

**Acceptance Criteria:**
- ✅ Scrollback capped at 1,000 lines
- ✅ Memory growth < 15 MB for 10K line output
- ✅ No UX degradation (1K lines is sufficient for 99% of use cases)

**Effort Breakdown:**
- Implementation: 0.25 hours
- Testing: 0.5 hours
- Documentation: 0.25 hours

---

#### Task 1.3: Fix PTY Resize (4 hours)

**Priority:** HIGH

**Current State:**
- Resize method is no-op (returns Ok() without action)
- Browser resize doesn't trigger shell SIGWINCH
- vim/tmux unusable (output wrapping breaks)

**Target State:**
- Proper SIGWINCH delivery to shell
- vim/tmux work correctly
- Output reflows on window resize

**Implementation:**

```rust
// terminal.rs - Fix set_terminal_size()
use nix::libc::{ioctl, TIOCSWINSZ, winsize};
use std::os::unix::io::AsRawFd;

pub async fn set_terminal_size(&self, cols: u16, rows: u16) -> Result<()> {
    trace!(
        session_id = %self.session_id,
        cols = cols,
        rows = rows,
        "Resizing terminal"
    );

    // Lock the master PTY
    let master = self.master.lock().await;

    // Get the raw file descriptor
    let fd = master.read_fd.as_raw_fd();

    // Create winsize structure
    let ws = winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // Call ioctl to set terminal size
    // SAFETY: fd is valid, ws is properly initialized
    let result = unsafe { ioctl(fd, TIOCSWINSZ, &ws) };

    if result < 0 {
        let err = std::io::Error::last_os_error();
        error!(
            session_id = %self.session_id,
            error = %err,
            "Failed to resize terminal"
        );
        return Err(anyhow!("Failed to resize terminal: {}", err));
    }

    info!(
        session_id = %self.session_id,
        cols = cols,
        rows = rows,
        "Terminal resized successfully"
    );

    Ok(())
}
```

**Frontend Integration:**

```javascript
// dashboard.js - Send resize events
state.terminal.onResize(({cols, rows}) => {
    if (state.ws && state.ws.readyState === WebSocket.OPEN) {
        // Send resize message as special control sequence
        const msg = `RESIZE${cols}x${rows}`;
        const encoder = new TextEncoder();
        state.ws.send(encoder.encode(msg));
    }
});
```

**Backend Handler:**

```rust
// server.rs - Handle resize messages in WebSocket handler
async fn handle_terminal_socket(socket: WebSocket, state: Arc<ServerState>) {
    // ... existing code ...

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                let text = String::from_utf8_lossy(&data);

                // Check for resize control message
                if text.starts_with("RESIZE") {
                    if let Some(size) = text.strip_prefix("RESIZE") {
                        if let Some((cols_str, rows_str)) = size.split_once('x') {
                            if let (Ok(cols), Ok(rows)) = (
                                cols_str.parse::<u16>(),
                                rows_str.parse::<u16>()
                            ) {
                                if let Err(e) = session.set_terminal_size(cols, rows).await {
                                    error!("Failed to resize terminal: {}", e);
                                }
                                continue; // Don't write resize message to PTY
                            }
                        }
                    }
                }

                // Normal input handling
                session.write_input(&data).await?;
            }
            // ... rest of handler ...
        }
    }
}
```

**Testing:**

```bash
# Test resize with vim
1. Open terminal
2. Run: vim test.txt
3. Resize browser window
4. Verify vim redraws correctly
5. Add text that wraps
6. Resize again
7. Verify text reflows correctly

# Test resize with tmux
1. Run: tmux
2. Create multiple panes (Ctrl+B, %)
3. Resize browser window
4. Verify all panes resize proportionally

# Test resize with long output
1. Run: seq 1 1000
2. Resize to smaller width
3. Verify output reflows (lines wrap)
4. Resize to larger width
5. Verify output expands
```

**Acceptance Criteria:**
- ✅ vim works correctly on resize
- ✅ tmux panes resize correctly
- ✅ Output reflows on window resize
- ✅ No errors in browser console or server logs

**Effort Breakdown:**
- Implementation: 2 hours
- Testing: 1.5 hours
- Documentation: 0.5 hours

---

#### Task 1.4: Fix Initial Prompt Delay (2 hours)

**Priority:** MEDIUM

**Current State:**
- Blank terminal for 1-2s on connect (race condition)
- Users think terminal is broken
- Poor first impression

**Target State:**
- Prompt appears within 100ms
- Immediate visual feedback

**Implementation:**

```javascript
// dashboard.js - Force prompt after connection
function initTerminalWebSocket() {
    // ... existing WebSocket setup ...

    state.ws.onopen = () => {
        console.log('[WebSocket] Connection opened successfully');
        updateTerminalConnectionStatus(true);

        // NEW: Force prompt if no output after 100ms
        let promptTimeout = setTimeout(() => {
            // Send newline to force prompt
            if (state.ws && state.ws.readyState === WebSocket.OPEN) {
                const encoder = new TextEncoder();
                state.ws.send(encoder.encode('\n'));
                console.log('[Terminal] Sent newline to trigger prompt');
            }
        }, 100);

        // Clear timeout if we receive output
        const originalOnMessage = state.ws.onmessage;
        state.ws.onmessage = (event) => {
            clearTimeout(promptTimeout);
            promptTimeout = null;
            originalOnMessage(event);
        };

        // ... rest of onopen handler ...
    };
}
```

**Alternative Approach (Backend):**

```rust
// terminal.rs - Send initial newline after PTY spawn
impl TerminalSession {
    pub fn spawn_shell() -> Result<Self> {
        // ... existing spawn logic ...

        let session = TerminalSession {
            session_id: session_id.clone(),
            child: Arc::new(Mutex::new(Some(child))),
            master: Arc::new(Mutex::new(PtyMaster::from_fd(master_fd))),
        };

        // NEW: Send initial newline to trigger prompt
        // This ensures the shell displays its prompt immediately
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = session.write_input(b"\n").await;
        });

        Ok(session)
    }
}
```

**Testing:**

```bash
# Verify prompt appears quickly
1. Open terminal tab
2. Measure time to first prompt (should be < 200ms)
3. Repeat 10 times to verify consistency

# Verify no double prompt
1. Connect to terminal
2. Verify only ONE prompt appears (not two)
3. Type command immediately after connect
4. Verify command executes correctly
```

**Acceptance Criteria:**
- ✅ Prompt appears within 200ms of connection
- ✅ No double prompts or extra newlines
- ✅ Works across all shells (bash, zsh, sh)

**Effort Breakdown:**
- Implementation: 1 hour
- Testing: 0.5 hours
- Documentation: 0.5 hours

---

#### Task 1.5: Improve Reconnect UX (2 hours)

**Priority:** MEDIUM

**Current State:**
- Silent reconnect is confusing
- Users don't know if frozen or reconnecting

**Target State:**
- Clear "Reconnecting..." message
- Success/failure notifications

**Implementation:**

```javascript
// dashboard.js - Reconnect UX improvements
function initTerminalWebSocket() {
    // ... existing setup ...

    state.ws.onclose = () => {
        console.log('[WebSocket] Connection closed');
        updateTerminalConnectionStatus(false);

        // NEW: Show reconnecting message in terminal
        if (state.terminal) {
            state.terminal.write('\r\n\x1b[33m✗ Connection lost. Reconnecting...\x1b[0m\r\n');
        }

        // Auto-reconnect with exponential backoff
        let reconnectDelay = 1000; // Start with 1s
        const maxDelay = 30000;    // Cap at 30s
        let reconnectAttempt = 0;

        const reconnect = () => {
            reconnectAttempt++;

            if (state.terminal) {
                state.terminal.write(
                    `\r\x1b[33m⟳ Reconnecting (attempt ${reconnectAttempt})...\x1b[0m\r\n`
                );
            }

            setTimeout(() => {
                if (state.currentTab === 'terminal') {
                    try {
                        initTerminalWebSocket();
                    } catch (e) {
                        console.error('Reconnect failed:', e);
                        reconnectDelay = Math.min(reconnectDelay * 2, maxDelay);
                        reconnect();
                    }
                }
            }, reconnectDelay);
        };

        reconnect();
    };

    state.ws.onopen = () => {
        // ... existing onopen handler ...

        // NEW: Show reconnection success
        if (reconnectAttempt > 0) {
            state.terminal.write(
                '\r\n\x1b[32m✓ Reconnected successfully\x1b[0m\r\n\r\n'
            );
        }
    };
}
```

**Visual Indicator:**

```javascript
// Add connection indicator to UI
function updateTerminalConnectionStatus(isConnected) {
    const statusEl = document.getElementById('connectionStatus');
    if (!statusEl) return;

    if (isConnected) {
        statusEl.className = 'connection-status connected';
        statusEl.innerHTML = '<span class="status-dot green"></span> Connected';
    } else {
        statusEl.className = 'connection-status disconnected';
        statusEl.innerHTML = '<span class="status-dot red"></span> Reconnecting...';
    }
}
```

**CSS:**

```css
.connection-status {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
}

.connection-status.connected {
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
}

.connection-status.disconnected {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    animation: pulse 2s infinite;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 6px;
}

.status-dot.green {
    background: #10b981;
}

.status-dot.red {
    background: #ef4444;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}
```

**Testing:**

```bash
# Test reconnect flow
1. Open terminal
2. Kill server (simulate connection loss)
3. Verify "Reconnecting..." message appears
4. Restart server
5. Verify "Reconnected successfully" message
6. Verify terminal is functional

# Test multiple reconnects
1. Disconnect/reconnect 5 times rapidly
2. Verify no memory leaks
3. Verify terminal remains functional
```

**Acceptance Criteria:**
- ✅ Clear reconnecting status visible
- ✅ Exponential backoff prevents flooding
- ✅ Success message on reconnect
- ✅ Terminal functional after reconnect

**Effort Breakdown:**
- Implementation: 1 hour
- Testing: 0.5 hours
- Documentation: 0.5 hours

---

#### Task 1.6: Add Session Limit (1 hour)

**Priority:** MEDIUM

**Current State:**
- No limit on concurrent terminal sessions
- Can exhaust file descriptors (crash server)

**Target State:**
- Maximum 10 concurrent sessions per IP
- Graceful rejection with error message

**Implementation:**

```rust
// server.rs - Add session tracking
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

struct TerminalSessionTracker {
    sessions: Arc<Mutex<HashMap<String, usize>>>, // IP -> session count
    max_sessions_per_ip: usize,
}

impl TerminalSessionTracker {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions_per_ip: 10,
        }
    }

    async fn can_create_session(&self, ip: &str) -> bool {
        let sessions = self.sessions.lock().await;
        sessions.get(ip).map_or(true, |count| *count < self.max_sessions_per_ip)
    }

    async fn register_session(&self, ip: String) {
        let mut sessions = self.sessions.lock().await;
        *sessions.entry(ip).or_insert(0) += 1;
    }

    async fn unregister_session(&self, ip: &str) {
        let mut sessions = self.sessions.lock().await;
        if let Some(count) = sessions.get_mut(ip) {
            *count -= 1;
            if *count == 0 {
                sessions.remove(ip);
            }
        }
    }
}

async fn terminal_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<ServerState>>
) -> Response {
    let ip = addr.ip().to_string();

    // Check session limit
    if !state.session_tracker.can_create_session(&ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many terminal sessions. Maximum 10 concurrent sessions per IP."
        ).into_response();
    }

    // Register session
    state.session_tracker.register_session(ip.clone()).await;

    ws.on_upgrade(move |socket| async move {
        handle_terminal_socket(socket, state.clone()).await;

        // Unregister session on close
        state.session_tracker.unregister_session(&ip).await;
    })
}
```

**Testing:**

```bash
# Test session limit
1. Open 10 terminal tabs (should all work)
2. Try to open 11th terminal
3. Verify error message displayed
4. Close one terminal
5. Open new terminal (should work now)

# Test cleanup
1. Open 5 terminals
2. Close all tabs
3. Open new terminal (should work - sessions cleaned up)
```

**Acceptance Criteria:**
- ✅ Maximum 10 sessions per IP enforced
- ✅ Clear error message on limit exceeded
- ✅ Sessions cleaned up on disconnect
- ✅ No session leaks

**Effort Breakdown:**
- Implementation: 0.5 hours
- Testing: 0.25 hours
- Documentation: 0.25 hours

---

### Phase 1 Summary

**Total Effort:** 12 hours

**Total Impact:**
- ✅ 90% CPU reduction (WebGL addon)
- ✅ Memory capped at 65MB (scrollback limit)
- ✅ vim/tmux fully functional (PTY resize)
- ✅ Better UX on connect (prompt delay fix)
- ✅ Clear reconnect status (UX improvement)
- ✅ Server stability (session limit)

**Next Steps:**
- Deploy to production
- Monitor metrics for 1-2 weeks
- Collect user feedback
- Decide on Phase 2 (WASM evaluation)

---

## 2. Phase 2: WASM Evaluation (16 hours) - CONDITIONAL

**Goal:** Build prototype and measure actual performance gains before committing to full implementation.

**Timeline:** Sprint 2-3 (after Phase 1 deployed)

**Effort:** 16 hours

**Deliverables:**
- Minimal WASM VT100 parser
- Performance benchmarks
- Go/No-Go decision

### Task 2.1: Profile JavaScript Hotspots (4 hours)

**Objective:** Identify exact performance bottlenecks in current implementation.

**Tools:**
- Chrome DevTools Performance tab
- Firefox Profiler
- Safari Web Inspector Timelines

**Profiling Scenarios:**

```javascript
// 1. Profile typical interactive session
// Record 30s: typing commands, vim, ls -la

// 2. Profile heavy output
// Record: yes | head -10000

// 3. Profile burst output
// Record: cat large_file.txt
```

**Analysis:**

```
Expected Hotspots:
1. VT100 parsing (xterm.js)
   - ANSI escape sequence parsing
   - UTF-8 decoding
   - String manipulation

2. Canvas rendering
   - Already addressed by WebGL addon in Phase 1

3. String operations
   - Concatenation, substring
   - UTF-8 encoding/decoding
```

**Deliverable:**
- Flamegraph of JavaScript execution
- Top 10 functions by CPU time
- Estimated improvement from WASM (conservative)

---

### Task 2.2: Design WASM Module (4 hours)

**Objective:** Define minimal WASM module interface.

**Scope Decision:**

**Option A: VT100 Parser Only (RECOMMENDED)**
- Parse ANSI escape sequences in Rust
- Pass parsed commands to xterm.js
- Minimal integration complexity
- **Estimated improvement:** 10-15ms latency

**Option B: Full Terminal Emulator**
- VT100 parser + Canvas renderer in Rust
- Complete replacement of xterm.js
- High integration complexity
- **Estimated improvement:** 15-25ms latency
- **Risk:** 3-5x more work

**Recommendation:** Start with Option A (VT100 parser only)

**Interface Design:**

```rust
// lib.rs - Minimal WASM VT100 parser
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VT100Parser {
    state: ParserState,
}

#[wasm_bindgen]
impl VT100Parser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: ParserState::new(),
        }
    }

    /// Parse raw PTY output
    /// Returns array of parsed commands (JSON)
    #[wasm_bindgen]
    pub fn parse(&mut self, data: &[u8]) -> String {
        let commands = self.state.parse(data);
        serde_json::to_string(&commands).unwrap()
    }
}

// Internal parser state
struct ParserState {
    // State machine for ANSI escape sequences
    // Buffer for incomplete sequences
}

// Parsed commands (JSON serializable)
#[derive(Serialize)]
enum TerminalCommand {
    Print(String),
    CursorMove { row: u16, col: u16 },
    ClearScreen,
    SetColor { fg: u8, bg: u8 },
    // ... other VT100 commands ...
}
```

**JavaScript Integration:**

```javascript
import init, { VT100Parser } from './terminal_wasm.js';

async function initWASM() {
    await init(); // Initialize WASM module
    const parser = new VT100Parser();

    // Replace xterm.js parser with WASM
    state.ws.onmessage = (event) => {
        const data = new Uint8Array(event.data);
        const commands = JSON.parse(parser.parse(data));

        // Execute commands via xterm.js API
        commands.forEach(cmd => {
            switch(cmd.type) {
                case 'Print':
                    terminal.write(cmd.text);
                    break;
                case 'CursorMove':
                    terminal.write(`\x1b[${cmd.row};${cmd.col}H`);
                    break;
                // ... other commands ...
            }
        });
    };
}
```

**Deliverable:**
- Interface specification (Rust + JavaScript)
- Data format design (JSON commands)
- Integration plan

---

### Task 2.3: Build Prototype (6 hours)

**Objective:** Implement minimal WASM parser and integrate with current terminal.

**Implementation Steps:**

1. **Set up Rust WASM project (1 hour):**
   ```bash
   cargo new --lib terminal-wasm
   cd terminal-wasm
   cargo add wasm-bindgen serde serde_json
   ```

2. **Implement VT100 parser (3 hours):**
   ```rust
   // Basic state machine for ANSI escape sequences
   // Support most common commands (90% of terminal output)
   // Optimize for speed (zero-copy where possible)
   ```

3. **Build and package (1 hour):**
   ```bash
   wasm-pack build --target web --release
   # Generates: pkg/terminal_wasm.js, pkg/terminal_wasm_bg.wasm
   ```

4. **Integrate with dashboard.js (1 hour):**
   ```javascript
   // Load WASM module
   // Replace parser
   // Test with real terminal
   ```

**Deliverable:**
- Working WASM parser (basic VT100 support)
- Integrated into dashboard.js
- Ready for benchmarking

---

### Task 2.4: Benchmark Prototype (2 hours)

**Objective:** Measure actual performance improvement.

**Benchmarks:**

```javascript
// benchmark-wasm.js
const benchmarks = {
    // Latency: Input echo
    inputLatency: async () => {
        const results = { js: [], wasm: [] };

        // Test with xterm.js parser
        for (let i = 0; i < 1000; i++) {
            const t0 = performance.now();
            terminal.write('test\n');
            await waitForRender();
            results.js.push(performance.now() - t0);
        }

        // Test with WASM parser
        switchToWASMParser();
        for (let i = 0; i < 1000; i++) {
            const t0 = performance.now();
            terminal.write('test\n');
            await waitForRender();
            results.wasm.push(performance.now() - t0);
        }

        return {
            js: { p50: percentile(results.js, 50), p95: percentile(results.js, 95) },
            wasm: { p50: percentile(results.wasm, 50), p95: percentile(results.wasm, 95) },
        };
    },

    // Throughput: Heavy output
    throughput: async () => {
        const data = 'x'.repeat(1024 * 1024);

        // Test with JS parser
        const t0 = performance.now();
        terminal.write(data);
        await waitForRender();
        const jsTime = performance.now() - t0;

        // Test with WASM parser
        switchToWASMParser();
        const t1 = performance.now();
        terminal.write(data);
        await waitForRender();
        const wasmTime = performance.now() - t1;

        return {
            js: data.length / (jsTime / 1000),
            wasm: data.length / (wasmTime / 1000),
        };
    },

    // CPU: Heavy output
    cpu: async () => {
        // Record CPU profile during heavy output
        // Compare CPU usage: JS vs WASM
    },
};
```

**Deliverable:**
- Benchmark results comparing JS vs WASM
- Performance improvement percentage
- Decision matrix populated with data

---

### Phase 2: Go/No-Go Decision

**After 16 hours of prototyping, evaluate results:**

**GO Criteria (Proceed to Phase 3):**
- ✅ Input latency p95 improves by ≥ 15% (≥ 2ms reduction)
- ✅ CPU usage reduces by ≥ 37% (< 5% during heavy output)
- ✅ WASM binary ≤ 200 KB compressed
- ✅ No Safari crashes or compatibility issues
- ✅ Memory overhead ≤ 15 MB

**NO-GO Criteria (Abort WASM approach):**
- ❌ Input latency improves < 10% (< 1.5ms reduction)
- ❌ CPU usage reduces < 20% (still > 8% during heavy output)
- ❌ WASM binary > 250 KB compressed
- ❌ Safari performance degrades or crashes
- ❌ Memory overhead > 20 MB

**Conditional (Team Discussion):**
- ⚠️ Metrics between GO and NO-GO ranges
- ⚠️ Some improvements, some regressions
- ⚠️ Safari compatibility issues (investigate workarounds)

**Decision Document:**
- Present findings to team
- Populate decision matrix with data
- Make Go/No-Go decision
- Document rationale

---

## 3. Phase 3: Full WASM Implementation (80 hours) - CONDITIONAL

**Goal:** Complete WASM terminal implementation (only if Phase 2 GO decision).

**Timeline:** Sprint 4-8 (2-3 months)

**Effort:** 80 hours

**Prerequisites:**
- ✅ Phase 2 prototype shows ≥ 15% improvement
- ✅ Team approval
- ✅ Budget allocated

### Task 3.1: Core Implementation (40 hours)

1. **Complete VT100 Parser (16 hours):**
   - All ANSI escape sequences
   - UTF-8 handling
   - Error recovery

2. **Canvas Renderer (16 hours):**
   - WebGL integration
   - Font rendering
   - Color palettes

3. **FFI Integration (6 hours):**
   - JavaScript glue code
   - Shared buffers
   - Error handling

4. **xterm.js Integration (2 hours):**
   - Replace parser
   - Maintain compatibility

### Task 3.2: Testing (24 hours)

1. **Unit Tests (Rust) (6 hours):**
   - Parser correctness
   - Edge cases
   - Performance benchmarks

2. **Integration Tests (8 hours):**
   - End-to-end scenarios
   - Browser compatibility
   - Fallback validation

3. **Performance Testing (4 hours):**
   - Benchmark suite
   - Regression detection
   - Safari testing

4. **Memory Leak Testing (4 hours):**
   - 24-hour stability test
   - Heap profiling
   - Leak detection

5. **Security Review (2 hours):**
   - Input validation
   - Buffer overflows
   - Untrusted data handling

### Task 3.3: Documentation (8 hours)

1. **Architecture Documentation (3 hours):**
   - WASM module design
   - FFI interface
   - Performance characteristics

2. **User Documentation (2 hours):**
   - System requirements
   - Troubleshooting
   - Fallback behavior

3. **Developer Documentation (3 hours):**
   - Build process
   - Testing guide
   - Debugging techniques

### Task 3.4: Deployment (8 hours)

1. **CI/CD Integration (3 hours):**
   - Build pipeline
   - Performance tests in CI
   - Regression detection

2. **Gradual Rollout (3 hours):**
   - 10% → 50% → 100%
   - Monitoring setup
   - Rollback plan

3. **Documentation and Training (2 hours):**
   - Release notes
   - Team training
   - User communication

---

## 4. Long-Term Enhancements (Future)

**Timeline:** Post Phase 3 (6+ months out)

**Effort:** Varies

### Enhancement 4.1: Session Persistence (12 hours)

**Goal:** Reconnect to existing PTY session after disconnect.

**Benefits:**
- Better UX for flaky connections
- Resume long-running commands
- Survive browser refresh

**Implementation:**
- Store session ID in localStorage
- Keep PTY alive on server for 5 minutes after disconnect
- Reconnect to existing session if available
- Display session resume message

**Effort:**
- Backend: 6 hours
- Frontend: 4 hours
- Testing: 2 hours

---

### Enhancement 4.2: Terminal Tabs (16 hours)

**Goal:** Multiple terminals in one dashboard.

**Benefits:**
- Power user productivity
- Switch between sessions
- Run parallel tasks

**Implementation:**
- Tab UI component
- Session management
- Keyboard shortcuts (Ctrl+T, Ctrl+Tab)

**Effort:**
- UI: 8 hours
- Session management: 6 hours
- Testing: 2 hours

---

### Enhancement 4.3: Advanced Copy/Paste (8 hours)

**Goal:** Better clipboard integration.

**Benefits:**
- Mouse selection copy
- Right-click paste
- Better Safari compatibility

**Implementation:**
- Selection API
- Clipboard API
- Keyboard shortcuts

**Effort:**
- Implementation: 5 hours
- Safari compatibility: 2 hours
- Testing: 1 hour

---

## 5. Optimization Principles

### 5.1 Performance First Principles

1. **Measure Before Optimizing:**
   - Always profile before assuming bottleneck
   - Use actual data, not intuition
   - Benchmark before and after

2. **Optimize for User Perception:**
   - p95 latency more important than p50
   - Frame drops worse than high CPU
   - Consistency better than raw speed

3. **Start Simple, Add Complexity Only If Needed:**
   - Phase 1 quick wins often sufficient
   - WASM adds complexity - only if data shows benefit
   - Fallback strategy for every optimization

4. **Safari is the Constraint:**
   - Design for Safari first
   - Chrome will be faster anyway
   - Test on actual Safari, not just WebKit

5. **Avoid Premature Optimization:**
   - Fix known issues first (Phase 1)
   - Prototype before committing (Phase 2)
   - Only proceed if data supports it

### 5.2 Testing Principles

1. **Automated Regression Prevention:**
   - Run performance tests in CI
   - Block PRs with > 10% degradation
   - Track metrics over time

2. **Real-World Simulation:**
   - Test with vim, tmux, htop
   - Test with real development workflows
   - Test long-running sessions (1+ hours)

3. **Cross-Browser Validation:**
   - Safari is primary target
   - Chrome/Firefox for validation
   - Mobile Safari if time permits

4. **Memory Leak Detection:**
   - 24-hour stress test
   - Heap snapshots before/after
   - No more than 5 MB/hour growth

### 5.3 Deployment Principles

1. **Gradual Rollout:**
   - Start with 10% of users
   - Monitor metrics closely
   - Increase to 50% after 1 week
   - Full rollout after 2 weeks

2. **Feature Flags:**
   - WASM can be disabled per user
   - Fallback to xterm.js always available
   - Admin can toggle globally

3. **Monitoring:**
   - Track performance SLO violations
   - Alert on crash rate > 0.5%
   - Daily review of metrics

4. **Rollback Plan:**
   - Prepared rollback script
   - Tested fallback path
   - Clear rollback criteria

---

## 6. Risk Management

### 6.1 Technical Risks

**Risk 1: WASM doesn't improve performance enough**
- **Likelihood:** MEDIUM
- **Impact:** HIGH (wasted 80 hours)
- **Mitigation:** Phase 2 prototype and Go/No-Go decision

**Risk 2: Safari compatibility issues**
- **Likelihood:** MEDIUM
- **Impact:** HIGH (can't ship)
- **Mitigation:** Test on Safari early in Phase 2

**Risk 3: Memory leaks in WASM**
- **Likelihood:** LOW
- **Impact:** HIGH (user complaints)
- **Mitigation:** 24-hour stability test before ship

**Risk 4: Bundle size too large**
- **Likelihood:** LOW
- **Impact:** MEDIUM (slow load)
- **Mitigation:** Hard limit of 200 KB, monitor during Phase 2

### 6.2 Project Risks

**Risk 1: Phase 1 fixes are sufficient (WASM not needed)**
- **Likelihood:** HIGH
- **Impact:** LOW (save time)
- **Mitigation:** Evaluate after Phase 1 deployed

**Risk 2: User complaints about Phase 1 changes**
- **Likelihood:** LOW
- **Impact:** MEDIUM (revert changes)
- **Mitigation:** Clear communication, gradual rollout

**Risk 3: Team capacity for Phase 3**
- **Likelihood:** MEDIUM
- **Impact:** MEDIUM (delayed)
- **Mitigation:** Only proceed if team has bandwidth

---

## 7. Success Metrics

### 7.1 Phase 1 Success Criteria

**Technical:**
- ✅ CPU usage < 5% during heavy output (vs 8-22%)
- ✅ Memory capped at 65 MB (vs 85+ MB)
- ✅ vim/tmux work correctly
- ✅ Prompt appears < 200ms after connect

**User Satisfaction:**
- ✅ No user complaints about terminal performance
- ✅ Positive feedback on vim/tmux support
- ✅ No increase in support tickets

**Operational:**
- ✅ No regressions in other dashboard features
- ✅ No increase in error rate or crashes
- ✅ Server stability maintained

### 7.2 Phase 2 Success Criteria

**Technical:**
- ✅ Input latency p95 ≤ 10ms (≥ 15% improvement)
- ✅ CPU usage < 5% (≥ 37% improvement)
- ✅ WASM binary ≤ 200 KB
- ✅ Safari compatibility verified

**Decision Quality:**
- ✅ Go/No-Go decision made with data
- ✅ Team consensus on next steps
- ✅ Budget allocated (if GO)

### 7.3 Phase 3 Success Criteria

**Technical:**
- ✅ All performance targets met (see WASM_PERFORMANCE_SPECIFICATION.md)
- ✅ No regressions in any metric
- ✅ Safari performance within 20% of Chrome

**User Satisfaction:**
- ✅ Positive feedback from beta users
- ✅ No increase in support tickets
- ✅ < 0.5% crash rate

**Operational:**
- ✅ Gradual rollout completed successfully
- ✅ Monitoring dashboards showing healthy metrics
- ✅ Team trained on WASM architecture

---

## 8. Deliverables Index

### Documentation Delivered

1. **PERFORMANCE_BASELINE_REPORT.md** (This Sprint)
   - Current performance measurements
   - Bottleneck identification
   - Safari considerations

2. **WASM_PERFORMANCE_SPECIFICATION.md** (This Sprint)
   - Performance targets
   - Bundle size budget
   - Go/No-Go criteria

3. **PERFORMANCE_OPTIMIZATION_STRATEGY.md** (This Document)
   - Three-phase roadmap
   - Task breakdown with effort estimates
   - Risk management

### Code Deliverables

**Phase 1 (12 hours):**
- [ ] WebGL addon integration
- [ ] Scrollback limit configuration
- [ ] PTY resize implementation
- [ ] Initial prompt delay fix
- [ ] Reconnect UX improvements
- [ ] Session limit enforcement

**Phase 2 (16 hours):**
- [ ] JavaScript hotspot profiling
- [ ] WASM module design
- [ ] WASM prototype (basic VT100 parser)
- [ ] Benchmark suite
- [ ] Go/No-Go decision document

**Phase 3 (80 hours - CONDITIONAL):**
- [ ] Complete WASM VT100 parser
- [ ] Canvas renderer (WebGL)
- [ ] FFI integration
- [ ] Comprehensive test suite
- [ ] CI/CD integration
- [ ] Production deployment

---

## 9. Next Steps

### Immediate (This Week)

1. ✅ Review this strategy document with team
2. ✅ Get approval for Phase 1 (12 hours)
3. ✅ Assign Phase 1 tasks
4. ✅ Set up tracking (Jira, GitHub Projects, etc.)

### Short-Term (Next 2 Weeks)

1. ✅ Complete Phase 1 implementation
2. ✅ Deploy to production (gradual rollout)
3. ✅ Monitor metrics for 1-2 weeks
4. ✅ Collect user feedback

### Medium-Term (Next 1-2 Months)

1. ⚠️ Decide on Phase 2 (based on Phase 1 results)
2. ⚠️ If GO: Execute Phase 2 prototype
3. ⚠️ Make Go/No-Go decision for Phase 3
4. ⚠️ If GO: Begin Phase 3 implementation

### Long-Term (6+ Months)

1. ⚠️ Session persistence
2. ⚠️ Terminal tabs
3. ⚠️ Advanced copy/paste

---

## Conclusion

This strategy provides a **data-driven, low-risk approach** to optimizing CCO's terminal performance:

1. **Phase 1 (12h):** Fix known issues, implement quick wins
   - High impact, low risk
   - Immediate user value
   - Proven techniques (WebGL, scrollback limits)

2. **Phase 2 (16h):** Prototype WASM, measure actual gains
   - Validate assumptions with data
   - Low investment before committing
   - Clear Go/No-Go criteria

3. **Phase 3 (80h):** Full WASM implementation (only if justified)
   - Proceed only if data supports it
   - Comprehensive testing and validation
   - Gradual rollout with monitoring

**Key Principles:**
- ✅ Measure before optimizing
- ✅ Fix baseline issues first
- ✅ Prototype before committing
- ✅ Always have a fallback

**Decision Points:**
- After Phase 1: Are quick wins sufficient?
- After Phase 2: Does prototype show ≥ 15% improvement?
- After Phase 3: Are all targets met?

**Recommended Path:**
1. Execute Phase 1 immediately (high ROI)
2. Monitor results for 1-2 weeks
3. Decide on Phase 2 based on user feedback
4. Only proceed to Phase 3 if prototype justifies investment

---

**Document Version:** 1.0
**Status:** Final
**Owner:** Performance Engineer
**Review Date:** After Phase 1 deployment
