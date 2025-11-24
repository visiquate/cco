#!/bin/bash

# Comprehensive Classifier Integration Test Suite
# Tests all CRUD operations with accuracy tracking

API_URL="http://127.0.0.1:3000/api/classify"
RESULTS_FILE="/tmp/classifier_test_results.json"
REPORT_FILE="/tmp/classifier_test_report.txt"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Initialize results
echo "{\"tests\": [], \"summary\": {}}" > "$RESULTS_FILE"

# Test counter
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Category counters
READ_TOTAL=0
READ_PASSED=0
CREATE_TOTAL=0
CREATE_PASSED=0
UPDATE_TOTAL=0
UPDATE_PASSED=0
DELETE_TOTAL=0
DELETE_PASSED=0

# Function to test a command
test_command() {
    local command="$1"
    local expected="$2"
    local category="$3"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -e "\n${YELLOW}Testing:${NC} $command"
    echo -e "${YELLOW}Expected:${NC} $expected"

    # Make API request
    START_TIME=$(date +%s%N)
    RESPONSE=$(curl -s -X POST "$API_URL" \
        -H "Content-Type: application/json" \
        -d "{\"command\": \"$command\"}")
    END_TIME=$(date +%s%N)

    LATENCY_MS=$(( (END_TIME - START_TIME) / 1000000 ))

    # Parse response
    CLASSIFICATION=$(echo "$RESPONSE" | jq -r '.classification // empty')
    CONFIDENCE=$(echo "$RESPONSE" | jq -r '.confidence // 0')
    REASONING=$(echo "$RESPONSE" | jq -r '.reasoning // ""')

    # Check if response is valid
    if [ -z "$CLASSIFICATION" ]; then
        echo -e "${RED}FAIL:${NC} No classification returned"
        echo -e "Response: $RESPONSE"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return
    fi

    # Check if classification matches expected
    if [ "$CLASSIFICATION" = "$expected" ]; then
        echo -e "${GREEN}PASS:${NC} $CLASSIFICATION (confidence: $CONFIDENCE, latency: ${LATENCY_MS}ms)"
        PASSED_TESTS=$((PASSED_TESTS + 1))

        # Update category counters
        case "$category" in
            READ)
                READ_PASSED=$((READ_PASSED + 1))
                ;;
            CREATE)
                CREATE_PASSED=$((CREATE_PASSED + 1))
                ;;
            UPDATE)
                UPDATE_PASSED=$((UPDATE_PASSED + 1))
                ;;
            DELETE)
                DELETE_PASSED=$((DELETE_PASSED + 1))
                ;;
        esac
    else
        echo -e "${RED}FAIL:${NC} Got $CLASSIFICATION, expected $expected"
        echo -e "Confidence: $CONFIDENCE"
        echo -e "Reasoning: $REASONING"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi

    # Update category totals
    case "$category" in
        READ)
            READ_TOTAL=$((READ_TOTAL + 1))
            ;;
        CREATE)
            CREATE_TOTAL=$((CREATE_TOTAL + 1))
            ;;
        UPDATE)
            UPDATE_TOTAL=$((UPDATE_TOTAL + 1))
            ;;
        DELETE)
            DELETE_TOTAL=$((DELETE_TOTAL + 1))
            ;;
    esac

    # Store result
    RESULT_JSON=$(cat <<EOF
{
  "command": "$command",
  "expected": "$expected",
  "actual": "$CLASSIFICATION",
  "confidence": $CONFIDENCE,
  "latency_ms": $LATENCY_MS,
  "reasoning": "$REASONING",
  "passed": $([ "$CLASSIFICATION" = "$expected" ] && echo "true" || echo "false")
}
EOF
)

    # Append to results file
    jq ".tests += [$RESULT_JSON]" "$RESULTS_FILE" > "${RESULTS_FILE}.tmp"
    mv "${RESULTS_FILE}.tmp" "$RESULTS_FILE"
}

echo "=========================================="
echo "CLASSIFIER INTEGRATION TEST SUITE"
echo "=========================================="
echo "API: $API_URL"
echo "Results: $RESULTS_FILE"
echo "Report: $REPORT_FILE"
echo "=========================================="

# ============================================
# READ OPERATIONS (Should ALL auto-allow)
# ============================================
echo -e "\n${YELLOW}========== READ OPERATIONS ==========${NC}"

