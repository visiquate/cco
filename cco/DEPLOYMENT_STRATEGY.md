# CCO Monitor Deployment Strategy

## Build & Release Pipeline

### Version Numbering

Following VisiQuate standard: **YYYY.MM.N**
- Current version: **2025.11.1**
- Build metadata: Git commit hash + build timestamp

### Cross-Platform Build Matrix

```yaml
# GitHub Actions Workflow
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          # macOS builds
          - os: macos-latest
            target: x86_64-apple-darwin
            binary: cco-monitor
          - os: macos-latest
            target: aarch64-apple-darwin
            binary: cco-monitor

          # Windows builds
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            binary: cco-monitor.exe
          - os: windows-latest
            target: aarch64-pc-windows-msvc
            binary: cco-monitor.exe

          # Linux builds (future)
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            binary: cco-monitor
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            binary: cco-monitor
```

### Binary Distribution

#### Single Binary Approach

```rust
// build.rs - Embed resources at compile time
fn main() {
    // Embed version info
    println!("cargo:rustc-env=VERSION={}", get_version());
    println!("cargo:rustc-env=GIT_COMMIT={}", get_git_commit());
    println!("cargo:rustc-env=BUILD_DATE={}", get_build_date());

    // Embed default config
    println!("cargo:rustc-env=DEFAULT_CONFIG={}",
             include_str!("../config/default.toml"));

    // Platform-specific build flags
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    }

    #[cfg(target_os = "windows")]
    {
        // Windows manifest for admin rights (if needed)
        embed_resource::compile("manifest.rc");
    }
}
```

#### Binary Sizes (Target)

| Platform | Architecture | Uncompressed | Compressed (UPX) |
|----------|-------------|--------------|------------------|
| macOS    | x86_64      | ~8 MB        | ~3 MB           |
| macOS    | aarch64     | ~8 MB        | ~3 MB           |
| Windows  | x86_64      | ~9 MB        | ~3.5 MB         |
| Windows  | aarch64     | ~9 MB        | ~3.5 MB         |

### Installation Methods

## macOS Installation

### Method 1: Homebrew (Recommended)

```bash
# Add VisiQuate tap
brew tap visiquate/tools

# Install cco-monitor
brew install cco-monitor

# Start as service (optional)
brew services start cco-monitor
```

### Method 2: Direct Download

```bash
#!/bin/bash
# install-macos.sh

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    BINARY_URL="https://github.com/visiquate/cco-monitor/releases/latest/download/cco-monitor-darwin-aarch64"
else
    BINARY_URL="https://github.com/visiquate/cco-monitor/releases/latest/download/cco-monitor-darwin-x86_64"
fi

# Download and install
echo "Downloading CCO Monitor for $ARCH..."
sudo curl -L "$BINARY_URL" -o /usr/local/bin/cco-monitor
sudo chmod +x /usr/local/bin/cco-monitor

# Create config directory
mkdir -p ~/.cco

# Initialize database
cco-monitor init

echo "Installation complete!"
```

### Method 3: LaunchAgent (Auto-start)

```xml
<!-- ~/Library/LaunchAgents/com.visiquate.cco-monitor.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.visiquate.cco-monitor</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/cco-monitor</string>
        <string>daemon</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/tmp/cco-monitor.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/cco-monitor.error.log</string>
</dict>
</plist>
```

```bash
# Load LaunchAgent
launchctl load ~/Library/LaunchAgents/com.visiquate.cco-monitor.plist
```

## Windows Installation

### Method 1: Installer (MSI)

```wix
<!-- cco-monitor.wxs - WiX installer definition -->
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="CCO Monitor"
           Language="1033" Version="2025.11.1.0"
           Manufacturer="VisiQuate"
           UpgradeCode="12345678-1234-1234-1234-123456789012">

    <Package InstallerVersion="200" Compressed="yes"
             InstallScope="perMachine" />

    <MajorUpgrade DowngradeErrorMessage="Newer version installed." />
    <MediaTemplate EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="CCO Monitor" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="CCO Monitor" />
      </Directory>
    </Directory>

    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="ProductComponent">
        <File Source="cco-monitor.exe" />

        <!-- Windows Service -->
        <ServiceInstall Id="CCOMonitorService"
                       Name="CCOMonitor"
                       DisplayName="CCO API Cost Monitor"
                       Type="ownProcess"
                       Start="auto"
                       ErrorControl="normal"
                       Account="LocalSystem" />

        <ServiceControl Id="StartService"
                       Start="install"
                       Stop="both"
                       Remove="uninstall"
                       Name="CCOMonitor" />
      </Component>
    </ComponentGroup>
  </Product>
</Wix>
```

