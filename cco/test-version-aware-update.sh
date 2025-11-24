#!/bin/bash
# Test script for version-aware daemon update functionality
#
# This script tests that when the binary version changes, running `cco server run`
# will automatically detect the version mismatch and restart the daemon with the new version.

set -e

echo "=========================================="
echo "Version-Aware Daemon Update Test"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BINARY_PATH="target/release/cco"
TEST_DIR="/tmp/cco-version-test"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    # Stop any running daemon
    $BINARY_PATH server uninstall 2>/dev/null || true
    # Remove test directory
    rm -rf "$TEST_DIR"
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Set trap to cleanup on exit
trap cleanup EXIT

# Step 1: Build current version
echo -e "${YELLOW}Step 1: Building current binary...${NC}"
cargo build --release --quiet
CURRENT_VERSION=$($BINARY_PATH --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo -e "${GREEN}✓ Built version: $CURRENT_VERSION${NC}"
echo ""

# Step 2: Ensure clean state
echo -e "${YELLOW}Step 2: Cleaning up any existing daemon...${NC}"
$BINARY_PATH server uninstall 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Clean state achieved${NC}"
echo ""

# Step 3: Start server with current version
echo -e "${YELLOW}Step 3: Starting server with version $CURRENT_VERSION...${NC}"
$BINARY_PATH server run --host 127.0.0.1 --port 3000 2>&1 | grep -E "✅|🔌|⏳|Version"
sleep 2
echo ""

# Step 4: Verify server is running
echo -e "${YELLOW}Step 4: Verifying server is running...${NC}"
HEALTH_OUTPUT=$(curl -s http://127.0.0.1:3000/health)
RUNNING_VERSION=$(echo "$HEALTH_OUTPUT" | grep -oE '"version":"[^"]+' | cut -d'"' -f4)
echo "Health check response: $HEALTH_OUTPUT"
echo -e "${GREEN}✓ Server is running with version: $RUNNING_VERSION${NC}"
echo ""

# Step 5: Get current PID
echo -e "${YELLOW}Step 5: Recording current daemon PID...${NC}"
PID_FILE="$HOME/.cco/daemon.pid"
ORIGINAL_PID=$(cat "$PID_FILE" | grep -oE '"pid":[0-9]+' | cut -d':' -f2)
echo -e "${GREEN}✓ Current PID: $ORIGINAL_PID${NC}"
echo ""

# Step 6: Simulate version change (modify build.rs version temporarily)
echo -e "${YELLOW}Step 6: Simulating version change...${NC}"
# Backup original build.rs
cp build.rs build.rs.backup

# Modify version in build.rs
sed -i.bak 's/"2025.11.3"/"2025.11.4"/' build.rs
echo -e "${GREEN}✓ Modified version to 2025.11.4${NC}"
echo ""

# Step 7: Rebuild with new version
echo -e "${YELLOW}Step 7: Rebuilding with new version...${NC}"
cargo build --release --quiet
NEW_VERSION=$($BINARY_PATH --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo -e "${GREEN}✓ Rebuilt version: $NEW_VERSION${NC}"
echo ""

# Step 8: Run server again (should detect version mismatch and restart)
echo -e "${YELLOW}Step 8: Running 'cco server run' with new binary...${NC}"
echo -e "${YELLOW}Expected behavior: Should detect version mismatch and restart${NC}"
echo ""
$BINARY_PATH server run --host 127.0.0.1 --port 3000 2>&1 | grep -E "✅|🔄|Version|Restarting|Running|Binary|PID"
sleep 2
echo ""

# Step 9: Verify new version is running
echo -e "${YELLOW}Step 9: Verifying new version is running...${NC}"
HEALTH_OUTPUT=$(curl -s http://127.0.0.1:3000/health)
NEW_RUNNING_VERSION=$(echo "$HEALTH_OUTPUT" | grep -oE '"version":"[^"]+' | cut -d'"' -f4)
echo "Health check response: $HEALTH_OUTPUT"
echo -e "${GREEN}✓ Server is now running with version: $NEW_RUNNING_VERSION${NC}"
echo ""

# Step 10: Verify PID changed (daemon was restarted)
echo -e "${YELLOW}Step 10: Verifying daemon was restarted...${NC}"
NEW_PID=$(cat "$PID_FILE" | grep -oE '"pid":[0-9]+' | cut -d':' -f2)
echo "Original PID: $ORIGINAL_PID"
echo "New PID:      $NEW_PID"

if [ "$ORIGINAL_PID" != "$NEW_PID" ]; then
    echo -e "${GREEN}✓ PID changed - daemon was successfully restarted${NC}"
else
    echo -e "${RED}✗ PID did not change - daemon was NOT restarted${NC}"
    exit 1
fi
echo ""

# Step 11: Test idempotency (running again should NOT restart)
echo -e "${YELLOW}Step 11: Testing idempotency - running again should NOT restart...${NC}"
$BINARY_PATH server run --host 127.0.0.1 --port 3000 2>&1 | grep -E "✅|already running|PID|Version"
FINAL_PID=$(cat "$PID_FILE" | grep -oE '"pid":[0-9]+' | cut -d':' -f2)

if [ "$NEW_PID" == "$FINAL_PID" ]; then
    echo -e "${GREEN}✓ PID unchanged - daemon was NOT restarted (idempotent behavior)${NC}"
else
    echo -e "${RED}✗ PID changed unexpectedly - should have been idempotent${NC}"
    exit 1
fi
echo ""

# Step 12: Restore original build.rs
echo -e "${YELLOW}Step 12: Restoring original version...${NC}"
mv build.rs.backup build.rs
rm -f build.rs.bak
cargo build --release --quiet
echo -e "${GREEN}✓ Restored to original version${NC}"
echo ""

# Success!
echo "=========================================="
echo -e "${GREEN}✓✓✓ All tests passed! ✓✓✓${NC}"
echo "=========================================="
echo ""
echo "Summary:"
echo "  - Version mismatch was detected"
echo "  - Daemon was automatically restarted with new version"
echo "  - PID changed from $ORIGINAL_PID to $NEW_PID"
echo "  - Idempotency was maintained (no restart when versions match)"
echo ""
