# CCO PowerShell Installer Script
# One-line installation: iwr -useb https://raw.githubusercontent.com/brentley/cco-releases/main/install.ps1 | iex

[CmdletBinding()]
param(
    [string]$Version = "",
    [ValidateSet("stable", "beta", "nightly")]
    [string]$Channel = "stable",
    [string]$InstallDir = "$env:LOCALAPPDATA\cco\bin",
    [switch]$Force,
    [switch]$Help
)

# Configuration
$GitHubOrg = "brentley"
$GitHubRepo = "cco-releases"
$ManifestUrl = "https://raw.githubusercontent.com/$GitHubOrg/$GitHubRepo/main/version-manifest.json"
$ConfigDir = "$env:APPDATA\cco"

# Colors for output
function Write-Info { Write-Host "[INFO]" -ForegroundColor Blue -NoNewline; Write-Host " $args" }
function Write-Success { Write-Host "[SUCCESS]" -ForegroundColor Green -NoNewline; Write-Host " $args" }
function Write-Warning { Write-Host "[WARNING]" -ForegroundColor Yellow -NoNewline; Write-Host " $args" }
function Write-ErrorMsg { Write-Host "[ERROR]" -ForegroundColor Red -NoNewline; Write-Host " $args" -ForegroundColor Red; exit 1 }

# Show help
if ($Help) {
    @"
CCO (Claude Code Orchestra) PowerShell Installer

Usage:
    iwr -useb $ManifestUrl | iex
    .\install.ps1 [options]

Options:
    -Version <string>    Install specific version (default: latest)
    -Channel <string>    Release channel: stable, beta, nightly (default: stable)
    -InstallDir <path>   Installation directory (default: $env:LOCALAPPDATA\cco\bin)
    -Force              Force reinstallation
    -Help               Show this help message

Examples:
    # Install latest stable version
    iwr -useb $ManifestUrl | iex

    # Install specific version
    .\install.ps1 -Version 0.3.0

    # Install beta channel
    .\install.ps1 -Channel beta

    # Force reinstall
    .\install.ps1 -Force
"@
    exit 0
}

# Banner
Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "       CCO (Claude Code Orchestra)       " -ForegroundColor Cyan
Write-Host "      PowerShell Installation Script      " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Check PowerShell version
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-ErrorMsg "PowerShell 5.0 or higher is required. Current version: $($PSVersionTable.PSVersion)"
}

# Detect architecture
function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "x64" { return "x86_64" }
        "x86" { return "x86" }
        "ARM64" { return "arm64" }
        default { Write-ErrorMsg "Unsupported architecture: $arch" }
    }
}

$Architecture = Get-Architecture
$Platform = "windows-$Architecture"
Write-Info "Detected platform: $Platform"

# Check if running as administrator (optional, but recommended)
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")
if (-not $IsAdmin) {
    Write-Warning "Not running as administrator. Some features may not work correctly."
}

# Create installation directory
if (!(Test-Path $InstallDir)) {
    Write-Info "Creating installation directory: $InstallDir"
    try {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    } catch {
        Write-ErrorMsg "Failed to create installation directory: $_"
    }
}

# Fetch version manifest
Write-Info "Fetching version manifest..."
try {
    $ManifestContent = Invoke-RestMethod -Uri $ManifestUrl -UseBasicParsing
} catch {
    Write-ErrorMsg "Failed to fetch version manifest: $_"
}

# Determine version to install
if ([string]::IsNullOrEmpty($Version)) {
    $Version = $ManifestContent.latest.$Channel
    if ([string]::IsNullOrEmpty($Version)) {
        Write-ErrorMsg "Failed to determine latest version for channel: $Channel"
    }
    Write-Info "Latest $Channel version: $Version"
} else {
    Write-Info "Installing specified version: $Version"
}

# Check for existing installation
$ExePath = Join-Path $InstallDir "cco.exe"
if ((Test-Path $ExePath) -and (-not $Force)) {
    try {
        $CurrentVersion = & $ExePath --version 2>$null | Select-String -Pattern '\d+\.\d+\.\d+' | ForEach-Object { $_.Matches[0].Value }

        if ($CurrentVersion -eq $Version) {
            Write-Success "CCO v$Version is already installed at $ExePath"
            exit 0
        } else {
            Write-Info "Updating CCO from v$CurrentVersion to v$Version"
        }
    } catch {
        Write-Warning "Could not determine installed version"
    }
}

# Get download information
$VersionInfo = $ManifestContent.versions.$Version
if ($null -eq $VersionInfo) {
    Write-ErrorMsg "Version $Version not found in manifest"
}

$PlatformInfo = $VersionInfo.platforms.$Platform
if ($null -eq $PlatformInfo) {
    Write-ErrorMsg "Platform $Platform not supported for version $Version"
}

$DownloadUrl = $PlatformInfo.url
$ExpectedSHA256 = $PlatformInfo.sha256
$FileSize = $PlatformInfo.compressed_size

Write-Info "Download URL: $DownloadUrl"
if ($FileSize) {
    Write-Info "File size: $([math]::Round($FileSize / 1MB, 2)) MB"
}

# Create temporary directory
$TempDir = Join-Path $env:TEMP "cco-install-$(Get-Random)"
try {
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
} catch {
    Write-ErrorMsg "Failed to create temporary directory: $_"
}

