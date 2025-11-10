#!/bin/bash

# Claude Army Setup Script
# This script helps set up the Claude Army development environment

set -e

echo "🤖⚔️  Claude Army Setup"
echo "====================="
echo ""

# Check Node.js version
echo "Checking Node.js version..."
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 16+ and try again."
    exit 1
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 16 ]; then
    echo "❌ Node.js version 16+ is required. Found version: $(node -v)"
    exit 1
fi
echo "✅ Node.js $(node -v) detected"

# Check if MCP servers are available
echo ""
echo "Checking MCP servers..."

if command -v claude &> /dev/null; then
    echo "✅ Claude CLI detected"
else
    echo "⚠️  Claude CLI not found. Install from: https://claude.ai/download"
fi

# Check claude-flow
echo "Checking claude-flow@alpha..."
if npx claude-flow@alpha --version &> /dev/null; then
    echo "✅ claude-flow@alpha available"
else
    echo "⚠️  claude-flow@alpha not available"
    echo "   It will be downloaded when needed via npx"
fi

# Check ruv-swarm
echo "Checking ruv-swarm..."
if npx ruv-swarm --version &> /dev/null; then
    echo "✅ ruv-swarm available"
else
    echo "⚠️  ruv-swarm not available"
    echo "   It will be downloaded when needed via npx"
fi

# Check MCP server configuration
echo ""
echo "Checking MCP server configuration..."
if [ -f ".claude/settings.local.json" ]; then
    echo "✅ Found .claude/settings.local.json"

    # Check which servers are disabled
    DISABLED=$(cat .claude/settings.local.json | grep -A 10 "disabledMcpjsonServers" | grep -E "claude-flow|ruv-swarm" || echo "")

    if [ -z "$DISABLED" ]; then
        echo "✅ claude-flow@alpha and ruv-swarm are ENABLED"
    else
        echo "⚠️  Some MCP servers are disabled:"
        echo "$DISABLED"
        echo ""
        echo "To enable them, edit .claude/settings.local.json and remove from disabledMcpjsonServers array"
    fi
else
    echo "❌ .claude/settings.local.json not found"
    echo "   Create this file to configure MCP servers"
fi

# Create necessary directories
echo ""
echo "Creating directory structure..."
mkdir -p src tests docs config scripts .github/workflows
echo "✅ Directory structure created"

# Check if credential manager is executable
echo ""
echo "Setting up credential manager..."
if [ -f "src/credential-manager.js" ]; then
    chmod +x src/credential-manager.js
    echo "✅ Credential manager ready"
else
    echo "⚠️  src/credential-manager.js not found"
fi

# Check if orchestrator is executable
if [ -f "src/army-orchestrator.js" ]; then
    chmod +x src/army-orchestrator.js
    echo "✅ Army orchestrator ready"
else
    echo "⚠️  src/army-orchestrator.js not found"
fi

# Initialize git if not already initialized
echo ""
if [ -d ".git" ]; then
    echo "✅ Git repository already initialized"
else
    echo "Initializing git repository..."
    git init
    echo "✅ Git repository initialized"
fi

# Check for package.json
echo ""
if [ -f "package.json" ]; then
    echo "✅ package.json found"
else
    echo "⚠️  package.json not found"
fi

# Summary
echo ""
echo "====================="
echo "Setup Summary"
echo "====================="
echo ""
echo "✅ Node.js: $(node -v)"
echo "✅ Directory structure created"
echo "✅ Scripts made executable"
echo ""
echo "Next Steps:"
echo "1. Enable MCP servers in .claude/settings.local.json (remove from disabled list)"
echo "2. Run: npm run army   (view army configuration)"
echo "3. Run: npm run help   (view usage guide)"
echo "4. Start using Claude Code to deploy your army!"
echo ""
echo "Example usage:"
echo '  "Build a REST API with authentication in Python"'
echo ""
echo "For detailed documentation, see:"
echo "  - README.md"
echo "  - docs/ARMY_USAGE_GUIDE.md"
echo "  - docs/EXAMPLE_WORKFLOW.md"
echo ""
echo "🤖⚔️  Claude Army is ready to deploy!"
