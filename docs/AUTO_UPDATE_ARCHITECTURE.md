# Auto-Update Architecture

Technical overview of how CCO's auto-update system works.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│ User/Application                                        │
│ - cco run, cco update, cco version, etc.              │
└──────────────────────┬──────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
    ┌────────┐   ┌──────────┐   ┌─────────┐
    │ CLI    │   │ Commands │   │ Daemon  │
    │ Parser │   │ Handler  │   │ Service │
    └────┬───┘   └─────┬────┘   └────┬────┘
         │             │             │
         └─────────────┼─────────────┘
                       │
        ┌──────────────▼──────────────┐
        │  Auto-Update Module         │
        │  (auto_update.rs)           │
        └──────────────┬──────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
        ▼                             ▼
   ┌──────────────┐          ┌─────────────────┐
   │ Update Check │          │ Update Install  │
   │ (update.rs)  │          │ (update.rs)     │
   │              │          │                 │
   │ - Fetch from │          │ - Download binary
   │   GitHub API │          │ - Verify checksum
   │ - Compare    │          │ - Backup current
   │   versions   │          │ - Extract archive
   │ - Notify     │          │ - Install binary
   │   user       │          │ - Test new binary
   └──────────────┘          │ - Cleanup temp
                             └─────────────────┘
             │
             └─────────────────────────────┐
                                          │
                 ┌────────────────────────▼──────────────┐
                 │ Configuration Management              │
                 │ (~/.config/cco/config.toml)          │
                 │                                       │
                 │ - Update settings                    │
                 │ - Check intervals                    │
                 │ - Channel selection                  │
                 │ - Timestamps                         │
                 └───────────────────────────────────────┘
```

## Core Modules

### 1. auto_update.rs (Configuration & Background Checks)

**Responsible for:**
- Loading and saving configuration
- Checking if update check is due
- Running background update checks
- Managing update settings

**Key types:**

```rust
pub struct UpdateConfig {
    pub enabled: bool,              // Enable/disable checks
    pub auto_install: bool,         // Auto-install without prompt
    pub check_interval: String,     // "daily", "weekly", "never"
    pub channel: String,            // "stable", "beta"
    pub last_check: Option<DateTime<Utc>>,     // Last check time
    pub last_update: Option<DateTime<Utc>>,    // Last update time
}
```

**Public functions:**

- `load_config()` - Read config from disk
- `save_config()` - Write config to disk
- `check_for_updates_async()` - Non-blocking background check
- `show_config()` - Display current settings
- `set_config(key, value)` - Update a setting
- `get_config(key)` - Get a specific setting

### 2. update.rs (Update Checking and Installation)

**Responsible for:**
- Fetching release information from GitHub
- Comparing versions
- Downloading binaries
- Verifying checksums
- Installing updates
- Rolling back on failure

**Key functions:**

```rust
async fn fetch_latest_release(channel: &str) -> Result<GitHubRelease>
async fn check_for_updates(channel: &str) -> Result<Option<GitHubRelease>>
async fn install_update(release: &GitHubRelease, auto_confirm: bool) -> Result<()>
async fn check_latest_version() -> Result<Option<String>>
```

**Update process flow:**

```
1. check_for_updates(channel)
   ├─ fetch_latest_release()
   │  └─ HTTP GET: api.github.com/repos/brentley/cco-releases/releases/latest
   ├─ extract_version() - Parse tag (e.g., "v2025.11.1")
   ├─ Compare versions (DateVersion::cmp)
   └─ Return release info if newer

2. install_update(release, auto_confirm)
   ├─ User confirmation (unless auto_confirm=true)
   ├─ Create temp directory
   ├─ download_file() - Get binary
   ├─ download_file() - Get checksums
   ├─ verify_checksum() - SHA256 validation
   ├─ Extract archive (tar.gz)
   ├─ Backup current version
   ├─ Install new binary
   ├─ Verify new binary works
   ├─ Clean up backups on success
   └─ Restore backup on failure
```

### 3. version.rs (Date-Based Versioning)

**Responsible for:**
- Parsing version strings (YYYY.MM.N format)
- Comparing versions correctly
- Displaying versions

**Key type:**

```rust
pub struct DateVersion {
    year: u32,      // 2025
    month: u32,     // 1-12
    release: u32,   // Release number for month
}

