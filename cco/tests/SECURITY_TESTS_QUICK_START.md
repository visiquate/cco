# Security & Auto-Update Tests - Quick Start Guide

## Overview

The `security_auto_update_tests.rs` test suite provides comprehensive coverage for:
- 12 security vulnerability fixes (CRITICAL → LOW priority)
- Auto-update default behavior (enabled by default)
- Configuration override mechanisms
- Edge cases and error handling

## Quick Test Commands

### Run All Tests
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo test --test security_auto_update_tests
```

### Run by Category

```bash
# CRITICAL security tests (checksum, repository validation)
cargo test --test security_auto_update_tests critical_security

# HIGH priority security tests (temp dirs, permissions, path traversal)
cargo test --test security_auto_update_tests high_priority_security

# MEDIUM priority security tests (size limits, cleanup, sanitization)
cargo test --test security_auto_update_tests medium_priority_security

# LOW priority security tests (user-agent, certificate pinning)
cargo test --test security_auto_update_tests low_priority_security

# Auto-update defaults (auto-install enabled)
cargo test --test security_auto_update_tests auto_update_defaults

# Configuration overrides
cargo test --test security_auto_update_tests configuration_overrides

# Edge cases
cargo test --test security_auto_update_tests edge_cases

# Performance tests
cargo test --test security_auto_update_tests performance

# Integration tests
cargo test --test security_auto_update_tests integration
```

### Run Specific Tests

```bash
# Test checksum verification
cargo test --test security_auto_update_tests test_checksum_verification_mandatory

# Test repository validation
cargo test --test security_auto_update_tests test_repository_ownership_validation

# Test auto-install default
cargo test --test security_auto_update_tests test_default_auto_install_enabled

# Test secure temp directories
cargo test --test security_auto_update_tests test_secure_temp_directory_permissions
```

### Include Ignored Tests (Future Features)

```bash
# Run all tests including ignored ones
cargo test --test security_auto_update_tests -- --ignored

# Show which tests are ignored
cargo test --test security_auto_update_tests -- --list | grep ignored
```

### Verbose Output

```bash
# Show test output (println! statements)
cargo test --test security_auto_update_tests -- --nocapture

# Show test names as they run
cargo test --test security_auto_update_tests -- --show-output
```

## Test Summary

```
Total Tests:     42
Passing:         36 (85.7%)
Ignored:         6 (14.3%)  [Future: GPG, cert pinning, integration]
Failed:          0 (0%)
```

### Security Tests Breakdown

| Priority | Tests | Status | Coverage |
|----------|-------|--------|----------|
| CRITICAL | 4 | ✅ All passing | 100% |
| HIGH | 10 | ✅ All passing | 100% |
| MEDIUM | 6 | ✅ 5 passing, 1 future | 83% |
| LOW | 2 | ✅ 1 passing, 1 future | 50% |

### Functional Tests Breakdown

| Category | Tests | Status |
|----------|-------|--------|
| Auto-Update Defaults | 6 | ✅ All passing |
| Configuration Overrides | 4 | ✅ All passing |
| Edge Cases | 9 | ✅ 4 passing, 5 integration |
| Performance | 4 | ✅ 2 passing, 2 integration |
| Integration | 2 | ✅ All passing |

## What Each Test Category Covers

### CRITICAL Security (4 tests)

1. **Checksum Verification** - Mandatory SHA256 validation
2. **Tamper Detection** - Reject modified binaries
3. **Repository Ownership** - Only `brentley/cco-releases`
4. **Typosquatting Prevention** - Block look-alike repos

**Impact:** Prevents MITM attacks and supply chain compromises

### HIGH Priority Security (10 tests)

1. **Secure Temp Directories** - 0o700 permissions
2. **Random Temp Names** - Prevent race conditions
3. **Temp Cleanup** - No leaks on errors
4. **Executable Permissions** - Validate 0o755
5. **Reject Non-Executable** - Binary validation
6. **Path Traversal** - Block `../` attacks
7. **Filename Sanitization** - Clean API responses
8. **Release Tag Validation** - Prevent injection

**Impact:** Prevents local privilege escalation and file system attacks

### MEDIUM Priority Security (6 tests)

1. **Download Size Limits** - Max 100MB
2. **Partial Cleanup** - Clean failed downloads
3. **No Disk Leaks** - Disk space recovery
4. **GPG Verification** - Future: Signature validation
5. **Version Sanitization** - Prevent command injection

**Impact:** Prevents disk exhaustion and injection attacks

### LOW Priority Security (2 tests)

1. **User-Agent Privacy** - Generic UA (no OS leakage)
2. **Certificate Pinning** - Future: GitHub CA pinning

**Impact:** Reduces fingerprinting, prevents MITM

### Auto-Update Defaults (6 tests)

Verifies that:
- Updates are enabled by default
- Auto-install is ON (no user prompts)
- Check interval is daily
- Channel is stable
- Background checks work correctly

### Configuration Overrides (4 tests)

Verifies:
- Can disable updates: `updates.enabled = false`
- Can disable auto-install: `updates.auto_install = false`
- Can change interval: `updates.check_interval = never`
- Config serialization works

## Expected Test Output

```
running 42 tests
test critical_security::test_checksum_verification_mandatory ... ok
test critical_security::test_checksum_rejects_tampered_binary ... ok
test critical_security::test_repository_ownership_validation ... ok
test critical_security::test_repository_typosquatting_prevention ... ok
test high_priority_security::test_secure_temp_directory_permissions ... ok
test high_priority_security::test_random_temp_directory_names ... ok
test high_priority_security::test_temp_cleanup_on_error ... ok
test high_priority_security::test_executable_permission_validation ... ok
test high_priority_security::test_reject_non_executable_binary ... ok
test high_priority_security::test_path_traversal_prevention ... ok
test high_priority_security::test_filename_sanitization ... ok
test high_priority_security::test_release_tag_validation ... ok
test medium_priority_security::test_download_size_limits ... ok
test medium_priority_security::test_partial_cleanup_on_download_failure ... ok
test medium_priority_security::test_no_disk_leaks_on_error ... ok
test medium_priority_security::test_gpg_signature_verification ... ignored
test medium_priority_security::test_version_string_sanitization ... ok
test low_priority_security::test_user_agent_privacy ... ok
test low_priority_security::test_github_certificate_pinning ... ignored
test auto_update_defaults::test_default_auto_install_enabled ... ok
test auto_update_defaults::test_enable_auto_install ... ok
test auto_update_defaults::test_check_interval_configurations ... ok
test auto_update_defaults::test_update_channel_configurations ... ok
test auto_update_defaults::test_background_check_timing ... ok
test configuration_overrides::test_disable_updates_via_config ... ok
test configuration_overrides::test_disable_auto_install_requires_confirm ... ok
test configuration_overrides::test_set_check_interval_never ... ok
test configuration_overrides::test_config_serialization ... ok
test edge_cases::test_download_interruption ... ok
test edge_cases::test_checksum_mismatch ... ok
test edge_cases::test_permission_denied_handling ... ok
test edge_cases::test_concurrent_update_prevention ... ok
test edge_cases::test_update_without_daemon ... ok
test edge_cases::test_disk_full_handling ... ok
test edge_cases::test_no_internet_graceful_degradation ... ignored
test edge_cases::test_github_rate_limit_handling ... ignored
test performance::test_background_check_non_blocking ... ok
test performance::test_logging_performance ... ok
test performance::test_update_download_low_impact ... ignored
test performance::test_memory_usage_during_update ... ignored
test integration::test_full_update_flow_simulation ... ok
test integration::test_rollback_on_verification_failure ... ok

