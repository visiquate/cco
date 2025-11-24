# Documentation Cleanup Checklist

## Architecture Pivot: Terminal → API Cost Monitoring Daemon

This document tracks the cleanup of all terminal-related documentation following the architecture pivot to focus CCO as a pure API cost monitoring daemon.

## Files Cleaned ✅

### Primary Documentation
- [x] README.md - Complete rewrite for cost monitoring daemon
- [ ] BUILDING.md - Update build instructions (remove terminal dependencies)
- [ ] USAGE.md - Update usage guide (remove terminal commands)
- [ ] TROUBLESHOOTING.md - Update troubleshooting (remove terminal issues)
- [ ] COST_SAVINGS.md - Review and update

### Files to DELETE (Terminal-Specific)

#### Root Level Documentation
- [ ] PTY_TERMINAL_ARCHITECTURE.md
- [ ] WASM_TERMINAL_ARCHITECTURE.md
- [ ] WASM_TERMINAL_PHASE1_PLAN.md
- [ ] WASM_TERMINAL_PHASE1_PROGRESS.md
- [ ] WASM_TERMINAL_PHASE1_FINAL_REPORT.md
- [ ] PHASE1_WASM_TERMINAL_REPORT.md
- [ ] WASM_TERMINAL_COMPARISON.md
- [ ] WASM_TERMINAL_RISK_MITIGATION.md
- [ ] WASM_ENHANCED_PROTOCOL_SPEC.md
- [ ] WASM_SERVER_PROTOCOL_ANALYSIS.md
- [ ] WASM_BACKEND_DELIVERABLES.md
- [ ] WASM_PERFORMANCE_SPECIFICATION.md
- [ ] TERMINAL_ARCHITECTURE_ALTERNATIVES.md
- [ ] TERMINAL_ARCHITECTURE_ALTERNATIVES_ANALYSIS.md
- [ ] TERMINAL_ARCHITECTURE_EXECUTIVE_SUMMARY.md
- [ ] TERMINAL_ALTERNATIVES_RESEARCH.md
- [ ] TERMINAL_ALTERNATIVES_QUICK_REFERENCE.md
- [ ] TERMINAL_FIX_STRATEGY.md
- [ ] TERMINAL_TEST_STRATEGY.md
- [ ] TERMINAL_TEST_QUICKSTART.md
- [ ] TERMINAL_TEST_REPORT.md
- [ ] TERMINAL_TEST_RESULTS_INDEX.md
- [ ] TERMINAL_E2E_COMPREHENSIVE_REPORT.md
- [ ] TERMINAL_KEYBOARD_E2E_TEST_REPORT.md
- [ ] TERMINAL_SEGFAULT_FIX.md
- [ ] TERMINAL_RESEARCH_SUMMARY.md

#### Test Reports (Terminal-Specific E2E)
- [ ] e2e-test-report-*.md (all 10 terminal test reports)

#### WASM Terminal Directory
- [ ] wasm-terminal/ (entire directory - WASM implementation)

#### Static Files (Terminal UI)
- [ ] static/wasm-terminal.js
- [ ] test-wasm-terminal.html

#### Test Files (Terminal-Specific)
- [ ] tests/e2e_terminal.spec.js
- [ ] tests/wasm_terminal_integration.spec.js
- [ ] tests/terminal_cdn_check.spec.js
- [ ] tests/terminal_fast.rs
- [ ] tests/terminal_integration.rs

#### Performance/Ready Signal (Terminal-Related)
- [ ] READY_SIGNAL_*.md (all ready signal docs - terminal-specific)
- [ ] PLAYWRIGHT_READY_SIGNAL_GUIDE.md
- [ ] test-ready-signal.js
- [ ] PERFORMANCE_BASELINE_REPORT.md (if terminal-focused)
- [ ] PERFORMANCE_DELIVERABLES_INDEX.md (if terminal-focused)
- [ ] PERFORMANCE_VALIDATION_COMPLETE.md (if terminal-focused)
- [ ] PERFORMANCE_OPTIMIZATION_STRATEGY.md (if terminal-focused)

### Files to UPDATE (Remove Terminal References)

#### Documentation
- [ ] docs/WEB_UI_GUIDE.md - Remove "Terminal" tab section
- [ ] docs/ANALYTICS_PIPELINE_ARCHITECTURE.md - Review for terminal refs
- [ ] docs/CLAUDE_CODE_INTEGRATION.md - Review for terminal refs
- [ ] INTEGRATION.md - Remove terminal integration
- [ ] TESTING_WEB_UI.md - Remove terminal testing
- [ ] QUICK_START.md - Remove terminal quick start
- [ ] INDEX.md - Update index (remove terminal entries)

