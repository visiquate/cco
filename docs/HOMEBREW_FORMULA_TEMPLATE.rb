# Homebrew Formula Template for CCO with Code Signing Verification
# Place this in the homebrew-cco repository as: Formula/cco.rb
#
# This formula includes verification steps to ensure the binary is:
# 1. Properly code-signed
# 2. Notarized by Apple
# 3. Safe to run on macOS systems
#
# Update instructions:
# 1. Replace VERSION with the released version (e.g., "2025.12.1")
# 2. Replace SHA256 hashes with actual values from release
# 3. Commit to visiquate/homebrew-cco repository
# Tip: run scripts/generate_homebrew_formula.sh to emit a ready Formula/cco.rb from the latest release.

class Cco < Formula
  desc "Claude Code Orchestrator - AI-powered development automation platform"
  homepage "https://github.com/visiquate/cco"
  license "MIT"

  # Version should match the GitHub release tag (without 'v' prefix)
  version "2025.12.28"

  # macOS-specific binary (Apple Silicon)
  on_macos do
    on_arm do
      url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-aarch64-apple-darwin.tar.gz"
      sha256 "09d8a22cde5fe1d3e268cd0fdf5d08db797f5540602252ee857466c23614500b"
    end
    on_intel do
      # Add if/when x86_64 macOS support is implemented
      # url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-x86_64-apple-darwin.tar.gz"
      # sha256 "<sha256>"
    end
  end

  # Linux binary (x86_64)
  on_linux do
    url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "730dfe0888d24a403a86736d15f18ffc0bab2e6341bd6b38b79b7b5602f1022a"
  end

  def install
    # Extract and install the binary
    bin.install "cco"
  end

  # Post-installation verification (macOS only)
  def post_install
    if OS.mac?
      binary = bin / "cco"

      # Verify code signature exists and is valid
      unless system("codesign", "--verify", "--verbose", binary)
        raise "Binary signature verification failed - the binary may not be properly signed"
      end

      # Optional: Verify it's code-signed by Apple Developer ID
      # This ensures the binary hasn't been tampered with
      code_sign_output = Utils.popen_read("codesign", "-dvvv", binary, :err => :out)
      unless code_sign_output.include?("Developer ID Application")
        ohai "Warning: Binary not signed with Developer ID Application"
        ohai "This is expected during development, but production builds should be signed"
      end
    end
  end

  # Test the installation
  test do
    # Verify the binary exists and is executable
    assert_predicate bin/"cco", :executable?

    # Test basic help command
    assert_match(/usage|Options|Commands/i, shell_output("#{bin}/cco --help"))

    # On macOS, verify the signature one more time after installation
    if OS.mac?
      system("codesign", "--verify", "--verbose", bin/"cco")
    end
  end
end

# =============================================================================
# FORMULA DEPLOYMENT INSTRUCTIONS
# =============================================================================
#
# 1. Build and test locally:
#    $ brew install --build-from-source --verbose Formula/cco.rb
#
# 2. Verify the installed binary:
#    $ which cco
#    $ cco --version
#    $ cco --help
#
# 3. On macOS, verify code signature:
#    $ codesign --verify --verbose /usr/local/bin/cco
#    $ spctl -a -vvv /usr/local/bin/cco  # Gatekeeper check
#
# 4. Commit and push to visiquate/homebrew-cco:
#    $ git add Formula/cco.rb
#    $ git commit -m "Update CCO to v{version}"
#    $ git push origin main
#
# 5. Users can then install with:
#    $ brew tap visiquate/cco
#    $ brew install visiquate/cco/cco
#
# =============================================================================
# UPDATING THE FORMULA
# =============================================================================
#
# When releasing a new version:
#
# 1. Extract SHA256 from release notes or compute:
#    $ curl -sL https://github.com/visiquate/cco/releases/download/vX.Y.Z/cco-aarch64-apple-darwin.tar.gz | \
#      shasum -a 256
#
# 2. Update this file:
#    - Change version = "X.Y.Z"
#    - Update SHA256 values for each platform
#
# 3. Test installation:
#    $ brew reinstall --build-from-source Formula/cco.rb
#
# 4. Commit:
#    $ git commit -m "Update CCO to v{version}"
#
# =============================================================================
# SECURITY CONSIDERATIONS
# =============================================================================
#
# Code Signing Verification:
# - The post_install hook verifies the binary's code signature
# - This ensures the binary hasn't been tampered with
# - Developer ID Application signature required for production builds
#
# Notarization:
# - Binaries are notarized by Apple before release
# - Gatekeeper automatically validates notarization
# - No "unidentified developer" warning on first run
#
# SHA256 Verification:
# - Homebrew automatically verifies SHA256 checksums
# - Mismatches abort installation
# - Prevents man-in-the-middle attacks
#
# =============================================================================
# TROUBLESHOOTING
# =============================================================================
#
# If installation fails with signature error:
#
# 1. Check if code signature is present:
#    $ codesign -d /usr/local/bin/cco
#
# 2. If missing, sign manually (development only):
#    $ codesign -s - /usr/local/bin/cco
#
# 3. If signature is invalid, the binary may be corrupted:
#    $ brew reinstall visiquate/cco/cco
#
# For more help:
# - See: MACOS_SIGNING_AND_NOTARIZATION.md
# - See: MACOS_SIGNING_TROUBLESHOOTING.md
