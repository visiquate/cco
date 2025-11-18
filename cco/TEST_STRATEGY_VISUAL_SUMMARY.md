# Test Strategy Visual Summary

**Quick visual reference for the comprehensive testing framework**

---

## The Problem (In One Picture)

```
CURRENT STATE (Single-Mode Testing)
┌────────────────────────────────────────────────────────────┐
│                   SMOKE TEST                                │
│                                                              │
│  ✓ Mode 1: cco run --port 3000      TESTED                │
│  ✗ Mode 2: cco (no args)            NOT TESTED             │
│                                       ↓                     │
│                                   FAILURE                   │
│                                   (Undetected)              │
│                                                              │
└────────────────────────────────────────────────────────────┘

RESULT: Mode-specific failures go undetected until production!
```

---

## The Solution (In One Picture)

```
PROPOSED STATE (Dual-Mode Testing)
┌────────────────────────────────────────────────────────────┐
│             COMPREHENSIVE SMOKE TEST                        │
│                                                              │
│  ✓ Mode 1: cco run --port 3000      TESTED                │
│  ✓ Mode 2: cco (no args)            TESTED (NEW!)          │
│                                                              │
│  Integration Tests:                                         │
│  ├─ mode1_server_basic.rs           TESTED                 │
│  └─ mode2_tui_daemon.rs             TESTED (NEW!)          │
│                                                              │
│  E2E Tests:                                                 │
│  ├─ mode1_comprehensive.sh          TESTED                 │
│  └─ mode2_comprehensive.sh          TESTED (NEW!)          │
│                                                              │
└────────────────────────────────────────────────────────────┘

RESULT: Both modes validated at every commit - no blind spots!
```

---

## Test Pyramid (Mode-Aware)

```
                          ▲
                         ╱│╲
                        ╱ │ ╲       ACCEPTANCE TESTS
                       ╱  │  ╲     (5min stability,
                      ╱   │   ╲     load testing,
                     ╱────┼────╲    perf benchmarks)
                    ╱    M1│M2  ╲
                   ╱      ╱│╲    ╲
                  ╱      ╱ │ ╲    ╲   E2E TESTS
                 ╱      ╱  │  ╲    ╲ (Real workflows)
                ╱──────╱───┼───╲────╲
               ╱      ╱   M1│M2  ╲    ╲
              ╱      ╱    ╱ │ ╲   ╲    ╲
             ╱      ╱    ╱  │  ╲   ╲    ╲
            ╱──────╱────╱───┼───╲───╲────╲
           ╱      ╱    ╱   M1│M2  ╲   ╲    ╲
          ╱      ╱    ╱    ╱ │ ╲   ╲   ╲    ╲
         ╱______╱____╱____╱__┼__╲___╲___╲____╲   INTEGRATION
        ╱                   M1 │ M2          ╲  TESTS
       ╱___________________________________────╲ (mode1_ & mode2_)
      ╱                   UNIT TESTS           ╲
     ╱                   (mode1 & mode2)        ╲
    ╱________________________________________────╲

Coverage: 70% unit, 20% integration, 10% E2E
Both modes tested at EVERY level
```

---

## Execution Modes Comparison

```
┌──────────────────────────────────────────────────────────────────┐
│ EXECUTION MODE COMPARISON                                        │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│ MODE 1: Explicit Server                                         │
│ ═════════════════════════════════════════════════════════════    │
│ Command:   cco run --debug --port 3000                           │
│ Startup:   Direct HTTP server launch                             │
│ Process:   Single process (server)                               │
│ Port:      Specified (3000, 3001, etc)                           │
│ TUI:       None (headless server mode)                           │
│ Test:      ✓ Current smoke test validates this                   │
│                                                                  │
│ Startup sequence:                                                │
│   Parse args → Create server state → Bind port → Listen          │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│ MODE 2: Default TUI/Daemon (CRITICAL)                           │
│ ═════════════════════════════════════════════════════════════    │
│ Command:   cco (no arguments)                                    │
│ Startup:   TUI + daemon (two processes, one binary)              │
│ Process:   Two (TUI process + daemon process)                    │
│ Port:      Default 3000 (hardcoded)                              │
│ TUI:       Terminal UI showing dashboard                         │
│ Test:      ✗ NOT TESTED (causes failures!)                       │
│                                                                  │
│ Startup sequence (CRITICAL):                                     │
│   TUI init → Spawn daemon → Daemon binds port → TUI connects     │
│             ^ Must happen BEFORE TUI tries to connect            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

KEY DIFFERENCE:
  Mode 1: Single process, no TUI, custom port
  Mode 2: Two processes, TUI needed, port 3000 hardcoded, TIMING CRITICAL
```