#### Test Documentation
- [ ] tests/README.md - Remove terminal test references
- [ ] COMPREHENSIVE_TEST_PLAN.md - Remove terminal tests
- [ ] TEST_DELIVERABLES_INDEX.md - Remove terminal deliverables
- [ ] TEST_DELIVERABLES_SUMMARY.md - Remove terminal summary
- [ ] E2E_TEST_EXECUTIVE_SUMMARY.md - Remove terminal E2E tests

#### Implementation Docs
- [ ] IMPLEMENTATION_NOTES.md - Remove terminal implementation notes
- [ ] IMPLEMENTATION_CHECKLIST.md - Remove terminal checklist items
- [ ] IMPLEMENTATION_INDEX.md - Update index
- [ ] FINAL_BUILD_AND_TEST_REPORT.md - Remove terminal build/test
- [ ] FINAL_REPORT_INDEX.md - Update index
- [ ] DELIVERABLES_SUMMARY.md - Remove terminal deliverables
- [ ] DEPLOYMENT_READY_CONFIRMATION.md - Review and update

#### Build/DevOps
- [ ] BUILD_PROCESS.md - Remove terminal build steps
- [ ] BUILD_SUMMARY.md - Remove terminal build summary
- [ ] BUILD_QUICK_START.md - Remove terminal quick start
- [ ] DEVOPS_BUILD_SPECIFICATION.md - Remove terminal deployment

### Source Code Comments
- [ ] src/server.rs - Remove terminal handler comments
- [ ] src/terminal.rs - Mark as deprecated or remove
- [ ] src/main.rs - Remove terminal CLI commands
- [ ] static/dashboard.js - Remove terminal tab JavaScript
- [ ] static/dashboard.html - Remove terminal tab HTML

### Configuration
- [ ] Cargo.toml - Review terminal dependencies (portable-pty, etc.)
- [ ] package.json - Review terminal npm dependencies (xterm.js)

## New Documentation to CREATE

### Cost Monitoring Focus
- [ ] docs/COST_MONITORING_ARCHITECTURE.md - Core cost tracking design
- [ ] docs/TUI_DASHBOARD_GUIDE.md - Text-based UI for non-browser environments
- [ ] docs/ANALYTICS_API_REFERENCE.md - Analytics API endpoints
- [ ] docs/COST_PREDICTION.md - Cost prediction and budgeting

### Configuration
- [ ] docs/CONFIGURATION_REFERENCE.md - Complete config file reference
- [ ] docs/MODEL_ROUTING_GUIDE.md - Model routing configuration
- [ ] docs/CACHE_STRATEGY_GUIDE.md - Caching strategies and tuning

### Cross-Platform
- [ ] docs/WINDOWS_SETUP.md - Windows-specific setup
- [ ] docs/MACOS_SETUP.md - macOS-specific setup
- [ ] docs/LINUX_SETUP.md - Linux-specific setup

## GitHub Updates

- [ ] Repository description: Update to "API cost monitoring daemon for Claude Code"
- [ ] Topics/tags: Add "cost-monitoring", "api-proxy", "analytics"
- [ ] Remove topics: "terminal", "wasm", "xterm"
- [ ] Update issue templates: Remove terminal-related templates
- [ ] Update PR template: Focus on cost monitoring features

## Verification Checklist

- [ ] No references to "terminal" in primary docs (README, USAGE, etc.)
- [ ] No references to "xterm", "wasm", "pty", "canvas", "keyboard"
- [ ] All docs focus on current/forward-looking features only
- [ ] No historical "journey" or "why we did X" discussions
- [ ] Clear beginner-friendly documentation
- [ ] Roadmap is future-focused only

## Summary Statistics

- **Total Files to Delete**: ~80+ files
- **Total Files to Update**: ~30+ files
- **Total Files to Create**: ~10+ files
- **Estimated Cleanup Time**: 2-3 hours
- **Documentation Size Reduction**: ~60-70% (terminal docs removed)

## Next Steps After Cleanup

1. Review all remaining documentation for accuracy
2. Test all documentation examples
3. Update GitHub repository metadata
4. Archive deleted files (optional: create archive branch)
5. Announce architecture pivot in changelog
6. Update contribution guidelines

---

**Status**: In Progress
**Started**: 2025-11-17
**Completed**: TBD
