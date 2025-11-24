# Auto-Update Implementation - Seamless Default Behavior

## Overview

CCO now features **automatic updates by default** with seamless background installation and comprehensive logging. Users can easily opt-out if they prefer manual control.

## Summary of Changes

### 1. Default Configuration (CHANGED)

**New Defaults in `UpdateConfig`:**
```rust
UpdateConfig {
    enabled: true,           // Auto-updates enabled by default
    auto_install: true,      // ⭐ NEW: Auto-install without prompting
    check_interval: "daily", // Check every day
    channel: "stable",       // Use stable channel
}
```

**Previous behavior:** Auto-updates were enabled but required user confirmation.
**New behavior:** Auto-updates install silently without user intervention.

### 2. CLI Changes

**`cco update` Command:**
```bash
# ⭐ NEW: Auto-install immediately (no confirmation)
cco update

# Check without installing
cco update --check

# Use beta channel (auto-installs)
cco update --channel beta

# ⭐ NEW: Prompt for confirmation (opt-out of auto-install)
cco update --prompt
```

**Changed:**
- Removed `--yes` flag (auto-install is now the default)
- Added `--prompt` flag to request confirmation (reverses the default)

### 3. Background Update Behavior

**On Daemon Startup:**
1. Automatically checks for updates
2. Downloads and installs updates silently if available
3. Logs all activities to `~/.cco/logs/updates.log`
4. Notifies user of successful update (restart required)

**During Runtime:**
- Checks at configured intervals (daily/weekly)
- Silent background installation
- Graceful error handling with retry capability

### 4. Environment Variable Overrides (NEW)

Users can temporarily override configuration without editing config files:

```bash
# Disable auto-updates entirely
export CCO_AUTO_UPDATE=false

# Use beta channel temporarily
export CCO_AUTO_UPDATE_CHANNEL=beta

# Change check interval
export CCO_AUTO_UPDATE_INTERVAL=weekly
```

**Priority:** Environment variables > User config file > Defaults

### 5. Configuration Management

**View Current Configuration:**
```bash
cco config show
```

**Output includes:**
- Current configuration values
- Environment variable overrides (if any)
- Effective configuration (after overrides)
- Update log location and size

**Example:**
```
Update Configuration:
  Enabled: true
  Auto-install: true
  Check interval: daily
  Channel: stable
  Last check: 2025-11-17 14:30:00 UTC
  Last update: Never

Environment Variable Overrides:
  CCO_AUTO_UPDATE_CHANNEL: beta

Effective Configuration (with overrides):
  Enabled: true
  Auto-install: true
  Check interval: daily
  Channel: beta

Update Log:
  Location: /Users/username/.cco/logs/updates.log
  Size: 42 KB
```

**Modify Configuration:**
```bash
# Disable auto-updates permanently
cco config set updates.enabled false

# Disable auto-install (require confirmation)
cco config set updates.auto_install false

# Change check interval
cco config set updates.check_interval weekly

# Change update channel
cco config set updates.channel beta
```

### 6. Update Logging (NEW)

**Log File Location:** `~/.cco/logs/updates.log`

**Logged Events:**
- Every update check attempt
- Update availability detection
- Download progress and completion
- Installation success/failure
- Error messages with context

**Example Log Entries:**
```
[2025-11-17 14:30:00 UTC] Checking for updates (channel: stable)
[2025-11-17 14:30:02 UTC] Update available: 2025.11.1 -> 2025.11.2
[2025-11-17 14:30:02 UTC] Auto-installing CCO 2025.11.2 in background...
[2025-11-17 14:30:05 UTC] Downloading CCO 2025.11.2...
[2025-11-17 14:30:12 UTC] Download completed successfully
[2025-11-17 14:30:12 UTC] Installing update...
[2025-11-17 14:30:15 UTC] Successfully updated to 2025.11.2
```

**Log Rotation:**
- Rotates automatically when log exceeds 10MB
- Keeps last 30 days of logs
- Old logs are automatically cleaned up

### 7. Graceful Service Handling

**Update Process:**
1. Check if daemon is running
2. Download new binary to temporary location
3. Verify new binary works (`--version` check)
4. Backup current binary
5. Atomically replace binary (Unix: atomic `rename`, Windows: best-effort)
6. Verify new binary post-install
7. Rollback automatically if verification fails
8. Clean up temporary files

**No Automatic Restart:**
- Updates do NOT automatically restart the daemon
- User must manually restart: `cco daemon restart`
- This prevents unexpected service interruptions

## User Opt-Out Options

Users have multiple ways to control auto-updates:

### Permanent Opt-Out (Config File)
```bash
# Disable all auto-updates
cco config set updates.enabled false

# Keep checking but require confirmation
cco config set updates.auto_install false

# Never check for updates
cco config set updates.check_interval never
```

### Temporary Opt-Out (Environment Variable)
```bash
# Temporarily disable for single run
CCO_AUTO_UPDATE=false cco run

# Temporarily disable globally (in ~/.bashrc or ~/.zshrc)
export CCO_AUTO_UPDATE=false
```