test_command "ls -la" "READ" "READ"
test_command "cat file.txt" "READ" "READ"
test_command "grep pattern file.txt" "READ" "READ"
test_command "git status" "READ" "READ"
test_command "git log --oneline" "READ" "READ"
test_command "git diff" "READ" "READ"
test_command "curl https://api.example.com" "READ" "READ"
test_command "curl -I https://api.example.com" "READ" "READ"
test_command "docker ps" "READ" "READ"
test_command "docker ps -a" "READ" "READ"
test_command "ps aux" "READ" "READ"
test_command "pwd" "READ" "READ"
test_command "head -20 file.txt" "READ" "READ"
test_command "tail -50 logs.txt" "READ" "READ"
test_command "find . -name '*.py'" "READ" "READ"
test_command "rg 'function'" "READ" "READ"

# ============================================
# CREATE OPERATIONS (Require permission)
# ============================================
echo -e "\n${YELLOW}========== CREATE OPERATIONS ==========${NC}"

test_command "mkdir new_dir" "CREATE" "CREATE"
test_command "touch new_file.txt" "CREATE" "CREATE"
test_command "npm install package" "CREATE" "CREATE"
test_command "git checkout -b feature/test" "CREATE" "CREATE"
test_command "docker build -t myapp ." "CREATE" "CREATE"
test_command "git init" "CREATE" "CREATE"
test_command "go get github.com/user/package" "CREATE" "CREATE"
test_command "pip install requests" "CREATE" "CREATE"
test_command "cargo new myproject" "CREATE" "CREATE"
test_command "mkdir -p path/to/directory" "CREATE" "CREATE"
test_command "docker run -d myapp" "CREATE" "CREATE"

# ============================================
# UPDATE OPERATIONS (Require permission)
# ============================================
echo -e "\n${YELLOW}========== UPDATE OPERATIONS ==========${NC}"

test_command "git commit -m 'test commit'" "UPDATE" "UPDATE"
test_command "npm update" "UPDATE" "UPDATE"
test_command "sed -i 's/old/new/' file.txt" "UPDATE" "UPDATE"
test_command "echo 'new line' >> file.txt" "UPDATE" "UPDATE"
test_command "chmod +x script.sh" "UPDATE" "UPDATE"
test_command "git rebase main" "UPDATE" "UPDATE"
test_command "git merge feature-branch" "UPDATE" "UPDATE"
test_command "git push origin main" "UPDATE" "UPDATE"
test_command "cargo update" "UPDATE" "UPDATE"

# ============================================
# DELETE OPERATIONS (Require permission)
# ============================================
echo -e "\n${YELLOW}========== DELETE OPERATIONS ==========${NC}"

test_command "rm file.txt" "DELETE" "DELETE"
test_command "rm -rf directory/" "DELETE" "DELETE"
test_command "git branch -d feature" "DELETE" "DELETE"
test_command "docker rm container_id" "DELETE" "DELETE"
test_command "pip uninstall package" "DELETE" "DELETE"
test_command "git clean -fd" "DELETE" "DELETE"
test_command "npm uninstall package" "DELETE" "DELETE"
test_command "rmdir empty_dir" "DELETE" "DELETE"
test_command "docker rmi image_id" "DELETE" "DELETE"

# ============================================
# EDGE CASES
# ============================================
echo -e "\n${YELLOW}========== EDGE CASES ==========${NC}"

test_command "cat file.txt | grep pattern | sort" "READ" "READ"
test_command "command > output.txt" "CREATE" "CREATE"
test_command "command >> output.txt" "UPDATE" "UPDATE"
test_command "mkdir test && cd test && git init" "CREATE" "CREATE"
test_command "ls | tee output.txt" "CREATE" "CREATE"

# ============================================
# GENERATE REPORT
# ============================================
echo -e "\n${YELLOW}=========================================="
echo "GENERATING TEST REPORT"
echo -e "==========================================${NC}\n"

