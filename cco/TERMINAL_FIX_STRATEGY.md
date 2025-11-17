# CCO Terminal Functionality Fix Strategy

## Executive Summary
The Claude Orchestra dashboard terminal is non-functional due to a fundamental mismatch between the frontend xterm.js implementation and the backend WebSocket handler. The terminal appears blank with no cursor, prompt, or shell functionality because the current implementation uses a JSON command/response pattern instead of raw terminal I/O with a real shell process.

## Root Cause Analysis

### 1. Protocol Mismatch
**Location:** `dashboard.js` lines 755-759 and `server.rs` lines 916-946

**Issue:** Frontend sends raw terminal bytes, backend expects JSON commands
- Frontend: `state.ws.send(new TextEncoder().encode(data))` (line 757)
- Backend: Expects JSON with `{"command": "..."}` structure (lines 919-920)

### 2. No Real Shell Process
**Location:** `server.rs` lines 889-989

**Issue:** Backend implements simulated commands instead of spawning actual shell
- Current: Simple pattern matching on commands (lines 957-989)
- Missing: PTY (pseudo-terminal) process management
- Missing: Shell process spawning (bash/sh/zsh)

### 3. Message Format Incompatibility
**Location:** `dashboard.js` lines 799-803 and `server.rs` lines 901-904

**Issue:** Data format mismatch between frontend and backend
- Frontend expects: Raw terminal output bytes
- Backend sends: JSON objects with `{"type": "output", "data": "..."}`

### 4. Terminal Initialization Issues
**Location:** `dashboard.js` lines 695-738

**Issue:** xterm.js and FitAddon initialization problems
- FitAddon loading timing issues (lines 713-738)
- WebSocket connection established before terminal ready
- No initial prompt or shell output displayed

## Architecture Decisions

### Decision 1: Implement Real PTY-based Terminal
**Rationale:** Users expect a real shell, not simulated commands
**Approach:** Use `portable-pty` crate for cross-platform PTY support
**Impact:** Full shell functionality with real command execution

### Decision 2: Use Raw Binary WebSocket Protocol
**Rationale:** Terminal I/O is inherently byte-based, not JSON
**Approach:** Direct byte streaming between xterm.js and PTY
**Impact:** Efficient, low-latency terminal interaction

### Decision 3: Implement Proper Session Management
**Rationale:** Multiple users may access the dashboard
**Approach:** Session-based PTY processes with cleanup
**Impact:** Isolated, secure terminal sessions

## Implementation Plan

### Phase 1: Backend Terminal Handler Rewrite (server.rs)

#### Step 1.1: Add PTY Dependencies
**File:** `cco/Cargo.toml`
**Lines to add:** After line 30
```toml
portable-pty = "0.8"
async-trait = "0.1"
bytes = "1.5"
```

#### Step 1.2: Create Terminal Session Manager
**New File:** `cco/src/terminal.rs`
**Implementation:**
- PTY process spawning with shell detection
- Session management with unique IDs
- Bidirectional I/O streaming
- Cleanup on disconnect

#### Step 1.3: Update WebSocket Handler
**File:** `cco/src/server.rs`
**Lines:** 889-954 (complete rewrite)
**Changes:**
- Remove JSON parsing
- Implement raw byte streaming
- Spawn PTY on connection
- Handle resize events

### Phase 2: Frontend Terminal Client Fix (dashboard.js)

#### Step 2.1: Fix WebSocket Message Handling
**File:** `cco/static/dashboard.js`
**Lines:** 786-817
**Changes:**
- Remove JSON parsing
- Handle raw ArrayBuffer data
- Implement proper binary mode

#### Step 2.2: Fix Terminal Initialization
**File:** `cco/static/dashboard.js`
**Lines:** 690-784
**Changes:**
- Ensure terminal ready before WebSocket
- Fix FitAddon timing issues
- Add resize handling

#### Step 2.3: Add Terminal Controls
**File:** `cco/static/dashboard.js`
**Lines:** 761-775
**Changes:**
- Implement working clear functionality
- Add copy with proper selection
- Handle terminal reset

### Phase 3: Protocol Implementation

