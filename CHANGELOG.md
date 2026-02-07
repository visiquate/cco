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
- **Plugin System Enhancement** - Skills from `cco-plugin/` directory now automatically loaded
  - Repository-local skills, agents, and commands available in Claude Code sessions
  - Launcher passes cco-plugin directory to Claude Code plugin system

### Changed
- **CI/CD Optimization** - All workflows now use shared local sccache for faster builds
  - 90% cache hit rate expected after first build
  - Build times reduced from ~90s to 5-15s on subsequent runs
  - Cache shared across multiple self-hosted runners on same machine

### Fixed
- Skills were not being loaded from `cco-plugin/` directory in launched Claude Code sessions

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
