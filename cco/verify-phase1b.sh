#!/bin/bash
# Phase 1B Verification Script
#
# This script demonstrates Phase 1B integration by:
# 1. Simulating daemon start (creating settings file)
# 2. Verifying hooks configuration in settings file
# 3. Showing environment variables that would be set

set -e

echo "=== Phase 1B Verification ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Step 1: Checking daemon configuration${NC}"
echo "Config file: ~/.cco/config.toml"
if [ -f ~/.cco/config.toml ]; then
    echo -e "${GREEN}✓${NC} Config file exists"
    echo "Hooks configuration:"
    grep -A 20 "^\[hooks\]" ~/.cco/config.toml 2>/dev/null || echo "  (using defaults)"
else
    echo -e "${YELLOW}!${NC} Config file not found (will use defaults)"
fi
echo ""

echo -e "${BLUE}Step 2: Checking orchestrator settings file${NC}"
SETTINGS_FILE="/tmp/.cco-orchestrator-settings"
echo "Settings file: $SETTINGS_FILE"

if [ -f "$SETTINGS_FILE" ]; then
    echo -e "${GREEN}✓${NC} Settings file exists"
    echo ""

    echo "Hooks section:"
    cat "$SETTINGS_FILE" | jq '.hooks' 2>/dev/null || echo "  (cannot parse JSON)"
    echo ""

    echo "Daemon section:"
    cat "$SETTINGS_FILE" | jq '.daemon' 2>/dev/null || echo "  (cannot parse JSON)"
    echo ""

    echo "Orchestrator section:"
    cat "$SETTINGS_FILE" | jq '.orchestrator' 2>/dev/null || echo "  (cannot parse JSON)"
    echo ""
else
    echo -e "${YELLOW}!${NC} Settings file not found"
    echo "  Run 'cco daemon start' to create it"
fi

echo -e "${BLUE}Step 3: Environment variables that would be set${NC}"
if [ -f "$SETTINGS_FILE" ]; then
    echo "ORCHESTRATOR_ENABLED=true"
    echo "ORCHESTRATOR_SETTINGS=$SETTINGS_FILE"
    echo "ORCHESTRATOR_API_URL=$(cat "$SETTINGS_FILE" | jq -r '.orchestrator.api_url' 2>/dev/null || echo 'http://localhost:3000')"
    echo ""

    echo "Hooks-specific variables:"
    HOOKS_ENABLED=$(cat "$SETTINGS_FILE" | jq -r '.hooks.enabled' 2>/dev/null || echo 'false')
    echo "ORCHESTRATOR_HOOKS_ENABLED=$HOOKS_ENABLED"

    ALLOW_FILE_READ=$(cat "$SETTINGS_FILE" | jq -r '.hooks.permissions.allow_file_read' 2>/dev/null || echo 'false')
    echo "ORCHESTRATOR_AUTO_ALLOW_READ=$ALLOW_FILE_READ"

    echo ""
    echo "Full hooks configuration (JSON):"
    cat "$SETTINGS_FILE" | jq -c '.hooks' 2>/dev/null || echo '  (cannot parse)'
else
    echo "  (settings file not found - start daemon first)"
fi

echo ""
echo -e "${BLUE}Step 4: Verification summary${NC}"

if [ -f "$SETTINGS_FILE" ]; then
    HAS_HOOKS=$(cat "$SETTINGS_FILE" | jq 'has("hooks")' 2>/dev/null || echo 'false')
    HAS_DAEMON=$(cat "$SETTINGS_FILE" | jq 'has("daemon")' 2>/dev/null || echo 'false')
    HAS_ORCHESTRATOR=$(cat "$SETTINGS_FILE" | jq 'has("orchestrator")' 2>/dev/null || echo 'false')

    if [ "$HAS_HOOKS" = "true" ] && [ "$HAS_DAEMON" = "true" ] && [ "$HAS_ORCHESTRATOR" = "true" ]; then
        echo -e "${GREEN}✓${NC} Phase 1B integration complete"
        echo "  - Settings file contains hooks configuration"
        echo "  - Settings file contains daemon information"
        echo "  - Settings file contains orchestrator configuration"
        echo "  - Ready for Claude Code to read via environment variables"
    else
        echo -e "${YELLOW}!${NC} Settings file incomplete"
        echo "  Has hooks: $HAS_HOOKS"
        echo "  Has daemon: $HAS_DAEMON"
        echo "  Has orchestrator: $HAS_ORCHESTRATOR"
    fi
else
    echo -e "${YELLOW}!${NC} Daemon not running"
    echo ""
    echo "To verify Phase 1B integration:"
    echo "  1. Run: cco daemon start"
    echo "  2. Run: $0"
    echo "  3. Check settings file is created with hooks config"
fi

echo ""
echo "=== End of Verification ==="
