# Agent Briefing: xterm.js Terminal Integration (Issue #26)

## CRITICAL: READ BEFORE STARTING

Per ORCHESTRATOR_RULES.md:
1. Each agent MUST read this FIRST
2. Each agent MUST read /Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md
3. Each agent should read FULLY without limits or offsets
4. Coordinate via Knowledge Manager

---

## BRIEF

**Status**: Ready for implementation
**Priority**: High
**Dependencies**: Rust backend (should be compiled first)
**Timeline**: Single sprint delivery
**Team Size**: 5 agents (parallel execution)

---

## AGENT ASSIGNMENTS

### AGENT 1: Chief Architect
**Model**: Opus 4.1
**Type**: architect
**Responsibility**: System design and coordination oversight

**TASK**: Design complete xterm.js terminal integration architecture

**Deliverables**:
1. Terminal integration architecture design document
2. Component interaction diagram
3. Error recovery and reconnection strategy
4. Theme management approach
5. Security design review for binary WebSocket protocol

**Key Decisions to Make**:
- Terminal state management approach
- WebSocket lifecycle management
- Error recovery strategy (exponential backoff? fixed interval?)
- Theme switching mechanism
- Connection status indicator design

**Report To**: Knowledge Manager with tag "architect-design"
**Blocking?**: No (but informs other agents)

---

### AGENT 2: Frontend Developer
**Model**: Sonnet 4.5
**Type**: frontend-expert
**Responsibility**: Implementation of xterm.js integration

**CRITICAL INSTRUCTION**:
**Read `/Users/brent/git/cc-orchestra/cco/static/dashboard.js` COMPLETELY** - lines 686-817 contain existing code that needs to be completed. Read the FULL file without limits or offset restrictions. Your context is dedicated and large enough to handle it.

**TASK**: Implement complete xterm.js terminal integration in dashboard.js

