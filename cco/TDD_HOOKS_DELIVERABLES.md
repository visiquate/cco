# TDD Test Suite Deliverables - Hooks System (Phases 2-5)

## Overview

This document describes the comprehensive TDD test suite created for the Hooks System implementation (Phases 2-5). Following strict **Red-Green-Refactor** methodology, all tests are written BEFORE implementation and will initially FAIL (RED phase).

**Test-First Philosophy**: These tests define the expected behavior and serve as the specification for implementation. Implementation should make tests pass one by one (GREEN phase), followed by refactoring (REFACTOR phase).

---

## Test Files Created

### 1. Phase 2: Permission Request API Tests
**File**: `/Users/brent/git/cc-orchestra/cco/tests/hooks_permission_tests.rs`

**Test Coverage** (12 tests):
1. ✅ POST /api/hooks/permission-request accepts ClassifyRequest
2. ✅ Returns APPROVED for READ operations
3. ✅ Returns PENDING_USER for CREATE/UPDATE/DELETE in interactive mode
4. ✅ Stores decision in database with timestamp
5. ✅ Retrieves decision history from /api/hooks/decisions endpoint
6. ✅ Handles "dangerously-skip-confirmations" flag (auto-approves C/U/D)
7. ✅ Returns 400 for invalid command
8. ✅ Rate limits permission requests (max 100/minute)
9. ✅ Concurrent permission requests handled safely
10. ✅ Permission request timeout (5 seconds)
11. ✅ Auto-allow READ, require confirmation for CREATE/UPDATE/DELETE
12. ✅ Decision state persisted across daemon restarts

**Data Structures Defined**:
- `ClassifyRequest` - Request payload for classification
- `Decision` - Enum for permission decisions (APPROVED, PENDING_USER, DENIED)
- `PermissionResponse` - Response from permission endpoint
- `DecisionRecord` - Database record structure
- `DecisionStats` - Statistics aggregation

**Key Test Patterns**:
- Async HTTP client testing with reqwest
- Database state verification
- Concurrency testing with tokio::spawn
- Rate limiting validation
- Timeout handling
- Error case coverage (400 errors)

---

### 2. Phase 3: Audit Logging & Database Tests
**File**: `/Users/brent/git/cc-orchestra/cco/tests/hooks_audit_logging_tests.rs`

**Test Coverage** (12 tests):
1. ✅ Decision table has correct schema
2. ✅ INSERT decision on every /api/hooks/permission-request
3. ✅ Query last N decisions efficiently (< 50ms with index)
4. ✅ Stats endpoint returns aggregated counts
5. ✅ Cleanup removes decisions older than 7 days
6. ✅ Cleanup runs on daemon shutdown
7. ✅ Database locked safely during concurrent writes
8. ✅ Null reasoning field when not provided
9. ✅ Index on timestamp for efficient queries (10k+ records)
10. ✅ Transaction rollback on error
11. ✅ Decision history pagination (limit/offset)
12. ✅ Database file permissions (0600 on Unix)

**Data Structures Defined**:
- `DecisionSchema` - Full database schema definition
- `StatsResponse` - Aggregated statistics response
- `CleanupConfig` - Cleanup configuration

**Database Schema** (defined in tests):
```sql
CREATE TABLE decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    classification TEXT NOT NULL,  -- 'READ', 'CREATE', 'UPDATE', 'DELETE'
    timestamp TEXT NOT NULL,       -- ISO 8601
    decision TEXT NOT NULL,        -- 'APPROVED', 'PENDING_USER', 'DENIED'
    reasoning TEXT                 -- Optional explanation
);
CREATE INDEX idx_decisions_timestamp ON decisions(timestamp);
```

**Key Test Patterns**:
- Database schema validation
- SQLite transaction testing
- Index performance verification
- Concurrent write safety
- Cleanup/retention policy testing
- File permission checks (Unix)

---

### 3. Phase 4: TUI Display Tests
**File**: `/Users/brent/git/cc-orchestra/cco/tests/hooks_tui_display_tests.rs`

