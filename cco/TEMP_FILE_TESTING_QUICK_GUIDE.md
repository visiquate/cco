# Temp File Testing Quick Guide

## Test Files Overview

### New Test Files (65 tests)
```
cco/tests/
├── daemon_temp_files_tests.rs          (17 tests) - Daemon temp file lifecycle
├── cli_launcher_temp_files_tests.rs    (20 tests) - CLI launcher integration
└── encryption_temp_files_tests.rs      (28 tests) - SBF encryption/decryption
```

### Updated Test Files (54 tests)
```
cco/tests/
└── cli_launcher_tests.rs               (54 tests) - Updated for temp files
```

## Running Tests

### All Tests
```bash
cargo test --tests
```

### Specific Test Suites
```bash
# Daemon temp file tests
cargo test --test daemon_temp_files_tests

# CLI launcher temp file tests
cargo test --test cli_launcher_temp_files_tests

# Encryption tests
cargo test --test encryption_temp_files_tests

# Updated CLI launcher tests
cargo test --test cli_launcher_tests
```

### Individual Tests
```bash
# Example: Test daemon creates temp files
cargo test test_daemon_creates_temp_files

# Example: Test launcher discovers temp settings
cargo test test_launcher_discovers_temp_settings

# Example: Test seal/unseal roundtrip
cargo test test_seal_unseal_roundtrip
```

## Test Categories

### 1. Daemon Tests (17 tests)
**Purpose:** Verify daemon creates/manages temp files correctly

**Key Tests:**
- `test_daemon_creates_temp_files` - Verifies all 4 temp files created
- `test_daemon_cleanup_removes_temp_files` - Verifies cleanup on shutdown
- `test_temp_files_contain_encrypted_data` - Verifies SBF header
- `test_temp_files_have_correct_permissions` - Verifies 0o644 permissions
- `test_temp_file_creation_under_100ms` - Performance benchmark

**Expected Temp Files:**
- `.cco-orchestrator-settings` (JSON)
- `.cco-agents-sealed` (SBF encrypted)
- `.cco-rules-sealed` (SBF encrypted)
- `.cco-hooks-sealed` (SBF encrypted)

### 2. CLI Launcher Tests (20 tests)
**Purpose:** Verify launcher discovers and uses temp files

**Key Tests:**
- `test_launcher_discovers_temp_settings` - Finds settings file
- `test_launcher_sets_env_vars` - Sets ORCHESTRATOR_* env vars
- `test_full_launcher_workflow` - End-to-end launcher flow
- `test_launcher_error_when_temp_files_missing` - Error handling
- `test_temp_file_discovery_under_10ms` - Performance benchmark

**Environment Variables Set:**
- `ORCHESTRATOR_ENABLED=true`
- `ORCHESTRATOR_SETTINGS=/tmp/.cco-orchestrator-settings`
- `ORCHESTRATOR_AGENTS=/tmp/.cco-agents-sealed`
- `ORCHESTRATOR_RULES=/tmp/.cco-rules-sealed`
- `ORCHESTRATOR_HOOKS=/tmp/.cco-hooks-sealed`
- `ORCHESTRATOR_API_URL=http://localhost:3000`

### 3. Encryption Tests (28 tests)
**Purpose:** Verify SBF encryption independent of VFS

**Key Tests:**
- `test_seal_unseal_roundtrip` - Basic encryption/decryption
- `test_machine_binding_same_machine_succeeds` - Machine binding
- `test_machine_binding_different_machine_fails` - Cross-machine prevention
- `test_hmac_detects_tampering` - Tamper detection
- `test_gzip_compression_reduces_size` - Compression
- `test_encryption_performance_small_data` - Performance (<10ms)

**SBF Format:**
- Header: `CCOSEAL1` (8 bytes)
- Version: `1` (4 bytes)
- IV/Nonce: 12 bytes
- Encrypted payload
- Auth tag: 16 bytes
- HMAC signature: 32 bytes

### 4. Updated CLI Tests (54 tests)
**Purpose:** CLI launcher with temp file integration

