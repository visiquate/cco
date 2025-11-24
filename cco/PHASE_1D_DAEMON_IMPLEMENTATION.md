# Phase 1d: Cross-Platform Daemon Lifecycle Management

## Overview

Phase 1d implements comprehensive cross-platform daemon lifecycle management for CCO (Claude Code Orchestra). This phase provides complete daemon start/stop/restart functionality, service installation, configuration management, and logging for both macOS and Linux systems.

## Completed Components

### 1. Core Daemon Module (`src/daemon/mod.rs`)
- **Module organization** with submodules for config, lifecycle, and service management
- **Path utilities** for daemon directories and file locations
- **Automated directory creation** for `~/.cco` and related subdirectories
- **Entry point** for daemon functionality

### 2. Configuration Management (`src/daemon/config.rs`)
- **DaemonConfig struct** with comprehensive configuration options:
  - Port, host, log level
  - Log rotation settings (size-based rotation, max files)
  - Database URL, cache size, cache TTL
  - Auto-start and health check settings
- **Validation** - comprehensive config validation with detailed error messages
- **TOML persistence** - save/load configuration from `~/.cco/config.toml`
- **Dynamic updates** - set/get individual configuration values
- **Default values** - sensible defaults for all configuration options

**Tests: 12 tests covering**
- Default configuration
- Validation (port, log level, cache settings)
- TOML save/load roundtrips
- Set/get operations
- Missing file handling

### 3. Daemon Lifecycle Management (`src/daemon/lifecycle.rs`)
- **DaemonManager struct** - core daemon control class
- **Start operation** - spawn daemon process with proper PID tracking
- **Stop operation** - graceful SIGTERM with fallback to SIGKILL
- **Restart operation** - stop and start with delay
- **Status checking** - get current daemon status and PID info
- **Process management**:
  - PID file creation and management
  - Stale PID file cleanup
  - Process signal handling (SIGTERM, SIGKILL)
  - Graceful shutdown with timeout

**Tests: 7 tests covering**
- Manager creation
- PID file serialization
- Process running detection
- Status checking
- Stale PID cleanup

### 4. Platform-Specific Service Management (`src/daemon/service/`)

#### macOS LaunchAgent (`src/daemon/service/macos.rs`)
- **MacOSService struct** - macOS service implementation
- **LaunchAgent creation** - generates plist files at `~/Library/LaunchAgents/com.anthropic.cco.daemon.plist`
- **Service lifecycle**:
  - Install - creates plist and loads with launchctl
  - Uninstall - unloads and removes plist
  - Enable/Disable - loads/unloads service
  - Is installed - checks plist existence
- **Automatic startup** - RunAtLoad=true in plist
- **Restart policy** - KeepAlive with SuccessfulExit=false
- **Log redirection** - stdout/stderr to daemon log file

**Tests: 3 tests covering**
- Service creation
- Plist generation with proper XML structure
- Service name constants

#### Linux systemd (`src/daemon/service/linux.rs`)
- **LinuxService struct** - Linux systemd implementation
- **Service unit creation** - generates systemd units at `~/.config/systemd/user/cco-daemon.service`
- **Service lifecycle**:
  - Install - creates unit file and enables with systemctl
  - Uninstall - disables and removes unit file
  - Enable/Disable - manages systemctl --user settings
  - Is installed - checks unit file existence
- **Automatic restart** - Restart=always, RestartSec=10
- **Log handling** - StandardOutput/StandardError to daemon log

**Tests: 3 tests covering**
- Service creation
- Unit generation with proper INI structure
- Install section configuration

### 5. CLI Command Integration (`src/main.rs`)
- **Daemon command** with subcommands:
  - `cco daemon start` - start daemon with port/host/cache options
  - `cco daemon stop` - stop running daemon
  - `cco daemon restart` - restart with new settings
  - `cco daemon status` - display daemon status
  - `cco daemon logs` - view daemon logs (static or follow mode)
  - `cco daemon install` - install as system service
  - `cco daemon uninstall` - remove from system service
  - `cco daemon enable` - enable service startup on boot
  - `cco daemon disable` - disable service startup
  - `cco daemon run` - run daemon in foreground (internal use)

## Key Features

### Configuration Management
- TOML-based configuration at `~/.cco/config.toml`
- Command-line overrides for port, host, cache settings
- Sensible defaults for all options
- Runtime validation with clear error messages

### Process Management
- Automatic PID file creation and tracking
- Graceful shutdown (10-second timeout with SIGTERM)
- Force shutdown with SIGKILL if graceful fails
- Stale PID file cleanup
- Single-instance prevention (checked via PID file)

### Cross-Platform Support
- **macOS**: LaunchAgent with automatic startup
- **Linux**: systemd user service with automatic startup
- Platform detection at compile-time
- Same API across both platforms