### Per-Command Opt-Out
```bash
# Prompt before installing
cco update --prompt
```

## Implementation Details

### Modified Files

1. **`/Users/brent/git/cc-orchestra/cco/src/auto_update/mod.rs`**
   - Changed `auto_install` default from `false` to `true`
   - Added `UpdateConfig::apply_env_overrides()` method
   - Added logging functions: `log_update_event()`, `rotate_log_file()`
   - Enhanced `check_for_updates()` with comprehensive logging
   - Enhanced `perform_update()` with better error handling and logging
   - Updated `check_for_updates_internal()` for silent auto-install
   - Enhanced `show_config()` to display env overrides and log location
   - Added tests for environment variable overrides

2. **`/Users/brent/git/cc-orchestra/cco/src/main.rs`**
   - Changed `--yes` flag to `--prompt` flag (reversed logic)
   - Auto-confirm is now the default behavior
   - Added comment explaining the behavior change

### New Functions

**`UpdateConfig::apply_env_overrides()`**
```rust
/// Apply environment variable overrides
pub fn apply_env_overrides(&mut self) {
    // CCO_AUTO_UPDATE=false - Disable auto-updates
    if let Ok(val) = std::env::var("CCO_AUTO_UPDATE") {
        if let Ok(enabled) = val.parse::<bool>() {
            self.enabled = enabled;
            self.auto_install = enabled;
        }
    }

    // CCO_AUTO_UPDATE_CHANNEL=beta - Override channel
    if let Ok(channel) = std::env::var("CCO_AUTO_UPDATE_CHANNEL") {
        if ["stable", "beta"].contains(&channel.as_str()) {
            self.channel = channel;
        }
    }

    // CCO_AUTO_UPDATE_INTERVAL=weekly - Override check interval
    if let Ok(interval) = std::env::var("CCO_AUTO_UPDATE_INTERVAL") {
        if ["daily", "weekly", "never"].contains(&interval.as_str()) {
            self.check_interval = interval;
        }
    }
}
```

**`log_update_event(message: &str)`**
```rust
/// Log an update event to the updates log file
fn log_update_event(message: &str) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_message = format!("[{}] {}\n", timestamp, message);

    if let Ok(log_file) = get_updates_log_file() {
        // Rotate log if it's too large (>10MB)
        if let Ok(metadata) = fs::metadata(&log_file) {
            if metadata.len() > 10 * 1024 * 1024 {
                let _ = rotate_log_file(&log_file);
            }
        }

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
        {
            let _ = file.write_all(log_message.as_bytes());
        }
    }

    // Also log to tracing
    tracing::info!("{}", message);
}
```

**`rotate_log_file(log_file: &Path)`**
```rust
/// Rotate log file (keep last 30 days)
fn rotate_log_file(log_file: &Path) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let rotated_name = format!(
        "{}.{}",
        log_file.file_name().unwrap().to_string_lossy(),
        timestamp
    );
    let rotated_path = log_file.with_file_name(rotated_name);

    // Move current log to rotated file
    fs::rename(log_file, &rotated_path)?;

    // Clean up old logs (keep last 30 days)
    if let Ok(log_dir) = get_log_dir() {
        let cutoff = Utc::now() - Duration::days(30);
        if let Ok(entries) = fs::read_dir(log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("updates.log.") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                let modified_datetime: DateTime<Utc> = modified.into();
                                if modified_datetime < cutoff {
                                    let _ = fs::remove_file(&path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
```

**`get_log_file_path()`**
```rust
/// Get the path to the updates log file (exported for external use)
pub fn get_log_file_path() -> Result<PathBuf> {
    get_updates_log_file()
}
```

### Test Coverage

**New Tests:**
```rust
#[test]
fn test_env_override_enabled() {
    std::env::set_var("CCO_AUTO_UPDATE", "false");
    let mut config = UpdateConfig::default();
    config.apply_env_overrides();
    assert!(!config.enabled);
    assert!(!config.auto_install);
    std::env::remove_var("CCO_AUTO_UPDATE");
}

#[test]
fn test_env_override_channel() {
    std::env::set_var("CCO_AUTO_UPDATE_CHANNEL", "beta");
    let mut config = UpdateConfig::default();
    config.apply_env_overrides();
    assert_eq!(config.channel, "beta");
    std::env::remove_var("CCO_AUTO_UPDATE_CHANNEL");
}

#[test]
fn test_env_override_interval() {
    std::env::set_var("CCO_AUTO_UPDATE_INTERVAL", "weekly");
    let mut config = UpdateConfig::default();
    config.apply_env_overrides();
    assert_eq!(config.check_interval, "weekly");
    std::env::remove_var("CCO_AUTO_UPDATE_INTERVAL");
}
```

