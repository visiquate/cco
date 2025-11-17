# xterm.js Terminal Integration - Agent Delegation (Issue #26)

## ORCHESTRATION STATUS: AWAITING AGENT EXECUTION

**Date**: 2025-11-16
**Issue**: GitHub Issue #26 - Frontend xterm.js integration
**Status**: Ready for agent delegation
**Priority**: High (depends on Rust backend completion)

---

## DELEGATION STRATEGY

This document outlines how the xterm.js terminal integration should be completed via agent delegation per ORCHESTRATOR_RULES.

### Required Agents (5 agents, spawned in parallel)

#### 1. Chief Architect (Opus 4.1)
- **Type**: `architect`
- **Model**: `opus-4-1`
- **Task**: Design complete terminal integration architecture
- **Scope**:
  - Terminal lifecycle management
  - Binary WebSocket protocol architecture
  - Error recovery and reconnection strategies
  - Theme management and CSS integration
  - Component interaction design

#### 2. Frontend Developer (Sonnet 4.5)
- **Type**: `frontend-expert`
- **Model**: `sonnet-4-5`
- **Task**: Implement xterm.js integration in dashboard.js
- **Scope**:
  - CRITICAL: Read `/Users/brent/git/cc-orchestra/cco/static/dashboard.js` FULLY (lines 686-817) without limits
  - Terminal initialization with xterm.js 5.3.0
  - FitAddon attachment for responsive resizing
  - Binary WebSocket protocol to `/api/terminal`
  - Keyboard input handling (special keys, paste)
  - Output streaming and real-time updates
  - Auto-reconnect logic (3-second retry)
  - Terminal control buttons (clear, copy)
  - Dark mode theme support
  - NO JSON encoding - pure binary UTF-8
  - Test in browser at `http://127.0.0.1:3000`

#### 3. Security Auditor (Sonnet 4.5)
- **Type**: `security-auditor`
- **Model**: `sonnet-4-5`
- **Task**: Review terminal integration security
- **Scope**:
  - WebSocket binary protocol validation
  - Input sanitization for terminal commands
  - Connection security and validation
  - Protection against terminal injection
  - Error handling security

#### 4. QA Engineer (Sonnet 4.5)
- **Type**: `test-automator`
- **Model**: `sonnet-4-5`
- **Task**: Create tests and verification checklist
- **Scope**:
  - Unit tests for terminal functions
  - Integration tests for WebSocket protocol
  - Manual testing checklist for browser
  - Test cases:
    - Terminal renders correctly
    - Keyboard input works
    - Commands execute and output appears
    - Binary protocol is working
    - Auto-reconnect functions
    - Buttons work (clear, copy)
    - No console errors

#### 5. Documentation Lead (Haiku 4.5)
- **Type**: `documentation-expert`
- **Model**: `haiku-4-5`
- **Task**: Document terminal integration
- **Scope**:
  - API reference for terminal WebSocket
  - Connection flow diagram
  - Binary protocol specification
  - Error handling documentation
  - Troubleshooting guide

---

## Context for Agents

### Files to Read
1. **Dashboard JavaScript** (CRITICAL - READ FULLY)
   - Path: `/Users/brent/git/cc-orchestra/cco/static/dashboard.js`
   - Lines: 686-817 (current terminal code)
   - Size: ~131 lines of incomplete code
   - **INSTRUCTION**: Read without limits or offsets - each agent has dedicated context

2. **Dashboard HTML**
   - Path: `/Users/brent/git/cc-orchestra/cco/static/dashboard.html`
   - xterm.js libraries loaded (lines 11-13)
   - Terminal div element (line 243, id="terminal")
   - Control buttons: `#terminalClearBtn`, `#terminalCopyBtn`

3. **Rust Backend WebSocket Handler**
   - Path: `/Users/brent/git/cc-orchestra/cco/src/server.rs`
   - Lines: 940-1036 (terminal WebSocket handler)
   - Endpoint: `/api/terminal`
   - Protocol: Binary (not JSON)

4. **Orchestrator Rules**
   - Path: `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md`
   - **MANDATORY**: Each agent reads this first

### Key Requirements

#### Terminal Initialization
```javascript
// Initialize Terminal instance
new Terminal({
    rows: 25,
    cols: 120,
    theme: {
        background: '#1e293b',
        foreground: '#f1f5f9',
        cursor: '#3b82f6',
    },
    fontSize: 14,
    fontFamily: 'Monaco, Menlo, Ubuntu Mono, monospace',
    cursorBlink: true,
    historyLength: 1000
});

// Apply FitAddon for responsive sizing
state.fitAddon = new FitAddon();
state.terminal.loadAddon(state.fitAddon);
state.fitAddon.fit();
```

#### Binary WebSocket Protocol
```javascript
// Connection
const wsUrl = `${protocol}//${window.location.host}/api/terminal`;
state.ws = new WebSocket(wsUrl);
state.ws.binaryType = 'arraybuffer';  // CRITICAL: Binary mode

