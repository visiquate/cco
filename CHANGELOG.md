# Changelog

All notable changes to Claude Code Orchestra will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to sequential versioning: YYYY.MM.N (year.month.sequence).

## [v2026.2.20] - 2026-02-07

### Added
- **Autopilot Mode** - Autonomous multi-agent coordination for extended development sessions
  - `/autopilot <goal>` skill enables Chief Architect to work autonomously for hours
  - Natural interruption: any user message pauses autopilot
  - Control commands: `/autopilot-resume`, `/autopilot-stop`, `/autopilot-status`
  - Progress updates every 2 minutes
  - Automatic agent spawning and coordination
  - Safety features: max 4-hour runtime, approval gates for risky operations
- **Embedded Plugin System** - All 137 plugin files compiled directly into the CCO binary
  - Skills, commands, and agent definitions available from any working directory
  - Single binary distribution with no external dependencies
  - Unified `cco-plugin/` directory replaces separate hooks and skills plugins
- **Subagent Thread Visualization** - TUI shows subagent threads under parent conversations
  - Indented tree view with model tier, agent type, description, message count, and cost
  - Correlates agents to Task tool calls via progress event matching
- **Model Tier Enforcement** - Mandatory model tier lookup table in orchestrator prompt
  - Prevents agents from being spawned with incorrect (expensive) model tiers
  - Includes verification checklist and correct/incorrect examples
- **TUI Time Filtering** - All dashboard sections now respect time range filters
  - Filter by today, this week, this month, or all time
  - Applies to Overall Summary, Cost by Project, and Cost by Model views
- **Daemon Auto-Restart** - Daemon automatically picks up newly installed binary after `cco update`

### Changed
- **CI/CD Optimization** - Dual caching strategy for faster builds
  - Shared sccache for C/C++ dependencies with 100% hit rate
  - Shared `CARGO_TARGET_DIR` for Rust incremental builds across runners
  - Build times reduced from ~90s to 5-15s on subsequent runs
- **Reproducible Builds** - `BUILD_DATE` derived from git commit timestamp instead of current date
- **Versioning** - Clarified YYYY.MM.N format (N is sequential, not day of month)

### Fixed
- Skills were not being loaded from `cco-plugin/` directory in launched Claude Code sessions
- Checksum generation now happens after artifact download to prevent mismatches
- Conversation title extraction skips system-generated messages
- Corrected agent counts: 117 total (1 Opus + 35 Sonnet + 81 Haiku)
- Token cost calculations consolidated into single pricing source of truth

## [2025.11.1] - 2025-11-28

### Removed
- **Sealed file system** (dead code) - Files were written but never read (~300 lines)
  - `generate_agents()`, `generate_rules()`, `generate_hooks()` stub methods
  - `.cco-agents-sealed`, `.cco-rules-sealed`, `.cco-hooks-sealed` files
  - Unused struct fields and accessor methods
- **Test stubs** - 75+ test functions with `todo!()` that panicked (~2,500 lines)
  - Deleted 5 complete test files containing only stubs
  - Cleaned 8 API endpoint stubs from knowledge_store_tests.rs
  - Cleaned 5 integration test stubs from knowledge_lancedb_tests.rs
- **Environment variable placeholders** - Removed unused null-returning env vars
- **Phase 1/Phase 2 terminology** - Removed future-promise language from docs

### Added
- **Real server metrics** implementation
  - Actual uptime tracking using `Instant::elapsed()`
  - Real memory usage via `sysinfo` crate (process RSS)
  - Active agents counter with `Arc<AtomicUsize>`

### Changed
- **Documentation overhaul** - Updated to reflect current reality
  - SECURITY_VALIDATION_TEMP_DIR_TRANSITION.md rewritten (honest security posture)
  - Code comments updated from "Phase 1/2" to "Currently/Future enhancement"
  - 7 source files with comment improvements

### Fixed
- **52 metadata type errors** in knowledge store tests
  - Changed `serde_json::Value` to `String` with `.to_string()`
  - Fixed test assertions to parse JSON strings correctly

### Security
- ✅ **Security audit approved** for production deployment
- All dependencies clean (no CVEs)
- `sysinfo` v0.30 added - 83.9M+ downloads, actively maintained
- Honest security model with appropriate controls
- Eliminated security theater (fake "sealed" files)

### Impact
- **Code reduction**: ~2,800+ lines of dead code and stubs removed
- **Quality improvement**: Zero `todo!()` panics remaining
- **Architecture cleanup**: Cleaner, more maintainable codebase
- **Production ready**: Honest implementation, real metrics, secure

### Technical Details
- Build status: ✅ Successful compilation (main codebase)
- Test suite: All implemented tests pass
- Working features verified: --agents flag, settings generation, daemon lifecycle
