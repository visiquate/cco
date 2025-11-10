#!/bin/bash

# Post-Compaction Restoration Script
# This script restores ALL critical state from MCP memory after a compaction event
# Run this IMMEDIATELY AFTER Claude Code compacts the conversation

set -e  # Exit on error

SESSION_ID=${1:-"default"}

echo "🔄 Post-Compaction Restoration Starting..."
echo "🆔 Session ID: $SESSION_ID"
echo ""

# Check if compaction metadata exists
echo "🔍 Checking for compaction data..."
METADATA=$(npx claude-flow@alpha memory retrieve --key "compaction/${SESSION_ID}/metadata" 2>/dev/null || echo "{}")

if [ "$METADATA" == "{}" ]; then
  echo "❌ No compaction data found for session: $SESSION_ID"
  echo "Available sessions:"
  npx claude-flow@alpha memory list --prefix "compaction/" | grep metadata || echo "  (none)"
  exit 1
fi

echo "✅ Found compaction data"
echo "📋 Metadata: $METADATA"
echo ""

# Restore Architect State
echo "📐 Restoring Architect state..."
npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/architect/specification" > /tmp/architect-specification.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/architect/decisions" > /tmp/architect-decisions.json 2>/dev/null || true

CURRENT_PHASE=$(npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/architect/current_phase" 2>/dev/null || echo "unknown")

export CURRENT_PHASE

# Restore Definition of Done
echo "✅ Restoring Definition of Done..."
npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/project/definition-of-done" > /tmp/definition-of-done.json 2>/dev/null || true

# Restore Credentials
echo "🔐 Restoring credentials inventory..."
npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/credentials/inventory" > /tmp/credentials-inventory.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/credentials/access-methods" > /tmp/credentials-access.json 2>/dev/null || true

# Restore Integration Configurations
echo "🔌 Restoring integration configurations..."

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/integrations/salesforce/config" > /tmp/salesforce-config.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/integrations/authentik/config" > /tmp/authentik-config.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/integrations/other-apis" > /tmp/other-apis.json 2>/dev/null || true

# Restore Agent States
echo "🤖 Restoring agent states..."

AGENTS=("python-specialist" "swift-specialist" "go-specialist" "rust-specialist" "flutter-specialist" \
        "api-explorer" "salesforce-specialist" "authentik-specialist" \
        "qa-engineer" "security-auditor" "devops-engineer" "docs-lead" "credential-manager")

for agent in "${AGENTS[@]}"; do
  npx claude-flow@alpha memory retrieve \
    --key "compaction/${SESSION_ID}/agents/${agent}/state" > "/tmp/${agent}-state.json" 2>/dev/null || true
done

# Restore File Structure
echo "📁 Restoring file structure..."
npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/files/structure" > /tmp/file-structure.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/files/critical" > /tmp/critical-files.json 2>/dev/null || true

# Restore Test Results
echo "🧪 Restoring test results..."
npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/testing/results" > /tmp/test-results.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/testing/coverage" > /tmp/test-coverage.json 2>/dev/null || true

# Restore Milestones and Tasks
echo "🎯 Restoring milestones and tasks..."
npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/milestones/completed" > /tmp/completed-milestones.json 2>/dev/null || true

npx claude-flow@alpha memory retrieve \
  --key "compaction/${SESSION_ID}/tasks/blocked" > /tmp/blocked-tasks.json 2>/dev/null || true

# Broadcast restoration to all agents
echo "📢 Broadcasting restoration to all agents..."
npx claude-flow@alpha hooks notify \
  --message "✅ Post-compaction restoration complete. All state restored from session: $SESSION_ID"

# Display restoration summary
echo ""
echo "✅ Post-Compaction Restoration Complete!"
echo ""
echo "📊 Restoration Summary:"
echo "  ├─ Current Phase: $CURRENT_PHASE"
echo "  ├─ Architect specification: $([ -s /tmp/architect-specification.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  ├─ Architecture decisions: $([ -s /tmp/architect-decisions.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  ├─ Definition of Done: $([ -s /tmp/definition-of-done.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  ├─ Credentials: $([ -s /tmp/credentials-inventory.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  ├─ Salesforce config: $([ -s /tmp/salesforce-config.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  ├─ Authentik config: $([ -s /tmp/authentik-config.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  ├─ Test results: $([ -s /tmp/test-results.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo "  └─ File structure: $([ -s /tmp/file-structure.json ] && echo '✅ Restored' || echo '⚠️  Empty')"
echo ""
echo "🎯 Next Steps:"
echo "  1. Review restored state files in /tmp/"
echo "  2. Check current phase: $CURRENT_PHASE"
echo "  3. Review completed milestones: cat /tmp/completed-milestones.json"
echo "  4. Check blocked tasks: cat /tmp/blocked-tasks.json"
echo "  5. Continue from where you left off"
echo ""
echo "💡 All agents have been notified of the restoration."