### Method 2: PowerShell Script

```powershell
# install-windows.ps1

# Require admin rights
if (-NOT ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator"))
{
    Write-Host "Please run as Administrator!" -ForegroundColor Red
    exit 1
}

# Detect architecture
$arch = if ([System.Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }

# Download URL
$downloadUrl = "https://github.com/visiquate/cco-monitor/releases/latest/download/cco-monitor-windows-$arch.exe"
$installPath = "$env:ProgramFiles\CCO Monitor"
$exePath = "$installPath\cco-monitor.exe"

# Create directory
New-Item -ItemType Directory -Force -Path $installPath

# Download binary
Write-Host "Downloading CCO Monitor..." -ForegroundColor Green
Invoke-WebRequest -Uri $downloadUrl -OutFile $exePath

# Add to PATH
$path = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
if ($path -notlike "*$installPath*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$path;$installPath", "Machine")
}

# Create config directory
$configPath = "$env:APPDATA\cco"
New-Item -ItemType Directory -Force -Path $configPath

# Initialize database
& $exePath init

# Install as Windows Service
New-Service -Name "CCOMonitor" `
    -BinaryPathName "$exePath daemon" `
    -DisplayName "CCO API Cost Monitor" `
    -StartupType Automatic `
    -Description "Monitors Anthropic API costs in real-time"

Start-Service -Name "CCOMonitor"

Write-Host "Installation complete!" -ForegroundColor Green
```

### Method 3: Chocolatey

```powershell
# Future: Submit to Chocolatey Community Repository
choco install cco-monitor
```

## Configuration Management

### Default Paths

| Platform | Config Path | Database Path | Log Path |
|----------|------------|---------------|----------|
| macOS    | `~/.cco/config.toml` | `~/.cco/metrics.db` | `~/.cco/logs/` |
| Windows  | `%APPDATA%\cco\config.toml` | `%APPDATA%\cco\metrics.db` | `%APPDATA%\cco\logs\` |
| Linux    | `~/.config/cco/config.toml` | `~/.local/share/cco/metrics.db` | `~/.local/share/cco/logs/` |

### First-Run Setup

```rust
// Auto-initialization on first run
pub async fn initialize_first_run() -> Result<()> {
    let config_dir = get_config_dir()?;

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;

        // Create default config
        let default_config = include_str!("../config/default.toml");
        let config_path = config_dir.join("config.toml");
        fs::write(&config_path, default_config)?;

        // Initialize SQLite database
        let db_path = config_dir.join("metrics.db");
        initialize_database(&db_path).await?;

        // Show welcome message
        println!("Welcome to CCO Monitor!");
        println!("Configuration created at: {}", config_path.display());
        println!("Database initialized at: {}", db_path.display());

        // Prompt for API key (optional)
        if env::var("ANTHROPIC_API_KEY").is_err() {
            println!("\nNo API key found. You can:");
            println!("1. Set ANTHROPIC_API_KEY environment variable");
            println!("2. Add it to {}", config_path.display());
        }
    }

    Ok(())
}
```

## Update Mechanism

### Auto-Update Check

```rust
pub struct UpdateChecker {
    current_version: Version,
    update_url: String,
    check_interval: Duration,
}

impl UpdateChecker {
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        let latest = self.fetch_latest_version().await?;