**File Locations**:
- **Target**: `/Users/brent/git/cc-orchestra/cco/static/dashboard.js`
- **Current Code**: Lines 686-817 (initTerminal and initTerminalWebSocket functions)
- **HTML**: `/Users/brent/git/cc-orchestra/cco/static/dashboard.html` (lines 11-13: libraries, line 243: #terminal div)
- **Rust Backend**: `/Users/brent/git/cc-orchestra/cco/src/server.rs` (lines 940-1036: WebSocket handler)

**Implementation Checklist**:

1. **Terminal Initialization**
   - [ ] Create Terminal instance with proper options
   - [ ] Options: rows: 25, cols: 120, fontSize: 14, fontFamily: monospace
   - [ ] Set theme: dark background (#1e293b), light text (#f1f5f9)
   - [ ] Open terminal in #terminal div
   - [ ] Set cursorBlink: true

2. **FitAddon Integration**
   - [ ] Load FitAddon addon
   - [ ] Call fit() initially
   - [ ] Setup window resize listener
   - [ ] Only fit when terminal tab is active

3. **WebSocket Connection (BINARY PROTOCOL)**
   - [ ] Connect to `/api/terminal` endpoint
   - [ ] Set `binaryType: 'arraybuffer'` (CRITICAL - binary, not text)
   - [ ] Use protocol: `ws://` or `wss://` based on page protocol
   - [ ] NO JSON encoding - pure UTF-8 bytes

4. **Keyboard Input Handler**
   - [ ] Implement `terminal.onData(data => { ... })`
   - [ ] Convert input to UTF-8 bytes: `new TextEncoder().encode(data)`
   - [ ] Send via WebSocket: `ws.send(bytes)`
   - [ ] Handle connection state (OPEN only)
   - [ ] Support special keys: Ctrl+C, Ctrl+D, arrows
   - [ ] Support paste: Cmd+V (Mac), Ctrl+V (others)

5. **Output Handler**
   - [ ] Implement `ws.onmessage = (event) => { ... }`
   - [ ] Check: `event.data instanceof ArrayBuffer`
   - [ ] Decode bytes: `new TextDecoder().decode(new Uint8Array(event.data))`
   - [ ] Write to terminal: `terminal.write(text)`
   - [ ] Real-time streaming (no buffering)

6. **Connection Management**
   - [ ] Handle `ws.onopen`: Log connection
   - [ ] Handle `ws.onerror`: Log and report
   - [ ] Handle `ws.onclose`: Auto-reconnect after 3 seconds
   - [ ] Use `setTimeout(initTerminalWebSocket, 3000)` for retry
   - [ ] Implement connection status indicator (if available)
   - [ ] Show "Connecting..." state
   - [ ] Show "Connected" when open
   - [ ] Show "Disconnected" on error/close

7. **Terminal Control Buttons**
   - [ ] #terminalClearBtn → `terminal.clear()`
   - [ ] #terminalCopyBtn → copy terminal content to clipboard
   - [ ] Use `navigator.clipboard.writeText(buffer)`
   - [ ] Handle copy errors gracefully
   - [ ] Get content: `terminal.getSelectionText()` or `terminal.toString()`

8. **Error Handling**
   - [ ] Catch and log all errors
   - [ ] Display user-friendly error messages
   - [ ] Graceful degradation if FitAddon unavailable
   - [ ] Prevent page unload on terminal errors
   - [ ] No unhandled promise rejections

9. **Theme Support**
   - [ ] Respect dark/light mode preference
   - [ ] Use CSS custom properties if available
   - [ ] Update terminal theme when user changes preference
   - [ ] Test with different themes

10. **Testing**
    - [ ] Test in browser at `http://127.0.0.1:3000`
    - [ ] Open Terminal tab
    - [ ] Type commands: `ls`, `pwd`, `echo "test"`
    - [ ] Verify output appears
    - [ ] Test Ctrl+C (interrupt)
    - [ ] Test resize (drag window edge)
    - [ ] Test Clear button
    - [ ] Test Copy button
    - [ ] Check browser console (no errors)
    - [ ] Test reconnect (close DevTools Network, wait 3s)

**Code Quality**:
- [ ] No console errors or warnings
- [ ] Proper error handling
- [ ] Comments for complex logic
- [ ] Consistent with existing code style
- [ ] No memory leaks (proper cleanup)

**Report To**: Knowledge Manager with tag "frontend-implementation"
**Blocking?**: Yes (primary deliverable)

---

### AGENT 3: Security Auditor
**Model**: Sonnet 4.5
**Type**: security-auditor
**Responsibility**: Security review of terminal integration

**TASK**: Review xterm.js terminal integration for security vulnerabilities

**Review Areas**:
1. **WebSocket Security**
   - [ ] Binary protocol is secure (no JSON injection risks)
   - [ ] Connection validation
   - [ ] Message size limits (64KB max from Rust)
   - [ ] No unvalidated data written to terminal

2. **Input Handling**
   - [ ] Terminal commands are not sanitized/filtered (expected behavior)
   - [ ] User control of input (no automatic command injection)
   - [ ] Special characters handled safely
   - [ ] Paste input doesn't execute automatically

3. **Output Handling**
   - [ ] Terminal output properly decoded (UTF-8)
   - [ ] No script injection risks
   - [ ] Binary data handled safely
   - [ ] Control sequences handled by xterm.js

4. **Connection Security**
   - [ ] Uses WSS for HTTPS pages
   - [ ] Uses WS for HTTP pages
   - [ ] No credential leakage in connection string
   - [ ] Connection state properly tracked

5. **Terminal Injection Protection**
   - [ ] No eval() or Function() in terminal code
   - [ ] No innerHTML usage (only term.write)
   - [ ] Proper escaping of special characters
   - [ ] xterm.js handles ANSI sequences safely

6. **Error Handling Security**
   - [ ] Error messages don't leak sensitive info
   - [ ] Errors don't crash page
   - [ ] Connection errors handled gracefully
   - [ ] No info disclosure in error logs

**Vulnerability Check**:
- [ ] No XSS vulnerabilities
- [ ] No command injection risks
- [ ] No privilege escalation paths
- [ ] No denial of service vectors
- [ ] Proper resource cleanup

**Report To**: Knowledge Manager with tag "security-review"
**Blocking?**: Yes (blocks merge if critical issues)

---

### AGENT 4: QA Engineer
**Model**: Sonnet 4.5
**Type**: test-automator
**Responsibility**: Testing and verification

**TASK**: Create comprehensive tests and verification checklist for terminal integration

**Test Categories**:

1. **Unit Tests**
   - Terminal initialization with correct options
   - FitAddon loading and fitting
   - WebSocket URL construction (http/https)
   - TextEncoder/TextDecoder usage
   - Error handling in each handler

2. **Integration Tests**
   - Terminal can connect to WebSocket
   - Input is properly encoded as UTF-8 bytes
   - Output is properly decoded from UTF-8 bytes
   - Real-time streaming without buffering
   - Auto-reconnect works after disconnect
   - Window resize triggers fit()

3. **Functional Tests**
   - Type command and see cursor move
   - Execute command and see output
   - Terminal Clear button works
   - Terminal Copy button works
   - Special keys work (Ctrl+C, Ctrl+D, arrows)
   - Paste works (Cmd+V on Mac, Ctrl+V on Linux/Windows)

4. **Browser Tests (Manual Checklist)**
   ```
   [ ] Open http://127.0.0.1:3000
   [ ] Click Terminal tab
   [ ] Terminal should render without errors
   [ ] Type 'ls' and press Enter
   [ ] Output should appear in terminal
   [ ] Type 'pwd' and press Enter
   [ ] Output should show current directory
   [ ] Press Ctrl+C
   [ ] Interrupt should work (no error)
   [ ] Resize browser window
   [ ] Terminal should resize with window
   [ ] Click Clear button
   [ ] Terminal should clear
   [ ] Type something, Click Copy button
   [ ] Text should copy to clipboard
   [ ] Close browser DevTools
   [ ] Open Console tab
   [ ] Should be NO errors or warnings
   [ ] Refresh page
   [ ] Terminal should reconnect
   [ ] Simulate disconnect:
     - Close tab's network connection
     - Wait 3 seconds
     - Should auto-reconnect
     - Terminal should be responsive
   ```

5. **Edge Cases**
   - Fast typing (rapid keystroke input)
   - Large output (100+ lines)
   - Binary data in output
   - Connection lost during typing
   - Multiple reconnects
   - Terminal tab hidden/shown
   - Page refresh during command

6. **Performance Tests**
   - Input latency < 100ms
   - Output appears immediately
   - No terminal lag
   - Memory usage stable
   - CPU usage low

**Test Deliverables**:
1. Unit test file (JavaScript/Jest or similar)
2. Integration test file
3. Manual testing checklist (markdown)
4. Test results report
5. Passing tests before completion

**Report To**: Knowledge Manager with tag "qa-testing"
**Blocking?**: Yes (tests must pass)

---

### AGENT 5: Documentation Lead
**Model**: Haiku 4.5
**Type**: documentation-expert
**Responsibility**: Documentation of terminal integration

**TASK**: Document the xterm.js terminal integration

**Documentation Deliverables**:

1. **API Reference**
   - `/api/terminal` WebSocket endpoint
   - Binary protocol specification
   - Message format (raw UTF-8 bytes)
   - Connection lifecycle

2. **Connection Flow Diagram**
   - User opens Terminal tab
   - Browser connects to /api/terminal
   - Terminal renders
   - User types → WebSocket send
   - Shell output → WebSocket receive
   - User closes → Connection closes

3. **Binary Protocol Specification**
   - Format: Raw UTF-8 bytes
   - Input: User keyboard input as bytes
   - Output: Shell output as bytes
   - No JSON encoding
   - Message size: Up to 64KB

4. **Error Handling Guide**
   - WebSocket connection errors
   - Terminal rendering errors
   - FitAddon availability
   - Auto-reconnect behavior
   - Debug tips

5. **Troubleshooting Guide**
   - Terminal doesn't appear
   - No command output
   - Slow input/output
   - Connection keeps dropping
   - Clear/Copy buttons don't work
   - Theme not applying
   - Debugging with browser DevTools

6. **Integration Example**
   - How terminal is initialized in dashboard.js
   - How to add new terminal features
   - How to customize appearance
   - How to monitor connection

7. **Performance Notes**
   - Binary protocol is efficient
   - FitAddon for responsive resizing
   - Auto-reconnect reduces downtime
   - UTF-8 encoding overhead minimal

**Documentation Format**:
- Markdown files in `/Users/brent/git/cc-orchestra/docs/`
- Code examples with syntax highlighting
- Diagrams in ASCII or SVG
- Troubleshooting sections
- FAQ section

**Report To**: Knowledge Manager with tag "documentation-complete"
**Blocking?**: No (can follow implementation)

---

## KNOWLEDGE MANAGER COORDINATION

### Store Results With:
```bash
node /Users/brent/git/cc-orchestra/src/knowledge-manager.js store \
  "Terminal Integration: [Agent Name] - [Status]" \
  --type "completion" \
  --agent "[agent-type]"
```

### Search for Architecture Decisions:
```bash
node /Users/brent/git/cc-orchestra/src/knowledge-manager.js search \
  "terminal architecture"
```

---

## TEAM SYNCHRONIZATION

**Parallel Execution**: All agents work in parallel
**Blocker Chain**:
1. Chief Architect → provides design
2. Frontend Developer → implements based on design
3. Security Auditor → reviews implementation
4. QA Engineer → tests implementation
5. Documentation Lead → documents final solution

**Daily Standup** (if needed):
- Status: What did you complete?
- Blockers: What's preventing progress?
- Questions: What needs clarification?

---

## SUCCESS DEFINITION

- [ ] Terminal renders at http://127.0.0.1:3000
- [ ] User can type and execute commands
- [ ] Output appears in real-time
- [ ] Clear and Copy buttons work
- [ ] Auto-reconnect works
- [ ] No console errors
- [ ] Security review passed
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Code merged and working

---

## RESOURCES

### Key Files
- **Target Implementation**: `/Users/brent/git/cc-orchestra/cco/static/dashboard.js` (lines 686-817)
- **HTML Elements**: `/Users/brent/git/cc-orchestra/cco/static/dashboard.html` (lines 11-13, 243)
- **Backend Handler**: `/Users/brent/git/cc-orchestra/cco/src/server.rs` (lines 940-1036)
- **Delegation Plan**: `/Users/brent/git/cc-orchestra/TERMINAL_INTEGRATION_DELEGATION.md`
- **Orchestrator Rules**: `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md`

### Libraries
- **xterm.js**: v5.3.0 from CDN
- **FitAddon**: v0.8.0 from CDN
- **JavaScript APIs**: TextEncoder, TextDecoder, WebSocket, File APIs

### References
- xterm.js Docs: https://xtermjs.org/
- WebSocket RFC: RFC 6455
- UTF-8 Encoding: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder

---

## NEXT STEPS

1. **Each agent**: Read this brief and referenced files
2. **Architect**: Design and store decisions
3. **Frontend Developer**: Implement based on design
4. **Security Auditor**: Review implementation
5. **QA Engineer**: Test and verify
6. **Documentation Lead**: Document solution
7. **All agents**: Report completion in Knowledge Manager
8. **Verify**: Terminal works at http://127.0.0.1:3000

---

**ORCHESTRATOR NOTE**: This brief establishes clear expectations for all agents. Follow it precisely and report any blockers immediately. The terminal integration is critical for the dashboard's usability.