#### Step 3.1: Define Binary Protocol
**Format:**
```
Client → Server:
- Type 0x00: Terminal input (raw bytes)
- Type 0x01: Resize event (cols, rows as u16)
- Type 0x02: Control command (clear, reset)

Server → Client:
- Type 0x00: Terminal output (raw bytes)
- Type 0x01: Control sequence (cursor, colors)
```

#### Step 3.2: Error Handling
- Graceful PTY process cleanup
- WebSocket reconnection logic
- Session restoration capability

### Phase 4: Security Considerations

#### Step 4.1: Input Sanitization
- Prevent shell injection attacks
- Limit command execution scope
- Implement timeout mechanisms

#### Step 4.2: Resource Management
- Limit concurrent PTY processes
- Implement session timeouts
- CPU/memory usage monitoring

## File Modifications Summary

### Backend Files
1. **cco/Cargo.toml** - Add PTY dependencies
2. **cco/src/terminal.rs** (NEW) - Terminal session management
3. **cco/src/server.rs:889-954** - Rewrite WebSocket handler
4. **cco/src/lib.rs** - Add terminal module

### Frontend Files
1. **cco/static/dashboard.js:690-817** - Fix terminal initialization
2. **cco/static/dashboard.js:755-759** - Fix message sending
3. **cco/static/dashboard.js:799-803** - Fix message receiving

### Testing Files
1. **cco/tests/terminal_test.rs** (NEW) - Terminal integration tests
2. **cco/tests/websocket_test.rs** (NEW) - WebSocket protocol tests

## Success Criteria

### Functional Requirements
✓ Terminal displays shell prompt on connection
✓ User can type commands and see output
✓ Terminal supports ANSI colors and control sequences
✓ Terminal resizes properly with window
✓ Copy/paste functionality works
✓ Clear screen command works
✓ Session persists during navigation

### Performance Requirements
✓ < 10ms latency for keystroke echo
✓ < 100ms for command execution start
✓ Supports 100+ concurrent sessions
✓ Memory usage < 10MB per session

### Security Requirements
✓ No shell injection vulnerabilities
✓ Sessions isolated from each other
✓ Automatic session cleanup on disconnect
✓ Rate limiting on command execution

## Risk Assessment

### High Risk
- **Shell access security**: Mitigate with restricted shell environment
- **Resource exhaustion**: Mitigate with session limits and timeouts

### Medium Risk
- **Cross-platform compatibility**: Test on Linux, macOS, Windows
- **Terminal emulation accuracy**: Use xterm.js defaults

### Low Risk
- **WebSocket stability**: Implement reconnection logic
- **Browser compatibility**: xterm.js handles this

## Timeline Estimate

### Phase 1: Backend (4-6 hours)
- PTY implementation: 2-3 hours
- WebSocket rewrite: 1-2 hours
- Testing: 1 hour

### Phase 2: Frontend (2-3 hours)
- Protocol update: 1 hour
- Initialization fixes: 1 hour
- Testing: 1 hour

### Phase 3: Integration (2-3 hours)
- End-to-end testing: 1-2 hours
- Bug fixes: 1 hour

### Total: 8-12 hours

## Alternative Approaches Considered

### 1. Keep JSON Protocol with Simulated Commands
**Rejected:** Doesn't provide real shell functionality users expect

### 2. Use SSH Instead of WebSocket
**Rejected:** Adds complexity, requires SSH server configuration

### 3. Embed Terminal via iframe to External Service
**Rejected:** Security concerns, external dependency

## Conclusion

The terminal functionality requires a complete rewrite of both frontend and backend components to implement a proper PTY-based terminal with raw binary WebSocket communication. This will provide users with a fully functional shell experience integrated into the CCO dashboard.

The implementation should be done by specialized agents:
- **Rust Specialist**: Backend PTY implementation and WebSocket handler
- **Frontend Developer**: xterm.js integration and WebSocket client
- **Security Auditor**: Review shell access and input sanitization
- **QA Engineer**: End-to-end terminal testing

This strategy ensures a robust, secure, and performant terminal implementation that meets user expectations for a development dashboard.