        if latest.version > self.current_version {
            Ok(Some(UpdateInfo {
                version: latest.version,
                download_url: latest.download_url,
                changelog: latest.changelog,
                size: latest.size,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn download_and_install(&self, update: UpdateInfo) -> Result<()> {
        // Download to temp
        let temp_path = download_binary(&update.download_url).await?;

        // Platform-specific installation
        #[cfg(target_os = "macos")]
        install_macos_update(temp_path)?;

        #[cfg(target_os = "windows")]
        install_windows_update(temp_path)?;

        Ok(())
    }
}
```

### Update Notification in TUI

```
┌─ Update Available ───────────────────────────────────────┐
│                                                          │
│  New version available: v2025.11.2                      │
│                                                          │
│  Changes:                                                │
│  • Fixed cost calculation for Opus model                │
│  • Added export to Excel format                         │
│  • Improved TUI performance                             │
│                                                          │
│  Download size: 3.2 MB                                  │
│                                                          │
│  [U]pdate Now  [L]ater  [S]kip This Version            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Deployment Testing

### Test Matrix

```yaml
# .github/workflows/test-deployment.yml
test-deployment:
  strategy:
    matrix:
      os: [macos-12, macos-13, windows-2019, windows-2022]

  steps:
    - name: Test Installation
      run: |
        # Run installation script
        ./scripts/install-${{ matrix.os }}.sh

        # Verify binary works
        cco-monitor --version

        # Test initialization
        cco-monitor init

        # Test TUI launch (headless)
        timeout 5 cco-monitor || true

        # Verify database created
        test -f ~/.cco/metrics.db
```

### Smoke Tests

```rust
#[cfg(test)]
mod deployment_tests {
    #[test]
    fn test_version_output() {
        let output = Command::new("./target/release/cco-monitor")
            .arg("--version")
            .output()
            .expect("Failed to run binary");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout)
            .contains("2025.11"));
    }

    #[test]
    fn test_help_output() {
        let output = Command::new("./target/release/cco-monitor")
            .arg("--help")
            .output()
            .expect("Failed to run binary");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout)
            .contains("Claude API Cost Monitor"));
    }

    #[test]
    fn test_init_command() {
        let temp_dir = tempdir().unwrap();
        env::set_var("CCO_CONFIG_PATH", temp_dir.path());

        let output = Command::new("./target/release/cco-monitor")
            .arg("init")
            .output()
            .expect("Failed to run init");

        assert!(output.status.success());
        assert!(temp_dir.path().join("config.toml").exists());
        assert!(temp_dir.path().join("metrics.db").exists());
    }
}
```

## Release Checklist

### Pre-Release

- [ ] Version bump in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Test on all target platforms
- [ ] Update documentation
- [ ] Create git tag

### Release Process

```bash
# 1. Create and push tag
git tag -a v2025.11.1 -m "Release version 2025.11.1"
git push origin v2025.11.1

# 2. GitHub Actions builds all binaries

# 3. Create GitHub Release
gh release create v2025.11.1 \
  --title "CCO Monitor v2025.11.1" \
  --notes-file CHANGELOG.md \
  ./dist/cco-monitor-*

# 4. Update Homebrew formula
brew bump-formula-pr \
  --url=https://github.com/visiquate/cco-monitor/archive/v2025.11.1.tar.gz \
  cco-monitor

# 5. Update installation scripts with new version
```

### Post-Release

- [ ] Verify GitHub Release assets
- [ ] Test installation methods
- [ ] Update documentation site
- [ ] Announce release
- [ ] Monitor for issues

## Rollback Strategy

### Version Pinning

```toml
# config.toml - Allow version pinning
[update]
channel = "stable"  # or "beta", "nightly"
auto_update = false
pin_version = "2025.11.1"  # Don't update past this
```

### Rollback Command

```bash
# Rollback to previous version
cco-monitor rollback

# Install specific version
cco-monitor install --version 2025.10.5
```

## Monitoring Deployment Health

### Telemetry (Optional, Opt-in)

```rust
pub struct Telemetry {
    enabled: bool,
    anonymous_id: Uuid,
    endpoint: String,
}

impl Telemetry {
    pub async fn report_health(&self) {
        if !self.enabled { return; }

        let report = HealthReport {
            version: env!("VERSION"),
            platform: env::consts::OS,
            arch: env::consts::ARCH,
            uptime: self.get_uptime(),
            error_count: self.get_error_count(),
            // No personal data
        };

        // Fire and forget
        let _ = self.send_report(report).await;
    }
}
```

---

*Strategy Version: 1.0*
*Date: November 17, 2024*
*Status: READY FOR IMPLEMENTATION*