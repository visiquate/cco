# Simplified Architecture (Temp Files)

## Three Components

CCO uses a simple architecture based on temporary files instead of virtual filesystems.

### 1. Daemon (background process on port 3000)

- Always running when CCO is active
- Manages orchestration
- Writes encrypted files to OS temp directory
- Provides HTTP API for metrics and coordination

### 2. Settings Files (in OS temp directory)

**Automatically created on daemon start:**

- `.cco-orchestrator-settings` - Main orchestration settings
- `.cco-agents-sealed` - 119 encrypted agent definitions
- `.cco-rules-sealed` - Orchestration rules
- `.cco-hooks-sealed` - Pre/post-compaction hooks

**Properties:**

- Auto-created on daemon start
- Auto-cleaned on daemon stop
- Discoverable by Claude Code via environment variables
- All files are encrypted before writing to disk

### 3. Claude Code (with orchestration)

- Launched with `--settings` flag pointing to temp file
- Reads encrypted settings from temp directory
- Integrates with daemon API for coordination
- Reports metrics back to daemon

## File Locations

**Temp directory paths by platform:**

| Platform | Temp Directory Path |
|----------|-------------------|
| macOS | `/var/folders/xx/xxx/T/` (where xx/xxx varies by user/session) |
| Windows | `C:\Users\[user]\AppData\Local\Temp\` |
| Linux | `/tmp/` |

**Access via environment variable:**

```bash
# macOS/Linux
echo $TMPDIR

# Windows (PowerShell)
echo $env:TEMP
```

## Encryption

All files are encrypted before writing to disk using military-grade encryption:

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Authentication**: HMAC-SHA256 for integrity verification
- **Key binding**: Derived from machine ID + username + build secret
- **No credentials in plaintext**: All agent definitions and settings encrypted

**Why encryption?**

- Protects sensitive orchestration logic
- Prevents tampering with agent definitions
- Ensures integrity across daemon restarts
- Machine-specific keys prevent file copying to other systems

## Architecture Diagram

```
┌─────────────────────────────────────────────────────┐
│  cco (CLI)                                          │
│  ├─ Ensures daemon is running                      │
│  ├─ Creates temp files in OS temp directory        │
│  ├─ Sets ORCHESTRATOR_* environment variables      │
│  └─ Launches Claude Code in current directory      │
└─────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────┐
│  CCO Daemon (Background)                            │
│  ├─ HTTP API Server (port 3000)                     │
│  ├─ Temp Files (OS temp directory)                  │
│  │   ├─ .cco-orchestrator-settings (encrypted)     │
│  │   ├─ .cco-agents-sealed (encrypted)             │
│  │   ├─ .cco-rules-sealed (encrypted)              │
│  │   └─ .cco-hooks-sealed (encrypted)              │
│  └─ Agent communication & metrics                   │
└─────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────┐
│  Claude Code (with Orchestration)                   │
│  ├─ Reads settings via --settings flag             │
│  ├─ Decrypts agent definitions from temp files     │
│  ├─ Coordinates 119 specialized agents             │
│  └─ Reports metrics to daemon API                  │
└─────────────────────────────────────────────────────┘
```

## How It Works

### Startup Flow

1. **User runs `cco`**
   - CLI checks if daemon is running (GET localhost:3000/health)
   - If not running: daemon auto-starts (takes ~3 seconds first time)

2. **Daemon creates temp files**
   - Generates encryption keys from machine ID + username
   - Writes encrypted files to `$TMPDIR`:
     - `.cco-orchestrator-settings`
     - `.cco-agents-sealed`
     - `.cco-rules-sealed`
     - `.cco-hooks-sealed`
   - Sets file permissions (owner read-only)

3. **CLI sets environment variables**
   ```bash
   ORCHESTRATOR_ENABLED=true
   ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings
   ORCHESTRATOR_API_URL=http://localhost:3000
   ```

4. **Claude Code launches**
   - Reads settings file path from environment
   - Decrypts and loads agent definitions
   - Connects to daemon for coordination
   - Ready for orchestrated development

### Shutdown Flow

1. **User stops daemon** (`cco daemon stop` or daemon exits)
2. **Daemon cleanup**
   - Removes all temp files
   - Closes HTTP server
   - Exits cleanly
3. **Temp directory cleaned**
   - No files remain after daemon stops
   - Next start creates fresh encrypted files

## Benefits of Temp File Approach

### Cross-Platform Compatibility

✅ **Works everywhere:**
- macOS (no macFUSE required)
- Windows (native support)
- Linux (standard temp directory)

✅ **No system dependencies:**
- No kernel extensions
- No FUSE modules
- No admin privileges needed

### Simplicity

✅ **Simple to understand:**
- Temp files are a familiar concept
- No virtual filesystems to explain
- Easy to debug (just check if files exist)

✅ **Simple to implement:**
- Standard OS temp directory APIs
- Built-in automatic cleanup
- No complex mount/unmount logic

### Security

✅ **Encrypted at rest:**
- All files encrypted with AES-256-GCM
- Machine-specific encryption keys
- Cannot copy files to other systems

✅ **Automatic cleanup:**
- Files removed on daemon shutdown
- No persistent state on disk
- Reduces attack surface

### Developer Experience

✅ **Zero configuration:**
- No setup required
- Works out of the box
- Runs as standard user

✅ **Fast startup:**
- No mount delays
- Instant file access
- Simple file system operations

## Comparison: Temp Files vs FUSE VFS

| Feature | Temp Files | FUSE VFS (old) |
|---------|-----------|----------------|
| **Setup** | None | Requires macFUSE installation |
| **Cross-platform** | macOS, Windows, Linux | macOS, Linux only |
| **Dependencies** | None | Kernel extensions, FUSE modules |
| **Admin required** | No | Yes (for first-time setup) |
| **Complexity** | Low | High |
| **Debugging** | Simple (check temp files) | Complex (mount state, VFS logs) |
| **Security** | Encrypted files | Encrypted virtual files |
| **Cleanup** | Automatic | Manual unmount needed |
| **Performance** | Fast | Virtual FS overhead |

## Environment Variables

**Set by CCO CLI when launching Claude Code:**

```bash
# Core settings
ORCHESTRATOR_ENABLED=true
ORCHESTRATOR_SETTINGS=$TMPDIR/.cco-orchestrator-settings