# Download binary
$ZipPath = Join-Path $TempDir "cco.zip"
Write-Info "Downloading CCO v$Version for $Platform..."

try {
    $ProgressPreference = 'SilentlyContinue'  # Disable progress bar for speed
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
    $ProgressPreference = 'Continue'
} catch {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-ErrorMsg "Failed to download CCO: $_"
}

# Verify checksum
if ($ExpectedSHA256) {
    Write-Info "Verifying download integrity..."
    try {
        $ActualSHA256 = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
        if ($ActualSHA256 -ne $ExpectedSHA256.ToLower()) {
            Remove-Item -Path $TempDir -Recurse -Force
            Write-ErrorMsg "Checksum verification failed!`nExpected: $ExpectedSHA256`nActual:   $ActualSHA256"
        }
        Write-Success "Checksum verified"
    } catch {
        Write-Warning "Could not verify checksum: $_"
    }
} else {
    Write-Warning "No checksum available for verification"
}

# Extract binary
Write-Info "Extracting CCO binary..."
try {
    # Use built-in extraction for Windows
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::ExtractToDirectory($ZipPath, $TempDir)

    # Find the cco.exe file
    $ExtractedExe = Get-ChildItem -Path $TempDir -Filter "cco.exe" -Recurse | Select-Object -First 1
    if ($null -eq $ExtractedExe) {
        throw "cco.exe not found in archive"
    }
} catch {
    Remove-Item -Path $TempDir -Recurse -Force
    Write-ErrorMsg "Failed to extract archive: $_"
}

# Backup existing installation
if (Test-Path $ExePath) {
    $BackupPath = "$ExePath.backup"
    Write-Info "Backing up existing installation to $BackupPath"
    try {
        Move-Item -Path $ExePath -Destination $BackupPath -Force
    } catch {
        Write-Warning "Could not backup existing installation: $_"
    }
}

# Install binary
Write-Info "Installing CCO to $ExePath"
try {
    Move-Item -Path $ExtractedExe.FullName -Destination $ExePath -Force

    # Remove backup if installation succeeded
    if (Test-Path "$ExePath.backup") {
        Remove-Item -Path "$ExePath.backup" -Force -ErrorAction SilentlyContinue
    }

    Write-Success "CCO v$Version installed successfully!"
} catch {
    # Restore backup on failure
    if (Test-Path "$ExePath.backup") {
        Move-Item -Path "$ExePath.backup" -Destination $ExePath -Force
    }
    Remove-Item -Path $TempDir -Recurse -Force
    Write-ErrorMsg "Failed to install CCO: $_"
}

# Clean up
Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue

# Update PATH if necessary
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Info "Adding $InstallDir to user PATH"
    try {
        $NewPath = "$InstallDir;$UserPath"
        [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
        $env:PATH = "$InstallDir;$env:PATH"
        Write-Success "PATH updated. Please restart your terminal for changes to take effect."
    } catch {
        Write-Warning "Could not update PATH automatically. Please add $InstallDir to your PATH manually."
    }
} else {
    Write-Info "Installation directory already in PATH"
}

# Create default configuration
if (!(Test-Path $ConfigDir)) {
    Write-Info "Creating configuration directory: $ConfigDir"
    try {
        New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    } catch {
        Write-Warning "Could not create configuration directory: $_"
    }
}

$ConfigPath = Join-Path $ConfigDir "config.toml"
if (!(Test-Path $ConfigPath)) {
    Write-Info "Creating default configuration..."
    try {
        @'
# CCO Configuration

[updates]
enabled = true
auto_install = false
check_interval = "daily"
channel = "stable"
notify_on_update = true
verify_signatures = true

[updates.schedule]
# Automatically populated by CCO

[telemetry]
enabled = false
'@ | Out-File -FilePath $ConfigPath -Encoding UTF8
        Write-Success "Default configuration created at $ConfigPath"
    } catch {
        Write-Warning "Could not create default configuration: $_"
    }
}

# Verify installation
Write-Info "Verifying installation..."
try {
    $InstalledVersion = & $ExePath --version 2>$null | Select-String -Pattern '\d+\.\d+\.\d+' | ForEach-Object { $_.Matches[0].Value }
    if ($InstalledVersion) {
        Write-Success "CCO v$InstalledVersion is ready to use!"

        Write-Host ""
        Write-Host "Getting started:" -ForegroundColor Cyan
        Write-Host "  1. Restart your terminal or refresh PATH"
        Write-Host "  2. Run: cco --help"
        Write-Host "  3. Configure: cco config show"
        Write-Host ""
        Write-Host "For documentation, visit: https://github.com/$GitHubOrg/$GitHubRepo" -ForegroundColor Blue
    } else {
        Write-ErrorMsg "Installation verification failed. CCO binary may be corrupted."
    }
} catch {
    Write-ErrorMsg "Installation verification failed: $_"
}

Write-Host ""
Write-Success "Installation complete!"
Write-Host ""

# Prompt to run CCO
$RunNow = Read-Host "Would you like to run 'cco --help' now? (y/n)"
if ($RunNow -eq 'y' -or $RunNow -eq 'Y') {
    & $ExePath --help
}