# Calculate percentages
OVERALL_ACCURACY=$(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")
READ_ACCURACY=$(awk "BEGIN {printf \"%.2f\", ($READ_PASSED / $READ_TOTAL) * 100}")
CREATE_ACCURACY=$(awk "BEGIN {printf \"%.2f\", ($CREATE_PASSED / $CREATE_TOTAL) * 100}")
UPDATE_ACCURACY=$(awk "BEGIN {printf \"%.2f\", ($UPDATE_PASSED / $UPDATE_TOTAL) * 100}")
DELETE_ACCURACY=$(awk "BEGIN {printf \"%.2f\", ($DELETE_PASSED / $DELETE_TOTAL) * 100}")

# Generate report
cat > "$REPORT_FILE" <<EOF
========================================
CLASSIFIER INTEGRATION TEST REPORT
========================================
Date: $(date)
API: $API_URL
Total Tests: $TOTAL_TESTS

========================================
OVERALL RESULTS
========================================
Passed: $PASSED_TESTS
Failed: $FAILED_TESTS
Accuracy: ${OVERALL_ACCURACY}%

========================================
CATEGORY BREAKDOWN
========================================

READ Operations:
  Total: $READ_TOTAL
  Passed: $READ_PASSED
  Failed: $((READ_TOTAL - READ_PASSED))
  Accuracy: ${READ_ACCURACY}%
  Target: 100%
  Status: $(awk "BEGIN {if ($READ_ACCURACY >= 100) print \"PASS\"; else print \"FAIL\"}")

CREATE Operations:
  Total: $CREATE_TOTAL
  Passed: $CREATE_PASSED
  Failed: $((CREATE_TOTAL - CREATE_PASSED))
  Accuracy: ${CREATE_ACCURACY}%
  Target: 90%
  Status: $(awk "BEGIN {if ($CREATE_ACCURACY >= 90) print \"PASS\"; else print \"FAIL\"}")

UPDATE Operations:
  Total: $UPDATE_TOTAL
  Passed: $UPDATE_PASSED
  Failed: $((UPDATE_TOTAL - UPDATE_PASSED))
  Accuracy: ${UPDATE_ACCURACY}%
  Target: 85%
  Status: $(awk "BEGIN {if ($UPDATE_ACCURACY >= 85) print \"PASS\"; else print \"FAIL\"}")

DELETE Operations:
  Total: $DELETE_TOTAL
  Passed: $DELETE_PASSED
  Failed: $((DELETE_TOTAL - DELETE_PASSED))
  Accuracy: ${DELETE_ACCURACY}%
  Target: 90%
  Status: $(awk "BEGIN {if ($DELETE_ACCURACY >= 90) print \"PASS\"; else print \"FAIL\"}")

========================================
OVERALL STATUS
========================================
Overall Accuracy: ${OVERALL_ACCURACY}%
Required: 88%
Status: $(awk "BEGIN {if ($OVERALL_ACCURACY >= 88) print \"PASS\"; else print \"FAIL\"}")

========================================
DEFINITION OF DONE
========================================
✓ All test cases executed: $TOTAL_TESTS tests
$(awk "BEGIN {if ($OVERALL_ACCURACY >= 88) print \"✓\"; else print \"✗\"}")  Accuracy ≥ 88%: ${OVERALL_ACCURACY}%
$(awk "BEGIN {if ($READ_ACCURACY >= 100) print \"✓\"; else print \"✗\"}")  READ accuracy: ${READ_ACCURACY}%
$(awk "BEGIN {if ($CREATE_ACCURACY >= 90) print \"✓\"; else print \"✗\"}")  CREATE accuracy: ${CREATE_ACCURACY}%
$(awk "BEGIN {if ($UPDATE_ACCURACY >= 85) print \"✓\"; else print \"✗\"}")  UPDATE accuracy: ${UPDATE_ACCURACY}%
$(awk "BEGIN {if ($DELETE_ACCURACY >= 90) print \"✓\"; else print \"✗\"}")  DELETE accuracy: ${DELETE_ACCURACY}%

========================================
RECOMMENDATIONS
========================================
EOF

# Add recommendations based on results
if awk "BEGIN {exit !($OVERALL_ACCURACY < 88)}"; then
    echo "❌ CRITICAL: Overall accuracy below 88% threshold" >> "$REPORT_FILE"
    echo "   - Review failed test cases" >> "$REPORT_FILE"
    echo "   - Retrain classifier model" >> "$REPORT_FILE"
    echo "   - Add more training data" >> "$REPORT_FILE"
else
    echo "✓ Overall accuracy meets requirement" >> "$REPORT_FILE"
fi

if awk "BEGIN {exit !($READ_ACCURACY < 100)}"; then
    echo "⚠ WARNING: READ operations not at 100% accuracy" >> "$REPORT_FILE"
    echo "   - All READ operations should auto-allow" >> "$REPORT_FILE"
fi

if awk "BEGIN {exit !($CREATE_ACCURACY < 90)}"; then
    echo "⚠ WARNING: CREATE operations below 90% target" >> "$REPORT_FILE"
fi

if awk "BEGIN {exit !($UPDATE_ACCURACY < 85)}"; then
    echo "⚠ WARNING: UPDATE operations below 85% target" >> "$REPORT_FILE"
fi

if awk "BEGIN {exit !($DELETE_ACCURACY < 90)}"; then
    echo "⚠ WARNING: DELETE operations below 90% target" >> "$REPORT_FILE"
fi

# Display report
cat "$REPORT_FILE"

echo ""
echo "Full results saved to: $RESULTS_FILE"
echo "Report saved to: $REPORT_FILE"
