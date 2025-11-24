# Testing Guide: Install and Update Commands

This guide covers testing the newly implemented `install`, `update`, and `config` commands for CCO.

## Prerequisites

- Rust toolchain installed (1.75+)
- Access to build the CCO binary
- macOS, Linux, or Windows system

## Building CCO

```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
```

The binary will be at: `target/release/cco`

## Testing Install Command

### Test 1: Fresh Installation

```bash
# Remove any existing installation
rm -f ~/.local/bin/cco

# Run install
./target/release/cco install
```

**Expected Output:**
```
→ Installing CCO v0.2.0...
→ Creating ~/.local/bin/
→ Copying binary to ~/.local/bin/cco
→ Detected shell: zsh
  PATH already configured in ~/.zshrc

✅ Installation complete!

Verify with: cco version
```

**Verify:**
```bash
ls -lh ~/.local/bin/cco
~/.local/bin/cco version
```

### Test 2: Idempotent Installation

```bash
# Run install again (should detect existing installation)
./target/release/cco install
```

**Expected Output:**
```
→ Installing CCO v0.2.0...
→ Creating ~/.local/bin/
⚠️  CCO is already installed at ~/.local/bin/cco
   Use --force to reinstall
```

### Test 3: Force Reinstall

```bash
./target/release/cco install --force
```

**Expected Output:**
```
→ Installing CCO v0.2.0...
→ Creating ~/.local/bin/
→ Copying binary to ~/.local/bin/cco
→ Detected shell: zsh
  ~/.local/bin is already in PATH

✅ Installation complete!
```

### Test 4: Shell Detection

Test with different shells:

```bash
# Test with bash
SHELL=/bin/bash ./target/release/cco install --force

# Test with zsh
SHELL=/bin/zsh ./target/release/cco install --force

# Test with fish
SHELL=/usr/local/bin/fish ./target/release/cco install --force
```

## Testing Config Command

### Test 1: Show Default Configuration

```bash
./target/release/cco config show
```

**Expected Output:**
```
Update Configuration:
  Enabled: true
  Auto-install: false
  Check interval: daily
  Channel: stable
  Last check: Never
  Last update: Never
```

### Test 2: Set Configuration Values

```bash
# Enable/disable updates
./target/release/cco config set updates.enabled false
./target/release/cco config get updates.enabled

# Change check interval
./target/release/cco config set updates.check_interval weekly
./target/release/cco config get updates.check_interval

# Enable auto-install
./target/release/cco config set updates.auto_install true
./target/release/cco config get updates.auto_install

# Change channel
./target/release/cco config set updates.channel beta
./target/release/cco config get updates.channel
```

### Test 3: Invalid Configuration

```bash
# Try invalid interval
./target/release/cco config set updates.check_interval monthly  # Should error

# Try invalid channel
./target/release/cco config set updates.channel nightly  # Should error

# Try invalid boolean
./target/release/cco config set updates.enabled maybe  # Should error
```

### Test 4: Configuration Persistence

```bash
# Set a value
./target/release/cco config set updates.auto_install true

# Verify it persists
./target/release/cco config show
cat ~/.config/cco/config.toml
```

## Testing Update Command

### Test 1: Check for Updates (No Release)

```bash
./target/release/cco update --check
```

**Expected Output:**
```
→ Checking for updates...
→ Current version: v0.2.0
Error: GitHub API returned status: 404 Not Found
```

This is expected since the `brentley/cco-releases` repository doesn't exist yet.

### Test 2: Update Command Help

```bash
./target/release/cco update --help
```

**Expected Output:**
```
Check for and install updates

Usage: cco update [OPTIONS]

Options:
      --check              Only check for updates, don't install
      --yes                Auto-confirm installation
      --channel <CHANNEL>  Update channel (stable or beta)
  -h, --help               Print help
```

### Test 3: Channel Selection

```bash
# Check stable channel
./target/release/cco update --check --channel stable

# Check beta channel
./target/release/cco update --check --channel beta
```

## Testing Auto-Update Background Check

### Test 1: Background Check on Startup

```bash
# Run any command (auto-update check happens in background)
./target/release/cco version

# Check that last_check timestamp is updated
./target/release/cco config show
```

The `Last check` field should be updated if:
- `updates.enabled = true`
- More than 1 day has passed since last check (for daily interval)

### Test 2: Disable Auto-Update

```bash
# Disable updates
./target/release/cco config set updates.enabled false

# Run command (no background check should happen)
./target/release/cco version

# Verify last_check didn't update
./target/release/cco config show
```

## Integration Testing (Once Release Repository Exists)

### Prerequisites
1. Create `brentley/cco-releases` repository on GitHub
2. Create a test release (v0.3.0) with assets
3. Include checksums.sha256 file

### Test 1: Successful Update Check

```bash
./target/release/cco update --check
```

**Expected Output:**
```
→ Checking for updates...
→ Current version: v0.2.0
→ Latest version: v0.3.0

ℹ️  New version available: v0.3.0
Run 'cco update' to install
```

### Test 2: Install Update

