#!/bin/bash

# Pre-Compaction Export Script
# This script exports ALL critical state to MCP memory before a compaction event
# Run this BEFORE Claude Code compacts the conversation

set -e  # Exit on error

TIMESTAMP=$(date +%s)
SESSION_ID=${SESSION_ID:-"default"}

echo "🔄 Pre-Compaction Export Starting..."
echo "📅 Timestamp: $TIMESTAMP"
echo "🆔 Session ID: $SESSION_ID"
echo ""

# Export Architect State
echo "📐 Exporting Architect state..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/architect/specification" \
  --value "$(cat /tmp/architect-specification.json 2>/dev/null || echo '{}')" \
  --ttl 86400  # 24 hours

npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/architect/decisions" \
  --value "$(cat /tmp/architect-decisions.json 2>/dev/null || echo '[]')" \
  --ttl 86400

npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/architect/current_phase" \
  --value "${CURRENT_PHASE:-unknown}" \
  --ttl 86400

# Export Definition of Done
echo "✅ Exporting Definition of Done..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/project/definition-of-done" \
  --value "$(cat /tmp/definition-of-done.json 2>/dev/null || echo '{}')" \
  --ttl 86400

# Export Credentials Inventory
echo "🔐 Exporting credentials inventory..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/credentials/inventory" \
  --value "$(cat /tmp/credentials-inventory.json 2>/dev/null || echo '[]')" \
  --ttl 86400

npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/credentials/access-methods" \
  --value "$(cat /tmp/credentials-access.json 2>/dev/null || echo '{}')" \
  --ttl 86400

# Export Integration Configurations
echo "🔌 Exporting integration configurations..."

# Salesforce
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/integrations/salesforce/config" \
  --value "$(cat /tmp/salesforce-config.json 2>/dev/null || echo '{}')" \
  --ttl 86400

# Authentik
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/integrations/authentik/config" \
  --value "$(cat /tmp/authentik-config.json 2>/dev/null || echo '{}')" \
  --ttl 86400

# Other APIs
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/integrations/other-apis" \
  --value "$(cat /tmp/other-apis.json 2>/dev/null || echo '[]')" \
  --ttl 86400

# Export Agent States
echo "🤖 Exporting agent states..."

AGENTS=("python-specialist" "swift-specialist" "go-specialist" "rust-specialist" "flutter-specialist" \
        "api-explorer" "salesforce-specialist" "authentik-specialist" \
        "qa-engineer" "security-auditor" "devops-engineer" "docs-lead" "credential-manager")

for agent in "${AGENTS[@]}"; do
  if [ -f "/tmp/${agent}-state.json" ]; then
    npx claude-flow@alpha memory store \
      --key "compaction/${SESSION_ID}/agents/${agent}/state" \
      --value "$(cat /tmp/${agent}-state.json)" \
      --ttl 86400
  fi
done

# Export File Structure
echo "📁 Exporting file structure..."
if [ -d "$(pwd)" ]; then
  find . -type f -name "*.py" -o -name "*.js" -o -name "*.go" -o -name "*.rs" -o -name "*.swift" -o -name "*.dart" | \
    jq -R -s -c 'split("\n")[:-1]' > /tmp/file-structure.json

  npx claude-flow@alpha memory store \
    --key "compaction/${SESSION_ID}/files/structure" \
    --value "$(cat /tmp/file-structure.json)" \
    --ttl 86400
fi

# Export Critical Files List
echo "📄 Exporting critical files..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/files/critical" \
  --value "$(cat /tmp/critical-files.json 2>/dev/null || echo '[]')" \
  --ttl 86400

# Export Test Results
echo "🧪 Exporting test results..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/testing/results" \
  --value "$(cat /tmp/test-results.json 2>/dev/null || echo '{}')" \
  --ttl 86400

npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/testing/coverage" \
  --value "$(cat /tmp/test-coverage.json 2>/dev/null || echo '{}')" \
  --ttl 86400

# Export Completed Milestones
echo "🎯 Exporting completed milestones..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/milestones/completed" \
  --value "$(cat /tmp/completed-milestones.json 2>/dev/null || echo '[]')" \
  --ttl 86400

# Export Blocked Tasks
echo "🚧 Exporting blocked tasks..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/tasks/blocked" \
  --value "$(cat /tmp/blocked-tasks.json 2>/dev/null || echo '[]')" \
  --ttl 86400

# Store compaction metadata
echo "📋 Storing compaction metadata..."
npx claude-flow@alpha memory store \
  --key "compaction/${SESSION_ID}/metadata" \
  --value "{\"timestamp\": $TIMESTAMP, \"exported_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\", \"session_id\": \"$SESSION_ID\"}" \
  --ttl 86400

# Notify all agents
echo "📢 Notifying agents of compaction..."
npx claude-flow@alpha hooks notify \
  --message "🔄 Pre-compaction export complete. State preserved in memory at compaction/${SESSION_ID}/"

echo ""
echo "✅ Pre-Compaction Export Complete!"
echo "🔑 Session ID: $SESSION_ID"
echo "⏰ Timestamp: $TIMESTAMP"
echo "📦 All critical state exported to MCP memory"
echo ""
echo "To restore after compaction, run:"
echo "  ./scripts/post-compaction.sh $SESSION_ID"