// Input: Send UTF-8 bytes
state.terminal.onData(data => {
    if (state.ws && state.ws.readyState === WebSocket.OPEN) {
        state.ws.send(new TextEncoder().encode(data));
    }
});

// Output: Receive UTF-8 bytes
state.ws.onmessage = event => {
    if (event.data instanceof ArrayBuffer) {
        const text = new TextDecoder().decode(new Uint8Array(event.data));
        state.terminal.write(text);
    }
};
```

#### Connection Management
```javascript
// Auto-reconnect on close
state.ws.onclose = () => {
    console.log('Terminal WebSocket closed');
    setTimeout(() => initTerminalWebSocket(), 3000);
};

// Error handling
state.ws.onerror = error => {
    console.error('Terminal WebSocket error:', error);
};
```

#### Terminal Controls
```javascript
// Clear button
document.getElementById('terminalClearBtn')?.addEventListener('click', () => {
    if (state.terminal) {
        state.terminal.clear();
    }
});

// Copy button
document.getElementById('terminalCopyBtn')?.addEventListener('click', () => {
    if (state.terminal) {
        const text = state.terminal.getSelectionText() || state.terminal.toString();
        navigator.clipboard.writeText(text);
    }
});
```

#### Window Resize
```javascript
window.addEventListener('resize', () => {
    if (state.terminal && state.currentTab === 'terminal' && state.fitAddon) {
        try {
            state.fitAddon.fit();
        } catch (error) {
            console.warn('Error fitting terminal:', error);
        }
    }
});
```

---

## Success Criteria

All of the following must be verified:

- [ ] Terminal renders in browser at `http://127.0.0.1:3000`
- [ ] User can type commands and see cursor moving
- [ ] Commands execute and output appears in real-time
- [ ] WebSocket binary protocol works (not JSON)
- [ ] Terminal resizes with window
- [ ] Clear button works (clears terminal)
- [ ] Copy button works (copies to clipboard)
- [ ] WebSocket auto-reconnects after disconnect
- [ ] No console errors or warnings
- [ ] Dark theme applies correctly
- [ ] Works after Rust backend compilation

---

## Agent Coordination Protocol

### Before Work
Each agent MUST:
1. Read `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md`
2. Read all context files FULLY without limits
3. Check knowledge manager for architect decisions

### During Work
Each agent MUST:
1. Store progress in knowledge manager
2. Coordinate with other agents via shared memory
3. Report blockers immediately

### After Work
Each agent MUST:
1. Run full test suite (QA Engineer)
2. Report completion with results
3. Store final decisions in knowledge manager

---

## Testing Plan

### Manual Testing Checklist (QA Engineer)
```
[ ] Open http://127.0.0.1:3000 in browser
[ ] Click Terminal tab - terminal should render
[ ] Type 'ls' command - see cursor move
[ ] Press Enter - command executes
[ ] See output appear in terminal
[ ] Type 'pwd' - verify output
[ ] Press Ctrl+C - interrupt works
[ ] Resize browser window - terminal resizes
[ ] Click Clear button - terminal clears
[ ] Click Copy button - text copies to clipboard
[ ] Close browser dev tools - check for console errors
[ ] Refresh page - terminal reconnects
[ ] Disconnect WebSocket - auto-reconnects in 3s
```

### Automated Tests (QA Engineer)
```
- Terminal initialization tests
- WebSocket connection tests
- Binary protocol tests
- Input/output streaming tests
- Auto-reconnect tests
- Button functionality tests
- Theme switching tests
```

---

## Delivery Checklist

- [ ] **Chief Architect**: Architecture design complete
- [ ] **Frontend Developer**: Code implementation complete and tested
- [ ] **Security Auditor**: Security review complete
- [ ] **QA Engineer**: Tests written and passing
- [ ] **Documentation Lead**: Documentation complete
- [ ] **All agents**: Report stored in knowledge manager
- [ ] **Terminal**: Fully functional in browser
- [ ] **GitHub Issue #26**: Ready to mark as complete

---

## Notes

- **Rust Backend Status**: Awaiting completion before full testing
- **xterm.js Version**: 5.3.0 (from CDN)
- **Protocol**: Binary UTF-8, no JSON
- **Endpoint**: `/api/terminal` (already implemented)
- **Backward Compatibility**: Maintains existing code structure
- **Performance**: Streaming binary protocol for low-latency

---

## References

- **xterm.js Documentation**: https://xtermjs.org/
- **FitAddon**: Responsive terminal sizing addon
- **WebSocket Binary Protocol**: RFC 6455 (WebSocket Protocol)
- **UTF-8 Encoding**: JavaScript TextEncoder/TextDecoder APIs
- **GitHub Issue**: #26 - Frontend xterm.js integration

---

**ORCHESTRATOR NOTE**: This document establishes the delegation contract for the xterm.js terminal integration. Agents should use this as their specification and coordinate through shared knowledge manager. The Frontend Developer will be primary implementer with oversight from Architect and Security Auditor.
