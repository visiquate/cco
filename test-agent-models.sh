#!/bin/bash

# Test Agent Model Configuration
# This script validates that Task calls use the correct configured models

echo ""
echo "🧪 Agent Model Configuration Test Suite"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test function
test_agent_model() {
  local agent_type=$1
  local expected_model=$2

  printf "Testing %-25s... " "$agent_type"

  # Get the configured model (extract just the model part after the colon)
  full_output=$(node src/agent-spawner.js get-model "$agent_type" 2>/dev/null || echo "ERROR")
  actual_model=$(echo "$full_output" | awk -F': ' '{print $2}')

  if [ "$actual_model" = "$expected_model" ]; then
    echo "${GREEN}✅ PASS${NC} (configured: $actual_model)"
    return 0
  else
    echo "${RED}❌ FAIL${NC} (expected: $expected_model, got: $actual_model)"
    return 1
  fi
}

# Run tests
passed=0
failed=0

# Test each agent (use set +e to prevent early exit on failure)
set +e

if test_agent_model "rust-specialist" "haiku"; then ((passed++)); else ((failed++)); fi
if test_agent_model "devops-engineer" "haiku"; then ((passed++)); else ((failed++)); fi
if test_agent_model "frontend-developer" "haiku"; then ((passed++)); else ((failed++)); fi
if test_agent_model "test-engineer" "haiku"; then ((passed++)); else ((failed++)); fi
if test_agent_model "documentation-expert" "haiku"; then ((passed++)); else ((failed++)); fi

set -e

echo ""
echo "📊 Test Results"
echo "==============="
echo "Passed: ${GREEN}$passed${NC}"
echo "Failed: ${RED}$failed${NC}"
echo ""

if [ $failed -eq 0 ]; then
  echo "${GREEN}✅ All agents configured correctly!${NC}"
  echo ""
  echo "📝 Next Steps:"
  echo "1. When calling Task tool, use these models:"
  echo "   - Task(..., \"rust-specialist\", \"haiku\")"
  echo "   - Task(..., \"devops-engineer\", \"haiku\")"
  echo "   - Task(..., \"frontend-developer\", \"haiku\")"
  echo "   - Task(..., \"test-engineer\", \"haiku\")"
  echo "   - Task(..., \"documentation-expert\", \"haiku\")"
  echo ""
  echo "2. Run this test again after making changes to verify"
  echo ""
  exit 0
else
  echo "${RED}❌ Some agents have incorrect models!${NC}"
  echo ""
  echo "📝 Fix Instructions:"
  echo "1. Check MODEL_OVERRIDE_FIX.md for detailed guide"
  echo "2. Update Task calls to use configured models"
  echo "3. Run: bash test-agent-models.sh"
  echo ""
  exit 1
fi