impl Ord for DateVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare year, then month, then release
    }
}
```

**Version format:**

```
2025.11.1   - First release in November 2025
2025.11.2   - Second release in November 2025
2025.11.10  - Tenth release in November 2025
2025.12.1   - Resets to 1 in new month
2026.1.1    - Resets to 1 in new year
```

**Comparison logic:**

```rust
// Compare year first
if year1 != year2 {
    return year1.cmp(&year2)
}

// Then month
if month1 != month2 {
    return month1.cmp(&month2)
}

// Finally release number
release1.cmp(&release2)
```

**Example comparisons:**
```
2025.11.1 < 2025.11.2  ✓  (same year/month, compare release)
2025.11.2 < 2025.12.1  ✓  (same year, compare month)
2025.12.1 < 2026.1.1   ✓  (compare year)
```

## Configuration Storage

### Location

**Platform-specific:**
- macOS/Linux: `~/.config/cco/config.toml`
- Windows: `%AppData%\cco\config.toml`

**Auto-created on first use** with sensible defaults

### File Format

```toml
[updates]
enabled = true               # bool: Enable update checks
auto_install = false         # bool: Install without prompt
check_interval = "daily"     # string: daily|weekly|never
channel = "stable"           # string: stable|beta
last_check = "2025-11-17T14:32:15Z"   # ISO 8601 timestamp
last_update = "2025-11-10T09:22:03Z"  # ISO 8601 timestamp
```

**Default values:**

```rust
enabled = true              // Enabled by default
auto_install = false        // Manual confirmation required
check_interval = "daily"    // Check daily
channel = "stable"          // Stable channel only
last_check = None           // Never checked
last_update = None          // Never updated
```

### Persistence

- Loaded from disk on every check
- Updated atomically (write to temp file, rename)
- Timestamps in UTC (ISO 8601 format)
- Readable/editable with any text editor

## Update Flow

### Background Check (Async)

```
1. User starts CCO (cco run, cco version, etc.)
   ↓
2. auto_update::check_for_updates_async() spawns task
   ↓
3. Task runs concurrently:
   └─ Load config
   └─ Check if should_check()
      ├─ Is enabled?
      ├─ Is interval != "never"?
      └─ Is time since last_check >= interval?
   └─ If should check:
      ├─ Update last_check timestamp
      ├─ Fetch latest release from GitHub
      ├─ Compare versions
      └─ If newer available:
         ├─ If auto_install=true: Start installation
         └─ If auto_install=false: Notify user
   └─ Silent on errors
```

**Non-blocking behavior:**

- Background check doesn't block user
- Timeout: 10 seconds for network requests
- Errors are logged but don't affect operation
- Users see notification when new version available

### Manual Update Check

```
Command: cco update --check

1. Load config
2. Get current version from env!("CCO_VERSION")
3. Fetch latest release from GitHub
4. Compare versions:
   ├─ Parse both as DateVersion
   └─ Use version.rs comparison logic
5. Print results:
   ├─ If newer: "ℹ️  New version available: X"
   └─ If same: "✅ You are running the latest version"