---

## Critical Test Coverage Matrix

```
┌─────────────────────┬──────────────┬──────────────┬─────────────┐
│ Test Category       │ Mode 1       │ Mode 2       │ Both Modes  │
│                     │ (run --port) │ (no args)    │ Together    │
├─────────────────────┼──────────────┼──────────────┼─────────────┤
│ Startup             │ ✓ Must Test  │ ✓ Must Test  │ ✓ Validate  │
│ Port Binding        │ ✓ Tested     │ ✓ Tested     │ ✓ Smoke     │
│ Health Endpoint     │ ✓ Tested     │ ✓ Tested     │ ✓ Smoke     │
│ API Endpoints       │ ✓ Tested     │ ✓ Tested     │ ✓ Smoke     │
│ TUI Startup         │ ✗ N/A        │ ✓ Must Test  │ -           │
│ Daemon/TUI Comm     │ ✗ N/A        │ ✓ Must Test  │ -           │
│ Graceful Shutdown   │ ✓ Tested     │ ✓ Tested     │ ✓ Smoke     │
│ Port Release        │ ✓ Tested     │ ✓ Tested     │ ✓ Smoke     │
│ No Zombie Processes │ ✓ Tested     │ ✓ Tested     │ ✓ E2E       │
└─────────────────────┴──────────────┴──────────────┴─────────────┘

LEGEND:
✓ = Test this
✗ = Not applicable
```

---

## Implementation Timeline

```
WEEK 1: Stop the Bleeding (Smoke Test)
┌──────────────────────────────────────────────────┐
│ smoke_test_comprehensive.sh                      │
│ ├─ Mode 1: cco run --port 3000                   │
│ ├─ Mode 2: cco (no args)                         │
│ └─ Both modes must pass                          │
│                                                  │
│ Effort: 1-2 hours                                │
│ Impact: Catch all mode-specific failures         │
└──────────────────────────────────────────────────┘

WEEK 2: Integration Coverage
┌──────────────────────────────────────────────────┐
│ Integration Tests                                │
│ ├─ mode1_server_basic.rs                         │
│ │  ├─ Startup on custom ports                    │
│ │  ├─ API endpoints                              │
│ │  └─ Graceful shutdown                          │
│ │                                                 │
│ ├─ mode2_tui_daemon.rs (CRITICAL)                │
│ │  ├─ Daemon binds BEFORE TUI connects           │
│ │  ├─ TUI successfully connects                   │
│ │  ├─ Both shutdown cleanly                       │
│ │  └─ Port released                              │
│ │                                                 │
│ Effort: 3-4 hours per module                     │
│ Impact: Test component interactions              │
└──────────────────────────────────────────────────┘

WEEK 3: E2E & Performance
┌──────────────────────────────────────────────────┐
│ E2E Workflow Tests                               │
│ ├─ mode1_comprehensive.sh                        │
│ ├─ mode2_comprehensive.sh                        │
│ │                                                 │
│ Acceptance Tests                                 │
│ ├─ 5-minute stability                            │
│ ├─ Load testing (100+ concurrent)                │
│ └─ Resource monitoring                           │
│                                                  │
│ Effort: 2-3 hours per module                     │
│ Impact: Real-world workflow validation           │
└──────────────────────────────────────────────────┘

TOTAL EFFORT: 2-3 weeks for complete coverage
```

---

## Must-Pass Tests (Non-Negotiable)

```
STARTUP TESTS
├─ [✓] Mode 1 server starts successfully
├─ [✓] Mode 2 TUI/daemon starts successfully
├─ [✓] Daemon binds port BEFORE TUI connects
└─ [✓] TUI connects to daemon within 5s

SHUTDOWN TESTS
├─ [✓] Ctrl+C initiates graceful shutdown
├─ [✓] Shutdown completes within 2 seconds
├─ [✓] Port immediately available for reuse
└─ [✓] No zombie processes remain

API TESTS
├─ [✓] GET /health returns 200 OK + valid JSON
├─ [✓] GET /api/agents returns agent list
├─ [✓] Both modes have accessible endpoints
└─ [✓] No 404 or 500 errors during normal operation

MODE SEPARATION
├─ [✓] Mode 1 tests are labeled "mode1_*"
├─ [✓] Mode 2 tests are labeled "mode2_*"
├─ [✓] CI/CD runs separate test jobs
└─ [✓] Smoke test validates BOTH modes together

If ANY of these fail: BLOCK PR FROM MERGING
```