**Key Changes:**
- `verify_vfs_mounted()` → `verify_temp_files_exist()`
- `/var/run/cco/` → `env::temp_dir()`
- VFS health checks → Temp file validation
- Fixed async/sync annotations

## Mock Strategy

### Temp Directory Mocking
```rust
// Create isolated test environment
let temp_dir = tempfile::tempdir().unwrap();
env::set_var("TMPDIR", temp_dir.path()); // macOS/Linux
env::set_var("TEMP", temp_dir.path());   // Windows

// ... run tests ...

// Cleanup happens automatically when temp_dir drops
```

### Example Test Structure
```rust
#[tokio::test]
async fn test_example() {
    // Arrange: Setup temp files
    let temp_dir = env::temp_dir();
    let settings_path = temp_dir.join(".cco-orchestrator-settings");
    fs::write(&settings_path, b"{}").unwrap();

    // Act: Run functionality
    let result = verify_temp_files_exist().await;

    // Assert: Verify behavior
    assert!(result.is_ok());

    // Cleanup
    fs::remove_file(&settings_path).ok();
}
```

## CI/CD Integration

### GitHub Actions Workflow
```yaml
test:
  runs-on: [ubuntu-latest, macos-latest, windows-latest]
  steps:
    - uses: actions/checkout@v3
    - uses: rust-build/rust-toolchain@v1
    - name: Run all tests
      run: cargo test --all
    - name: Run temp file tests specifically
      run: |
        cargo test --test daemon_temp_files_tests
        cargo test --test cli_launcher_temp_files_tests
        cargo test --test encryption_temp_files_tests
```

**Benefits:**
- No FUSE setup needed
- No special privileges required
- Works on all platforms (Mac, Windows, Linux)
- Standard temp directory handling

## Debugging Tips

### Enable Test Logging
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Check Temp Files Manually
```bash
# List temp files (macOS/Linux)
ls -la /tmp/.cco-*

# List temp files (Windows)
dir %TEMP%\.cco-*
```

### Verify SBF Header
```bash
# Check if file has SBF header
xxd /tmp/.cco-agents-sealed | head -1
# Should show: 43 43 4f 53 45 41 4c 31 (CCOSEAL1)
```

### Test Individual Scenarios
```bash
# Test only creation
cargo test test_daemon_creates_temp_files -- --exact

# Test only cleanup
cargo test test_daemon_cleanup_removes_temp_files -- --exact

# Test only encryption
cargo test test_seal_unseal_roundtrip -- --exact
```

## Performance Targets

| Operation | Target | Test |
|-----------|--------|------|
| Temp file creation | <100ms | `test_temp_file_creation_under_100ms` |
| Temp file cleanup | <50ms | `test_temp_file_cleanup_under_50ms` |
| Temp file discovery | <10ms | `test_temp_file_discovery_under_10ms` |
| Env var setting | <5ms | `test_env_var_setting_under_5ms` |
| Small encryption | <10ms | `test_encryption_performance_small_data` |
| Large encryption | <100ms | `test_encryption_performance_large_data` |

## Common Issues

### Issue: Temp files not cleaned up
**Solution:** Ensure daemon shutdown handler runs cleanup

### Issue: Permission denied on temp files
**Solution:** Check file permissions (should be 0o644)

### Issue: Cross-platform path issues
**Solution:** Use `env::temp_dir()` and `PathBuf::join()`

### Issue: SBF header validation fails
**Solution:** Verify sealed data starts with "CCOSEAL1"

## Next Steps

1. **Implement temp file creation in daemon**
2. **Implement temp file discovery in CLI launcher**
3. **Run full test suite**
4. **Verify coverage (target: 90%+)**
5. **Deploy to CI/CD**

---

**Quick Reference:** `/Users/brent/git/cc-orchestra/cco/TEST_SUITE_REFACTOR_SUMMARY.md`
**Test Files:** `/Users/brent/git/cc-orchestra/cco/tests/`
**Status:** ✅ Ready for Implementation
