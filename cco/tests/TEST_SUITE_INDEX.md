# CCO Test Suite Index

**Test Suite Coverage:** FUSE VFS + CLI Enhancements
**Target Coverage:** 90%+ across all phases
**Test Frameworks:** Rust (tokio::test), Shell Scripts (E2E)

---

## Test Organization

### FUSE VFS Tests (4 Phases)

#### Phase 1: Mount/Unmount and Basic File Operations
**File:** `fuse_vfs_phase1_tests.rs`
**Coverage Target:** 90%+
**Test Count:** 20+ tests

- **Unit Tests (13):**
  - `test_mount_creates_directory` - Verify mount point creation
  - `test_mount_succeeds_with_valid_path` - Valid mount succeeds
  - `test_unmount_cleans_up_filesystem` - Cleanup on unmount
  - `test_file_read_agents_json` - Read agents.sealed
  - `test_file_read_orchestrator_json` - Read orchestrator.sealed
  - `test_file_read_hooks_json` - Read hooks.sealed
  - `test_file_read_manifest` - Read and validate .manifest
  - `test_file_read_health` - Health check returns "OK"
  - `test_file_not_found_returns_enoent` - ENOENT error handling
  - `test_directory_listing` - Directory structure validation
  - `test_concurrent_reads` - Multiple simultaneous reads
  - `test_permission_error_handling` - Permission denied handling
  - `test_already_mounted_handling` - Double mount prevention

- **Integration Tests (5):**
  - `test_daemon_lifecycle_mounts_vfs_on_start` - Auto-mount on daemon start
  - `test_daemon_lifecycle_unmounts_vfs_on_stop` - Auto-unmount on daemon stop
  - `test_vfs_files_accessible_via_cli` - CLI file access
  - `test_mount_survives_concurrent_access` - Concurrent load testing
  - `test_unmount_with_open_file_handles` - Graceful handle closure

- **Error Path Tests (5):**
  - `test_mount_permission_denied` - Permission errors
  - `test_mount_path_already_exists` - Existing path handling
  - `test_mount_path_not_writable` - Read-only path rejection
  - `test_file_read_timeout` - Read timeout handling
  - `test_file_read_invalid_path` - Path traversal prevention

---

#### Phase 2: Encryption & Sealing
**File:** `fuse_vfs_phase2_tests.rs`
**Coverage Target:** 90%+
**Test Count:** 25+ tests

- **Unit Tests: Encryption (13):**
  - `test_aes256gcm_encrypt_decrypt_roundtrip` - Encryption round-trip
  - `test_different_iv_produces_different_ciphertext` - IV randomness
  - `test_authentication_tag_validates` - AEAD tag verification
  - `test_authentication_tag_fails_on_corruption` - Tampering detection
  - `test_key_derivation_machine_binding` - Machine-specific keys
  - `test_key_derivation_different_user_different_key` - User isolation
  - `test_key_derivation_different_purpose_different_key` - Purpose separation
  - `test_hmac_signature_valid_for_sealed_file` - Signature validation
  - `test_hmac_signature_fails_on_tampering` - Tamper detection
  - `test_gzip_compression_decompresses` - Compression round-trip
  - `test_sbf1_header_parsing` - Header structure validation
  - `test_sbf1_version_mismatch_rejected` - Version validation
  - `test_sbf1_corrupted_header_rejected` - Corrupted header detection

- **Integration Tests: Sealing (7):**
  - `test_seal_agents_json` - Seal agent definitions
  - `test_unseal_agents_json` - Unseal agent definitions
  - `test_sealed_file_serving_from_vfs` - Serve sealed files from VFS
  - `test_cross_machine_unsealing_fails` - Machine binding validation
  - `test_unsealing_with_wrong_password_fails` - Authentication failure
  - `test_large_file_encryption` - Large file handling
  - `test_all_three_sealed_files` - All sealed files work

- **Security Tests: Encryption (5):**
  - `test_key_derivation_machine_id_from_etc_machine_id` - Machine ID source
  - `test_key_derivation_user_id_isolation` - UID incorporation
  - `test_timing_attack_resistance` - Constant-time comparison
  - `test_iv_uniqueness_across_encryptions` - IV uniqueness
  - `test_nonce_reuse_prevention` - Nonce generation validation

---

#### Phase 3: Anti-Debugging & Protection
**File:** `fuse_vfs_phase3_tests.rs`
**Coverage Target:** 90%+
**Test Count:** 20+ tests

- **Unit Tests: Debugger Detection (4):**
  - `test_ptrace_detection_under_gdb` - GDB detection (Linux)
  - `test_ptrace_detection_under_lldb` - LLDB detection (macOS)
  - `test_ptrace_detection_under_strace` - strace detection (Linux)
  - `test_no_false_positive_in_normal_execution` - No false positives