---

## CI/CD Pipeline Flow

```
Developer commits code
        ↓
┌───────────────────────────────────────┐
│  GitHub Actions Triggers              │
└───────────────────────────────────────┘
        ↓
  ┌─────────────────────────────────┐
  │ BUILD: cargo build --release    │
  └─────────────────────────────────┘
        ↓ (if build fails → block)

  Parallel execution:
  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
  │ TEST MODE 1      │  │ TEST MODE 2      │  │ UNIT TESTS       │
  │ cargo test       │  │ cargo test       │  │ cargo test --lib │
  │ --test           │  │ --test           │  │                  │
  │ '*mode1*'        │  │ '*mode2*'        │  │                  │
  └──────────────────┘  └──────────────────┘  └──────────────────┘
         ↓ (both must pass)
  ┌──────────────────────────────────┐
  │ SMOKE TEST: Both Modes           │
  │ ./smoke_test_comprehensive.sh    │
  └──────────────────────────────────┘
         ↓ (if any fails → block PR)

  ✓ All tests passed → PR approved
  ✗ Any test failed → Require fixes
```

---

## File Structure

```
cco/
├── COMPREHENSIVE_TEST_STRATEGY.md       ← Main strategy document
├── TEST_STRATEGY_QUICK_START.md         ← Implementation guide
├── TEST_STRATEGY_VISUAL_SUMMARY.md      ← This file
│
├── tests/
│   ├── smoke_test_comprehensive.sh      ← Both Mode 1 + Mode 2
│   │
│   ├── integration/
│   │   ├── mode1_server_basic.rs        ← Mode 1 only
│   │   ├── mode1_server_endpoints.rs    ← Mode 1 only
│   │   ├── mode1_server_shutdown.rs     ← Mode 1 only
│   │   ├── mode2_tui_daemon.rs          ← Mode 2 only
│   │   └── mode2_tui_connection.rs      ← Mode 2 only
│   │
│   ├── e2e/
│   │   ├── mode1_comprehensive.sh       ← Mode 1 only
│   │   ├── mode2_comprehensive.sh       ← Mode 2 only
│   │   └── helpers.sh                   ← Shared utilities
│   │
│   └── common/
│       ├── test_helpers.rs              ← Shared Rust utilities
│       └── fixtures/                    ← Test data
│
└── .github/workflows/
    └── test.yml                         ← Updated CI/CD pipeline
```

---

## Success Metrics

```
METRIC                          CURRENT    TARGET    HOW TO MEASURE
─────────────────────────────────────────────────────────────────
Mode 1 tests passing            Unknown    100%      cargo test --test '*mode1*'
Mode 2 tests passing            0%         100%      cargo test --test '*mode2*'
Smoke test time                 Unknown    < 5min    time ./smoke_test_comprehensive.sh
Test coverage                   Unknown    > 80%     cargo tarpaulin
CI/CD pass rate                 Unknown    100%      GitHub Actions checks
Mode-specific blind spots       Many       Zero      Code review + test naming
Days without mode-specific bugs Unknown    ∞         Track in bug reports

Once all metrics hit target → Production ready!
```

---

## From This Document

- **Main Strategy** → `COMPREHENSIVE_TEST_STRATEGY.md` (70 pages, comprehensive)
- **Quick Start** → `TEST_STRATEGY_QUICK_START.md` (implementation roadmap)
- **This** → `TEST_STRATEGY_VISUAL_SUMMARY.md` (visual reference)

---

## Key Takeaways

1. **Current state is blind**: Only Mode 1 (explicit server) is tested
2. **Mode 2 is critical**: Default `cco` command (TUI/daemon) completely untested
3. **Solution is multi-layered**: Smoke test → Integration tests → E2E tests
4. **Implementation is fast**: 2-3 weeks to complete coverage
5. **Prevention is critical**: Future mode-specific failures caught immediately

---

## Next Steps

1. **Week 1**: Create smoke test validating both modes
2. **Week 2**: Add Mode 1 and Mode 2 integration tests
3. **Week 3**: Add E2E workflow tests and performance benchmarks
4. **Ongoing**: Maintain test coverage as code evolves

**Status**: Ready to implement
**Effort**: 2-3 weeks
**Payoff**: Zero future mode-specific test failures
