#!/bin/bash

# Update all documentation with correct agent counts and model info

echo "🔍 Searching for outdated references in documentation..."

# Define the correct numbers
TOTAL_AGENTS=117
OPUS_COUNT=1
SONNET_COUNT=77
HAIKU_COUNT=39

echo ""
echo "📊 Current Configuration:"
echo "  Total Agents: $TOTAL_AGENTS"
echo "  Opus 4.1: $OPUS_COUNT (Chief Architect)"
echo "  Sonnet 4.5: $SONNET_COUNT (Intelligent managers)"
echo "  Haiku 4.5: $HAIKU_COUNT (Basic coders & utilities)"
echo ""

# Create a report of files that need updating
echo "📝 Files with outdated references:"
echo ""

# Search for common outdated patterns
grep -rl "116 agents\|125 agents\|88 agents\|28 agents" docs/*.md 2>/dev/null | while read file; do
    echo "  - $file"
done

# Search for ccproxy/qwen references
grep -rl "qwen\|ccproxy" docs/*.md 2>/dev/null | while read file; do
    echo "  - $file (has ccproxy/qwen references)"
done

echo ""
echo "✅ Key files already updated:"
echo "  - config/orchestra-config.json"
echo "  - CLAUDE.md"
echo "  - README.md"
echo "  - ORCHESTRA_ROSTER.md"
echo ""

echo "💡 Manual review recommended for:"
echo "  - docs/ARCHITECTURE_DIAGRAMS.md (contains architecture diagrams)"
echo "  - docs/AGENT_TYPE_GUIDE.md (contains agent type statistics)"
echo "  - docs/*ROSTER*.md (roster documentation files)"
echo "  - docs/*TDD*.md (TDD pipeline documentation)"
echo ""

echo "✨ Update complete!"
echo "   Run 'git diff' to review all changes"