**Updated Test:**
```rust
#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.updates.enabled);
    assert!(config.updates.auto_install); // Changed: auto_install is now true by default
    assert_eq!(config.updates.check_interval, "daily");
    assert_eq!(config.updates.channel, "stable");
}
```

**All 15 tests pass:**
```bash
running 15 tests
test auto_update::github::tests::test_archive_extensions ... ok
test auto_update::github::tests::test_platform_detection ... ok
test auto_update::github::tests::test_platform_strings ... ok
test auto_update::tests::test_default_config ... ok
test auto_update::github::tests::test_asset_name_generation ... ok
test auto_update::tests::test_env_override_interval ... ok
test auto_update::tests::test_env_override_channel ... ok
test auto_update::tests::test_env_override_enabled ... ok
test auto_update::tests::test_should_check_disabled ... ok
test auto_update::tests::test_should_check_never_checked ... ok
test auto_update::tests::test_should_check_recently_checked ... ok
test auto_update::tests::test_should_check_old_check ... ok
test auto_update::updater::tests::test_get_install_path ... ok
test auto_update::updater::tests::test_checksum_verification ... ok
test auto_update::updater::tests::test_verify_current_binary ... ok

test result: ok. 15 passed; 0 failed; 0 ignored
```

## Security Considerations

1. **Checksum Verification:** All downloads are verified with SHA256 checksums
2. **Binary Verification:** New binary is tested before installation
3. **Atomic Replacement:** Binary replacement is atomic on Unix systems
4. **Rollback Capability:** Automatic rollback if new binary fails verification
5. **Backup Creation:** Current binary is backed up before replacement
6. **Repository Verification:** Only downloads from trusted GitHub repository

## Performance Impact

- **Background checks:** Non-blocking, spawn as tokio task
- **Network overhead:** Minimal (~30s timeout for GitHub API)
- **Disk space:** ~2x binary size during update (temporary)
- **CPU usage:** Negligible (only during download/install)
- **Log rotation:** Automatic cleanup keeps disk usage minimal

## Migration Path

**For existing users:**
1. Update CCO to this version
2. Existing configuration is preserved
3. If `auto_install: false` in config, it remains false
4. If no config exists, new defaults apply (auto-install enabled)
5. Environment variables can override any setting

**Recommended approach for cautious users:**
```bash
# After updating, disable auto-install if desired
cco config set updates.auto_install false

# Or check config to see current status
cco config show
```

## Troubleshooting

### Common Issues

**Q: How do I disable auto-updates?**
A: Run `cco config set updates.enabled false`

**Q: Where are update logs stored?**
A: Run `cco config show` (look for "Update Log" section)

**Q: How do I force an update check?**
A: Run `cco update`

**Q: Updates keep failing, what should I check?**
A: Check logs at `~/.cco/logs/updates.log` for error messages

**Q: How do I see the last update check time?**
A: Run `cco config show`

**Q: Can I temporarily disable updates?**
A: Yes, set `CCO_AUTO_UPDATE=false` environment variable

## Future Enhancements

Potential improvements for future versions:

1. **Daemon Restart Integration:** Automatically restart daemon after successful update
2. **Notification System:** Desktop notifications for updates
3. **Update Scheduling:** Specify exact times for update checks
4. **Pre/Post Update Hooks:** Custom scripts before/after updates
5. **Network Awareness:** Skip updates on metered connections
6. **Progress Indicators:** Show download progress in terminal
7. **Staged Rollouts:** Gradual rollout to percentage of users
8. **Delta Updates:** Only download changed parts of binary
9. **Signature Verification:** GPG/minisign signature verification

## Documentation

### User-facing Documentation

**Quick Start:**
```
CCO automatically updates itself by default. To disable:
  cco config set updates.enabled false
```

**Configuration Options:**
```
updates.enabled        - Enable/disable auto-updates (true/false)
updates.auto_install   - Auto-install without prompting (true/false)
updates.check_interval - How often to check (daily/weekly/never)
updates.channel        - Update channel (stable/beta)
```

**Environment Variables:**
```
CCO_AUTO_UPDATE=false              - Disable auto-updates
CCO_AUTO_UPDATE_CHANNEL=beta       - Use beta channel
CCO_AUTO_UPDATE_INTERVAL=weekly    - Check weekly
```

## Conclusion

This implementation achieves the goal of **seamless, automatic updates by default** while providing multiple opt-out mechanisms for users who prefer manual control. The comprehensive logging system ensures transparency and aids in troubleshooting.

**Key Benefits:**
✅ Zero-friction updates for most users
✅ Easy opt-out for cautious users
✅ Comprehensive logging for debugging
✅ Environment variable overrides for flexibility
✅ Robust error handling and rollback
✅ No service interruptions during updates

**Files Modified:**
- `/Users/brent/git/cc-orchestra/cco/src/auto_update/mod.rs` (enhanced)
- `/Users/brent/git/cc-orchestra/cco/src/main.rs` (CLI changes)

**Tests:** All 15 tests passing ✅

**Build Status:** Clean build with only minor warnings ✅
