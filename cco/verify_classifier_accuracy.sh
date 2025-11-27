#!/bin/bash
# CRUD Classifier Accuracy Verification Script
# Tests classifier accuracy after prompt parsing bug fix
# Target: ≥93.75% accuracy

set -e

DAEMON_URL="http://127.0.0.1:3000"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "CRUD Classifier Accuracy Verification"
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# Check daemon is running
echo "Checking daemon status..."
if ! curl -s "${DAEMON_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}ERROR: Daemon not running at ${DAEMON_URL}${NC}"
    echo "Please start daemon: cco daemon start"
    exit 1
fi

# Helper function to test classification
test_command() {
    local command="$1"
    local expected="$2"
    local category="$3"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Make request
    response=$(curl -s -X POST "${DAEMON_URL}/api/classify" \
        -H "Content-Type: application/json" \
        -d "{\"command\":\"$command\"}" 2>&1)

    # Extract classification
    classification=$(echo "$response" | grep -o '"classification":"[^"]*"' | cut -d'"' -f4 2>/dev/null || echo "ERROR")

    # Check result
    if [ "$classification" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $category: $command → $classification"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}✗${NC} $category: $command → $classification (expected $expected)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

echo "Starting classification tests..."
echo ""

# ==========================================
# Section 1: Basic READ Operations (8 tests)
# ==========================================
echo "Section 1: Basic READ Operations"
echo "--------------------------------"

test_command "ls" "Read" "READ"
test_command "cat /etc/passwd" "Read" "READ"
test_command "git status" "Read" "READ"
test_command "grep pattern file.txt" "Read" "READ"
test_command "ps aux" "Read" "READ"
test_command "docker ps" "Read" "READ"
test_command "find . -name '*.txt'" "Read" "READ"
test_command "curl -I https://example.com" "Read" "READ"

echo ""

# ==========================================
# Section 2: CREATE Operations (4 tests)
# ==========================================
echo "Section 2: CREATE Operations"
echo "----------------------------"

test_command "mkdir test" "Create" "CREATE"
test_command "touch newfile" "Create" "CREATE"
test_command "echo data > file.txt" "Create" "CREATE"
test_command "git init" "Create" "CREATE"

echo ""

# ==========================================
# Section 3: UPDATE Operations (4 tests)
# ==========================================
echo "Section 3: UPDATE Operations"
echo "----------------------------"

test_command "echo data >> file.txt" "Update" "UPDATE"
test_command "chmod 755 file" "Update" "UPDATE"
test_command "mv file1 file2" "Update" "UPDATE"
test_command "git commit -m 'test'" "Update" "UPDATE"

echo ""

# ==========================================
# Section 4: DELETE Operations (4 tests)
# ==========================================
echo "Section 4: DELETE Operations"
echo "----------------------------"

test_command "rm file.txt" "Delete" "DELETE"
test_command "rm -rf directory" "Delete" "DELETE"
test_command "docker rm container" "Delete" "DELETE"
test_command "git branch -d feature" "Delete" "DELETE"

echo ""

# ==========================================
# Section 5: SQL Operations (4 tests)
# ==========================================
echo "Section 5: SQL Operations"
echo "-------------------------"

test_command "SELECT * FROM users" "Read" "SQL"
test_command "INSERT INTO users VALUES (1)" "Create" "SQL"
test_command "UPDATE users SET name='test'" "Update" "SQL"
test_command "DELETE FROM users WHERE id=1" "Delete" "SQL"

echo ""

# ==========================================
# Section 6: System Commands (4 tests)
# ==========================================
echo "Section 6: System Commands"
echo "--------------------------"

test_command "apt-get install package" "Create" "SYSTEM"
test_command "systemctl status service" "Read" "SYSTEM"
test_command "chmod 755 script.sh" "Update" "SYSTEM"
test_command "userdel username" "Delete" "SYSTEM"

echo ""

# ==========================================
# Section 7: Edge Cases (4 tests)
# ==========================================
echo "Section 7: Edge Cases"
echo "---------------------"

test_command "cat file | grep pattern | sort > output.txt" "Create" "EDGE"
test_command "mv 'file name.txt' 'new name.txt'" "Update" "EDGE"
test_command "curl -H 'Authorization: Bearer sk_test123'" "Read" "EDGE"
test_command "docker logs app 2>&1 | grep ERROR" "Read" "EDGE"

echo ""

# ==========================================
# Calculate Statistics
# ==========================================
echo "=========================================="
echo "Test Results Summary"
echo "=========================================="
echo ""

ACCURACY=$(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")

echo "Total Tests:   $TOTAL_TESTS"
echo "Passed:        $PASSED_TESTS"
echo "Failed:        $FAILED_TESTS"
echo "Accuracy:      ${ACCURACY}%"
echo ""

# Check if meets target
TARGET_ACCURACY=93.75

if (( $(echo "$ACCURACY >= $TARGET_ACCURACY" | bc -l) )); then
    echo -e "${GREEN}✓ PASSED${NC} - Accuracy ${ACCURACY}% meets target ≥${TARGET_ACCURACY}%"
    echo ""
    echo "Production Ready: YES"
    exit 0
else
    echo -e "${RED}✗ FAILED${NC} - Accuracy ${ACCURACY}% below target ${TARGET_ACCURACY}%"
    echo ""
    echo "Production Ready: NO"
    echo "Required Improvement: $(awk "BEGIN {printf \"%.2f\", $TARGET_ACCURACY - $ACCURACY}")%"
    exit 1
fi
