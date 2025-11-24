# CCO Installer for Windows PowerShell
# Usage: irm https://raw.githubusercontent.com/USER/cc-orchestra/main/install.ps1 | iex
#
# This script downloads and installs the CCO binary to $env:ProgramFiles\CCO

$ErrorActionPreference = "Stop"

# Configuration
$REPO = "USER/cc-orchestra"
$INSTALL_DIR = "$env:ProgramFiles\CCO"
$BINARY_NAME = "cco.exe"

# Colors for output
function Write-Info($message) {
    Write-Host "ℹ️  $message" -ForegroundColor Blue
}

function Write-Success($message) {
    Write-Host "✅ $message" -ForegroundColor Green
}

function Write-Warn($message) {
    Write-Host "⚠️  $message" -ForegroundColor Yellow
}

function Write-Fail($message) {
    Write-Host "❌ $message" -ForegroundColor Red
}

# Header
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host "   CCO Installer - Claude Code Orchestra" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Warn "Not running as Administrator"
    Write-Info "Installation will continue, but you may need to provide admin credentials"
}

# Detect architecture
$ARCH = $env:PROCESSOR_ARCHITECTURE
Write-Info "Detected architecture: $ARCH"

if ($ARCH -ne "AMD64") {
    Write-Fail "Unsupported architecture: $ARCH"
    Write-Info "Only x86_64 (AMD64) is currently supported for Windows"
    exit 1
}

$PLATFORM = "windows-x86_64"
Write-Success "Platform identified: $PLATFORM"

# Get latest version from GitHub API
Write-Info "Fetching latest release version..."
try {
    $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
    $VERSION = $releaseInfo.tag_name -replace '^v', ''
} catch {
    Write-Fail "Failed to fetch latest version: $_"
    Write-Info "You can manually download from: https://github.com/$REPO/releases"
    exit 1
}

Write-Success "Latest version: $VERSION"

# Construct download URL
$ARCHIVE_NAME = "cco-v$VERSION-$PLATFORM.zip"
$DOWNLOAD_URL = "https://github.com/$REPO/releases/download/v$VERSION/$ARCHIVE_NAME"
$CHECKSUM_URL = "https://github.com/$REPO/releases/download/v$VERSION/cco-v$VERSION-$PLATFORM.sha256"

Write-Info "Download URL: $DOWNLOAD_URL"

# Create temporary directory
$TMP_DIR = Join-Path $env:TEMP "cco-install-$(Get-Random)"
New-Item -ItemType Directory -Force -Path $TMP_DIR | Out-Null

try {
    Push-Location $TMP_DIR

    # Download binary archive
    Write-Info "Downloading CCO v$VERSION..."
    try {
        Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $ARCHIVE_NAME -UseBasicParsing
    } catch {
        Write-Fail "Download failed: $_"
        Write-Info "Please check your internet connection and try again"
        exit 1
    }

    Write-Success "Downloaded $ARCHIVE_NAME"

    # Download and verify checksum
    Write-Info "Verifying checksum..."
    try {
        Invoke-WebRequest -Uri $CHECKSUM_URL -OutFile "checksum.txt" -UseBasicParsing
        $EXPECTED_CHECKSUM = (Get-Content "checksum.txt").Split()[0]

        $hash = Get-FileHash -Path $ARCHIVE_NAME -Algorithm SHA256
        $ACTUAL_CHECKSUM = $hash.Hash.ToLower()

        if ($EXPECTED_CHECKSUM -eq $ACTUAL_CHECKSUM) {
            Write-Success "Checksum verified"
        } else {
            Write-Fail "Checksum mismatch!"
            Write-Fail "Expected: $EXPECTED_CHECKSUM"
            Write-Fail "Got: $ACTUAL_CHECKSUM"
            exit 1
        }
    } catch {
        Write-Warn "Checksum verification skipped: $_"
    }

    # Extract archive
    Write-Info "Extracting archive..."
    Expand-Archive -Path $ARCHIVE_NAME -DestinationPath "." -Force

    if (-not (Test-Path $BINARY_NAME)) {
        Write-Fail "Binary not found in archive"
        exit 1
    }

    # Test binary
    Write-Info "Testing binary..."
    try {
        $versionOutput = & ".\$BINARY_NAME" version 2>&1
        Write-Success "Binary test passed"
    } catch {
        Write-Fail "Binary test failed: $_"
        exit 1
    }

    # Create installation directory
    Write-Info "Creating installation directory..."
    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
    }

    # Install binary
    Write-Info "Installing to $INSTALL_DIR\$BINARY_NAME..."
    try {
        Copy-Item -Path $BINARY_NAME -Destination "$INSTALL_DIR\$BINARY_NAME" -Force
    } catch {
        Write-Fail "Installation failed: $_"
        Write-Info "You may need to run this script as Administrator"
        exit 1
    }

    Write-Success "CCO installed successfully!"

    # Add to PATH if not already present
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$INSTALL_DIR*") {
        Write-Info "Adding to PATH..."
        [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$INSTALL_DIR", "User")
        $env:PATH = "$env:PATH;$INSTALL_DIR"
        Write-Success "Added to PATH (restart terminal for changes to take effect)"
    }

    # Verify installation
    Write-Info "Verifying installation..."
    $ccoPath = Get-Command cco -ErrorAction SilentlyContinue
    if ($ccoPath) {
        Write-Success "Installation verified"
    } else {
        Write-Warn "Installation succeeded but 'cco' not found in PATH"
        Write-Info "Please restart your terminal or add $INSTALL_DIR to your PATH"
    }

} finally {
    Pop-Location
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
}

# Display version
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Success "Installation Complete!"
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""
Write-Host "Installed version:"
try {
    & cco version
} catch {
    Write-Host $versionOutput
}
Write-Host ""

# Post-installation instructions
Write-Host "📝 Next Steps:" -ForegroundColor Yellow
Write-Host ""
Write-Host "1. Set your Anthropic API key:"
Write-Host "   `$env:ANTHROPIC_API_KEY = 'sk-ant-...'"
Write-Host ""
Write-Host "2. Start CCO:"
Write-Host "   cco run"
Write-Host ""
Write-Host "3. View dashboard:"
Write-Host "   Open http://localhost:3000 in your browser"
Write-Host ""
Write-Host "4. (Optional) Install as Windows Service:"
Write-Host "   Run PowerShell as Administrator, then:"
Write-Host "   sc create CCO binPath= `"$INSTALL_DIR\$BINARY_NAME run`" start= auto"
Write-Host "   sc start CCO"
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""
Write-Host "For help and documentation:"
Write-Host "  cco --help"
Write-Host "  https://github.com/$REPO"
Write-Host ""