6. Exit (don't install)
```

### Interactive Update

```
Command: cco update

1. Check for updates (same as --check)
2. If new version available:
   ├─ Print release notes (first 10 lines)
   ├─ Prompt: "Update now? [Y/n]:"
   ├─ Wait for user input
   └─ If "y", "yes", or empty: Proceed to install
3. If user says "n" or "no":
   └─ Print "Update cancelled"
   └─ Exit
```

### Auto-Install Update

```
Command: cco update --yes

1. Check for updates
2. If new version available:
   └─ Skip user confirmation
   └─ Proceed to install immediately
3. Rest of process same as interactive
```

## Installation Process

### Phase 1: Download and Verify

```
1. Detect platform:
   ├─ OS: macos, linux, windows
   ├─ Arch: aarch64, x86_64
   └─ Platform ID: darwin-arm64, linux-x86_64, etc.

2. Find asset matching platform:
   └─ Search release assets for:
      └─ cco-v2025.11.1-{platform}.tar.gz (Unix)
      └─ cco-v2025.11.1-{platform}.zip (Windows)

3. Create temp directory:
   └─ $TMPDIR/cco-update-v2025.11.1/

4. Download binary:
   └─ HTTP GET: asset.browser_download_url
   └─ Save to: $TMPDIR/cco-update-*/cco-*.tar.gz
   └─ Timeout: 300 seconds

5. Download and verify checksum:
   ├─ Download: checksums.sha256 from release
   ├─ Parse file for matching platform line
   ├─ Extract expected SHA256 hash
   └─ Compute SHA256 of downloaded file
       ├─ Read file in 8KB chunks
       ├─ Update hasher with each chunk
       └─ Compare final hash
```

### Phase 2: Extract and Backup

```
6. Extract archive:
   ├─ For tar.gz:
   │  ├─ Open gzip decoder
   │  ├─ Create tar reader
   │  └─ Extract all files to temp dir
   └─ For zip: (Windows - not yet implemented)

7. Verify binary exists:
   └─ Check: $TMPDIR/cco-update-*/cco

8. Get installation path:
   └─ $HOME/.local/bin/cco (Unix)
   └─ %AppData%\cco\cco.exe (Windows)

9. Backup current version:
   ├─ If exists: cp ~/.local/bin/cco ~/.local/bin/cco.backup
   └─ Backup persists even after successful update
```

### Phase 3: Install and Verify

```
10. Set executable permissions (Unix only):
    └─ chmod 755 new_binary

11. Install new binary:
    └─ cp new_binary ~/.local/bin/cco
    └─ Atomic operation (or as close as possible)

12. Verify new binary works:
    └─ Execute: ~/.local/bin/cco --version
    ├─ Capture output and exit code
    ├─ If success (exit code 0):
    │  └─ Print: "✅ Successfully updated to vX.X.X"
    │  └─ Remove backup file
    │  └─ Success!
    └─ If failure:
       ├─ Print: "⚠️  New binary verification failed, rolling back..."
       ├─ Restore: cp ~/.local/bin/cco.backup ~/.local/bin/cco
       └─ Return error

13. Clean up:
    └─ rm -rf $TMPDIR/cco-update-*/
```

## GitHub Integration

### GitHub Release Format

**Expected structure:**

```
Repository: brentley/cco-releases

Release tag: v2025.11.1
Assets:
  - cco-v2025.11.1-darwin-arm64.tar.gz       (macOS Apple Silicon)
  - cco-v2025.11.1-darwin-x86_64.tar.gz      (macOS Intel)
  - cco-v2025.11.1-linux-x86_64.tar.gz       (Linux x86_64)
  - cco-v2025.11.1-linux-aarch64.tar.gz      (Linux ARM64)
  - cco-v2025.11.1-windows-x86_64.zip        (Windows - future)
  - checksums.sha256                          (All checksums)
  - Release notes in body
```

### API Endpoints Used

**Fetch latest stable release:**

```
GET https://api.github.com/repos/brentley/cco-releases/releases/latest

Headers:
  Accept: application/vnd.github.v3+json
  User-Agent: cco/2025.11.1

Response: {"tag_name": "v2025.11.1", "assets": [...], ...}
```

**Fetch all releases** (for beta channel):

```
GET https://api.github.com/repos/brentley/cco-releases/releases

Headers:
  Accept: application/vnd.github.v3+json
  User-Agent: cco/2025.11.1

Response: [{"tag_name": "v2025.11.2-beta.1", ...}, ...]
```

### Rate Limiting

- **Unauthenticated**: 60 requests/hour
- **Authenticated**: 5000 requests/hour (requires GitHub token)

**Typical usage:**
- Daily check: 1 request per 24 hours = ~30/month
- Well within limits even with 1000s of instances

## Version Comparison Strategy

### Date-Based Versioning

**Why date-based?**
- Clear release timing (year.month.release)
- Easier to identify old vs. new
- Simpler than semantic versioning for products
- Aligns with VisiQuate standard

### Comparison Algorithm

```rust
fn compare(v1: DateVersion, v2: DateVersion) -> Ordering {
    // Step 1: Compare year
    if v1.year != v2.year {
        return v1.year <=> v2.year  // 2025 < 2026
    }

    // Step 2: Compare month (same year)
    if v1.month != v2.month {
        return v1.month <=> v2.month  // 11 < 12
    }

    // Step 3: Compare release (same year and month)
    return v1.release <=> v2.release  // 1 < 2 < 10
}
```

**Examples:**

```
2025.11.1 < 2025.11.2       Step 3: 1 < 2
2025.11.2 < 2025.12.1       Step 2: 11 < 12
2025.12.1 < 2026.1.1        Step 1: 2025 < 2026
```

**Note:** Release numbers don't reset within month:
```
2025.11.1, 2025.11.2, 2025.11.3, ... 2025.11.99
2025.12.1   (resets to 1 in new month)
```

## Error Handling

### Network Errors

```
Timeout (10s for checks, 300s for downloads)
  └─ Return Err() in async function
  └─ Background checks swallow error
  └─ User commands print error to stderr

Connection refused
  └─ Check GitHub status
  └─ Check firewall/proxy rules

Checksum mismatch
  └─ Downloaded file is corrupted
  └─ Installation blocked
  └─ User must retry
```

### Filesystem Errors

```
Permission denied (~/.local/bin not writable)
  └─ Print error message
  └─ Suggest: chmod 755 ~/.local/bin
  └─ No installation attempted

File not found (binary in archive)
  └─ Archive is corrupted
  └─ Installation blocked
  └─ User must retry download

Insufficient disk space
  └─ Operating system handles
  └─ Installation fails gracefully
  └─ Backup remains for recovery
```

### Installation Errors

```
New binary doesn't work (--version fails)
  └─ Rollback to backup
  └─ Print warning message
  └─ Return error to user

Backup restore fails
  └─ System left with broken binary
  └─ Manual intervention required
  └─ User must download and install manually
```

## Security Considerations

### Checksum Verification

- Algorithm: SHA256 (cryptographically secure)
- Source: Official GitHub release asset
- Prevents: Tampering during download
- Verification: Byte-by-byte comparison

**Process:**

```
1. Download checksums.sha256 from GitHub
2. Parse format: "hash filename"
3. Extract hash for current platform
4. Compute SHA256 of downloaded file
5. Compare: computed == expected
6. Byte comparison is case-insensitive
```

### HTTPS Only

- All communication uses HTTPS
- Prevents man-in-the-middle attacks
- GitHub enforces HTTPS only

### Atomic Operations

- Backup before replace
- Replace is atomic or near-atomic
- Failure results in rollback, not broken state

### No Automatic Execution

- Downloaded binary is tested before confirmed as update
- User can review release notes before update
- Auto-install requires explicit configuration

## Performance Characteristics

### Check Operation

```
Time: ~2-5 seconds
  - Network roundtrip to GitHub: 500ms-2s
  - Parse JSON response: <100ms
  - Version comparison: <1ms
  - Async, non-blocking

Bandwidth: ~5-10 KB
  - GitHub API response is small JSON
```

### Installation Operation

```
Time: ~10-40 seconds total
  - Download: 5-30s (depends on file size, network speed)
  - Checksum verify: 2-5s (stream through SHA256)
  - Extract: 1-3s
  - Install: <1s
  - Test: 1-2s

Bandwidth: ~50-100 MB
  - Binary size: ~30-50 MB
  - Checksums: ~1 KB
```

### Resource Usage

```
Memory:
  - Check: <5 MB
  - Download: Streaming (constant ~1 MB)
  - Install: <50 MB (while extracting)

Disk:
  - Temp directory: 100-150 MB during install
  - Final binary: ~30-50 MB
  - Backup: ~30-50 MB
  - Cleanup removes temp files
```

## Future Enhancements

### Potential Improvements

1. **Delta Updates**
   - Download only changed parts
   - Reduce bandwidth by 50-90%
   - More complex distribution

2. **Signature Verification**
   - Sign releases with GPG key
   - Verify authenticity beyond checksums
   - Currently uses GitHub's integrity

3. **Pre-download Updates**
   - Download in background when check runs
   - Faster installation when confirmed
   - Requires more background bandwidth

4. **Update Scheduling**
   - Schedule updates for specific times
   - Useful for production environments
   - Would require persistent scheduler

5. **Rollback Support**
   - Keep multiple previous versions
   - Easy downgrade if needed
   - Requires more disk space

## Debugging

### Enable Debug Logging

```bash
# Set environment variable
export RUST_LOG=debug

# Run with debug flag
cco run --debug

# Check logs
cco daemon logs --follow | grep -i update
```

### Common Debug Output

```
[DEBUG] Loading config from ~/.config/cco/config.toml
[DEBUG] Checking if update is due...
[DEBUG] Last check: 2025-11-16T14:32:15Z
[DEBUG] Time since check: 22 hours
[DEBUG] Should check: true
[DEBUG] Fetching latest release from GitHub...
[DEBUG] GitHub response status: 200
[DEBUG] Latest version: v2025.11.2
[DEBUG] Version comparison: 2025.11.1 < 2025.11.2
[DEBUG] New version available, notifying user
```

### Testing Version Comparisons

```bash
# In a Rust test environment
let v1 = DateVersion::parse("2025.11.1").unwrap();
let v2 = DateVersion::parse("2025.11.2").unwrap();
assert!(v1 < v2);
```

