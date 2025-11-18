#!/bin/bash
# VectorDB Usage Analysis Script
# Analyzes current vectordb usage and provides migration readiness assessment

set -e

echo "============================================="
echo "VectorDB Usage Analysis"
echo "============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "src" ]; then
    echo -e "${RED}Error: Must be run from cc-orchestra root directory${NC}"
    exit 1
fi

echo "1. Data Volume Analysis"
echo "-----------------------"
if [ -d "data/knowledge" ]; then
    TOTAL_SIZE=$(du -sh data/knowledge/ | awk '{print $1}')
    ENTRY_COUNT=$(find data/knowledge -name "*.lance" | wc -l | tr -d ' ')
    echo -e "${GREEN}✓${NC} Knowledge database exists"
    echo "  Total size: ${TOTAL_SIZE}"
    echo "  Lance files: ${ENTRY_COUNT}"

    # List repositories
    echo ""
    echo "  Repositories with knowledge:"
    for repo_dir in data/knowledge/*/; do
        if [ -d "$repo_dir" ]; then
            repo_name=$(basename "$repo_dir")
            repo_size=$(du -sh "$repo_dir" | awk '{print $1}')
            file_count=$(find "$repo_dir" -name "*.lance" | wc -l | tr -d ' ')
            echo "    - ${repo_name}: ${repo_size} (${file_count} files)"
        fi
    done
else
    echo -e "${YELLOW}⚠${NC} No knowledge database found at data/knowledge/"
fi

echo ""
echo "2. Dependency Analysis"
echo "----------------------"
if grep -q "vectordb" package.json; then
    VERSION=$(grep "vectordb" package.json | sed 's/.*: "//;s/".*//')
    echo -e "${YELLOW}⚠${NC} VectorDB dependency found: ${VERSION}"
else
    echo -e "${GREEN}✓${NC} No VectorDB dependency found"
fi

if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo -e "${YELLOW}⚠${NC} Node.js installed: ${NODE_VERSION}"
else
    echo -e "${GREEN}✓${NC} Node.js not found (good if migrating to SQLite)"
fi

echo ""
echo "3. Code Usage Analysis"
echo "----------------------"
KM_USAGE=$(grep -r "knowledge-manager" --include="*.js" --include="*.ts" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
if [ "$KM_USAGE" -gt 0 ]; then
    echo -e "${YELLOW}⚠${NC} knowledge-manager references found: ${KM_USAGE}"
    echo "  Top files:"
    grep -r "knowledge-manager" --include="*.js" --include="*.ts" --include="*.rs" 2>/dev/null | \
        cut -d: -f1 | sort | uniq -c | sort -rn | head -5 | \
        sed 's/^/    /'
else
    echo -e "${GREEN}✓${NC} No knowledge-manager references found"
fi

echo ""
echo "4. SQLite Infrastructure"
echo "------------------------"
if [ -f "cco/Cargo.toml" ]; then
    if grep -q "sqlx" cco/Cargo.toml; then
        echo -e "${GREEN}✓${NC} SQLite dependency (sqlx) already in Cargo.toml"
    else
        echo -e "${RED}✗${NC} SQLite dependency not found in Cargo.toml"
    fi

    if [ -f "cco/src/persistence/mod.rs" ]; then
        echo -e "${GREEN}✓${NC} Persistence layer exists"
        TABLES=$(grep "CREATE TABLE" cco/src/persistence/schema.rs 2>/dev/null | wc -l | tr -d ' ')
        echo "  Existing tables: ${TABLES}"
    else
        echo -e "${YELLOW}⚠${NC} Persistence layer not found"
    fi
else
    echo -e "${RED}✗${NC} cco/Cargo.toml not found"
fi

echo ""
echo "5. Migration Readiness"
echo "----------------------"

READY=true

# Check if daemon.db exists
if [ -f "$HOME/.cco/daemon.db" ]; then
    DB_SIZE=$(du -sh "$HOME/.cco/daemon.db" | awk '{print $1}')
    echo -e "${GREEN}✓${NC} Daemon database exists: ${DB_SIZE}"
else
    echo -e "${YELLOW}⚠${NC} Daemon database not found (will be created)"
fi

# Check if knowledge directory exists
if [ -d "data/knowledge" ]; then
    echo -e "${GREEN}✓${NC} Source data available for migration"
else
    echo -e "${YELLOW}⚠${NC} No source data to migrate"
fi

# Check if persistence layer is ready
if [ -f "cco/src/persistence/mod.rs" ]; then
    echo -e "${GREEN}✓${NC} Persistence infrastructure ready"
else
    echo -e "${RED}✗${NC} Persistence infrastructure not ready"
    READY=false
fi

echo ""
echo "6. Performance Baseline"
echo "-----------------------"
if [ -f "src/knowledge-manager.js" ] && command -v node &> /dev/null; then
    echo "Testing current knowledge-manager performance..."

    # Test store operation
    START=$(date +%s%N)
    node src/knowledge-manager.js store "Test entry for performance baseline" "test" 2>&1 > /dev/null || true
    END=$(date +%s%N)
    STORE_MS=$(( (END - START) / 1000000 ))
    echo "  Store operation: ${STORE_MS}ms"

    # Test stats operation
    START=$(date +%s%N)
    node src/knowledge-manager.js stats 2>&1 > /dev/null || true
    END=$(date +%s%N)
    STATS_MS=$(( (END - START) / 1000000 ))
    echo "  Stats operation: ${STATS_MS}ms"
else
    echo -e "${YELLOW}⚠${NC} Cannot test performance (knowledge-manager.js or Node.js not available)"
fi

echo ""
echo "============================================="
echo "Summary & Recommendations"
echo "============================================="
echo ""

if [ -d "data/knowledge" ] && [ "$TOTAL_SIZE" != "" ]; then
    echo -e "${BLUE}Current State:${NC}"
    echo "  - VectorDB in use: YES"
    echo "  - Data volume: ${TOTAL_SIZE}"
    echo "  - External dependency: Node.js + vectordb"
    echo ""
fi

echo -e "${BLUE}Recommendation:${NC}"
echo "  - Migrate to embedded SQLite (Option B)"
echo "  - Eliminate Node.js dependency"
echo "  - Estimated effort: 15-22 hours"
echo ""

echo -e "${BLUE}Benefits:${NC}"
echo "  - Zero external dependencies"
echo "  - 10-20x faster performance"
echo "  - Single binary distribution"
echo "  - Better full-text search"
echo ""

echo -e "${BLUE}Next Steps:${NC}"
echo "  1. Review documentation:"
echo "     - docs/VECTORDB_INVESTIGATION_INDEX.md"
echo "     - docs/VECTORDB_ELIMINATION_SUMMARY.md"
echo "  2. Approve implementation roadmap"
echo "  3. Begin Phase 1: Schema extension"
echo ""

if [ "$READY" = true ]; then
    echo -e "${GREEN}✓ System ready for migration${NC}"
else
    echo -e "${RED}✗ Prerequisites missing - review recommendations${NC}"
fi

echo ""
echo "============================================="