### Logging
- File-based logging at `~/.cco/daemon.log`
- Log rotation support (configurable size)
- Maximum retained files (configurable)
- Follow mode (tail -f style) in CLI

## File Structure

```
cco/src/daemon/
├── mod.rs                 # Module definition and utilities
├── config.rs              # Configuration management
├── lifecycle.rs           # Daemon lifecycle operations
└── service/
    ├── mod.rs             # Service trait and platform detection
    ├── macos.rs           # macOS LaunchAgent implementation
    └── linux.rs           # Linux systemd implementation
```

## Commands & Usage

### Start Daemon
```bash
cco daemon start --port 3000 --host 127.0.0.1
```

### Stop Daemon
```bash
cco daemon stop
```

### Restart Daemon
```bash
cco daemon restart --port 3000
```

### Check Status
```bash
cco daemon status
```

### View Logs
```bash
cco daemon logs                 # Show last 50 lines
cco daemon logs -n 100         # Show last 100 lines
cco daemon logs --follow       # Follow logs (tail -f)
```

### Install as Service
```bash
cco daemon install             # Creates and enables service
```

### Uninstall Service
```bash
cco daemon uninstall          # Removes service
```

### Enable/Disable Service
```bash
cco daemon enable              # Enable autostart on boot
cco daemon disable             # Disable autostart
```

## Configuration File Example

```toml
port = 3000
host = "127.0.0.1"
log_level = "info"
log_rotation_size = 10485760
log_max_files = 5
database_url = "sqlite://analytics.db"
cache_size = 1073741824
cache_ttl = 3600
auto_start = true
health_checks = true
health_check_interval = 30
```

## Test Coverage

### Total Tests: 51
- **Unit Tests (lib)**: 19 tests
  - Configuration: 12 tests
  - Lifecycle: 7 tests
- **Integration Tests**: 32 tests
  - Configuration: 20 tests
  - Service (generic): 2 tests
  - macOS-specific: 3 tests
  - Linux-specific: 3 tests
  - Daemon utilities: 4 tests

### Test Categories
1. **Configuration Tests**
   - Default values
   - Validation (ports, levels, sizes)
   - TOML serialization/deserialization
   - Set/get operations
   - Missing file handling

2. **Lifecycle Tests**
   - Manager creation
   - PID file management
   - Process detection
   - Status checking
   - Cleanup operations

3. **Service Tests**
   - Service creation
   - plist/unit generation
   - XML/INI structure validation
   - Service installation flow

## Platform Support

### macOS
- **Minimum**: macOS 10.12 (Sierra)
- **Service Location**: `~/Library/LaunchAgents/com.anthropic.cco.daemon.plist`
- **Startup**: User login (per LaunchAgent behavior)
- **Commands**: `launchctl load/unload`

### Linux
- **Minimum**: systemd-based distributions
- **Service Location**: `~/.config/systemd/user/cco-daemon.service`
- **Startup**: User session (WantedBy=default.target)
- **Commands**: `systemctl --user`

## Directory Structure

```
~/.cco/
├── daemon.pid            # Current daemon PID and metadata
├── daemon.log            # Daemon log file (rotated)
├── config.toml           # Daemon configuration
└── pids/                 # Legacy PID directory (if created by other tools)
    └── cco-*.pid
```

## Implementation Details

### Daemon Manager
The `DaemonManager` struct provides the core daemon control API:
- Manages all lifecycle operations
- Handles process signals (SIGTERM, SIGKILL)
- Manages PID files and cleanup
- Provides status information

### Service Manager Trait
The `ServiceManager` trait abstracts platform-specific operations:
- Install/uninstall services
- Enable/disable services
- Check installation status

Platform implementations:
- `MacOSService` - LaunchAgent for macOS
- `LinuxService` - systemd for Linux

### Configuration Validation
Comprehensive validation ensures:
- Port is in valid range (1-65535, non-zero)
- Log level is valid (debug, info, warn, error)
- Cache settings are non-zero
- Log rotation settings are valid
- All paths are accessible/creatable

## Future Enhancements

1. **Windows Support** - Service manager for Windows
2. **Health Checks** - Periodic health check endpoint
3. **Metrics Export** - Expose metrics to external systems
4. **Automatic Restart** - Crash detection and restart
5. **Clustering** - Multi-instance coordination
6. **Custom Logging** - Pluggable logging backends
7. **Configuration Hot-Reload** - Watch for config changes

## Summary

Phase 1d successfully implements a complete daemon lifecycle management system for CCO with:
- Full cross-platform support (macOS, Linux)
- Comprehensive configuration management
- Proper process lifecycle handling
- System service integration
- Extensive test coverage (51 tests, all passing)
- Clean CLI interface
- Production-ready code

The implementation follows Rust best practices with proper error handling, resource management, and safety guarantees.