test result: ok. 36 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out
```

## Troubleshooting

### Test Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo test --test security_auto_update_tests --no-run
```

### Test Failures

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test --test security_auto_update_tests

# Run single failing test with output
cargo test --test security_auto_update_tests test_name -- --nocapture
```

### Permission Issues (Unix)

Some tests require Unix-specific features:
```bash
# These tests are Unix-only (use #[cfg(unix)])
- test_secure_temp_directory_permissions
- test_executable_permission_validation
- test_reject_non_executable_binary
- test_permission_denied_handling
```

On Windows, these tests are automatically skipped.

## Integration with CI/CD

Add to your CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Run Security Tests
  run: cargo test --test security_auto_update_tests

- name: Run Security Tests (with ignored)
  run: cargo test --test security_auto_update_tests -- --ignored
  continue-on-error: true  # Future tests may fail
```

## Test Maintenance

### When to Update Tests

1. **Security Auditor makes changes** → Update corresponding tests
2. **Default behavior changes** → Update `auto_update_defaults` tests
3. **New security vulnerabilities** → Add new test cases
4. **Configuration options added** → Add to `configuration_overrides`

### Adding New Tests

```rust
// Add to appropriate module in security_auto_update_tests.rs
mod critical_security {
    #[test]
    fn test_new_security_feature() {
        // Test implementation
    }
}
```

### Future Tests to Enable

When features are implemented, remove `#[ignore]`:

1. `test_gpg_signature_verification` - After GPG support added
2. `test_github_certificate_pinning` - After cert pinning implemented
3. `test_no_internet_graceful_degradation` - With network simulation
4. `test_github_rate_limit_handling` - With API mocking
5. `test_update_download_low_impact` - With live daemon
6. `test_memory_usage_during_update` - With profiling tools

## Verification Checklist

Before deployment, verify:

- [ ] All CRITICAL tests passing
- [ ] All HIGH priority tests passing
- [ ] No security test failures
- [ ] Auto-install defaults verified
- [ ] Configuration overrides work
- [ ] Edge cases handled gracefully
- [ ] Performance tests passing
- [ ] Integration tests passing

## Related Documentation

- Full report: `SECURITY_AUTO_UPDATE_TEST_REPORT.md`
- Source code: `src/auto_update/mod.rs`
- Test file: `tests/security_auto_update_tests.rs`
- Security fixes: Implemented by Security Auditor
- Auto-install: Default enabled at `src/auto_update/mod.rs:164`

---

**Last Updated:** 2025-11-17
**Test Suite Version:** 1.0
**Maintained By:** QA/Test Engineer