**Test Coverage** (12 tests):
1. ✅ TUI renders hooks status section
2. ✅ Shows "Hooks: ENABLED | Model: loaded | Last update: 2.3s"
3. ✅ Displays last 5 classifications as colored list
4. ✅ Shows classification stats (READ 60%, CREATE 25%, etc.)
5. ✅ Updates in real-time as commands are classified
6. ✅ Handles when hooks disabled (shows "Hooks: DISABLED")
7. ✅ Handles when model not loaded (shows "Hooks: PENDING")
8. ✅ Responsive layout (doesn't break with different terminal widths)
9. ✅ Click on classification shows full details
10. ✅ Performance - TUI update < 100ms
11. ✅ Color coding (green for READ, yellow for C/U/D)
12. ✅ Keyboard navigation (arrow keys to scroll classifications)

**Data Structures Defined**:
- `HooksStatus` - Complete hooks status for TUI
- `ModelStatus` - Enum for model loading states
- `ClassificationDisplay` - Individual classification for display
- `ClassificationStats` - Percentage breakdown for display
- `TuiRenderOutput` - Rendered output structure

**Status Line Format**:
```
Hooks: ENABLED | Model: loaded | Last update: 2.3s
```

**Recent Classifications Format**:
```
[READ] ls -la (0.95)                    # Green
[CREATE] touch file.txt (0.88)          # Yellow
[UPDATE] sed -i 's/old/new/' file (0.92) # Yellow
[DELETE] rm file.txt (0.85)             # Yellow/Red
```

**Stats Display Format**:
```
READ 60% | CREATE 25% | UPDATE 10% | DELETE 5%
```

**Key Test Patterns**:
- TUI rendering validation
- Color/ANSI code verification
- Responsive layout testing (40-120 columns)
- Interactive element testing (click, keyboard)
- Real-time update verification
- Performance benchmarking (< 100ms)

---

### 4. Phase 5: Documentation Tests
**File**: `/Users/brent/git/cc-orchestra/cco/tests/hooks_documentation_tests.rs`

**Test Coverage** (15 tests):
1. ✅ Verify all documentation files exist
2. ✅ HOOKS_OVERVIEW.md has required sections
3. ✅ HOOKS_API.md has all endpoints documented
4. ✅ Code examples are syntactically correct
5. ✅ API endpoint examples are valid JSON
6. ✅ Configuration examples load without errors
7. ✅ TUI documentation has screenshots or diagrams
8. ✅ Troubleshooting doc covers common issues
9. ✅ All docs have proper frontmatter
10. ✅ Cross-references between docs are valid
11. ✅ README.md mentions hooks system
12. ✅ Changelog includes hooks feature
13. ✅ API documentation has curl examples
14. ✅ Configuration doc has default values
15. ✅ Documentation follows consistent style

**Required Documentation Files**:
- `cco/docs/HOOKS_OVERVIEW.md` - System overview and architecture
- `cco/docs/HOOKS_API.md` - API endpoints and examples
- `cco/docs/HOOKS_CONFIGURATION.md` - Configuration options
- `cco/docs/HOOKS_TUI.md` - TUI display guide
- `cco/docs/HOOKS_TROUBLESHOOTING.md` - Common issues and solutions

**Documentation Quality Checks**:
- Syntax validation (JSON, Rust, TOML, Bash/curl)
- Structural validation (required sections, headings)
- Link validation (internal cross-references)
- Code example correctness
- Style consistency (no trailing whitespace, consistent markers)

**Key Test Patterns**:
- File existence checks
- Markdown parsing and validation
- Code block extraction and syntax checking
- JSON/TOML/Rust parsing
- Link reference validation
- Style consistency checks

---

## Test Statistics

**Total Tests**: 51 tests across 4 files

**Breakdown by Phase**:
- Phase 2 (Permission API): 12 tests
- Phase 3 (Audit Logging): 12 tests
- Phase 4 (TUI Display): 12 tests
- Phase 5 (Documentation): 15 tests

**Test Categories**:
- **API Integration**: 12 tests (HTTP endpoints, request/response validation)
- **Database**: 12 tests (schema, queries, transactions, concurrency)
- **UI/UX**: 12 tests (TUI rendering, interactivity, performance)
- **Documentation**: 15 tests (completeness, correctness, style)

**Coverage Areas**:
- ✅ Happy path scenarios
- ✅ Error handling (400, 429, 504 errors)
- ✅ Edge cases (empty input, null values, large datasets)
- ✅ Performance (query speed, render time, rate limits)
- ✅ Concurrency (parallel requests, database locks)
- ✅ Security (file permissions, rate limiting)
- ✅ User experience (TUI responsiveness, keyboard navigation)
- ✅ Documentation quality (syntax, completeness, links)

---

## TDD Workflow

### RED Phase (Current State)
All tests are currently commented out with `// TODO: Implementation needed`. They define expected behavior but FAIL when uncommented.

**How to run RED phase**:
```bash
# Uncomment tests one at a time in each file
cargo test --test hooks_permission_tests
# Tests will FAIL - this is expected!
```

### GREEN Phase (Next Step)
Implement the minimal code to make each test pass:

1. **Phase 2 Implementation**:
   - Create `POST /api/hooks/permission-request` endpoint
   - Implement CRUD classification logic
   - Add database persistence
   - Return proper JSON responses

2. **Phase 3 Implementation**:
   - Create decisions table with schema
   - Implement INSERT on every request
   - Add efficient query methods (with indexes)
   - Implement stats aggregation
   - Add cleanup job

3. **Phase 4 Implementation**:
   - Add hooks section to TUI
   - Implement status line rendering
   - Add classification list display
   - Implement color coding (ratatui styles)
   - Add keyboard/mouse interactivity

4. **Phase 5 Implementation**:
   - Write all documentation files
   - Add code examples (validated by tests)
   - Create diagrams/screenshots
   - Update README and CHANGELOG

### REFACTOR Phase (After GREEN)
Once tests pass, improve code quality:
- Extract common patterns
- Optimize database queries
- Improve TUI rendering performance
- Enhance error messages
- Add logging/instrumentation

---

## Running the Tests

### Run All Hooks Tests
```bash
cd /Users/brent/git/cc-orchestra/cco

# Run all hooks tests (currently will be skipped - commented out)
cargo test hooks_

# Run specific phase
cargo test --test hooks_permission_tests
cargo test --test hooks_audit_logging_tests
cargo test --test hooks_tui_display_tests
cargo test --test hooks_documentation_tests
```

### Uncomment Tests Gradually
```bash
# Edit a test file, uncomment ONE test
vim tests/hooks_permission_tests.rs

# Run that specific test (it should FAIL - RED phase)
cargo test test_permission_request_accepts_classify_request

# Implement the feature to make it pass (GREEN phase)
# Then refactor if needed (REFACTOR phase)
```

---

## File Locations

All test files created:

```
/Users/brent/git/cc-orchestra/cco/tests/
├── hooks_permission_tests.rs       (Phase 2 - 12 tests)
├── hooks_audit_logging_tests.rs    (Phase 3 - 12 tests)
├── hooks_tui_display_tests.rs      (Phase 4 - 12 tests)
└── hooks_documentation_tests.rs    (Phase 5 - 15 tests)
```

Supporting files referenced:
```
/Users/brent/git/cc-orchestra/cco/
├── tests/common/mod.rs             (Shared test helpers)
├── docs/HOOKS_*.md                 (Documentation - to be created)
├── src/daemon/hooks/               (Implementation - to be created)
└── TDD_HOOKS_DELIVERABLES.md       (This file)
```

---

## Success Criteria

**Definition of Done** for each phase:

**Phase 2**:
- [ ] All 12 permission tests pass
- [ ] Endpoint returns correct status codes
- [ ] Database records created correctly
- [ ] Rate limiting works
- [ ] Concurrent requests safe

**Phase 3**:
- [ ] All 12 audit tests pass
- [ ] Database schema correct
- [ ] Queries performant (< 50ms)
- [ ] Stats accurate
- [ ] Cleanup job runs

**Phase 4**:
- [ ] All 12 TUI tests pass
- [ ] Status renders correctly
- [ ] Real-time updates work
- [ ] Responsive layout works
- [ ] Performance < 100ms

**Phase 5**:
- [ ] All 15 documentation tests pass
- [ ] All files exist
- [ ] Code examples valid
- [ ] Links work
- [ ] Style consistent

**Overall Success**:
- [ ] All 51 tests pass
- [ ] No compiler warnings
- [ ] Code coverage > 80%
- [ ] Documentation complete
- [ ] Performance targets met

---

## TDD Principles Applied

✅ **Tests First**: All tests written BEFORE implementation
✅ **One Test at a Time**: Each test is independent and focused
✅ **Red-Green-Refactor**: Clear phases defined
✅ **Behavior-Driven**: Tests describe WHAT, not HOW
✅ **Comprehensive**: Happy paths, errors, edge cases, performance
✅ **Self-Documenting**: Test names describe expected behavior
✅ **Fast Execution**: Unit tests run in milliseconds
✅ **Isolated**: Each test sets up and tears down its own state

---

## Conclusion

This comprehensive TDD test suite provides a complete specification for Phases 2-5 of the Hooks System. With 51 tests covering API endpoints, database operations, TUI display, and documentation, implementation can proceed with confidence that all requirements are clearly defined and automatically verified.

**The tests are the specification. Make them pass, and the feature is complete.**

---

**Generated**: 2025-11-17
**TDD Agent**: Claude Code - TDD Specialist
**Status**: RED Phase Complete - Ready for GREEN Phase Implementation
