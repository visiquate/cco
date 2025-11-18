# CCO Project Pivot - Coordination Document

## Executive Summary

The CCO project is **immediately pivoting** from a WASM-based terminal web interface to a focused **API Cost Monitoring Daemon**. This document coordinates the transition and assigns responsibilities to specialist agents.

## Critical Changes

### What We're REMOVING (Phase 1 - Immediate)

```
DELETE THESE:
├── wasm-terminal/          # Entire directory
├── static/
│   ├── terminal-adapter.js
│   ├── test-wasm-terminal*.html
│   └── wasm-terminal.js
├── src/terminal.rs         # Terminal module
├── tests/
│   ├── terminal_*.rs       # All terminal tests
│   └── e2e_terminal.spec.js
└── build-wasm.sh          # WASM build script
```

### What We're BUILDING (Phase 2-4)

```
NEW STRUCTURE:
├── src/
│   ├── daemon/
│   │   ├── mod.rs         # Daemon core
│   │   ├── monitor.rs     # Cost monitoring engine
│   │   ├── parser.rs      # Log parser
│   │   └── metrics.rs     # Metrics aggregation
│   ├── tui/
│   │   ├── mod.rs         # TUI framework
│   │   ├── dashboard.rs   # Main dashboard
│   │   ├── widgets.rs     # Custom widgets
│   │   └── themes.rs      # Color themes
│   ├── storage/
│   │   ├── mod.rs         # Storage abstraction
│   │   ├── sqlite.rs      # SQLite backend
│   │   └── schema.rs      # Database schema
│   └── config.rs          # Configuration management
```

## Agent Task Assignments

### Phase 1: Terminal Removal (Day 1)

#### **Rust Backend Developer**
**Priority: URGENT**
**Complexity: Medium**

```
TASK: Remove all WASM terminal code and update build system

RESPONSIBILITIES:
1. Delete wasm-terminal/ directory
2. Remove terminal.rs from src/
3. Clean up terminal references in:
   - lib.rs
   - server.rs
   - main.rs
4. Remove terminal tests
5. Update Cargo.toml dependencies
6. Update build scripts
7. Ensure project still compiles

DELIVERABLES:
- Clean codebase without terminal code
- Updated Cargo.toml
- Passing compilation
```

#### **DevOps Engineer**
**Priority: HIGH**
**Complexity: Low**

```
TASK: Update build pipeline and remove WASM artifacts

RESPONSIBILITIES:
1. Remove WASM build steps from CI/CD
2. Clean up GitHub Actions workflows
3. Remove wasm32-unknown-unknown target
4. Update release artifacts list

DELIVERABLES:
- Updated GitHub Actions
- Clean build pipeline
```

### Phase 2: Core Daemon Implementation (Days 2-3)

#### **Rust Backend Developer**
**Priority: CRITICAL**
**Complexity: High**

```
TASK: Implement core API cost monitoring daemon

RESPONSIBILITIES:
1. Create daemon architecture:
   - Background service runner
   - Cross-platform compatibility
   - Signal handling (SIGTERM, SIGINT)

2. Implement log monitoring:
   - File watcher for CCO logs
   - JSON log parser
   - Event stream processing

3. Cost calculation engine:
   - Model tier detection
   - Token counting
   - Pricing calculation
   - Cache token handling

4. Metrics aggregation:
   - Session tracking
   - Per-model metrics
   - Rolling history (25 calls)

DELIVERABLES:
- Working daemon process
- Log monitoring system
- Cost calculation engine
- In-memory metrics store
```

#### **Database Engineer**
**Priority: HIGH**
**Complexity: Medium**

```
TASK: Implement SQLite persistence layer

RESPONSIBILITIES:
1. Design database schema
2. Implement SQLite integration with sqlx
3. Create migration system
4. Build data access layer:
   - Insert API calls
   - Query aggregates
   - Clean old data

5. Performance optimization:
   - Batch inserts
   - Proper indexing
   - Query optimization

DELIVERABLES:
- SQLite schema
- Data access module
- Migration scripts
- Performance benchmarks
```

### Phase 3: TUI Dashboard (Days 4-5)

#### **TUI Frontend Developer**
**Priority: CRITICAL**
**Complexity: High**

```
TASK: Build ratatui-based terminal dashboard

RESPONSIBILITIES:
1. Implement TUI framework:
   - Layout system
   - Event handling
   - Refresh loop

2. Create dashboard widgets:
   - Header with status
   - Live monitor panel
   - Cost summary panel
   - Model breakdown table
   - Recent calls table
   - Footer with shortcuts

3. Interactive features:
   - Keyboard navigation
   - Scrollable tables
   - Export dialog
   - Settings view

4. Visual polish:
   - Color themes
   - Animations
   - Progress indicators
   - Status icons

DELIVERABLES:
- Complete TUI dashboard
- Dark/light themes
- Export functionality
- Responsive layout
```

#### **UX Designer**
**Priority: MEDIUM**
**Complexity: Low**

```
TASK: Review and refine TUI user experience

RESPONSIBILITIES:
1. Review TUI layout
2. Optimize information hierarchy
3. Improve keyboard shortcuts
4. Enhance visual feedback
5. Test accessibility

DELIVERABLES:
- UX review report
- Improvement recommendations
- Accessibility checklist
```

### Phase 4: Integration & Testing (Days 6-7)

#### **QA Engineer**
**Priority: HIGH**
**Complexity: Medium**