```bash
./target/release/cco update --yes
```

**Expected Output:**
```
→ Checking for updates...
→ Current version: v0.2.0
→ Latest version: v0.3.0

What's new in v0.3.0:
  • Feature 1
  • Feature 2
  • Bug fix

→ Downloading CCO v0.3.0...
→ Verifying checksum...
  ✓ Checksum verified
→ Extracting archive...
→ Backing up current version...
→ Installing update...
✅ Successfully updated to v0.3.0

Restart CCO to use the new version.
```

### Test 3: Rollback on Failure

To test rollback, you would need to:
1. Modify the checksum file to have an incorrect hash
2. Run update
3. Verify it rolls back to the previous version

### Test 4: Auto-Install

```bash
# Enable auto-install
./target/release/cco config set updates.auto_install true

# Run any command (auto-install should happen in background)
./target/release/cco version
```

## Edge Cases to Test

### 1. Network Failures

```bash
# Disconnect from internet
# Run update check
./target/release/cco update --check
# Should handle gracefully with timeout error
```

### 2. Permission Issues

```bash
# Remove write permission from ~/.local/bin
chmod 555 ~/.local/bin

# Try to install
./target/release/cco install
# Should show permission error

# Restore permissions
chmod 755 ~/.local/bin
```

### 3. Disk Space

```bash
# Check behavior when disk is full
# (Difficult to test manually)
```

### 4. Interrupted Update

```bash
# Start an update and kill the process mid-download
./target/release/cco update &
# Kill process after a few seconds
kill %1

# Verify backup exists and can be restored
ls -lh ~/.local/bin/cco.backup
```

## Platform-Specific Tests

### macOS

```bash
# Test on both Intel and Apple Silicon
uname -m  # Should show x86_64 or arm64

# Test shell detection for .bash_profile vs .bashrc
./target/release/cco install
```

### Linux

```bash
# Test on different distributions
./target/release/cco install

# Verify .bashrc is used (not .bash_profile)
cat ~/.bashrc | grep "\.local/bin"
```

### Windows (Future)

```bash
# Test PowerShell installer
# Test .local/bin creation in user profile
# Test PATH update in Windows
```

## Performance Testing

### 1. Install Speed

```bash
time ./target/release/cco install --force
# Should complete in < 1 second
```

### 2. Update Check Speed

```bash
time ./target/release/cco update --check
# Should complete in < 5 seconds
```

### 3. Background Check Impact

```bash
# Should not slow down other commands
time ./target/release/cco version
# Should be nearly instant
```

## Security Testing

### 1. Checksum Verification

- Modify the downloaded binary
- Verify update fails checksum verification
- Verify rollback occurs

### 2. HTTPS Enforcement

- Verify all downloads use HTTPS
- Check user-agent headers are set
- Verify timeout handling

### 3. File Permissions

```bash
# Check installed binary permissions
ls -l ~/.local/bin/cco
# Should be: -rwxr-xr-x (755)
```

## Cleanup After Testing

```bash
# Remove test installation
rm -f ~/.local/bin/cco
rm -f ~/.local/bin/cco.backup

# Remove test configuration
rm -rf ~/.config/cco

# Remove PATH modifications from shell RC files
# (Edit ~/.zshrc, ~/.bashrc, etc. manually if needed)
```

## Known Limitations

1. **Windows Support**: ZIP extraction not yet implemented
2. **Release Repository**: Must be created before full testing
3. **GPG Signatures**: Not yet implemented (planned for future)
4. **Delta Updates**: Not implemented (full binary download only)

## Troubleshooting

### "Command not found: cco"

- Verify ~/.local/bin is in PATH: `echo $PATH | grep ".local/bin"`
- Restart shell or run: `source ~/.zshrc`

### "Permission denied"

- Check binary permissions: `ls -l ~/.local/bin/cco`
- Make executable: `chmod +x ~/.local/bin/cco`

### "GitHub API rate limit exceeded"

- Wait 1 hour for rate limit to reset
- Or authenticate with GitHub token

### "Checksum verification failed"

- Verify the checksums.sha256 file is correct
- Check that the binary wasn't corrupted during download
- Retry the download

## Success Criteria

All tests pass:
- ✅ Fresh installation works
- ✅ Idempotent installation (no duplicate PATH entries)
- ✅ Force reinstall works
- ✅ Shell detection works (zsh, bash, fish)
- ✅ Configuration management works
- ✅ Configuration persists across runs
- ✅ Update check detects version correctly
- ✅ Background update check runs (when enabled)
- ✅ Error handling is graceful
- ✅ File permissions are correct
- ✅ Binary is executable after install

## Next Steps

1. **Create Release Repository**: Set up `brentley/cco-releases` on GitHub
2. **CI/CD Integration**: Add GitHub Actions for building releases
3. **Create Test Release**: Publish v0.3.0 for integration testing
4. **Full Update Flow**: Test complete update cycle with real releases
5. **Documentation**: Create public installation guide
6. **Windows Support**: Implement ZIP extraction and Windows installer