# API endpoint
ORCHESTRATOR_API_URL=http://localhost:3000

# Platform detection (set automatically)
# macOS
TMPDIR=/var/folders/xx/xxx/T/

# Linux
TMPDIR=/tmp/

# Windows (PowerShell)
$env:TEMP=C:\Users\[user]\AppData\Local\Temp\
```

## File Security

### Encryption Details

**Key Derivation:**
```
encryption_key = HKDF-SHA256(
  machine_id +
  username +
  build_secret
)
```

**File Format:**
```
[16-byte nonce][encrypted data][16-byte auth tag]
```

**Properties:**
- Unique key per machine + user
- Cannot decrypt on different machine
- Integrity verification via auth tag
- Replay attack protection via nonce

### File Permissions

**Unix (macOS, Linux):**
```bash
-rw------- (600)  # Owner read/write only
```

**Windows:**
```
Owner: Full Control
Others: No access
```

## Troubleshooting

### Files not found?

**Check temp directory:**
```bash
# macOS/Linux
ls -la $TMPDIR/.cco-*

# Windows (PowerShell)
Get-ChildItem $env:TEMP\.cco-*
```

**Solution:**
```bash
# Restart daemon to recreate files
cco daemon restart
```

### Daemon won't start?

**Check logs:**
```bash
cco daemon logs
```

**Common issues:**
- Port 3000 in use → Use `cco daemon start --port 3001`
- Temp directory not writable → Check permissions

### Claude Code can't find settings?

**Verify environment variable:**
```bash
# Should point to temp directory
echo $ORCHESTRATOR_SETTINGS
```

**Solution:**
```bash
# Always launch via `cco` command
cco  # Sets variables automatically

# Not directly:
claude-code  # Variables not set
```

## Summary

The temp file approach provides:

- ✅ **Simplicity**: No complex virtual filesystems
- ✅ **Cross-platform**: Works on macOS, Windows, Linux
- ✅ **Zero setup**: No system dependencies
- ✅ **Security**: Encrypted files with automatic cleanup
- ✅ **Performance**: Fast file access, no VFS overhead
- ✅ **Reliability**: Standard OS temp directory APIs
- ✅ **Easy debugging**: Just check if files exist

**Key concept**: CCO uses temporary encrypted files instead of a virtual filesystem, making it simpler, more portable, and easier to use.