```
TASK: Comprehensive testing of daemon and TUI

RESPONSIBILITIES:
1. Unit tests:
   - Cost calculations
   - Log parsing
   - Metrics aggregation

2. Integration tests:
   - Database operations
   - Log monitoring
   - TUI components

3. Cross-platform testing:
   - macOS (x86_64, aarch64)
   - Windows (x86_64)
   - Different terminal emulators

4. Performance testing:
   - Memory usage
   - CPU usage
   - Startup time

DELIVERABLES:
- Test suite
- Test results report
- Performance benchmarks
- Bug reports
```

#### **Security Auditor**
**Priority: MEDIUM**
**Complexity: Low**

```
TASK: Security review of daemon

RESPONSIBILITIES:
1. Review API key handling
2. Audit file permissions
3. Check database security
4. Review network access
5. Validate input sanitization

DELIVERABLES:
- Security audit report
- Recommendations
- Threat model
```

### Documentation & Communication

#### **Technical Writer**
**Priority: HIGH**
**Complexity: Medium**

```
TASK: Update all documentation for new architecture

RESPONSIBILITIES:
1. Update README.md
2. Remove terminal references
3. Create installation guide
4. Write user manual
5. Document configuration
6. Create troubleshooting guide

DELIVERABLES:
- Updated README
- Installation guide
- User documentation
- API reference
```

#### **DevOps Engineer**
**Priority: MEDIUM**
**Complexity: Medium**

```
TASK: Create deployment and distribution system

RESPONSIBILITIES:
1. Build release pipeline
2. Create installers:
   - macOS (Homebrew formula)
   - Windows (MSI installer)
3. Setup auto-update mechanism
4. Create installation scripts

DELIVERABLES:
- Release pipeline
- Installation packages
- Update system
- Distribution strategy
```

## Timeline & Milestones

```
Day 1 (Nov 17):
├── Morning:    Terminal code removal
├── Afternoon:  Build system cleanup
└── Evening:    Verification & commits

Day 2-3 (Nov 18-19):
├── Daemon core implementation
├── Log monitoring system
├── SQLite persistence
└── Cost calculation engine

Day 4-5 (Nov 20-21):
├── TUI dashboard development
├── Interactive features
├── Visual polish
└── Export functionality

Day 6-7 (Nov 22-23):
├── Integration testing
├── Cross-platform testing
├── Documentation updates
├── Release preparation
└── LAUNCH v2025.11.1
```

## Critical Success Factors

1. **Clean Removal**: All terminal code completely removed
2. **Lightweight Daemon**: < 50MB RAM, < 1% CPU
3. **Accurate Costs**: Precise calculation with cache tokens
4. **Responsive TUI**: 60 FPS, smooth updates
5. **Cross-Platform**: Works on macOS and Windows
6. **Easy Installation**: Single binary, no dependencies

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Log format changes | High | Flexible parser, version detection |
| TUI compatibility | Medium | Test multiple terminals, fallback mode |
| Performance issues | Medium | Profiling, optimization, caching |
| SQLite corruption | Low | WAL mode, backups, recovery |
| Update failures | Low | Rollback mechanism, version pinning |

## Communication Protocol

### Daily Standups
```
Each agent reports:
1. Yesterday's progress
2. Today's goals
3. Blockers
4. Help needed
```

### Coordination Points
```
Critical handoffs:
- Backend → TUI: Metrics API
- Backend → Database: Schema
- TUI → DevOps: Binary requirements
- QA → All: Bug reports
```

### Knowledge Sharing
```bash
# Store progress in Knowledge Manager
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Progress: [component] - [status]" \
  --type status --agent [agent-name]
```

## Definition of Done

### Phase 1 ✓
- [ ] All terminal code removed
- [ ] Project compiles without errors
- [ ] Build pipeline updated
- [ ] No WASM references remain

### Phase 2 ✓
- [ ] Daemon runs in background
- [ ] Monitors CCO logs successfully
- [ ] Calculates costs accurately
- [ ] Persists to SQLite

### Phase 3 ✓
- [ ] TUI displays all metrics
- [ ] Keyboard navigation works
- [ ] Export feature functional
- [ ] Themes implemented

### Phase 4 ✓
- [ ] All tests pass
- [ ] Documentation complete
- [ ] Installers created
- [ ] Released to GitHub

## GitHub Updates Required

### Issues to Close
- All terminal-related issues
- WASM performance issues
- Browser compatibility issues

### Issues to Create
1. "API Cost Monitoring Daemon"
2. "TUI Dashboard Implementation"
3. "Cross-platform Deployment"
4. "SQLite Metrics Storage"

### README Updates
- Remove terminal features
- Add daemon description
- Update installation instructions
- Add TUI screenshots

## Questions for User

1. **API Key Management**: Should we support multiple API keys?
2. **Cloud Sync**: Future feature for team cost tracking?
3. **Alert System**: Email/Slack notifications for limits?
4. **Export Formats**: CSV sufficient or need Excel/JSON?
5. **History Retention**: 90 days enough or need more?

## Start Signal

**All agents should begin Phase 1 immediately upon receiving this coordination document.**

Priority order:
1. Rust Backend Developer - Remove terminal code
2. DevOps Engineer - Update build system
3. All others - Prepare for Phase 2

---

*Coordination Version: 1.0*
*Date: November 17, 2024*
*Status: EXECUTE IMMEDIATELY*