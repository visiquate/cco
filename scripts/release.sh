#!/bin/bash
# CCO Release Script
# Automatically creates a release tag using today's date in YYYY.MM.D format
#
# Usage: ./scripts/release.sh [message]
#   message: Optional release message (default: auto-generated from recent commits)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get current version using sequential counter
YEAR=$(date +%Y)
MONTH=$(date +%-m)

# Find highest existing counter for this month
MAX_COUNTER=$(git tag -l "v${YEAR}.${MONTH}.*" | \
    sed "s/v${YEAR}\.${MONTH}\.//" | \
    sort -n | \
    tail -1)

# If no existing tags, start at 1; otherwise increment
if [ -z "$MAX_COUNTER" ]; then
    COUNTER=1
else
    COUNTER=$((MAX_COUNTER + 1))
fi

VERSION="v${YEAR}.${MONTH}.${COUNTER}"

echo -e "${BLUE}📦 Creating release ${VERSION} (release #${COUNTER} in ${YEAR}-${MONTH})${NC}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}CCO Release Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${YELLOW}Today's date: $(date +%Y-%m-%d)${NC}"
echo -e "${YELLOW}Version tag:  ${VERSION}${NC}"
echo ""

# Validate that tag doesn't already exist
if git rev-parse "$VERSION" >/dev/null 2>&1; then
    echo -e "${RED}❌ Error: Tag $VERSION already exists!${NC}"
    echo "   Existing tags for this month:"
    git tag -l "v${YEAR}.${MONTH}.*" | sort -V
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must run from cc-orchestra root directory${NC}"
    exit 1
fi

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo -e "${RED}Error: Uncommitted changes detected${NC}"
    echo "Please commit or stash changes before releasing."
    git status --short
    exit 1
fi

# Generate release message
if [ -n "${1:-}" ]; then
    RELEASE_MSG="$1"
else
    echo -e "${BLUE}Generating release notes from recent commits...${NC}"
    # Get commits since last tag
    LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [ -n "$LAST_TAG" ]; then
        COMMITS=$(git log "${LAST_TAG}..HEAD" --oneline --no-merges | head -20)
    else
        COMMITS=$(git log --oneline --no-merges -20)
    fi

    RELEASE_MSG="Release ${VERSION}

Changes:
${COMMITS}"
fi

echo ""
echo -e "${BLUE}Release message:${NC}"
echo "---"
echo "$RELEASE_MSG"
echo "---"
echo ""

read -p "Create release ${VERSION}? (y/N) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}Aborted${NC}"
    exit 1
fi

# Ensure we're up to date
echo -e "${YELLOW}Pushing any unpushed commits...${NC}"
git push origin main

# Create and push tag
echo -e "${YELLOW}Creating tag ${VERSION}...${NC}"
git tag -a "${VERSION}" -m "${RELEASE_MSG}"

echo -e "${YELLOW}Pushing tag to origin...${NC}"
git push origin "${VERSION}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✓ Release ${VERSION} created successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "View release: ${BLUE}https://github.com/langstons/cco/releases/tag/${VERSION}${NC}"