- **Unit Tests: Memory Protection (4):**
  - `test_guard_pages_protect_buffer` - Guard page functionality
  - `test_stack_canary_detects_overflow` - Stack canary detection
  - `test_sensitive_data_zeroing` - Data zeroing verification
  - `test_zeroize_crate_effective` - Zeroize crate validation

- **Integration Tests (3):**
  - `test_anti_debug_in_full_daemon_lifecycle` - Full daemon anti-debug
  - `test_memory_protection_under_load` - Protection under stress
  - `test_graceful_degradation_on_debug_detection` - Graceful handling

- **Security Tests (3 + 9 manual):**
  - Manual tests for debugger access prevention
  - Memory dump analysis
  - Code injection prevention

---

#### Phase 4: Monitoring & Health
**File:** `fuse_vfs_phase4_tests.rs`
**Coverage Target:** 90%+
**Test Count:** 25+ tests

- **Unit Tests: Health Endpoint (4):**
  - `test_health_endpoint_returns_json` - JSON response validation
  - `test_health_endpoint_status_field` - Status field presence
  - `test_health_endpoint_mounted_field` - Mounted field accuracy
  - `test_health_endpoint_uptime_field` - Uptime tracking

- **Unit Tests: Prometheus Metrics (4):**
  - `test_prometheus_metrics_format` - Prometheus format validation
  - `test_file_read_latency_metric` - Read latency metric
  - `test_unsealing_latency_metric` - Unsealing latency metric
  - `test_auth_failure_metric` - Authentication failure counter

- **Integration Tests (3):**
  - `test_health_checks_under_load` - Health endpoint under stress
  - `test_error_logging_does_not_expose_keys` - Log safety
  - `test_prometheus_metrics_correct` - Metric accuracy

- **Performance Tests (4):**
  - `test_file_read_latency_less_than_2ms` - Read performance
  - `test_unsealing_latency_less_than_5ms` - Unsealing performance
  - `test_100_concurrent_reads` - Concurrent load handling
  - `test_1000_file_reads_per_second` - Throughput target

- **Monitoring Tests (3):**
  - `test_health_monitor_detects_failures` - Failure detection
  - `test_metrics_exported_to_prometheus` - Metrics export
  - `test_health_endpoint_response_time` - Endpoint performance

- **Error Handling Tests (3):**
  - `test_health_check_when_vfs_not_mounted` - Degraded status
  - `test_metrics_available_even_when_vfs_down` - Metrics availability
  - `test_error_logging_structured` - Structured error logs

---

### CLI Enhancement Tests (4 Phases)

#### CLI Launcher Tests
**File:** `cli_launcher_tests.rs`
**Coverage Target:** 90%+
**Test Count:** 50+ tests

**Phase 1: Launcher Module (10 tests)**
- Unit tests with mocking:
  - `test_ensure_daemon_running_already_running`
  - `test_ensure_daemon_running_starts_if_not_running`
  - `test_ensure_daemon_running_timeout_after_3_seconds`
  - `test_verify_vfs_mounted_succeeds_if_healthy`
  - `test_verify_vfs_mounted_fails_if_not_mounted`
  - `test_verify_vfs_mounted_fails_if_unhealthy`
  - `test_set_orchestrator_env_vars`
  - `test_find_claude_code_executable_finds_in_path`
  - `test_find_claude_code_executable_fails_if_not_found`
  - `test_launch_claude_code_full_flow`

- Integration tests (5):
  - `test_full_workflow_daemon_start_to_claude_launch`
  - `test_daemon_auto_start_succeeds`
  - `test_vfs_health_check_succeeds_after_daemon_start`
  - `test_environment_variables_inherited_by_claude`
  - `test_current_working_directory_preserved`

- Error path tests (9):
  - All error scenarios with recovery suggestions

**Phase 2: TUI Subcommand (7 tests)**
- `test_launch_tui_checks_daemon`
- `test_launch_tui_auto_starts_if_needed`
- `test_launch_tui_cancels_if_user_refuses`
- `test_launch_tui_calls_existing_tui_code`
- `test_cco_tui_command_launches_dashboard`
- `test_tui_and_launcher_can_run_simultaneously`
- `test_tui_daemon_auto_start_flow`

**Phase 3: CLI Routing (13 tests)**
- Command parsing and routing validation
- Pass-through argument handling
- Backward compatibility verification

**Phase 4: Full Test Suite (11 tests)**
- Performance benchmarks (startup < 3s, health < 100ms, parsing < 50ms)
- E2E workflows (clean environment, crash recovery, multiple sessions)
- Backward compatibility (all existing commands)

---

## Test Execution

