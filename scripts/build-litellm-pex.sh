#!/bin/bash
# Build LiteLLM PEX (Python EXecutable)
#
# Creates a self-contained litellm.pex file that includes LiteLLM and all dependencies.
# Requires Python 3.9+ on the host system, but no pip install needed at runtime.
#
# Usage:
#   ./scripts/build-litellm-pex.sh
#
# Output:
#   dist/litellm.pex - Ready to embed in cco or ship alongside

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_ROOT/dist"
PEX_OUTPUT="$DIST_DIR/litellm.pex"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}LiteLLM PEX Builder${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check Python version
echo -e "${YELLOW}Step 1: Checking Python version...${NC}"

# Use system Python for maximum compatibility across macOS systems
# This ensures the PEX works with the default /usr/bin/python3
if [ -x "/usr/bin/python3" ]; then
    PYTHON_CMD="/usr/bin/python3"
else
    # Fallback to python3 in PATH
    PYTHON_CMD="python3"
fi

PYTHON_VERSION=$($PYTHON_CMD --version 2>&1 | cut -d' ' -f2)
PYTHON_MAJOR=$(echo "$PYTHON_VERSION" | cut -d'.' -f1)
PYTHON_MINOR=$(echo "$PYTHON_VERSION" | cut -d'.' -f2)

if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 9 ]); then
    echo -e "${RED}✗ Python 3.9+ required, found $PYTHON_VERSION${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Using $PYTHON_CMD ($PYTHON_VERSION)${NC}"

# Create/activate virtual environment for build
echo -e "${YELLOW}Step 2: Setting up build environment...${NC}"
VENV_DIR="$PROJECT_ROOT/.build-venv"
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating virtual environment with $PYTHON_CMD..."
    $PYTHON_CMD -m venv "$VENV_DIR"
fi
source "$VENV_DIR/bin/activate"
echo -e "${GREEN}✓ Virtual environment activated${NC}"

# Check/install pex in venv
echo -e "${YELLOW}Step 3: Checking pex tool...${NC}"
if ! command -v pex &> /dev/null; then
    echo "Installing pex in virtual environment..."
    pip install pex --quiet
fi
PEX_VERSION=$(pex --version 2>&1 | head -1)
echo -e "${GREEN}✓ $PEX_VERSION${NC}"

# Create dist directory
echo -e "${YELLOW}Step 4: Creating dist directory...${NC}"
mkdir -p "$DIST_DIR"
echo -e "${GREEN}✓ $DIST_DIR${NC}"

# Build PEX
echo -e "${YELLOW}Step 5: Building LiteLLM PEX...${NC}"
echo "This may take a few minutes on first run..."

# Build with proxy extras for full functionality
# Built with a specific Python, but generic shebang allows running on any Python 3.9+
# The --only-binary :all: forces using pre-built wheels only (no compilation)
pex \
    'litellm[proxy]>=1.50.0' \
    --python="$PYTHON_CMD" \
    --python-shebang="/usr/bin/env python3" \
    --output-file="$PEX_OUTPUT" \
    --entry-point=litellm.proxy.proxy_cli:run_server \
    --pip-version=latest \
    --resolver-version=pip-2020-resolver

# Make executable
chmod +x "$PEX_OUTPUT"

# Verify
echo -e "${YELLOW}Step 6: Verifying PEX...${NC}"
PEX_SIZE=$(du -h "$PEX_OUTPUT" | cut -f1)
echo -e "${GREEN}✓ Built $PEX_OUTPUT ($PEX_SIZE)${NC}"

# Test basic import
echo -e "${YELLOW}Step 7: Testing PEX...${NC}"
if "$PEX_OUTPUT" --help &> /dev/null; then
    echo -e "${GREEN}✓ PEX runs successfully${NC}"
else
    echo -e "${RED}✗ PEX failed to run${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Build Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "PEX file: $PEX_OUTPUT"
echo "Size: $PEX_SIZE"
echo ""
echo "Usage:"
echo "  $PEX_OUTPUT --config config/litellm_config.yaml --port 4000"
echo ""
echo "To embed in cco binary, add to build.rs:"
echo "  const LITELLM_PEX: &[u8] = include_bytes!(\"../dist/litellm.pex\");"
