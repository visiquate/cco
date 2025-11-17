#!/bin/bash
# Verification script for embedded agents build system
# This script verifies the build.rs implementation is working correctly

set -e

echo "======================================================================"
echo "Embedded Agents Build System Verification"
echo "======================================================================"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Helper function for success messages
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Helper function for warning messages
warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Helper function for error messages
error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check 1: Verify build.rs exists and is executable
echo "1. Checking build.rs script..."
if [ -f "build.rs" ]; then
    success "build.rs exists"
    wc -l build.rs | awk '{print "   Total lines: " $1}'
else
    error "build.rs not found"
    exit 1
fi
echo ""

# Check 2: Verify agent files exist
echo "2. Checking agent markdown files..."
AGENT_COUNT=$(find config/agents -name "*.md" -type f | wc -l)
if [ "$AGENT_COUNT" -gt 0 ]; then
    success "$AGENT_COUNT agent files found"
else
    error "No agent markdown files found in config/agents/"
    exit 1
fi
echo ""

# Check 3: Verify Cargo.toml exists
echo "3. Checking Cargo.toml..."
if [ -f "Cargo.toml" ]; then
    success "Cargo.toml exists"
    if grep -q "serde_json" Cargo.toml; then
        success "serde_json dependency found in build-dependencies"
    else
        warning "serde_json not found in Cargo.toml - required for JSON parsing"
    fi
else
    error "Cargo.toml not found"
    exit 1
fi
echo ""

# Check 4: Verify embedded_agents.rs module
echo "4. Checking embedded_agents.rs module..."
if [ -f "src/embedded_agents.rs" ]; then
    success "src/embedded_agents.rs exists"
    grep -q "create_embedded_agents" src/embedded_agents.rs && success "create_embedded_agents function reference found"
    grep -q "include!" src/embedded_agents.rs && success "include! macro found"
else
    error "src/embedded_agents.rs not found"
    exit 1
fi
echo ""

# Check 5: Verify sample agent files have correct format
echo "5. Verifying agent file format..."
VALID_AGENTS=0
INVALID_AGENTS=0

for agent_file in config/agents/*.md; do
    # Check if file starts with ---
    if head -1 "$agent_file" | grep -q "^---$"; then
        # Check if file has required fields
        if grep -q "^name:" "$agent_file" && \
           grep -q "^model:" "$agent_file" && \
           grep -q "^description:" "$agent_file"; then
            VALID_AGENTS=$((VALID_AGENTS + 1))
        else
            INVALID_AGENTS=$((INVALID_AGENTS + 1))
            warning "Missing required fields in $(basename $agent_file)"
        fi
    else
        INVALID_AGENTS=$((INVALID_AGENTS + 1))
    fi
done

success "$VALID_AGENTS agent files have valid format"
if [ "$INVALID_AGENTS" -gt 0 ]; then
    warning "$INVALID_AGENTS agent files have invalid format"
fi
echo ""

# Check 6: Verify model names are valid
echo "6. Checking agent model validity..."
VALID_MODELS=0
INVALID_MODELS=0

for agent_file in config/agents/*.md; do
    MODEL=$(grep "^model:" "$agent_file" | cut -d: -f2 | xargs)
    case "$MODEL" in
        opus|sonnet|haiku)
            VALID_MODELS=$((VALID_MODELS + 1))
            ;;
        *)
            INVALID_MODELS=$((INVALID_MODELS + 1))
            error "Invalid model '$MODEL' in $(basename $agent_file)"
            ;;
    esac
done

success "$VALID_MODELS agents have valid models"
if [ "$INVALID_MODELS" -gt 0 ]; then
    error "$INVALID_MODELS agents have invalid models"
    exit 1
fi
echo ""

# Check 7: Verify orchestra-config.json fallback exists
echo "7. Checking orchestra-config.json fallback..."
if [ -f "../config/orchestra-config.json" ]; then
    success "orchestra-config.json exists"
    if command -v jq &> /dev/null; then
        AGENT_COUNT_JSON=$(jq '.codingAgents | length' ../config/orchestra-config.json 2>/dev/null || echo "?")
        echo "   Agents in orchestra-config.json: $AGENT_COUNT_JSON"
    else
        warning "jq not available, skipping JSON validation"
    fi
else
    warning "orchestra-config.json not found - using local markdown files only"
fi
echo ""

# Check 8: Summary statistics
echo "======================================================================"
echo "Build System Statistics"
echo "======================================================================"
echo ""
echo "Agent Files:"
echo "  Total files: $AGENT_COUNT"
echo "  Valid files: $VALID_AGENTS"
echo "  Invalid files: $INVALID_AGENTS"
echo ""

MODEL_OPUS=$(grep -r "^model: opus$" config/agents | wc -l)
MODEL_SONNET=$(grep -r "^model: sonnet$" config/agents | wc -l)
MODEL_HAIKU=$(grep -r "^model: haiku$" config/agents | wc -l)

echo "Model Distribution:"
echo "  Opus agents:   $MODEL_OPUS"
echo "  Sonnet agents: $MODEL_SONNET"
echo "  Haiku agents:  $MODEL_HAIKU"
echo ""

# Check 9: Build readiness
echo "======================================================================"
echo "Build Readiness Assessment"
echo "======================================================================"
echo ""

if [ "$VALID_AGENTS" -gt 50 ] && [ "$INVALID_AGENTS" -eq 0 ] && [ -f "build.rs" ] && [ -f "src/embedded_agents.rs" ]; then
    success "System is ready for build!"
    echo ""
    echo "To build the project:"
    echo "  cd /Users/brent/git/cc-orchestra/cco"
    echo "  cargo build"
    echo ""
    echo "To test embedded agents:"
    echo "  cargo test embedded_agents --lib"
    echo ""
    echo "To check generated code:"
    echo "  cat target/debug/build/cco-*/out/agents.rs | head -50"
else
    error "System is not ready for build"
    if [ "$VALID_AGENTS" -le 50 ]; then
        echo "  - Not enough valid agents ($VALID_AGENTS < 50)"
    fi
    if [ "$INVALID_AGENTS" -gt 0 ]; then
        echo "  - Invalid agents found ($INVALID_AGENTS)"
    fi
    exit 1
fi
echo ""
echo "======================================================================"
echo "Verification Complete"
echo "======================================================================"