### Running All Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --test fuse_vfs_phase1_tests

# Run specific test
cargo test test_mount_creates_directory
```

### Running by Phase
```bash
# FUSE VFS Phase 1
cargo test --test fuse_vfs_phase1_tests

# FUSE VFS Phase 2
cargo test --test fuse_vfs_phase2_tests

# FUSE VFS Phase 3
cargo test --test fuse_vfs_phase3_tests

# FUSE VFS Phase 4
cargo test --test fuse_vfs_phase4_tests

# CLI Launcher
cargo test --test cli_launcher_tests
```

### Coverage Analysis
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/index.html
```

### Performance Benchmarks
```bash
# Install cargo-criterion
cargo install cargo-criterion

# Run performance tests
cargo test test_file_read_latency_less_than_2ms -- --ignored
cargo test test_unsealing_latency_less_than_5ms -- --ignored
cargo test test_launcher_startup_under_3_seconds -- --ignored
```

---

## Test Categories

### Unit Tests
- **Count:** ~70 tests
- **Coverage:** Individual functions and modules
- **Speed:** Fast (<1s total)
- **Dependencies:** Mocked

### Integration Tests
- **Count:** ~30 tests
- **Coverage:** Component interactions
- **Speed:** Medium (~10s total)
- **Dependencies:** Real implementations

### E2E Tests
- **Count:** ~15 tests
- **Coverage:** Full system workflows
- **Speed:** Slow (~60s total)
- **Dependencies:** Full daemon + VFS + CLI

### Security Tests
- **Count:** ~12 tests
- **Coverage:** Threat model validation
- **Speed:** Varies
- **Dependencies:** Some require manual testing

### Performance Tests
- **Count:** ~10 tests
- **Coverage:** Performance targets
- **Speed:** Medium (~30s total)
- **Dependencies:** Real implementations

---

## Coverage Targets by Component

| Component | Tests | Coverage Target | Current Coverage |
|-----------|-------|-----------------|------------------|
| **FUSE VFS Phase 1** | 20+ | 90% | TBD |
| **FUSE VFS Phase 2** | 25+ | 90% | TBD |
| **FUSE VFS Phase 3** | 20+ | 90% | TBD |
| **FUSE VFS Phase 4** | 25+ | 90% | TBD |
| **CLI Launcher** | 50+ | 90% | TBD |
| **Total** | 140+ | 90% | TBD |

---

## Test Implementation Status

### Phase 1: Mount/Unmount ✅
- [x] Test file created
- [ ] Mock implementations
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Coverage verified

### Phase 2: Encryption & Sealing ✅
- [x] Test file created
- [ ] Mock implementations
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Coverage verified

### Phase 3: Anti-Debugging ✅
- [x] Test file created
- [ ] Mock implementations
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Manual security tests documented

### Phase 4: Monitoring & Health ✅
- [x] Test file created
- [ ] Mock implementations
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Performance benchmarks passing

### CLI Launcher ✅
- [x] Test file created
- [ ] Mock implementations
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] E2E tests passing

---

## Next Steps

1. **Implement FUSE VFS** (Phases 1-4)
2. **Implement CLI Launcher** (Phases 1-4)
3. **Replace mocks** with actual implementations in tests
4. **Run test suite** and verify 90%+ coverage
5. **Fix failing tests** and edge cases
6. **Performance optimization** to meet targets
7. **Security audit** (manual tests)
8. **CI/CD integration** (GitHub Actions)

---

## CI/CD Integration

### GitHub Actions Workflow
```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all
      - name: Coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Test Data Management

### Test Fixtures
- **Location:** `cco/tests/fixtures/`
- **Sealed Files:** Pre-generated sealed test files
- **Agent Configs:** Sample agent definitions
- **Mock Responses:** API response fixtures

### Test Utilities
- **Location:** `cco/tests/common/`
- **Helpers:** Common test setup/teardown
- **Mocks:** Shared mock implementations
- **Assertions:** Custom assertion functions

---

## Documentation

### Test Documentation Files
- `TEST_SUITE_INDEX.md` - This file (overview and index)
- `TEST_EXECUTION_GUIDE.md` - Detailed execution instructions
- `TEST_COVERAGE_REPORT.md` - Coverage analysis (generated)
- `TEST_FAILURE_ANALYSIS.md` - Common failure patterns

### Related Documentation
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_IMPLEMENTATION_PLAN.md`
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_CLI_ENHANCEMENTS.md`
- `/Users/brent/git/cc-orchestra/cco/docs/FUSE_VFS_QUICK_REFERENCE.md`

---

**Last Updated:** 2025-11-17
**Status:** Test Suite Complete - Ready for Implementation
**Next Review:** After Phase 1 implementation complete
